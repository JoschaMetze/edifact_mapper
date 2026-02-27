//! Comprehensive roundtrip tests for PID 55001 — all entities.
//!
//! For each entity: assemble tree from fixture → forward map → reverse map →
//! compare original vs. reconstructed segment elements.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use mig_assembly::assembler::{AssembledTree, Assembler};
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_types::schema::mig::MigSchema;
use std::collections::HashSet;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/pid_55001";
const MESSAGE_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/message";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(std::path::Path::new(SCHEMA_DIR))
}

fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        return None;
    }
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    Some(filter_mig_for_pid(&mig, &numbers))
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
    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new(MAPPINGS_DIR);
    if !msg_dir.exists() || !tx_dir.exists() {
        return None;
    }
    MappingEngine::load_merged(&[msg_dir, tx_dir])
        .ok()
        .map(|e| e.with_path_resolver(path_resolver()))
}

/// Helper: compare segment elements from original instance vs reverse-mapped.
/// Returns (matched, total) counts.
fn compare_segments(
    entity: &str,
    original_segments: &[mig_assembly::assembler::AssembledSegment],
    reconstructed_segments: &[mig_assembly::assembler::AssembledSegment],
) -> (usize, usize) {
    let mut matched = 0;
    let mut total = 0;

    for orig in original_segments {
        total += 1;
        // Find matching segment by tag (and qualifier if present)
        let recon = reconstructed_segments.iter().find(|r| {
            r.tag == orig.tag
                && r.elements.first().and_then(|e| e.first())
                    == orig.elements.first().and_then(|e| e.first())
        });

        if let Some(recon) = recon {
            if orig.elements == recon.elements {
                matched += 1;
            } else {
                eprintln!("  {} {}: elements mismatch", entity, orig.tag,);
                eprintln!("    original:      {:?}", orig.elements);
                eprintln!("    reconstructed: {:?}", recon.elements);
            }
        } else {
            eprintln!(
                "  {} {}: not found in reconstructed (original: {:?})",
                entity, orig.tag, orig.elements
            );
        }
    }

    (matched, total)
}

// ── Nachricht: root-level BGM + DTM ──

#[test]
fn test_nachricht_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Nachricht")
        .expect("Nachricht definition");

    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("Nachricht BO4E: {}", bo4e);

    assert_eq!(
        bo4e.get("nachrichtentyp").and_then(|v| v.as_str()),
        Some("E01")
    );
    assert_eq!(
        bo4e.get("nachrichtennummer").and_then(|v| v.as_str()),
        Some("ALEXANDE951842BGM")
    );
    assert!(bo4e
        .get("erstellungsdatum")
        .and_then(|v| v.as_str())
        .is_some());

    let reconstructed = engine.map_reverse(&bo4e, def);

    // BGM roundtrip
    let orig_bgm = tree.segments.iter().find(|s| s.tag == "BGM").unwrap();
    let recon_bgm = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "BGM")
        .unwrap();
    assert_eq!(
        orig_bgm.elements, recon_bgm.elements,
        "BGM should roundtrip"
    );

    // DTM+137 roundtrip
    let orig_dtm = tree
        .segments
        .iter()
        .find(|s| s.tag == "DTM" && s.elements[0][0] == "137")
        .unwrap();
    let recon_dtm = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "DTM" && s.elements[0][0] == "137")
        .unwrap();
    assert_eq!(
        orig_dtm.elements, recon_dtm.elements,
        "DTM+137 should roundtrip"
    );
}

// ── Marktteilnehmer: SG2 → NAD+MS, NAD+MR ──

#[test]
fn test_marktteilnehmer_roundtrip_all() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Marktteilnehmer")
        .expect("Marktteilnehmer definition");

    for rep in 0..2 {
        let original = MappingEngine::resolve_group_instance(&tree, "SG2", rep).unwrap();
        let bo4e = engine.map_forward(&tree, def, rep);
        let reconstructed = engine.map_reverse(&bo4e, def);

        let orig_nad = original.segments.iter().find(|s| s.tag == "NAD").unwrap();
        let recon_nad = reconstructed
            .segments
            .iter()
            .find(|s| s.tag == "NAD")
            .unwrap();
        assert_eq!(
            orig_nad.elements, recon_nad.elements,
            "NAD should roundtrip for SG2[{rep}]"
        );
    }
}

// ── Prozessdaten: SG4 → IDE, DTM[92], DTM[93], STS ──

