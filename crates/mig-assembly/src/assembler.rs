//! Recursive descent assembler — MIG-guided segment consumption.
//!
//! The assembler walks the MIG tree structure and consumes matching
//! segments from the input. It produces a generic tree representation
//! that can be converted to typed PID structs.

use crate::cursor::SegmentCursor;
use crate::matcher;
use crate::tokenize::OwnedSegment;
use crate::AssemblyError;
use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};
use serde::{Deserialize, Serialize};

/// A generic assembled tree node (before PID-specific typing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledTree {
    pub segments: Vec<AssembledSegment>,
    pub groups: Vec<AssembledGroup>,
    /// Index in `segments` where post-group segments start (e.g., UNT, UNZ).
    /// Segments before this index appear before groups in EDIFACT order.
    #[serde(default)]
    pub post_group_start: usize,
}

/// An assembled segment with its data elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledSegment {
    pub tag: String,
    /// `elements[i][j]` = component `j` of element `i`
    pub elements: Vec<Vec<String>>,
}

/// An assembled segment group (may repeat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledGroup {
    pub group_id: String,
    pub repetitions: Vec<AssembledGroupInstance>,
}

/// One repetition of a segment group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledGroupInstance {
    pub segments: Vec<AssembledSegment>,
    pub child_groups: Vec<AssembledGroup>,
}

impl AssembledGroupInstance {
    /// Create a virtual `AssembledTree` scoped to this group instance.
    ///
    /// The instance's own segments become the tree's root segments,
    /// and its child groups become the tree's groups. This enables
    /// running `MappingEngine::map_all_forward()` on a single
    /// transaction group as if it were a complete message.
    pub fn as_assembled_tree(&self) -> AssembledTree {
        AssembledTree {
            segments: self.segments.clone(),
            groups: self.child_groups.clone(),
            post_group_start: self.segments.len(),
        }
    }
}

/// MIG-guided assembler.
///
/// Takes a MIG schema and uses it as a grammar to guide consumption
/// of parsed EDIFACT segments. Produces a generic `AssembledTree`.
pub struct Assembler<'a> {
    mig: &'a MigSchema,
}

impl<'a> Assembler<'a> {
    pub fn new(mig: &'a MigSchema) -> Self {
        Self { mig }
    }

    /// Assemble segments into a generic tree following MIG structure.
    pub fn assemble_generic(
        &self,
        segments: &[OwnedSegment],
    ) -> Result<AssembledTree, AssemblyError> {
        let mut cursor = SegmentCursor::new(segments.len());
        let mut tree = AssembledTree {
            segments: Vec::new(),
            groups: Vec::new(),
            post_group_start: 0,
        };

        // Track which MIG segment indices were matched in the first pass
        let mut matched_seg_indices = Vec::new();

        // Process top-level segments (first pass — before groups)
        for (i, mig_seg) in self.mig.segments.iter().enumerate() {
            if cursor.is_exhausted() {
                break;
            }
            if let Some(assembled) = self.try_consume_segment(segments, &mut cursor, mig_seg)? {
                tree.segments.push(assembled);
                matched_seg_indices.push(i);
            }
        }

        // Process segment groups
        for mig_group in &self.mig.segment_groups {
            if cursor.is_exhausted() {
                break;
            }
            if let Some(assembled) = self.try_consume_group(segments, &mut cursor, mig_group)? {
                tree.groups.push(assembled);
            }
        }

        // Mark where post-group segments start
        tree.post_group_start = tree.segments.len();

        // Second pass: try unmatched top-level segments (e.g., UNT, UNZ after groups)
        for (i, mig_seg) in self.mig.segments.iter().enumerate() {
            if cursor.is_exhausted() {
                break;
            }
            if matched_seg_indices.contains(&i) {
                continue;
            }
            if let Some(assembled) = self.try_consume_segment(segments, &mut cursor, mig_seg)? {
                tree.segments.push(assembled);
            }
        }

        Ok(tree)
    }

    fn try_consume_segment(
        &self,
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
        mig_seg: &MigSegment,
    ) -> Result<Option<AssembledSegment>, AssemblyError> {
        if cursor.is_exhausted() {
            return Ok(None);
        }
        let seg = &segments[cursor.position()];
        if matcher::matches_segment_tag(&seg.id, &mig_seg.id) {
            let assembled = owned_to_assembled(seg);
            cursor.advance();
            Ok(Some(assembled))
        } else {
            Ok(None) // Segment not present (optional)
        }
    }

