use automapper_api::state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = AppState::new();

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());
    let app = automapper_api::build_router_with_static_dir(state, &static_dir);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind");

    tracing::info!(
        "automapper-api listening on {}, static_dir={}",
        bind_addr,
        static_dir
    );

    axum::serve(listener, app).await.expect("server error");
}
