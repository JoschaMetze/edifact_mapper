use mig_bo4e::engine::MappingEngine;
use std::path::Path;

const MESSAGE_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/message";

#[test]
fn test_load_message_level_mapping_files() {
    let msg_dir = Path::new(MESSAGE_DIR);
    if !msg_dir.exists() {
        eprintln!("message/ mappings dir not found, skipping");
        return;
    }

    let engine = MappingEngine::load(msg_dir).unwrap();
    assert_eq!(
        engine.definitions().len(),
        3,
        "Message directory should have exactly 3 mapping files (marktteilnehmer, nachricht, kontakt)"
    );
    assert!(engine.definition_for_entity("Marktteilnehmer").is_some());
    assert!(engine.definition_for_entity("Nachricht").is_some());
    assert!(engine.definition_for_entity("Kontakt").is_some());
}

#[test]
fn test_load_real_mapping_files() {
    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55001");
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dir not found, skipping");
        return;
    }

    // Transaction-level only
    let tx_engine = MappingEngine::load(tx_dir).unwrap();
    assert!(
        tx_engine.definitions().len() >= 14,
        "Expected at least 14 transaction mapping files for pid_55001, got {}",
        tx_engine.definitions().len()
    );

    // Transaction engine should NOT have message-level entities
    assert!(
        tx_engine.definition_for_entity("Marktteilnehmer").is_none(),
        "Transaction engine should not have Marktteilnehmer"
    );
    assert!(
        tx_engine.definition_for_entity("Nachricht").is_none(),
        "Transaction engine should not have Nachricht"
    );

    // Transaction engine should have transaction-level entities
    assert!(tx_engine.definition_for_entity("Marktlokation").is_some());
    assert!(tx_engine.definition_for_entity("Geschaeftspartner").is_some());
    assert!(tx_engine.definition_for_entity("Ansprechpartner").is_some());
    assert!(tx_engine.definition_for_entity("Produktpaket").is_some());
    assert!(tx_engine
        .definition_for_entity("ProduktpaketPriorisierung")
        .is_some());
    assert!(tx_engine.definition_for_entity("EnfgDaten").is_some());
    assert!(tx_engine
        .definition_for_entity("RuhendeMarktlokation")
        .is_some());

    // Combined engine should have all entities
    let combined = MappingEngine::load_merged(&[msg_dir, tx_dir]).unwrap();
    assert!(
        combined.definitions().len() >= 17,
        "Combined should have at least 17 mapping files, got {}",
        combined.definitions().len()
    );
    assert!(combined.definition_for_entity("Marktteilnehmer").is_some());
    assert!(combined.definition_for_entity("Nachricht").is_some());
    assert!(combined.definition_for_entity("Kontakt").is_some());
    assert!(combined.definition_for_entity("Marktlokation").is_some());
    assert!(combined.definition_for_entity("Geschaeftspartner").is_some());

    // SG10 zuordnung definitions
    assert!(
        combined
            .definitions()
            .iter()
            .any(|d| d.meta.source_group == "SG4.SG8:0.SG10"),
        "Should have SG10 definition for Produktpaket zuordnung"
    );
    assert!(
        combined
            .definitions()
            .iter()
            .any(|d| d.meta.source_group == "SG4.SG8:1.SG10"),
        "Should have SG10 definition for ProduktpaketPriorisierung zuordnung"
    );
}

