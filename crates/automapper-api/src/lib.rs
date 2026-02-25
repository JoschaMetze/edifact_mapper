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
pub mod validation_bridge;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use grpc::inspection::InspectionServiceImpl;
use grpc::inspection_proto::inspection_service_server::InspectionServiceServer;
use grpc::transform::TransformServiceImpl;
use grpc::transform_proto::transform_service_server::TransformServiceServer;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "EDIFACT BO4E Automapper API",
        version = "2.0.0",
        description = "Bidirectional EDIFACT <-> BO4E conversion for the German energy market"
    ),
    paths(
        routes::health::health_check,
        routes::inspect::inspect_edifact,
        routes::coordinators::list_coordinators,
        routes::coordinators::get_coordinator,
        routes::fixtures::list_fixtures,
        routes::fixtures::get_fixture,
        routes::convert_v2::convert_v2,
        routes::reverse_v2::reverse_v2,
        routes::validate_v2::validate_v2,
    ),
    tags(
        (name = "health", description = "Service health"),
        (name = "v1", description = "V1 endpoints — inspection, coordinators, fixtures"),
        (name = "v2", description = "V2 endpoints — MIG-driven EDIFACT ↔ BO4E conversion"),
    )
)]
struct ApiDoc;

/// Build the Swagger UI router in its own stack frame to avoid stack overflow
/// from the large embedded static assets in debug builds.
#[inline(never)]
fn swagger_ui_router() -> Router {
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into()
}

/// Build the complete Axum router with REST routes, gRPC services, and middleware.
///
/// Does not include Swagger UI (to keep the async future small enough for
/// tokio worker threads in tests). Use [`build_router_with_static_dir`] for
/// the full production router.
pub fn build_router(state: state::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let grpc_router = build_grpc_router(&state);

    let rest_router = Router::new()
        .nest("/api/v1", routes::api_routes())
        .nest("/api/v2", routes::api_v2_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state);

    rest_router
        .merge(grpc_router)
        .fallback_service(ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Build the router with a custom static file directory and Swagger UI.
///
/// This is the full production router including interactive API documentation
/// at `/swagger-ui/` and the OpenAPI spec at `/api-docs/openapi.json`.
pub fn build_router_with_static_dir(state: state::AppState, static_dir: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let grpc_router = build_grpc_router(&state);

    let rest_router = Router::new()
        .nest("/api/v1", routes::api_routes())
        .nest("/api/v2", routes::api_v2_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state);

    rest_router
        .merge(swagger_ui_router())
        .merge(grpc_router)
        .fallback_service(ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Build an HTTP-only router (no gRPC, no Swagger UI) for testing scenarios.
pub fn build_http_router(state: state::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::api_routes())
        .nest("/api/v2", routes::api_v2_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

fn build_grpc_router(state: &state::AppState) -> Router {
    let transform_service =
        TransformServiceServer::new(TransformServiceImpl::new(state.registry.clone()));
    let inspection_service =
        InspectionServiceServer::new(InspectionServiceImpl::new(state.registry.clone()));

    tonic::service::Routes::new(transform_service)
        .add_service(inspection_service)
        .into_axum_router()
}