#[test]
fn test_prozessdaten_roundtrip_full() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Prozessdaten" && d.meta.source_group == "SG4")
        .expect("Prozessdaten SG4 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    let reconstructed = engine.map_reverse(&bo4e, def);

    // IDE
    let orig_ide = original.segments.iter().find(|s| s.tag == "IDE").unwrap();
    let recon_ide = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "IDE")
        .unwrap();
    assert_eq!(
        orig_ide.elements, recon_ide.elements,
        "IDE should roundtrip"
    );

    // DTM+92 and DTM+93
    for q in &["92", "93"] {
        let orig = original
            .segments
            .iter()
            .find(|s| s.tag == "DTM" && s.elements[0][0] == *q)
            .unwrap();
        let recon = reconstructed
            .segments
            .iter()
            .find(|s| s.tag == "DTM" && s.elements[0][0] == *q)
            .unwrap();
        assert_eq!(orig.elements, recon.elements, "DTM+{q} should roundtrip");
    }

    // STS
    let orig_sts = original.segments.iter().find(|s| s.tag == "STS").unwrap();
    let recon_sts = reconstructed
        .segments
        .iter()
        .find(|s| s.tag == "STS")
        .unwrap();
    assert_eq!(
        orig_sts.elements, recon_sts.elements,
        "STS should roundtrip"
    );
}

// ── Marktlokation: SG4.SG5 → LOC+Z16 ──

#[test]
fn test_marktlokation_roundtrip_full() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Marktlokation" && d.meta.source_group == "SG4.SG5")
        .expect("Marktlokation SG5 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG5", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) =
        compare_segments("Marktlokation", &original.segments, &reconstructed.segments);
    assert_eq!(
        matched, total,
        "Marktlokation: all {total} segments should roundtrip"
    );
}

// ── Prozessdaten RFF+Z13: SG4.SG6 → pruefidentifikator ──

#[test]
fn test_prozessdaten_rff_z13_roundtrip_full() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.source_group == "SG4.SG6")
        .expect("SG4.SG6 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG6", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "Prozessdaten(RFF)",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "Prozessdaten(RFF): all {total} segments should roundtrip"
    );
}

// ── Produktpaket: SG4.SG8 rep 0 → SEQ+Z79, PIA ──

#[test]
fn test_produktpaket_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Produktpaket" && d.meta.source_group == "SG4.SG8")
        .expect("Produktpaket SG8 definition");

    // SG8 rep 0 = Z79 (Bestandteil eines Produktpakets)
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("Produktpaket BO4E: {}", bo4e);

    assert_eq!(
        bo4e.get("produktpaketId").and_then(|v| v.as_str()),
        Some("1")
    );
    assert_eq!(
        bo4e.get("produktCode").and_then(|v| v.as_str()),
        Some("9991000002082")
    );

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) =
        compare_segments("Produktpaket", &original.segments, &reconstructed.segments);
    assert_eq!(
        matched, total,
        "Produktpaket: all {total} segments should roundtrip"
    );
}

// ── ProduktpaketPriorisierung: SG4.SG8 rep 1 → SEQ+ZH0 ──

#[test]
fn test_produktpaket_priorisierung_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "ProduktpaketPriorisierung" && d.meta.source_group == "SG4.SG8")
        .expect("ProduktpaketPriorisierung SG8 definition");

    // SG8 rep 1 = ZH0 (Priorisierung erforderliches Produktpaket)
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8", 1).unwrap();
    let bo4e = engine.map_forward(&tree, def, 1);
    eprintln!("ProduktpaketPriorisierung BO4E: {}", bo4e);

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "ProduktpaketPriorisierung",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "ProduktpaketPriorisierung: all {total} segments should roundtrip"
    );
}

// ── Marktlokation Daten: SG4.SG8 rep 2 → SEQ+Z01 ──

#[test]
fn test_marktlokation_daten_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Marktlokation" && d.meta.source_group == "SG4.SG8")
        .expect("Marktlokation SG8 definition");

    // SG8 rep 2 = Z01 (Daten der Marktlokation)
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8", 2).unwrap();
    let bo4e = engine.map_forward(&tree, def, 2);
    eprintln!("Marktlokation Daten BO4E: {}", bo4e);

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "Marktlokation Daten",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "Marktlokation Daten: all {total} segments should roundtrip"
    );
}

// ── EnfgDaten: SG4.SG8 rep 3 → SEQ+Z75 ──

