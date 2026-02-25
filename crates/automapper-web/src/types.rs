//! Shared types for the frontend.

use serde::{Deserialize, Serialize};

/// Conversion direction.
///
/// Currently only EDIFACT -> BO4E is supported via the MIG-driven v2 pipeline.
/// BO4E -> EDIFACT reverse mapping will be added when the MIG reverse pipeline is ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    EdifactToBo4e,
}

impl Direction {
    pub fn label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "EDIFACT -> BO4E",
        }
    }

    pub fn input_label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "EDIFACT",
        }
    }

    pub fn output_label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "BO4E JSON",
        }
    }

    pub fn api_path(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "/api/v2/convert",
        }
    }

    pub fn input_placeholder(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => concat!(
                "UNA:+.? '\n",
                "UNB+UNOC:3+sender+recipient+231215:1200+123456789'\n",
                "UNH+1+UTILMD:D:11A:UN:5.2e'\n",
                "BGM+E01+DOC001'\n",
                "...\n",
                "UNT+42+1'\n",
                "UNZ+1+123456789'"
            ),
        }
    }
}

/// V2 conversion request (EDIFACT → BO4E via MIG-driven pipeline).
#[derive(Debug, Clone, Serialize)]
pub struct ConvertV2Request {
    pub input: String,
    pub mode: String,
    pub format_version: String,
}

/// V2 conversion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertV2Response {
    pub mode: String,
    pub result: serde_json::Value,
    pub duration_ms: f64,
    /// Optional validation report included when validation is requested alongside conversion.
    pub validation: Option<serde_json::Value>,
}

/// Inspect request.
#[derive(Debug, Clone, Serialize)]
pub struct InspectRequest {
    pub edifact: String,
}

/// Inspect response.
#[derive(Debug, Clone, Deserialize)]
pub struct InspectResponse {
    pub segments: Vec<SegmentNode>,
    pub segment_count: usize,
    pub message_type: Option<String>,
    pub format_version: Option<String>,
}

/// A single EDIFACT segment.
#[derive(Debug, Clone, Deserialize)]
pub struct SegmentNode {
    pub tag: String,
    pub line_number: u32,
    pub raw_content: String,
    pub elements: Vec<DataElement>,
    pub children: Option<Vec<SegmentNode>>,
}

/// A data element within a segment.
#[derive(Debug, Clone, Deserialize)]
pub struct DataElement {
    pub position: u32,
    pub value: Option<String>,
    pub components: Option<Vec<ComponentElement>>,
}

/// A component element within a composite data element.
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentElement {
    pub position: u32,
    pub value: Option<String>,
}

/// A mapping trace entry.
#[derive(Debug, Clone, Deserialize)]
pub struct TraceEntry {
    pub mapper: String,
    pub source_segment: String,
    pub target_path: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

/// An error entry from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorEntry {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
    pub severity: String,
}

/// A single fixture entry from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct FixtureEntry {
    pub name: String,
    pub pid: String,
    pub has_edi: bool,
    pub has_bo4e: bool,
}

/// Response from `GET /api/v1/fixtures`.
#[derive(Debug, Clone, Deserialize)]
pub struct FixtureListResponse {
    pub fixtures: Vec<FixtureEntry>,
}

/// Coordinator info.
#[derive(Debug, Clone, Deserialize)]
pub struct CoordinatorInfo {
    pub message_type: String,
    pub description: String,
    pub supported_versions: Vec<String>,
}

/// V2 validation request.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateV2Request {
    /// Raw EDIFACT content to validate.
    pub input: String,
    /// Format version (e.g., "FV2504").
    pub format_version: String,
    /// Validation level: "structure", "conditions", or "full". Defaults to "full".
    pub level: String,
}

impl Default for ValidateV2Request {
    fn default() -> Self {
        Self {
            input: String::new(),
            format_version: String::new(),
            level: "full".to_string(),
        }
    }
}

/// V2 validation response.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateV2Response {
    /// The validation report as JSON.
    pub report: serde_json::Value,
    /// Validation duration in milliseconds.
    pub duration_ms: f64,
}

/// Extract validation issues from a `ValidationReport` JSON value into `ApiErrorEntry` entries.
///
/// The backend `ValidationIssue` has:
/// - `severity` (PascalCase: "Error"/"Warning"/"Info") -> mapped to lowercase
/// - `code` -> passed through
/// - `message` -> passed through
/// - `field_path` (optional) -> mapped to `ApiErrorEntry.location`
///
/// Returns an empty `Vec` if the `issues` array is missing or not an array.
pub fn extract_validation_issues(report: &serde_json::Value) -> Vec<ApiErrorEntry> {
    let Some(issues) = report.get("issues").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    issues
        .iter()
        .filter_map(|issue| {
            let severity = issue.get("severity")?.as_str()?.to_lowercase();
            let code = issue.get("code")?.as_str()?.to_string();
            let message = issue.get("message")?.as_str()?.to_string();
            let location = issue
                .get("field_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(ApiErrorEntry {
                code,
                message,
                location,
                severity,
            })
        })
        .collect()
}

/// Health response.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub available_coordinators: Vec<String>,
    pub uptime_seconds: f64,
}
