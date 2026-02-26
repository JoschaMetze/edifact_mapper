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
/// filter segments and nested groups. Then merge groups with the same ID
/// into a single group definition so the assembler can handle all
/// repetitions of a group type (e.g., multiple SG8 SEQ variants).
fn filter_groups(
    groups: &[MigSegmentGroup],
    ahb_numbers: &HashSet<String>,
) -> Vec<MigSegmentGroup> {
    let filtered: Vec<MigSegmentGroup> = groups
        .iter()
        .filter(|group| group_matches_ahb(group, ahb_numbers))
        .map(|group| filter_group_contents(group, ahb_numbers))
        .collect();
    merge_same_id_groups(filtered)
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

/// Merge groups with the same ID into a single group definition.
///
/// After PID filtering, the MIG may contain multiple groups with the same ID
/// but different variants (e.g., 4 SG8 definitions for SEQ+ZD7, Z98, ZF1, ZF3).
/// The assembler matches groups by entry tag only and would consume all repetitions
/// under the first definition. Merging produces a single group per ID with the union
/// of all segments and nested groups, enabling the assembler to handle all variants.
fn merge_same_id_groups(groups: Vec<MigSegmentGroup>) -> Vec<MigSegmentGroup> {
    // Collect groups by ID, preserving first-seen order
    let mut seen_ids: Vec<String> = Vec::new();
    let mut by_id: Vec<Vec<MigSegmentGroup>> = Vec::new();

    for g in groups {
        if let Some(pos) = seen_ids.iter().position(|id| *id == g.id) {
            by_id[pos].push(g);
        } else {
            seen_ids.push(g.id.clone());
            by_id.push(vec![g]);
        }
    }

    by_id
        .into_iter()
        .map(|variants| {
            if variants.len() == 1 {
                variants.into_iter().next().unwrap()
            } else {
                merge_group_variants(variants)
            }
        })
        .collect()
}

/// Merge multiple variants of the same group into one.
///
/// Segments are unioned by tag: for each tag, include the maximum number of
/// occurrences found in any single variant (e.g., if one variant has 2 CAV
/// segments and others have 1, the merged group has 2). Order is preserved
/// from the first variant that introduces each tag.
///
/// Nested groups are collected from all variants and recursively merged.
/// They are sorted by numeric suffix (SG9 before SG10) to maintain
/// standard EDIFACT group ordering.
fn merge_group_variants(variants: Vec<MigSegmentGroup>) -> MigSegmentGroup {
    let first = &variants[0];

    // Merge segments: for each tag, keep max count across variants
    let mut merged_segments: Vec<MigSegment> = Vec::new();
    for variant in &variants {
        for seg in &variant.segments {
            let count_in_merged = merged_segments.iter().filter(|s| s.id == seg.id).count();
            let count_in_variant = variant.segments.iter().filter(|s| s.id == seg.id).count();
            if count_in_variant > count_in_merged {
                for _ in 0..(count_in_variant - count_in_merged) {
                    merged_segments.push(seg.clone());
                }
            }
        }
    }

    // Collect all nested groups from all variants and recursively merge
    let mut all_nested: Vec<MigSegmentGroup> = Vec::new();
    for variant in &variants {
        all_nested.extend(variant.nested_groups.iter().cloned());
    }
    let mut merged_nested = merge_same_id_groups(all_nested);
    // Sort by group number (SG9 < SG10) for standard EDIFACT ordering
    merged_nested.sort_by_key(|g| extract_group_number(&g.id));

    MigSegmentGroup {
        id: first.id.clone(),
        name: first.name.clone(),
        description: first.description.clone(),
        counter: first.counter.clone(),
        level: first.level,
        max_rep_std: variants.iter().map(|v| v.max_rep_std).max().unwrap_or(1),
        max_rep_spec: variants.iter().map(|v| v.max_rep_spec).max().unwrap_or(1),
        status_std: first.status_std.clone(),
        status_spec: first.status_spec.clone(),
        segments: merged_segments,
        nested_groups: merged_nested,
    }
}

/// Extract numeric suffix from a group ID (e.g., "SG10" → 10).
fn extract_group_number(id: &str) -> u32 {
    id.strip_prefix("SG")
        .and_then(|s| s.parse().ok())
        .unwrap_or(u32::MAX)
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
    fn test_filter_merges_same_id_groups() {
        // Simulate 3 SG8 variants (ZD7, Z98, ZF1) like in PID 55013
        let sg8_zd7 = group(
            "SG8",
            vec![seg("SEQ", Some("00089")), seg("RFF", Some("00090"))],
            vec![group(
                "SG10",
                vec![seg("CCI", Some("00092")), seg("CAV", Some("00093"))],
                vec![],
            )],
        );
        let sg8_z98 = group(
            "SG8",
            vec![seg("SEQ", Some("00114"))],
            vec![
                group("SG9", vec![seg("QTY", Some("00116"))], vec![]),
                group(
                    "SG10",
                    vec![seg("CCI", Some("00122")), seg("CAV", Some("00125"))],
                    vec![],
                ),
            ],
        );
        let sg8_zf3 = group(
            "SG8",
            vec![seg("SEQ", Some("00291")), seg("RFF", Some("00292"))],
            vec![group(
                "SG10",
                vec![
                    seg("CCI", Some("00295")),
                    seg("CAV", Some("00296")),
                    seg("CAV", Some("00297")),
                ],
                vec![],
            )],
        );

        let sg4 = group(
            "SG4",
            vec![seg("IDE", Some("00020"))],
            vec![sg8_zd7, sg8_z98, sg8_zf3],
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

        let ahb_numbers: HashSet<String> = [
            "00020", "00089", "00090", "00092", "00093", "00114", "00116", "00122", "00125",
            "00291", "00292", "00295", "00296", "00297",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let filtered = filter_mig_for_pid(&mig, &ahb_numbers);
        let sg4 = &filtered.segment_groups[0];

        // All 3 SG8 variants should be merged into 1
        assert_eq!(sg4.nested_groups.len(), 1, "SG8 variants should be merged");
        let sg8 = &sg4.nested_groups[0];
        assert_eq!(sg8.id, "SG8");

        // Merged segments: SEQ (from all) + RFF (from ZD7/ZF3)
        let seg_tags: Vec<&str> = sg8.segments.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(seg_tags, vec!["SEQ", "RFF"]);

        // Merged nested groups: SG9 (from Z98) + SG10 (from all), sorted by number
        assert_eq!(sg8.nested_groups.len(), 2, "should have SG9 and SG10");
        assert_eq!(sg8.nested_groups[0].id, "SG9");
        assert_eq!(sg8.nested_groups[1].id, "SG10");

        // SG10 should have merged segments: CCI + 2 CAVs (max from ZF3)
        let sg10_tags: Vec<&str> = sg8.nested_groups[1]
            .segments
            .iter()
            .map(|s| s.id.as_str())
            .collect();
        assert_eq!(sg10_tags, vec!["CCI", "CAV", "CAV"]);
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
