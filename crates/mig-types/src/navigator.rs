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

    /// Count repetitions of a child group within a specific parent group instance.
    ///
    /// * `parent_path` - Path to the parent group (e.g., `&["SG4", "SG8"]`)
    /// * `parent_instance` - Which repetition of the parent group (0-based)
    /// * `child_group_id` - ID of the child group to count (e.g., `"SG10"`)
    fn child_group_instance_count(
        &self,
        parent_path: &[&str],
        parent_instance: usize,
        child_group_id: &str,
    ) -> usize {
        let _ = (parent_path, parent_instance, child_group_id);
        0
    }

    /// Find all segments with the given tag within a child group instance.
    ///
    /// * `segment_id` - Segment tag to find (e.g., "CCI")
    /// * `parent_path` - Path to the parent group (e.g., `&["SG4", "SG8"]`)
    /// * `parent_instance` - Which repetition of the parent group (0-based)
    /// * `child_group_id` - ID of the child group (e.g., `"SG10"`)
    /// * `child_instance` - Which repetition of the child group (0-based)
    fn find_segments_in_child_group(
        &self,
        segment_id: &str,
        parent_path: &[&str],
        parent_instance: usize,
        child_group_id: &str,
        child_instance: usize,
    ) -> Vec<OwnedSegment> {
        let _ = (segment_id, parent_path, parent_instance, child_group_id, child_instance);
        vec![]
    }

    /// Extract a single value from the first matching segment in a group instance.
    ///
    /// More efficient than `find_segments_in_group` when only one value is needed.
    ///
    /// * `segment_id` - Segment tag to find
    /// * `element_index` - Which element to extract from
    /// * `component_index` - Which component within the element
    /// * `group_path` - Path of group IDs from root
    /// * `instance_index` - Which repetition of the innermost group (0-based)
    fn extract_value_in_group(
        &self,
        segment_id: &str,
        element_index: usize,
        component_index: usize,
        group_path: &[&str],
        instance_index: usize,
    ) -> Option<String> {
        let _ = (segment_id, element_index, component_index, group_path, instance_index);
        None
    }
}
