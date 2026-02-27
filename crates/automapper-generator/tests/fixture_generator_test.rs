use automapper_generator::fixture_generator::generate_fixture;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use mig_assembly::assembler::Assembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use std::collections::HashSet;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";

fn load_pid_assembler(
    pid: &str,
) -> Option<(
    automapper_generator::schema::mig::MigSchema,
    HashSet<String>,
)> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        eprintln!("MIG/AHB XML not found, skipping");
        return None;
    }

    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid_entry = ahb.workflows.iter().find(|w| w.id == pid)?;
    let numbers: HashSet<String> = pid_entry.segment_numbers.iter().cloned().collect();

    Some((mig, numbers))
}

#[test]
fn test_generate_fixture_55001_tokenizes() {
    let schema_path =
        Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");
    if !schema_path.exists() {
        eprintln!("Schema not found, skipping");
        return;
    }

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(schema_path).unwrap()).unwrap();

    let edi = generate_fixture(&schema);

    // Must tokenize without errors
    let segments = parse_to_segments(edi.as_bytes()).expect("Generated fixture must tokenize");
    assert!(
        segments.len() > 10,
        "Expected at least 10 segments, got {}",
        segments.len()
    );

    // Check envelope segments exist
    let tags: Vec<&str> = segments.iter().map(|s| s.id.as_str()).collect();
    assert!(tags.contains(&"UNB"), "Missing UNB");
    assert!(tags.contains(&"UNH"), "Missing UNH");
    assert!(tags.contains(&"BGM"), "Missing BGM");
    assert!(tags.contains(&"UNT"), "Missing UNT");
    assert!(tags.contains(&"UNZ"), "Missing UNZ");

    eprintln!(
        "PID 55001 generated fixture: {} segments, tokenizes OK",
        segments.len()
    );
}

#[test]
fn test_generate_fixture_55001_assembles() {
    let schema_path =
        Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");
    if !schema_path.exists() {
        eprintln!("Schema not found, skipping");
        return;
    }

    let Some((mig, numbers)) = load_pid_assembler("55001") else {
        return;
    };

    let filtered = filter_mig_for_pid(&mig, &numbers);
    let assembler = Assembler::new(&filtered);

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(schema_path).unwrap()).unwrap();

    let edi = generate_fixture(&schema);
    let segments = parse_to_segments(edi.as_bytes()).expect("Must tokenize");
    let tree = assembler
        .assemble_generic(&segments)
        .expect("Must assemble");

    // Should have groups captured
    assert!(
        !tree.groups.is_empty(),
        "Expected SG groups in assembled tree"
    );

    eprintln!(
        "PID 55001 assembled: {} root segments, {} top-level groups",
        tree.segments.len(),
        tree.groups.len()
    );
}

#[test]
fn test_generate_fixture_uncovered_pid_55043() {
    let schema_path =
        Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55043_schema.json");
    if !schema_path.exists() {
        eprintln!("Schema not found, skipping");
        return;
    }

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(schema_path).unwrap()).unwrap();

    let edi = generate_fixture(&schema);
    let segments = parse_to_segments(edi.as_bytes()).expect("Must tokenize");
    assert!(segments.len() > 50, "PID 55043 should be a large fixture");

    // Check PID-specific segments are present
    let tags: Vec<&str> = segments.iter().map(|s| s.id.as_str()).collect();
    assert!(tags.contains(&"LOC"), "Missing LOC segments");
    assert!(tags.contains(&"SEQ"), "Missing SEQ segments");
    assert!(tags.contains(&"CCI"), "Missing CCI segments");
    assert!(tags.contains(&"NAD"), "Missing NAD segments");

    eprintln!(
        "PID 55043 generated fixture: {} segments, tokenizes OK",
        segments.len()
    );

    // Try assembly if MIG/AHB available
    if let Some((mig, numbers)) = load_pid_assembler("55043") {
        let filtered = filter_mig_for_pid(&mig, &numbers);
        let assembler = Assembler::new(&filtered);
        let tree = assembler
            .assemble_generic(&segments)
            .expect("Must assemble");
        assert!(!tree.groups.is_empty());
        eprintln!(
            "PID 55043 assembled: {} root segments, {} top-level groups",
            tree.segments.len(),
            tree.groups.len()
        );
    }
}
