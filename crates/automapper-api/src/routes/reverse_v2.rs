//! V2 reverse endpoint: BO4E → EDIFACT.
//!
//! Accepts BO4E JSON at interchange/nachricht/transaktion level
//! and converts back to an EDIFACT string or MIG tree.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::contracts::reverse_v2::{
    normalize_to_interchange, ReverseMode, ReverseV2Request, ReverseV2Response,
};
use crate::error::ApiError;
use crate::routes::reverse_pipeline::{
    extract_pid, load_reverse_context, render_full_edifact, render_message_segments,
    reverse_map_nachricht,
};
use crate::state::AppState;

/// Build v2 reverse routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/reverse", post(reverse_v2))
}

/// `POST /api/v2/reverse` — BO4E to EDIFACT reverse conversion.
#[utoipa::path(
    post,
    path = "/api/v2/reverse",
    request_body = ReverseV2Request,
    responses(
        (status = 200, description = "Reverse conversion result", body = ReverseV2Response),
        (status = 400, description = "Bad request"),
        (status = 422, description = "Conversion error"),
    ),
    tag = "v2"
)]
pub(crate) async fn reverse_v2(
    State(state): State<AppState>,
    Json(req): Json<ReverseV2Request>,
) -> Result<Json<ReverseV2Response>, ApiError> {
    let start = std::time::Instant::now();

    // Step 1: Normalize input to Interchange
    let interchange = normalize_to_interchange(&req.input, &req.level, req.envelope.as_ref())
        .map_err(|e| ApiError::BadRequest {
            message: format!("Input normalization error: {e}"),
        })?;

    // TODO: detect message type/variant from nachrichtenTyp
    let msg_variant = "UTILMD_Strom";
    let delimiters = edifact_types::EdifactDelimiters::default();

    let mut all_edifact_parts: Vec<String> = Vec::new();

    // Step 2: Process each message
    for nachricht in &interchange.nachrichten {
        let pid = extract_pid(nachricht)?;
        let ctx = load_reverse_context(&state, &req.format_version, msg_variant, pid)?;
        let tree = reverse_map_nachricht(&ctx, nachricht);

        match req.mode {
            ReverseMode::MigTree => {
                return Ok(Json(ReverseV2Response {
                    mode: "mig-tree".to_string(),
                    result: serde_json::to_value(&tree).unwrap_or_default(),
                    duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                }));
            }
            ReverseMode::Edifact => {
                all_edifact_parts.push(render_message_segments(
                    &ctx,
                    nachricht,
                    &tree,
                    &delimiters,
                ));
            }
        }
    }

    // Step 3: Wrap with envelope
    let full_edifact = render_full_edifact(&interchange, &all_edifact_parts);

    Ok(Json(ReverseV2Response {
        mode: "edifact".to_string(),
        result: serde_json::Value::String(full_edifact),
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}
