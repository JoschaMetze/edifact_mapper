use automapper_generator::conditions::claude_generator::ClaudeConditionGenerator;
use automapper_generator::conditions::condition_types::ConditionInput;
use automapper_generator::conditions::prompt::*;

#[test]
fn test_system_prompt_contains_key_instructions() {
    let prompt = build_system_prompt();

    assert!(
        prompt.contains("ConditionEvaluator"),
        "should mention the trait"
    );
    assert!(
        prompt.contains("EvaluationContext"),
        "should mention the context"
    );
    assert!(
        prompt.contains("ConditionResult"),
        "should mention the result type"
    );
    assert!(
        prompt.contains("is_external"),
        "should explain external conditions"
    );
    assert!(prompt.contains("JSON"), "should request JSON output");
}

#[test]
fn test_user_prompt_includes_conditions() {
    let conditions = vec![
        ConditionInput {
            id: "1".to_string(),
            description: "Wenn Aufteilung vorhanden".to_string(),
            referencing_fields: Some(vec!["SG8/SEQ (Muss [1])".to_string()]),
        },
        ConditionInput {
            id: "2".to_string(),
            description: "Wenn Netznutzung vorhanden".to_string(),
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

    assert!(prompt.contains("UTILMD"), "should include message type");
    assert!(prompt.contains("FV2510"), "should include format version");
    assert!(
        prompt.contains("[1]: Wenn Aufteilung vorhanden"),
        "should include condition 1"
    );
    assert!(
        prompt.contains("[2]: Wenn Netznutzung vorhanden"),
        "should include condition 2"
    );
    assert!(
        prompt.contains("SG8/SEQ (Muss [1])"),
        "should include referencing fields"
    );
    assert!(
        prompt.contains("Example"),
        "should include examples section"
    );
}

#[test]
fn test_default_examples_exist() {
    let examples = default_example_implementations();
    assert!(examples.len() >= 3, "should have at least 3 examples");
    assert!(
        examples[0].contains("evaluate_"),
        "examples should contain function signatures"
    );
}

#[test]
fn test_parse_valid_json_response() {
    let generator = ClaudeConditionGenerator::new(4);

    let json = r#"{
        "conditions": [
            {
                "id": "42",
                "implementation": "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}",
                "confidence": "high",
                "reasoning": "Simple field existence check",
                "is_external": false
            }
        ]
    }"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].condition_number, 42);
    assert_eq!(
        conditions[0].confidence,
        automapper_generator::conditions::condition_types::ConfidenceLevel::High
    );
    assert!(!conditions[0].is_external);
    assert!(conditions[0].rust_code.is_some());
}

#[test]
fn test_parse_response_with_markdown_wrapper() {
    let generator = ClaudeConditionGenerator::new(4);

    let json = r#"```json
{
    "conditions": [
        {
            "id": "8",
            "implementation": null,
            "confidence": "high",
            "reasoning": "External condition",
            "is_external": true,
            "external_name": "DataClearingRequired"
        }
    ]
}
```"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 1);
    assert!(conditions[0].is_external);
    assert!(conditions[0].rust_code.is_none());
    assert_eq!(
        conditions[0].external_name.as_deref(),
        Some("DataClearingRequired")
    );
}

#[test]
fn test_parse_response_invalid_json() {
    let generator = ClaudeConditionGenerator::new(4);

    let result = generator.parse_response("not json at all");
    assert!(result.is_err());
}

#[test]
fn test_parse_response_mixed_confidence() {
    let generator = ClaudeConditionGenerator::new(4);

    let json = r#"{
        "conditions": [
            {
                "id": "1",
                "implementation": "ConditionResult::True",
                "confidence": "high",
                "reasoning": "Simple",
                "is_external": false
            },
            {
                "id": "2",
                "implementation": "ConditionResult::Unknown",
                "confidence": "medium",
                "reasoning": "Needs review",
                "is_external": false
            },
            {
                "id": "3",
                "implementation": null,
                "confidence": "low",
                "reasoning": "Too complex",
                "is_external": false
            }
        ]
    }"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 3);

    use automapper_generator::conditions::condition_types::ConfidenceLevel;
    assert_eq!(conditions[0].confidence, ConfidenceLevel::High);
    assert_eq!(conditions[1].confidence, ConfidenceLevel::Medium);
    assert_eq!(conditions[2].confidence, ConfidenceLevel::Low);
    assert!(conditions[2].rust_code.is_none());
}
