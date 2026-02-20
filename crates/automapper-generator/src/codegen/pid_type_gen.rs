//! Code generation for PID-specific composition types.
//!
//! Cross-references AHB field definitions against the MIG tree
//! to determine which segment groups, segments, and fields
//! exist for each PID.

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::ahb::{AhbSchema, Pruefidentifikator};
use crate::schema::mig::{MigSchema, MigSegmentGroup};

/// Analyzed structure of a single PID.
#[derive(Debug, Clone)]
pub struct PidStructure {
    pub pid_id: String,
    pub beschreibung: String,
    pub kommunikation_von: Option<String>,
    /// Top-level groups present in this PID.
    pub groups: Vec<PidGroupInfo>,
    /// Top-level segments (outside groups) present in this PID.
    pub top_level_segments: Vec<String>,
}

/// Information about a segment group's usage within a PID.
#[derive(Debug, Clone)]
pub struct PidGroupInfo {
    pub group_id: String,
    /// Qualifier values that disambiguate this group (e.g., "ZD5", "ZD6").
    /// Empty if the group is not qualifier-disambiguated.
    pub qualifier_values: Vec<String>,
    /// AHB status for this group occurrence ("Muss", "Kann", etc.)
    pub ahb_status: String,
    /// Human-readable AHB-derived field name (e.g., "Absender", "Summenzeitreihe Arbeit/Leistung").
    pub ahb_name: Option<String>,
    /// Trigger segment + data element for qualifier discrimination.
    /// E.g., ("NAD", "3035") for SG2, ("SEQ", "1229") for SG8.
    pub discriminator: Option<(String, String)>,
    /// Nested child groups present in this PID's usage.
    pub child_groups: Vec<PidGroupInfo>,
    /// Segments present in this group for this PID.
    pub segments: BTreeSet<String>,
}

/// Analyze which MIG tree nodes a PID uses, based on its AHB field definitions.
pub fn analyze_pid_structure(pid: &Pruefidentifikator, _mig: &MigSchema) -> PidStructure {
    let mut top_level_segments: BTreeSet<String> = BTreeSet::new();
    let mut group_map: BTreeMap<String, PidGroupInfo> = BTreeMap::new();

    for field in &pid.fields {
        let parts: Vec<&str> = field.segment_path.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        if parts[0].starts_with("SG") {
            let group_id = parts[0].to_string();
            let entry = group_map
                .entry(group_id.clone())
                .or_insert_with(|| PidGroupInfo {
                    group_id: group_id.clone(),
                    qualifier_values: Vec::new(),
                    ahb_status: field.ahb_status.clone(),
                    ahb_name: None,
                    discriminator: None,
                    child_groups: Vec::new(),
                    segments: BTreeSet::new(),
                });

            if parts.len() > 1 && !parts[1].starts_with("SG") {
                entry.segments.insert(parts[1].to_string());
            }

            // Handle nested groups (SG4/SG8/...)
            if parts.len() > 1 && parts[1].starts_with("SG") {
                let child_id = parts[1].to_string();
                if !entry.child_groups.iter().any(|c| c.group_id == child_id) {
                    let mut child_segments = BTreeSet::new();
                    if parts.len() > 2 && !parts[2].starts_with("SG") {
                        child_segments.insert(parts[2].to_string());
                    }
                    entry.child_groups.push(PidGroupInfo {
                        group_id: child_id,
                        qualifier_values: Vec::new(),
                        ahb_status: field.ahb_status.clone(),
                        ahb_name: None,
                        discriminator: None,
                        child_groups: Vec::new(),
                        segments: child_segments,
                    });
                } else if parts.len() > 2 && !parts[2].starts_with("SG") {
                    if let Some(child) = entry
                        .child_groups
                        .iter_mut()
                        .find(|c| c.group_id == child_id)
                    {
                        child.segments.insert(parts[2].to_string());
                    }
                }
            }
        } else {
            top_level_segments.insert(parts[0].to_string());
        }
    }

    PidStructure {
        pid_id: pid.id.clone(),
        beschreibung: pid.beschreibung.clone(),
        kommunikation_von: pid.kommunikation_von.clone(),
        groups: group_map.into_values().collect(),
        top_level_segments: top_level_segments.into_iter().collect(),
    }
}

