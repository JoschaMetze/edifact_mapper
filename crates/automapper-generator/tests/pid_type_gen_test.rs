use automapper_generator::codegen::pid_type_gen;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn load_utilmd() -> (
    automapper_generator::schema::mig::MigSchema,
    automapper_generator::schema::ahb::AhbSchema,
) {
    let mig = parse_mig(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    )
    .unwrap();
    let ahb = parse_ahb(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    )
    .unwrap();
    (mig, ahb)
}

#[test]
fn test_analyze_pid_structure() {
    let (mig, ahb) = load_utilmd();

    // Pick a specific PID to analyze
    let pid = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();
    let structure = pid_type_gen::analyze_pid_structure(pid, &mig);

    // Should identify which top-level groups are present
    assert!(!structure.groups.is_empty(), "PID should have groups");
    // Should identify SG2 (NAD parties) as present
    assert!(
        structure.groups.iter().any(|g| g.group_id == "SG2"),
        "Missing SG2"
    );
    // Should identify SG4 (transaction) as present
    assert!(
        structure.groups.iter().any(|g| g.group_id == "SG4"),
        "Missing SG4"
    );
}

#[test]
fn test_detect_qualifier_disambiguation() {
    let (mig, ahb) = load_utilmd();

    // Find a PID that has multiple SG8 usages with different SEQ qualifiers
    let pid = ahb
        .workflows
        .iter()
        .find(|w| {
            let structure = pid_type_gen::analyze_pid_structure(w, &mig);
            structure.groups.iter().any(|g| g.group_id == "SG4")
        })
        .expect("Should find a PID with SG4");

    let structure = pid_type_gen::analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

    // Check that SG8 groups under SG4 are disambiguated by qualifier
    let sg4 = structure
        .groups
        .iter()
        .find(|g| g.group_id == "SG4")
        .unwrap();
    let sg8_groups: Vec<_> = sg4
        .child_groups
        .iter()
        .filter(|g| g.group_id == "SG8")
        .collect();

    // If this PID has multiple SG8 usages, they should have different qualifier_values
    if sg8_groups.len() > 1 {
        let qualifiers: Vec<_> = sg8_groups.iter().map(|g| &g.qualifier_values).collect();
        assert!(
            qualifiers.windows(2).all(|w| w[0] != w[1]),
            "SG8 groups should be disambiguated by qualifier: {:?}",
            qualifiers
        );
    }

    // At least some SG8 groups should have qualifier values
    let with_qualifiers: Vec<_> = sg8_groups
        .iter()
        .filter(|g| !g.qualifier_values.is_empty())
        .collect();
    assert!(
        !with_qualifiers.is_empty(),
        "Expected some SG8 groups with qualifier values"
    );
}

#[test]
fn test_generate_pid_struct() {
    let (mig, ahb) = load_utilmd();
    let pid = ahb
        .workflows
        .iter()
        .find(|w| w.id == "55001")
        .or_else(|| ahb.workflows.first())
        .unwrap();

    let pid_source = pid_type_gen::generate_pid_struct(pid, &mig, &ahb);

    // Should generate a struct named Pid{id}
    let expected_struct = format!("pub struct Pid{}", pid.id);
    assert!(
        pid_source.contains(&expected_struct),
        "Missing {} struct in:\n{}",
        expected_struct,
        &pid_source[..pid_source.len().min(500)]
    );
    // Should have doc comment with description
    assert!(pid_source.contains(&pid.beschreibung));
    // Should compose shared group types (Sg2, Sg4, Sg8, etc.)
    assert!(
        pid_source.contains("Sg2") || pid_source.contains("Sg4") || pid_source.contains("Sg8"),
        "Should reference shared group types"
    );
    // Should derive standard traits
    assert!(pid_source.contains("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"));
}

#[test]
fn test_generate_pid_types_writes_files() {
    let output_dir = tempfile::tempdir().unwrap();
    let (mig, ahb) = load_utilmd();

    pid_type_gen::generate_pid_types(&mig, &ahb, "FV2504", output_dir.path()).unwrap();

    let pids_dir = output_dir.path().join("fv2504").join("utilmd").join("pids");
    assert!(pids_dir.exists(), "pids/ directory should exist");

    // Should generate a file for each PID
    let pid_files: Vec<_> = std::fs::read_dir(&pids_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".rs") && name != "mod.rs"
        })
        .collect();

    // Should have at least some PID files
    assert!(
        pid_files.len() > 10,
        "Expected many PID files, got {}",
        pid_files.len()
    );

    // Should have a mod.rs
    assert!(pids_dir.join("mod.rs").exists(), "Missing pids/mod.rs");

    // mod.rs should declare all PID modules
    let mod_content = std::fs::read_to_string(pids_dir.join("mod.rs")).unwrap();
    assert!(
        mod_content.contains("pub mod pid_55001") || mod_content.contains("pub mod pid_55035"),
        "mod.rs should declare PID modules"
    );
}

