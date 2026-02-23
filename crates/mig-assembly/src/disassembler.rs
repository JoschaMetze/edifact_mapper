//! Tree disassembler — converts AssembledTree back to ordered segments.
//!
//! Walks the MIG schema tree in order. For each MIG node that has
//! corresponding data in the assembled tree, emits segments in MIG order.

use crate::assembler::{AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree};
use automapper_generator::schema::mig::{MigSchema, MigSegmentGroup};

/// Output segment from disassembly (owned data, ready for rendering).
#[derive(Debug, Clone)]
pub struct DisassembledSegment {
    pub tag: String,
    pub elements: Vec<Vec<String>>,
}

/// MIG-guided disassembler — walks the MIG tree to emit segments in correct order.
pub struct Disassembler<'a> {
    mig: &'a MigSchema,
}

impl<'a> Disassembler<'a> {
    pub fn new(mig: &'a MigSchema) -> Self {
        Self { mig }
    }

    /// Disassemble a tree into ordered segments following MIG sequence.
    ///
    /// Emits segments in correct EDIFACT order:
    /// 1. Pre-group top-level segments (e.g., UNB, UNH, BGM, DTM)
    /// 2. Groups (recursively, in MIG order)
    /// 3. Post-group top-level segments (e.g., UNT, UNZ)
    ///
    /// Uses MIG-guided ordering: walks the MIG schema tree and looks up
    /// matching data in the assembled tree. This handles both assembler output
    /// (already in MIG order) and reverse-mapped trees (may be in arbitrary order).
    pub fn disassemble(&self, tree: &AssembledTree) -> Vec<DisassembledSegment> {
        let mut output = Vec::new();

        // 1. Emit pre-group segments in MIG order
        let pre_group = &tree.segments[..tree.post_group_start];
        let mut consumed = vec![false; pre_group.len()];
        for mig_seg in &self.mig.segments {
            if let Some(idx) = pre_group
                .iter()
                .enumerate()
                .position(|(i, s)| !consumed[i] && s.tag == mig_seg.id)
            {
                output.push(assembled_to_disassembled(&pre_group[idx]));
                consumed[idx] = true;
            }
        }

        // 2. Emit groups in MIG order (lookup by group ID with consumption tracking)
        let mut consumed_groups = vec![false; tree.groups.len()];
        for mig_group in &self.mig.segment_groups {
            if let Some(idx) = tree
                .groups
                .iter()
                .enumerate()
                .position(|(i, g)| !consumed_groups[i] && g.group_id == mig_group.id)
            {
                self.emit_group(&tree.groups[idx], mig_group, &mut output);
                consumed_groups[idx] = true;
            }
        }

        // 3. Emit post-group segments (e.g., UNT, UNZ)
        for seg in &tree.segments[tree.post_group_start..] {
            output.push(assembled_to_disassembled(seg));
        }

        output
    }

    fn emit_group(
        &self,
        group: &AssembledGroup,
        mig_group: &MigSegmentGroup,
        output: &mut Vec<DisassembledSegment>,
    ) {
        for instance in &group.repetitions {
            self.emit_group_instance(instance, mig_group, output);
        }
    }

    fn emit_group_instance(
        &self,
        instance: &AssembledGroupInstance,
        mig_group: &MigSegmentGroup,
        output: &mut Vec<DisassembledSegment>,
    ) {
        // Emit segments in MIG order using tag-based lookup with consumption tracking.
        // This handles both assembler output (in MIG order) and reverse-mapped trees
        // (may be in arbitrary order).
        let mut consumed = vec![false; instance.segments.len()];
        for mig_seg in &mig_group.segments {
            if let Some(idx) = instance
                .segments
                .iter()
                .enumerate()
                .position(|(i, s)| !consumed[i] && s.tag == mig_seg.id)
            {
                output.push(assembled_to_disassembled(&instance.segments[idx]));
                consumed[idx] = true;
            }
        }

        // Child groups — lookup by group ID with consumption tracking
        let mut consumed_child = vec![false; instance.child_groups.len()];
        for nested_mig in &mig_group.nested_groups {
            if let Some(idx) = instance
                .child_groups
                .iter()
                .enumerate()
                .position(|(i, g)| !consumed_child[i] && g.group_id == nested_mig.id)
            {
                self.emit_group(&instance.child_groups[idx], nested_mig, output);
                consumed_child[idx] = true;
            }
        }
    }
}