#[test]
fn test_marktlokation_mapping_structure() {
    let mappings_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55001");
    if !mappings_dir.exists() {
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    let def = engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Marktlokation" && d.meta.source_group == "SG4.SG5")
        .expect("Marktlokation SG5 definition");

    assert_eq!(def.meta.bo4e_type, "Marktlokation");
    assert_eq!(def.meta.source_group, "SG4.SG5");
    assert!(def.fields.contains_key("loc.1.0"));
    assert!(def.fields.contains_key("loc.0"));
}

#[test]
fn test_geschaeftspartner_mapping_fields() {
    let mappings_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55001");
    if !mappings_dir.exists() {
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    let def = engine.definition_for_entity("Geschaeftspartner").unwrap();

    assert_eq!(def.meta.source_group, "SG4.SG12");
    assert!(def.fields.contains_key("nad.3.0"));
    assert!(def.fields.contains_key("nad.5"));
    assert!(def.fields.contains_key("nad.7"));
    assert!(def.fields.contains_key("nad.8"));
}

#[test]
fn test_load_pid_55002_mapping_files() {
    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55002");
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("55002 mappings/ dir not found, skipping");
        return;
    }

    // Transaction-level only
    let tx_engine = MappingEngine::load(tx_dir).unwrap();
    assert!(
        tx_engine.definitions().len() >= 16,
        "Expected at least 16 transaction mapping files for pid_55002, got {}",
        tx_engine.definitions().len()
    );

    // LOC entities (transaction-level)
    assert!(tx_engine.definition_for_entity("Marktlokation").is_some());
    assert!(tx_engine.definition_for_entity("Messlokation").is_some());
    assert!(tx_engine.definition_for_entity("Netzlokation").is_some());
    assert!(tx_engine
        .definition_for_entity("SteuerbareRessource")
        .is_some());
    assert!(tx_engine
        .definition_for_entity("TechnischeRessource")
        .is_some());
    assert!(tx_engine
        .definition_for_entity("RuhendeMarktlokation")
        .is_some());
    assert!(tx_engine.definition_for_entity("Prozessdaten").is_some());

    // Message-level entities should NOT be in transaction dir
    assert!(
        tx_engine.definition_for_entity("Marktteilnehmer").is_none(),
        "Transaction engine should not have Marktteilnehmer"
    );
    assert!(
        tx_engine.definition_for_entity("Nachricht").is_none(),
        "Transaction engine should not have Nachricht"
    );

    // Combined engine should have everything
    let combined = MappingEngine::load_merged(&[msg_dir, tx_dir]).unwrap();
    assert!(combined.definition_for_entity("Marktteilnehmer").is_some());
    assert!(combined.definition_for_entity("Nachricht").is_some());
    assert!(combined.definition_for_entity("Kontakt").is_some());

    // SG8 info groups with discriminators
    let info_groups: Vec<&str> = tx_engine
        .definitions()
        .iter()
        .filter(|d| d.meta.source_group == "SG4.SG8" && d.meta.discriminator.is_some())
        .map(|d| d.meta.entity.as_str())
        .collect();
    assert!(
        info_groups.contains(&"Marktlokation"),
        "Should have SG8 info for Marktlokation"
    );
    assert!(
        info_groups.contains(&"Netzlokation"),
        "Should have SG8 info for Netzlokation"
    );
    assert!(
        info_groups.contains(&"SteuerbareRessource"),
        "Should have SG8 info for SteuerbareRessource"
    );
    assert!(
        info_groups.contains(&"Messlokation"),
        "Should have SG8 info for Messlokation"
    );

    // SG10 zuordnung groups
    for i in 0..4 {
        let sg = format!("SG4.SG8:{i}.SG10");
        assert!(
            tx_engine
                .definitions()
                .iter()
                .any(|d| d.meta.source_group == sg),
            "Should have SG10 zuordnung at {sg}"
        );
    }

    // Messlokation zuordnung should use ZF0 (gMSB), not Z91
    let melo_zuordnung = tx_engine
        .definitions()
        .iter()
        .find(|d| d.meta.entity == "Messlokation" && d.meta.source_group == "SG4.SG8:3.SG10")
        .expect("Messlokation SG10 definition");
    let cf = melo_zuordnung.companion_fields.as_ref().unwrap();
    assert!(
        cf.contains_key("cav.0.1"),
        "Messlokation zuordnung should map CAV without qualifier (ZF0 only)"
    );

    // Prozessdaten should NOT have DTM+92 (only DTM+93 in 55002)
    let prozess = tx_engine.definition_for_entity("Prozessdaten").unwrap();
    assert!(
        !prozess.fields.contains_key("dtm[92].0.1"),
        "55002 Prozessdaten should not have DTM+92"
    );
    assert!(
        prozess.fields.contains_key("dtm[93].0.1"),
        "55002 Prozessdaten should have DTM+93"
    );
}
