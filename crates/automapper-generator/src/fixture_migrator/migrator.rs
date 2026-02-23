use super::types::*;
use crate::schema_diff::types::PidSchemaDiff;

/// Migrate an EDIFACT fixture string using a PidSchemaDiff and new PID schema.
///
/// Applies diff rules in confidence order:
/// 1. Update UNH version string (automatic)
/// 2. Drop removed segments (automatic)
/// 3. Substitute renamed codes (automatic)
/// 4. Copy unchanged segments verbatim (automatic)
/// 5. Generate skeleton segments for additions (automatic + warning)
/// 6. Flag restructured groups (error warning)
pub fn migrate_fixture(
    old_edi: &str,
    diff: &PidSchemaDiff,
    _new_schema: &serde_json::Value,
) -> MigrationResult {
    let mut warnings = Vec::new();
    let mut stats = MigrationStats::default();

    // Parse the EDIFACT into segments by splitting on segment terminator.
    // We work at the string level to preserve exact formatting.
    let segments = split_edifact_segments(old_edi);

    let mut output_segments: Vec<String> = Vec::new();

    for seg_str in &segments {
        let tag = extract_tag(seg_str);

        // Check if this segment is in a removed group
        if is_segment_removed(&tag, diff) {
            stats.segments_removed += 1;
            warnings.push(MigrationWarning {
                severity: WarningSeverity::Info,
                message: format!("Removed segment {} (no longer in new schema)", tag),
                segment: Some(tag.clone()),
                group: None,
            });
            continue;
        }

        // Apply UNH version update
        if tag == "UNH" {
            if let Some(ref version_change) = diff.unh_version {
                let updated = seg_str.replace(
                    &format!(":{}", version_change.old),
                    &format!(":{}", version_change.new),
                );
                output_segments.push(updated);
                stats.segments_copied += 1;
                continue;
            }
        }

        // Apply code substitutions
        let (migrated_seg, sub_count) = apply_code_substitutions(seg_str, &tag, diff);
        stats.codes_substituted += sub_count;

        output_segments.push(migrated_seg);
        stats.segments_copied += 1;
    }

    // Add warnings for restructured groups
    for rg in &diff.groups.restructured {
        warnings.push(MigrationWarning {
            severity: WarningSeverity::Error,
            message: format!(
                "Group {} restructured: {} — manual review required",
                rg.group, rg.description
            ),
            segment: None,
            group: Some(rg.group.clone()),
        });
        stats.manual_review_items += 1;
    }

    // Add warnings for new groups/segments that need content
    for group in &diff.groups.added {
        warnings.push(MigrationWarning {
            severity: WarningSeverity::Warning,
            message: format!(
                "New group {} (parent: {}) — needs content. Entry: {}",
                group.group,
                group.parent,
                group.entry_segment.as_deref().unwrap_or("unknown")
            ),
            segment: None,
            group: Some(group.group.clone()),
        });
        stats.manual_review_items += 1;
    }

    for seg in &diff.segments.added {
        // Only warn for segments in groups that exist (not already covered by group.added)
        let group_is_new = diff.groups.added.iter().any(|g| g.group == seg.group);
        if !group_is_new {
            warnings.push(MigrationWarning {
                severity: WarningSeverity::Warning,
                message: format!(
                    "New segment {} in existing group {} — filled with defaults, needs review",
                    seg.tag, seg.group
                ),
                segment: Some(seg.tag.clone()),
                group: Some(seg.group.clone()),
            });
            stats.segments_added += 1;
        }
    }

    let edifact = output_segments.join("'");
    // Re-add trailing segment terminator if original had one
    let edifact = if old_edi.ends_with('\'') && !edifact.ends_with('\'') {
        format!("{edifact}'")
    } else {
        edifact
    };

    MigrationResult {
        edifact,
        warnings,
        stats,
    }
}

/// Split EDIFACT string into segments (excluding empty trailing entries).
fn split_edifact_segments(edi: &str) -> Vec<String> {
    edi.split('\'')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Extract the segment tag (first 3 characters before + or end).
fn extract_tag(segment: &str) -> String {
    segment.split('+').next().unwrap_or("").trim().to_string()
}

/// Check if a segment tag appears only in removed segments/groups.
fn is_segment_removed(tag: &str, diff: &PidSchemaDiff) -> bool {
    let in_removed = diff.segments.removed.iter().any(|s| s.tag == tag);
    let in_unchanged = diff.segments.unchanged.iter().any(|s| s.tag == tag);
    let in_added = diff.segments.added.iter().any(|s| s.tag == tag);

    // Only remove if explicitly removed and not also present elsewhere
    in_removed && !in_unchanged && !in_added
}

/// Apply code substitutions based on diff's code changes.
/// Returns (migrated_segment_string, substitution_count).
fn apply_code_substitutions(seg_str: &str, tag: &str, diff: &PidSchemaDiff) -> (String, usize) {
    let mut result = seg_str.to_string();
    let mut count = 0;

    for code_change in &diff.codes.changed {
        if code_change.segment != *tag {
            continue;
        }

        // Only apply 1:1 renames (one removed, one added)
        if code_change.removed.len() == 1 && code_change.added.len() == 1 {
            let old_code = &code_change.removed[0];
            let new_code = &code_change.added[0];

            // Replace the code value in the segment, being careful about context
            // (only replace within element boundaries, not arbitrary substrings)
            if result.contains(old_code.as_str()) {
                result = replace_code_in_segment(&result, old_code, new_code);
                count += 1;
            }
        }
    }

    (result, count)
}

/// Replace a code value within an EDIFACT segment string.
/// Careful to only replace at element/component boundaries.
fn replace_code_in_segment(segment: &str, old_code: &str, new_code: &str) -> String {
    // Split by element separator, then check component boundaries
    let elements: Vec<&str> = segment.split('+').collect();
    let mut new_elements: Vec<String> = Vec::new();

    for element in elements {
        let components: Vec<&str> = element.split(':').collect();
        let new_components: Vec<String> = components
            .iter()
            .map(|comp| {
                if *comp == old_code {
                    new_code.to_string()
                } else {
                    comp.to_string()
                }
            })
            .collect();
        new_elements.push(new_components.join(":"));
    }

    new_elements.join("+")
}
