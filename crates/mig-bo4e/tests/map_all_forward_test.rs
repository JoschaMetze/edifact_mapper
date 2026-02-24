//! Integration test for `MappingEngine::map_all_forward()`.
//!
//! Loads a PID-filtered MIG for 55001, assembles a fixture,
//! and verifies that all entity keys are produced with correct values.

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
const MESSAGE_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/message";

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

    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new(MAPPINGS_DIR);
    if !msg_dir.exists() || !tx_dir.exists() {
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

    // Load combined mapping engine (message + transaction) and call map_all_forward
    let engine = MappingEngine::load_merged(&[msg_dir, tx_dir]).unwrap();
    let result = engine.map_all_forward(&tree);

    eprintln!(
        "map_all_forward result:\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // All entity keys should be present (Merkmal data merged into parent entities)
    let expected_entities = [
        "nachricht",
        "marktteilnehmer",
        "prozessdaten",
        "marktlokation",
        "produktpaket",
        "produktpaketPriorisierung",
        "enfgDaten",
        "ansprechpartner",
        "geschaeftspartner",
        // Note: "kontakt" (SG2.SG3) is defined in the message-level mappings but
        // won't appear here because this fixture has no CTA+IC segment in SG3.
    ];

    let obj = result.as_object().expect("result should be JSON object");
    for entity in &expected_entities {
        assert!(obj.contains_key(*entity), "Missing entity key: {entity}");
    }

    // Marktteilnehmer should be an array of 2 (SG2 has 2 reps, no discriminator)
    let mt = obj.get("marktteilnehmer").unwrap();
    assert!(mt.is_array(), "Marktteilnehmer should be an array");
    assert_eq!(
        mt.as_array().unwrap().len(),
        2,
        "Marktteilnehmer should have 2 entries (NAD+MS, NAD+MR)"
    );

    // Spot-check values
    let nachricht = obj.get("nachricht").unwrap();
    assert_eq!(
        nachricht.get("nachrichtentyp").and_then(|v| v.as_str()),
        Some("E01"),
        "Nachricht.nachrichtentyp should be E01"
    );

    let prozess = obj.get("prozessdaten").unwrap();
    assert!(
        prozess.get("vorgangId").and_then(|v| v.as_str()).is_some(),
        "Prozessdaten should have vorgangId"
    );
    assert_eq!(
        prozess.get("pruefidentifikator").and_then(|v| v.as_str()),
        Some("55001"),
        "Prozessdaten should have pruefidentifikator merged from RFF+Z13"
    );

    let malo = obj.get("marktlokation").unwrap();
    assert!(
        malo.get("marktlokationsId")
            .and_then(|v| v.as_str())
            .is_some(),
        "Marktlokation should have marktlokationsId"
    );

    // Marktlokation should have merged companion data from SG10 (Haushaltskunde)
    assert!(
        malo.get("marktlokationEdifact").is_some(),
        "Marktlokation should have marktlokationEdifact companion from SG10"
    );

    // Discriminator-resolved entities should be single objects (not arrays)
    assert!(
        obj.get("produktpaket").unwrap().is_object(),
        "Produktpaket should be a single object (resolved via discriminator)"
    );
    assert!(
        obj.get("ansprechpartner").unwrap().is_object(),
        "Ansprechpartner should be a single object (resolved via discriminator)"
    );
    assert!(
        obj.get("geschaeftspartner").unwrap().is_object(),
        "Geschaeftspartner should be a single object (resolved via discriminator)"
    );

    // Companion fields merged from SG10 into parent entities
    let pp = obj.get("produktpaket").unwrap();
    assert!(
        pp.get("produktpaketEdifact").is_some(),
        "Produktpaket should have companion data from SG10 (Produkteigenschaft)"
    );
    let pp_companion = pp.get("produktpaketEdifact").unwrap();
    assert_eq!(
        pp_companion.get("merkmalCode").and_then(|v| v.as_str()),
        Some("Z66"),
        "Produktpaket companion should have merkmalCode Z66"
    );

    let ppp = obj.get("produktpaketPriorisierung").unwrap();
    assert!(
        ppp.get("produktpaketPriorisierungEdifact").is_some(),
        "ProduktpaketPriorisierung should have companion data from SG10"
    );

    let enfg = obj.get("enfgDaten").unwrap();
    assert!(
        enfg.get("enfgDatenEdifact").is_some(),
        "EnfgDaten should have companion data from SG10 (EnFG privilege)"
    );

    eprintln!(
        "map_all_forward: all {} entity keys present and correct",
        expected_entities.len()
    );
}

#[test]
fn test_map_all_forward_55001_with_code_enrichment() {
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

    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new(MAPPINGS_DIR);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping: mappings not found");
        return;
    }

    let schema_path =
        Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");
    if !schema_path.exists() {
        eprintln!("Skipping: PID schema not found");
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

    // Load engine WITH code lookup for enrichment
    let code_lookup = mig_bo4e::code_lookup::CodeLookup::from_schema_file(schema_path).unwrap();
    let engine = MappingEngine::load_merged(&[msg_dir, tx_dir])
        .unwrap()
        .with_code_lookup(code_lookup);

    let result = engine.map_all_forward(&tree);

    eprintln!(
        "map_all_forward (enriched) result:\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // Companion fields should now be enriched objects
    let malo = result.get("marktlokation").unwrap();
    let malo_companion = malo.get("marktlokationEdifact").unwrap();
    let hk = malo_companion.get("haushaltskunde").unwrap();
    assert!(
        hk.is_object(),
        "haushaltskunde should be an enriched object, got: {hk}"
    );
    assert!(hk.get("code").is_some(), "should have code field");
    assert!(hk.get("meaning").is_some(), "should have meaning field");

    let pp = result.get("produktpaket").unwrap();
    let pp_companion = pp.get("produktpaketEdifact").unwrap();
    let mc = pp_companion.get("merkmalCode").unwrap();
    assert_eq!(mc.get("code").and_then(|v| v.as_str()), Some("Z66"));
    assert_eq!(
        mc.get("meaning").and_then(|v| v.as_str()),
        Some("Produkteigenschaft")
    );

    // Data-type fields should still be plain strings
    let pe = pp_companion.get("produkteigenschaftCode");
    if let Some(pe) = pe {
        assert!(
            pe.is_string(),
            "data-type companion field should remain a plain string, got: {pe}"
        );
    }
}
