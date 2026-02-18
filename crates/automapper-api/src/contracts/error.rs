//! Error types returned in API responses.

use serde::{Deserialize, Serialize};

/// An error entry in an API response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorEntry {
    /// Machine-readable error code (e.g., "UNKNOWN_TYPE", "PARSE_ERROR").
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// Optional location in the source content (e.g., "segment 5, byte 234").
    pub location: Option<String>,

    /// Error severity.
    pub severity: ErrorSeverity,
}

/// Severity level for API errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}
