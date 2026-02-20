//! Integration tests for entity-level forward+reverse mapping.
//!
//! Uses real PID 55001 fixture with PID-filtered MIG to test:
//! - Forward: assembled tree → BO4E JSON
//! - Reverse: BO4E JSON → assembled group instance
//! - Roundtrip: tree → BO4E → tree identity

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::{AssembledTree, Assembler};
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
const MAPPINGS_DIR: &str = "../../mappings/FV2504/UTILMD_Strom";

fn load_pid_filtered_mig(pid_id: &str) -> Option<(MigSchema, HashSet<String>)> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        return None;
    }

    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    let filtered = filter_mig_for_pid(&mig, &numbers);
    Some((filtered, numbers))
}

fn assemble_fixture(mig: &MigSchema, fixture_name: &str) -> Option<AssembledTree> {
    let path = Path::new(FIXTURE_DIR).join(fixture_name);
    if !path.exists() {
        return None;
    }
    let content = std::fs::read(&path).ok()?;
    let segments = parse_to_segments(&content).ok()?;
    let assembler = Assembler::new(mig);
    assembler.assemble_generic(&segments).ok()
}

fn load_engine() -> Option<MappingEngine> {
    let dir = Path::new(MAPPINGS_DIR);
    if !dir.exists() {
        return None;
    }
    MappingEngine::load(dir).ok()
}

// ── Marktlokation: SG4.SG5 → LOC+Z16 ──

#[test]
fn test_marktlokation_forward_mapping() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktlokation")
        .expect("Marktlokation definition should exist");

    let bo4e = engine.map_forward(&tree, def, 0);

    assert_eq!(
        bo4e.get("marktlokations_id").and_then(|v| v.as_str()),
        Some("12345678900"),
        "Should extract MaLo ID from LOC+Z16 segment"
    );
}

#[test]
fn test_marktlokation_reverse_mapping() {
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktlokation")
        .expect("Marktlokation definition should exist");

    let bo4e = serde_json::json!({
        "marktlokations_id": "12345678900"
    });

    let instance = engine.map_reverse(&bo4e, def);

    // Should produce LOC segment with qualifier and ID
    assert_eq!(instance.segments.len(), 1, "Should have one LOC segment");
    let loc = &instance.segments[0];
    assert_eq!(loc.tag, "LOC");
    assert_eq!(loc.elements.len(), 2, "LOC should have qualifier + ID");
    assert_eq!(
        loc.elements[0],
        vec!["Z16"],
        "Qualifier should be Z16 (from default)"
    );
    assert_eq!(
        loc.elements[1],
        vec!["12345678900"],
        "ID should match BO4E value"
    );
}

#[test]
fn test_marktlokation_roundtrip() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktlokation")
        .expect("Marktlokation definition should exist");

    // Forward: tree → BO4E
    let bo4e = engine.map_forward(&tree, def, 0);

    // Reverse: BO4E → tree instance
    let reconstructed = engine.map_reverse(&bo4e, def);

    // Compare with original LOC from SG4.SG5
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG5", 0)
        .expect("SG4.SG5 should exist in tree");

    let original_loc = original
        .segments
        .iter()
        .find(|s| s.tag == "LOC")
        .expect("Original should have LOC segment");
    let reconstructed_loc = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "LOC")
        .expect("Reconstructed should have LOC segment");

    assert_eq!(
        original_loc.elements, reconstructed_loc.elements,
        "LOC segment elements should roundtrip identically"
    );
}

#[test]
fn test_nested_group_navigation() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };

    // Navigate to SG4.SG5 directly
    let sg5_instance = MappingEngine::resolve_group_instance(&tree, "SG4.SG5", 0);
    assert!(sg5_instance.is_some(), "SG4.SG5 should be navigable");

    let instance = sg5_instance.unwrap();
    let loc = instance.segments.iter().find(|s| s.tag == "LOC");
    assert!(loc.is_some(), "SG5 should contain a LOC segment");
    assert_eq!(loc.unwrap().elements[0][0], "Z16");

    // Navigate to SG4.SG6 (RFF reference)
    let sg6_instance = MappingEngine::resolve_group_instance(&tree, "SG4.SG6", 0);
    assert!(sg6_instance.is_some(), "SG4.SG6 should be navigable");

    let instance = sg6_instance.unwrap();
    let rff = instance.segments.iter().find(|s| s.tag == "RFF");
    assert!(rff.is_some(), "SG6 should contain an RFF segment");

    // Navigate to SG4.SG8 (SEQ groups — 4 repetitions)
    for rep in 0..4 {
        let sg8 = MappingEngine::resolve_group_instance(&tree, "SG4.SG8", rep);
        assert!(sg8.is_some(), "SG4.SG8[{rep}] should be navigable");
    }
    let sg8_rep4 = MappingEngine::resolve_group_instance(&tree, "SG4.SG8", 4);
    assert!(sg8_rep4.is_none(), "SG4.SG8[4] should be out of range");

    // Top-level SG2 still works
    let sg2 = MappingEngine::resolve_group_instance(&tree, "SG2", 0);
    assert!(sg2.is_some(), "Top-level SG2 should be navigable");
    let sg2 = MappingEngine::resolve_group_instance(&tree, "SG2", 1);
    assert!(sg2.is_some(), "SG2[1] should be navigable (NAD+MR)");
}

