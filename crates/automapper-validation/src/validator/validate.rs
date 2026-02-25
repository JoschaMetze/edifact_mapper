//! Main EdifactValidator implementation.

use std::collections::{HashMap, HashSet};

use crate::eval::{
    ConditionEvaluator, ConditionExprEvaluator, ConditionResult, EvaluationContext,
    ExternalConditionProvider,
};
use mig_types::navigator::GroupNavigator;
use mig_types::segment::OwnedSegment;

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
/// The validator is a pure validation engine: it receives pre-parsed
/// segments, an AHB workflow, and an external condition provider.
/// Parsing and message-type detection are the caller's responsibility.
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
///     &segments,
///     &ahb_workflow,
///     &external,
///     ValidationLevel::Full,
/// );
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

    /// Validate pre-parsed EDIFACT segments against an AHB workflow.
    ///
    /// # Arguments
    ///
    /// * `segments` - Pre-parsed EDIFACT segments
    /// * `workflow` - AHB workflow definition for the PID
    /// * `external` - Provider for external conditions
    /// * `level` - Validation strictness level
    ///
    /// # Returns
    ///
    /// A `ValidationReport` with all issues found.
    pub fn validate(
        &self,
        segments: &[OwnedSegment],
        workflow: &AhbWorkflow,
        external: &dyn ExternalConditionProvider,
        level: ValidationLevel,
    ) -> ValidationReport {
        let mut report = ValidationReport::new(self.evaluator.message_type(), level)
            .with_format_version(self.evaluator.format_version())
            .with_pruefidentifikator(&workflow.pruefidentifikator);

        let ctx = EvaluationContext::new(&workflow.pruefidentifikator, external, segments);

        if matches!(level, ValidationLevel::Conditions | ValidationLevel::Full) {
            self.validate_conditions(workflow, &ctx, &mut report);
        }

        report
    }

    /// Validate with a group navigator for group-scoped condition queries.
    ///
    /// Same as [`validate`] but passes a `GroupNavigator` to the
    /// `EvaluationContext`, enabling conditions to query segments within
    /// specific group instances (e.g., "in derselben SG8").
    pub fn validate_with_navigator(
        &self,
        segments: &[OwnedSegment],
        workflow: &AhbWorkflow,
        external: &dyn ExternalConditionProvider,
        level: ValidationLevel,
        navigator: &dyn GroupNavigator,
    ) -> ValidationReport {
        let mut report = ValidationReport::new(self.evaluator.message_type(), level)
            .with_format_version(self.evaluator.format_version())
            .with_pruefidentifikator(&workflow.pruefidentifikator);

        let ctx = EvaluationContext::with_navigator(
            &workflow.pruefidentifikator,
            external,
            segments,
            navigator,
        );

        if matches!(level, ValidationLevel::Conditions | ValidationLevel::Full) {
            self.validate_conditions(workflow, &ctx, &mut report);
        }

        report
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
                    if is_mandatory_status(&field.ahb_status)
                        && !is_field_present(ctx, field)
                        && !is_parent_group_absent(ctx, &field.segment_path)
                    {
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

        // Cross-field code validation: aggregate allowed codes across all field
        // rules sharing the same segment path, then check each segment instance
        // against the combined set. This avoids false positives from per-field
        // validation (e.g., NAD/3035 with [MS] for sender and [MR] for receiver).
        self.validate_codes_cross_field(workflow, ctx, report);
    }

    /// Validate qualifier codes by aggregating allowed values across field rules.
    ///
    /// When a group navigator is available, codes are grouped by segment path
    /// (e.g., `SG2/NAD/3035` → `{MS, MR}`) and segments are looked up within
    /// the specific group, giving precise per-group validation.
    ///
    /// Without a navigator, falls back to grouping by segment tag and unioning
    /// all codes across groups to avoid cross-group false positives.
    ///
    /// Only validates simple qualifier paths (`[SG/]*/SEG/ELEMENT`) where the
    /// code is in `element[0][0]`. Composite paths are skipped.
    fn validate_codes_cross_field(
        &self,
        workflow: &AhbWorkflow,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        if ctx.navigator.is_some() {
            self.validate_codes_group_scoped(workflow, ctx, report);
        } else {
            self.validate_codes_tag_scoped(workflow, ctx, report);
        }
    }

    /// Group-scoped code validation: group codes by path, use navigator to
    /// find segments within each group, check against that group's codes only.
    fn validate_codes_group_scoped(
        &self,
        workflow: &AhbWorkflow,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        // Group allowed codes by (group_path_key, segment_tag).
        // E.g., "SG2/NAD/3035" → key ("SG2", "NAD"), "SG4/SG12/NAD/3035" → key ("SG4/SG12", "NAD")
        let mut codes_by_group: HashMap<(String, String), HashSet<&str>> = HashMap::new();

        for field in &workflow.fields {
            if field.codes.is_empty() || !is_qualifier_field(&field.segment_path) {
                continue;
            }
            let tag = extract_segment_id(&field.segment_path);
            let group_key = extract_group_path_key(&field.segment_path);
            let entry = codes_by_group.entry((group_key, tag)).or_default();
            for code in &field.codes {
                if code.ahb_status == "X" || code.ahb_status.starts_with("Muss") {
                    entry.insert(&code.value);
                }
            }
        }

        let nav = ctx.navigator.unwrap();

        for ((group_key, tag), allowed_codes) in &codes_by_group {
            if allowed_codes.is_empty() {
                continue;
            }

            let group_path: Vec<&str> = if group_key.is_empty() {
                Vec::new()
            } else {
                group_key.split('/').collect()
            };

            // Iterate over all instances of this group and check segments.
            if group_path.is_empty() {
                // No group prefix (e.g., bare "NAD/3035") — check flat segments.
                Self::check_segments_against_codes(
                    ctx.find_segments(tag),
                    allowed_codes,
                    tag,
                    &format!("{tag}/qualifier"),
                    report,
                );
            } else {
                let instance_count = nav.group_instance_count(&group_path);
                for i in 0..instance_count {
                    let segments = nav.find_segments_in_group(tag, &group_path, i);
                    let refs: Vec<&OwnedSegment> = segments.iter().collect();
                    Self::check_segments_against_codes(
                        refs,
                        allowed_codes,
                        tag,
                        &format!("{group_key}/{tag}/qualifier"),
                        report,
                    );
                }
            }
        }
    }

    /// Fallback: group codes by segment tag (unioning across all groups) and
    /// check all segments of that tag. Used when no navigator is available.
    fn validate_codes_tag_scoped(
        &self,
        workflow: &AhbWorkflow,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        let mut codes_by_tag: HashMap<String, HashSet<&str>> = HashMap::new();

        for field in &workflow.fields {
            if field.codes.is_empty() || !is_qualifier_field(&field.segment_path) {
                continue;
            }
            let tag = extract_segment_id(&field.segment_path);
            let entry = codes_by_tag.entry(tag).or_default();
            for code in &field.codes {
                if code.ahb_status == "X" || code.ahb_status.starts_with("Muss") {
                    entry.insert(&code.value);
                }
            }
        }

        for (tag, allowed_codes) in &codes_by_tag {
            if allowed_codes.is_empty() {
                continue;
            }
            Self::check_segments_against_codes(
                ctx.find_segments(tag),
                allowed_codes,
                tag,
                &format!("{tag}/qualifier"),
                report,
            );
        }
    }

    /// Check a list of segments' qualifier (element[0][0]) against allowed codes.
    fn check_segments_against_codes(
        segments: Vec<&OwnedSegment>,
        allowed_codes: &HashSet<&str>,
        _tag: &str,
        field_path: &str,
        report: &mut ValidationReport,
    ) {
        for segment in segments {
            if let Some(code_value) = segment
                .elements
                .first()
                .and_then(|e| e.first())
                .filter(|v| !v.is_empty())
            {
                if !allowed_codes.contains(code_value.as_str()) {
                    let mut sorted_codes: Vec<&str> = allowed_codes.iter().copied().collect();
                    sorted_codes.sort_unstable();
                    report.add_issue(
                        ValidationIssue::new(
                            Severity::Error,
                            ValidationCategory::Code,
                            ErrorCodes::CODE_NOT_ALLOWED_FOR_PID,
                            format!(
                                "Code '{}' is not allowed for this PID. Allowed: [{}]",
                                code_value,
                                sorted_codes.join(", ")
                            ),
                        )
                        .with_field_path(field_path)
                        .with_actual(code_value)
                        .with_expected(sorted_codes.join(", ")),
                    );
                }
            }
        }
    }
}