#[test]
fn test_enfg_daten_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "EnfgDaten" && d.meta.source_group == "SG4.SG8")
        .expect("EnfgDaten SG8 definition");

    // SG8 rep 3 = Z75 (Daten des Kunden des Lieferanten / EnFG)
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8", 3).unwrap();
    let bo4e = engine.map_forward(&tree, def, 3);
    eprintln!("EnfgDaten BO4E: {}", bo4e);

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) =
        compare_segments("EnfgDaten", &original.segments, &reconstructed.segments);
    assert_eq!(
        matched, total,
        "EnfgDaten: all {total} segments should roundtrip"
    );
}

// ── Produktpaket Zuordnung: SG4.SG8:0.SG10 → CCI+Z66, CAV+ZH9, CAV+ZV4 ──

#[test]
fn test_produktpaket_zuordnung_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Produktpaket" && d.meta.source_group == "SG4.SG8:0.SG10")
        .expect("Produktpaket SG10 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8:0.SG10", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("Produktpaket zuordnung BO4E: {}", bo4e);

    // Companion fields should be under the companion type key
    let companion = bo4e
        .get("produktpaketEdifact")
        .expect("Should have companion");
    assert_eq!(
        companion.get("merkmalCode").and_then(|v| v.as_str()),
        Some("Z66")
    );
    assert_eq!(
        companion
            .get("produkteigenschaftCode")
            .and_then(|v| v.as_str()),
        Some("9991000002107")
    );
    assert_eq!(
        companion.get("produktdetailWert").and_then(|v| v.as_str()),
        Some("4000")
    );

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "Produktpaket zuordnung",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "Produktpaket zuordnung: all {total} segments should roundtrip"
    );
}

// ── ProduktpaketPriorisierung Zuordnung: SG4.SG8:1.SG10 → CCI+Z65, CAV+Z75 ──

#[test]
fn test_produktpaket_priorisierung_zuordnung_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| {
            d.meta.entity == "ProduktpaketPriorisierung" && d.meta.source_group == "SG4.SG8:1.SG10"
        })
        .expect("ProduktpaketPriorisierung SG10 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8:1.SG10", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("ProduktpaketPriorisierung zuordnung BO4E: {}", bo4e);

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "ProduktpaketPriorisierung zuordnung",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "ProduktpaketPriorisierung zuordnung: all {total} segments should roundtrip"
    );
}

// ── Marktlokation Zuordnung: SG4.SG8:2.SG10 → CCI+++Z18 ──

#[test]
fn test_marktlokation_zuordnung_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Marktlokation" && d.meta.source_group == "SG4.SG8:2.SG10")
        .expect("Marktlokation SG10 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8:2.SG10", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("Marktlokation zuordnung BO4E: {}", bo4e);

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "Marktlokation zuordnung",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "Marktlokation zuordnung: all {total} segments should roundtrip"
    );
}

// ── EnfgDaten Zuordnung: SG4.SG8:3.SG10 → CCI+Z61, CAV+ZU5 ──

#[test]
fn test_enfg_daten_zuordnung_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "EnfgDaten" && d.meta.source_group == "SG4.SG8:3.SG10")
        .expect("EnfgDaten SG10 definition");

    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG8:3.SG10", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("EnfgDaten zuordnung BO4E: {}", bo4e);

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "EnfgDaten zuordnung",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "EnfgDaten zuordnung: all {total} segments should roundtrip"
    );
}

// ── Ansprechpartner: SG4.SG12 rep 0 → NAD+Z09 ──

#[test]
fn test_ansprechpartner_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Ansprechpartner")
        .expect("Ansprechpartner definition");

    // SG12 rep 0 = Z09
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG12", 0).unwrap();
    let bo4e = engine.map_forward(&tree, def, 0);
    eprintln!("Ansprechpartner BO4E: {}", bo4e);

    assert_eq!(
        bo4e.get("nachname").and_then(|v| v.as_str()),
        Some("Muster")
    );
    assert_eq!(bo4e.get("vorname").and_then(|v| v.as_str()), Some("Max"));
    assert_eq!(bo4e.get("anrede").and_then(|v| v.as_str()), Some("Herr"));

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "Ansprechpartner",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "Ansprechpartner: all {total} segments should roundtrip"
    );
}

// ── Geschaeftspartner: SG4.SG12 rep 1 → NAD+Z04 ──

