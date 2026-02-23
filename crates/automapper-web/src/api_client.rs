//! REST API client for calling the automapper-api server.
//!
//! In production, the API is served on the same origin (single binary).
//! During development, trunk proxies `/api/*` to the API server.

use gloo_net::http::Request;

use crate::types::{
    ConvertRequest, ConvertResponse, ConvertV2Request, ConvertV2Response, CoordinatorInfo,
    FixtureListResponse, HealthResponse, InspectRequest, InspectResponse,
};

/// Base URL for API calls. Empty string means same origin.
const API_BASE: &str = "";

/// Convert EDIFACT → BO4E using the v2 MIG-driven pipeline.
pub async fn convert_v2(
    input: &str,
    mode: &str,
    format_version: &str,
) -> Result<ConvertV2Response, String> {
    let request_body = ConvertV2Request {
        input: input.to_string(),
        mode: mode.to_string(),
        format_version: format_version.to_string(),
    };

    let url = format!("{API_BASE}/api/v2/convert");

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<ConvertV2Response>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(format!("HTTP {status}: {body}"))
    }
}

/// Convert content using the v1 legacy pipeline.
pub async fn convert(
    api_path: &str,
    content: &str,
    format_version: Option<String>,
    include_trace: bool,
) -> Result<ConvertResponse, String> {
    let request_body = ConvertRequest {
        content: content.to_string(),
        format_version,
        include_trace,
    };

    let url = format!("{API_BASE}{api_path}");

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<ConvertResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(format!("HTTP {status}: {body}"))
    }
}

/// Inspect EDIFACT content, returning a segment tree.
pub async fn inspect_edifact(edifact: &str) -> Result<InspectResponse, String> {
    let request_body = InspectRequest {
        edifact: edifact.to_string(),
    };

    let url = format!("{API_BASE}/api/v1/inspect/edifact");

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<InspectResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}

/// List available coordinators.
pub async fn list_coordinators() -> Result<Vec<CoordinatorInfo>, String> {
    let url = format!("{API_BASE}/api/v1/coordinators");

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<Vec<CoordinatorInfo>>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}

/// List available fixture files for a message type and format version.
pub async fn list_fixtures(
    message_type: &str,
    format_version: &str,
) -> Result<FixtureListResponse, String> {
    let url = format!(
        "{API_BASE}/api/v1/fixtures?message_type={message_type}&format_version={format_version}"
    );

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<FixtureListResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}

/// Fetch the content of a specific fixture file.
pub async fn get_fixture_content(
    message_type: &str,
    format_version: &str,
    name: &str,
    file_type: &str,
) -> Result<String, String> {
    let url = format!(
        "{API_BASE}/api/v1/fixtures/{message_type}/{format_version}/{name}?type={file_type}"
    );

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .text()
            .await
            .map_err(|e| format!("failed to read response: {e}"))
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(format!("HTTP {status}: {body}"))
    }
}

/// Get health status.
pub async fn get_health() -> Result<HealthResponse, String> {
    let url = format!("{API_BASE}/health");

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}
