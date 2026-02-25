//! V2 validation endpoint.
//!
//! Validates EDIFACT content against AHB rules, returning a `ValidationReport`
//! with structure diagnostics and condition evaluation results.

use std::collections::HashSet;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::assembler::Assembler;
use mig_assembly::navigator::AssembledTreeNavigator;
use mig_assembly::pid_detect::detect_pid;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;

use crate::contracts::validate_v2::{ValidateV2Request, ValidateV2Response};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 validation routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/validate", post(validate_v2))
}

/// `POST /api/v2/validate` -- validate EDIFACT against AHB rules.
///
/// Parses the EDIFACT content, detects the PID, assembles with diagnostics,
/// and runs validation. Returns a `ValidationReport` as JSON.
#[utoipa::path(
    post,
    path = "/api/v2/validate",
    request_body = ValidateV2Request,
    responses(
        (status = 200, description = "Validation report", body = ValidateV2Response),
        (status = 400, description = "Bad request"),
        (status = 422, description = "Validation setup error"),
    ),
    tag = "v2"
)]
pub(crate) async fn validate_v2(
    State(state): State<AppState>,
    Json(req): Json<ValidateV2Request>,
) -> Result<Json<ValidateV2Response>, ApiError> {
    let start = std::time::Instant::now();

    // Parse the validation level from the string
    let level = match req.level.as_str() {
        "structure" => automapper_validation::ValidationLevel::Structure,
        "conditions" => automapper_validation::ValidationLevel::Conditions,
        _ => automapper_validation::ValidationLevel::Full,
    };

    // Step 1: Tokenize EDIFACT
    let segments =
        parse_to_segments(req.input.as_bytes()).map_err(|e| ApiError::ConversionError {
            message: format!("tokenization error: {e}"),
        })?;

    // Step 2: Split into messages
    let chunks = mig_assembly::split_messages(segments).map_err(|e| ApiError::ConversionError {
        message: format!("message splitting error: {e}"),
    })?;

    // Step 3: Process the first message (validate one message at a time)
    let msg_chunk = chunks
        .messages
        .first()
        .ok_or_else(|| ApiError::BadRequest {
            message: "No messages found in EDIFACT content".to_string(),
        })?;

    let all_segments = msg_chunk.all_segments();

    // Step 4: Detect PID
    let pid = detect_pid(&all_segments).map_err(|e| ApiError::ConversionError {
        message: format!("PID detection error: {e}"),
    })?;

    // Step 5: Load AHB schema from registry
    // TODO: detect message type/variant from UNH segment
    let msg_variant = "UTILMD_Strom";
    let ahb = state
        .mig_registry
        .ahb_schema(&req.format_version, msg_variant)
        .ok_or_else(|| ApiError::Internal {
            message: format!(
                "No AHB schema available for {}/{}",
                req.format_version, msg_variant
            ),
        })?;

    // Step 6: Build AhbWorkflow from schema
    let workflow =
        crate::validation_bridge::ahb_workflow_from_schema(ahb, &pid).ok_or_else(|| {
            ApiError::ConversionError {
                message: format!("PID {pid} not found in AHB"),
            }
        })?;

    // Step 7: Load MIG service, filter for PID, assemble with diagnostics
    let service = state
        .mig_registry
        .service(&req.format_version)
        .ok_or_else(|| ApiError::BadRequest {
            message: format!(
                "No MIG service available for format version '{}'",
                req.format_version
            ),
        })?;

    let ahb_workflow =
        ahb.workflows
            .iter()
            .find(|w| w.id == pid)
            .ok_or_else(|| ApiError::ConversionError {
                message: format!("PID {pid} not found in AHB workflows"),
            })?;

    let ahb_numbers: HashSet<String> = ahb_workflow.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);
    let assembler = Assembler::new(&filtered_mig);
    let (tree, structure_diagnostics) = assembler.assemble_with_diagnostics(&all_segments);

    // Step 8: Build external condition provider
    let external: Box<dyn automapper_validation::eval::ExternalConditionProvider> =
        if let Some(ref conditions) = req.external_conditions {
            Box::new(automapper_validation::MapExternalProvider::new(
                conditions.clone(),
            ))
        } else {
            Box::new(automapper_validation::eval::NoOpExternalProvider)
        };

    // Step 9: Create condition evaluator and validator
    let evaluator = automapper_validation::UtilmdConditionEvaluatorFV2504::default();
    let validator = automapper_validation::EdifactValidator::new(evaluator);

    // Step 10: Run validation with group navigator
    let navigator = AssembledTreeNavigator::new(&tree);
    let mut report = validator.validate_with_navigator(
        &all_segments,
        &workflow,
        external.as_ref(),
        level,
        &navigator,
    );

    // Step 11: Add structure diagnostics as ValidationIssues
    for diag in structure_diagnostics {
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Warning,
            automapper_validation::ValidationCategory::Structure,
            automapper_validation::ErrorCodes::UNEXPECTED_SEGMENT,
            diag.message,
        ));
    }

    // Step 12: Serialize and return
    let report_json = serde_json::to_value(&report).map_err(|e| ApiError::Internal {
        message: format!("Failed to serialize validation report: {e}"),
    })?;

    Ok(Json(ValidateV2Response {
        report: report_json,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_evaluator_returns_unknown_for_unimplemented_conditions() {
        let evaluator = automapper_validation::UtilmdConditionEvaluatorFV2504::default();

        let segments = vec![];
        let external = automapper_validation::eval::NoOpExternalProvider;
        let ctx = automapper_validation::EvaluationContext::new("55001", &external, &segments);

        use automapper_validation::ConditionEvaluator;
        // Generated evaluator returns Unknown for conditions not yet implemented
        assert_eq!(
            evaluator.evaluate(999, &ctx),
            automapper_validation::ConditionResult::Unknown
        );
        assert_eq!(evaluator.message_type(), "UTILMD");
        assert_eq!(evaluator.format_version(), "FV2504");
    }

    #[test]
    fn parse_validation_level_from_string() {
        // Test the level parsing logic
        let cases = vec![
            (
                "structure",
                automapper_validation::ValidationLevel::Structure,
            ),
            (
                "conditions",
                automapper_validation::ValidationLevel::Conditions,
            ),
            ("full", automapper_validation::ValidationLevel::Full),
            ("unknown", automapper_validation::ValidationLevel::Full),
            ("", automapper_validation::ValidationLevel::Full),
        ];

        for (input, expected) in cases {
            let level = match input {
                "structure" => automapper_validation::ValidationLevel::Structure,
                "conditions" => automapper_validation::ValidationLevel::Conditions,
                _ => automapper_validation::ValidationLevel::Full,
            };
            assert_eq!(level, expected, "Failed for input: {input:?}");
        }
    }
}