/// Find the trigger segment and qualifying data element for a group from the MIG.
///
/// Returns `(segment_id, data_element_id)` — e.g., `("SEQ", "1229")` for SG8.
fn find_group_qualifier(group_id: &str, mig: &MigSchema) -> Option<(String, String)> {
    fn find_in_group(target_id: &str, group: &MigSegmentGroup) -> Option<(String, String)> {
        if group.id == target_id {
            if let Some(seg) = group.segments.first() {
                for de in &seg.data_elements {
                    if !de.codes.is_empty() {
                        return Some((seg.id.clone(), de.id.clone()));
                    }
                }
                for comp in &seg.composites {
                    for de in &comp.data_elements {
                        if !de.codes.is_empty() {
                            return Some((seg.id.clone(), de.id.clone()));
                        }
                    }
                }
            }
            return None;
        }
        for nested in &group.nested_groups {
            if let Some(result) = find_in_group(target_id, nested) {
                return Some(result);
            }
        }
        None
    }

    for group in &mig.segment_groups {
        if let Some(result) = find_in_group(group_id, group) {
            return Some(result);
        }
    }
    None
}

/// Derive the AHB field name for a group from its entry segment's AHB definition.
///
/// Looks for the AHB field that references this group's entry segment path
/// (e.g., "SG2/NAD" for SG2, "SG4/SG8/SEQ" for SG8 under SG4).
fn derive_ahb_name(
    pid: &Pruefidentifikator,
    group_path: &str,
    entry_segment: &str,
) -> Option<String> {
    let target_prefix = format!("{}/{}", group_path, entry_segment);
    pid.fields
        .iter()
        .find(|f| f.segment_path.starts_with(&target_prefix))
        .and_then(|f| {
            let name = f.name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
}

/// Populate discriminator and ahb_name on top-level groups and their children.
fn enrich_group_info(group: &mut PidGroupInfo, pid: &Pruefidentifikator, mig: &MigSchema, parent_path: &str) {
    let group_path = if parent_path.is_empty() {
        group.group_id.clone()
    } else {
        format!("{}/{}", parent_path, group.group_id)
    };

    // Set discriminator from MIG qualifier detection
    if let Some((seg, de)) = find_group_qualifier(&group.group_id, mig) {
        group.discriminator = Some((seg.clone(), de));

        // Derive AHB name from the trigger segment's first field definition
        if group.ahb_name.is_none() {
            group.ahb_name = derive_ahb_name(pid, &group_path, &seg);
        }
    }

    // Recursively enrich child groups
    for child in &mut group.child_groups {
        enrich_group_info(child, pid, mig, &group_path);
    }
}

/// Enhanced analysis that detects qualifier disambiguation for repeated segment groups.
///
/// When the same group (e.g., SG8) appears multiple times under a parent with different
/// qualifying values (e.g., SEQ+ZD5 vs SEQ+ZD6), this function splits them into separate
/// `PidGroupInfo` entries with their respective `qualifier_values`.
pub fn analyze_pid_structure_with_qualifiers(
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    _ahb: &AhbSchema,
) -> PidStructure {
    let mut top_level_segments: BTreeSet<String> = BTreeSet::new();
    let mut group_map: BTreeMap<String, GroupOccurrenceTracker> = BTreeMap::new();

    for field in &pid.fields {
        let parts: Vec<&str> = field.segment_path.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        if parts[0].starts_with("SG") {
            let group_id = parts[0].to_string();
            let tracker = group_map
                .entry(group_id.clone())
                .or_insert_with(|| GroupOccurrenceTracker::new(group_id.clone()));

            if parts.len() > 1 && !parts[1].starts_with("SG") {
                tracker.add_segment(parts[1]);
            }

            // Handle nested groups with qualifier tracking
            if parts.len() > 1 && parts[1].starts_with("SG") {
                let child_id = parts[1].to_string();

                // Check if this field is a qualifying field for the child group
                if let Some((trigger_seg, trigger_de)) = find_group_qualifier(&child_id, mig) {
                    // Build the full qualifying path: SG4/SG8/SEQ/1229
                    let qual_suffix = format!("{}/{}", trigger_seg, trigger_de);
                    let remaining: String = parts[2..].join("/");

                    if remaining == qual_suffix && !field.codes.is_empty() {
                        // This is a qualifying field — extract code values
                        let codes: Vec<String> = field
                            .codes
                            .iter()
                            .filter(|c| c.ahb_status.as_deref().is_some_and(|s| s.contains('X')))
                            .map(|c| c.value.clone())
                            .collect();

                        if !codes.is_empty() {
                            tracker.start_child_occurrence(
                                &child_id,
                                codes,
                                field.ahb_status.clone(),
                            );
                            // Also add the trigger segment
                            tracker.add_child_segment(&child_id, &trigger_seg);
                            continue;
                        }
                    }
                }

                // Check for group-level field (path is just "SG4/SG8" — marks occurrence start)
                if parts.len() == 2 {
                    tracker.mark_child_occurrence_boundary(&child_id, field.ahb_status.clone());
                    continue;
                }

                // Regular child group field — add segment to current occurrence
                if parts.len() > 2 && !parts[2].starts_with("SG") {
                    tracker.add_child_segment(&child_id, parts[2]);
                }

                // Handle deeply nested groups (SG4/SG8/SG10/...)
                if parts.len() > 2 && parts[2].starts_with("SG") {
                    tracker.add_child_nested_group(&child_id, parts[2]);
                    if parts.len() > 3 && !parts[3].starts_with("SG") {
                        tracker.add_child_nested_segment(&child_id, parts[2], parts[3]);
                    }
                }
            }
        } else {
            top_level_segments.insert(parts[0].to_string());
        }
    }

    let mut groups: Vec<PidGroupInfo> = group_map
        .into_values()
        .map(|t| t.into_group_info())
        .collect();

    // Enrich all groups with discriminator and AHB name info
    for group in &mut groups {
        enrich_group_info(group, pid, mig, "");
    }

    PidStructure {
        pid_id: pid.id.clone(),
        beschreibung: pid.beschreibung.clone(),
        kommunikation_von: pid.kommunikation_von.clone(),
        groups,
        top_level_segments: top_level_segments.into_iter().collect(),
    }
}

/// Tracks multiple occurrences of child groups under a parent.
struct GroupOccurrenceTracker {
    group_id: String,
    segments: BTreeSet<String>,
    child_trackers: BTreeMap<String, ChildGroupTracker>,
}

struct ChildGroupTracker {
    group_id: String,
    /// Each occurrence is (qualifier_values, ahb_status, segments, nested_groups)
    occurrences: Vec<ChildOccurrence>,
}

struct ChildOccurrence {
    qualifier_values: Vec<String>,
    ahb_status: String,
    segments: BTreeSet<String>,
    nested_groups: BTreeMap<String, BTreeSet<String>>,
}

impl GroupOccurrenceTracker {
    fn new(group_id: String) -> Self {
        Self {
            group_id,
            segments: BTreeSet::new(),
            child_trackers: BTreeMap::new(),
        }
    }

    fn add_segment(&mut self, seg_id: &str) {
        self.segments.insert(seg_id.to_string());
    }

    fn ensure_child(&mut self, child_id: &str) -> &mut ChildGroupTracker {
        self.child_trackers
            .entry(child_id.to_string())
            .or_insert_with(|| ChildGroupTracker {
                group_id: child_id.to_string(),
                occurrences: Vec::new(),
            })
    }

    fn start_child_occurrence(
        &mut self,
        child_id: &str,
        qualifier_values: Vec<String>,
        ahb_status: String,
    ) {
        let tracker = self.ensure_child(child_id);
        tracker.occurrences.push(ChildOccurrence {
            qualifier_values,
            ahb_status,
            segments: BTreeSet::new(),
            nested_groups: BTreeMap::new(),
        });
    }

    fn mark_child_occurrence_boundary(&mut self, child_id: &str, ahb_status: String) {
        let tracker = self.ensure_child(child_id);
        // If no occurrences yet, or the last occurrence already has qualifier values,
        // this group-level field starts a new (potentially qualifier-less) occurrence
        if tracker.occurrences.is_empty()
            || !tracker
                .occurrences
                .last()
                .unwrap()
                .qualifier_values
                .is_empty()
        {
            tracker.occurrences.push(ChildOccurrence {
                qualifier_values: Vec::new(),
                ahb_status,
                segments: BTreeSet::new(),
                nested_groups: BTreeMap::new(),
            });
        }
    }

    fn add_child_segment(&mut self, child_id: &str, seg_id: &str) {
        let tracker = self.ensure_child(child_id);
        if let Some(occ) = tracker.occurrences.last_mut() {
            occ.segments.insert(seg_id.to_string());
        } else {
            // No occurrence started yet — create a default one
            tracker.occurrences.push(ChildOccurrence {
                qualifier_values: Vec::new(),
                ahb_status: String::new(),
                segments: BTreeSet::from([seg_id.to_string()]),
                nested_groups: BTreeMap::new(),
            });
        }
    }

    fn add_child_nested_group(&mut self, child_id: &str, nested_id: &str) {
        let tracker = self.ensure_child(child_id);
        if let Some(occ) = tracker.occurrences.last_mut() {
            occ.nested_groups.entry(nested_id.to_string()).or_default();
        }
    }

    fn add_child_nested_segment(&mut self, child_id: &str, nested_id: &str, seg_id: &str) {
        let tracker = self.ensure_child(child_id);
        if let Some(occ) = tracker.occurrences.last_mut() {
            occ.nested_groups
                .entry(nested_id.to_string())
                .or_default()
                .insert(seg_id.to_string());
        }
    }

    fn into_group_info(self) -> PidGroupInfo {
        let mut child_groups = Vec::new();

        for (_child_id, tracker) in self.child_trackers {
            if tracker.occurrences.len() <= 1 {
                // Single occurrence — merge into one PidGroupInfo
                let occ = tracker.occurrences.into_iter().next();
                let (qualifier_values, ahb_status, segments, nested) = match occ {
                    Some(o) => (
                        o.qualifier_values,
                        o.ahb_status,
                        o.segments,
                        o.nested_groups,
                    ),
                    None => (Vec::new(), String::new(), BTreeSet::new(), BTreeMap::new()),
                };

                child_groups.push(PidGroupInfo {
                    group_id: tracker.group_id,
                    qualifier_values,
                    ahb_status,
                    ahb_name: None,
                    discriminator: None,
                    child_groups: nested
                        .into_iter()
                        .map(|(nid, segs)| PidGroupInfo {
                            group_id: nid,
                            qualifier_values: Vec::new(),
                            ahb_status: String::new(),
                            ahb_name: None,
                            discriminator: None,
                            child_groups: Vec::new(),
                            segments: segs,
                        })
                        .collect(),
                    segments,
                });
            } else {
                // Multiple occurrences — create separate entries
                for occ in tracker.occurrences {
                    child_groups.push(PidGroupInfo {
                        group_id: tracker.group_id.clone(),
                        qualifier_values: occ.qualifier_values,
                        ahb_status: occ.ahb_status,
                        ahb_name: None,
                        discriminator: None,
                        child_groups: occ
                            .nested_groups
                            .into_iter()
                            .map(|(nid, segs)| PidGroupInfo {
                                group_id: nid,
                                qualifier_values: Vec::new(),
                                ahb_status: String::new(),
                                ahb_name: None,
                                discriminator: None,
                                child_groups: Vec::new(),
                                segments: segs,
                            })
                            .collect(),
                        segments: occ.segments,
                    });
                }
            }
        }

        PidGroupInfo {
            group_id: self.group_id,
            qualifier_values: Vec::new(),
            ahb_status: String::new(),
            ahb_name: None,
            discriminator: None,
            child_groups,
            segments: self.segments,
        }
    }
}

// ---------------------------------------------------------------------------
// Code Generation: PID Structs
// ---------------------------------------------------------------------------

/// Sanitize text for use in a `///` doc comment — collapse newlines and trim.
fn sanitize_doc(s: &str) -> String {
    s.replace('\r', "")
        .replace('\n', " ")
        .replace("  ", " ")
        .trim()
        .to_string()
}

/// Generate a Rust struct source for a specific PID that composes PID-specific wrapper types.
pub fn generate_pid_struct(pid: &Pruefidentifikator, mig: &MigSchema, ahb: &AhbSchema) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let mut out = String::new();

    let struct_name = format!("Pid{}", pid.id);

    // First, emit wrapper structs for all groups (deduplicated)
    emit_wrapper_structs(&struct_name, &structure.groups, &mut out);

    // Then emit the main PID struct
    out.push_str(&format!(
        "/// PID {}: {}\n",
        pid.id,
        sanitize_doc(&pid.beschreibung)
    ));
    if let Some(ref komm) = pid.kommunikation_von {
        out.push_str(&format!("/// Kommunikation: {}\n", sanitize_doc(komm)));
    }
    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    // Top-level segments — use super::super:: since PID files are in pids/ subdir
    for seg_id in &structure.top_level_segments {
        let field_name = seg_id.to_lowercase();
        let seg_type = format!(
            "super::super::segments::Seg{}",
            capitalize_segment_id(seg_id)
        );
        out.push_str(&format!("    pub {field_name}: {seg_type},\n"));
    }

    // Groups with wrapper type names
    for group in &structure.groups {
        emit_pid_group_field_v2(&struct_name, group, &mut out, "    ");
    }

    out.push_str("}\n");

    out
}

/// Capitalize a segment ID for struct naming: "NAD" -> "Nad", "UNH" -> "Unh"
fn capitalize_segment_id(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let rest: String = chars.map(|c| c.to_ascii_lowercase()).collect();
            format!("{}{}", first.to_ascii_uppercase(), rest)
        }
    }
}

