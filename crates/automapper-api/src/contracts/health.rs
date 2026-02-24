//! Health check types.

use serde::{Deserialize, Serialize};

/// Response body for `GET /health`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    /// Whether the service is healthy.
    pub healthy: bool,

    /// Application version string.
    pub version: String,

    /// Available coordinator message types.
    pub available_coordinators: Vec<String>,

    /// Server uptime in seconds.
    pub uptime_seconds: f64,
}