/// Check if a field's required segment instance is present.
///
/// For fields with qualifier codes (e.g., `SG2/NAD/3035` with `[MR]`),
/// checks that a segment with one of those qualifier values exists.
/// Otherwise just checks that any segment with the tag is present.
///
/// This prevents false negatives where `NAD+MS` is present but the
/// validator incorrectly says NAD "exists" when `NAD+MR` is missing.
fn is_field_present(ctx: &EvaluationContext, field: &AhbFieldRule) -> bool {
    let segment_id = extract_segment_id(&field.segment_path);

    // If field has qualifier codes and is a simple qualifier path,
    // check for a segment with one of those specific qualifiers.
    if !field.codes.is_empty() && is_qualifier_field(&field.segment_path) {
        let required_codes: Vec<&str> = field.codes.iter().map(|c| c.value.as_str()).collect();
        let matching = ctx.find_segments(&segment_id);
        return matching.iter().any(|seg| {
            seg.elements
                .first()
                .and_then(|e| e.first())
                .is_some_and(|v| required_codes.contains(&v.as_str()))
        });
    }

    ctx.has_segment(&segment_id)
}

/// Check if a field's parent group is absent (has zero instances).
///
/// For a path like `SG2/SG3/CTA/3139`, the group path is `["SG2", "SG3"]`.
/// If the navigator reports 0 instances for that group, the group is absent
/// and fields inside it shouldn't be flagged as missing.
///
/// Returns `false` (not absent) when:
/// - The field has no group prefix (e.g., `NAD/3035`)
/// - No navigator is available (can't determine group presence)
fn is_parent_group_absent(ctx: &EvaluationContext, field_path: &str) -> bool {
    let group_path: Vec<&str> = field_path
        .split('/')
        .take_while(|p| p.starts_with("SG"))
        .collect();

    if group_path.is_empty() {
        return false;
    }

    match ctx.navigator {
        Some(nav) => nav.group_instance_count(&group_path) == 0,
        None => false,
    }
}