/// Make a Rust field name for a group (snake_case).
pub fn make_wrapper_field_name(group: &PidGroupInfo) -> String {
    if group.qualifier_values.is_empty() {
        group.group_id.to_lowercase()
    } else {
        format!(
            "{}_{}",
            group.group_id.to_lowercase(),
            group.qualifier_values.join("_").to_lowercase()
        )
    }
}

/// Make a Rust wrapper type name for a group (PascalCase).
fn make_wrapper_type_name(pid_struct_name: &str, group: &PidGroupInfo) -> String {
    let suffix = capitalize_segment_id(&group.group_id);
    if group.qualifier_values.is_empty() {
        format!("{pid_struct_name}{suffix}")
    } else {
        let qual_suffix: String = group
            .qualifier_values
            .iter()
            .map(|v| capitalize_segment_id(v))
            .collect::<Vec<_>>()
            .join("");
        format!("{pid_struct_name}{suffix}{qual_suffix}")
    }
}

/// Check if a group is an empty boundary marker (no segments, no qualifiers, no useful children).
fn is_empty_group(group: &PidGroupInfo) -> bool {
    group.segments.is_empty()
        && group.qualifier_values.is_empty()
        && group.child_groups.iter().all(is_empty_group)
}

/// Collected wrapper struct definition: name → (doc_comment, segments, child_field_lines).
/// Used to deduplicate child types (e.g., SG10) that appear under multiple parent instances.
struct WrapperDef {
    doc: String,
    segments: BTreeSet<String>,
    child_fields: Vec<String>,
}

