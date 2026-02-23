//! Shared types for the frontend.

use serde::{Deserialize, Serialize};

/// Conversion direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    EdifactToBo4e,
    Bo4eToEdifact,
}

impl Direction {
    pub fn label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "EDIFACT -> BO4E",
            Direction::Bo4eToEdifact => "BO4E -> EDIFACT",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Direction::EdifactToBo4e => Direction::Bo4eToEdifact,
            Direction::Bo4eToEdifact => Direction::EdifactToBo4e,
        }
    }

    pub fn input_label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "EDIFACT",
            Direction::Bo4eToEdifact => "BO4E JSON",
        }
    }

    pub fn output_label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "BO4E JSON",
            Direction::Bo4eToEdifact => "EDIFACT",
        }
    }

    pub fn api_path(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "/api/v2/convert",
            Direction::Bo4eToEdifact => "/api/v1/convert/bo4e-to-edifact",
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
            Direction::Bo4eToEdifact => concat!(
                "{\n",
                "  \"transaktions_id\": \"TXN001\",\n",
                "  \"absender\": { ... },\n",
                "  \"empfaenger\": { ... },\n",
                "  \"marktlokationen\": [ ... ]\n",
                "}"
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
}

/// V1 conversion request (legacy pipeline, used for BO4E → EDIFACT).
#[derive(Debug, Clone, Serialize)]
pub struct ConvertRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_version: Option<String>,
    pub include_trace: bool,
}

/// V1 conversion response (legacy pipeline).
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertResponse {
    pub success: bool,
    pub result: Option<String>,
    pub trace: Vec<TraceEntry>,
    pub errors: Vec<ApiErrorEntry>,
    pub duration_ms: f64,
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

/// Health response.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub available_coordinators: Vec<String>,
    pub uptime_seconds: f64,
}
