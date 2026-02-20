//! PID-specific MIG filtering using AHB segment numbers.
//!
//! The MIG XML defines ALL possible segments/groups for a message type.
//! The AHB for a specific PID references a subset via the `Number` attribute.
//! This module filters a full MIG schema to produce a PID-specific one
//! that the assembler can use without ambiguity.

use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};
use std::collections::HashSet;

/// Filter a MIG schema to only include segments and groups whose
/// segment `number` fields appear in the given AHB number set.
///
/// This resolves ambiguity when the MIG has multiple definitions
/// of the same group (e.g., two SG4 variants for IDE+Z01 vs IDE+24)
/// by keeping only the variant(s) that match the PID's AHB.
pub fn filter_mig_for_pid(mig: &MigSchema, ahb_numbers: &HashSet<String>) -> MigSchema {
    MigSchema {
        message_type: mig.message_type.clone(),
        variant: mig.variant.clone(),
        version: mig.version.clone(),
        publication_date: mig.publication_date.clone(),
        author: mig.author.clone(),
        format_version: mig.format_version.clone(),
        source_file: mig.source_file.clone(),
        segments: filter_segments(&mig.segments, ahb_numbers),
        segment_groups: filter_groups(&mig.segment_groups, ahb_numbers),
    }
}

/// Interchange-level segment tags that are always kept regardless of AHB filtering.
/// The AHB only covers message content (UNH→UNT), not the interchange wrapper.
const TRANSPORT_SEGMENTS: &[&str] = &["UNA", "UNB", "UNZ"];

/// Filter top-level segments: keep transport segments (UNA, UNB, UNZ),
/// those with no Number, or those whose Number is in the AHB set.
fn filter_segments(segments: &[MigSegment], ahb_numbers: &HashSet<String>) -> Vec<MigSegment> {
    segments
        .iter()
        .filter(|seg| {
            if TRANSPORT_SEGMENTS.contains(&seg.id.as_str()) {
                return true;
            }
            match &seg.number {
                None => true,
                Some(num) => ahb_numbers.contains(num),
            }
        })
        .cloned()
        .collect()
}

/// Filter segment groups: keep a group if its entry segment's Number
/// is in the AHB set (or has no Number). Within kept groups, recursively
/// filter segments and nested groups.
fn filter_groups(
    groups: &[MigSegmentGroup],
    ahb_numbers: &HashSet<String>,
) -> Vec<MigSegmentGroup> {
    groups
        .iter()
        .filter(|group| group_matches_ahb(group, ahb_numbers))
        .map(|group| filter_group_contents(group, ahb_numbers))
        .collect()
}

/// Check if a group should be kept: its entry segment (first segment)
/// must have a Number in the AHB set, or no Number at all.
fn group_matches_ahb(group: &MigSegmentGroup, ahb_numbers: &HashSet<String>) -> bool {
    match group.segments.first() {
        None => false, // Empty group — skip
        Some(entry) => match &entry.number {
            None => true, // No Number on entry segment — keep
            Some(num) => ahb_numbers.contains(num),
        },
    }
}

