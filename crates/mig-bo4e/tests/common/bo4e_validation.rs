//! Shared BO4E schema validation helpers for roundtrip tests.
//!
//! Validates that forward-mapped JSON field names match real fields on
//! bo4e-german types. Emits warnings for unknown fields.

use mig_bo4e::engine::MappingEngine;
use std::collections::{HashMap, HashSet};

/// Keys injected by the mapping engine that aren't part of bo4e-german types.
const ENGINE_METADATA_KEYS: &[&str] = &["boTyp", "versionStruktur"];

/// Validate BO4E JSON keys by deserializing into a concrete type and re-serializing.
/// Returns the list of keys present in `json` but absent after the serde roundtrip.
fn validate_bo4e_keys<T>(json: &serde_json::Value, exclude_keys: &[&str]) -> Vec<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let original_keys: HashSet<&str> = match json.as_object() {
        Some(obj) => obj.keys().map(|k| k.as_str()).collect(),
        None => return vec![],
    };

    let typed: T = match serde_json::from_value(json.clone()) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let roundtripped = serde_json::to_value(&typed).unwrap();
    let roundtripped_keys: HashSet<&str> = match roundtripped.as_object() {
        Some(obj) => obj.keys().map(|k| k.as_str()).collect(),
        None => return vec![],
    };

    let mut exclude: HashSet<&str> = exclude_keys.iter().copied().collect();
    for k in ENGINE_METADATA_KEYS {
        exclude.insert(k);
    }

    let mut unknown: Vec<String> = original_keys
        .difference(&roundtripped_keys)
        .filter(|k| !exclude.contains(**k))
        .map(|k| k.to_string())
        .collect();
    unknown.sort();
    unknown
}

/// Dispatch to concrete bo4e-german types based on bo4e_type string.
fn validate_entity(
    bo4e_type: &str,
    companion_type: Option<&str>,
    json: &serde_json::Value,
) -> Vec<String> {
    let companion_key = companion_type.map(|ct| {
        let mut chars = ct.chars();
        match chars.next() {
            Some(c) => c.to_lowercase().to_string() + chars.as_str(),
            None => String::new(),
        }
    });
    let exclude: Vec<&str> = companion_key.iter().map(|s| s.as_str()).collect();

    match bo4e_type {
        "Marktlokation" => validate_bo4e_keys::<bo4e_german::Marktlokation>(json, &exclude),
        "Messlokation" => validate_bo4e_keys::<bo4e_german::Messlokation>(json, &exclude),
        "Netzlokation" => validate_bo4e_keys::<bo4e_german::Netzlokation>(json, &exclude),
        "Geschaeftspartner" => validate_bo4e_keys::<bo4e_german::Geschaeftspartner>(json, &exclude),
        "Marktteilnehmer" => validate_bo4e_keys::<bo4e_german::Marktteilnehmer>(json, &exclude),
        "SteuerbareRessource" => {
            validate_bo4e_keys::<bo4e_german::SteuerbareRessource>(json, &exclude)
        }
        "TechnischeRessource" => {
            validate_bo4e_keys::<bo4e_german::TechnischeRessource>(json, &exclude)
        }
        "Bilanzierung" => validate_bo4e_keys::<bo4e_german::Bilanzierung>(json, &exclude),
        "Lokationszuordnung" => {
            validate_bo4e_keys::<bo4e_german::Lokationszuordnung>(json, &exclude)
        }
        "Zaehler" => validate_bo4e_keys::<bo4e_german::Zaehler>(json, &exclude),
        "Vertrag" => validate_bo4e_keys::<bo4e_german::Vertrag>(json, &exclude),
        "Adresse" => validate_bo4e_keys::<bo4e_german::Adresse>(json, &exclude),
        "Zaehlwerk" => validate_bo4e_keys::<bo4e_german::Zaehlwerk>(json, &exclude),
        _ => vec![],
    }
}

/// Build a lookup from camelCase entity key → (bo4e_type, companion_type).
fn build_type_lookup(engines: &[&MappingEngine]) -> HashMap<String, (String, Option<String>)> {
    let mut lookup: HashMap<String, (String, Option<String>)> = HashMap::new();
    for engine in engines {
        for def in engine.definitions() {
            let entity = &def.meta.entity;
            let key = {
                let mut chars = entity.chars();
                match chars.next() {
                    Some(c) => c.to_lowercase().to_string() + chars.as_str(),
                    None => continue,
                }
            };
            lookup
                .entry(key)
                .and_modify(|(_, existing_ct)| {
                    if existing_ct.is_none() && def.meta.companion_type.is_some() {
                        *existing_ct = def.meta.companion_type.clone();
                    }
                })
                .or_insert_with(|| (def.meta.bo4e_type.clone(), def.meta.companion_type.clone()));
        }
    }
    lookup
}

/// Validate a mapped message against bo4e-german types.
/// Emits warnings for unknown field names. Does NOT panic.
pub fn validate_mapped_message(
    pid: &str,
    fixture_name: &str,
    msg_engine: &MappingEngine,
    tx_engine: &MappingEngine,
    mapped: &mig_bo4e::model::MappedMessage,
) {
    let type_lookup = build_type_lookup(&[msg_engine, tx_engine]);
    let mut all_unknown = Vec::new();

    // Validate message-level stammdaten
    if let Some(obj) = mapped.stammdaten.as_object() {
        for (key, value) in obj {
            if let Some((bo4e_type, companion_type)) = type_lookup.get(key) {
                validate_value(
                    key,
                    bo4e_type,
                    companion_type.as_deref(),
                    value,
                    &mut all_unknown,
                );
            }
        }
    }

    // Validate transaction-level stammdaten
    for (tx_idx, tx) in mapped.transaktionen.iter().enumerate() {
        if let Some(obj) = tx.stammdaten.as_object() {
            for (key, value) in obj {
                if let Some((bo4e_type, companion_type)) = type_lookup.get(key) {
                    let mut entity_unknown = Vec::new();
                    validate_value(
                        key,
                        bo4e_type,
                        companion_type.as_deref(),
                        value,
                        &mut entity_unknown,
                    );
                    for u in entity_unknown {
                        all_unknown.push(format!("tx[{tx_idx}].{u}"));
                    }
                }
            }
        }
    }

    if !all_unknown.is_empty() {
        eprintln!(
            "PID {pid} ({fixture_name}): BO4E schema validation WARNING — {} unknown field(s):",
            all_unknown.len()
        );
        for field in &all_unknown {
            eprintln!("  - {field}");
        }
    }
}

fn validate_value(
    entity_key: &str,
    bo4e_type: &str,
    companion_type: Option<&str>,
    value: &serde_json::Value,
    out: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(_) => {
            for key in validate_entity(bo4e_type, companion_type, value) {
                out.push(format!("{entity_key}.{key}"));
            }
        }
        serde_json::Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                for key in validate_entity(bo4e_type, companion_type, item) {
                    out.push(format!("{entity_key}[{i}].{key}"));
                }
            }
        }
        _ => {}
    }
}
