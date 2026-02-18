//! Conversion request and response types.

use serde::{Deserialize, Serialize};

use super::error::ApiErrorEntry;
use super::trace::TraceEntry;

/// Request body for `POST /api/v1/convert/edifact-to-bo4e`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertRequest {
    /// The raw content to convert (EDIFACT string or BO4E JSON).
    pub content: String,

    /// Optional format version override (e.g., "FV2504", "FV2510").
    /// If omitted, auto-detected from the content.
    pub format_version: Option<String>,

    /// Whether to include a mapping trace in the response.
    #[serde(default)]
    pub include_trace: bool,
}

/// Response body for conversion endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResponse {
    /// Whether the conversion succeeded.
    pub success: bool,

    /// The converted content (BO4E JSON or EDIFACT string).
    /// `None` if the conversion failed.
    pub result: Option<String>,

    /// Mapping trace entries (empty if `include_trace` was false).
    pub trace: Vec<TraceEntry>,

    /// Errors encountered during conversion.
    pub errors: Vec<ApiErrorEntry>,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,
}
