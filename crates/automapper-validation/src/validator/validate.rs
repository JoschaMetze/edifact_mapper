//! Main EdifactValidator implementation.

use crate::error::ValidationError;
use crate::eval::{
    ConditionEvaluator, ConditionExprEvaluator, ConditionResult, EvaluationContext,
    ExternalConditionProvider,
};

use super::codes::ErrorCodes;
use super::issue::{Severity, ValidationCategory, ValidationIssue};
use super::level::ValidationLevel;
use super::report::ValidationReport;

/// AHB field definition for validation.
///
/// Represents a single field in an AHB rule table with its status
/// and allowed codes for a specific Pruefidentifikator.
#[derive(Debug, Clone)]
pub struct AhbFieldRule {
    /// Segment path (e.g., "SG2/NAD/C082/3039").
    pub segment_path: String,

    /// Human-readable field name (e.g., "MP-ID des MSB").
    pub name: String,

    /// AHB status (e.g., "Muss [182] ∧ [152]", "X", "Kann").
    pub ahb_status: String,

    /// Allowed code values with their AHB status.
    pub codes: Vec<AhbCodeRule>,
}

/// An allowed code value within an AHB field rule.
#[derive(Debug, Clone)]
pub struct AhbCodeRule {
    /// The code value (e.g., "E01", "Z33").
    pub value: String,

    /// Description of the code (e.g., "Anmeldung").
    pub description: String,

    /// AHB status for this code (e.g., "X", "Muss").
    pub ahb_status: String,
}

/// AHB workflow definition for a specific Pruefidentifikator.
#[derive(Debug, Clone)]
pub struct AhbWorkflow {
    /// The Pruefidentifikator (e.g., "11001", "55001").
    pub pruefidentifikator: String,

    /// Description of the workflow.
    pub description: String,

    /// Communication direction (e.g., "NB an LF").
    pub communication_direction: Option<String>,

    /// All field rules for this workflow.
    pub fields: Vec<AhbFieldRule>,
}

/// Validates EDIFACT messages against AHB business rules.
///
/// The validator is generic over the `ConditionEvaluator` implementation,
/// which is typically generated from AHB XML schemas.
///
/// # Example
///
/// ```ignore
/// use automapper_validation::validator::EdifactValidator;
/// use automapper_validation::eval::NoOpExternalProvider;
///
/// let evaluator = UtilmdConditionEvaluatorFV2510::new();
/// let validator = EdifactValidator::new(evaluator);
/// let external = NoOpExternalProvider;
///
/// let report = validator.validate(
///     edifact_bytes,
///     ValidationLevel::Full,
///     &external,
///     Some(&ahb_workflow),
/// )?;
///
/// if !report.is_valid() {
///     for error in report.errors() {
///         eprintln!("{error}");
///     }
/// }
/// ```
pub struct EdifactValidator<E: ConditionEvaluator> {
    evaluator: E,
}

impl<E: ConditionEvaluator> EdifactValidator<E> {
    /// Create a new validator with the given condition evaluator.
    pub fn new(evaluator: E) -> Self {
        Self { evaluator }
    }

