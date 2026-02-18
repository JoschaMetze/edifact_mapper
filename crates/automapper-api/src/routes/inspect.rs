//! EDIFACT inspection endpoints.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::contracts::inspect::{InspectRequest, InspectResponse};
use crate::error::ApiError;
use crate::state::AppState;

/// Build inspection routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/inspect/edifact", post(inspect_edifact))
}

/// `POST /api/v1/inspect/edifact` — Parse EDIFACT into a segment tree.
async fn inspect_edifact(
    State(state): State<AppState>,
    Json(request): Json<InspectRequest>,
) -> Result<Json<InspectResponse>, ApiError> {
    tracing::info!(
        "Inspecting EDIFACT content, length={}",
        request.edifact.len()
    );

    let response = state.registry.inspect_edifact(&request.edifact)?;

    tracing::info!(
        "Parsed {} segments, message_type={:?}",
        response.segment_count,
        response.message_type
    );

    Ok(Json(response))
}
