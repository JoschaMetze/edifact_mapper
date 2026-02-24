//! V2 reverse endpoint: BO4E → EDIFACT.
//!
//! Accepts BO4E JSON at interchange/nachricht/transaktion level
//! and converts back to an EDIFACT string or MIG tree.

use std::collections::HashSet;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::disassembler::Disassembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;

use crate::contracts::reverse_v2::{
    normalize_to_interchange, ReverseMode, ReverseV2Request, ReverseV2Response,
};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 reverse routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/reverse", post(reverse_v2))
}

/// `POST /api/v2/reverse` — BO4E to EDIFACT reverse conversion.
async fn reverse_v2(
    State(state): State<AppState>,
    Json(req): Json<ReverseV2Request>,
) -> Result<Json<ReverseV2Response>, ApiError> {
    let start = std::time::Instant::now();

    // Step 1: Normalize input to Interchange
    let interchange = normalize_to_interchange(&req.input, &req.level, req.envelope.as_ref())
        .map_err(|e| ApiError::BadRequest {
            message: format!("Input normalization error: {e}"),
        })?;

    // Step 2: Get MIG service for the format version
    let service = state
        .mig_registry
        .service(&req.format_version)
        .ok_or_else(|| ApiError::BadRequest {
            message: format!(
                "No MIG service available for format version '{}'",
                req.format_version
            ),
        })?;

    // TODO: detect message type/variant from nachrichtenTyp
    let msg_variant = "UTILMD_Strom";

    // Step 3: Reconstruct envelope segments
    let unb = mig_bo4e::model::rebuild_unb(&interchange.nachrichtendaten);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let mut all_edifact_parts: Vec<String> = Vec::new();

    // UNA + UNB
    let una_str = delimiters.to_una_string();
    let unb_segments = vec![mig_assembly::disassembler::DisassembledSegment {
        tag: unb.id.clone(),
        elements: unb.elements.clone(),
    }];
    let unb_str = render_edifact(&unb_segments, &delimiters);

    // Step 4: Process each message
    for nachricht in &interchange.nachrichten {
        // Extract PID from first transaction's transaktionsdaten
        let pid = nachricht
            .transaktionen
            .first()
            .and_then(|tx| tx.transaktionsdaten.get("pruefidentifikator"))
            .and_then(|v| {
                // Handle both plain string and enriched {"code": "55001", "meaning": "..."} formats
                v.as_str()
                    .or_else(|| v.get("code").and_then(|c| c.as_str()))
            })
            .ok_or_else(|| ApiError::BadRequest {
                message: "No pruefidentifikator found in transaktionsdaten".to_string(),
            })?;

        // Look up AHB for PID → segment numbers → filtered MIG
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
        let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);

        // Load split engines
        let (msg_engine, tx_engine) = state
            .mig_registry
            .mapping_engines_split(&req.format_version, msg_variant, pid)
            .ok_or_else(|| ApiError::Internal {
                message: format!(
                    "No mapping engines for {}/{}/pid_{}",
                    req.format_version, msg_variant, pid
                ),
            })?;

        // Build MappedMessage from Nachricht
        let mapped = mig_bo4e::model::MappedMessage {
            stammdaten: nachricht.stammdaten.clone(),
            transaktionen: nachricht.transaktionen.clone(),
        };

        // Reverse map → AssembledTree
        let tree =
            mig_bo4e::MappingEngine::map_interchange_reverse(msg_engine, tx_engine, &mapped, "SG4");

        match req.mode {
            ReverseMode::MigTree => {
                // Return tree JSON for this message
                return Ok(Json(ReverseV2Response {
                    mode: "mig-tree".to_string(),
                    result: serde_json::to_value(&tree).unwrap_or_default(),
                    duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                }));
            }
            ReverseMode::Edifact => {
                // Disassemble tree → ordered segments
                let disassembler = Disassembler::new(&filtered_mig);
                let dis_segments = disassembler.disassemble(&tree);

                // Build UNH + body + UNT
                let unh = mig_bo4e::model::rebuild_unh(
                    &nachricht.unh_referenz,
                    &nachricht.nachrichten_typ,
                );
                let unh_dis = mig_assembly::disassembler::DisassembledSegment {
                    tag: unh.id.clone(),
                    elements: unh.elements.clone(),
                };

                // Segment count = UNH + body segments + UNT
                let seg_count = 1 + dis_segments.len() + 1;
                let unt = mig_bo4e::model::rebuild_unt(seg_count, &nachricht.unh_referenz);
                let unt_dis = mig_assembly::disassembler::DisassembledSegment {
                    tag: unt.id.clone(),
                    elements: unt.elements.clone(),
                };

                let mut msg_segments = vec![unh_dis];
                msg_segments.extend(dis_segments);
                msg_segments.push(unt_dis);

                all_edifact_parts.push(render_edifact(&msg_segments, &delimiters));
            }
        }
    }

    // Step 5: Build UNZ and concatenate
    let interchange_ref = interchange
        .nachrichtendaten
        .get("interchangeRef")
        .and_then(|v| v.as_str())
        .unwrap_or("00000");
    let unz = mig_bo4e::model::rebuild_unz(interchange.nachrichten.len(), interchange_ref);
    let unz_segments = vec![mig_assembly::disassembler::DisassembledSegment {
        tag: unz.id.clone(),
        elements: unz.elements.clone(),
    }];

    let mut full_edifact = una_str;
    full_edifact.push_str(&unb_str);
    for part in &all_edifact_parts {
        full_edifact.push_str(part);
    }
    full_edifact.push_str(&render_edifact(&unz_segments, &delimiters));

    Ok(Json(ReverseV2Response {
        mode: "edifact".to_string(),
        result: serde_json::Value::String(full_edifact),
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}
