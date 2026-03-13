use std::collections::HashMap;
use std::path::Path;

use mig_bo4e::engine::{MappingEngine, VariantCache};

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

/// load_cached_or_toml uses cache when present.
#[test]
fn test_load_cached_or_toml_uses_cache() {
    let toml_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/message");
    if !toml_dir.exists() {
        eprintln!("Skipping: TOML dir not found");
        return;
    }

    let engine = MappingEngine::load(toml_dir).expect("load TOML");
    let tmp = tempfile::tempdir().expect("tempdir");
    let cache_path = tmp.path().join("msg.bin");
    engine.save_cached(&cache_path).expect("save_cached");

    let loaded = MappingEngine::load_cached_or_toml(&cache_path, toml_dir).expect("load");
    assert_eq!(loaded.definitions().len(), engine.definitions().len());
}

/// load_cached_or_toml falls back to TOML when cache missing.
#[test]
fn test_load_cached_or_toml_fallback() {
    let toml_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/message");
    if !toml_dir.exists() {
        eprintln!("Skipping: TOML dir not found");
        return;
    }

    let nonexistent = Path::new("/tmp/nonexistent_cache_test_12345.bin");
    let loaded = MappingEngine::load_cached_or_toml(nonexistent, toml_dir).expect("load");
    assert!(!loaded.definitions().is_empty());
}

/// VariantCache roundtrip: build from engines, save, reload, verify contents.
#[test]
fn test_variant_cache_roundtrip() {
    let toml_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/message");
    if !toml_dir.exists() {
        eprintln!("Skipping test_variant_cache_roundtrip: TOML dir not found");
        return;
    }

    let msg_engine = MappingEngine::load(toml_dir).expect("load message TOML");
    let msg_defs = msg_engine.definitions().to_vec();

    let mut tx_map = HashMap::new();
    let mut combined_map = HashMap::new();

    // Use message defs as stand-in for tx/combined to test the roundtrip
    tx_map.insert("pid_55001".to_string(), msg_defs.clone());
    combined_map.insert("pid_55001".to_string(), msg_defs.clone());

    let cache = VariantCache {
        message_defs: msg_defs.clone(),
        transaction_defs: tx_map,
        combined_defs: combined_map,
        code_lookups: HashMap::new(),
        mig_schema: None,
        segment_structure: None,
        pid_segment_numbers: HashMap::new(),
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let cache_path = tmp.path().join("UTILMD_Strom.json");

    cache.save(&cache_path).expect("save variant cache");
    assert!(cache_path.exists(), "variant cache file should exist");

    let loaded = VariantCache::load(&cache_path).expect("load variant cache");

    assert_eq!(
        loaded.message_defs.len(),
        msg_defs.len(),
        "message defs count mismatch"
    );
    assert!(
        loaded.transaction_defs.contains_key("pid_55001"),
        "should have pid_55001 tx defs"
    );
    assert!(
        loaded.combined_defs.contains_key("pid_55001"),
        "should have pid_55001 combined defs"
    );
    assert_eq!(
        loaded.transaction_defs["pid_55001"].len(),
        msg_defs.len(),
        "tx defs count mismatch"
    );
}

/// VariantCache from compile_all: verify the consolidated file is loadable.
#[test]
fn test_variant_cache_from_compile() {
    let cache_path = Path::new("../../cache/mappings/FV2504/UTILMD_Strom.json");
    if !cache_path.exists() {
        eprintln!(
            "Skipping test_variant_cache_from_compile: {} not found (run compile-mappings first)",
            cache_path.display()
        );
        return;
    }

    let vc = VariantCache::load(cache_path).expect("load variant cache");

    assert!(
        !vc.message_defs.is_empty(),
        "variant cache should have message defs"
    );
    assert!(
        !vc.transaction_defs.is_empty(),
        "variant cache should have transaction defs"
    );
    assert!(
        !vc.combined_defs.is_empty(),
        "variant cache should have combined defs"
    );

    // Verify a known PID exists
    assert!(
        vc.combined_defs.contains_key("pid_55001"),
        "should have pid_55001 in combined defs"
    );

    eprintln!(
        "VariantCache: {} message defs, {} tx PIDs, {} combined PIDs",
        vc.message_defs.len(),
        vc.transaction_defs.len(),
        vc.combined_defs.len()
    );
}
