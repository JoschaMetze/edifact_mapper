use mig_bo4e::engine::MappingEngine;

#[test]
fn test_reverse_map_simple_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test_entity.toml"),
        r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "location_id"
"#,
    )
    .unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    let bo4e_value = serde_json::json!({
        "location_id": "DE0001234567890"
    });

    let result = engine.populate_field(&bo4e_value, "location_id", "loc.c517.d3225");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "DE0001234567890");
}

#[test]
fn test_reverse_map_nested_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test_entity.toml"),
        r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "address.city"
"#,
    )
    .unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    let bo4e_value = serde_json::json!({
        "address": {
            "city": "Berlin"
        }
    });

    let result = engine.populate_field(&bo4e_value, "address.city", "loc.c517.d3225");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "Berlin");
}

#[test]
fn test_reverse_map_missing_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test_entity.toml"),
        r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "location_id"
"#,
    )
    .unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    let bo4e_value = serde_json::json!({
        "other_field": "value"
    });

    let result = engine.populate_field(&bo4e_value, "location_id", "loc.c517.d3225");
    assert!(result.is_none(), "Missing field should return None");
}

#[test]
fn test_build_segment_from_bo4e() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test_entity.toml"),
        r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "location_id"
"#,
    )
    .unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    let bo4e_value = serde_json::json!({
        "location_id": "DE0001234567890"
    });

    let segment = engine.build_segment_from_bo4e(&bo4e_value, "LOC", "location_id");
    assert_eq!(segment.tag, "LOC");
    assert_eq!(segment.elements.len(), 1);
    assert_eq!(segment.elements[0][0], "DE0001234567890");
}

#[test]
fn test_build_group_from_bo4e() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test_entity.toml"),
        r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "location_id"
"nad.c082.d3039" = "partner_id"
"#,
    )
    .unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();
    let def = engine.definition_for_entity("TestEntity").unwrap().clone();

    let bo4e_value = serde_json::json!({
        "location_id": "DE0001234567890",
        "partner_id": "9876543210"
    });

    let group = engine.build_group_from_bo4e(&bo4e_value, &def);
    assert_eq!(group.group_id, "SG8");
    assert_eq!(group.repetitions.len(), 1);

    let instance = &group.repetitions[0];
    // Should have 2 segments (LOC and NAD)
    assert_eq!(instance.segments.len(), 2);
}
