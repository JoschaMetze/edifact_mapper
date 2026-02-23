use mig_bo4e::engine::MappingEngine;
use std::path::Path;

#[test]
fn test_load_real_mapping_files() {
    let mappings_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55001");
    if !mappings_dir.exists() {
        eprintln!("mappings/ dir not found, skipping");
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    assert!(
        engine.definitions().len() >= 17,
        "Expected at least 17 mapping files, got {}",
        engine.definitions().len()
    );
    assert!(engine.definition_for_entity("Marktlokation").is_some());
    assert!(engine.definition_for_entity("Marktteilnehmer").is_some());
    assert!(engine.definition_for_entity("Geschaeftspartner").is_some());
    assert!(engine.definition_for_entity("Nachricht").is_some());
    assert!(engine.definition_for_entity("Produktpaket").is_some());
    assert!(engine
        .definition_for_entity("ProduktpaketPriorisierung")
        .is_some());
    assert!(engine.definition_for_entity("EnfgDaten").is_some());
    assert!(engine.definition_for_entity("Ansprechpartner").is_some());
    assert!(engine
        .definition_for_entity("RuhendeMarktlokation")
        .is_some());
    assert!(engine.definition_for_entity("Kontakt").is_some());
    // Merkmal/zuordnung data is now merged into parent entities (Produktpaket, etc.)
    // via companion_fields — verify SG10 definitions exist by source_group
    assert!(
        engine
            .definitions()
            .iter()
            .any(|d| d.meta.source_group == "SG4.SG8:0.SG10"),
        "Should have SG10 definition for Produktpaket zuordnung"
    );
    assert!(
        engine
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
