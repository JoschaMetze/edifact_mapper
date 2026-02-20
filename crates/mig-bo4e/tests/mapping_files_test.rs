use mig_bo4e::engine::MappingEngine;
use std::path::Path;

#[test]
fn test_load_real_mapping_files() {
    let mappings_dir = Path::new("../../mappings");
    if !mappings_dir.exists() {
        eprintln!("mappings/ dir not found, skipping");
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    assert!(
        engine.definitions().len() >= 3,
        "Expected at least 3 mapping files, got {}",
        engine.definitions().len()
    );
    assert!(engine.definition_for_entity("Marktlokation").is_some());
    assert!(engine.definition_for_entity("Messlokation").is_some());
    assert!(engine.definition_for_entity("Geschaeftspartner").is_some());
}

#[test]
fn test_marktlokation_mapping_structure() {
    let mappings_dir = Path::new("../../mappings");
    if !mappings_dir.exists() {
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    let def = engine.definition_for_entity("Marktlokation").unwrap();

    assert_eq!(def.meta.bo4e_type, "bo4e::Marktlokation");
    assert_eq!(
        def.meta.companion_type.as_deref(),
        Some("MarktlokationEdifact")
    );
    assert_eq!(def.meta.source_group, "SG8");
    assert!(def.meta.discriminator.is_some());
    assert!(def.fields.contains_key("loc.c517.d3225"));
    assert!(def.companion_fields.is_some());
}

#[test]
fn test_geschaeftspartner_mapping_fields() {
    let mappings_dir = Path::new("../../mappings");
    if !mappings_dir.exists() {
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    let def = engine.definition_for_entity("Geschaeftspartner").unwrap();

    assert_eq!(def.meta.source_group, "SG2");
    assert!(def.fields.contains_key("nad.d3035"));
    assert!(def.fields.contains_key("nad.c082.d3039"));
    assert!(def.fields.contains_key("nad.c058.d3124"));
    assert!(def.fields.contains_key("nad.d3164"));
    assert!(def.fields.contains_key("nad.d3251"));
}
