use automapper_generator::validation::schema_validator::*;
use std::collections::HashSet;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_validate_generated_code_no_errors() {
    let tmp = TempDir::new().unwrap();

    // Write a generated file that only references known types
    let code = r#"
use automapper_validation::condition::{ConditionEvaluator, ConditionResult, EvaluationContext};

pub struct TestEvaluator;

impl ConditionEvaluator for TestEvaluator {
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult {
        match condition {
            1 => ConditionResult::True,
            _ => ConditionResult::Unknown,
        }
    }

    fn is_external(&self, _condition: u32) -> bool {
        false
    }
}
"#;

    std::fs::write(tmp.path().join("test_evaluator.rs"), code).unwrap();

    let known_types = HashSet::new(); // No BO4E types needed for this code
    let report = validate_generated_code(tmp.path(), &known_types).unwrap();

    assert!(report.errors.is_empty(), "should have no errors");
}

#[test]
fn test_validate_generated_code_warns_unknown_type() {
    let tmp = TempDir::new().unwrap();

    let code = r#"
use automapper_validation::condition::{ConditionEvaluator, ConditionResult};

fn check_marktlokation(malo: &Marktlokation) -> bool {
    true
}
"#;

    std::fs::write(tmp.path().join("test_mapper.rs"), code).unwrap();

    let known_types = HashSet::new(); // Marktlokation is NOT in the known types
    let report = validate_generated_code(tmp.path(), &known_types).unwrap();

    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.message.contains("Marktlokation")),
        "should warn about unknown Marktlokation type"
    );
}

#[test]
fn test_validate_generated_code_known_type_passes() {
    let tmp = TempDir::new().unwrap();

    let code = r#"
fn check_marktlokation(malo: &Marktlokation) -> bool {
    true
}
"#;

    std::fs::write(tmp.path().join("test_mapper.rs"), code).unwrap();

    let mut known_types = HashSet::new();
    known_types.insert("Marktlokation".to_string());

    let report = validate_generated_code(tmp.path(), &known_types).unwrap();

    assert!(
        !report
            .warnings
            .iter()
            .any(|w| w.message.contains("Marktlokation")),
        "should NOT warn about known Marktlokation type"
    );
}

#[test]
fn test_validate_missing_generated_dir() {
    let known_types = HashSet::new();
    let result = validate_generated_code(Path::new("/nonexistent/dir"), &known_types);
    assert!(result.is_err());
}

#[test]
fn test_validation_report_display() {
    let issue = SchemaValidationIssue {
        file: "test.rs".to_string(),
        line: 42,
        message: "type 'Foo' not found".to_string(),
    };
    assert_eq!(issue.to_string(), "test.rs:42: type 'Foo' not found");
}
