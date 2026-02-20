//! Integration test for `MappingEngine::map_all_forward()`.
//!
//! Loads a PID-filtered MIG for 55001, assembles a fixture,
//! and verifies that all 15 entity keys are produced with correct values.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use mig_assembly::assembler::Assembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use std::collections::HashSet;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/pid_55001";

#[test]
fn test_map_all_forward_55001() {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        eprintln!("Skipping: MIG/AHB XML not found");
        return;
    }

    let fixture_path = Path::new(FIXTURE_DIR).join("55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !fixture_path.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }

    let mappings_dir = Path::new(MAPPINGS_DIR);
    if !mappings_dir.exists() {
        eprintln!("Skipping: mappings not found");
        return;
    }

    // Load PID-filtered MIG
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
    let pid = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(&mig, &numbers);

    // Assemble fixture
    let content = std::fs::read(&fixture_path).unwrap();
    let segments = parse_to_segments(&content).unwrap();
    let assembler = Assembler::new(&filtered_mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    // Load mapping engine and call map_all_forward
    let engine = MappingEngine::load(mappings_dir).unwrap();
    let result = engine.map_all_forward(&tree);

    eprintln!(
        "map_all_forward result:\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // All 15 entity keys should be present
    let expected_entities = [
        "Nachricht",
        "Marktteilnehmer",
        "Prozessdaten",
        "Marktlokation",
        "ProzessReferenz",
        "Zaehlpunkt",
        "Messstellenbetrieb",
        "Geraet",
        "Netznutzungsabrechnung",
        "MerkmalZaehlpunkt",
        "MerkmalMessstellenbetrieb",
        "MerkmalGeraet",
        "MerkmalNetznutzung",
        "Ansprechpartner",
        "Geschaeftspartner",
    ];

    let obj = result.as_object().expect("result should be JSON object");
    for entity in &expected_entities {
        assert!(obj.contains_key(*entity), "Missing entity key: {entity}");
    }

    // Marktteilnehmer should be an array of 2 (SG2 has 2 reps, no discriminator)
    let mt = obj.get("Marktteilnehmer").unwrap();
    assert!(mt.is_array(), "Marktteilnehmer should be an array");
    assert_eq!(
        mt.as_array().unwrap().len(),
        2,
        "Marktteilnehmer should have 2 entries (NAD+MS, NAD+MR)"
    );

    // Spot-check values
    let nachricht = obj.get("Nachricht").unwrap();
    assert_eq!(
        nachricht.get("nachrichtentyp").and_then(|v| v.as_str()),
        Some("E01"),
        "Nachricht.nachrichtentyp should be E01"
    );

    let prozess = obj.get("Prozessdaten").unwrap();
    assert!(
        prozess.get("vorgang_id").and_then(|v| v.as_str()).is_some(),
        "Prozessdaten should have vorgang_id"
    );

    let malo = obj.get("Marktlokation").unwrap();
    assert!(
        malo.get("marktlokations_id")
            .and_then(|v| v.as_str())
            .is_some(),
        "Marktlokation should have marktlokations_id"
    );

    // Discriminator-resolved entities should be single objects (not arrays)
    assert!(
        obj.get("Zaehlpunkt").unwrap().is_object(),
        "Zaehlpunkt should be a single object (resolved via discriminator)"
    );
    assert!(
        obj.get("Ansprechpartner").unwrap().is_object(),
        "Ansprechpartner should be a single object (resolved via discriminator)"
    );
    assert!(
        obj.get("Geschaeftspartner").unwrap().is_object(),
        "Geschaeftspartner should be a single object (resolved via discriminator)"
    );

    eprintln!(
        "map_all_forward: all {} entity keys present and correct",
        expected_entities.len()
    );
}
