use mig_assembly::assembler::{
    AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree,
};
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;

fn test_schema() -> serde_json::Value {
    serde_json::json!({
        "beschreibung": "Test PID",
        "fields": {
            "sg4": {
                "source_group": "SG4",
                "segments": [],
                "children": {
                    "sg5_z16": {
                        "source_group": "SG5",
                        "segments": [
                            {
                                "id": "LOC",
                                "name": "Marktlokation",
                                "elements": [
                                    {
                                        "id": "3227",
                                        "index": 0,
                                        "name": "Lokation, Qualifier",
                                        "type": "code"
                                    },
                                    {
                                        "composite": "C517",
                                        "index": 1,
                                        "name": "Lokationsidentifikation",
                                        "components": [
                                            {
                                                "id": "3225",
                                                "sub_index": 0,
                                                "name": "MaLo-ID",
                                                "type": "data"
                                            },
                                            {
                                                "id": "1131",
                                                "sub_index": 1,
                                                "name": "Codeliste, Code",
                                                "type": "data"
                                            },
                                            {
                                                "id": "3055",
                                                "sub_index": 2,
                                                "name": "Verantwortliche Stelle",
                                                "type": "code"
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    },
                    "sg8_z98": {
                        "source_group": "SG8",
                        "segments": [
                            {
                                "id": "SEQ",
                                "name": "Reihenfolge",
                                "elements": [
                                    {
                                        "id": "1229",
                                        "index": 0,
                                        "name": "Handlung, Code",
                                        "type": "code"
                                    },
                                    {
                                        "composite": "C286",
                                        "index": 1,
                                        "name": "Information über eine Folge",
                                        "components": [
                                            {
                                                "id": "1050",
                                                "sub_index": 0,
                                                "name": "Referenz auf Zeitraum-ID",
                                                "type": "data"
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }
    })
}

fn make_tree() -> AssembledTree {
    AssembledTree {
        segments: vec![],
        groups: vec![AssembledGroup {
            group_id: "SG4".to_string(),
            repetitions: vec![AssembledGroupInstance {
                segments: vec![],
                child_groups: vec![
                    AssembledGroup {
                        group_id: "SG5".to_string(),
                        repetitions: vec![AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "LOC".to_string(),
                                elements: vec![
                                    vec!["Z16".to_string()],
                                    vec![
                                        "DE0001234567890".to_string(),
                                        "".to_string(),
                                        "9".to_string(),
                                    ],
                                ],
                            }],
                            child_groups: vec![],
                        }],
                    },
                    AssembledGroup {
                        group_id: "SG8".to_string(),
                        repetitions: vec![AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "SEQ".to_string(),
                                elements: vec![vec!["Z98".to_string()], vec!["REF123".to_string()]],
                            }],
                            child_groups: vec![],
                        }],
                    },
                ],
            }],
        }],
        post_group_start: 0,
    }
}

/// Test that TOML with EDIFACT ID paths works when PathResolver is applied.
#[test]
fn named_paths_forward_mapping() {
    let dir = tempfile::tempdir().unwrap();
    // TOML uses named EDIFACT ID paths
    std::fs::write(
        dir.path().join("marktlokation.toml"),
        r#"
[meta]
entity = "Marktlokation"
bo4e_type = "Marktlokation"
source_group = "SG4.SG5"
discriminator = "LOC.d3227=Z16"

[fields]
"loc.d3227" = { target = "qualifier", default = "Z16" }
"loc.c517.d3225" = "marktlokations_id"
"loc.c517.d3055" = "codestelle"
"#,
    )
    .unwrap();

    let schema = test_schema();
    let resolver = PathResolver::from_schema(&schema);
    let engine = MappingEngine::load(dir.path())
        .unwrap()
        .with_path_resolver(resolver);

    let tree = make_tree();
    let def = engine.definition_for_entity("Marktlokation").unwrap();

    // Verify discriminator was resolved
    assert_eq!(
        def.meta.discriminator.as_deref(),
        Some("LOC.0=Z16"),
        "Named discriminator should resolve to numeric"
    );

    // Forward map
    let result = engine.map_forward(&tree, def, 0);
    assert_eq!(result["marktlokations_id"], "DE0001234567890");
    assert_eq!(result["codestelle"], "9");
}

/// Test that TOML with EDIFACT ID paths works for reverse mapping.
#[test]
fn named_paths_reverse_mapping() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("seq_mapping.toml"),
        r#"
[meta]
entity = "SeqTest"
bo4e_type = "SeqTest"
source_group = "SG4.SG8"

[fields]
"seq.d1229" = { target = "handlung", default = "Z98" }
"seq.c286.d1050" = "zeitraum_ref"
"#,
    )
    .unwrap();

    let schema = test_schema();
    let resolver = PathResolver::from_schema(&schema);
    let engine = MappingEngine::load(dir.path())
        .unwrap()
        .with_path_resolver(resolver);

    let tree = make_tree();
    let def = engine.definition_for_entity("SeqTest").unwrap();

    // Verify paths were resolved to numeric
    assert!(
        def.fields.contains_key("seq.0"),
        "seq.d1229 should resolve to seq.0"
    );
    assert!(
        def.fields.contains_key("seq.1.0"),
        "seq.c286.d1050 should resolve to seq.1.0"
    );

    // Forward map
    let result = engine.map_forward(&tree, def, 0);
    assert_eq!(result["handlung"], "Z98");
    assert_eq!(result["zeitraum_ref"], "REF123");

    // Reverse map
    let reversed = engine.map_reverse(&result, def);
    assert_eq!(reversed.segments.len(), 1);
    assert_eq!(reversed.segments[0].tag, "SEQ");
    assert_eq!(reversed.segments[0].elements[0][0], "Z98");
    assert_eq!(reversed.segments[0].elements[1][0], "REF123");
}

/// Test that mixed numeric and named paths work in the same TOML file.
#[test]
fn mixed_path_styles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("mixed.toml"),
        r#"
[meta]
entity = "MixedTest"
bo4e_type = "MixedTest"
source_group = "SG4.SG5"
discriminator = "LOC.0=Z16"

[fields]
"loc.0" = { target = "qualifier", default = "Z16" }
"loc.c517.d3225" = "id_named"
"loc.1.2" = "codestelle_numeric"
"#,
    )
    .unwrap();

    let schema = test_schema();
    let resolver = PathResolver::from_schema(&schema);
    let engine = MappingEngine::load(dir.path())
        .unwrap()
        .with_path_resolver(resolver);

    let tree = make_tree();
    let def = engine.definition_for_entity("MixedTest").unwrap();

    // Named path resolved, numeric path unchanged
    assert!(def.fields.contains_key("loc.1.0"), "Named should resolve");
    assert!(def.fields.contains_key("loc.1.2"), "Numeric stays as-is");
    assert!(def.fields.contains_key("loc.0"), "Numeric stays as-is");

    let result = engine.map_forward(&tree, def, 0);
    assert_eq!(result["id_named"], "DE0001234567890");
    assert_eq!(result["codestelle_numeric"], "9");
}

/// Test companion_fields with named paths.
#[test]
fn named_paths_companion_fields() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("companion.toml"),
        r#"
[meta]
entity = "CompanionTest"
bo4e_type = "CompanionTest"
companion_type = "CompanionTestEdifact"
source_group = "SG4.SG5"
discriminator = "LOC.d3227=Z16"

[fields]
"loc.c517.d3225" = "marktlokations_id"

[companion_fields]
"loc.d3227" = { target = "qualifier", default = "Z16" }
"loc.c517.d3055" = "codestelle"
"#,
    )
    .unwrap();

    let schema = test_schema();
    let resolver = PathResolver::from_schema(&schema);
    let engine = MappingEngine::load(dir.path())
        .unwrap()
        .with_path_resolver(resolver);

    let def = engine.definition_for_entity("CompanionTest").unwrap();

    // Verify companion_fields resolved
    let cf = def.companion_fields.as_ref().unwrap();
    assert!(cf.contains_key("loc.0"), "companion loc.d3227 → loc.0");
    assert!(
        cf.contains_key("loc.1.2"),
        "companion loc.c517.d3055 → loc.1.2"
    );
}
