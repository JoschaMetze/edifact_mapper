//! MIG-aware segment structure lookup.
//!
//! Provides the expected element count for each segment tag, derived from the MIG schema.
//! Used by `MappingEngine::map_reverse` to pad trailing empty elements so that
//! reconstructed EDIFACT segments match the original structure.

use std::collections::HashMap;

use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

/// Maps segment tags to their expected element count from the MIG schema.
///
/// Element count = `data_elements.len() + composites.len()` for each segment.
/// This matches the EDIFACT convention where each data element or composite
/// occupies one element position separated by `+`.
pub struct SegmentStructure {
    pub(crate) element_counts: HashMap<String, usize>,
}

impl SegmentStructure {
    /// Build from a MIG schema by walking all segments (top-level and within groups).
    ///
    /// First occurrence of a segment tag wins — the same tag always has the same
    /// element structure in EDIFACT.
    pub fn from_mig(mig: &MigSchema) -> Self {
        let mut element_counts = HashMap::new();

        // Top-level segments
        for seg in &mig.segments {
            Self::register_segment(&mut element_counts, seg);
        }

        // Segments within groups (recursive)
        for group in &mig.segment_groups {
            Self::walk_group(&mut element_counts, group);
        }

        Self { element_counts }
    }

    /// Look up the expected element count for a segment tag.
    pub fn element_count(&self, tag: &str) -> Option<usize> {
        self.element_counts.get(&tag.to_uppercase()).copied()
    }

    fn register_segment(counts: &mut HashMap<String, usize>, seg: &MigSegment) {
        let tag = seg.id.to_uppercase();
        counts
            .entry(tag)
            .or_insert_with(|| seg.data_elements.len() + seg.composites.len());
    }

    fn walk_group(counts: &mut HashMap<String, usize>, group: &MigSegmentGroup) {
        for seg in &group.segments {
            Self::register_segment(counts, seg);
        }
        for nested in &group.nested_groups {
            Self::walk_group(counts, nested);
        }
    }
}