    fn try_consume_group(
        &self,
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
        mig_group: &MigSegmentGroup,
    ) -> Result<Option<AssembledGroup>, AssemblyError> {
        let mut repetitions = Vec::new();
        let entry_segment = mig_group.segments.first().ok_or_else(|| {
            AssemblyError::ParseError(format!("Group {} has no segments", mig_group.id))
        })?;

        // Loop for repeating groups
        while !cursor.is_exhausted() {
            let seg = &segments[cursor.position()];
            if !matcher::matches_segment_tag(&seg.id, &entry_segment.id) {
                break; // Current segment doesn't match group entry — stop repeating
            }

            let mut instance = AssembledGroupInstance {
                segments: Vec::new(),
                child_groups: Vec::new(),
            };

            // Consume segments within this group instance
            for group_seg in &mig_group.segments {
                if cursor.is_exhausted() {
                    break;
                }
                if let Some(assembled) = self.try_consume_segment(segments, cursor, group_seg)? {
                    instance.segments.push(assembled);
                }
            }

            // Consume nested groups
            for nested in &mig_group.nested_groups {
                if cursor.is_exhausted() {
                    break;
                }
                if let Some(assembled) = self.try_consume_group(segments, cursor, nested)? {
                    instance.child_groups.push(assembled);
                }
            }

            repetitions.push(instance);
        }

        if repetitions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AssembledGroup {
                group_id: mig_group.id.clone(),
                repetitions,
            }))
        }
    }
}

