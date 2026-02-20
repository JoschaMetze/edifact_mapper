use automapper_generator::codegen::pid_type_gen;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
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

#[test]
#[ignore] // Only run explicitly: cargo test -p automapper-generator generate_real_pid -- --ignored
fn generate_real_pid_types() {
    let output_dir = Path::new("../../crates/mig-types/src/generated");
    let (mig, ahb) = load_utilmd();

    pid_type_gen::generate_pid_types(&mig, &ahb, "FV2504", output_dir)
        .expect("Failed to generate PID types");

    println!("Generated PID types to {:?}", output_dir);
}
