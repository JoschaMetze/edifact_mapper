//! Snapshot tests for ValidationReport JSON structure.
//!
//! These tests use `insta` to capture the serialized JSON shape of
//! `ValidationReport`, catching accidental schema regressions.

use std::collections::HashMap;

use automapper_validation::eval::{
    ConditionEvaluator, ConditionResult, EvaluationContext, NoOpExternalProvider,
};
use automapper_validation::validator::validate::{AhbFieldRule, AhbWorkflow};
use automapper_validation::validator::{EdifactValidator, ValidationLevel};
use mig_types::segment::OwnedSegment;

// ---------------------------------------------------------------------------
// Test evaluator
// ---------------------------------------------------------------------------

/// A simple HashMap-backed evaluator for snapshot tests.
/// Returns `Unknown` for any condition not in the map.
struct TestEvaluator {
    results: HashMap<u32, ConditionResult>,
}

impl TestEvaluator {
    fn new(results: HashMap<u32, ConditionResult>) -> Self {
        Self { results }
    }
}

impl ConditionEvaluator for TestEvaluator {
    fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
        self.results
            .get(&condition)
            .copied()
            .unwrap_or(ConditionResult::Unknown)
    }

    fn is_external(&self, _condition: u32) -> bool {
        false
    }

    fn message_type(&self) -> &str {
        "UTILMD"
    }

    fn format_version(&self) -> &str {
        "FV2510"
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_segment(id: &str, elements: Vec<Vec<&str>>) -> OwnedSegment {
    OwnedSegment {
        id: id.to_string(),
        elements: elements
            .into_iter()
            .map(|e| e.into_iter().map(|c| c.to_string()).collect())
            .collect(),
        segment_number: 1,
    }
}

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------

/// A clean report: one bare "Muss" field with a matching NAD segment present.
/// The report should be valid with zero issues.
#[test]
fn test_snapshot_clean_report() {
    let evaluator = TestEvaluator::new(HashMap::new());
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let workflow = AhbWorkflow {
        pruefidentifikator: "55001".to_string(),
        description: "Anmeldung MaLo".to_string(),
        communication_direction: Some("NB an LF".to_string()),
        fields: vec![AhbFieldRule {
            segment_path: "SG2/NAD/3035".to_string(),
            name: "Partnerrolle".to_string(),
            ahb_status: "Muss".to_string(),
            codes: vec![],
        }],
    };

    // Provide a matching NAD segment so the mandatory field is satisfied.
    let segments = vec![make_segment(
        "NAD",
        vec![vec!["MS"], vec!["1234567890123", "", "293"]],
    )];

    let report = validator.validate(&segments, &workflow, &external, ValidationLevel::Full);

    let json = serde_json::to_value(&report).expect("report serializes to JSON");
    insta::assert_json_snapshot!("clean_report", json);
}

/// A report with errors: conditions evaluate to True but no segments are
/// provided, so mandatory fields are missing.
#[test]
fn test_snapshot_report_with_errors() {
    // All referenced conditions evaluate to True.
    let mut conditions = HashMap::new();
    conditions.insert(182, ConditionResult::True);
    conditions.insert(152, ConditionResult::True);

    let evaluator = TestEvaluator::new(conditions);
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let workflow = AhbWorkflow {
        pruefidentifikator: "11001".to_string(),
        description: "Lieferbeginn".to_string(),
        communication_direction: Some("LF an NB".to_string()),
        fields: vec![
            AhbFieldRule {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss".to_string(),
                codes: vec![],
            },
            AhbFieldRule {
                segment_path: "SG2/NAD/C082/3039".to_string(),
                name: "MP-ID des MSB".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            },
            AhbFieldRule {
                segment_path: "SG4/DTM/C507/2380".to_string(),
                name: "Lieferbeginn-Datum".to_string(),
                ahb_status: "X".to_string(),
                codes: vec![],
            },
        ],
    };

    // Empty segments -> all mandatory fields are missing.
    let report = validator.validate(&[], &workflow, &external, ValidationLevel::Full);

    let json = serde_json::to_value(&report).expect("report serializes to JSON");
    insta::assert_json_snapshot!("report_with_errors", json);
}