/// Collect all wrapper struct definitions recursively, merging duplicates.
fn collect_wrapper_defs(
    pid_struct_name: &str,
    group: &PidGroupInfo,
    defs: &mut BTreeMap<String, WrapperDef>,
) {
    if is_empty_group(group) {
        return;
    }

    // Collect children first (depth-first)
    for child in &group.child_groups {
        collect_wrapper_defs(pid_struct_name, child, defs);
    }

    let wrapper_name = make_wrapper_type_name(pid_struct_name, group);

    // Build doc comment
    let mut doc = String::new();
    if let Some(ref name) = group.ahb_name {
        doc.push_str(&format!("/// {} — {}\n", group.group_id, sanitize_doc(name)));
    } else {
        doc.push_str(&format!("/// {}\n", group.group_id));
    }
    if !group.qualifier_values.is_empty() {
        doc.push_str(&format!(
            "/// Qualifiers: {}\n",
            group.qualifier_values.join(", ")
        ));
    }

    // Build child field lines
    let mut child_fields = Vec::new();
    for child in &group.child_groups {
        if is_empty_group(child) {
            continue;
        }
        let child_type = make_wrapper_type_name(pid_struct_name, child);
        let child_field = make_wrapper_field_name(child);
        child_fields.push(format!("    pub {child_field}: Vec<{child_type}>,"));
    }

    // Merge with existing definition (union of segments + child fields)
    if let Some(existing) = defs.get_mut(&wrapper_name) {
        existing.segments.extend(group.segments.iter().cloned());
        for field in &child_fields {
            if !existing.child_fields.contains(field) {
                existing.child_fields.push(field.clone());
            }
        }
    } else {
        defs.insert(
            wrapper_name,
            WrapperDef {
                doc,
                segments: group.segments.clone(),
                child_fields,
            },
        );
    }
}

