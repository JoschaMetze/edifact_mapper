use super::types::PidSchemaDiff;

/// Render a PidSchemaDiff as a human-readable markdown report.
pub fn render_diff_markdown(diff: &PidSchemaDiff) -> String {
    let mut md = String::new();

    md.push_str(&format!(
        "# PID Schema Diff: {} ({} → {})\n\n",
        diff.pid, diff.old_version, diff.new_version
    ));
    md.push_str(&format!("**Message type:** {}  \n", diff.message_type));
    if let Some(ref v) = diff.unh_version {
        md.push_str(&format!("**UNH version:** {} → {}  \n", v.old, v.new));
    }
    md.push_str("\n---\n\n");

    // Groups
    if !diff.groups.added.is_empty()
        || !diff.groups.removed.is_empty()
        || !diff.groups.restructured.is_empty()
    {
        md.push_str("## Group Changes\n\n");

        if !diff.groups.added.is_empty() {
            md.push_str("### Added Groups\n\n");
            md.push_str("| Group | Parent | Entry Segment |\n");
            md.push_str("|-------|--------|---------------|\n");
            for g in &diff.groups.added {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    g.group,
                    g.parent,
                    g.entry_segment.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }

        if !diff.groups.removed.is_empty() {
            md.push_str("### Removed Groups\n\n");
            md.push_str("| Group | Parent |\n");
            md.push_str("|-------|--------|\n");
            for g in &diff.groups.removed {
                md.push_str(&format!("| {} | {} |\n", g.group, g.parent));
            }
            md.push('\n');
        }

        if !diff.groups.restructured.is_empty() {
            md.push_str("### Restructured Groups (Manual Review Required)\n\n");
            for g in &diff.groups.restructured {
                md.push_str(&format!("- **{}**: {}\n", g.group, g.description));
            }
            md.push('\n');
        }
    }

    // Segments
    if !diff.segments.added.is_empty() || !diff.segments.removed.is_empty() {
        md.push_str("## Segment Changes\n\n");

        if !diff.segments.added.is_empty() {
            md.push_str("### Added Segments\n\n");
            md.push_str("| Segment | Group | Context |\n");
            md.push_str("|---------|-------|---------|\n");
            for s in &diff.segments.added {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    s.tag,
                    s.group,
                    s.context.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }

        if !diff.segments.removed.is_empty() {
            md.push_str("### Removed Segments\n\n");
            md.push_str("| Segment | Group | Context |\n");
            md.push_str("|---------|-------|---------|\n");
            for s in &diff.segments.removed {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    s.tag,
                    s.group,
                    s.context.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }
    }

    // Codes
    if !diff.codes.changed.is_empty() {
        md.push_str("## Code Changes\n\n");
        md.push_str("| Segment | Element | Group | Added | Removed |\n");
        md.push_str("|---------|---------|-------|-------|---------|\n");
        for c in &diff.codes.changed {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                c.segment,
                c.element,
                c.group,
                c.added.join(", "),
                c.removed.join(", "),
            ));
        }
        md.push('\n');
    }

    // Elements
    if !diff.elements.added.is_empty() || !diff.elements.removed.is_empty() {
        md.push_str("## Element Changes\n\n");

        if !diff.elements.added.is_empty() {
            md.push_str("### Added Elements\n\n");
            md.push_str("| Segment | Group | Index | Sub-Index | Description |\n");
            md.push_str("|---------|-------|-------|-----------|-------------|\n");
            for e in &diff.elements.added {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    e.segment,
                    e.group,
                    e.index,
                    e.sub_index
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "-".into()),
                    e.description.as_deref().unwrap_or("-"),
                ));
            }
            md.push('\n');
        }

        if !diff.elements.removed.is_empty() {
            md.push_str("### Removed Elements\n\n");
            md.push_str("| Segment | Group | Index | Sub-Index | Description |\n");
            md.push_str("|---------|-------|-------|-----------|-------------|\n");
            for e in &diff.elements.removed {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    e.segment,
                    e.group,
                    e.index,
                    e.sub_index
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "-".into()),
                    e.description.as_deref().unwrap_or("-"),
                ));
            }
            md.push('\n');
        }
    }

    if diff.is_empty() {
        md.push_str("**No differences found.**\n");
    }

    md
}
