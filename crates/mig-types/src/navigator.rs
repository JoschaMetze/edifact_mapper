//! Group-scoped segment navigation trait.

use crate::segment::OwnedSegment;

/// Provides group-scoped segment access for condition evaluation.
///
/// Implementations translate hierarchical group paths (e.g., `["SG4", "SG8"]`)
/// into segment lookups scoped to a specific group instance.
pub trait GroupNavigator: Send + Sync {
    /// Find all segments with the given tag within a specific group instance.
    ///
    /// * `segment_id` - Segment tag to find (e.g., "SEQ", "CCI")
    /// * `group_path` - Path of group IDs from root (e.g., `&["SG4", "SG8"]`)
    /// * `instance_index` - Which repetition of the innermost group (0-based)
    fn find_segments_in_group(
        &self,
        segment_id: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> Vec<OwnedSegment>;

    /// Find segments matching a tag + qualifier within a group instance.
    ///
    /// * `segment_id` - Segment tag to find
    /// * `element_index` - Which element contains the qualifier
    /// * `qualifier` - Expected qualifier value
    /// * `group_path` - Path of group IDs from root
    /// * `instance_index` - Which repetition of the innermost group (0-based)
    fn find_segments_with_qualifier_in_group(
        &self,
        segment_id: &str,
        element_index: usize,
        qualifier: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> Vec<OwnedSegment>;

    /// Count repetitions of a group at the given path.
    fn group_instance_count(&self, group_path: &[&str]) -> usize;
}
