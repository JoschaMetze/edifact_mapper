//! Owned EDIFACT segment — a crate-independent segment representation.
//!
//! `OwnedSegment` stores parsed EDIFACT segment data as owned `String`s.
//! It lives in `mig-types` so that both generated PID types and the
//! assembly machinery can reference it without circular dependencies.

use serde::{Deserialize, Serialize};

/// An owned version of a parsed EDIFACT segment — stores String data.
///
/// Used for the two-pass assembler: pass 1 collects segments into this
/// type, pass 2 consumes them guided by the MIG schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnedSegment {
    /// Segment identifier (e.g., "NAD", "LOC", "DTM").
    pub id: String,
    /// Elements, where each element is a vector of component strings.
    /// `elements[i][j]` = component `j` of element `i`.
    pub elements: Vec<Vec<String>>,
    /// 1-based segment number within the message.
    pub segment_number: u32,
}

impl OwnedSegment {
    /// Gets the first component of element at `index`, or empty string if missing.
    pub fn get_element(&self, index: usize) -> &str {
        self.elements
            .get(index)
            .and_then(|e| e.first())
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Gets a specific component within an element, or empty string if missing.
    pub fn get_component(&self, element_index: usize, component_index: usize) -> &str {
        self.elements
            .get(element_index)
            .and_then(|e| e.get(component_index))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Checks if the segment has the given ID (case-insensitive).
    pub fn is(&self, segment_id: &str) -> bool {
        self.id.eq_ignore_ascii_case(segment_id)
    }
}
