//! Automapper REST API server.
//!
//! Provides HTTP endpoints for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! coordinator discovery, and health checks. Also serves gRPC via tonic on the
//! same port and the Leptos WASM frontend as static files.

pub mod contracts;
pub mod error;
pub mod grpc;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use grpc::inspection::InspectionServiceImpl;
use grpc::inspection_proto::inspection_service_server::InspectionServiceServer;
use grpc::transform::TransformServiceImpl;
use grpc::transform_proto::transform_service_server::TransformServiceServer;

/// Build the complete Axum router with REST routes, gRPC services, and middleware.
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

    // Build gRPC services
    let transform_service =
        TransformServiceServer::new(TransformServiceImpl::new(state.registry.clone()));
    let inspection_service =
        InspectionServiceServer::new(InspectionServiceImpl::new(state.registry.clone()));

    // Build the gRPC router (tonic services exposed via Axum)
    let grpc_router = tonic::service::Routes::new(transform_service)
        .add_service(inspection_service)
        .into_axum_router();

    // Build REST routes with state first, then merge stateless gRPC router
    let rest_router = Router::new()
        .nest("/api/v1", routes::api_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state);

    rest_router
        .merge(grpc_router)
        .fallback_service(ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Build an HTTP-only router (no gRPC) for testing scenarios.
pub fn build_http_router(state: state::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::api_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
