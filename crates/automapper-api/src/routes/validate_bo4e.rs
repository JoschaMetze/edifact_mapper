//! V2 validate-bo4e endpoint.
//!
//! Accepts BO4E JSON, reverse-maps to EDIFACT, validates against AHB rules,
//! and enriches validation errors with BO4E field paths.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::assembler::Assembler;
use mig_assembly::navigator::AssembledTreeNavigator;
use mig_assembly::tokenize::parse_to_segments;

use crate::contracts::reverse_v2::normalize_to_interchange;
use crate::contracts::validate_bo4e::{ValidateBo4eRequest, ValidateBo4eResponse};
use crate::error::ApiError;
use crate::routes::reverse_pipeline::{
    extract_pid, load_reverse_context, render_full_edifact, render_message_segments,
    reverse_map_nachricht,
};
use crate::state::AppState;

/// Build v2 validate-bo4e routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/validate-bo4e", post(validate_bo4e))
}

/// `POST /api/v2/validate-bo4e` — validate BO4E JSON via reverse mapping.
///
/// Chains: normalize → reverse-map → render EDIFACT → tokenize → assemble →
/// validate → enrich with BO4E field paths.
#[utoipa::path(
    post,
    path = "/api/v2/validate-bo4e",
    request_body = ValidateBo4eRequest,
    responses(
        (status = 200, description = "Validation report with BO4E paths", body = ValidateBo4eResponse),
        (status = 400, description = "Bad request"),
        (status = 422, description = "Conversion or validation error"),
    ),
    tag = "v2"
)]
pub(crate) async fn validate_bo4e(
    State(state): State<AppState>,
    Json(req): Json<ValidateBo4eRequest>,
) -> Result<Json<ValidateBo4eResponse>, ApiError> {
    let start = std::time::Instant::now();

    // Step 1: Normalize BO4E input to Interchange
    let interchange = normalize_to_interchange(&req.input, &req.level, req.envelope.as_ref())
        .map_err(|e| ApiError::BadRequest {
            message: format!("Input normalization error: {e}"),
        })?;

    // TODO: detect message type/variant from nachrichtenTyp
    let msg_variant = "UTILMD_Strom";

    // Step 2: Process the first message
    let nachricht = interchange
        .nachrichten
        .first()
        .ok_or_else(|| ApiError::BadRequest {
            message: "No messages (nachrichten) in input".to_string(),
        })?;

    let pid = extract_pid(nachricht)?;
    let ctx = load_reverse_context(&state, &req.format_version, msg_variant, pid)?;

    // Step 3: Reverse map BO4E → AssembledTree → EDIFACT
    let tree = reverse_map_nachricht(&ctx, nachricht);
    let delimiters = edifact_types::EdifactDelimiters::default();
    let msg_edifact = render_message_segments(&ctx, nachricht, &tree, &delimiters);
    let full_edifact = render_full_edifact(&interchange, &[msg_edifact]);

    // Step 4: Re-tokenize the rendered EDIFACT
    let segments =
        parse_to_segments(full_edifact.as_bytes()).map_err(|e| ApiError::ConversionError {
            message: format!("Re-tokenization error: {e}"),
        })?;

    let chunks = mig_assembly::split_messages(segments).map_err(|e| ApiError::ConversionError {
        message: format!("Message splitting error: {e}"),
    })?;

    let msg_chunk = chunks
        .messages
        .first()
        .ok_or_else(|| ApiError::ConversionError {
            message: "No messages found after re-tokenization".to_string(),
        })?;

    let all_segments = msg_chunk.all_segments();

    // Step 5: Assemble with diagnostics for validation
    let assembler = Assembler::new(&ctx.filtered_mig);
    let (assembled_tree, structure_diagnostics) =
        assembler.assemble_with_diagnostics(&all_segments);

    // Step 6: Build AhbWorkflow and validate
    // PID existence was already verified by load_reverse_context
    let workflow = crate::validation_bridge::ahb_workflow_from_schema(ctx.ahb, pid)
        .expect("PID verified in load_reverse_context");

    let external: Box<dyn automapper_validation::eval::ExternalConditionProvider> =
        if let Some(ref conditions) = req.external_conditions {
            Box::new(automapper_validation::MapExternalProvider::new(
                conditions.clone(),
            ))
        } else {
            Box::new(automapper_validation::eval::NoOpExternalProvider)
        };

    let evaluator = automapper_validation::UtilmdConditionEvaluatorFV2504::default();
    let validator = automapper_validation::EdifactValidator::new(evaluator);

    let navigator = AssembledTreeNavigator::new(&assembled_tree);
    let mut report = validator.validate_with_navigator(
        &all_segments,
        &workflow,
        external.as_ref(),
        req.validation_level,
        &navigator,
    );

    // Add structure diagnostics
    for diag in structure_diagnostics {
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Warning,
            automapper_validation::ValidationCategory::Structure,
            automapper_validation::ErrorCodes::UNEXPECTED_SEGMENT,
            diag.message,
        ));
    }

    // Step 7: Build Bo4eFieldIndex and enrich report with BO4E paths
    let mut all_defs: Vec<_> = ctx.msg_engine.definitions().to_vec();
    all_defs.extend(ctx.tx_engine.definitions().iter().cloned());
    let field_index = mig_bo4e::Bo4eFieldIndex::build(&all_defs, &ctx.filtered_mig);
    report.enrich_bo4e_paths(|path| field_index.resolve(path));

    // Step 8: Serialize and return
    let report_json = serde_json::to_value(&report).map_err(|e| ApiError::Internal {
        message: format!("Failed to serialize validation report: {e}"),
    })?;

    Ok(Json(ValidateBo4eResponse {
        report: report_json,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}
