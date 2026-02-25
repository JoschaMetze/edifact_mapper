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
fn test_user_prompt_resolves_ahb_notation() {
    let conditions = vec![
        ConditionInput {
            id: "23".to_string(),
            description: "Wenn in dieser SG4 das STS+E01++A05/A99 (Status der Antwort) vorhanden"
                .to_string(),
            referencing_fields: None,
        },
        ConditionInput {
            id: "7".to_string(),
            description: "Wenn SG4 STS+7++ZG9/ZH1/ZH2 (Transaktionsgrund) vorhanden".to_string(),
            referencing_fields: None,
        },
        ConditionInput {
            id: "12".to_string(),
            description: "Wenn SG4 DTM+471 (Ende zum nächstmöglichem Termin) nicht vorhanden"
                .to_string(),
            referencing_fields: None,
        },
    ];
    let context = ConditionContext {
        message_type: "UTILMD",
        format_version: "FV2504",
        mig_schema: None,
        example_implementations: vec![],
    };
    let prompt = build_user_prompt(&conditions, &context);

    // STS+E01++A05/A99: elements[0]=E01, elements[1]=(empty), elements[2]=A05/A99
    assert!(
        prompt.contains("elements[0]=E01"),
        "should resolve first element of STS+E01++A05/A99"
    );
    assert!(
        prompt.contains("elements[1]=(empty)"),
        "should mark the empty element between ++ in STS+E01++A05/A99"
    );
    assert!(
        prompt.contains("elements[2]=A05/A99"),
        "should resolve A05/A99 at elements[2]"
    );

    // STS+7++ZG9/ZH1/ZH2: elements[0]=7, elements[1]=(empty), elements[2]=ZG9/ZH1/ZH2
    assert!(
        prompt.contains("elements[2]=ZG9/ZH1/ZH2"),
        "should resolve ZG9/ZH1/ZH2 at elements[2]"
    );

    // DTM+471: elements[0]=471
    assert!(
        prompt.contains("elements[0]=471"),
        "should resolve DTM+471"
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
fn test_parse_truncated_response_recovers_complete_conditions() {
    let generator = ClaudeConditionGenerator::new(4);

    // Simulates a truncated response — two complete conditions, third cut off mid-JSON
    let json = r#"```json
{
  "conditions": [
    {
      "id": "8",
      "implementation": "ctx.external.evaluate(\"data_clearing_required\")",
      "confidence": "high",
      "reasoning": "External condition",
      "is_external": true,
      "external_name": "data_clearing_required"
    },
    {
      "id": "9",
      "implementation": "ctx.external.evaluate(\"sender_is_msb\")",
      "confidence": "high",
      "reasoning": "Requires registry lookup",
      "is_external": true,
      "external_name": "sender_is_msb"
    },
    {
      "id": "10",
      "implementation": "ctx.external."#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 2, "should recover 2 complete conditions");
    assert_eq!(conditions[0].condition_number, 8);
    assert_eq!(conditions[1].condition_number, 9);
}

#[test]
fn test_parse_truncated_markdown_block_no_closing_fence() {
    let generator = ClaudeConditionGenerator::new(4);

    // Complete JSON but markdown block has no closing ```
    let json = r#"```json
{
  "conditions": [
    {
      "id": "42",
      "implementation": "ConditionResult::True",
      "confidence": "high",
      "reasoning": "Always true",
      "is_external": false
    }
  ]
}
"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].condition_number, 42);
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

#[test]
fn test_user_prompt_detects_group_scope() {
    let conditions = vec![
        ConditionInput {
            id: "15".to_string(),
            description:
                "Wenn in derselben SG8 das SEQ+Z98 (Informative Daten) vorhanden".to_string(),
            referencing_fields: None,
        },
        ConditionInput {
            id: "23".to_string(),
            description: "Wenn in dieser SG4 das STS+E01++A05/A99 (Status) vorhanden".to_string(),
            referencing_fields: None,
        },
    ];
    let context = ConditionContext {
        message_type: "UTILMD",
        format_version: "FV2504",
        mig_schema: None,
        example_implementations: vec![],
    };
    let prompt = build_user_prompt(&conditions, &context);
    assert!(
        prompt.contains("GROUP-SCOPED"),
        "should annotate group-scoped conditions"
    );
    assert!(
        prompt.contains("SG8"),
        "should identify the target group for condition 15"
    );
    assert!(
        prompt.contains("SG4"),
        "should identify the target group for condition 23"
    );
}

#[test]
fn test_system_prompt_documents_group_scoped_api() {
    let prompt = build_system_prompt();
    assert!(
        prompt.contains("find_segments_in_group"),
        "should document group-scoped find"
    );
    assert!(
        prompt.contains("find_segments_with_qualifier_in_group"),
        "should document group-scoped qualifier find"
    );
    assert!(
        prompt.contains("group_instance_count"),
        "should document group instance count"
    );
}
