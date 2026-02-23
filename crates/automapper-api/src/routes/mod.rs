//! Route handlers for the REST API.

pub mod convert_v2;
pub mod coordinators;
pub mod fixtures;
pub mod health;
pub mod inspect;
pub mod reverse_v2;

use axum::Router;

use crate::state::AppState;

/// Build all `/api/v1/*` routes.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(inspect::routes())
        .merge(coordinators::routes())
        .merge(fixtures::routes())
}

/// Build all `/api/v2/*` routes.
pub fn api_v2_routes() -> Router<AppState> {
    Router::new()
        .merge(convert_v2::routes())
        .merge(reverse_v2::routes())
}
