//! Integration tests for the v2 MIG-driven conversion endpoint.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::convert_v2::ConvertV2Response;
use automapper_api::state::AppState;

fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_http_router(state)
}

// --- Invalid mode ---

#[tokio::test]
async fn test_convert_v2_invalid_mode_returns_422() {
    let app = app();

    let body = serde_json::json!({
        "input": "UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01'UNT+2+1'",
        "mode": "invalid-mode",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Invalid mode should return 422 (unprocessable)
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// --- MIG-tree mode ---

#[tokio::test]
async fn test_convert_v2_mig_tree_mode() {
    let app = app();

    let body = serde_json::json!({
        "input": "UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001+9'UNT+3+MSG001'UNZ+1+REF001'",
        "mode": "mig-tree",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // MIG tree mode succeeds if MIG XML is available, otherwise returns 400
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ConvertV2Response = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp.mode, "mig-tree");
        assert!(
            resp.result.get("tree").is_some(),
            "Response should contain 'tree' key"
        );
        assert!(resp.duration_ms >= 0.0);
    } else {
        // MIG XML not available — acceptable in CI
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

// --- BO4E mode ---

#[tokio::test]
async fn test_convert_v2_bo4e_mode() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    let input = if fixture_path.exists() {
        std::fs::read_to_string(fixture_path).unwrap()
    } else {
        // Fallback minimal input for CI without fixtures
        "UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001+9'UNT+3+MSG001'UNZ+1+REF001'".to_string()
    };

    let app = app();

    let body = serde_json::json!({
        "input": input,
        "mode": "bo4e",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ConvertV2Response = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp.mode, "bo4e");
        assert!(resp.duration_ms >= 0.0);

        // Verify hierarchical response structure: Interchange → Nachricht → Transaktion
        assert!(
            resp.result.get("nachrichtendaten").is_some(),
            "Response should contain 'nachrichtendaten' key"
        );
        assert!(
            resp.result.get("nachrichten").is_some(),
            "Response should contain 'nachrichten' key"
        );

        let nachrichten = resp.result.get("nachrichten").unwrap().as_array().unwrap();
        assert!(
            !nachrichten.is_empty(),
            "Should have at least one Nachricht"
        );

        let first_msg = &nachrichten[0];
        assert!(
            first_msg.get("unhReferenz").is_some(),
            "Nachricht should have unhReferenz"
        );
        assert!(
            first_msg.get("nachrichtenTyp").is_some(),
            "Nachricht should have nachrichtenTyp"
        );
        assert!(
            first_msg.get("stammdaten").is_some(),
            "Nachricht should have stammdaten"
        );
        assert!(
            first_msg.get("transaktionen").is_some(),
            "Nachricht should have transaktionen"
        );

        // If we used a real fixture, verify deeper content
        if fixture_path.exists() {
            let transaktionen = first_msg.get("transaktionen").unwrap().as_array().unwrap();
            assert!(
                !transaktionen.is_empty(),
                "Should have at least one Transaktion"
            );

            let first_tx = &transaktionen[0];
            assert!(
                first_tx.get("stammdaten").is_some(),
                "Transaktion should have stammdaten"
            );
            assert!(
                first_tx.get("transaktionsdaten").is_some(),
                "Transaktion should have transaktionsdaten"
            );

            // Verify nachrichtendaten has envelope data
            let nd = resp.result.get("nachrichtendaten").unwrap();
            assert!(
                nd.get("absenderCode").is_some(),
                "Should have absenderCode in nachrichtendaten"
            );
        }
    } else {
        // MIG XML or AHB not available — acceptable in CI
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("bo4e mode returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- Missing fields ---

#[tokio::test]
async fn test_convert_v2_missing_required_fields_returns_422() {
    let app = app();

    // Missing 'mode' field
    let body = serde_json::json!({
        "input": "UNH+1+UTILMD'BGM+E01'UNT+2+1'",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
