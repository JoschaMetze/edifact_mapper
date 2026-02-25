use automapper_generator::conditions::codegen::generate_condition_evaluator_file;
use automapper_generator::conditions::condition_types::*;
use std::collections::HashMap;

fn make_test_conditions() -> Vec<GeneratedCondition> {
    vec![
        GeneratedCondition {
            condition_number: 1,
            rust_code: None,
            is_external: true,
            confidence: ConfidenceLevel::High,
            reasoning: Some("Requires external context".to_string()),
            external_name: Some("message_splitting".to_string()),
            original_description: Some("Wenn Aufteilung vorhanden".to_string()),
            referencing_fields: Some(vec!["SG8/SEQ (Muss [1])".to_string()]),
        },
        GeneratedCondition {
            condition_number: 2,
            rust_code: Some(
                "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}"
                    .to_string(),
            ),
            is_external: false,
            confidence: ConfidenceLevel::High,
            reasoning: Some("Simple field check".to_string()),
            external_name: None,
            original_description: Some("Wenn Marktlokation vorhanden".to_string()),
            referencing_fields: None,
        },
        GeneratedCondition {
            condition_number: 99,
            rust_code: Some("ConditionResult::Unknown".to_string()),
            is_external: false,
            confidence: ConfidenceLevel::Medium,
            reasoning: Some("Needs review".to_string()),
            external_name: None,
            original_description: Some("Komplexe Bedingung".to_string()),
            referencing_fields: None,
        },
        GeneratedCondition {
            condition_number: 100,
            rust_code: None,
            is_external: false,
            confidence: ConfidenceLevel::Low,
            reasoning: Some("Too complex for auto-generation".to_string()),
            external_name: None,
            original_description: Some("Sehr komplexe Bedingung".to_string()),
            referencing_fields: None,
        },
    ]
}

#[test]
fn test_generate_condition_evaluator_file_structure() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test_ahb.xml",
        &HashMap::new(),
    );

    assert!(output.contains("auto-generated"), "should have header");
    assert!(
        output.contains("pub struct UtilmdConditionEvaluatorFV2510"),
        "should have struct name"
    );
    assert!(
        output.contains("impl ConditionEvaluator for UtilmdConditionEvaluatorFV2510"),
        "should impl trait"
    );
}

#[test]
fn test_generate_match_arms() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("1 => self.evaluate_1(ctx)"),
        "should have match arm for condition 1"
    );
    assert!(
        output.contains("2 => self.evaluate_2(ctx)"),
        "should have match arm for condition 2"
    );
    assert!(
        output.contains("99 => self.evaluate_99(ctx)"),
        "should have match arm for condition 99"
    );
    assert!(
        output.contains("_ => ConditionResult::Unknown"),
        "should have default match arm"
    );
}

#[test]
fn test_external_condition_output() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("ctx.external.evaluate(\"message_splitting\")"),
        "external condition should delegate to provider"
    );
    assert!(
        output.contains("EXTERNAL"),
        "external condition should have EXTERNAL doc comment"
    );
    assert!(
        output.contains("external_conditions.insert(1)"),
        "should register condition 1 as external"
    );
}

#[test]
fn test_high_confidence_condition_has_implementation() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("ctx.transaktion.marktlokationen.is_empty()"),
        "high-confidence condition should have generated implementation"
    );
}

#[test]
fn test_medium_confidence_has_review_marker() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("REVIEW"),
        "medium-confidence condition should have REVIEW comment"
    );
}

#[test]
fn test_low_confidence_has_todo() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("TODO: Condition [100] requires manual implementation"),
        "low-confidence condition should have TODO"
    );
}

#[test]
fn test_preserved_methods_included() {
    let conditions = vec![GeneratedCondition {
        condition_number: 5,
        rust_code: Some("ConditionResult::True".to_string()),
        is_external: false,
        confidence: ConfidenceLevel::High,
        reasoning: None,
        external_name: None,
        original_description: None,
        referencing_fields: None,
    }];

    let mut preserved = HashMap::new();
    preserved.insert(
        10,
        "    fn evaluate_10(&self, _ctx: &EvaluationContext) -> ConditionResult {\n        ConditionResult::False // previously generated\n    }\n".to_string(),
    );

    let output =
        generate_condition_evaluator_file("UTILMD", "FV2510", &conditions, "test.xml", &preserved);

    assert!(
        output.contains("5 => self.evaluate_5(ctx)"),
        "should have new condition"
    );
    assert!(
        output.contains("10 => self.evaluate_10(ctx)"),
        "should have preserved condition in match"
    );
    assert!(
        output.contains("previously generated"),
        "should include preserved method body"
    );
}

#[test]
fn test_condition_evaluator_snapshot() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test_ahb.xml",
        &HashMap::new(),
    );

    // Replace the dynamic timestamp line for stable snapshots
    let stable_output = output
        .lines()
        .map(|line| {
            if line.starts_with("// Generated:") {
                "// Generated: 2026-02-18T00:00:00Z"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!("condition_evaluator_fv2510", stable_output);
}
