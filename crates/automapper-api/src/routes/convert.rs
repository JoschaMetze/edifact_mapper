//! EDIFACT <-> BO4E conversion endpoints.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::contracts::convert::{ConvertRequest, ConvertResponse};
use crate::error::ApiError;
use crate::state::AppState;

/// Build conversion routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/convert/edifact-to-bo4e", post(convert_edifact_to_bo4e))
        .route("/convert/bo4e-to-edifact", post(convert_bo4e_to_edifact))
}

/// `POST /api/v1/convert/edifact-to-bo4e` — Convert EDIFACT to BO4E JSON.
async fn convert_edifact_to_bo4e(
    State(state): State<AppState>,
    Json(request): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    tracing::info!(
        "Converting EDIFACT to BO4E, content_length={}, format_version={:?}",
        request.content.len(),
        request.format_version
    );

    let response = state.registry.convert_edifact_to_bo4e(
        &request.content,
        request.format_version.as_deref(),
        request.include_trace,
    )?;

    Ok(Json(response))
}

/// `POST /api/v1/convert/bo4e-to-edifact` — Convert BO4E JSON to EDIFACT.
async fn convert_bo4e_to_edifact(
    State(state): State<AppState>,
    Json(request): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    tracing::info!(
        "Converting BO4E to EDIFACT, content_length={}, format_version={:?}",
        request.content.len(),
        request.format_version
    );

    let response = state
        .registry
        .convert_bo4e_to_edifact(&request.content, request.format_version.as_deref())?;

    Ok(Json(response))
}