    /// Validate an EDIFACT message.
    ///
    /// # Arguments
    ///
    /// * `input` - Raw EDIFACT bytes
    /// * `level` - Validation strictness level
    /// * `external` - Provider for external conditions
    /// * `workflow` - Optional AHB workflow definition for the PID
    ///
    /// # Returns
    ///
    /// A `ValidationReport` with all issues found, or an error if
    /// the EDIFACT content could not be parsed at all.
    pub fn validate(
        &self,
        input: &[u8],
        level: ValidationLevel,
        external: &dyn ExternalConditionProvider,
        workflow: Option<&AhbWorkflow>,
    ) -> Result<ValidationReport, ValidationError> {
        let input_str = std::str::from_utf8(input)
            .map_err(|_| ValidationError::Parse(edifact_parser::ParseError::UnexpectedEof))?;

        // Collect segments using the parser
        let segments = self.parse_segments(input_str)?;

        // Detect message type from UNH segment
        let message_type = self.detect_message_type(&segments).unwrap_or("UNKNOWN");

        // Build the report
        let mut report = ValidationReport::new(message_type, level)
            .with_format_version(self.evaluator.format_version());

        // Detect PID from RFF+Z13
        let pid_string = self
            .detect_pruefidentifikator(&segments)
            .map(|s| s.to_string());
        if let Some(ref pid) = pid_string {
            report.pruefidentifikator = Some(pid.clone());
        }

        // Create evaluation context
        let pid_ref = pid_string.as_deref().unwrap_or("");
        let ctx = EvaluationContext::new(pid_ref, external, &segments);

        // Structure validation (always performed)
        self.validate_structure(&segments, &mut report);

        // Condition validation (if level >= Conditions and workflow provided)
        if matches!(level, ValidationLevel::Conditions | ValidationLevel::Full) {
            if let Some(wf) = workflow {
                self.validate_conditions(wf, &ctx, &mut report);
            }
        }

        Ok(report)
    }

    /// Parse EDIFACT content into segments.
    fn parse_segments<'a>(
        &self,
        _input: &'a str,
    ) -> Result<Vec<edifact_types::RawSegment<'a>>, ValidationError> {
        // TODO: Use EdifactStreamParser from edifact-parser crate to parse segments.
        // For now, return empty vec. The actual parsing will be wired up when
        // Feature 1 (edifact-parser) is integrated.
        Ok(Vec::new())
    }

    /// Detect the message type from the UNH segment.
    fn detect_message_type<'a>(
        &self,
        segments: &'a [edifact_types::RawSegment<'a>],
    ) -> Option<&'a str> {
        segments
            .iter()
            .find(|s| s.id == "UNH")
            .and_then(|unh| unh.elements.get(1))
            .and_then(|e| e.first())
            .copied()
    }

    /// Detect the Pruefidentifikator from RFF+Z13.
    fn detect_pruefidentifikator<'a>(
        &self,
        segments: &'a [edifact_types::RawSegment<'a>],
    ) -> Option<&'a str> {
        segments.iter().find_map(|s| {
            if s.id != "RFF" {
                return None;
            }
            let qualifier = s.elements.first()?.first()?;
            if *qualifier == "Z13" {
                s.elements.first()?.get(1).copied()
            } else {
                None
            }
        })
    }

    /// Validate EDIFACT structure (segment presence, ordering).
    fn validate_structure(
        &self,
        _segments: &[edifact_types::RawSegment],
        _report: &mut ValidationReport,
    ) {
        // TODO: Implement MIG structure validation when MIG schema types
        // are available from automapper-generator. For now, this is a
        // placeholder that will be filled in when the generator crate
        // provides MigSchema types.
    }

    /// Validate AHB conditions for each field in the workflow.
    fn validate_conditions(
        &self,
        workflow: &AhbWorkflow,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        let expr_eval = ConditionExprEvaluator::new(&self.evaluator);

        for field in &workflow.fields {
            // Evaluate the AHB status condition expression
            let condition_result = expr_eval.evaluate_status(&field.ahb_status, ctx);

            match condition_result {
                ConditionResult::True => {
                    // Condition is met - field is required/applicable
                    if is_mandatory_status(&field.ahb_status) {
                        let segment_id = extract_segment_id(&field.segment_path);
                        if !ctx.has_segment(&segment_id) {
                            report.add_issue(
                                ValidationIssue::new(
                                    Severity::Error,
                                    ValidationCategory::Ahb,
                                    ErrorCodes::MISSING_REQUIRED_FIELD,
                                    format!(
                                        "Required field '{}' at {} is missing",
                                        field.name, field.segment_path
                                    ),
                                )
                                .with_field_path(&field.segment_path)
                                .with_rule(&field.ahb_status),
                            );
                        }
                    }

                    // Validate code values if field has code restrictions
                    self.validate_field_codes(field, ctx, report);
                }
                ConditionResult::False => {
                    // Condition not met - field not required, skip
                }
                ConditionResult::Unknown => {
                    // Cannot determine - add info-level warning
                    report.add_issue(
                        ValidationIssue::new(
                            Severity::Info,
                            ValidationCategory::Ahb,
                            ErrorCodes::CONDITION_UNKNOWN,
                            format!(
                                "Condition for field '{}' could not be fully evaluated (external conditions missing)",
                                field.name
                            ),
                        )
                        .with_field_path(&field.segment_path)
                        .with_rule(&field.ahb_status),
                    );
                }
            }
        }
    }

    /// Validate code values for a field against AHB allowed codes.
    fn validate_field_codes(
        &self,
        field: &AhbFieldRule,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        if field.codes.is_empty() {
            return;
        }

        let allowed_codes: Vec<&str> = field
            .codes
            .iter()
            .filter(|c| c.ahb_status == "X" || c.ahb_status.starts_with("Muss"))
            .map(|c| c.value.as_str())
            .collect();

        if allowed_codes.is_empty() {
            return;
        }

        let segment_id = extract_segment_id(&field.segment_path);
        let matching_segments = ctx.find_segments(&segment_id);

        for segment in matching_segments {
            if let Some(first_element) = segment.elements.first() {
                if let Some(code_value) = first_element.first() {
                    if !code_value.is_empty() && !allowed_codes.contains(code_value) {
                        report.add_issue(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationCategory::Code,
                                ErrorCodes::CODE_NOT_ALLOWED_FOR_PID,
                                format!(
                                    "Code '{}' is not allowed for this PID. Allowed: [{}]",
                                    code_value,
                                    allowed_codes.join(", ")
                                ),
                            )
                            .with_field_path(&field.segment_path)
                            .with_actual(*code_value)
                            .with_expected(allowed_codes.join(", ")),
                        );
                    }
                }
            }
        }
    }
}

