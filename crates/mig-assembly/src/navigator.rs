//! GroupNavigator implementation backed by AssembledTree.

use crate::assembler::{AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree};
use mig_types::navigator::GroupNavigator;
use mig_types::segment::OwnedSegment;

/// Wraps an `AssembledTree` reference to provide group-scoped segment queries.
pub struct AssembledTreeNavigator<'a> {
    tree: &'a AssembledTree,
}

impl<'a> AssembledTreeNavigator<'a> {
    pub fn new(tree: &'a AssembledTree) -> Self {
        Self { tree }
    }
}

impl GroupNavigator for AssembledTreeNavigator<'_> {
    fn find_segments_in_group(
        &self,
        segment_id: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> Vec<OwnedSegment> {
        let Some(instance) = resolve_instance(&self.tree.groups, group_path, instance_index) else {
            return Vec::new();
        };
        instance
            .segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.tag.eq_ignore_ascii_case(segment_id))
            .map(|(i, s)| to_owned(s, i as u32))
            .collect()
    }

    fn find_segments_with_qualifier_in_group(
        &self,
        segment_id: &str,
        element_index: usize,
        qualifier: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> Vec<OwnedSegment> {
        self.find_segments_in_group(segment_id, group_path, instance_index)
            .into_iter()
            .filter(|s| {
                s.elements
                    .get(element_index)
                    .and_then(|e| e.first())
                    .is_some_and(|v| v == qualifier)
            })
            .collect()
    }

    fn group_instance_count(&self, group_path: &[&str]) -> usize {
        resolve_group(&self.tree.groups, group_path)
            .map(|g| g.repetitions.len())
            .unwrap_or(0)
    }

    fn child_group_instance_count(
        &self,
        parent_path: &[&str],
        parent_instance: usize,
        child_group_id: &str,
    ) -> usize {
        let Some(parent) = resolve_instance(&self.tree.groups, parent_path, parent_instance) else {
            return 0;
        };
        parent
            .child_groups
            .iter()
            .find(|g| g.group_id == child_group_id)
            .map(|g| g.repetitions.len())
            .unwrap_or(0)
    }

    fn find_segments_in_child_group(
        &self,
        segment_id: &str,
        parent_path: &[&str],
        parent_instance: usize,
        child_group_id: &str,
        child_instance: usize,
    ) -> Vec<OwnedSegment> {
        let Some(parent) = resolve_instance(&self.tree.groups, parent_path, parent_instance) else {
            return Vec::new();
        };
        let Some(child_group) = parent
            .child_groups
            .iter()
            .find(|g| g.group_id == child_group_id)
        else {
            return Vec::new();
        };
        let Some(instance) = child_group.repetitions.get(child_instance) else {
            return Vec::new();
        };
        instance
            .segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.tag.eq_ignore_ascii_case(segment_id))
            .map(|(i, s)| to_owned(s, i as u32))
            .collect()
    }

    fn extract_value_in_group(
        &self,
        segment_id: &str,
        element_index: usize,
        component_index: usize,
        group_path: &[&str],
        instance_index: usize,
    ) -> Option<String> {
        let instance = resolve_instance(&self.tree.groups, group_path, instance_index)?;
        let seg = instance
            .segments
            .iter()
            .find(|s| s.tag.eq_ignore_ascii_case(segment_id))?;
        seg.elements
            .get(element_index)?
            .get(component_index)
            .cloned()
    }
}

/// Navigate group hierarchy to find an AssembledGroup at the given path.
fn resolve_group<'a>(groups: &'a [AssembledGroup], path: &[&str]) -> Option<&'a AssembledGroup> {
    if path.is_empty() {
        return None;
    }
    let group = groups.iter().find(|g| g.group_id == path[0])?;
    if path.len() == 1 {
        return Some(group);
    }
    // Navigate deeper: use first repetition of intermediate groups
    let instance = group.repetitions.first()?;
    resolve_group(&instance.child_groups, &path[1..])
}