/// For a kept group, filter its internal segments and nested groups.
fn filter_group_contents(
    group: &MigSegmentGroup,
    ahb_numbers: &HashSet<String>,
) -> MigSegmentGroup {
    MigSegmentGroup {
        id: group.id.clone(),
        name: group.name.clone(),
        description: group.description.clone(),
        counter: group.counter.clone(),
        level: group.level,
        max_rep_std: group.max_rep_std,
        max_rep_spec: group.max_rep_spec,
        status_std: group.status_std.clone(),
        status_spec: group.status_spec.clone(),
        segments: filter_segments(&group.segments, ahb_numbers),
        nested_groups: filter_groups(&group.nested_groups, ahb_numbers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automapper_generator::schema::mig::MigSegment;

    fn seg(id: &str, number: Option<&str>) -> MigSegment {
        MigSegment {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            counter: None,
            level: 0,
            number: number.map(|n| n.to_string()),
            max_rep_std: 1,
            max_rep_spec: 1,
            status_std: None,
            status_spec: None,
            example: None,
            data_elements: vec![],
            composites: vec![],
        }
    }

    fn group(id: &str, segments: Vec<MigSegment>, nested: Vec<MigSegmentGroup>) -> MigSegmentGroup {
        MigSegmentGroup {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            counter: None,
            level: 1,
            max_rep_std: 99,
            max_rep_spec: 99,
            status_std: None,
            status_spec: None,
            segments,
            nested_groups: nested,
        }
    }

    #[test]
    fn test_filter_selects_correct_sg4_variant() {
        // Simulate two SG4 variants like in real MIG
        let sg4_list = group("SG4", vec![seg("IDE", Some("00012"))], vec![]);
        let sg4_txn = group(
            "SG4",
            vec![
                seg("IDE", Some("00020")),
                seg("DTM", Some("00023")),
                seg("STS", Some("00035")),
            ],
            vec![
                group("SG5", vec![seg("LOC", Some("00049"))], vec![]),
                group("SG6", vec![seg("RFF", Some("00056"))], vec![]),
            ],
        );

        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: String::new(),
            author: String::new(),
            format_version: "FV2504".to_string(),
            source_file: String::new(),
            segments: vec![seg("UNH", Some("00003")), seg("BGM", Some("00004"))],
            segment_groups: vec![sg4_list, sg4_txn],
        };

        // PID 55001 references Numbers 00003, 00004, 00020, 00023, 00035, 00049, 00056
        let ahb_numbers: HashSet<String> = [
            "00003", "00004", "00020", "00023", "00035", "00049", "00056",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let filtered = filter_mig_for_pid(&mig, &ahb_numbers);

        // Should keep both top-level segments
        assert_eq!(filtered.segments.len(), 2);

        // Should keep only the transaction SG4 (00020), not the list SG4 (00012)
        assert_eq!(filtered.segment_groups.len(), 1);
        let sg4 = &filtered.segment_groups[0];
        assert_eq!(sg4.id, "SG4");
        assert_eq!(sg4.segments.len(), 3); // IDE, DTM, STS
        assert_eq!(sg4.nested_groups.len(), 2); // SG5, SG6
    }

    #[test]
    fn test_filter_keeps_transport_and_matching_segments() {
        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: String::new(),
            author: String::new(),
            format_version: "FV2504".to_string(),
            source_file: String::new(),
            segments: vec![
                seg("UNA", None),          // Transport — always keep
                seg("UNB", Some("00001")), // Transport — always keep
                seg("UNH", Some("00003")), // In AHB
                seg("DTM", Some("00099")), // Not in AHB
                seg("UNZ", Some("00527")), // Transport — always keep
            ],
            segment_groups: vec![],
        };

        let ahb_numbers: HashSet<String> = ["00003"].iter().map(|s| s.to_string()).collect();
        let filtered = filter_mig_for_pid(&mig, &ahb_numbers);

        // UNA, UNB, UNH, UNZ kept; DTM filtered out
        assert_eq!(filtered.segments.len(), 4);
        assert_eq!(filtered.segments[0].id, "UNA");
        assert_eq!(filtered.segments[1].id, "UNB");
        assert_eq!(filtered.segments[2].id, "UNH");
        assert_eq!(filtered.segments[3].id, "UNZ");
    }

    #[test]
    fn test_filter_removes_nested_groups_not_in_ahb() {
        let sg8_z79 = group(
            "SG8",
            vec![seg("SEQ", Some("00081"))],
            vec![group("SG10", vec![seg("CCI", Some("00083"))], vec![])],
        );
        let sg8_z99 = group(
            "SG8",
            vec![seg("SEQ", Some("00999"))], // Not in AHB
            vec![],
        );

        let sg4 = group(
            "SG4",
            vec![seg("IDE", Some("00020"))],
            vec![sg8_z79, sg8_z99],
        );

        let mig = MigSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "S2.1".to_string(),
            publication_date: String::new(),
            author: String::new(),
            format_version: "FV2504".to_string(),
            source_file: String::new(),
            segments: vec![],
            segment_groups: vec![sg4],
        };

        let ahb_numbers: HashSet<String> = ["00020", "00081", "00083"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let filtered = filter_mig_for_pid(&mig, &ahb_numbers);

        let sg4 = &filtered.segment_groups[0];
        // Only the SG8 with SEQ Number 00081 should survive
        assert_eq!(sg4.nested_groups.len(), 1);
        assert_eq!(
            sg4.nested_groups[0].segments[0].number,
            Some("00081".to_string())
        );
    }
}
