use serde::{Deserialize, Serialize};

use super::common::{Cardinality, CodeDefinition};

/// Complete MIG schema for a message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSchema {
    /// The EDIFACT message type (e.g., "UTILMD", "ORDERS").
    pub message_type: String,
    /// Optional variant (e.g., "Strom", "Gas").
    pub variant: Option<String>,
    /// Version number from the MIG (e.g., "S2.1", "1.4a").
    pub version: String,
    /// Publication date string.
    pub publication_date: String,
    /// Author (typically "BDEW").
    pub author: String,
    /// Format version directory (e.g., "FV2504").
    pub format_version: String,
    /// Path to the source XML file.
    pub source_file: String,
    /// Top-level segment definitions (not in groups).
    pub segments: Vec<MigSegment>,
    /// Segment group definitions (contain more segments).
    pub segment_groups: Vec<MigSegmentGroup>,
}

/// A segment (S_*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSegment {
    /// Segment identifier (e.g., "UNH", "BGM", "NAD").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of the segment.
    pub description: Option<String>,
    /// Position counter (e.g., "0010", "0020").
    pub counter: Option<String>,
    /// Nesting level (0=root, 1=first level, etc.).
    pub level: i32,
    /// Sequence number within the message.
    pub number: Option<String>,
    /// Standard maximum repetitions.
    pub max_rep_std: i32,
    /// Specification maximum repetitions.
    pub max_rep_spec: i32,
    /// Standard status (M=Mandatory, C=Conditional, etc.).
    pub status_std: Option<String>,
    /// Specification status (M, R, D, O, N).
    pub status_spec: Option<String>,
    /// Example EDIFACT string.
    pub example: Option<String>,
    /// Direct child data elements.
    pub data_elements: Vec<MigDataElement>,
    /// Child composite elements.
    pub composites: Vec<MigComposite>,
}

impl MigSegment {
    /// Returns the effective cardinality based on spec or std status.
    pub fn cardinality(&self) -> Cardinality {
        let status = self
            .status_spec
            .as_deref()
            .or(self.status_std.as_deref())
            .unwrap_or("C");
        Cardinality::from_status(status)
    }

    /// Returns the effective max repetitions (spec overrides std).
    pub fn max_rep(&self) -> i32 {
        self.max_rep_spec.max(self.max_rep_std)
    }
}

/// A segment group (G_SG*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSegmentGroup {
    /// Group identifier (e.g., "SG1", "SG2", "SG10").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of the segment group.
    pub description: Option<String>,
    /// Position counter (e.g., "0070", "0500").
    pub counter: Option<String>,
    /// Nesting level.
    pub level: i32,
    /// Standard maximum repetitions.
    pub max_rep_std: i32,
    /// Specification maximum repetitions.
    pub max_rep_spec: i32,
    /// Standard status.
    pub status_std: Option<String>,
    /// Specification status.
    pub status_spec: Option<String>,
    /// Segments directly in this group.
    pub segments: Vec<MigSegment>,
    /// Nested segment groups.
    pub nested_groups: Vec<MigSegmentGroup>,
}

impl MigSegmentGroup {
    /// Returns the effective cardinality.
    pub fn cardinality(&self) -> Cardinality {
        let status = self
            .status_spec
            .as_deref()
            .or(self.status_std.as_deref())
            .unwrap_or("C");
        Cardinality::from_status(status)
    }
}

/// A composite element (C_*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigComposite {
    /// Composite identifier (e.g., "S009", "C002").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Standard status.
    pub status_std: Option<String>,
    /// Specification status.
    pub status_spec: Option<String>,
    /// Child data elements within this composite.
    pub data_elements: Vec<MigDataElement>,
    /// Position of this composite within its parent segment (0-based).
    pub position: usize,
}

/// A data element (D_*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigDataElement {
    /// Element identifier (e.g., "0062", "3035").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Standard status.
    pub status_std: Option<String>,
    /// Specification status.
    pub status_spec: Option<String>,
    /// Standard format (e.g., "an..14", "n13").
    pub format_std: Option<String>,
    /// Specification format.
    pub format_spec: Option<String>,
    /// Allowed code values, if restricted.
    pub codes: Vec<CodeDefinition>,
    /// Position within parent (0-based).
    pub position: usize,
}
