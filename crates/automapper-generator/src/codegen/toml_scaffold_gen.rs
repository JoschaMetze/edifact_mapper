//! Generate TOML mapping scaffolds from PID schema.
//!
//! Produces one `.toml` file per entity path with MIG segment paths pre-filled.
//! Developers fill in `target` (BO4E field name) and optional `enum_map`.

use crate::codegen::pid_type_gen::{
    analyze_pid_structure_with_qualifiers, make_wrapper_field_name, PidGroupInfo,
};
use crate::schema::ahb::AhbSchema;
use crate::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

/// Generate TOML scaffold for a single PID group field.
pub fn generate_group_scaffold(
    group: &PidGroupInfo,
    field_name: &str,
    entity_hint: &str,
    mig: &MigSchema,
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

    // For each segment in the group, enumerate its element paths using numeric indices
    for seg_id in &group.segments {
        if let Some(mig_seg) = find_segment_in_mig(seg_id, &group.group_id, mig) {
            let seg_lower = seg_id.to_lowercase();
            // Emit data element paths
            for (ei, _de) in mig_seg.data_elements.iter().enumerate() {
                out.push_str(&format!("\"{seg_lower}.{ei}\" = {{ target = \"\" }}\n"));
            }
            // Emit composite element paths
            for (ci, comp) in mig_seg.composites.iter().enumerate() {
                let elem_idx = mig_seg.data_elements.len() + ci;
                for (di, _de) in comp.data_elements.iter().enumerate() {
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
    let mut results = Vec::new();

    fn collect_scaffolds(
        group: &PidGroupInfo,
        field_name: &str,
        mig: &MigSchema,
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
            let content = generate_group_scaffold(group, field_name, &entity_hint, mig);
            results.push((filename, content));
        }
        for child in &group.child_groups {
            let child_field = make_wrapper_field_name(child);
            let child_path = format!("{field_name}.{child_field}");
            collect_scaffolds(child, &child_path, mig, results);
        }
    }

    for group in &structure.groups {
        let field_name = make_wrapper_field_name(group);
        collect_scaffolds(group, &field_name, mig, &mut results);
    }

    results
}

fn find_segment_in_mig<'a>(
    seg_id: &str,
    group_id: &str,
    mig: &'a MigSchema,
) -> Option<&'a MigSegment> {
    fn find_in_group<'a>(
        seg_id: &str,
        target_group: &str,
        group: &'a MigSegmentGroup,
    ) -> Option<&'a MigSegment> {
        if group.id == target_group {
            return group
                .segments
                .iter()
                .find(|s| s.id.eq_ignore_ascii_case(seg_id));
        }
        for nested in &group.nested_groups {
            if let Some(s) = find_in_group(seg_id, target_group, nested) {
                return Some(s);
            }
        }
        None
    }

    // Check top-level segments first
    if let Some(seg) = mig
        .segments
        .iter()
        .find(|s| s.id.eq_ignore_ascii_case(seg_id))
    {
        return Some(seg);
    }
    // Check groups
    for group in &mig.segment_groups {
        if let Some(seg) = find_in_group(seg_id, group_id, group) {
            return Some(seg);
        }
    }
    None
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
        let sg2 = structure.groups.iter().find(|g| g.group_id == "SG2").unwrap();

        let scaffold = generate_group_scaffold(sg2, "sg2", "Marktteilnehmer", &mig);
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