fn owned_to_assembled(seg: &OwnedSegment) -> AssembledSegment {
    AssembledSegment {
        tag: seg.id.clone(),
        elements: seg.elements.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn make_owned_seg(id: &str, elements: Vec<Vec<&str>>) -> OwnedSegment {
        OwnedSegment {
            id: id.to_string(),
            elements: elements
                .into_iter()
                .map(|e| e.into_iter().map(|c| c.to_string()).collect())
                .collect(),
            segment_number: 0,
        }
    }

    fn make_mig_schema(segments: Vec<&str>, groups: Vec<MigSegmentGroup>) -> MigSchema {
        MigSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "S2.1".to_string(),
            publication_date: "2025-03-20".to_string(),
            author: "BDEW".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test".to_string(),
            segments: segments.into_iter().map(make_mig_segment).collect(),
            segment_groups: groups,
        }
    }

    #[test]
    fn test_assembler_top_level_segments_only() {
        let mig = make_mig_schema(vec!["UNH", "BGM", "DTM", "UNT"], vec![]);

        let segments = vec![
            make_owned_seg("UNH", vec![vec!["001", "UTILMD:D:11A:UN:S2.1"]]),
            make_owned_seg("BGM", vec![vec!["E01", "DOC001"]]),
            make_owned_seg("DTM", vec![vec!["137", "20250101", "102"]]),
            make_owned_seg("UNT", vec![vec!["4", "001"]]),
        ];

        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&segments).unwrap();

        assert_eq!(result.segments.len(), 4);
        assert_eq!(result.segments[0].tag, "UNH");
        assert_eq!(result.segments[1].tag, "BGM");
        assert_eq!(result.segments[2].tag, "DTM");
        assert_eq!(result.segments[3].tag, "UNT");
        assert!(result.groups.is_empty());
    }

    #[test]
    fn test_assembler_with_segment_group() {
        let mig = make_mig_schema(
            vec!["UNH", "BGM"],
            vec![
                make_mig_group("SG2", vec!["NAD"], vec![]),
                make_mig_group("SG4", vec!["IDE", "STS"], vec![]),
            ],
        );

        let segments = vec![
            make_owned_seg("UNH", vec![vec!["001"]]),
            make_owned_seg("BGM", vec![vec!["E01"]]),
            make_owned_seg("NAD", vec![vec!["MS", "9900123"]]),
            make_owned_seg("NAD", vec![vec!["MR", "9900456"]]),
            make_owned_seg("IDE", vec![vec!["24", "TX001"]]),
            make_owned_seg("STS", vec![vec!["7"], vec!["Z33"]]),
        ];

        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&segments).unwrap();

        // Top-level: UNH, BGM
        assert_eq!(result.segments.len(), 2);
        // SG2: 2 repetitions (two NAD segments)
        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.groups[0].group_id, "SG2");
        assert_eq!(result.groups[0].repetitions.len(), 2);
        assert_eq!(result.groups[0].repetitions[0].segments[0].tag, "NAD");
        assert_eq!(result.groups[0].repetitions[1].segments[0].tag, "NAD");
        // SG4: 1 repetition (IDE + STS)
        assert_eq!(result.groups[1].group_id, "SG4");
        assert_eq!(result.groups[1].repetitions.len(), 1);
        assert_eq!(result.groups[1].repetitions[0].segments.len(), 2);
    }

    #[test]
    fn test_assembler_nested_groups() {
        let sg3 = make_mig_group("SG3", vec!["CTA", "COM"], vec![]);
        let mig = make_mig_schema(
            vec!["UNH", "BGM"],
            vec![make_mig_group("SG2", vec!["NAD"], vec![sg3])],
        );

        let segments = vec![
            make_owned_seg("UNH", vec![vec!["001"]]),
            make_owned_seg("BGM", vec![vec!["E01"]]),
            make_owned_seg("NAD", vec![vec!["MS", "9900123"]]),
            make_owned_seg("CTA", vec![vec!["IC", "Kontakt"]]),
            make_owned_seg("COM", vec![vec!["040@example.com", "EM"]]),
        ];

        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&segments).unwrap();

        // SG2 has 1 repetition
        let sg2 = &result.groups[0];
        assert_eq!(sg2.group_id, "SG2");
        assert_eq!(sg2.repetitions.len(), 1);

        let sg2_inst = &sg2.repetitions[0];
        assert_eq!(sg2_inst.segments[0].tag, "NAD");

        // SG3 nested inside SG2
        assert_eq!(sg2_inst.child_groups.len(), 1);
        let sg3 = &sg2_inst.child_groups[0];
        assert_eq!(sg3.group_id, "SG3");
        assert_eq!(sg3.repetitions[0].segments.len(), 2);
        assert_eq!(sg3.repetitions[0].segments[0].tag, "CTA");
        assert_eq!(sg3.repetitions[0].segments[1].tag, "COM");
    }

    #[test]
    fn test_assembler_optional_segments_skipped() {
        // MIG expects UNH, BGM, DTM, UNT but input has no DTM
        let mig = make_mig_schema(vec!["UNH", "BGM", "DTM", "UNT"], vec![]);

        let segments = vec![
            make_owned_seg("UNH", vec![vec!["001"]]),
            make_owned_seg("BGM", vec![vec!["E01"]]),
            make_owned_seg("UNT", vec![vec!["2", "001"]]),
        ];

        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&segments).unwrap();

        // DTM is skipped (optional), UNT consumed
        assert_eq!(result.segments.len(), 3);
        assert_eq!(result.segments[0].tag, "UNH");
        assert_eq!(result.segments[1].tag, "BGM");
        assert_eq!(result.segments[2].tag, "UNT");
    }

    #[test]
    fn test_assembler_empty_segments() {
        let mig = make_mig_schema(vec!["UNH"], vec![]);
        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&[]).unwrap();
        assert!(result.segments.is_empty());
        assert!(result.groups.is_empty());
    }

    #[test]
    fn test_assembler_preserves_element_data() {
        let mig = make_mig_schema(vec!["DTM"], vec![]);

        let segments = vec![make_owned_seg(
            "DTM",
            vec![vec!["137", "202501010000+01", "303"]],
        )];

        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&segments).unwrap();

        let dtm = &result.segments[0];
        assert_eq!(dtm.elements[0][0], "137");
        assert_eq!(dtm.elements[0][1], "202501010000+01");
        assert_eq!(dtm.elements[0][2], "303");
    }

    #[test]
    fn test_group_instance_as_assembled_tree() {
        // Build an SG4 instance with root segments (IDE, STS) and child groups (SG5)
        let sg5 = AssembledGroup {
            group_id: "SG5".to_string(),
            repetitions: vec![AssembledGroupInstance {
                segments: vec![AssembledSegment {
                    tag: "LOC".to_string(),
                    elements: vec![vec!["Z16".to_string(), "DE000111222333".to_string()]],
                }],
                child_groups: vec![],
            }],
        };

        let sg4_instance = AssembledGroupInstance {
            segments: vec![
                AssembledSegment {
                    tag: "IDE".to_string(),
                    elements: vec![vec!["24".to_string(), "TX001".to_string()]],
                },
                AssembledSegment {
                    tag: "STS".to_string(),
                    elements: vec![vec!["7".to_string()]],
                },
            ],
            child_groups: vec![sg5],
        };

        let sub_tree = sg4_instance.as_assembled_tree();

        // Root segments of sub-tree are the SG4 instance's segments
        assert_eq!(sub_tree.segments.len(), 2);
        assert_eq!(sub_tree.segments[0].tag, "IDE");
        assert_eq!(sub_tree.segments[1].tag, "STS");

        // Groups of sub-tree are the SG4 instance's child groups
        assert_eq!(sub_tree.groups.len(), 1);
        assert_eq!(sub_tree.groups[0].group_id, "SG5");

        // post_group_start marks where root segments end
        assert_eq!(sub_tree.post_group_start, 2);
    }

    #[test]
    fn test_assembler_from_parsed_edifact() {
        // End-to-end: parse raw EDIFACT, then assemble
        let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001+9'DTM+137:20250101:102'UNT+3+MSG001'UNZ+1+REF001'";
        let segments = crate::tokenize::parse_to_segments(input).unwrap();

        let mig = make_mig_schema(vec!["UNB", "UNH", "BGM", "DTM", "UNT", "UNZ"], vec![]);

        let assembler = Assembler::new(&mig);
        let result = assembler.assemble_generic(&segments).unwrap();

        assert!(result.segments.iter().any(|s| s.tag == "UNH"));
        assert!(result.segments.iter().any(|s| s.tag == "BGM"));
        assert!(result.segments.iter().any(|s| s.tag == "DTM"));
    }
}
