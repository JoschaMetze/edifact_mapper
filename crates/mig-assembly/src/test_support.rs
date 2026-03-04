//! Shared test helpers for constructing MIG schema test fixtures.
//!
//! Available to both unit tests (`#[cfg(test)]` modules) and integration tests.

use mig_types::schema::mig::{MigSegment, MigSegmentGroup};

/// Create a minimal `MigSegment` for testing, with only the `id` set meaningfully.
pub fn make_mig_segment(id: &str) -> MigSegment {
    MigSegment {
        id: id.to_string(),
        name: id.to_string(),
        description: None,
        counter: None,
        level: 0,
        number: None,
        max_rep_std: 1,
        max_rep_spec: 1,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        example: None,
        data_elements: vec![],
        composites: vec![],
    }
}

/// Create a minimal `MigSegmentGroup` for testing.
pub fn make_mig_group(
    id: &str,
    segments: Vec<&str>,
    nested: Vec<MigSegmentGroup>,
) -> MigSegmentGroup {
    MigSegmentGroup {
        id: id.to_string(),
        name: id.to_string(),
        description: None,
        counter: None,
        level: 1,
        max_rep_std: 99,
        max_rep_spec: 99,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        segments: segments.into_iter().map(make_mig_segment).collect(),
        nested_groups: nested,
    }
}
