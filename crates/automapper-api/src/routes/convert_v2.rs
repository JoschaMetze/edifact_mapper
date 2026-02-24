//! V2 conversion endpoint.
//!
//! Supports two conversion modes:
//! - `mig-tree`: tokenize + assemble → return tree as JSON
//! - `bo4e`: tokenize + split messages + per-message PID detection + assemble + TOML mapping → return hierarchical `Interchange` JSON

use std::collections::HashSet;

use axum::extract::{Query, State};
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::assembler::Assembler;
use mig_assembly::pid_detect::detect_pid;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;

use crate::contracts::convert_v2::{
    ConvertMode, ConvertV2Query, ConvertV2Request, ConvertV2Response,
};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 conversion routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/convert", post(convert_v2))
}

/// `POST /api/v2/convert` — MIG-driven conversion endpoint.
///
/// Query parameters:
/// - `enrich_codes` (bool, default `true`): When `false`, code fields are emitted
///   as plain strings instead of `{"code": "...", "meaning": "..."}` objects.
#[utoipa::path(
    post,
    path = "/api/v2/convert",
    params(ConvertV2Query),
    request_body = ConvertV2Request,
    responses(
        (status = 200, description = "Conversion result", body = ConvertV2Response),
        (status = 400, description = "Bad request"),
        (status = 422, description = "Conversion error"),
    ),
    tag = "v2"
)]
pub(crate) async fn convert_v2(
    State(state): State<AppState>,
    Query(query): Query<ConvertV2Query>,
    Json(req): Json<ConvertV2Request>,
) -> Result<Json<ConvertV2Response>, ApiError> {
    let enrich_codes = query.enrich_codes.unwrap_or(true);
    let start = std::time::Instant::now();

    match req.mode {
        ConvertMode::MigTree => {
            let service = state
                .mig_registry
                .service(&req.format_version)
                .ok_or_else(|| ApiError::BadRequest {
                    message: format!(
                        "No MIG service available for format version '{}'",
                        req.format_version
                    ),
                })?;

            let tree =
                service
                    .convert_to_tree(&req.input)
                    .map_err(|e| ApiError::ConversionError {
                        message: e.to_string(),
                    })?;

            Ok(Json(ConvertV2Response {
                mode: "mig-tree".to_string(),
                result: serde_json::json!({ "tree": tree }),
                duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            }))
        }
        ConvertMode::Bo4e => {
            let service = state
                .mig_registry
                .service(&req.format_version)
                .ok_or_else(|| ApiError::BadRequest {
                    message: format!(
                        "No MIG service available for format version '{}'",
                        req.format_version
                    ),
                })?;

            // Step 1: Tokenize
            let segments =
                parse_to_segments(req.input.as_bytes()).map_err(|e| ApiError::ConversionError {
                    message: format!("tokenization error: {e}"),
                })?;

            // Step 2: Split into messages
            let chunks =
                mig_assembly::split_messages(segments).map_err(|e| ApiError::ConversionError {
                    message: format!("message splitting error: {e}"),
                })?;

            // Step 3: Extract envelope nachrichtendaten
            let nachrichtendaten = mig_bo4e::model::extract_nachrichtendaten(&chunks.envelope);

            // Step 4: Process each message
            // TODO: detect message type/variant from UNH segment
            let msg_variant = "UTILMD_Strom";
            let mut nachrichten = Vec::new();

            for (msg_idx, msg_chunk) in chunks.messages.iter().enumerate() {
                let all_segments = msg_chunk.all_segments();

                // Detect PID from this message's segments
                let pid = detect_pid(&all_segments).map_err(|e| ApiError::ConversionError {
                    message: format!("PID detection error in message {msg_idx}: {e}"),
                })?;

                // Look up AHB for PID segment numbers
                let ahb = state
                    .mig_registry
                    .ahb_schema(&req.format_version, msg_variant)
                    .ok_or_else(|| ApiError::Internal {
                        message: format!(
                            "No AHB schema available for {}/{}",
                            req.format_version, msg_variant
                        ),
                    })?;

                let workflow = ahb.workflows.iter().find(|w| w.id == pid).ok_or_else(|| {
                    ApiError::ConversionError {
                        message: format!("PID {pid} not found in AHB (message {msg_idx})"),
                    }
                })?;

                let ahb_numbers: HashSet<String> =
                    workflow.segment_numbers.iter().cloned().collect();

                // Filter MIG for this PID and assemble
                let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);
                let assembler = Assembler::new(&filtered_mig);
                let tree = assembler.assemble_generic(&all_segments).map_err(|e| {
                    ApiError::ConversionError {
                        message: format!("assembly error in message {msg_idx}: {e}"),
                    }
                })?;

                // Load split engines (message-level + transaction-level)
                let (msg_engine, tx_engine) = state
                    .mig_registry
                    .mapping_engines_split(&req.format_version, msg_variant, &pid)
                    .ok_or_else(|| ApiError::Internal {
                        message: format!(
                            "No mapping engines for {}/{}/pid_{}",
                            req.format_version, msg_variant, pid
                        ),
                    })?;

                // Map with split engines into hierarchical result
                let mapped = mig_bo4e::MappingEngine::map_interchange(
                    msg_engine,
                    tx_engine,
                    &tree,
                    "SG4",
                    enrich_codes,
                );

                // Extract UNH fields
                let (unh_referenz, nachrichten_typ) =
                    mig_bo4e::model::extract_unh_fields(&msg_chunk.unh);

                nachrichten.push(mig_bo4e::Nachricht {
                    unh_referenz,
                    nachrichten_typ,
                    stammdaten: mapped.stammdaten,
                    transaktionen: mapped.transaktionen,
                });
            }

            let interchange = mig_bo4e::Interchange {
                nachrichtendaten,
                nachrichten,
            };

            Ok(Json(ConvertV2Response {
                mode: "bo4e".to_string(),
                result: serde_json::to_value(&interchange).unwrap_or_default(),
                duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            }))
        }
    }
}
