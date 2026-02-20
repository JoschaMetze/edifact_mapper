use mig_assembly::assembler::{
    AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree,
};
use mig_bo4e::engine::MappingEngine;

fn make_test_tree() -> AssembledTree {
    AssembledTree {
        segments: vec![],
        groups: vec![AssembledGroup {
            group_id: "SG8".to_string(),
            repetitions: vec![AssembledGroupInstance {
                segments: vec![AssembledSegment {
                    tag: "LOC".to_string(),
                    elements: vec![
                        vec!["Z16".to_string()],             // qualifier (element 0)
                        vec!["DE0001234567890".to_string()], // C517.3225 (element 1)
                    ],
                }],
                child_groups: vec![],
            }],
        }],
        post_group_start: 0,
    }
}

#[test]
fn test_forward_map_simple_field() {
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
    let tree = make_test_tree();

    let result = engine.extract_field(&tree, "SG8", "loc.c517.d3225", 0);
    assert!(result.is_some(), "Should extract location ID from tree");
    assert_eq!(result.unwrap(), "DE0001234567890");
}

#[test]
fn test_forward_map_qualifier_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test_entity.toml"),
        r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.d3227" = "qualifier"
"#,
    )
    .unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();
    let tree = make_test_tree();

    // Single-component path ("loc.d3227") maps to element[0][0]
    let result = engine.extract_field(&tree, "SG8", "loc.d3227", 0);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "Z16");
}

#[test]
fn test_forward_map_missing_group() {
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
    let tree = make_test_tree();

    let result = engine.extract_field(&tree, "SG99", "loc.c517.d3225", 0);
    assert!(result.is_none(), "Missing group should return None");
}

#[test]
fn test_forward_map_out_of_range_repetition() {
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
    let tree = make_test_tree();

    let result = engine.extract_field(&tree, "SG8", "loc.c517.d3225", 5);
    assert!(
        result.is_none(),
        "Out-of-range repetition should return None"
    );
}

#[test]
fn test_extract_from_instance_directly() {
    let instance = AssembledGroupInstance {
        segments: vec![AssembledSegment {
            tag: "NAD".to_string(),
            elements: vec![vec!["MS".to_string()], vec!["9876543210".to_string()]],
        }],
        child_groups: vec![],
    };

    let result = MappingEngine::extract_from_instance(&instance, "nad.c082.d3039");
    assert_eq!(result, Some("9876543210".to_string()));
}