/// Navigate to a specific group instance at the given path.
fn resolve_instance<'a>(
    groups: &'a [AssembledGroup],
    path: &[&str],
    instance_index: usize,
) -> Option<&'a AssembledGroupInstance> {
    let group = resolve_group(groups, path)?;
    group.repetitions.get(instance_index)
}

fn to_owned(seg: &AssembledSegment, segment_number: u32) -> OwnedSegment {
    OwnedSegment {
        id: seg.tag.clone(),
        elements: seg.elements.clone(),
        segment_number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::{
        AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree,
    };
    use mig_types::navigator::GroupNavigator;

    fn make_seg(tag: &str, elements: Vec<Vec<&str>>) -> AssembledSegment {
        AssembledSegment {
            tag: tag.to_string(),
            elements: elements
                .into_iter()
                .map(|e| e.into_iter().map(|c| c.to_string()).collect())
                .collect(),
        }
    }

    fn tree_with_sg4_sg8() -> AssembledTree {
        // SG4[0] -> segments: [IDE, STS]
        //        -> SG8[0]: [SEQ+Z98, CCI+Z30++Z07]
        //             -> SG10[0]: [CCI+Z23, CAV+Z91:value1]
        //             -> SG10[1]: [CCI, CAV+:value2]
        //        -> SG8[1]: [SEQ+Z01, CCI+++ZC0]
        //             (no SG10)
        AssembledTree {
            segments: vec![make_seg("UNH", vec![vec!["001"]])],
            groups: vec![AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![
                        make_seg("IDE", vec![vec!["24", "TX001"]]),
                        make_seg("STS", vec![vec!["E01"], vec![], vec!["A05"]]),
                    ],
                    child_groups: vec![AssembledGroup {
                        group_id: "SG8".to_string(),
                        repetitions: vec![
                            AssembledGroupInstance {
                                segments: vec![
                                    make_seg("SEQ", vec![vec!["Z98"]]),
                                    make_seg("CCI", vec![vec!["Z30"], vec![], vec!["Z07"]]),
                                ],
                                child_groups: vec![AssembledGroup {
                                    group_id: "SG10".to_string(),
                                    repetitions: vec![
                                        AssembledGroupInstance {
                                            segments: vec![
                                                make_seg("CCI", vec![vec!["Z23"]]),
                                                make_seg("CAV", vec![vec!["Z91", "value1"]]),
                                            ],
                                            child_groups: vec![],
                                            skipped_segments: vec![],
                                        },
                                        AssembledGroupInstance {
                                            segments: vec![
                                                make_seg("CCI", vec![vec![""]]),
                                                make_seg("CAV", vec![vec!["", "value2"]]),
                                            ],
                                            child_groups: vec![],
                                            skipped_segments: vec![],
                                        },
                                    ],
                                }],
                                skipped_segments: vec![],
                            },
                            AssembledGroupInstance {
                                segments: vec![
                                    make_seg("SEQ", vec![vec!["Z01"]]),
                                    make_seg("CCI", vec![vec![""], vec![], vec!["ZC0"]]),
                                ],
                                child_groups: vec![],
                                skipped_segments: vec![],
                            },
                        ],
                    }],
                    skipped_segments: vec![],
                }],
            }],
            post_group_start: 1,
            inter_group_segments: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn test_find_in_sg8_instance_0() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        let segs = nav.find_segments_in_group("SEQ", &["SG4", "SG8"], 0);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].get_element(0), "Z98");
    }

    #[test]
    fn test_find_in_sg8_instance_1() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        let segs = nav.find_segments_in_group("SEQ", &["SG4", "SG8"], 1);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].get_element(0), "Z01");
    }

    #[test]
    fn test_qualifier_in_group_scoped() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        let segs = nav.find_segments_with_qualifier_in_group("CCI", 2, "ZC0", &["SG4", "SG8"], 1);
        assert_eq!(segs.len(), 1);
        // NOT in instance 0
        assert!(nav
            .find_segments_with_qualifier_in_group("CCI", 2, "ZC0", &["SG4", "SG8"], 0)
            .is_empty());
    }

    #[test]
    fn test_group_instance_count() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        assert_eq!(nav.group_instance_count(&["SG4"]), 1);
        assert_eq!(nav.group_instance_count(&["SG4", "SG8"]), 2);
        assert_eq!(nav.group_instance_count(&["SG4", "SG5"]), 0);
    }

    #[test]
    fn test_find_in_sg4_directly() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        let segs = nav.find_segments_in_group("STS", &["SG4"], 0);
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn test_invalid_path_returns_empty() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        assert!(nav.find_segments_in_group("SEQ", &["SG99"], 0).is_empty());
        assert!(nav
            .find_segments_in_group("SEQ", &["SG4", "SG8"], 99)
            .is_empty());
        assert!(nav.find_segments_in_group("SEQ", &[], 0).is_empty());
    }

    #[test]
    fn test_child_group_instance_count() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        // SG8[0] has 2 SG10 children
        assert_eq!(
            nav.child_group_instance_count(&["SG4", "SG8"], 0, "SG10"),
            2
        );
        // SG8[1] has no SG10 children
        assert_eq!(
            nav.child_group_instance_count(&["SG4", "SG8"], 1, "SG10"),
            0
        );
        // Non-existent child group
        assert_eq!(
            nav.child_group_instance_count(&["SG4", "SG8"], 0, "SG12"),
            0
        );
    }

    #[test]
    fn test_find_segments_in_child_group() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        // SG8[0] -> SG10[0] has CCI+Z23
        let segs = nav.find_segments_in_child_group("CCI", &["SG4", "SG8"], 0, "SG10", 0);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].get_element(0), "Z23");
        // SG8[0] -> SG10[1] has CCI with empty qualifier
        let segs = nav.find_segments_in_child_group("CCI", &["SG4", "SG8"], 0, "SG10", 1);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].get_element(0), "");
        // SG8[0] -> SG10[0] has CAV
        let segs = nav.find_segments_in_child_group("CAV", &["SG4", "SG8"], 0, "SG10", 0);
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn test_child_group_invalid_path() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        // Invalid parent path
        assert_eq!(nav.child_group_instance_count(&["SG99"], 0, "SG10"), 0);
        // Invalid parent instance
        assert!(nav
            .find_segments_in_child_group("CCI", &["SG4", "SG8"], 99, "SG10", 0)
            .is_empty());
        // Invalid child instance
        assert!(nav
            .find_segments_in_child_group("CCI", &["SG4", "SG8"], 0, "SG10", 99)
            .is_empty());
    }

    #[test]
    fn test_extract_value_in_group() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        // SEQ qualifier in SG8[0]
        assert_eq!(
            nav.extract_value_in_group("SEQ", 0, 0, &["SG4", "SG8"], 0),
            Some("Z98".to_string()),
        );
        // SEQ qualifier in SG8[1]
        assert_eq!(
            nav.extract_value_in_group("SEQ", 0, 0, &["SG4", "SG8"], 1),
            Some("Z01".to_string()),
        );
    }

    #[test]
    fn test_extract_value_missing() {
        let tree = tree_with_sg4_sg8();
        let nav = AssembledTreeNavigator::new(&tree);
        // Non-existent segment
        assert_eq!(
            nav.extract_value_in_group("LOC", 0, 0, &["SG4", "SG8"], 0),
            None
        );
        // Element index out of bounds
        assert_eq!(
            nav.extract_value_in_group("SEQ", 5, 0, &["SG4", "SG8"], 0),
            None
        );
        // Component index out of bounds
        assert_eq!(
            nav.extract_value_in_group("SEQ", 0, 5, &["SG4", "SG8"], 0),
            None
        );
        // Invalid group path
        assert_eq!(nav.extract_value_in_group("SEQ", 0, 0, &["SG99"], 0), None);
    }
}
