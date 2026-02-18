//! REST API client for calling the automapper-api server.
//!
//! In production, the API is served on the same origin (single binary).
//! During development, trunk proxies `/api/*` to the API server.

use gloo_net::http::Request;

use crate::types::{
    ConvertRequest, ConvertResponse, CoordinatorInfo, HealthResponse, InspectRequest,
    InspectResponse,
};

/// Base URL for API calls. Empty string means same origin.
const API_BASE: &str = "";

/// Convert content using the specified direction endpoint.
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
