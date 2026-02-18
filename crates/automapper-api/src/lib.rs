//! Automapper REST API server.
//!
//! Provides HTTP endpoints for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! coordinator discovery, and health checks. Also serves the Leptos WASM frontend
//! as static files.

pub mod contracts;
pub mod error;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Build the complete Axum router with all routes, middleware, and static file serving.
pub fn build_router(state: state::AppState) -> Router {
    build_router_with_static_dir(state, "static")
}

/// Build the router with a custom static file directory.
///
/// The static directory should contain the compiled Leptos WASM frontend.
/// In production, this is typically `./static/` next to the binary.
pub fn build_router_with_static_dir(state: state::AppState, static_dir: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::api_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .fallback_service(ServeDir::new(static_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