fn assembled_to_disassembled(seg: &AssembledSegment) -> DisassembledSegment {
    DisassembledSegment {
        tag: seg.tag.clone(),
        elements: seg.elements.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::{
        AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree,
    };
    use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

    fn make_mig_segment(id: &str) -> MigSegment {
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

    fn make_mig_group(
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

    #[test]
    fn test_disassemble_top_level_only() {
        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "S2.1".to_string(),
            publication_date: "2025-03-20".to_string(),
            author: "BDEW".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![make_mig_segment("UNH"), make_mig_segment("BGM")],
            segment_groups: vec![],
        };

        let tree = AssembledTree {
            segments: vec![
                AssembledSegment {
                    tag: "UNH".to_string(),
                    elements: vec![
                        vec!["1".to_string()],
                        vec![
                            "UTILMD".to_string(),
                            "D".to_string(),
                            "11A".to_string(),
                            "UN".to_string(),
                            "S2.1".to_string(),
                        ],
                    ],
                },
                AssembledSegment {
                    tag: "BGM".to_string(),
                    elements: vec![
                        vec!["E01".to_string()],
                        vec!["MSG001".to_string()],
                        vec!["9".to_string()],
                    ],
                },
            ],
            groups: vec![],
            post_group_start: 2,
        };

        let disassembler = Disassembler::new(&mig);
        let segments = disassembler.disassemble(&tree);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].tag, "UNH");
        assert_eq!(segments[1].tag, "BGM");
        assert_eq!(segments[0].elements[0], vec!["1"]);
    }

    #[test]
    fn test_disassemble_with_groups() {
        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: "".to_string(),
            author: "".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![make_mig_segment("UNH"), make_mig_segment("BGM")],
            segment_groups: vec![make_mig_group("SG2", vec!["NAD", "LOC"], vec![])],
        };

        let tree = AssembledTree {
            segments: vec![
                AssembledSegment {
                    tag: "UNH".to_string(),
                    elements: vec![vec!["1".to_string()]],
                },
                AssembledSegment {
                    tag: "BGM".to_string(),
                    elements: vec![vec!["E01".to_string()]],
                },
            ],
            post_group_start: 2,
            groups: vec![AssembledGroup {
                group_id: "SG2".to_string(),
                repetitions: vec![
                    AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "NAD".to_string(),
                            elements: vec![vec!["MS".to_string()]],
                        }],
                        child_groups: vec![],
                    },
                    AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "NAD".to_string(),
                            elements: vec![vec!["MR".to_string()]],
                        }],
                        child_groups: vec![],
                    },
                ],
            }],
        };

        let disassembler = Disassembler::new(&mig);
        let segments = disassembler.disassemble(&tree);

        assert_eq!(segments.len(), 4); // UNH, BGM, NAD(MS), NAD(MR)
        assert_eq!(segments[0].tag, "UNH");
        assert_eq!(segments[1].tag, "BGM");
        assert_eq!(segments[2].tag, "NAD");
        assert_eq!(segments[2].elements[0][0], "MS");
        assert_eq!(segments[3].tag, "NAD");
        assert_eq!(segments[3].elements[0][0], "MR");
    }

    #[test]
    fn test_disassemble_nested_groups() {
        let sg3 = make_mig_group("SG3", vec!["CTA", "COM"], vec![]);
        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: "".to_string(),
            author: "".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![make_mig_segment("UNH")],
            segment_groups: vec![make_mig_group("SG2", vec!["NAD"], vec![sg3])],
        };

        let tree = AssembledTree {
            segments: vec![AssembledSegment {
                tag: "UNH".to_string(),
                elements: vec![vec!["1".to_string()]],
            }],
            post_group_start: 1,
            groups: vec![AssembledGroup {
                group_id: "SG2".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "NAD".to_string(),
                        elements: vec![vec!["MS".to_string()]],
                    }],
                    child_groups: vec![AssembledGroup {
                        group_id: "SG3".to_string(),
                        repetitions: vec![AssembledGroupInstance {
                            segments: vec![
                                AssembledSegment {
                                    tag: "CTA".to_string(),
                                    elements: vec![vec!["IC".to_string()]],
                                },
                                AssembledSegment {
                                    tag: "COM".to_string(),
                                    elements: vec![vec![
                                        "040@ex.com".to_string(),
                                        "EM".to_string(),
                                    ]],
                                },
                            ],
                            child_groups: vec![],
                        }],
                    }],
                }],
            }],
        };

        let disassembler = Disassembler::new(&mig);
        let segments = disassembler.disassemble(&tree);

        assert_eq!(segments.len(), 4); // UNH, NAD, CTA, COM
        assert_eq!(segments[0].tag, "UNH");
        assert_eq!(segments[1].tag, "NAD");
        assert_eq!(segments[2].tag, "CTA");
        assert_eq!(segments[3].tag, "COM");
    }

    #[test]
    fn test_disassemble_empty_tree() {
        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: "".to_string(),
            author: "".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: vec![make_mig_segment("UNH")],
            segment_groups: vec![],
        };

        let tree = AssembledTree {
            segments: vec![],
            groups: vec![],
            post_group_start: 0,
        };

        let disassembler = Disassembler::new(&mig);
        let segments = disassembler.disassemble(&tree);
        assert!(segments.is_empty());
    }
}
