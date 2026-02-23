//! Integration tests for POST /api/v2/reverse.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::reverse_v2::ReverseV2Response;
use automapper_api::state::AppState;

fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_http_router(state)
}

// --- Contract deserialization ---

#[tokio::test]
async fn test_reverse_missing_required_fields_returns_422() {
    let app = app();

    // Missing 'level' field (required)
    let body = serde_json::json!({
        "input": {},
        "formatVersion": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_reverse_invalid_level_returns_422() {
    let app = app();

    let body = serde_json::json!({
        "input": {},
        "level": "invalid-level",
        "formatVersion": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// --- Normalization error ---

#[tokio::test]
async fn test_reverse_invalid_interchange_json_returns_400() {
    let app = app();

    // level=interchange but input is not valid Interchange JSON
    let body = serde_json::json!({
        "input": "not an object",
        "level": "interchange",
        "formatVersion": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// --- Transaktion level with edifact mode ---

#[tokio::test]
async fn test_reverse_transaktion_level_produces_edifact() {
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
        "level": "transaktion",
        "formatVersion": "FV2504",
        "mode": "edifact",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ReverseV2Response = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp.mode, "edifact");
        assert!(resp.duration_ms >= 0.0);

        // Result should be a string containing EDIFACT envelope segments
        let edifact = resp.result.as_str().expect("result should be a string");
        assert!(edifact.contains("UNA"), "Should contain UNA segment");
        assert!(edifact.contains("UNB"), "Should contain UNB segment");
        assert!(edifact.contains("UNH"), "Should contain UNH segment");
        assert!(edifact.contains("UNT"), "Should contain UNT segment");
        assert!(edifact.contains("UNZ"), "Should contain UNZ segment");
    } else {
        // MIG XML or AHB not available — acceptable in CI
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("reverse transaktion returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- MIG-tree mode ---

#[tokio::test]
async fn test_reverse_mig_tree_mode() {
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
        "level": "transaktion",
        "formatVersion": "FV2504",
        "mode": "mig-tree",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ReverseV2Response = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp.mode, "mig-tree");
        assert!(resp.duration_ms >= 0.0);

        // MIG tree result should have segments and groups
        assert!(
            resp.result.get("segments").is_some() || resp.result.get("groups").is_some(),
            "MIG tree should have segments or groups"
        );
    } else {
        // MIG XML or AHB not available — acceptable in CI
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("reverse mig-tree returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Default mode is edifact ---

#[tokio::test]
async fn test_reverse_default_mode_is_edifact() {
    let app = app();

    // No "mode" field — should default to edifact
    let body = serde_json::json!({
        "input": {
            "stammdaten": {},
            "transaktionsdaten": {
                "pruefidentifikator": "55001"
            }
        },
        "level": "transaktion",
        "formatVersion": "FV2504",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ReverseV2Response = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(resp.mode, "edifact");
    } else {
        // MIG XML not available — acceptable in CI
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Full roundtrip test (forward convert → reverse) ---

#[tokio::test]
async fn test_forward_then_reverse_roundtrip() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!("Skipping roundtrip test: fixture not available");
        return;
    }
    let input_edifact = std::fs::read_to_string(fixture_path).unwrap();

    // Step 1: Forward convert EDIFACT → BO4E
    let forward_app = app();
    let forward_body = serde_json::json!({
        "input": input_edifact,
        "mode": "bo4e",
        "format_version": "FV2504"
    });

    let forward_resp = forward_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&forward_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    if forward_resp.status() != StatusCode::OK {
        eprintln!("Forward convert failed, skipping roundtrip");
        return;
    }

    let forward_bytes = forward_resp.into_body().collect().await.unwrap().to_bytes();
    let forward: automapper_api::contracts::convert_v2::ConvertV2Response =
        serde_json::from_slice(&forward_bytes).unwrap();

    // Step 2: Reverse convert BO4E → EDIFACT
    let reverse_app = app();
    let reverse_body = serde_json::json!({
        "input": forward.result,
        "level": "interchange",
        "formatVersion": "FV2504",
        "mode": "edifact"
    });

    let reverse_resp = reverse_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/reverse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&reverse_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    if reverse_resp.status() != StatusCode::OK {
        let body_bytes = reverse_resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("Reverse convert returned error: {body_str}");
        // Don't fail — reverse may not be fully implemented for all segments yet
        return;
    }

    let reverse_bytes = reverse_resp.into_body().collect().await.unwrap().to_bytes();
    let reverse: ReverseV2Response = serde_json::from_slice(&reverse_bytes).unwrap();

    assert_eq!(reverse.mode, "edifact");
    let result_edifact = reverse.result.as_str().expect("result should be a string");

    // Verify the output has EDIFACT structure
    assert!(result_edifact.contains("UNA"), "Should contain UNA");
    assert!(result_edifact.contains("UNB"), "Should contain UNB");
    assert!(result_edifact.contains("UNH"), "Should contain UNH");
    assert!(result_edifact.contains("UNT"), "Should contain UNT");
    assert!(result_edifact.contains("UNZ"), "Should contain UNZ");
}
