use automapper_generator::conditions::claude_generator::ClaudeConditionGenerator;
use automapper_generator::conditions::condition_types::*;
use automapper_generator::conditions::prompt::*;

/// Test that the ClaudeConditionGenerator can parse a realistic canned response.
#[test]
fn test_parse_mock_claude_response() {
    // Simulate the response that the mock_claude.sh script would return
    let mock_response = include_str!("fixtures/mock_claude_response.json");

    let generator = ClaudeConditionGenerator::new(4);
    let conditions = generator.parse_response(mock_response).unwrap();

    assert_eq!(conditions.len(), 3);

    // Condition 1: external
    assert_eq!(conditions[0].condition_number, 1);
    assert!(conditions[0].is_external);
    assert_eq!(conditions[0].confidence, ConfidenceLevel::High);
    assert!(conditions[0].rust_code.is_none());

    // Condition 2: high confidence with implementation
    assert_eq!(conditions[1].condition_number, 2);
    assert!(!conditions[1].is_external);
    assert_eq!(conditions[1].confidence, ConfidenceLevel::High);
    assert!(conditions[1].rust_code.is_some());

    // Condition 3: medium confidence
    assert_eq!(conditions[2].condition_number, 3);
    assert_eq!(conditions[2].confidence, ConfidenceLevel::Medium);
}

/// Test the full pipeline: build prompt -> parse response -> generate output file.
#[test]
fn test_end_to_end_condition_generation_pipeline() {
    let mock_response = include_str!("fixtures/mock_claude_response.json");

    // Step 1: Build prompt
    let conditions = vec![
        ConditionInput {
            id: "1".to_string(),
            description: "Wenn Aufteilung vorhanden".to_string(),
            referencing_fields: None,
        },
        ConditionInput {
            id: "2".to_string(),
            description: "Wenn Marktlokation vorhanden".to_string(),
            referencing_fields: Some(vec!["SG8/SEQ (Muss [2])".to_string()]),
        },
        ConditionInput {
            id: "3".to_string(),
            description: "Wenn Zeitraum gueltig".to_string(),
            referencing_fields: None,
        },
    ];

    let context = ConditionContext {
        message_type: "UTILMD",
        format_version: "FV2510",
        mig_schema: None,
        example_implementations: default_example_implementations(),
    };

    let prompt = build_user_prompt(&conditions, &context);
    assert!(!prompt.is_empty());

    // Step 2: Parse response
    let generator = ClaudeConditionGenerator::new(4);
    let generated = generator.parse_response(mock_response).unwrap();

    // Enrich with original descriptions
    let enriched: Vec<GeneratedCondition> = generated
        .into_iter()
        .map(|mut gc| {
            if let Some(input) = conditions
                .iter()
                .find(|c| c.id == gc.condition_number.to_string())
            {
                gc.original_description = Some(input.description.clone());
                gc.referencing_fields = input.referencing_fields.clone();
            }
            gc
        })
        .collect();

    // Step 3: Generate output file
    let output = automapper_generator::conditions::codegen::generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &enriched,
        "test_ahb.xml",
        &std::collections::HashMap::new(),
    );

    // Verify the output contains expected elements
    assert!(output.contains("UtilmdConditionEvaluatorFV2510"));
    assert!(output.contains("evaluate_1"));
    assert!(output.contains("evaluate_2"));
    assert!(output.contains("evaluate_3"));
    assert!(output.contains("message_splitting"));
    assert!(output.contains("Wenn Aufteilung vorhanden"));
    assert!(output.contains("ConditionEvaluator"));
}
