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
        engine.definitions().len() >= 15,
        "Expected at least 15 mapping files, got {}",
        engine.definitions().len()
    );
    assert!(engine.definition_for_entity("Marktlokation").is_some());
    assert!(engine.definition_for_entity("Marktteilnehmer").is_some());
    assert!(engine.definition_for_entity("Geschaeftspartner").is_some());
    assert!(engine.definition_for_entity("Nachricht").is_some());
    assert!(engine.definition_for_entity("Zaehlpunkt").is_some());
    assert!(engine.definition_for_entity("Messstellenbetrieb").is_some());
    assert!(engine.definition_for_entity("Geraet").is_some());
    assert!(engine
        .definition_for_entity("Netznutzungsabrechnung")
        .is_some());
    assert!(engine.definition_for_entity("Ansprechpartner").is_some());
    assert!(engine.definition_for_entity("MerkmalZaehlpunkt").is_some());
    assert!(engine
        .definition_for_entity("MerkmalMessstellenbetrieb")
        .is_some());
    assert!(engine.definition_for_entity("MerkmalGeraet").is_some());
    assert!(engine.definition_for_entity("MerkmalNetznutzung").is_some());
}

#[test]
fn test_marktlokation_mapping_structure() {
    let mappings_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55001");
    if !mappings_dir.exists() {
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    let def = engine.definition_for_entity("Marktlokation").unwrap();

    assert_eq!(def.meta.bo4e_type, "Marktlokation");
    assert_eq!(def.meta.source_group, "SG4.SG5");
    assert!(def.fields.contains_key("loc.c517.d3225"));
    assert!(def.fields.contains_key("loc.d3227"));
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