#[test]
fn test_geschaeftspartner_roundtrip() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let def = engine
        .definition_for_entity("Geschaeftspartner")
        .expect("Geschaeftspartner definition");

    // SG12 rep 1 = Z04
    let original = MappingEngine::resolve_group_instance(&tree, "SG4.SG12", 1).unwrap();
    let bo4e = engine.map_forward(&tree, def, 1);
    eprintln!("Geschaeftspartner BO4E: {}", bo4e);

    assert_eq!(
        bo4e.get("nachname").and_then(|v| v.as_str()),
        Some("Muster")
    );
    assert_eq!(bo4e.get("ort").and_then(|v| v.as_str()), Some("Berlin"));
    assert_eq!(
        bo4e.get("postleitzahl").and_then(|v| v.as_str()),
        Some("10115")
    );
    assert_eq!(bo4e.get("land").and_then(|v| v.as_str()), Some("DE"));

    let reconstructed = engine.map_reverse(&bo4e, def);

    let (matched, total) = compare_segments(
        "Geschaeftspartner",
        &original.segments,
        &reconstructed.segments,
    );
    assert_eq!(
        matched, total,
        "Geschaeftspartner: all {total} segments should roundtrip"
    );
}

// ── Comprehensive: all entities in one test ──

#[test]
fn test_all_entities_roundtrip_55001() {
    let Some(mig) = load_pid_filtered_mig("55001") else {
        return;
    };
    let Some(tree) = assemble_fixture(&mig, "55001_UTILMD_S2.1_ALEXANDE121980.edi") else {
        return;
    };
    let Some(engine) = load_engine() else { return };

    let mut total_matched = 0;
    let mut total_segments = 0;

    // Map of (entity_name, source_group, repetition)
    let entities: Vec<(&str, &str, usize)> = vec![
        ("Nachricht", "", 0),
        ("Marktteilnehmer", "SG2", 0),
        ("Marktteilnehmer", "SG2", 1),
        ("Prozessdaten", "SG4", 0),
        ("Marktlokation", "SG4.SG5", 0),
        ("Prozessdaten", "SG4.SG6", 0),
        ("Produktpaket", "SG4.SG8", 0),
        ("ProduktpaketPriorisierung", "SG4.SG8", 1),
        ("Marktlokation", "SG4.SG8", 2),
        ("EnfgDaten", "SG4.SG8", 3),
        ("Produktpaket", "SG4.SG8:0.SG10", 0),
        ("ProduktpaketPriorisierung", "SG4.SG8:1.SG10", 0),
        ("Marktlokation", "SG4.SG8:2.SG10", 0),
        ("EnfgDaten", "SG4.SG8:3.SG10", 0),
        ("Ansprechpartner", "SG4.SG12", 0),
        ("Geschaeftspartner", "SG4.SG12", 1),
    ];

    for (entity_name, source_group, rep) in &entities {
        let def = engine
            .definitions()
            .iter()
            .find(|d| d.meta.entity == *entity_name && d.meta.source_group == *source_group)
            .unwrap_or_else(|| panic!("Missing definition for {entity_name} at {source_group}"));

        // Get original segments
        let original_segments = if source_group.is_empty() {
            // Root-level segments (pre-group only, skip transport UNB/UNH)
            tree.segments[..tree.post_group_start]
                .iter()
                .filter(|s| s.tag != "UNB" && s.tag != "UNH")
                .cloned()
                .collect::<Vec<_>>()
        } else {
            let instance = MappingEngine::resolve_group_instance(&tree, source_group, *rep)
                .unwrap_or_else(|| panic!("Failed to resolve {source_group}[{rep}]"));
            instance.segments.clone()
        };

        // Forward + reverse
        let bo4e = engine.map_forward(&tree, def, *rep);
        let reconstructed = engine.map_reverse(&bo4e, def);

        let (matched, total) =
            compare_segments(entity_name, &original_segments, &reconstructed.segments);
        total_matched += matched;
        total_segments += total;

        if matched == total {
            eprintln!(
                "  {} [{source_group}:{rep}] — {matched}/{total} segments OK",
                entity_name
            );
        } else {
            eprintln!(
                "  {} [{source_group}:{rep}] — {matched}/{total} segments (MISMATCH)",
                entity_name
            );
        }
    }

    eprintln!("\nTotal: {total_matched}/{total_segments} segments roundtrip byte-identical");
    assert_eq!(
        total_matched, total_segments,
        "All segments should roundtrip identically"
    );
}
