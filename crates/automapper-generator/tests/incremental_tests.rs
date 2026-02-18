use automapper_generator::conditions::metadata::*;
use std::collections::{HashMap, HashSet};
use tempfile::TempDir;

/// Tests the full incremental flow:
/// 1. First generation creates metadata
/// 2. Second generation with same descriptions preserves high-confidence conditions
/// 3. Changed description triggers regeneration
#[test]
fn test_incremental_generation_flow() {
    let tmp = TempDir::new().unwrap();
    let metadata_path = tmp.path().join("conditions.json");

    // === Step 1: First generation (all new) ===
    let conditions_v1 = vec![
        ("1".to_string(), "Wenn Aufteilung vorhanden".to_string()),
        ("2".to_string(), "Wenn Marktlokation vorhanden".to_string()),
        ("3".to_string(), "Wenn Zeitraum gueltig".to_string()),
    ];

    let decision_v1 = decide_regeneration(&conditions_v1, None, &HashSet::new(), false);

    assert_eq!(
        decision_v1.to_regenerate.len(),
        3,
        "first run should regenerate all"
    );
    assert!(decision_v1.to_preserve.is_empty());

    // Simulate generation results
    let mut meta_conditions = HashMap::new();
    for (id, desc) in &conditions_v1 {
        meta_conditions.insert(
            id.clone(),
            ConditionMetadata {
                confidence: "high".to_string(),
                reasoning: Some("Generated".to_string()),
                description_hash: compute_description_hash(desc),
                is_external: id == "1",
            },
        );
    }

    let metadata_v1 = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    save_metadata(&metadata_path, &metadata_v1).unwrap();

    // === Step 2: Second generation with same descriptions ===
    let loaded = load_metadata(&metadata_path).unwrap().unwrap();
    assert_eq!(loaded.conditions.len(), 3);

    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());
    existing_ids.insert("2".to_string());
    existing_ids.insert("3".to_string());

    let decision_v2 = decide_regeneration(&conditions_v1, Some(&loaded), &existing_ids, false);

    assert!(
        decision_v2.to_regenerate.is_empty(),
        "same descriptions should preserve all: {:?}",
        decision_v2.to_regenerate
    );
    assert_eq!(decision_v2.to_preserve.len(), 3);

    // === Step 3: Changed description triggers regeneration ===
    let conditions_v3 = vec![
        ("1".to_string(), "Wenn Aufteilung vorhanden".to_string()), // unchanged
        (
            "2".to_string(),
            "Wenn Marktlokation vorhanden UND aktiv".to_string(),
        ), // CHANGED
        ("3".to_string(), "Wenn Zeitraum gueltig".to_string()),     // unchanged
    ];

    let decision_v3 = decide_regeneration(&conditions_v3, Some(&loaded), &existing_ids, false);

    assert_eq!(
        decision_v3.to_regenerate.len(),
        1,
        "only changed condition should regenerate"
    );
    assert_eq!(decision_v3.to_regenerate[0].condition_id, "2");
    assert_eq!(
        decision_v3.to_regenerate[0].reason,
        RegenerationReason::Stale
    );
    assert_eq!(decision_v3.to_preserve.len(), 2);
}

#[test]
fn test_metadata_persistence_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.conditions.json");

    let mut conditions = HashMap::new();
    conditions.insert(
        "42".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: Some("Simple check".to_string()),
            description_hash: "abcd1234".to_string(),
            is_external: true,
        },
    );
    conditions.insert(
        "99".to_string(),
        ConditionMetadata {
            confidence: "low".to_string(),
            reasoning: Some("Complex temporal logic".to_string()),
            description_hash: "efgh5678".to_string(),
            is_external: false,
        },
    );

    let original = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "UTILMD_AHB_Strom_2_1.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions,
    };

    save_metadata(&path, &original).unwrap();

    let loaded = load_metadata(&path).unwrap().unwrap();

    assert_eq!(loaded.format_version, "FV2510");
    assert_eq!(loaded.conditions.len(), 2);
    assert_eq!(loaded.conditions["42"].confidence, "high");
    assert!(loaded.conditions["42"].is_external);
    assert_eq!(loaded.conditions["99"].confidence, "low");
    assert!(!loaded.conditions["99"].is_external);
}

#[test]
fn test_load_nonexistent_metadata_returns_none() {
    let result = load_metadata(std::path::Path::new("/nonexistent/path.json")).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_new_condition_added_to_ahb() {
    let desc1 = "Wenn Aufteilung vorhanden";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash(desc1),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    // Condition 2 is new (not in metadata)
    let conditions = vec![
        ("1".to_string(), desc1.to_string()),
        ("2".to_string(), "Neubedingung".to_string()),
    ];

    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(decision.to_regenerate[0].condition_id, "2");
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::New);
    assert_eq!(decision.to_preserve.len(), 1);
    assert_eq!(decision.to_preserve[0], "1");
}

#[test]
fn test_missing_implementation_triggers_regeneration() {
    let desc = "Wenn Aufteilung vorhanden";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash(desc),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), desc.to_string())];

    // Condition 1 is in metadata but NOT in existing implementation file
    let existing_ids = HashSet::new(); // empty = no existing implementations

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(
        decision.to_regenerate[0].reason,
        RegenerationReason::MissingImplementation
    );
}