/// Emit all wrapper structs (deduplicated) for the PID's groups.
fn emit_wrapper_structs(pid_struct_name: &str, groups: &[PidGroupInfo], out: &mut String) {
    let mut defs: BTreeMap<String, WrapperDef> = BTreeMap::new();
    for group in groups {
        collect_wrapper_defs(pid_struct_name, group, &mut defs);
    }

    for (name, def) in &defs {
        out.push_str(&def.doc);
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {name} {{\n"));
        for seg_id in &def.segments {
            let field_name = seg_id.to_lowercase();
            let seg_type = format!(
                "super::super::segments::Seg{}",
                capitalize_segment_id(seg_id)
            );
            out.push_str(&format!("    pub {field_name}: Option<{seg_type}>,\n"));
        }
        for field_line in &def.child_fields {
            out.push_str(field_line);
            out.push('\n');
        }
        out.push_str("}\n\n");
    }
}

/// Emit a field in the containing struct that references the wrapper type.
fn emit_pid_group_field_v2(
    pid_struct_name: &str,
    group: &PidGroupInfo,
    out: &mut String,
    indent: &str,
) {
    if is_empty_group(group) {
        return;
    }
    let wrapper_type = make_wrapper_type_name(pid_struct_name, group);
    let field_name = make_wrapper_field_name(group);

    // Groups can repeat per MIG
    out.push_str(&format!("{indent}pub {field_name}: Vec<{wrapper_type}>,\n"));
}

// ---------------------------------------------------------------------------
// Orchestrator: File Generation
// ---------------------------------------------------------------------------

use std::path::Path;

use crate::error::GeneratorError;

/// Generate all PID composition type files for a given AHB and write to disk.
///
/// Creates: `{output_dir}/{fv_lower}/{msg_lower}/pids/pid_{id}.rs` + `mod.rs`
pub fn generate_pid_types(
    mig: &MigSchema,
    ahb: &AhbSchema,
    format_version: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError> {
    let fv_lower = format_version.to_lowercase();
    let msg_lower = ahb.message_type.to_lowercase();
    let pids_dir = output_dir.join(&fv_lower).join(&msg_lower).join("pids");
    std::fs::create_dir_all(&pids_dir)?;

    let mut mod_entries = Vec::new();

    for pid in &ahb.workflows {
        let source = generate_pid_struct(pid, mig, ahb);
        let module_name = format!("pid_{}", pid.id.to_lowercase());
        let filename = format!("{module_name}.rs");

        let full_source = format!(
            "//! Auto-generated PID {} types.\n\
             //! {}\n\
             //! Do not edit manually.\n\n\
             use serde::{{Deserialize, Serialize}};\n\n\
             {source}",
            pid.id,
            sanitize_doc(&pid.beschreibung)
        );

        std::fs::write(pids_dir.join(&filename), full_source)?;
        mod_entries.push(module_name);
    }

    // Write mod.rs
    let mut mod_rs = String::from(
        "//! Per-PID composition types.\n\
         //! Do not edit manually.\n\n",
    );
    for module in &mod_entries {
        mod_rs.push_str(&format!("pub mod {module};\n"));
    }
    std::fs::write(pids_dir.join("mod.rs"), mod_rs)?;

    // Ensure the parent mod.rs includes `pub mod pids;`
    let parent_mod_path = output_dir.join(&fv_lower).join(&msg_lower).join("mod.rs");
    if parent_mod_path.exists() {
        let parent_mod = std::fs::read_to_string(&parent_mod_path)?;
        if !parent_mod.contains("pub mod pids;") {
            let updated = format!("{parent_mod}pub mod pids;\n");
            std::fs::write(&parent_mod_path, updated)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{ahb_parser, mig_parser};
    use std::path::PathBuf;

    fn load_mig_ahb() -> (MigSchema, AhbSchema) {
        let mig_path = PathBuf::from(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        );
        let ahb_path = PathBuf::from(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml",
        );
        if !mig_path.exists() || !ahb_path.exists() {
            panic!("MIG/AHB XML files not found — run from workspace root");
        }
        let mig =
            mig_parser::parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        let ahb =
            ahb_parser::parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        (mig, ahb)
    }

    #[test]
    fn test_pid_55001_structure_has_named_groups() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

        // SG2 should exist
        let _sg2 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG2")
            .unwrap();
        assert!(!structure.groups.is_empty());

        // SG4 should exist with child groups
        let sg4 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG4")
            .unwrap();
        assert!(!sg4.child_groups.is_empty());

        // SG4's child SG8 groups should have qualifier discrimination
        let sg8_children: Vec<_> = sg4
            .child_groups
            .iter()
            .filter(|c| c.group_id == "SG8")
            .collect();
        let has_qualified = sg8_children
            .iter()
            .any(|c| !c.qualifier_values.is_empty());
        assert!(
            has_qualified,
            "SG8 groups should have qualifier discrimination"
        );
    }

    #[test]
    fn test_pid_55001_sg2_has_ahb_names() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

        let sg2 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG2")
            .unwrap();
        // SG2 should have a discriminator (NAD qualifier)
        assert!(
            sg2.discriminator.is_some(),
            "SG2 should have NAD discriminator"
        );
    }

    #[test]
    fn test_generate_pid_55001_struct_snapshot() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let source = generate_pid_struct(pid, &mig, &ahb);
        insta::assert_snapshot!("pid_55001_struct", source);
    }
}
