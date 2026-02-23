use mig_bo4e::definition::{FieldMapping, MappingDefinition};
use mig_bo4e::engine::MappingEngine;

#[test]
fn test_parse_mapping_toml() {
    let toml_str = r#"
[meta]
entity = "Marktlokation"
bo4e_type = "bo4e::Marktlokation"
companion_type = "MarktlokationEdifact"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z01'"

[fields]
"loc.c517.d3225" = "marktlokationsId"
"loc.d3227" = { target = "lokationstyp", transform = "loc_qualifier_to_type" }

[fields."sg9_characteristics"]
"cci.c240.d7037" = "characteristic_code"

[companion_fields]
"dtm.c507.d2380" = { target = "gueltigAb", when = "dtm.c507.d2005 == '157'" }
"#;

    let def: MappingDefinition = toml::from_str(toml_str).unwrap();

    assert_eq!(def.meta.entity, "Marktlokation");
    assert_eq!(def.meta.source_group, "SG8");
    assert_eq!(
        def.meta.discriminator.as_deref(),
        Some("seq.d1245.qualifier == 'Z01'")
    );
    assert!(!def.fields.is_empty());
    assert!(def.fields.contains_key("loc.c517.d3225"));

    // Check simple field
    match &def.fields["loc.c517.d3225"] {
        FieldMapping::Simple(target) => assert_eq!(target, "marktlokationsId"),
        other => panic!("Expected Simple, got {:?}", other),
    }

    // Check structured field
    match &def.fields["loc.d3227"] {
        FieldMapping::Structured(s) => {
            assert_eq!(s.target, "lokationstyp");
            assert_eq!(s.transform.as_deref(), Some("loc_qualifier_to_type"));
        }
        other => panic!("Expected Structured, got {:?}", other),
    }

    // Check nested group field
    match &def.fields["sg9_characteristics"] {
        FieldMapping::Nested(map) => {
            assert!(map.contains_key("cci.c240.d7037"));
        }
        other => panic!("Expected Nested, got {:?}", other),
    }

    assert!(def.companion_fields.is_some());
    let companion = def.companion_fields.unwrap();
    assert!(companion.contains_key("dtm.c507.d2380"));
}

#[test]
fn test_parse_minimal_mapping() {
    let toml_str = r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG1"

[fields]
"seg.d1234" = "some_field"
"#;

    let def: MappingDefinition = toml::from_str(toml_str).unwrap();
    assert_eq!(def.meta.entity, "TestEntity");
    assert!(def.meta.companion_type.is_none());
    assert!(def.meta.discriminator.is_none());
    assert!(def.companion_fields.is_none());
    assert!(def.complex_handlers.is_none());
}

#[test]
fn test_parse_with_complex_handlers() {
    let toml_str = r#"
[meta]
entity = "ComplexEntity"
bo4e_type = "ComplexEntity"
source_group = "SG4"

[fields]
"seg.d1234" = "field_a"

[[complex_handlers]]
name = "resolve_cross_refs"
description = "Resolves cross-references between entities"

[[complex_handlers]]
name = "compute_derived"
"#;

    let def: MappingDefinition = toml::from_str(toml_str).unwrap();
    let handlers = def.complex_handlers.unwrap();
    assert_eq!(handlers.len(), 2);
    assert_eq!(handlers[0].name, "resolve_cross_refs");
    assert!(handlers[0].description.is_some());
    assert_eq!(handlers[1].name, "compute_derived");
    assert!(handlers[1].description.is_none());
}

#[test]
fn test_load_mappings_from_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("marktlokation.toml"),
        r#"
[meta]
entity = "Marktlokation"
bo4e_type = "bo4e::Marktlokation"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z01'"

[fields]
"loc.c517.d3225" = "marktlokationsId"
"#,
    )
    .unwrap();

    std::fs::write(
        dir.path().join("messlokation.toml"),
        r#"
[meta]
entity = "Messlokation"
bo4e_type = "bo4e::Messlokation"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z02'"

[fields]
"loc.c517.d3225" = "messlokationsId"
"#,
    )
    .unwrap();

    // Non-toml files should be ignored
    std::fs::write(dir.path().join("readme.txt"), "not a toml file").unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    assert_eq!(engine.definitions().len(), 2);
    assert!(engine.definition_for_entity("Marktlokation").is_some());
    assert!(engine.definition_for_entity("Messlokation").is_some());
    assert!(engine.definition_for_entity("Nonexistent").is_none());
}

#[test]
fn test_load_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let engine = MappingEngine::load(dir.path()).unwrap();
    assert_eq!(engine.definitions().len(), 0);
}

#[test]
fn test_load_invalid_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bad.toml"), "this is not valid toml {{{").unwrap();

    let result = MappingEngine::load(dir.path());
    assert!(result.is_err());
}