// ---------------------------------------------------------------------------
// AHB Validation Tests
// ---------------------------------------------------------------------------

/// Extract top-level group IDs that a PID references in its AHB field definitions.
fn extract_ahb_group_ids(
    pid: &automapper_generator::schema::ahb::Pruefidentifikator,
) -> BTreeSet<String> {
    let mut groups = BTreeSet::new();
    for field in &pid.fields {
        let first = field.segment_path.split('/').next().unwrap_or("");
        if first.starts_with("SG") {
            groups.insert(first.to_string());
        }
    }
    groups
}

/// Extract top-level segment IDs (outside groups) from AHB fields.
fn extract_ahb_top_level_segments(
    pid: &automapper_generator::schema::ahb::Pruefidentifikator,
) -> BTreeSet<String> {
    let mut segs = BTreeSet::new();
    for field in &pid.fields {
        let first = field.segment_path.split('/').next().unwrap_or("");
        if !first.is_empty() && !first.starts_with("SG") {
            segs.insert(first.to_string());
        }
    }
    segs
}

#[test]
fn test_generated_pid_count_matches_ahb_workflows() {
    let (_mig, ahb) = load_utilmd();

    let pids_dir = Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids");
    assert!(pids_dir.exists(), "pids/ directory should exist");

    let pid_files: Vec<_> = std::fs::read_dir(pids_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.starts_with("pid_") && name.ends_with(".rs") && name != "mod.rs"
        })
        .collect();

    assert_eq!(
        pid_files.len(),
        ahb.workflows.len(),
        "Generated PID file count ({}) should match AHB workflow count ({})",
        pid_files.len(),
        ahb.workflows.len()
    );
}

