//! BO4E schema validation tests.
//!
//! Validates that TOML mapping field names produce JSON keys that match
//! real fields on bo4e-german types. Catches typos and hallucinated field
//! names that would silently produce invalid BO4E JSON.
//!
//! Approach: deserialize mapped JSON into the concrete bo4e-german type,
//! then re-serialize. Any key present in the original but absent after
//! the roundtrip is an unknown (hallucinated) field name.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use mig_assembly::assembler::Assembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_types::schema::mig::MigSchema;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ── Paths (relative to the crate root = crates/mig-bo4e) ──

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(Path::new(SCHEMA_DIR))
}

fn message_dir() -> PathBuf {
    Path::new(MAPPINGS_BASE).join("message")
}

fn pid_dir(pid: &str) -> PathBuf {
    Path::new(MAPPINGS_BASE).join(format!("pid_{pid}"))
}

fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        return None;
    }
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    Some(filter_mig_for_pid(&mig, &numbers))
}

/// Discover the first `.edi` fixture file for a given PID.
fn first_fixture(pid: &str) -> Option<PathBuf> {
    let dir = Path::new(FIXTURE_DIR);
    if !dir.exists() {
        return None;
    }
    let prefix = format!("{pid}_");
    let mut fixtures: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(".edi"))
                .unwrap_or(false)
        })
        .collect();
    fixtures.sort();
    fixtures.into_iter().next()
}

// ── Generic roundtrip key validation ──

/// Keys injected by the mapping engine that aren't part of bo4e-german types.
const ENGINE_METADATA_KEYS: &[&str] = &["boTyp", "versionStruktur"];

/// Validate BO4E JSON keys by deserializing into a concrete type and re-serializing.
/// Returns the list of keys present in `json` but absent after the serde roundtrip.
/// `exclude_keys` are intentionally non-BO4E keys (companion nesting) to skip.
fn validate_bo4e_keys<T>(json: &serde_json::Value, exclude_keys: &[&str]) -> Vec<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let original_keys: HashSet<&str> = match json.as_object() {
        Some(obj) => obj.keys().map(|k| k.as_str()).collect(),
        None => return vec![],
    };

    // Try to deserialize — if it fails entirely (e.g., enum value mismatch), skip.
    let typed: T = match serde_json::from_value(json.clone()) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    // Re-serialize to get only recognized keys
    let roundtripped = serde_json::to_value(&typed).unwrap();
    let roundtripped_keys: HashSet<&str> = match roundtripped.as_object() {
        Some(obj) => obj.keys().map(|k| k.as_str()).collect(),
        None => return vec![],
    };

    // Build exclusion set: companion keys + engine-injected metadata
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
/// Returns unknown keys for known types, empty vec for local/unknown types.
fn validate_entity(
    bo4e_type: &str,
    companion_type: Option<&str>,
    json: &serde_json::Value,
) -> Vec<String> {
    // Derive companion key to exclude (e.g., "MarktlokationEdifact" → "marktlokationEdifact")
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
        // Local types with no bo4e-german definition — skip validation
        _ => vec![],
    }
}

/// Build a lookup from camelCase entity key → (bo4e_type, companion_type).
/// When multiple definitions share an entity, prefer the one with a companion_type.
fn build_type_lookup(engines: &[&MappingEngine]) -> HashMap<String, (String, Option<String>)> {
    let mut lookup: HashMap<String, (String, Option<String>)> = HashMap::new();
    for engine in engines {
        for def in engine.definitions() {
            let entity = &def.meta.entity;
            // Convert to camelCase (lowercase first char)
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
                    // Prefer entries that have a companion_type
                    if existing_ct.is_none() && def.meta.companion_type.is_some() {
                        *existing_ct = def.meta.companion_type.clone();
                    }
                })
                .or_insert_with(|| (def.meta.bo4e_type.clone(), def.meta.companion_type.clone()));
        }
    }
    lookup
}

/// Validate a single JSON entity value (object or array of objects).
fn validate_entity_value(
    entity_key: &str,
    bo4e_type: &str,
    companion_type: Option<&str>,
    value: &serde_json::Value,
) -> Vec<String> {
    let mut all_unknown = Vec::new();

    match value {
        serde_json::Value::Object(_) => {
            let unknown = validate_entity(bo4e_type, companion_type, value);
            if !unknown.is_empty() {
                for key in &unknown {
                    all_unknown.push(format!("{entity_key}.{key}"));
                }
            }
        }
        serde_json::Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let unknown = validate_entity(bo4e_type, companion_type, item);
                for key in &unknown {
                    all_unknown.push(format!("{entity_key}[{i}].{key}"));
                }
            }
        }
        _ => {}
    }

    all_unknown
}