/// Check if an AHB status is mandatory (Muss or X prefix).
fn is_mandatory_status(status: &str) -> bool {
    let trimmed = status.trim();
    trimmed.starts_with("Muss") || trimmed.starts_with('X')
}

/// Extract the segment ID from a field path like "SG2/NAD/C082/3039" -> "NAD".
fn extract_segment_id(path: &str) -> String {
    for part in path.split('/') {
        // Skip segment group identifiers and composite/element identifiers
        if part.starts_with("SG") || part.starts_with("C_") || part.starts_with("D_") {
            continue;
        }
        // Return first 3-letter uppercase segment identifier
        if part.len() >= 3
            && part
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return part.to_string();
        }
    }
    // Fallback: return the last part
    path.split('/').next_back().unwrap_or(path).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{ConditionResult as CR, NoOpExternalProvider};
    use std::collections::HashMap;

    /// Mock evaluator for testing the validator.
    struct MockEvaluator {
        results: HashMap<u32, CR>,
    }

    impl MockEvaluator {
        fn new(results: Vec<(u32, CR)>) -> Self {
            Self {
                results: results.into_iter().collect(),
            }
        }

        fn all_true(ids: &[u32]) -> Self {
            Self::new(ids.iter().map(|&id| (id, CR::True)).collect())
        }
    }

    impl ConditionEvaluator for MockEvaluator {
        fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> CR {
            self.results.get(&condition).copied().unwrap_or(CR::Unknown)
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

    // === Helper function tests ===

    #[test]
    fn test_is_mandatory_status() {
        assert!(is_mandatory_status("Muss"));
        assert!(is_mandatory_status("Muss [182] ∧ [152]"));
        assert!(is_mandatory_status("X"));
        assert!(is_mandatory_status("X [567]"));
        assert!(!is_mandatory_status("Soll [1]"));
        assert!(!is_mandatory_status("Kann [1]"));
        assert!(!is_mandatory_status(""));
    }

    #[test]
    fn test_extract_segment_id_simple() {
        assert_eq!(extract_segment_id("NAD"), "NAD");
    }

    #[test]
    fn test_extract_segment_id_with_sg_prefix() {
        assert_eq!(extract_segment_id("SG2/NAD/C082/3039"), "NAD");
    }

    #[test]
    fn test_extract_segment_id_nested_sg() {
        assert_eq!(extract_segment_id("SG4/SG8/SEQ/C286/6350"), "SEQ");
    }

    // === Validator tests with mock data ===

    #[test]
    fn test_validate_missing_mandatory_field() {
        let evaluator = MockEvaluator::all_true(&[182, 152]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "SG2/NAD/C082/3039".to_string(),
                name: "MP-ID des MSB".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        // Validate empty EDIFACT (will have no segments)
        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Should have an error for missing mandatory field
        assert!(!report.is_valid());
        let errors: Vec<_> = report.errors().collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCodes::MISSING_REQUIRED_FIELD);
        assert!(errors[0].message.contains("MP-ID des MSB"));
    }

    #[test]
    fn test_validate_condition_false_no_error() {
        // When condition evaluates to False, field is not required
        let evaluator = MockEvaluator::new(vec![(182, CR::True), (152, CR::False)]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "NAD".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Condition is false, so field is not required - no error
        assert!(report.is_valid());
    }

    #[test]
    fn test_validate_condition_unknown_adds_info() {
        // When condition is Unknown, add an info-level note
        let evaluator = MockEvaluator::new(vec![(182, CR::True)]);
        // 152 is not registered -> Unknown
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "NAD".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Should be valid (Unknown is not an error) but have an info issue
        assert!(report.is_valid());
        let infos: Vec<_> = report.infos().collect();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].code, ErrorCodes::CONDITION_UNKNOWN);
    }

    #[test]
    fn test_validate_structure_level_skips_conditions() {
        let evaluator = MockEvaluator::all_true(&[182, 152]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "NAD".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        // With Structure level, conditions are not checked
        let report = validator
            .validate(b"", ValidationLevel::Structure, &external, Some(&workflow))
            .unwrap();

        // No AHB errors because conditions were not evaluated
        assert!(report.is_valid());
        assert_eq!(report.by_category(ValidationCategory::Ahb).count(), 0);
    }

    #[test]
    fn test_validate_no_workflow_no_condition_errors() {
        let evaluator = MockEvaluator::all_true(&[]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        // No workflow provided
        let report = validator
            .validate(b"", ValidationLevel::Full, &external, None)
            .unwrap();

        assert!(report.is_valid());
    }

    #[test]
    fn test_validate_bare_muss_always_required() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: Some("NB an LF".to_string()),
            fields: vec![AhbFieldRule {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss".to_string(), // No conditions
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Bare "Muss" with no conditions -> unconditionally required -> missing = error
        assert!(!report.is_valid());
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_validate_x_status_is_mandatory() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "DTM".to_string(),
                name: "Datum".to_string(),
                ahb_status: "X".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        assert!(!report.is_valid());
        let errors: Vec<_> = report.errors().collect();
        assert_eq!(errors[0].code, ErrorCodes::MISSING_REQUIRED_FIELD);
    }

    #[test]
    fn test_validate_soll_not_mandatory() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "DTM".to_string(),
                name: "Datum".to_string(),
                ahb_status: "Soll".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Soll is not mandatory, so missing is not an error
        assert!(report.is_valid());
    }

    #[test]
    fn test_report_includes_metadata() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let report = validator
            .validate(b"", ValidationLevel::Full, &external, None)
            .unwrap();

        assert_eq!(report.format_version.as_deref(), Some("FV2510"));
        assert_eq!(report.level, ValidationLevel::Full);
    }
}