#[test]
fn test_every_pid_struct_has_groups_from_ahb_fields() {
    let (mig, ahb) = load_utilmd();

    let mut mismatches = Vec::new();

    for pid in &ahb.workflows {
        let ahb_groups = extract_ahb_group_ids(pid);
        let structure = pid_type_gen::analyze_pid_structure(pid, &mig);
        let struct_groups: BTreeSet<String> = structure
            .groups
            .iter()
            .map(|g| g.group_id.clone())
            .collect();

        // Every group referenced in AHB fields should appear in the analyzed structure
        let missing: BTreeSet<_> = ahb_groups.difference(&struct_groups).collect();
        if !missing.is_empty() {
            mismatches.push(format!(
                "PID {}: AHB references groups {:?} but struct is missing {:?}",
                pid.id, ahb_groups, missing
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "AHB-to-PID group mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn test_every_pid_struct_has_top_level_segments_from_ahb() {
    let (mig, ahb) = load_utilmd();

    let mut mismatches = Vec::new();

    for pid in &ahb.workflows {
        let ahb_segs = extract_ahb_top_level_segments(pid);
        let structure = pid_type_gen::analyze_pid_structure(pid, &mig);
        let struct_segs: BTreeSet<String> = structure.top_level_segments.iter().cloned().collect();

        let missing: BTreeSet<_> = ahb_segs.difference(&struct_segs).collect();
        if !missing.is_empty() {
            mismatches.push(format!(
                "PID {}: AHB references top-level segments {:?} but struct is missing {:?}",
                pid.id, ahb_segs, missing
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "AHB-to-PID top-level segment mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn test_sg1_presence_matches_ahb_fields() {
    let (mig, ahb) = load_utilmd();

    // These 7 PIDs should have SG1 based on AHB analysis
    let expected_sg1_pids: BTreeSet<&str> = [
        "55065", "55067", "55069", "55070", "55074", "55195", "55201",
    ]
    .into_iter()
    .collect();

    let mut actual_sg1_pids = BTreeSet::new();
    let mut wrong_presence = Vec::new();
    let mut wrong_absence = Vec::new();

    for pid in &ahb.workflows {
        let ahb_groups = extract_ahb_group_ids(pid);
        let has_sg1_in_ahb = ahb_groups.contains("SG1");

        let structure = pid_type_gen::analyze_pid_structure(pid, &mig);
        let has_sg1_in_struct = structure.groups.iter().any(|g| g.group_id == "SG1");

        if has_sg1_in_struct {
            actual_sg1_pids.insert(pid.id.as_str());
        }

        if has_sg1_in_ahb && !has_sg1_in_struct {
            wrong_absence.push(format!("PID {} has SG1 in AHB but not in struct", pid.id));
        }
        if !has_sg1_in_ahb && has_sg1_in_struct {
            wrong_presence.push(format!("PID {} has SG1 in struct but not in AHB", pid.id));
        }
    }

    assert!(
        wrong_absence.is_empty() && wrong_presence.is_empty(),
        "SG1 mismatches:\n{}\n{}",
        wrong_absence.join("\n"),
        wrong_presence.join("\n")
    );

    // Verify the specific set of PIDs with SG1
    let actual_ids: BTreeSet<&str> = actual_sg1_pids.into_iter().collect();
    assert_eq!(
        actual_ids, expected_sg1_pids,
        "PIDs with SG1 don't match expected set"
    );
}

#[test]
fn test_generated_pid_files_match_analyzed_groups() {
    let (mig, ahb) = load_utilmd();

    let pids_dir = Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids");
    let mut mismatches = Vec::new();

    for pid in &ahb.workflows {
        let structure = pid_type_gen::analyze_pid_structure(pid, &mig);
        let file_path = pids_dir.join(format!("pid_{}.rs", pid.id.to_lowercase()));

        let content = std::fs::read_to_string(&file_path).unwrap_or_else(|_| {
            panic!("Missing generated file for PID {}: {:?}", pid.id, file_path)
        });

        // Check each group from the structure appears as a field in the generated file
        for group in &structure.groups {
            let field_name = group.group_id.to_lowercase();
            if !content.contains(&format!("pub {field_name}:")) {
                mismatches.push(format!(
                    "PID {} file missing field for group {}",
                    pid.id, group.group_id
                ));
            }
        }

        // Check top-level segments
        for seg in &structure.top_level_segments {
            let field_name = seg.to_lowercase();
            if !content.contains(&format!("pub {field_name}:")) {
                mismatches.push(format!(
                    "PID {} file missing field for top-level segment {}",
                    pid.id, seg
                ));
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "Generated file mismatches ({}):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn test_all_pids_have_common_base_segments() {
    let (mig, ahb) = load_utilmd();

    // Every UTILMD PID should have UNH, BGM, DTM, UNT as top-level segments
    let expected_base = ["BGM", "DTM", "UNH", "UNT"];
    let mut missing_base = Vec::new();

    for pid in &ahb.workflows {
        let structure = pid_type_gen::analyze_pid_structure(pid, &mig);
        let top_segs: BTreeSet<String> = structure.top_level_segments.iter().cloned().collect();

        for seg in &expected_base {
            if !top_segs.contains(*seg) {
                missing_base.push(format!("PID {} missing base segment {}", pid.id, seg));
            }
        }
    }

    assert!(
        missing_base.is_empty(),
        "PIDs missing base segments:\n{}",
        missing_base.join("\n")
    );
}

#[test]
fn test_pid_group_distribution() {
    let (mig, ahb) = load_utilmd();

    // Collect statistics on group usage across PIDs
    let mut group_counts: BTreeMap<String, usize> = BTreeMap::new();

    for pid in &ahb.workflows {
        let structure = pid_type_gen::analyze_pid_structure(pid, &mig);
        for group in &structure.groups {
            *group_counts.entry(group.group_id.clone()).or_insert(0) += 1;
        }
    }

    let total = ahb.workflows.len();

    // SG2 and SG4 should be present in almost all PIDs (NAD parties and transaction data)
    let sg2_count = group_counts.get("SG2").copied().unwrap_or(0);
    let sg4_count = group_counts.get("SG4").copied().unwrap_or(0);

    assert!(
        sg2_count > total * 9 / 10,
        "SG2 should be in >90% of PIDs, but only in {}/{} ({}%)",
        sg2_count,
        total,
        sg2_count * 100 / total
    );
    assert!(
        sg4_count > total * 9 / 10,
        "SG4 should be in >90% of PIDs, but only in {}/{} ({}%)",
        sg4_count,
        total,
        sg4_count * 100 / total
    );

    // SG1 should be rare (only 7 PIDs)
    let sg1_count = group_counts.get("SG1").copied().unwrap_or(0);
    assert_eq!(
        sg1_count, 7,
        "SG1 should be in exactly 7 PIDs, found {}",
        sg1_count
    );
}

#[test]
#[ignore] // Only run explicitly: cargo test -p automapper-generator generate_real_pid -- --ignored
fn generate_real_pid_types() {
    let output_dir = Path::new("../../crates/mig-types/src/generated");
    let (mig, ahb) = load_utilmd();

    pid_type_gen::generate_pid_types(&mig, &ahb, "FV2504", output_dir)
        .expect("Failed to generate PID types");

    println!("Generated PID types to {:?}", output_dir);
}
