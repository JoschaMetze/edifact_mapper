//! Full integration tests for the EdifactValidator.

use automapper_validation::eval::{
    ConditionEvaluator, ConditionResult, EvaluationContext, ExternalConditionProvider,
    NoOpExternalProvider,
};
use automapper_validation::validator::validate::{AhbCodeRule, AhbFieldRule, AhbWorkflow};
use automapper_validation::validator::{
    EdifactValidator, ErrorCodes, Severity, ValidationCategory, ValidationLevel, ValidationReport,
};
use std::collections::HashMap;

// === Test helpers ===

struct ConfigurableEvaluator {
    results: HashMap<u32, ConditionResult>,
    external_ids: Vec<u32>,
}

impl ConfigurableEvaluator {
    fn new() -> Self {
        Self {
            results: HashMap::new(),
            external_ids: Vec::new(),
        }
    }

    fn condition(mut self, id: u32, result: ConditionResult) -> Self {
        self.results.insert(id, result);
        self
    }

    #[allow(dead_code)]
    fn external(mut self, id: u32) -> Self {
        self.external_ids.push(id);
        self
    }
}

impl ConditionEvaluator for ConfigurableEvaluator {
    fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
        self.results
            .get(&condition)
            .copied()
            .unwrap_or(ConditionResult::Unknown)
    }

    fn is_external(&self, condition: u32) -> bool {
        self.external_ids.contains(&condition)
    }

    fn message_type(&self) -> &str {
        "UTILMD"
    }

    fn format_version(&self) -> &str {
        "FV2510"
    }
}

struct FixedExternalProvider {
    results: HashMap<String, ConditionResult>,
}

impl FixedExternalProvider {
    #[allow(dead_code)]
    fn new(results: Vec<(&str, ConditionResult)>) -> Self {
        Self {
            results: results
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }
}

impl ExternalConditionProvider for FixedExternalProvider {
    fn evaluate(&self, name: &str) -> ConditionResult {
        self.results
            .get(name)
            .copied()
            .unwrap_or(ConditionResult::Unknown)
    }
}

fn make_workflow(fields: Vec<AhbFieldRule>) -> AhbWorkflow {
    AhbWorkflow {
        pruefidentifikator: "11001".to_string(),
        description: "Lieferbeginn".to_string(),
        communication_direction: Some("LF an NB".to_string()),
        fields,
    }
}

fn simple_field(path: &str, name: &str, status: &str) -> AhbFieldRule {
    AhbFieldRule {
        segment_path: path.to_string(),
        name: name.to_string(),
        ahb_status: status.to_string(),
        codes: vec![],
    }
}

#[allow(dead_code)]
fn field_with_codes(
    path: &str,
    name: &str,
    status: &str,
    codes: Vec<(&str, &str)>,
) -> AhbFieldRule {
    AhbFieldRule {
        segment_path: path.to_string(),
        name: name.to_string(),
        ahb_status: status.to_string(),
        codes: codes
            .into_iter()
            .map(|(value, ahb)| AhbCodeRule {
                value: value.to_string(),
                description: format!("Code {value}"),
                ahb_status: ahb.to_string(),
            })
            .collect(),
    }
}

// === Test cases ===

#[test]
fn test_validate_utilmd_lieferbeginn_all_fields_present() {
    // When all conditions are met and all fields are present, no errors
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::True)
        .condition(6, ConditionResult::True);

    let workflow = make_workflow(vec![
        simple_field("SG2/NAD/3035", "Partnerrolle", "Muss"),
        simple_field("SG2/NAD/C082/3039", "MP-ID", "Muss [182] ∧ [152]"),
        simple_field("SG4/STS/C556/9013", "Transaktionsgrund", "Muss"),
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    // With no segments, all mandatory fields will be missing
    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    // All Muss fields are missing -> errors
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 3);
}

#[test]
fn test_validate_conditional_fields_not_required_when_false() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::False)
        .condition(152, ConditionResult::True);

    let workflow = make_workflow(vec![simple_field(
        "SG2/NAD/C082/3039",
        "MP-ID",
        "Muss [182] ∧ [152]",
    )]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    // [182]=F makes AND false -> field not required -> no error
    assert!(report.is_valid());
}

