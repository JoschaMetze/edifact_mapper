//! Code generation for MIG-tree Rust types.
//!
//! Reads `MigSchema` and emits Rust source code for:
//! - Code enums (one per data element with defined codes)
//! - Composite structs
//! - Segment structs
//! - Segment group structs

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::error::GeneratorError;
use crate::parsing::mig_parser::parse_mig;
use crate::schema::common::CodeDefinition;
use crate::schema::mig::{MigComposite, MigDataElement, MigSchema, MigSegment, MigSegmentGroup};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all data elements with codes from the entire MIG tree.
/// Returns a map of element_id -> Vec<CodeDefinition>, deduplicated.
fn collect_code_elements(mig: &MigSchema) -> BTreeMap<String, Vec<CodeDefinition>> {
    let mut result: BTreeMap<String, Vec<CodeDefinition>> = BTreeMap::new();

    fn visit_data_element(de: &MigDataElement, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        if !de.codes.is_empty() {
            let entry = result.entry(de.id.clone()).or_default();
            for code in &de.codes {
                if !entry.iter().any(|c| c.value == code.value) {
                    entry.push(code.clone());
                }
            }
        }
    }

    fn visit_composite(comp: &MigComposite, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        for de in &comp.data_elements {
            visit_data_element(de, result);
        }
    }

    fn visit_segment(seg: &MigSegment, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        for de in &seg.data_elements {
            visit_data_element(de, result);
        }
        for comp in &seg.composites {
            visit_composite(comp, result);
        }
    }

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        for seg in &group.segments {
            visit_segment(seg, result);
        }
        for nested in &group.nested_groups {
            visit_group(nested, result);
        }
    }

    for seg in &mig.segments {
        visit_segment(seg, &mut result);
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }

    result
}

/// Trim trailing whitespace/newlines from generated code, ensuring a single trailing newline.
fn trim_trailing(s: String) -> String {
    let trimmed: String = s
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n", trimmed.trim_end())
}

/// Sanitize a string for use in a `///` doc comment — collapse newlines/tabs and trim.
fn sanitize_doc(s: &str) -> String {
    s.replace('\r', "")
        .replace('\n', " ")
        .replace('\t', "    ")
        .replace("  ", " ")
        .trim()
        .to_string()
}

/// Emit a doc comment, handling multiline text safely.
fn emit_doc(out: &mut String, text: &str) {
    let sanitized = sanitize_doc(text);
    if !sanitized.is_empty() {
        out.push_str(&format!("    /// {sanitized}\n"));
    }
}

/// Sanitize a code value into a valid Rust identifier.
fn sanitize_variant_name(value: &str) -> String {
    let trimmed = value.trim();
    let mut name = String::new();
    for ch in trimmed.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            name.push('_');
        }
    }
    if name.is_empty() {
        return "Empty".to_string();
    }
    if name.chars().next().unwrap().is_ascii_digit() {
        name = format!("_{name}");
    }
    name
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

/// Determine the Rust type for a data element field.
fn data_element_type(
    de: &MigDataElement,
    code_elements: &BTreeMap<String, Vec<CodeDefinition>>,
) -> String {
    if code_elements.contains_key(&de.id) {
        format!("D{}Qualifier", de.id)
    } else {
        "String".to_string()
    }
}

/// Determine if an element is optional based on status.
fn is_optional(status_spec: &Option<String>, status_std: &Option<String>) -> bool {
    let status = status_spec
        .as_deref()
        .or(status_std.as_deref())
        .unwrap_or("C");
    matches!(status, "C" | "O" | "N" | "D")
}

// ---------------------------------------------------------------------------
// Collectors
// ---------------------------------------------------------------------------

