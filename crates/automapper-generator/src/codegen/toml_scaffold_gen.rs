//! Generate TOML mapping scaffolds from PID schema.
//!
//! Produces one `.toml` file per entity path with MIG segment paths pre-filled.
//! Developers fill in `target` (BO4E field name) and optional `enum_map`.

use std::collections::HashMap;

use crate::codegen::pid_type_gen::{
    analyze_pid_structure_with_qualifiers, build_mig_number_index, make_wrapper_field_name,
    PidGroupInfo,
};
use crate::schema::ahb::AhbSchema;
use crate::schema::common::CodeDefinition;
use crate::schema::mig::{MigSchema, MigSegment};

/// Generate TOML scaffold for a single PID group field.
pub fn generate_group_scaffold(
    group: &PidGroupInfo,
    field_name: &str,
    entity_hint: &str,
    mig: &MigSchema,
    number_index: &HashMap<String, &MigSegment>,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# AUTO-GENERATED scaffold for {entity_hint} → {field_name}\n"
    ));
    out.push_str("# Fill in \"target\" fields with BO4E field names.\n\n");
    out.push_str("[meta]\n");
    out.push_str(&format!("entity = \"{entity_hint}\"\n"));
    out.push_str(&format!("bo4e_type = \"{entity_hint}\"\n"));
    out.push_str(&format!("source_group = \"{}\"\n", group.group_id));
    out.push_str(&format!("source_path = \"{field_name}\"\n\n"));
    out.push_str("[fields]\n");

    // For each segment in the group, enumerate its element paths with metadata comments
    for seg_id in &group.segments {
        // Direct Number-based lookup, fall back to top-level MIG search
        let mig_seg = group
            .segment_mig_numbers
            .get(seg_id)
            .and_then(|num| number_index.get(num).copied())
            .or_else(|| {
                mig.segments
                    .iter()
                    .find(|s| s.id.eq_ignore_ascii_case(seg_id))
            });
        if let Some(mig_seg) = mig_seg {
            let seg_lower = seg_id.to_lowercase();
            let seg_name = mig_seg
                .description
                .as_deref()
                .filter(|d| !d.is_empty())
                .unwrap_or(&mig_seg.name);
            out.push_str(&format!("# --- {} ({}) ---\n", seg_id, seg_name));

            // Emit data element paths with name/code comments
            for (ei, de) in mig_seg.data_elements.iter().enumerate() {
                let de_name = de
                    .description
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .unwrap_or(&de.name);
                let codes_hint = format_codes_hint_from_mig(&de.codes);
                out.push_str(&format!("# D{} {}{}\n", de.id, de_name, codes_hint));
                out.push_str(&format!("\"{seg_lower}.{ei}\" = {{ target = \"\" }}\n"));
            }
            // Emit composite element paths with name/code comments
            for (ci, comp) in mig_seg.composites.iter().enumerate() {
                let elem_idx = mig_seg.data_elements.len() + ci;
                let comp_name = comp
                    .description
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .unwrap_or(&comp.name);
                out.push_str(&format!("# {} {}\n", comp.id, comp_name));
                for (di, de) in comp.data_elements.iter().enumerate() {
                    let de_name = de
                        .description
                        .as_deref()
                        .filter(|d| !d.is_empty())
                        .unwrap_or(&de.name);
                    let codes_hint = format_codes_hint_from_mig(&de.codes);
                    out.push_str(&format!("#   D{} {}{}\n", de.id, de_name, codes_hint));
                    out.push_str(&format!(
                        "\"{seg_lower}.{elem_idx}.{di}\" = {{ target = \"\" }}\n"
                    ));
                }
            }
        }
    }

    out
}

/// Generate scaffolds for all groups in a PID.
pub fn generate_pid_scaffolds(
    pid_id: &str,
    mig: &MigSchema,
    ahb: &AhbSchema,
) -> Vec<(String, String)> {
    let pid = match ahb.workflows.iter().find(|p| p.id == pid_id) {
        Some(p) => p,
        None => return vec![],
    };

    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let number_index = build_mig_number_index(mig);
    let mut results = Vec::new();

    fn collect_scaffolds<'a>(
        group: &PidGroupInfo,
        field_name: &str,
        mig: &'a MigSchema,
        number_index: &HashMap<String, &'a MigSegment>,
        results: &mut Vec<(String, String)>,
    ) {
        if group.segments.is_empty() && group.child_groups.is_empty() {
            return;
        }
        if !group.segments.is_empty() {
            let entity_hint = group
                .ahb_name
                .as_deref()
                .unwrap_or(&group.group_id)
                .replace(", ", "_")
                .replace(' ', "_");
            let filename = format!("{field_name}.toml");
            let content =
                generate_group_scaffold(group, field_name, &entity_hint, mig, number_index);
            results.push((filename, content));
        }
        for child in &group.child_groups {
            let child_field = make_wrapper_field_name(child);
            let child_path = format!("{field_name}.{child_field}");
            collect_scaffolds(child, &child_path, mig, number_index, results);
        }
    }

    for group in &structure.groups {
        let field_name = make_wrapper_field_name(group);
        collect_scaffolds(group, &field_name, mig, &number_index, &mut results);
    }

    results
}

/// Format a short codes hint like ` [Z18=Regelzone, Z66=Zaehlpunkttyp]`.
fn format_codes_hint_from_mig(codes: &[CodeDefinition]) -> String {
    if codes.is_empty() {
        return String::new();
    }
    let entries: Vec<String> = codes
        .iter()
        .take(5)
        .map(|c| {
            let name = c
                .description
                .as_deref()
                .filter(|d| !d.is_empty())
                .unwrap_or(&c.name);
            if name.is_empty() {
                c.value.clone()
            } else {
                format!("{}={}", c.value, name)
            }
        })
        .collect();
    let suffix = if codes.len() > 5 { ", ..." } else { "" };
    format!(" [{}{}]", entries.join(", "), suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{ahb_parser, mig_parser};
    use std::path::Path;

    fn load_mig_ahb() -> Option<(MigSchema, AhbSchema)> {
        let mig_path = Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        );
        let ahb_path = Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml",
        );
        if !mig_path.exists() || !ahb_path.exists() {
            return None;
        }
        let mig = mig_parser::parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
        let ahb = ahb_parser::parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
        Some((mig, ahb))
    }

    #[test]
    fn test_scaffold_for_pid_55001_sg2() {
        let Some((mig, ahb)) = load_mig_ahb() else {
            return;
        };
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);
        let sg2 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG2")
            .unwrap();

        let number_index = build_mig_number_index(&mig);
        let scaffold = generate_group_scaffold(sg2, "sg2", "Marktteilnehmer", &mig, &number_index);
        assert!(
            scaffold.contains("source_path = \"sg2\""),
            "Should have source_path"
        );
        assert!(scaffold.contains("[fields]"), "Should have fields section");
        assert!(scaffold.contains("\"nad."), "Should have NAD element paths");
    }

    #[test]
    fn test_generate_pid_scaffolds_55001() {
        let Some((mig, ahb)) = load_mig_ahb() else {
            return;
        };
        let scaffolds = generate_pid_scaffolds("55001", &mig, &ahb);
        assert!(!scaffolds.is_empty(), "Should produce some scaffolds");

        // Check we got an SG2-level scaffold
        let sg2_scaffold = scaffolds.iter().find(|(name, _)| name == "sg2.toml");
        assert!(sg2_scaffold.is_some(), "Should have sg2.toml scaffold");
    }
}
