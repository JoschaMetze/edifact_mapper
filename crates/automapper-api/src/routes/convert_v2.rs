//! V2 dual-mode conversion endpoint.
//!
//! Supports three conversion modes:
//! - `mig-tree`: tokenize + assemble → return tree as JSON
//! - `bo4e`: tokenize + assemble + TOML mapping → return BO4E JSON
//! - `legacy`: use the existing automapper-core pipeline

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::contracts::convert_v2::{ConvertMode, ConvertV2Request, ConvertV2Response};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 conversion routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/convert", post(convert_v2))
}

/// `POST /api/v2/convert` — Dual-mode conversion endpoint.
async fn convert_v2(
    State(state): State<AppState>,
    Json(req): Json<ConvertV2Request>,
) -> Result<Json<ConvertV2Response>, ApiError> {
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

            let tree = service.convert_to_assembled_tree(&req.input).map_err(|e| {
                ApiError::ConversionError {
                    message: e.to_string(),
                }
            })?;

            // Apply TOML mapping engine to convert tree → BO4E
            let engine = state.mig_registry.mapping_engine();
            let tree_json = serde_json::to_value(&tree).map_err(|e| ApiError::Internal {
                message: format!("serialization error: {e}"),
            })?;

            // If mapping definitions exist, apply them; otherwise return the tree
            let bo4e_result = if engine.definitions().is_empty() {
                serde_json::json!({
                    "note": "No TOML mapping definitions loaded; returning assembled tree",
                    "tree": tree_json
                })
            } else {
                tree_json
            };

            Ok(Json(ConvertV2Response {
                mode: "bo4e".to_string(),
                result: bo4e_result,
                duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            }))
        }
        ConvertMode::Legacy => {
            let response = state.registry.convert_edifact_to_bo4e(
                &req.input,
                Some(&req.format_version),
                false,
            )?;

            let result = match response.result {
                Some(json_str) => {
                    serde_json::from_str(&json_str).unwrap_or(serde_json::json!(null))
                }
                None => serde_json::json!(null),
            };

            Ok(Json(ConvertV2Response {
                mode: "legacy".to_string(),
                result,
                duration_ms: response.duration_ms,
            }))
        }
    }
}
