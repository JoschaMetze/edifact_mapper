//! Coordinator discovery endpoints.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::contracts::coordinators::CoordinatorInfo;
use crate::error::ApiError;
use crate::state::AppState;

/// Build coordinator routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/coordinators", get(list_coordinators))
        .route("/coordinators/{message_type}", get(get_coordinator))
}

/// `GET /api/v1/coordinators` — List all available coordinators.
#[utoipa::path(
    get,
    path = "/api/v1/coordinators",
    responses(
        (status = 200, description = "List of available coordinators", body = Vec<CoordinatorInfo>),
    ),
    tag = "v1"
)]
pub(crate) async fn list_coordinators(State(state): State<AppState>) -> Json<Vec<CoordinatorInfo>> {
    let coordinators: Vec<CoordinatorInfo> = state.registry.list().into_iter().cloned().collect();
    Json(coordinators)
}

/// `GET /api/v1/coordinators/{message_type}` — Get a specific coordinator.
#[utoipa::path(
    get,
    path = "/api/v1/coordinators/{message_type}",
    params(
        ("message_type" = String, Path, description = "EDIFACT message type (e.g. UTILMD)"),
    ),
    responses(
        (status = 200, description = "Coordinator details", body = CoordinatorInfo),
        (status = 404, description = "Coordinator not found"),
    ),
    tag = "v1"
)]
pub(crate) async fn get_coordinator(
    State(state): State<AppState>,
    Path(message_type): Path<String>,
) -> Result<Json<CoordinatorInfo>, ApiError> {
    state
        .registry
        .get(&message_type)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound {
            message: format!("no coordinator for message type '{message_type}'"),
        })
}