/// Check if an AHB status is mandatory (Muss or X prefix).
fn is_mandatory_status(status: &str) -> bool {
    let trimmed = status.trim();
    trimmed.starts_with("Muss") || trimmed.starts_with('X')
}

/// Check if a field path points to a simple qualifier element (element[0] of the segment).
///
/// Returns `true` for paths like `[SG/]*/SEG/ELEMENT` where the data element is
/// directly under the segment (no composite wrapper). These fields have their code
/// in `element[0][0]` and can be validated.
///
/// Returns `false` for composite paths like `SEG/COMPOSITE/ELEMENT` (e.g.,
/// `UNH/S009/0065`) where the element is inside a composite at an unknown index.
fn is_qualifier_field(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.starts_with("SG")).collect();
    // Expected: [SEGMENT_TAG, ELEMENT_ID] — exactly 2 parts after SG stripping.
    // If 3+ parts, there's a composite layer (e.g., [SEG, COMPOSITE, ELEMENT]).
    parts.len() == 2
}

/// Extract the group path prefix from a field path.
///
/// `"SG2/NAD/3035"` → `"SG2"`, `"SG4/SG12/NAD/3035"` → `"SG4/SG12"`,
/// `"NAD/3035"` → `""` (no group prefix).
fn extract_group_path_key(path: &str) -> String {
    let sg_parts: Vec<&str> = path
        .split('/')
        .take_while(|p| p.starts_with("SG"))
        .collect();
    sg_parts.join("/")
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

        // Validate with no segments
        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

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

        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

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

        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

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
        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Structure);

        // No AHB errors because conditions were not evaluated
        assert!(report.is_valid());
        assert_eq!(report.by_category(ValidationCategory::Ahb).count(), 0);
    }

    #[test]
    fn test_validate_empty_workflow_no_condition_errors() {
        let evaluator = MockEvaluator::all_true(&[]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let empty_workflow = AhbWorkflow {
            pruefidentifikator: String::new(),
            description: String::new(),
            communication_direction: None,
            fields: vec![],
        };

        let report = validator.validate(&[], &empty_workflow, &external, ValidationLevel::Full);

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

        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

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

        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

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

        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Conditions);

        // Soll is not mandatory, so missing is not an error
        assert!(report.is_valid());
    }

    #[test]
    fn test_report_includes_metadata() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: String::new(),
            communication_direction: None,
            fields: vec![],
        };

        let report = validator.validate(&[], &workflow, &external, ValidationLevel::Full);

        assert_eq!(report.format_version.as_deref(), Some("FV2510"));
        assert_eq!(report.level, ValidationLevel::Full);
        assert_eq!(report.message_type, "UTILMD");
        assert_eq!(report.pruefidentifikator.as_deref(), Some("55001"));
    }

    #[test]
    fn test_validate_with_navigator_returns_report() {
        let evaluator = MockEvaluator::all_true(&[]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;
        let nav = crate::eval::NoOpGroupNavigator;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![],
        };

        let report = validator.validate_with_navigator(
            &[],
            &workflow,
            &external,
            ValidationLevel::Full,
            &nav,
        );
        assert!(report.is_valid());
    }

    #[test]
    fn test_code_validation_skips_composite_paths() {
        // UNH/S009/0065 has codes like ["UTILMD"], but the code is in element[1]
        // (composite S009), not element[0] (message reference).
        // The validator should skip code validation for composite paths.
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let unh_segment = OwnedSegment {
            id: "UNH".to_string(),
            elements: vec![
                vec!["ALEXANDE951842".to_string()], // element 0: message ref
                vec![
                    "UTILMD".to_string(),
                    "D".to_string(),
                    "11A".to_string(),
                    "UN".to_string(),
                    "S2.1".to_string(),
                ],
            ],
            segment_number: 1,
        };

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "UNH/S009/0065".to_string(),
                    name: "Nachrichtentyp".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "UTILMD".to_string(),
                        description: "Stammdaten".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "UNH/S009/0052".to_string(),
                    name: "Version".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "D".to_string(),
                        description: "Draft".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        let report = validator.validate(
            &[unh_segment],
            &workflow,
            &external,
            ValidationLevel::Conditions,
        );

        // Should NOT produce COD002 false positives for composite element paths
        let code_errors: Vec<_> = report
            .by_category(ValidationCategory::Code)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(
            code_errors.is_empty(),
            "Expected no code errors for composite paths, got: {:?}",
            code_errors
        );
    }

    #[test]
    fn test_cross_field_code_validation_valid_qualifiers() {
        // NAD/3035 has separate field rules: [MS] for sender, [MR] for receiver.
        // Cross-field validation unions them → {MS, MR}. Both segments are valid.
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let nad_ms = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MS".to_string()]],
            segment_number: 4,
        };
        let nad_mr = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MR".to_string()]],
            segment_number: 5,
        };

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Absender".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MS".to_string(),
                        description: "Absender".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Empfaenger".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MR".to_string(),
                        description: "Empfaenger".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        let report = validator.validate(
            &[nad_ms, nad_mr],
            &workflow,
            &external,
            ValidationLevel::Conditions,
        );

        let code_errors: Vec<_> = report
            .by_category(ValidationCategory::Code)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(
            code_errors.is_empty(),
            "Expected no code errors for valid qualifiers, got: {:?}",
            code_errors
        );
    }

    #[test]
    fn test_cross_field_code_validation_catches_invalid_qualifier() {
        // NAD+MT is not in the allowed set {MS, MR} → should produce COD002.
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let nad_ms = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MS".to_string()]],
            segment_number: 4,
        };
        let nad_mt = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MT".to_string()]], // invalid
            segment_number: 5,
        };

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Absender".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MS".to_string(),
                        description: "Absender".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Empfaenger".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MR".to_string(),
                        description: "Empfaenger".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        let report = validator.validate(
            &[nad_ms, nad_mt],
            &workflow,
            &external,
            ValidationLevel::Conditions,
        );

        let code_errors: Vec<_> = report
            .by_category(ValidationCategory::Code)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert_eq!(code_errors.len(), 1, "Expected one COD002 error for MT");
        assert!(code_errors[0].message.contains("MT"));
        assert!(code_errors[0].message.contains("MR"));
        assert!(code_errors[0].message.contains("MS"));
    }

    #[test]
    fn test_cross_field_code_validation_unions_across_groups() {
        // SG2/NAD/3035 allows {MS, MR}, SG4/SG12/NAD/3035 allows {Z04, Z09}.
        // Since find_segments("NAD") returns all NADs, codes must be unioned
        // by tag: {MS, MR, Z04, Z09}. NAD+MT should be caught, all others pass.
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let segments = vec![
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["MS".to_string()]],
                segment_number: 3,
            },
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["MR".to_string()]],
                segment_number: 4,
            },
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["Z04".to_string()]],
                segment_number: 20,
            },
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["Z09".to_string()]],
                segment_number: 21,
            },
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["MT".to_string()]], // invalid
                segment_number: 22,
            },
        ];

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Absender".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MS".to_string(),
                        description: "Absender".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Empfaenger".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MR".to_string(),
                        description: "Empfaenger".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG4/SG12/NAD/3035".to_string(),
                    name: "Anschlussnutzer".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "Z04".to_string(),
                        description: "Anschlussnutzer".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG4/SG12/NAD/3035".to_string(),
                    name: "Korrespondenzanschrift".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "Z09".to_string(),
                        description: "Korrespondenzanschrift".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        let report =
            validator.validate(&segments, &workflow, &external, ValidationLevel::Conditions);

        let code_errors: Vec<_> = report
            .by_category(ValidationCategory::Code)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert_eq!(
            code_errors.len(),
            1,
            "Expected exactly one COD002 error for MT, got: {:?}",
            code_errors
        );
        assert!(code_errors[0].message.contains("MT"));
    }

    #[test]
    fn test_is_qualifier_field_simple_paths() {
        assert!(is_qualifier_field("NAD/3035"));
        assert!(is_qualifier_field("SG2/NAD/3035"));
        assert!(is_qualifier_field("SG4/SG8/SEQ/6350"));
        assert!(is_qualifier_field("LOC/3227"));
    }

    #[test]
    fn test_is_qualifier_field_composite_paths() {
        assert!(!is_qualifier_field("UNH/S009/0065"));
        assert!(!is_qualifier_field("NAD/C082/3039"));
        assert!(!is_qualifier_field("SG2/NAD/C082/3039"));
    }

    #[test]
    fn test_is_qualifier_field_bare_segment() {
        assert!(!is_qualifier_field("NAD"));
        assert!(!is_qualifier_field("SG2/NAD"));
    }

    #[test]
    fn test_missing_qualifier_instance_is_detected() {
        // NAD+MS is present but NAD+MR is missing.
        // The Empfaenger field requires [MR] → should produce AHB001.
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let nad_ms = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MS".to_string()]],
            segment_number: 3,
        };

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Absender".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MS".to_string(),
                        description: "Absender".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Empfaenger".to_string(),
                    ahb_status: "Muss".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MR".to_string(),
                        description: "Empfaenger".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        let report =
            validator.validate(&[nad_ms], &workflow, &external, ValidationLevel::Conditions);

        let ahb_errors: Vec<_> = report
            .by_category(ValidationCategory::Ahb)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert_eq!(
            ahb_errors.len(),
            1,
            "Expected AHB001 for missing NAD+MR, got: {:?}",
            ahb_errors
        );
        assert!(ahb_errors[0].message.contains("Empfaenger"));
    }

    #[test]
    fn test_present_qualifier_instance_no_error() {
        // Both NAD+MS and NAD+MR present → no AHB001 for either.
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let segments = vec![
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["MS".to_string()]],
                segment_number: 3,
            },
            OwnedSegment {
                id: "NAD".to_string(),
                elements: vec![vec!["MR".to_string()]],
                segment_number: 4,
            },
        ];

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Absender".to_string(),
                    ahb_status: "Muss".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MS".to_string(),
                        description: "Absender".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Empfaenger".to_string(),
                    ahb_status: "Muss".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MR".to_string(),
                        description: "Empfaenger".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        let report =
            validator.validate(&segments, &workflow, &external, ValidationLevel::Conditions);

        let ahb_errors: Vec<_> = report
            .by_category(ValidationCategory::Ahb)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(
            ahb_errors.is_empty(),
            "Expected no AHB001 errors, got: {:?}",
            ahb_errors
        );
    }

    #[test]
    fn test_extract_group_path_key() {
        assert_eq!(extract_group_path_key("SG2/NAD/3035"), "SG2");
        assert_eq!(extract_group_path_key("SG4/SG12/NAD/3035"), "SG4/SG12");
        assert_eq!(extract_group_path_key("NAD/3035"), "");
        assert_eq!(extract_group_path_key("SG4/SG8/SEQ/6350"), "SG4/SG8");
    }

    #[test]
    fn test_absent_optional_group_no_missing_field_error() {
        // SG3 is optional ("Kann"). If SG3 is absent, its children CTA/3139
        // and CTA/C056/3412 should NOT produce AHB001 errors.
        use mig_types::navigator::GroupNavigator;

        struct NavWithoutSG3;
        impl GroupNavigator for NavWithoutSG3 {
            fn find_segments_in_group(&self, _: &str, _: &[&str], _: usize) -> Vec<OwnedSegment> {
                vec![]
            }
            fn find_segments_with_qualifier_in_group(
                &self,
                _: &str,
                _: usize,
                _: &str,
                _: &[&str],
                _: usize,
            ) -> Vec<OwnedSegment> {
                vec![]
            }
            fn group_instance_count(&self, group_path: &[&str]) -> usize {
                match group_path {
                    ["SG2"] => 2,        // two NAD groups present
                    ["SG2", "SG3"] => 0, // SG3 is absent
                    _ => 0,
                }
            }
        }

        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;
        let nav = NavWithoutSG3;

        // Only NAD segments present, no CTA
        let segments = vec![
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![vec!["MS".into()]],
                segment_number: 3,
            },
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![vec!["MR".into()]],
                segment_number: 4,
            },
        ];

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/SG3/CTA/3139".to_string(),
                    name: "Funktion des Ansprechpartners, Code".to_string(),
                    ahb_status: "Muss".to_string(),
                    codes: vec![],
                },
                AhbFieldRule {
                    segment_path: "SG2/SG3/CTA/C056/3412".to_string(),
                    name: "Name vom Ansprechpartner".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![],
                },
            ],
        };

        let report = validator.validate_with_navigator(
            &segments,
            &workflow,
            &external,
            ValidationLevel::Conditions,
            &nav,
        );

        let ahb_errors: Vec<_> = report
            .by_category(ValidationCategory::Ahb)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert!(
            ahb_errors.is_empty(),
            "Expected no AHB001 errors when SG3 is absent, got: {:?}",
            ahb_errors
        );
    }

    #[test]
    fn test_present_group_still_checks_mandatory_fields() {
        // If SG3 IS present but CTA is missing within it → AHB001 error.
        use mig_types::navigator::GroupNavigator;

        struct NavWithSG3;
        impl GroupNavigator for NavWithSG3 {
            fn find_segments_in_group(&self, _: &str, _: &[&str], _: usize) -> Vec<OwnedSegment> {
                vec![]
            }
            fn find_segments_with_qualifier_in_group(
                &self,
                _: &str,
                _: usize,
                _: &str,
                _: &[&str],
                _: usize,
            ) -> Vec<OwnedSegment> {
                vec![]
            }
            fn group_instance_count(&self, group_path: &[&str]) -> usize {
                match group_path {
                    ["SG2"] => 1,
                    ["SG2", "SG3"] => 1, // SG3 is present
                    _ => 0,
                }
            }
        }

        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;
        let nav = NavWithSG3;

        // SG3 is present (nav says 1 instance) but CTA is not in flat segments
        let segments = vec![OwnedSegment {
            id: "NAD".into(),
            elements: vec![vec!["MS".into()]],
            segment_number: 3,
        }];

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "SG2/SG3/CTA/3139".to_string(),
                name: "Funktion des Ansprechpartners, Code".to_string(),
                ahb_status: "Muss".to_string(),
                codes: vec![],
            }],
        };

        let report = validator.validate_with_navigator(
            &segments,
            &workflow,
            &external,
            ValidationLevel::Conditions,
            &nav,
        );

        let ahb_errors: Vec<_> = report
            .by_category(ValidationCategory::Ahb)
            .filter(|i| i.severity == Severity::Error)
            .collect();
        assert_eq!(
            ahb_errors.len(),
            1,
            "Expected AHB001 error when SG3 is present but CTA missing"
        );
        assert!(ahb_errors[0].message.contains("CTA"));
    }

    #[test]
    fn test_group_scoped_code_validation_with_navigator() {
        // With a navigator, SG2/NAD is checked against {MS, MR} only,
        // and SG4/SG12/NAD is checked against {Z04, Z09} only.
        // NAD+MT in SG2 → error with allowed [MR, MS] (not the full union).
        use mig_types::navigator::GroupNavigator;

        struct TestNav;
        impl GroupNavigator for TestNav {
            fn find_segments_in_group(
                &self,
                segment_id: &str,
                group_path: &[&str],
                _instance_index: usize,
            ) -> Vec<OwnedSegment> {
                if segment_id != "NAD" {
                    return vec![];
                }
                match group_path {
                    ["SG2"] => vec![
                        OwnedSegment {
                            id: "NAD".into(),
                            elements: vec![vec!["MS".into()]],
                            segment_number: 3,
                        },
                        OwnedSegment {
                            id: "NAD".into(),
                            elements: vec![vec!["MT".into()]], // invalid in SG2
                            segment_number: 4,
                        },
                    ],
                    ["SG4", "SG12"] => vec![
                        OwnedSegment {
                            id: "NAD".into(),
                            elements: vec![vec!["Z04".into()]],
                            segment_number: 20,
                        },
                        OwnedSegment {
                            id: "NAD".into(),
                            elements: vec![vec!["Z09".into()]],
                            segment_number: 21,
                        },
                    ],
                    _ => vec![],
                }
            }
            fn find_segments_with_qualifier_in_group(
                &self,
                _: &str,
                _: usize,
                _: &str,
                _: &[&str],
                _: usize,
            ) -> Vec<OwnedSegment> {
                vec![]
            }
            fn group_instance_count(&self, group_path: &[&str]) -> usize {
                match group_path {
                    ["SG2"] | ["SG4", "SG12"] => 1,
                    _ => 0,
                }
            }
        }

        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;
        let nav = TestNav;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Absender".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MS".to_string(),
                        description: "Absender".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: "Empfaenger".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "MR".to_string(),
                        description: "Empfaenger".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG4/SG12/NAD/3035".to_string(),
                    name: "Anschlussnutzer".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "Z04".to_string(),
                        description: "Anschlussnutzer".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
                AhbFieldRule {
                    segment_path: "SG4/SG12/NAD/3035".to_string(),
                    name: "Korrespondenzanschrift".to_string(),
                    ahb_status: "X".to_string(),
                    codes: vec![AhbCodeRule {
                        value: "Z09".to_string(),
                        description: "Korrespondenzanschrift".to_string(),
                        ahb_status: "X".to_string(),
                    }],
                },
            ],
        };

        // All segments flat (for condition evaluation), navigator provides group scope.
        let all_segments = vec![
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![vec!["MS".into()]],
                segment_number: 3,
            },
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![vec!["MT".into()]],
                segment_number: 4,
            },
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![vec!["Z04".into()]],
                segment_number: 20,
            },
            OwnedSegment {
                id: "NAD".into(),
                elements: vec![vec!["Z09".into()]],
                segment_number: 21,
            },
        ];

        let report = validator.validate_with_navigator(
            &all_segments,
            &workflow,
            &external,
            ValidationLevel::Conditions,
            &nav,
        );

        let code_errors: Vec<_> = report
            .by_category(ValidationCategory::Code)
            .filter(|i| i.severity == Severity::Error)
            .collect();

        // Only one error: MT in SG2 (not allowed in {MS, MR}).
        // Z04 and Z09 are NOT checked against {MS, MR} — group-scoped.
        assert_eq!(
            code_errors.len(),
            1,
            "Expected exactly one COD002 error for MT in SG2, got: {:?}",
            code_errors
        );
        assert!(code_errors[0].message.contains("MT"));
        // Error should show only SG2's allowed codes, not the full union
        assert!(code_errors[0].message.contains("MR"));
        assert!(code_errors[0].message.contains("MS"));
        assert!(
            !code_errors[0].message.contains("Z04"),
            "SG4/SG12 codes should not leak into SG2 error"
        );
        // Field path should include the group
        assert!(
            code_errors[0]
                .field_path
                .as_deref()
                .unwrap_or("")
                .contains("SG2"),
            "Error field_path should reference SG2, got: {:?}",
            code_errors[0].field_path
        );
    }
}