/// Collect all unique composites from the MIG tree.
/// When the same composite ID appears multiple times, keeps the definition with
/// the most data elements (richest definition).
fn collect_composites(mig: &MigSchema) -> BTreeMap<String, MigComposite> {
    let mut result: BTreeMap<String, MigComposite> = BTreeMap::new();

    fn visit_segment(seg: &MigSegment, result: &mut BTreeMap<String, MigComposite>) {
        for comp in &seg.composites {
            let entry = result.entry(comp.id.clone());
            match entry {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(comp.clone());
                }
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    if comp.data_elements.len() > o.get().data_elements.len() {
                        o.insert(comp.clone());
                    }
                }
            }
        }
    }

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, MigComposite>) {
        for seg in &group.segments {
            visit_segment(seg, result);
        }
        for nested in &group.nested_groups {
            visit_group(nested, result);
        }
    }

    for seg in &mig.segments {
        visit_segment(seg, &mut result);
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }
    result
}

/// Collect all unique segments from the MIG tree.
/// When the same segment ID appears multiple times, keeps the definition with
/// the most fields (data elements + composites).
fn collect_segments(mig: &MigSchema) -> BTreeMap<String, MigSegment> {
    let mut result: BTreeMap<String, MigSegment> = BTreeMap::new();

    fn field_count(seg: &MigSegment) -> usize {
        seg.data_elements.len() + seg.composites.len()
    }

    fn insert_or_keep_richest(result: &mut BTreeMap<String, MigSegment>, seg: &MigSegment) {
        let entry = result.entry(seg.id.clone());
        match entry {
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(seg.clone());
            }
            std::collections::btree_map::Entry::Occupied(mut o) => {
                if field_count(seg) > field_count(o.get()) {
                    o.insert(seg.clone());
                }
            }
        }
    }

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, MigSegment>) {
        for seg in &group.segments {
            insert_or_keep_richest(result, seg);
        }
        for nested in &group.nested_groups {
            visit_group(nested, result);
        }
    }

    for seg in &mig.segments {
        insert_or_keep_richest(&mut result, seg);
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }
    result
}

// ---------------------------------------------------------------------------
// Code Generation: Enums
// ---------------------------------------------------------------------------

