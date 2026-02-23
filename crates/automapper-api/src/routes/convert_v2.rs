//! V2 conversion endpoint.
//!
//! Supports two conversion modes:
//! - `mig-tree`: tokenize + assemble → return tree as JSON
//! - `bo4e`: tokenize + assemble + TOML mapping → return BO4E JSON

use std::collections::HashSet;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::assembler::Assembler;
use mig_assembly::pid_detect::detect_pid;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;

use crate::contracts::convert_v2::{ConvertMode, ConvertV2Request, ConvertV2Response};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 conversion routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/convert", post(convert_v2))
}

/// Convert PascalCase to camelCase: "Marktlokation" → "marktlokation",
/// "ProduktpaketPriorisierung" → "produktpaketPriorisierung".
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
    }
}

/// `POST /api/v2/convert` — MIG-driven conversion endpoint.
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

            // Step 1: Tokenize
            let segments =
                parse_to_segments(req.input.as_bytes()).map_err(|e| ApiError::ConversionError {
                    message: format!("tokenization error: {e}"),
                })?;

            // Step 2: Detect PID
            let pid = detect_pid(&segments).map_err(|e| ApiError::ConversionError {
                message: format!("PID detection error: {e}"),
            })?;

            // Step 3: Look up AHB for PID segment numbers
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

            let workflow = ahb.workflows.iter().find(|w| w.id == pid).ok_or_else(|| {
                ApiError::ConversionError {
                    message: format!("PID {pid} not found in AHB"),
                }
            })?;

            let ahb_numbers: HashSet<String> = workflow.segment_numbers.iter().cloned().collect();

            // Step 4: Filter MIG for this PID and assemble
            let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);
            let assembler = Assembler::new(&filtered_mig);
            let tree =
                assembler
                    .assemble_generic(&segments)
                    .map_err(|e| ApiError::ConversionError {
                        message: format!("assembly error: {e}"),
                    })?;

            // Step 5: Apply TOML mappings
            let engine = state
                .mig_registry
                .mapping_engine_for_pid(&req.format_version, msg_variant, &pid)
                .ok_or_else(|| ApiError::Internal {
                    message: format!(
                        "No TOML mappings available for {}/{}/pid_{}",
                        req.format_version, msg_variant, pid
                    ),
                })?;

            let entities = engine.map_all_forward(&tree);

            // Restructure: extract Prozessdaten as transaktionsdaten, rest goes into stammdaten
            let mut stammdaten = serde_json::Map::new();
            let mut transaktionsdaten = serde_json::Value::Null;

            if let Some(obj) = entities.as_object() {
                for (key, value) in obj {
                    if key == "Prozessdaten" {
                        transaktionsdaten = value.clone();
                    } else {
                        stammdaten.insert(to_camel_case(key), value.clone());
                    }
                }
            }

            Ok(Json(ConvertV2Response {
                mode: "bo4e".to_string(),
                result: serde_json::json!({
                    "pid": pid,
                    "formatVersion": req.format_version,
                    "transaktionsdaten": transaktionsdaten,
                    "stammdaten": stammdaten,
                }),
                duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            }))
        }
    }
}