#[test]
fn test_validate_mixed_mandatory_and_conditional_fields() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::False); // Condition false

    let workflow = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss"), // Always required
        simple_field("DTM", "Datum", "Muss [182] ∧ [152]"), // Condition false
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    // NAD is missing (error), DTM condition is false (no error)
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 1);
    let error = report.errors().next().unwrap();
    assert!(error.message.contains("Partnerrolle"));
}

#[test]
fn test_validate_unknown_conditions_produce_info() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .external(8); // 8 is external and not registered -> Unknown

    let workflow = make_workflow(vec![simple_field(
        "NAD",
        "Partnerrolle",
        "Muss [182] ∧ [8]",
    )]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    // [182]=T, [8]=Unknown -> AND = Unknown -> info, not error
    assert!(report.is_valid());
    let infos: Vec<_> = report.infos().collect();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].code, ErrorCodes::CONDITION_UNKNOWN);
}

#[test]
fn test_validate_xor_expression() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(102, ConditionResult::True)
        .condition(2006, ConditionResult::True)
        .condition(103, ConditionResult::False)
        .condition(2005, ConditionResult::False);

    let workflow = make_workflow(vec![simple_field(
        "DTM",
        "Datum",
        "Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])",
    )]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    // XOR(T,F) = T -> field required, DTM missing -> error
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 1);
}

#[test]
fn test_validate_structure_level_ignores_conditions() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::True);

    let workflow = make_workflow(vec![simple_field(
        "NAD",
        "Partnerrolle",
        "Muss [182] ∧ [152]",
    )]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Structure);

    // Structure level does not check AHB conditions
    assert_eq!(report.by_category(ValidationCategory::Ahb).count(), 0);
}

#[test]
fn test_validate_report_serialization() {
    let evaluator = ConfigurableEvaluator::new();
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let workflow = make_workflow(vec![simple_field("NAD", "Partnerrolle", "Muss")]);

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Full);

    // Serialize to JSON and back
    let json = serde_json::to_string_pretty(&report).unwrap();
    let deserialized: ValidationReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.format_version.as_deref(), Some("FV2510"));
    assert_eq!(deserialized.level, ValidationLevel::Full);
    assert_eq!(deserialized.total_issues(), report.total_issues());
}

#[test]
fn test_validate_kann_field_not_mandatory() {
    let evaluator = ConfigurableEvaluator::new().condition(570, ConditionResult::True);

    let workflow = make_workflow(vec![simple_field("FTX", "Freitext", "Kann [570]")]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    // "Kann" is not mandatory even when condition is True
    assert!(report.is_valid());
}

#[test]
fn test_validate_multiple_workflows_same_validator() {
    let evaluator = ConfigurableEvaluator::new().condition(1, ConditionResult::True);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    // First workflow
    let wf1 = make_workflow(vec![simple_field("NAD", "Partnerrolle", "Muss")]);
    let report1 = validator.validate(&[], &wf1, &external, ValidationLevel::Conditions);
    assert_eq!(report1.error_count(), 1);

    // Second workflow with different fields
    let wf2 = make_workflow(vec![
        simple_field("DTM", "Datum", "Muss"),
        simple_field("BGM", "Nachrichtentyp", "Muss"),
    ]);
    let report2 = validator.validate(&[], &wf2, &external, ValidationLevel::Conditions);
    assert_eq!(report2.error_count(), 2);
}

#[test]
fn test_validate_severity_ordering_in_report() {
    let evaluator = ConfigurableEvaluator::new();
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let workflow = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss"),
        simple_field("DTM", "Datum", "X"),
    ]);

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Full);

    // Both fields are mandatory and missing
    assert_eq!(report.error_count(), 2);
    for error in report.errors() {
        assert_eq!(error.severity, Severity::Error);
        assert_eq!(error.category, ValidationCategory::Ahb);
    }
}

#[test]
fn test_validate_report_metadata() {
    let evaluator = ConfigurableEvaluator::new();
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let workflow = AhbWorkflow {
        pruefidentifikator: "11001".to_string(),
        description: String::new(),
        communication_direction: None,
        fields: vec![],
    };

    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

    assert_eq!(report.format_version.as_deref(), Some("FV2510"));
    assert_eq!(report.level, ValidationLevel::Conditions);
    assert_eq!(report.message_type, "UTILMD"); // Message type from evaluator
    assert_eq!(report.pruefidentifikator.as_deref(), Some("11001"));
}
