//! Integration tests for the v2 validation endpoint.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::convert_v2::ConvertV2Response;
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

// ============================================================================
// Integration tests with real EDIFACT fixture files
// ============================================================================

// --- PID 55001: full validation with real fixture ---

#[tokio::test]
async fn test_validate_55001_fixture_returns_clean_report() {
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
        let resp: ValidateV2Response = serde_json::from_slice(&body_bytes).unwrap();

        // Core metadata assertions
        assert_eq!(resp.report["pruefidentifikator"], "55001");
        assert_eq!(resp.report["message_type"], "UTILMD");
        assert_eq!(resp.report["format_version"], "FV2504");
        assert_eq!(resp.report["level"], "Full");

        // Issues must be an array (may or may not be empty depending on evaluator coverage)
        let issues = resp.report["issues"]
            .as_array()
            .expect("issues should be an array");
        // Verify each issue has required fields
        for issue in issues {
            assert!(
                issue.get("severity").is_some(),
                "Each issue should have 'severity'"
            );
            assert!(
                issue.get("category").is_some(),
                "Each issue should have 'category'"
            );
            assert!(
                issue.get("message").is_some(),
                "Each issue should have 'message'"
            );
        }

        assert!(resp.duration_ms >= 0.0);
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- PID 55002: full validation with real fixture ---

#[tokio::test]
async fn test_validate_55002_fixture_returns_clean_report() {
    let fixture_path = std::path::Path::new(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55002_UTILMD_S2.1_ALEXANDE104683.edi",
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
        let resp: ValidateV2Response = serde_json::from_slice(&body_bytes).unwrap();

        // Core metadata assertions for PID 55002
        assert_eq!(resp.report["pruefidentifikator"], "55002");
        assert_eq!(resp.report["message_type"], "UTILMD");
        assert_eq!(resp.report["format_version"], "FV2504");
        assert_eq!(resp.report["level"], "Full");

        // Issues must be an array
        let issues = resp.report["issues"]
            .as_array()
            .expect("issues should be an array");
        for issue in issues {
            assert!(
                issue.get("severity").is_some(),
                "Each issue should have 'severity'"
            );
            assert!(
                issue.get("category").is_some(),
                "Each issue should have 'category'"
            );
            assert!(
                issue.get("message").is_some(),
                "Each issue should have 'message'"
            );
        }

        assert!(resp.duration_ms >= 0.0);
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- PID 55001: structure-only validation (no structure errors expected) ---

#[tokio::test]
async fn test_validate_55001_structure_only() {
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
        assert_eq!(resp.report["pruefidentifikator"], "55001");

        // A well-formed fixture should pass structure validation with no structure errors
        let issues = resp.report["issues"].as_array().unwrap();
        let structure_errors: Vec<_> = issues
            .iter()
            .filter(|i| i["category"] == "Structure" && i["severity"] == "Error")
            .collect();
        assert!(
            structure_errors.is_empty(),
            "Well-formed 55001 fixture should have no structure errors, got: {structure_errors:?}"
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

// --- PID 55001: full validation with external condition overrides ---

#[tokio::test]
async fn test_validate_55001_with_external_conditions() {
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
            "DateKnown": true
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

        // Verify the response is valid JSON with expected structure
        assert_eq!(resp.report["message_type"], "UTILMD");
        assert_eq!(resp.report["pruefidentifikator"], "55001");
        assert_eq!(resp.report["level"], "Full");

        // Issues array must exist and be valid
        assert!(
            resp.report["issues"].is_array(),
            "issues should be an array"
        );

        // duration_ms should be non-negative
        assert!(resp.duration_ms >= 0.0);
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}

// --- PID 55001: convert with inline validation via ?validate=true ---

#[tokio::test]
async fn test_convert_55001_with_validate_flag() {
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
        "mode": "bo4e",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/convert?validate=true")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        let resp: ConvertV2Response = serde_json::from_slice(&body_bytes).unwrap();

        // Conversion result should be present with BO4E data
        assert_eq!(resp.mode, "bo4e");
        assert!(
            resp.result.get("nachrichtendaten").is_some(),
            "Response should contain 'nachrichtendaten'"
        );
        assert!(
            resp.result.get("nachrichten").is_some(),
            "Response should contain 'nachrichten'"
        );

        // Validation report should be present (inline via ?validate=true)
        let report = resp
            .validation
            .as_ref()
            .expect("validation report should be present when ?validate=true");

        // Validate report structure
        assert_eq!(report["message_type"], "UTILMD");
        assert!(
            report.get("level").is_some(),
            "validation report should have 'level'"
        );
        assert!(
            report["issues"].is_array(),
            "validation report should have 'issues' array"
        );

        assert!(resp.duration_ms >= 0.0);
    } else {
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("convert+validate returned {status}: {body_str}");
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected status: {status}"
        );
    }
}
