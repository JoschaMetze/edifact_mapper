//! Health check endpoint.

use axum::extract::State;
use axum::Json;

use crate::contracts::health::HealthResponse;
use crate::state::AppState;

/// `GET /health` — Returns service health status.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health status", body = HealthResponse),
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let coordinators: Vec<String> = state
        .registry
        .list()
        .iter()
        .map(|c| c.message_type.clone())
        .collect();

    Json(HealthResponse {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        available_coordinators: coordinators,
        uptime_seconds: state.startup.elapsed().as_secs_f64(),
    })
}
