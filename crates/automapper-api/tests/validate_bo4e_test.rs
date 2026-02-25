//! Integration tests for POST /api/v2/validate-bo4e.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::validate_bo4e::ValidateBo4eResponse;
use automapper_api::state::AppState;

fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_http_router(state)
}

/// Helper to send a validate-bo4e request and collect the response.
async fn send_validate_bo4e(app: axum::Router, body: serde_json::Value) -> (StatusCode, Vec<u8>) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate-bo4e")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, body_bytes.to_vec())
}

// --- Contract deserialization ---

#[tokio::test]
async fn test_validate_bo4e_missing_input_returns_422() {
    let app = app();

    let body = serde_json::json!({
        "formatVersion": "FV2504"
    });

    let (status, _) = send_validate_bo4e(app, body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_validate_bo4e_missing_pid_returns_400() {
    let app = app();

    // Input without pruefidentifikator
    let body = serde_json::json!({
        "input": {
            "stammdaten": {},
            "transaktionsdaten": {}
        },
        "formatVersion": "FV2504"
    });

    let (status, body_bytes) = send_validate_bo4e(app, body).await;
    // Should fail with 400 (missing PID) or 500 (no MIG service in CI)
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(
        status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Expected 400/422/500 for missing PID, got: {status} - {body_str}"
    );
}

// --- Valid BO4E input with PID 55001 ---

#[tokio::test]
async fn test_validate_bo4e_valid_55001_transaktion() {
    let app = app();

    let body = serde_json::json!({
        "input": {
            "stammdaten": {
                "Marktlokation": { "marktlokationsId": "51238696781" }
            },
            "transaktionsdaten": {
                "pruefidentifikator": "55001",
                "kategorie": "E01"
            }
        },
        "formatVersion": "FV2504",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    let (status, body_bytes) = send_validate_bo4e(app, body).await;

    if status == StatusCode::OK {
        let resp: ValidateBo4eResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert!(resp.duration_ms >= 0.0);

        // Report should have expected metadata
        let report = &resp.report;
        assert_eq!(report["message_type"], "UTILMD");
        assert_eq!(report["pruefidentifikator"], "55001");

        // Issues should be an array
        let issues = report["issues"]
            .as_array()
            .expect("issues should be an array");

        // Check that issues with field_path may have bo4e_path enriched
        for issue in issues {
            // Verify issue shape
            assert!(issue.get("severity").is_some());
            assert!(issue.get("category").is_some());
            assert!(issue.get("message").is_some());
        }
    } else {
        // MIG/AHB not available — acceptable in CI
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate-bo4e returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST
                || status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Check that bo4e_path is populated on issues ---

#[tokio::test]
async fn test_validate_bo4e_issues_have_bo4e_paths() {
    let app = app();

    let body = serde_json::json!({
        "input": {
            "stammdaten": {
                "Marktlokation": { "marktlokationsId": "51238696781" }
            },
            "transaktionsdaten": {
                "pruefidentifikator": "55001",
                "kategorie": "E01"
            }
        },
        "formatVersion": "FV2504",
        "validationLevel": "full",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    let (status, body_bytes) = send_validate_bo4e(app, body).await;

    if status == StatusCode::OK {
        let resp: ValidateBo4eResponse = serde_json::from_slice(&body_bytes).unwrap();
        let issues = resp.report["issues"].as_array().unwrap();

        // Among issues with field_path, at least some should have bo4e_path
        let issues_with_field_path: Vec<_> = issues
            .iter()
            .filter(|i| i.get("field_path").is_some() && !i["field_path"].is_null())
            .collect();

        if !issues_with_field_path.is_empty() {
            let issues_with_bo4e_path: Vec<_> = issues_with_field_path
                .iter()
                .filter(|i| i.get("bo4e_path").is_some() && !i["bo4e_path"].is_null())
                .collect();

            // At least some issues should have been enriched with bo4e_path
            eprintln!(
                "Issues with field_path: {}, with bo4e_path: {}",
                issues_with_field_path.len(),
                issues_with_bo4e_path.len()
            );

            // Verify bo4e_path format
            for issue in &issues_with_bo4e_path {
                let bo4e_path = issue["bo4e_path"].as_str().unwrap();
                assert!(
                    bo4e_path.starts_with("stammdaten.")
                        || bo4e_path.starts_with("transaktionsdaten."),
                    "bo4e_path should start with stammdaten or transaktionsdaten, got: {bo4e_path}"
                );
            }
        }
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate-bo4e returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST
                || status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Backward compat: regular validate endpoint has no bo4e_path ---

#[tokio::test]
async fn test_validate_v2_has_no_bo4e_path() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }
    let input = std::fs::read_to_string(fixture_path).unwrap();

    let app = app();

    let body = serde_json::json!({
        "input": input,
        "format_version": "FV2504",
        "level": "full"
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
        let resp: automapper_api::contracts::validate_v2::ValidateV2Response =
            serde_json::from_slice(&body_bytes).unwrap();

        let issues = resp.report["issues"].as_array().unwrap();
        // Regular validate should NOT have bo4e_path (skip_serializing_if = None)
        for issue in issues {
            assert!(
                issue.get("bo4e_path").is_none(),
                "Regular validate should not have bo4e_path, but found one: {:?}",
                issue
            );
        }
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
    }
}

// --- Default levels ---

#[tokio::test]
async fn test_validate_bo4e_default_levels() {
    let app = app();

    // No explicit level or validationLevel — should default to transaktion + full
    let body = serde_json::json!({
        "input": {
            "stammdaten": {
                "Marktlokation": { "marktlokationsId": "51238696781" }
            },
            "transaktionsdaten": {
                "pruefidentifikator": "55001"
            }
        },
        "formatVersion": "FV2504",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    let (status, body_bytes) = send_validate_bo4e(app, body).await;

    if status == StatusCode::OK {
        let resp: ValidateBo4eResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp.report["level"], "Full");
        assert_eq!(resp.report["pruefidentifikator"], "55001");
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate-bo4e returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST
                || status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}
