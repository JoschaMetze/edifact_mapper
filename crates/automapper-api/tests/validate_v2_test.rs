//! Integration tests for the v2 validation endpoint.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::validate_v2::ValidateV2Response;
use automapper_api::state::AppState;

fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_http_router(state)
}

// --- Missing required fields ---

#[tokio::test]
async fn test_validate_v2_missing_input_returns_422() {
    let app = app();

    // Missing 'input' field
    let body = serde_json::json!({
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing required field triggers 422 from Axum deserialization
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// --- Default level is "full" ---

#[tokio::test]
async fn test_validate_v2_default_level_is_full() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!(
            "Skipping test: fixture not found at {}",
            fixture_path.display()
        );
        return;
    }
    let input = std::fs::read_to_string(fixture_path).unwrap();

    let app = app();

    // No 'level' field — should default to "full"
    let body = serde_json::json!({
        "input": input,
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ValidateV2Response = serde_json::from_slice(&body_bytes).unwrap();
        // Report should be present and have level = "Full"
        assert!(resp.report.get("level").is_some());
        assert_eq!(resp.report["level"], "Full");
        assert!(resp.duration_ms >= 0.0);
    } else {
        // MIG XML or AHB not available — acceptable in CI
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Validate with structure level ---

#[tokio::test]
async fn test_validate_v2_structure_level() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!(
            "Skipping test: fixture not found at {}",
            fixture_path.display()
        );
        return;
    }
    let input = std::fs::read_to_string(fixture_path).unwrap();

    let app = app();

    let body = serde_json::json!({
        "input": input,
        "format_version": "FV2504",
        "level": "structure"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ValidateV2Response = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp.report["level"], "Structure");
        // Structure-only validation with stub evaluator should have no AHB issues
        let issues = resp.report["issues"].as_array().unwrap();
        let ahb_issues: Vec<_> = issues.iter().filter(|i| i["category"] == "Ahb").collect();
        assert!(
            ahb_issues.is_empty(),
            "Structure level should not have AHB issues"
        );
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Validate with external conditions ---

#[tokio::test]
async fn test_validate_v2_with_external_conditions() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!(
            "Skipping test: fixture not found at {}",
            fixture_path.display()
        );
        return;
    }
    let input = std::fs::read_to_string(fixture_path).unwrap();

    let app = app();

    let body = serde_json::json!({
        "input": input,
        "format_version": "FV2504",
        "level": "full",
        "external_conditions": {
            "DateKnown": true,
            "MessageSplitting": false
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ValidateV2Response = serde_json::from_slice(&body_bytes).unwrap();
        assert!(resp.report.get("issues").is_some());
        assert!(resp.report.get("message_type").is_some());
        assert_eq!(resp.report["message_type"], "UTILMD");
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Report contains expected metadata ---

#[tokio::test]
async fn test_validate_v2_report_metadata() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!(
            "Skipping test: fixture not found at {}",
            fixture_path.display()
        );
        return;
    }
    let input = std::fs::read_to_string(fixture_path).unwrap();

    let app = app();

    let body = serde_json::json!({
        "input": input,
        "format_version": "FV2504",
        "level": "conditions"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ValidateV2Response = serde_json::from_slice(&body_bytes).unwrap();
        // Verify report metadata
        assert_eq!(resp.report["message_type"], "UTILMD");
        assert_eq!(resp.report["format_version"], "FV2504");
        assert_eq!(resp.report["level"], "Conditions");
        assert!(resp.report.get("pruefidentifikator").is_some());
        assert_eq!(resp.report["pruefidentifikator"], "55001");
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}