// ── Prozessdaten: SG4 → IDE+24, DTM+92, DTM+93, STS ──

#[test]
fn test_prozessdaten_forward_mapping() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Prozessdaten")
        .expect("Prozessdaten definition should exist");

    let bo4e = engine.map_forward(&tree, def, 0);

    assert_eq!(
        bo4e.get("vorgang_id").and_then(|v| v.as_str()),
        Some("ALEXANDE542328517"),
        "Should extract Vorgang ID from IDE"
    );
    assert!(
        bo4e.get("gueltig_ab")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v.starts_with("2025053")),
        "Should extract valid-from date from DTM+92"
    );
    assert!(
        bo4e.get("gueltig_bis")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v.starts_with("2025123")),
        "Should extract valid-to date from DTM+93"
    );
    assert_eq!(
        bo4e.get("transaktionsgrund").and_then(|v| v.as_str()),
        Some("E01"),
        "Should extract Transaktionsgrund from STS"
    );
}

#[test]
fn test_prozessdaten_reverse_mapping() {
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Prozessdaten")
        .expect("Prozessdaten definition should exist");

    let bo4e = serde_json::json!({
        "vorgang_id": "TEST123",
        "gueltig_ab": "202505312200+00",
        "gueltig_bis": "202512312300+00",
        "transaktionsgrund": "E01",
        "transaktionsgrund_ergaenzung": "ZW4",
        "transaktionsgrund_ergaenzung_befristete_anmeldung": "E03"
    });

    let instance = engine.map_reverse(&bo4e, def);

    // Should produce IDE, DTM (x2), STS segments
    let tags: Vec<&str> = instance.segments.iter().map(|s| s.tag.as_str()).collect();
    assert!(tags.contains(&"IDE"), "Should have IDE segment");
    assert!(tags.contains(&"STS"), "Should have STS segment");

    // Should have two separate DTM segments (one for [92], one for [93])
    let dtm_count = tags.iter().filter(|&&t| t == "DTM").count();
    assert_eq!(dtm_count, 2, "Should have two DTM segments (92 and 93)");

    // Check DTM qualifiers
    let dtms: Vec<&mig_assembly::assembler::AssembledSegment> = instance
        .segments
        .iter()
        .filter(|s| s.tag == "DTM")
        .collect();
    let dtm_qualifiers: Vec<&str> = dtms.iter().map(|d| d.elements[0][0].as_str()).collect();
    assert!(
        dtm_qualifiers.contains(&"92"),
        "Should have DTM with qualifier 92"
    );
    assert!(
        dtm_qualifiers.contains(&"93"),
        "Should have DTM with qualifier 93"
    );
}

#[test]
fn test_prozessdaten_roundtrip() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Prozessdaten")
        .expect("Prozessdaten definition should exist");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4", 0)
        .expect("SG4 should exist");

    // Forward
    let bo4e = engine.map_forward(&tree, def, 0);

    // Reverse
    let reconstructed = engine.map_reverse(&bo4e, def);

    // Compare IDE segment
    let orig_ide = original.segments.iter().find(|s| s.tag == "IDE").unwrap();
    let recon_ide = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "IDE")
        .unwrap();
    assert_eq!(orig_ide.elements, recon_ide.elements, "IDE should roundtrip");

    // Compare DTM segments by qualifier
    for qualifier in &["92", "93"] {
        let orig_dtm = original
            .segments
            .iter()
            .find(|s| {
                s.tag == "DTM" && s.elements[0].first().map(|v| v.as_str()) == Some(qualifier)
            })
            .unwrap_or_else(|| panic!("Original should have DTM+{qualifier}"));
        let recon_dtm = reconstructed
            .segments
            .iter()
            .find(|s| {
                s.tag == "DTM" && s.elements[0].first().map(|v| v.as_str()) == Some(qualifier)
            })
            .unwrap_or_else(|| panic!("Reconstructed should have DTM+{qualifier}"));
        assert_eq!(
            orig_dtm.elements, recon_dtm.elements,
            "DTM+{qualifier} should roundtrip"
        );
    }
}

// ── ProzessReferenz: SG4.SG6 → RFF+Z13 ──

#[test]
fn test_prozess_referenz_forward_mapping() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("ProzessReferenz")
        .expect("ProzessReferenz definition should exist");

    let bo4e = engine.map_forward(&tree, def, 0);

    assert_eq!(
        bo4e.get("pid_id").and_then(|v| v.as_str()),
        Some("55001"),
        "Should extract PID reference from RFF+Z13"
    );
}

