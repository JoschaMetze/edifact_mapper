use std::path::Path;

use mig_bo4e::engine::MappingEngine;

/// Load from TOML dir, save to cache, reload, verify definitions match.
#[test]
fn test_roundtrip_cache_simple() {
    let toml_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/message");
    if !toml_dir.exists() {
        eprintln!("Skipping test_roundtrip_cache_simple: TOML dir not found");
        return;
    }

    let engine = MappingEngine::load(toml_dir).expect("load TOML");
    let defs_original = engine.definitions();

    let tmp = tempfile::tempdir().expect("tempdir");
    let cache_path = tmp.path().join("test.bincode");

    engine.save_cached(&cache_path).expect("save_cached");
    assert!(cache_path.exists(), "cache file should exist");

    let engine2 = MappingEngine::load_cached(&cache_path).expect("load_cached");
    let defs_cached = engine2.definitions();

    assert_eq!(
        defs_original.len(),
        defs_cached.len(),
        "definition count mismatch"
    );

    for (orig, cached) in defs_original.iter().zip(defs_cached.iter()) {
        assert_eq!(orig.meta.entity, cached.meta.entity, "entity mismatch");
        assert_eq!(
            orig.meta.source_group, cached.meta.source_group,
            "source_group mismatch"
        );
        assert_eq!(
            orig.meta.bo4e_type, cached.meta.bo4e_type,
            "bo4e_type mismatch"
        );
        assert_eq!(
            orig.fields.len(),
            cached.fields.len(),
            "fields count mismatch for entity {}",
            orig.meta.entity
        );
    }
}

/// Load TOML, apply PathResolver, save cache, reload, verify numeric paths survive.
#[test]
fn test_roundtrip_cache_with_path_resolution() {
    use mig_bo4e::path_resolver::PathResolver;

    let toml_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/message");
    let schema_dir = Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids");
    if !toml_dir.exists() || !schema_dir.exists() {
        eprintln!("Skipping test_roundtrip_cache_with_path_resolution: dirs not found");
        return;
    }

    let resolver = PathResolver::from_schema_dir(schema_dir);
    let engine = MappingEngine::load(toml_dir)
        .expect("load TOML")
        .with_path_resolver(resolver);

    let defs_resolved = engine.definitions();

    let tmp = tempfile::tempdir().expect("tempdir");
    let cache_path = tmp.path().join("resolved.bincode");

    engine.save_cached(&cache_path).expect("save_cached");

    let engine2 = MappingEngine::load_cached(&cache_path).expect("load_cached");
    let defs_cached = engine2.definitions();

    assert_eq!(defs_resolved.len(), defs_cached.len());

    // Verify that field keys survived the roundtrip (they should be numeric after resolution)
    for (resolved, cached) in defs_resolved.iter().zip(defs_cached.iter()) {
        let resolved_keys: Vec<&String> = resolved.fields.keys().collect();
        let cached_keys: Vec<&String> = cached.fields.keys().collect();
        assert_eq!(
            resolved_keys, cached_keys,
            "field keys mismatch for entity {}",
            resolved.meta.entity
        );
    }
}