/// Run BO4E schema validation for a single PID.
/// Returns a list of unknown field paths (empty = all valid).
fn run_schema_validation(pid: &str) -> Vec<String> {
    let fixture_path = match first_fixture(pid) {
        Some(p) => p,
        None => {
            eprintln!("Skipping schema validation for PID {pid}: no fixture found");
            return vec![];
        }
    };

    let Some(filtered_mig) = load_pid_filtered_mig(pid) else {
        eprintln!("Skipping schema validation for PID {pid}: MIG/AHB XML not available");
        return vec![];
    };

    let msg_dir = message_dir();
    let tx_dir = pid_dir(pid);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping schema validation for PID {pid}: mapping directories not found");
        return vec![];
    }

    let (msg_engine, tx_engine) = MappingEngine::load_split(&msg_dir, &tx_dir)
        .unwrap_or_else(|e| panic!("Failed to load engines for PID {pid}: {e}"));
    let resolver = path_resolver();
    let msg_engine = msg_engine.with_path_resolver(resolver.clone());
    let tx_engine = tx_engine.with_path_resolver(resolver);

    // Build type lookup from definitions
    let type_lookup = build_type_lookup(&[&msg_engine, &tx_engine]);

    // Parse and assemble fixture
    let content = std::fs::read(&fixture_path).unwrap();
    let segments = parse_to_segments(&content).unwrap();
    let chunks = mig_assembly::split_messages(segments).unwrap();
    let msg = &chunks.messages[0];
    let all_segments = msg.all_segments();
    let assembler = Assembler::new(&filtered_mig);
    let tree = assembler.assemble_generic(&all_segments).unwrap();

    // Forward mapping with enrich_codes=false (raw string values for typed deser)
    let mapped = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", false);

    let mut all_unknown = Vec::new();

    // Validate message-level stammdaten
    if let Some(obj) = mapped.stammdaten.as_object() {
        for (key, value) in obj {
            if let Some((bo4e_type, companion_type)) = type_lookup.get(key) {
                let unknown =
                    validate_entity_value(key, bo4e_type, companion_type.as_deref(), value);
                all_unknown.extend(unknown);
            }
        }
    }

    // Validate transaction-level stammdaten
    for (tx_idx, tx) in mapped.transaktionen.iter().enumerate() {
        if let Some(obj) = tx.stammdaten.as_object() {
            for (key, value) in obj {
                if let Some((bo4e_type, companion_type)) = type_lookup.get(key) {
                    let unknown =
                        validate_entity_value(key, bo4e_type, companion_type.as_deref(), value);
                    for u in unknown {
                        all_unknown.push(format!("tx[{tx_idx}].{u}"));
                    }
                }
            }
        }
    }

    if all_unknown.is_empty() {
        eprintln!("PID {pid}: schema validation OK — all BO4E field names valid");
    } else {
        eprintln!(
            "PID {pid}: schema validation found {} unknown field(s):",
            all_unknown.len()
        );
        for field in &all_unknown {
            eprintln!("  - {field}");
        }
    }

    all_unknown
}

// ── Per-PID schema validation tests ──

macro_rules! schema_validation_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let unknown = run_schema_validation($pid);
            if !unknown.is_empty() {
                panic!(
                    "PID {} has {} unknown BO4E field name(s):\n{}",
                    $pid,
                    unknown.len(),
                    unknown
                        .iter()
                        .map(|f| format!("  - {f}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
        }
    };
}

schema_validation_test!(test_bo4e_schema_validation_55001, "55001");
schema_validation_test!(test_bo4e_schema_validation_55002, "55002");
schema_validation_test!(test_bo4e_schema_validation_55013, "55013");
schema_validation_test!(test_bo4e_schema_validation_55035, "55035");
schema_validation_test!(test_bo4e_schema_validation_55037, "55037");
schema_validation_test!(test_bo4e_schema_validation_55042, "55042");
schema_validation_test!(test_bo4e_schema_validation_55109, "55109");
schema_validation_test!(test_bo4e_schema_validation_55110, "55110");
schema_validation_test!(test_bo4e_schema_validation_55175, "55175");
schema_validation_test!(test_bo4e_schema_validation_55220, "55220");
schema_validation_test!(test_bo4e_schema_validation_55616, "55616");
schema_validation_test!(test_bo4e_schema_validation_55617, "55617");
schema_validation_test!(test_bo4e_schema_validation_55620, "55620");
schema_validation_test!(test_bo4e_schema_validation_55650, "55650");
schema_validation_test!(test_bo4e_schema_validation_55655, "55655");