#[test]
fn test_prozess_referenz_reverse_mapping() {
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("ProzessReferenz")
        .expect("ProzessReferenz definition should exist");

    let bo4e = serde_json::json!({ "pid_id": "55001" });
    let instance = engine.map_reverse(&bo4e, def);

    assert_eq!(instance.segments.len(), 1);
    let rff = &instance.segments[0];
    assert_eq!(rff.tag, "RFF");
    assert_eq!(rff.elements[0], vec!["Z13", "55001"]);
}

#[test]
fn test_prozess_referenz_roundtrip() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("ProzessReferenz")
        .expect("ProzessReferenz definition should exist");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG6", 0)
        .expect("SG4.SG6 should exist");

    let bo4e = engine.map_forward(&tree, def, 0);
    let reconstructed = engine.map_reverse(&bo4e, def);

    let original_rff = original.segments.iter().find(|s| s.tag == "RFF").unwrap();
    let reconstructed_rff = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "RFF")
        .unwrap();

    assert_eq!(
        original_rff.elements, reconstructed_rff.elements,
        "RFF elements should roundtrip identically"
    );
}

// ── Marktteilnehmer: SG2 → NAD+MS / NAD+MR ──

#[test]
fn test_marktteilnehmer_forward_mapping_ms() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktteilnehmer")
        .expect("Marktteilnehmer definition should exist");

    // SG2[0] = NAD+MS (sender)
    let bo4e = engine.map_forward(&tree, def, 0);

    assert_eq!(
        bo4e.get("marktrolle").and_then(|v| v.as_str()),
        Some("MS"),
        "Should extract marktrolle MS"
    );
    assert_eq!(
        bo4e.get("rollencodenummer").and_then(|v| v.as_str()),
        Some("9978842000002"),
        "Should extract sender MP ID as rollencodenummer"
    );
    assert_eq!(
        bo4e.get("rollencodetyp").and_then(|v| v.as_str()),
        Some("BDEW"),
        "Should translate 293 → BDEW via enum_map"
    );
}

#[test]
fn test_marktteilnehmer_forward_mapping_mr() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktteilnehmer")
        .expect("Marktteilnehmer definition should exist");

    // SG2[1] = NAD+MR (recipient)
    let bo4e = engine.map_forward(&tree, def, 1);

    assert_eq!(
        bo4e.get("marktrolle").and_then(|v| v.as_str()),
        Some("MR"),
        "Should extract marktrolle MR"
    );
    assert_eq!(
        bo4e.get("rollencodenummer").and_then(|v| v.as_str()),
        Some("9900269000000"),
        "Should extract recipient MP ID as rollencodenummer"
    );
    assert_eq!(
        bo4e.get("rollencodetyp").and_then(|v| v.as_str()),
        Some("BDEW"),
        "Should translate 293 → BDEW via enum_map"
    );
}

#[test]
fn test_marktteilnehmer_reverse_mapping() {
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktteilnehmer")
        .expect("Marktteilnehmer definition should exist");

    let bo4e = serde_json::json!({
        "marktrolle": "MS",
        "rollencodenummer": "9978842000002",
        "rollencodetyp": "BDEW"
    });

    let instance = engine.map_reverse(&bo4e, def);

    assert_eq!(instance.segments.len(), 1, "Should have one NAD segment");
    let nad = &instance.segments[0];
    assert_eq!(nad.tag, "NAD");
    assert_eq!(nad.elements.len(), 2, "NAD should have qualifier + C082");
    assert_eq!(nad.elements[0], vec!["MS"]);
    // C082: party_id at [0], empty at [1], agency at [2]
    assert_eq!(nad.elements[1].len(), 3);
    assert_eq!(nad.elements[1][0], "9978842000002");
    assert_eq!(nad.elements[1][1], "", "Middle component should be empty");
    assert_eq!(
        nad.elements[1][2], "293",
        "BDEW should reverse-map to 293"
    );
}

#[test]
fn test_marktteilnehmer_roundtrip() {
    let Some((mig, _)) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktteilnehmer")
        .expect("Marktteilnehmer definition should exist");

    // Test both repetitions (MS and MR)
    for rep in 0..2 {
        let original = MappingEngine::resolve_group_instance(&tree, "SG2", rep)
            .unwrap_or_else(|| panic!("SG2[{rep}] should exist"));

        // Forward: tree → BO4E
        let bo4e = engine.map_forward(&tree, def, rep);

        // Reverse: BO4E → tree instance
        let reconstructed = engine.map_reverse(&bo4e, def);

        let original_nad = original
            .segments
            .iter()
            .find(|s| s.tag == "NAD")
            .unwrap_or_else(|| panic!("SG2[{rep}] should have NAD"));
        let reconstructed_nad = reconstructed
            .segments
            .iter()
            .find(|s| s.tag == "NAD")
            .expect("Reconstructed should have NAD");

        assert_eq!(
            original_nad.elements, reconstructed_nad.elements,
            "NAD elements should roundtrip identically for SG2[{rep}]"
        );
    }
}
