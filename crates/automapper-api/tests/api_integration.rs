//! Integration tests for the REST API.
//!
//! Uses tower::ServiceExt to call the router directly without a running server.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::convert::ConvertResponse;
use automapper_api::contracts::coordinators::CoordinatorInfo;
use automapper_api::contracts::health::HealthResponse;
use automapper_api::contracts::inspect::InspectResponse;
use automapper_api::state::AppState;

fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_router(state)
}

// --- Health ---

#[tokio::test]
async fn test_health_returns_200() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();

    assert!(health.healthy);
    assert!(!health.version.is_empty());
    assert!(!health.available_coordinators.is_empty());
}

// --- Coordinators ---

#[tokio::test]
async fn test_list_coordinators_returns_200() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/coordinators")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let coordinators: Vec<CoordinatorInfo> = serde_json::from_slice(&body).unwrap();

    assert!(!coordinators.is_empty());
    assert!(coordinators.iter().any(|c| c.message_type == "UTILMD"));
}

// --- Inspect ---

#[tokio::test]
async fn test_inspect_edifact_returns_segments() {
    let app = app();

    let edifact = r#"UNH+1+UTILMD:D:11A:UN:5.2e'BGM+E01+DOC001'UNT+3+1'"#;
    let body = serde_json::json!({ "edifact": edifact });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inspect/edifact")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let inspect: InspectResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(inspect.segment_count, 3);
    assert_eq!(inspect.segments[0].tag, "UNH");
    assert_eq!(inspect.segments[1].tag, "BGM");
    assert_eq!(inspect.segments[2].tag, "UNT");
    assert_eq!(inspect.message_type, Some("UTILMD".to_string()));
}

#[tokio::test]
async fn test_inspect_empty_edifact_returns_400() {
    let app = app();

    let body = serde_json::json!({ "edifact": "" });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inspect/edifact")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Empty content should return 400 or error
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

// --- Convert EDIFACT to BO4E ---

#[tokio::test]
async fn test_convert_edifact_to_bo4e_accepts_valid_json() {
    let app = app();

    // Minimal EDIFACT that at minimum exercises the endpoint
    let body = serde_json::json!({
        "content": "UNB+UNOC:3+sender+receiver+231215:1200+ref001'UNH+1+UTILMD:D:11A:UN:5.2e'BGM+E01+DOC001'UNT+3+1'UNZ+1+ref001'",
        "include_trace": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/convert/edifact-to-bo4e")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 200 (success) or 422 (conversion error) — not 500
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: ConvertResponse = serde_json::from_slice(&body).unwrap();

    // Response should be well-formed regardless of success
    assert!(resp.duration_ms >= 0.0);
}

// --- Convert BO4E to EDIFACT ---

#[tokio::test]
async fn test_convert_bo4e_to_edifact_rejects_invalid_json() {
    let app = app();

    let body = serde_json::json!({
        "content": "this is not valid json for bo4e",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/convert/bo4e-to-edifact")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return an error, not 500
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

// --- CORS ---

#[tokio::test]
async fn test_cors_preflight_returns_200() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/coordinators")
                .header("origin", "http://localhost:3000")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .contains_key("access-control-allow-origin"));
}
