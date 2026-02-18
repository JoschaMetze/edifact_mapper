//! Route handlers for the REST API.

pub mod convert;
pub mod coordinators;
pub mod health;
pub mod inspect;

use axum::Router;

use crate::state::AppState;

/// Build all `/api/v1/*` routes.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(convert::routes())
        .merge(inspect::routes())
        .merge(coordinators::routes())
}
