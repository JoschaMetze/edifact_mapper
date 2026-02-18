//! Mapping trace types.

use serde::{Deserialize, Serialize};

/// A single entry in the mapping trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Name of the mapper/writer that processed this step.
    pub mapper: String,

    /// Source EDIFACT segment reference (e.g., "NAD (line 5)").
    pub source_segment: String,

    /// Target BO4E path (e.g., "geschaeftspartner.name1").
    pub target_path: String,

    /// Mapped value, if available.
    pub value: Option<String>,

    /// Optional note about the mapping step.
    pub note: Option<String>,
}
