//! EDIFACT inspection request and response types.

use serde::{Deserialize, Serialize};

/// Request body for `POST /api/v1/inspect/edifact`.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct InspectRequest {
    /// The raw EDIFACT content to inspect.
    pub edifact: String,
}

/// Response body for EDIFACT inspection.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InspectResponse {
    /// Flat list of parsed segments.
    pub segments: Vec<SegmentNode>,

    /// Total number of segments parsed.
    pub segment_count: usize,

    /// Detected message type (e.g., "UTILMD"), if found in UNH.
    pub message_type: Option<String>,

    /// Detected format version, if derivable from the content.
    pub format_version: Option<String>,
}

/// A single EDIFACT segment in the tree.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(no_recursion)]
pub struct SegmentNode {
    /// Segment tag (e.g., "UNH", "NAD", "LOC").
    pub tag: String,

    /// 1-based line number (segment ordinal position).
    pub line_number: u32,

    /// Raw segment content (without the segment terminator).
    pub raw_content: String,

    /// Parsed data elements within this segment.
    pub elements: Vec<DataElement>,

    /// Child segments (for hierarchical grouping; `None` for flat output).
    pub children: Option<Vec<SegmentNode>>,
}

/// A data element within a segment.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DataElement {
    /// 1-based position within the segment.
    pub position: u32,

    /// Simple element value (if not composite).
    pub value: Option<String>,

    /// Component elements (if composite, i.e., contains `:` separators).
    pub components: Option<Vec<ComponentElement>>,
}

/// A component element within a composite data element.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ComponentElement {
    /// 1-based position within the composite element.
    pub position: u32,

    /// Component value.
    pub value: Option<String>,
}
