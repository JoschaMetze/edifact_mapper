use crate::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

/// An ordered segment entry with group context, used for coordinator generation.
#[derive(Debug, Clone)]
pub struct OrderedSegmentEntry {
    /// The segment identifier (e.g., "NAD", "BGM").
    pub segment_id: String,
    /// The counter value for ordering (e.g., "0010", "0020").
    pub counter: String,
    /// The nesting level.
    pub level: i32,
    /// Maximum repetitions allowed.
    pub max_rep: i32,
    /// Whether the segment is optional.
    pub is_optional: bool,
    /// The containing group ID (None for top-level).
    pub group_id: Option<String>,
    /// The containing group's max repetitions.
    pub group_max_rep: i32,
}

/// Extracts all segments in MIG-defined order (by Counter attribute).
pub fn extract_ordered_segments(schema: &MigSchema) -> Vec<OrderedSegmentEntry> {
    let mut entries = Vec::new();

    // Add top-level segments
    for segment in &schema.segments {
        entries.push(create_entry(segment, None, 1));
    }

    // Add segments from groups (recursively)
    for group in &schema.segment_groups {
        extract_from_group(group, &mut entries);
    }

    // Sort by counter (numeric comparison)
    entries.sort_by_key(|e| parse_counter(&e.counter));

    entries
}

fn extract_from_group(group: &MigSegmentGroup, entries: &mut Vec<OrderedSegmentEntry>) {
    let group_max_rep = group.max_rep_std.max(group.max_rep_spec);

    for segment in &group.segments {
        entries.push(create_entry(segment, Some(&group.id), group_max_rep));
    }

    for nested in &group.nested_groups {
        extract_from_group(nested, entries);
    }
}

fn create_entry(
    segment: &MigSegment,
    group_id: Option<&str>,
    group_max_rep: i32,
) -> OrderedSegmentEntry {
    let status = segment
        .status_spec
        .as_deref()
        .or(segment.status_std.as_deref())
        .unwrap_or("C");
    let is_optional = !matches!(status, "M" | "R");
    let max_rep = segment.max_rep_std.max(segment.max_rep_spec);

    OrderedSegmentEntry {
        segment_id: segment.id.clone(),
        counter: segment
            .counter
            .clone()
            .unwrap_or_else(|| "0000".to_string()),
        level: segment.level,
        max_rep,
        is_optional,
        group_id: group_id.map(|s| s.to_string()),
        group_max_rep,
    }
}

fn parse_counter(counter: &str) -> i32 {
    counter.parse().unwrap_or(0)
}
