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
