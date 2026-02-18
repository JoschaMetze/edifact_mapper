use automapper_generator::conditions::condition_types::*;
use automapper_generator::conditions::metadata::*;
use std::collections::{HashMap, HashSet};

#[test]
fn test_confidence_level_display() {
    assert_eq!(ConfidenceLevel::High.to_string(), "high");
    assert_eq!(ConfidenceLevel::Medium.to_string(), "medium");
    assert_eq!(ConfidenceLevel::Low.to_string(), "low");
}

#[test]
fn test_confidence_level_from_str() {
    assert_eq!(
        "high".parse::<ConfidenceLevel>().unwrap(),
        ConfidenceLevel::High
    );
    assert_eq!(
        "Medium".parse::<ConfidenceLevel>().unwrap(),
        ConfidenceLevel::Medium
    );
    assert_eq!(
        "LOW".parse::<ConfidenceLevel>().unwrap(),
        ConfidenceLevel::Low
    );
    assert!("unknown".parse::<ConfidenceLevel>().is_err());
}

#[test]
fn test_generated_condition_serialization_roundtrip() {
    let condition = GeneratedCondition {
        condition_number: 42,
        rust_code: Some("ctx.transaktion.marktlokationen.is_empty()".to_string()),
        is_external: false,
        confidence: ConfidenceLevel::High,
        reasoning: Some("Simple field check".to_string()),
        external_name: None,
        original_description: Some("Wenn Marktlokation vorhanden".to_string()),
        referencing_fields: Some(vec!["SG8/SEQ (Muss [42])".to_string()]),
    };

    let json = serde_json::to_string(&condition).unwrap();
    let deserialized: GeneratedCondition = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.condition_number, 42);
    assert_eq!(deserialized.confidence, ConfidenceLevel::High);
    assert!(!deserialized.is_external);
    assert!(deserialized.rust_code.is_some());
}

#[test]
fn test_claude_response_parsing() {
    let json = r#"{
        "conditions": [
            {
                "id": "1",
                "implementation": "ctx.transaktion.aufteilung.is_some()",
                "confidence": "high",
                "reasoning": "Simple option check",
                "is_external": false
            },
            {
                "id": "8",
                "implementation": null,
                "confidence": "high",
                "reasoning": "Requires external business context",
                "is_external": true,
                "external_name": "DataClearingRequired"
            }
        ]
    }"#;

    let response: ClaudeConditionResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.conditions.len(), 2);
    assert_eq!(response.conditions[0].id, "1");
    assert!(!response.conditions[0].is_external);
    assert!(response.conditions[1].is_external);
    assert_eq!(
        response.conditions[1].external_name.as_deref(),
        Some("DataClearingRequired")
    );
}

#[test]
fn test_compute_description_hash() {
    let hash1 = compute_description_hash("Wenn Aufteilung vorhanden");
    let hash2 = compute_description_hash("Wenn Aufteilung vorhanden");
    let hash3 = compute_description_hash("Wenn Aufteilung NICHT vorhanden");

    assert_eq!(hash1, hash2, "same input should produce same hash");
    assert_ne!(
        hash1, hash3,
        "different input should produce different hash"
    );
    assert_eq!(hash1.len(), 8, "hash should be 8 hex characters");
}

#[test]
fn test_metadata_serialization_roundtrip() {
    let mut conditions = HashMap::new();
    conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: Some("Simple check".to_string()),
            description_hash: "abcd1234".to_string(),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "UTILMD_AHB_Strom_2_1.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions,
    };

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    let deserialized: ConditionMetadataFile = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.format_version, "FV2510");
    assert_eq!(deserialized.conditions.len(), 1);
    assert_eq!(deserialized.conditions["1"].confidence, "high");
}

#[test]
fn test_decide_regeneration_all_new() {
    let conditions = vec![
        ("1".to_string(), "Wenn Aufteilung vorhanden".to_string()),
        ("2".to_string(), "Wenn Netznutzung".to_string()),
    ];

    let decision = decide_regeneration(&conditions, None, &HashSet::new(), false);

    assert_eq!(decision.to_regenerate.len(), 2);
    assert!(decision.to_preserve.is_empty());
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::New);
}

#[test]
fn test_decide_regeneration_force_all() {
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash("Wenn Aufteilung vorhanden"),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), "Wenn Aufteilung vorhanden".to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, true);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::Forced);
}

#[test]
fn test_decide_regeneration_preserve_high_confidence() {
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
    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert!(decision.to_regenerate.is_empty());
    assert_eq!(decision.to_preserve, vec!["1"]);
}

#[test]
fn test_decide_regeneration_stale_description() {
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash("OLD description"),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), "NEW description".to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::Stale);
}

#[test]
fn test_decide_regeneration_low_confidence_regenerated() {
    let desc = "Complex temporal condition";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "99".to_string(),
        ConditionMetadata {
            confidence: "low".to_string(),
            reasoning: Some("Too complex".to_string()),
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

    let conditions = vec![("99".to_string(), desc.to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("99".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(
        decision.to_regenerate[0].reason,
        RegenerationReason::LowConfidence
    );
}