/// Generate Rust enum definitions for all data elements that have code lists.
pub fn generate_enums(mig: &MigSchema) -> String {
    let code_elements = collect_code_elements(mig);
    let mut out = String::new();

    out.push_str("//! Auto-generated code enums from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("#![allow(clippy::enum_variant_names, non_camel_case_types)]\n\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    for (element_id, codes) in &code_elements {
        let enum_name = format!("D{element_id}Qualifier");

        // Derive block
        out.push_str("#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub enum {enum_name} {{\n"));

        for code in codes {
            let variant = sanitize_variant_name(&code.value);
            let doc_text = code
                .description
                .as_ref()
                .filter(|d| !d.is_empty())
                .map(|d| d.as_str())
                .or(Some(&code.name))
                .filter(|s| !s.is_empty());
            if let Some(text) = doc_text {
                emit_doc(&mut out, text);
            }
            out.push_str(&format!("    {variant},\n"));
        }

        // Unknown variant for forward compatibility
        out.push_str("    /// Unrecognized code value\n");
        out.push_str("    Unknown(String),\n");
        out.push_str("}\n\n");

        // Display impl
        out.push_str(&format!("impl std::fmt::Display for {enum_name} {{\n"));
        out.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        out.push_str("        match self {\n");
        for code in codes {
            let variant = sanitize_variant_name(&code.value);
            let raw = code.value.trim();
            out.push_str(&format!(
                "            Self::{variant} => write!(f, \"{raw}\"),\n"
            ));
        }
        out.push_str("            Self::Unknown(s) => write!(f, \"{}\", s),\n");
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // FromStr impl
        out.push_str(&format!("impl std::str::FromStr for {enum_name} {{\n"));
        out.push_str("    type Err = std::convert::Infallible;\n\n");
        out.push_str("    fn from_str(s: &str) -> Result<Self, Self::Err> {\n");
        out.push_str("        Ok(match s.trim() {\n");
        for code in codes {
            let variant = sanitize_variant_name(&code.value);
            let raw = code.value.trim();
            out.push_str(&format!("            \"{raw}\" => Self::{variant},\n"));
        }
        out.push_str("            other => Self::Unknown(other.to_string()),\n");
        out.push_str("        })\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Code Generation: Composites
// ---------------------------------------------------------------------------

/// Build field names for data elements, disambiguating duplicates by appending `_N`.
fn build_de_field_names(data_elements: &[MigDataElement]) -> Vec<String> {
    // Count occurrences of each ID
    let mut id_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for de in data_elements {
        *id_counts.entry(&de.id).or_insert(0) += 1;
    }
    // For IDs appearing more than once, append position index
    let mut id_seen: BTreeMap<&str, usize> = BTreeMap::new();
    data_elements
        .iter()
        .map(|de| {
            let count = id_counts[de.id.as_str()];
            if count > 1 {
                let idx = id_seen.entry(&de.id).or_insert(0);
                *idx += 1;
                format!("d{}_{}", de.id, idx)
            } else {
                format!("d{}", de.id)
            }
        })
        .collect()
}

/// Build field names for composites, disambiguating duplicates by appending `_N`.
fn build_composite_field_names(composites: &[MigComposite]) -> Vec<String> {
    let mut id_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for comp in composites {
        *id_counts.entry(&comp.id).or_insert(0) += 1;
    }
    let mut id_seen: BTreeMap<&str, usize> = BTreeMap::new();
    composites
        .iter()
        .map(|comp| {
            let count = id_counts[comp.id.as_str()];
            if count > 1 {
                let idx = id_seen.entry(&comp.id).or_insert(0);
                *idx += 1;
                format!("c{}_{}", comp.id.to_lowercase(), idx)
            } else {
                format!("c{}", comp.id.to_lowercase())
            }
        })
        .collect()
}

/// Generate Rust struct definitions for all composites in the MIG.
pub fn generate_composites(mig: &MigSchema) -> String {
    let composites = collect_composites(mig);
    let code_elements = collect_code_elements(mig);
    let mut out = String::new();

    out.push_str("//! Auto-generated composite structs from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("use super::enums::*;\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    for (comp_id, comp) in &composites {
        let struct_name = format!("Composite{comp_id}");
        let field_names = build_de_field_names(&comp.data_elements);

        let doc = comp
            .description
            .as_ref()
            .filter(|d| !d.is_empty())
            .map(|d| d.as_str())
            .unwrap_or(&comp.name);
        if !doc.is_empty() {
            let sanitized = sanitize_doc(doc);
            out.push_str(&format!("/// {comp_id} — {sanitized}\n"));
        }
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));

        for (de, field_name) in comp.data_elements.iter().zip(field_names.iter()) {
            let base_type = data_element_type(de, &code_elements);
            let optional = is_optional(&de.status_spec, &de.status_std);

            let doc = de.description.as_ref().unwrap_or(&de.name);
            if !doc.is_empty() {
                emit_doc(&mut out, doc);
            }
            if optional {
                out.push_str(&format!("    pub {field_name}: Option<{base_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {base_type},\n"));
            }
        }

        out.push_str("}\n\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Code Generation: Segments
// ---------------------------------------------------------------------------

/// Generate Rust struct definitions for all segments in the MIG.
pub fn generate_segments(mig: &MigSchema) -> String {
    let segments = collect_segments(mig);
    let code_elements = collect_code_elements(mig);
    let mut out = String::new();

    out.push_str("//! Auto-generated segment structs from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("#![allow(non_snake_case)]\n\n");
    out.push_str("use super::composites::*;\n");
    out.push_str("use super::enums::*;\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    for (seg_id, seg) in &segments {
        let struct_name = format!("Seg{}", capitalize_segment_id(seg_id));
        let de_field_names = build_de_field_names(&seg.data_elements);

        let doc = sanitize_doc(seg.description.as_ref().unwrap_or(&seg.name));
        out.push_str(&format!("/// {} segment — {}\n", seg_id, doc));
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));

        // Direct data elements
        for (de, field_name) in seg.data_elements.iter().zip(de_field_names.iter()) {
            let base_type = data_element_type(de, &code_elements);
            let optional = is_optional(&de.status_spec, &de.status_std);

            let doc = de
                .description
                .as_ref()
                .filter(|d| !d.is_empty())
                .map(|d| d.as_str())
                .unwrap_or(&de.name);
            if !doc.is_empty() {
                emit_doc(&mut out, doc);
            }
            if optional {
                out.push_str(&format!("    pub {field_name}: Option<{base_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {base_type},\n"));
            }
        }

        // Composites — disambiguate duplicate IDs by appending _N
        let comp_field_names = build_composite_field_names(&seg.composites);
        for (comp, field_name) in seg.composites.iter().zip(comp_field_names.iter()) {
            let comp_type = format!("Composite{}", comp.id);
            let optional = is_optional(&comp.status_spec, &comp.status_std);

            let doc = comp
                .description
                .as_ref()
                .filter(|d| !d.is_empty())
                .map(|d| d.as_str())
                .unwrap_or(&comp.name);
            if !doc.is_empty() {
                emit_doc(&mut out, doc);
            }
            if optional {
                out.push_str(&format!("    pub {field_name}: Option<{comp_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {comp_type},\n"));
            }
        }

        out.push_str("}\n\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Code Generation: Groups
// ---------------------------------------------------------------------------

/// Merged view of a segment group, combining segments and nested groups from all
/// MIG XML occurrences of the same group ID.
struct MergedGroup {
    id: String,
    name: String,
    /// Union of all segments across all occurrences, keyed by segment ID.
    /// Each segment keeps the richest definition (most data elements/composites).
    segments: BTreeMap<String, MigSegment>,
    /// Recursively merged nested groups.
    nested: BTreeMap<String, MergedGroup>,
}

/// Build a merged map of all segment groups from the MIG tree.
///
/// The MIG XML can define the same group ID (e.g., G_SG4) multiple times with
/// different contents — once for each variant (e.g., IDE+Z01 vs IDE+24). This
/// function merges them into a single definition per group ID, collecting the
/// union of all segments and nested groups.
fn build_merged_groups(groups: &[MigSegmentGroup]) -> BTreeMap<String, MergedGroup> {
    let mut map: BTreeMap<String, MergedGroup> = BTreeMap::new();

    fn merge_into(map: &mut BTreeMap<String, MergedGroup>, group: &MigSegmentGroup) {
        let entry = map.entry(group.id.clone()).or_insert_with(|| MergedGroup {
            id: group.id.clone(),
            name: group.name.clone(),
            segments: BTreeMap::new(),
            nested: BTreeMap::new(),
        });

        // Merge segments — keep the richest definition per segment ID
        for seg in &group.segments {
            let seg_entry = entry.segments.entry(seg.id.clone());
            match seg_entry {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert(seg.clone());
                }
                std::collections::btree_map::Entry::Occupied(mut o) => {
                    let existing_count = o.get().data_elements.len() + o.get().composites.len();
                    let new_count = seg.data_elements.len() + seg.composites.len();
                    if new_count > existing_count {
                        o.insert(seg.clone());
                    }
                }
            }
        }

        // Recursively merge nested groups
        for nested in &group.nested_groups {
            merge_into(&mut entry.nested, nested);
        }
    }

    for group in groups {
        merge_into(&mut map, group);
    }

    map
}

/// Generate Rust struct definitions for all segment groups in the MIG.
pub fn generate_groups(mig: &MigSchema) -> String {
    let merged = build_merged_groups(&mig.segment_groups);
    let mut out = String::new();
    let mut emitted: BTreeSet<String> = BTreeSet::new();

    out.push_str("//! Auto-generated segment group structs from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("use super::segments::*;\n");
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    fn emit_merged_group(group: &MergedGroup, out: &mut String, emitted: &mut BTreeSet<String>) {
        let group_num = group.id.trim_start_matches("SG");
        let struct_name = format!("Sg{group_num}");

        if !emitted.insert(struct_name.clone()) {
            return;
        }

        if !group.name.is_empty() {
            let doc = sanitize_doc(&group.name);
            out.push_str(&format!("/// {} — {}\n", group.id, doc));
        }
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));

        // Segments in this group (sorted by segment ID for deterministic output)
        for seg in group.segments.values() {
            let field_name = seg.id.to_lowercase();
            let seg_type = format!("Seg{}", capitalize_segment_id(&seg.id));
            let optional = is_optional(&seg.status_spec, &seg.status_std);
            let repeating = seg.max_rep() > 1;

            if repeating {
                out.push_str(&format!("    pub {field_name}: Vec<{seg_type}>,\n"));
            } else if optional {
                out.push_str(&format!("    pub {field_name}: Option<{seg_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {seg_type},\n"));
            }
        }

        // Nested groups (sorted by group ID for deterministic output)
        for nested in group.nested.values() {
            let nested_num = nested.id.trim_start_matches("SG");
            let nested_name = format!("sg{nested_num}");
            let nested_type = format!("Sg{nested_num}");

            // Nested groups in a MIG are always repeatable (Vec)
            out.push_str(&format!("    pub {nested_name}: Vec<{nested_type}>,\n"));
        }

        out.push_str("}\n\n");

        // Recurse into nested groups
        for nested in group.nested.values() {
            emit_merged_group(nested, out, emitted);
        }
    }

    for group in merged.values() {
        emit_merged_group(group, &mut out, &mut emitted);
    }

    out
}

// ---------------------------------------------------------------------------
// Orchestrator: File Generation
// ---------------------------------------------------------------------------

/// Generate all MIG type files for a given MIG XML and write them to disk.
///
/// Creates the directory structure:
///   `{output_dir}/{fv_lower}/{msg_lower}/enums.rs`
///   `{output_dir}/{fv_lower}/{msg_lower}/composites.rs`
///   `{output_dir}/{fv_lower}/{msg_lower}/segments.rs`
///   `{output_dir}/{fv_lower}/{msg_lower}/groups.rs`
///   `{output_dir}/{fv_lower}/{msg_lower}/mod.rs`
pub fn generate_mig_types(
    mig_path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError> {
    let mig = parse_mig(mig_path, message_type, variant, format_version)?;

    let fv_lower = format_version.to_lowercase();
    let msg_lower = message_type.to_lowercase();
    let base_dir = output_dir.join(&fv_lower).join(&msg_lower);
    std::fs::create_dir_all(&base_dir)?;

    std::fs::write(
        base_dir.join("enums.rs"),
        trim_trailing(generate_enums(&mig)),
    )?;
    std::fs::write(
        base_dir.join("composites.rs"),
        trim_trailing(generate_composites(&mig)),
    )?;
    std::fs::write(
        base_dir.join("segments.rs"),
        trim_trailing(generate_segments(&mig)),
    )?;
    std::fs::write(
        base_dir.join("groups.rs"),
        trim_trailing(generate_groups(&mig)),
    )?;

    let mod_rs = format!(
        "//! Generated {message_type} types for {format_version}.\n\
         //! Do not edit manually.\n\n\
         pub mod composites;\n\
         pub mod enums;\n\
         pub mod groups;\n\
         pub mod segments;\n"
    );
    std::fs::write(base_dir.join("mod.rs"), mod_rs)?;

    // Write parent mod.rs for the format version directory
    let fv_mod = format!("pub mod {msg_lower};\n");
    std::fs::write(output_dir.join(&fv_lower).join("mod.rs"), fv_mod)?;

    Ok(())
}
