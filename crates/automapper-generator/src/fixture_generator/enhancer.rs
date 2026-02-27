//! MappedMessage enhancer — replaces placeholder values with realistic data.
//!
//! Walks a `MappedMessage` JSON structure and replaces placeholder values
//! (generic IDs, names, addresses, dates) with deterministic, realistic data
//! derived from a seed value.

use std::collections::HashMap;

use mig_bo4e::definition::{FieldMapping, MappingDefinition};
use mig_bo4e::model::{MappedMessage, Transaktion};
use serde_json::Value;

use super::id_generators::{
    generate_gln, generate_malo_id, generate_melo_id, generate_nelo_id, generate_reference_id,
    generate_steuress_id, generate_techress_id,
};
use super::seed_data::{pick, pick_index, ADDRESSES, ANREDEN, NACHNAMEN, TITEL, VORNAMEN};

/// Maps BO4E field names to valid code lists (values with >1 option).
pub type CodeMap = HashMap<String, Vec<String>>;

/// Configuration for the enhancer, providing deterministic seed-based generation.
#[derive(Debug, Clone)]
pub struct EnhancerConfig {
    /// Base seed for all generation.
    pub seed: u64,
    /// Variant index — selects which code value to use from code lists.
    pub variant: usize,
}

impl EnhancerConfig {
    /// Create a new enhancer config.
    pub fn new(seed: u64, variant: usize) -> Self {
        Self { seed, variant }
    }

    /// Compute a deterministic sub-seed for a specific (entity, field) pair.
    ///
    /// Uses FNV-1a-style hashing of entity and field names mixed with the base seed
    /// so that different fields get different but reproducible values.
    pub fn field_seed(&self, entity: &str, field: &str) -> u64 {
        let mut h = self.seed;
        for b in entity.bytes() {
            h = h.wrapping_mul(1_099_511_628_211).wrapping_add(b as u64);
        }
        for b in field.bytes() {
            h = h.wrapping_mul(1_099_511_628_211).wrapping_add(b as u64);
        }
        h
    }
}

/// Build a code map keyed by BO4E field names.
///
/// Walks the PID schema to find code-type fields with >1 valid code value,
/// then maps them to BO4E field names using the TOML mapping definitions.
///
/// Returns a map from BO4E field name (e.g., "transaktionsgrund") to the list
/// of valid code values.
pub fn build_code_map(schema: &Value, definitions: &[MappingDefinition]) -> CodeMap {
    // Step 1: Build EDIFACT element ID → code values from schema
    let mut edifact_codes: HashMap<String, Vec<String>> = HashMap::new();
    walk_schema_for_codes(schema, &mut edifact_codes);

    // Step 2: Build BO4E field name → EDIFACT element ID from TOML definitions
    let mut bo4e_to_edifact_id: HashMap<String, String> = HashMap::new();
    for def in definitions {
        collect_field_edifact_ids(&def.fields, &mut bo4e_to_edifact_id);
        if let Some(ref companion) = def.companion_fields {
            collect_field_edifact_ids(companion, &mut bo4e_to_edifact_id);
        }
    }

    // Step 3: Combine — BO4E field name → code values
    let mut map = CodeMap::new();
    for (bo4e_name, edifact_id) in &bo4e_to_edifact_id {
        if let Some(codes) = edifact_codes.get(edifact_id) {
            let entry = map.entry(bo4e_name.clone()).or_default();
            if codes.len() > entry.len() {
                *entry = codes.clone();
            }
        }
    }
    map
}

/// Extract the EDIFACT data element ID from a TOML field path.
///
/// Examples:
/// - `"sts.c556.d9013"` → `Some("9013")`
/// - `"loc.d3227"` → `Some("3227")`
/// - `"cav[Z91].c889.d7111"` → `Some("7111")`
/// - `"loc.1.0"` (numeric legacy) → `None`
fn extract_edifact_element_id(path: &str) -> Option<&str> {
    // Get the last dot-separated component
    let last = path.rsplit('.').next()?;
    // Strip qualifier brackets if present (e.g., "d7111" from "cav[Z91].c889.d7111")
    let last = last.split('[').next()?;
    // Must start with 'd' followed by digits (EDIFACT data element ID)
    if last.starts_with('d') && last[1..].chars().all(|c| c.is_ascii_digit()) && last.len() > 1 {
        Some(&last[1..])
    } else {
        None
    }
}

/// Walk TOML field mappings to collect BO4E field name → EDIFACT element ID.
fn collect_field_edifact_ids(
    fields: &indexmap::IndexMap<String, FieldMapping>,
    out: &mut HashMap<String, String>,
) {
    for (edifact_path, mapping) in fields {
        let bo4e_name = match mapping {
            FieldMapping::Simple(name) => {
                if name.is_empty() {
                    continue;
                }
                name.as_str()
            }
            FieldMapping::Structured(s) => {
                if s.target.is_empty() {
                    continue;
                }
                s.target.as_str()
            }
            FieldMapping::Nested(_) => continue,
        };
        if let Some(eid) = extract_edifact_element_id(edifact_path) {
            out.insert(bo4e_name.to_string(), eid.to_string());
        }
    }
}

/// Recursively walk the schema JSON to extract code lists keyed by EDIFACT element ID.
fn walk_schema_for_codes(value: &Value, map: &mut HashMap<String, Vec<String>>) {
    match value {
        Value::Object(obj) => {
            // Check if this is a code element/component with >1 code
            if obj.get("type").and_then(|v| v.as_str()) == Some("code") {
                if let Some(codes) = obj.get("codes").and_then(|v| v.as_array()) {
                    if codes.len() > 1 {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            let values: Vec<String> = codes
                                .iter()
                                .filter_map(|c| c.get("value").and_then(|v| v.as_str()))
                                .map(String::from)
                                .collect();
                            if values.len() > 1 {
                                // Use the ID as key; if already present, keep the longer list
                                let entry = map.entry(id.to_string()).or_default();
                                if values.len() > entry.len() {
                                    *entry = values;
                                }
                            }
                        }
                    }
                }
            }
            // Recurse into all values
            for v in obj.values() {
                walk_schema_for_codes(v, map);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                walk_schema_for_codes(v, map);
            }
        }
        _ => {}
    }
}

/// Enhance a `MappedMessage` by replacing placeholder values with realistic data.
///
/// Walks both message-level `stammdaten` and per-transaction `stammdaten`/`transaktionsdaten`,
/// replacing IDs, names, addresses, dates, and generic placeholders with deterministic values.
pub fn enhance_mapped_message(
    msg: &mut MappedMessage,
    code_map: &Option<CodeMap>,
    config: &EnhancerConfig,
) {
    // Enhance message-level stammdaten
    enhance_json_object(&mut msg.stammdaten, config, code_map);

    // Enhance each transaction with offset seeds
    for (tx_idx, tx) in msg.transaktionen.iter_mut().enumerate() {
        let tx_offset = (tx_idx as u64) * 1000;
        let tx_config = EnhancerConfig::new(config.seed + tx_offset, config.variant);
        enhance_transaktion(tx, code_map, &tx_config);
    }
}

/// Enhance a single transaction's stammdaten and transaktionsdaten.
fn enhance_transaktion(tx: &mut Transaktion, code_map: &Option<CodeMap>, config: &EnhancerConfig) {
    enhance_json_object(&mut tx.stammdaten, config, code_map);
    // transaktionsdaten is a flat entity object (vorgangId, dates, etc.),
    // not a nested entity container — enhance it directly as an entity.
    enhance_entity_value(&mut tx.transaktionsdaten, "Prozessdaten", config, code_map);
}

/// Walk a JSON object that maps entity names to entity values (object or array of objects).
fn enhance_json_object(value: &mut Value, config: &EnhancerConfig, code_map: &Option<CodeMap>) {
    if let Value::Object(map) = value {
        for (entity_name, entity_value) in map.iter_mut() {
            // Skip companion type objects
            if entity_name.ends_with("Edifact") {
                continue;
            }

            match entity_value {
                Value::Object(_) => {
                    enhance_entity_value(entity_value, entity_name, config, code_map);
                }
                Value::Array(arr) => {
                    for (i, item) in arr.iter_mut().enumerate() {
                        let arr_config =
                            EnhancerConfig::new(config.seed + (i as u64) * 100, config.variant);
                        enhance_entity_value(item, entity_name, &arr_config, code_map);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Enhance a single entity JSON object by replacing placeholder field values.
///
/// Applies field recognition rules in priority order to replace IDs, names,
/// addresses, dates, code values, and generic placeholders.
pub fn enhance_entity_value(
    value: &mut Value,
    entity_name: &str,
    config: &EnhancerConfig,
    code_map: &Option<CodeMap>,
) {
    let obj = match value.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    // Pre-compute the address tuple seed so all address fields use the same address
    let addr_seed = config.field_seed(entity_name, "address_tuple");
    let addr_idx = pick_index(ADDRESSES.len(), addr_seed);
    let addr = &ADDRESSES[addr_idx];

    // Collect keys to iterate (we need to avoid borrowing issues)
    let keys: Vec<String> = obj.keys().cloned().collect();

    for key in &keys {
        // Skip companion type nested objects (keys ending in "Edifact")
        if key.ends_with("Edifact") {
            continue;
        }

        let current_value = match obj.get(key) {
            Some(v) => v.clone(),
            None => continue,
        };

        // Only process string values at this level
        let current_str = match current_value.as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };

        let field_seed = config.field_seed(entity_name, key);

        // Apply field recognition rules in priority order
        let new_value = match key.as_str() {
            // Location IDs
            "marktlokationsId" => Some(generate_malo_id(field_seed)),
            "messlokationsId" => Some(generate_melo_id(field_seed)),
            "netzlokationsId" => Some(generate_nelo_id(field_seed)),
            "steuerbareRessourceId" => Some(generate_steuress_id(field_seed)),
            "technischeRessourceId" => Some(generate_techress_id(field_seed)),
            "tranchenId" => Some(generate_malo_id(field_seed + 7)),

            // GLN identifiers (13-digit values)
            "identifikation" | "absenderCode" | "empfaengerCode" => {
                if is_13_digit_value(&current_str) {
                    Some(generate_gln(field_seed))
                } else {
                    None
                }
            }

            // Name fields
            "nachname" => Some(pick(NACHNAMEN, field_seed).to_string()),
            "vorname" => Some(pick(VORNAMEN, field_seed).to_string()),
            "anrede" => Some(pick(ANREDEN, field_seed).to_string()),
            "titel" => {
                if current_str.is_empty() {
                    None // Skip if empty
                } else {
                    Some(pick(TITEL, field_seed).to_string())
                }
            }

            // Address fields — all use the same coherent address tuple
            "strasse" => Some(addr.strasse.to_string()),
            "hausnummer" => Some(addr.hausnummer.to_string()),
            "ort" => Some(addr.ort.to_string()),
            "postleitzahl" => Some(addr.plz.to_string()),
            "region" => Some(addr.bundesland.to_string()),
            "land" => Some("DE".to_string()),

            // Reference IDs
            "vorgangId" => Some(generate_reference_id(field_seed)),

            // Date fields
            _ if is_date_field(key) => Some(generate_date(&current_str, field_seed, key)),

            // Catch-all: reference-like fields and generic placeholders
            _ if key.ends_with("Referenz") || key.ends_with("referenz") => {
                Some(generate_reference_id(field_seed))
            }
            _ if is_generic_placeholder(&current_str) => Some(generate_reference_id(field_seed)),
            _ if is_code_field(key, code_map) => apply_code_variant(key, code_map, config),
            _ => None,
        };

        if let Some(new_val) = new_value {
            obj.insert(key.clone(), Value::String(new_val));
        }
    }
}

/// Check if a string value looks like a 13-digit GLN.
fn is_13_digit_value(s: &str) -> bool {
    s.len() == 13 && s.chars().all(|c| c.is_ascii_digit())
}

/// Check if a field name looks like a date field.
fn is_date_field(key: &str) -> bool {
    key.ends_with("Ab")
        || key.ends_with("Bis")
        || key.ends_with("Von")
        || key.starts_with("gueltig")
        || key.ends_with("Datum")
        || key.ends_with("datum")
}

/// Check if a field has an entry in the code map.
fn is_code_field(key: &str, code_map: &Option<CodeMap>) -> bool {
    if let Some(cm) = code_map {
        cm.contains_key(key)
    } else {
        false
    }
}

/// Apply a code variant selection from the code map.
fn apply_code_variant(
    key: &str,
    code_map: &Option<CodeMap>,
    config: &EnhancerConfig,
) -> Option<String> {
    if let Some(cm) = code_map {
        if let Some(codes) = cm.get(key) {
            if !codes.is_empty() {
                let idx = config.variant % codes.len();
                return Some(codes[idx].clone());
            }
        }
    }
    None
}

/// Check if a value looks like a generic placeholder that should be replaced.
fn is_generic_placeholder(s: &str) -> bool {
    matches!(s, "X" | "TESTID" | "TESTPRODUCT") || s.starts_with("GENERATED")
}

/// Generate a deterministic date string in EDIFACT 303 format.
///
/// Start dates (`*Ab`, `*Von`) use base offset; end dates get +365.
/// Format: `CCYYMMDD120000?+00` (but `?+00` is the EDIFACT escape — in BO4E
/// JSON we store the raw `+00`).
fn generate_date(current_value: &str, seed: u64, field_name: &str) -> String {
    // Determine if this is an end date (add 365 days offset)
    let is_end = field_name.ends_with("Bis");
    let day_offset = (seed % 730) + if is_end { 365 } else { 0 };

    // Calculate date from 2024-01-01 + offset days
    let base_year = 2024u64;
    let mut year = base_year;
    let mut remaining = day_offset;

    // Walk through years
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    // Walk through months
    let month_days = days_per_month(year);
    let mut month = 1u64;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        month += 1;
    }
    let day = remaining + 1;

    // Detect format from current value
    let format = detect_date_format(current_value);
    match format {
        DateFormat::Edifact303 => format!("{year:04}{month:02}{day:02}120000+00"),
        DateFormat::Edifact102 => format!("{year:04}{month:02}{day:02}"),
    }
}

/// Detected date format from the current placeholder value.
#[derive(Debug, PartialEq)]
enum DateFormat {
    /// CCYYMMDDHHMMSS+TZ (length >= 15), also the fallback
    Edifact303,
    /// CCYYMMDD (length 8)
    Edifact102,
}

fn detect_date_format(value: &str) -> DateFormat {
    // Strip any EDIFACT escape sequences for analysis
    let clean = value.replace("?+", "+");
    if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_digit()) {
        DateFormat::Edifact102
    } else {
        // Default to 303 format (CCYYMMDDHHMMSS+TZ)
        DateFormat::Edifact303
    }
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_per_month(year: u64) -> [u64; 12] {
    let feb = if is_leap_year(year) { 29 } else { 28 };
    [31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_config(seed: u64) -> EnhancerConfig {
        EnhancerConfig::new(seed, 0)
    }

    #[test]
    fn test_enhance_replaces_marktlokations_id() {
        let mut value = json!({
            "marktlokationsId": "DE000111222333"
        });
        let config = make_config(42);
        enhance_entity_value(&mut value, "Marktlokation", &config, &None);

        let new_id = value["marktlokationsId"].as_str().unwrap();
        assert_ne!(new_id, "DE000111222333", "MaLo ID should be replaced");
        assert_eq!(new_id.len(), 11, "MaLo ID must be 11 digits, got: {new_id}");
        assert!(
            new_id.chars().all(|c| c.is_ascii_digit()),
            "MaLo ID must be all digits, got: {new_id}"
        );
    }

    #[test]
    fn test_enhance_replaces_nachname() {
        let mut value = json!({
            "nachname": "Mustermann"
        });
        let config = make_config(42);
        enhance_entity_value(&mut value, "Geschaeftspartner", &config, &None);

        let name = value["nachname"].as_str().unwrap();
        assert_ne!(name, "", "Name should not be empty");
        assert!(
            NACHNAMEN.contains(&name),
            "Name '{}' should be from seed data",
            name
        );
    }

    #[test]
    fn test_enhance_replaces_address_coherently() {
        let mut value = json!({
            "strasse": "Musterstrasse",
            "hausnummer": "1",
            "ort": "Musterstadt",
            "postleitzahl": "12345",
            "region": "XX"
        });
        let config = make_config(42);
        enhance_entity_value(&mut value, "Geschaeftspartner", &config, &None);

        let plz = value["postleitzahl"].as_str().unwrap();
        let ort = value["ort"].as_str().unwrap();

        // Find which address was picked and verify coherence
        let matching = ADDRESSES.iter().find(|a| a.plz == plz);
        assert!(
            matching.is_some(),
            "PLZ '{}' must come from seed addresses",
            plz
        );
        let addr = matching.unwrap();
        assert_eq!(
            addr.ort, ort,
            "City '{}' must match PLZ '{}' address (expected '{}')",
            ort, plz, addr.ort
        );
        assert_eq!(
            value["strasse"].as_str().unwrap(),
            addr.strasse,
            "Street must match the same address"
        );
        assert_eq!(
            value["hausnummer"].as_str().unwrap(),
            addr.hausnummer,
            "House number must match the same address"
        );
        assert_eq!(
            value["region"].as_str().unwrap(),
            addr.bundesland,
            "Region must match the same address"
        );
    }

    #[test]
    fn test_enhance_preserves_companion_qualifiers() {
        let mut value = json!({
            "nachname": "Mustermann",
            "geschaeftspartnerEdifact": {
                "nad_qualifier": "Z04",
                "codelist_code": "293"
            }
        });
        let config = make_config(42);
        enhance_entity_value(&mut value, "Geschaeftspartner", &config, &None);

        // Companion object must be untouched
        let companion = &value["geschaeftspartnerEdifact"];
        assert_eq!(
            companion["nad_qualifier"].as_str().unwrap(),
            "Z04",
            "Companion qualifier must not be modified"
        );
        assert_eq!(
            companion["codelist_code"].as_str().unwrap(),
            "293",
            "Companion codelist_code must not be modified"
        );
    }

    #[test]
    fn test_enhance_deterministic() {
        let mut value1 = json!({
            "marktlokationsId": "DE000111222333",
            "nachname": "Mustermann",
            "ort": "Musterstadt"
        });
        let mut value2 = value1.clone();

        let config = make_config(42);
        enhance_entity_value(&mut value1, "Test", &config, &None);
        enhance_entity_value(&mut value2, "Test", &config, &None);

        assert_eq!(value1, value2, "Same seed must produce identical output");
    }

    #[test]
    fn test_enhance_gln_identifikation() {
        let mut value = json!({
            "identifikation": "1234567890128"
        });
        let config = make_config(99);
        enhance_entity_value(&mut value, "Marktteilnehmer", &config, &None);

        let new_id = value["identifikation"].as_str().unwrap();
        assert_ne!(new_id, "1234567890128", "GLN should be replaced");
        assert_eq!(new_id.len(), 13, "GLN must be 13 digits, got: {new_id}");
        assert!(
            new_id.chars().all(|c| c.is_ascii_digit()),
            "GLN must be all digits, got: {new_id}"
        );
    }

    #[test]
    fn test_enhance_reference_ids() {
        let mut value = json!({
            "vorgangId": "GENERATED00001",
            "externeReferenz": "TESTREF"
        });
        let config = make_config(42);
        enhance_entity_value(&mut value, "Prozessdaten", &config, &None);

        let vorgang = value["vorgangId"].as_str().unwrap();
        assert_ne!(vorgang, "GENERATED00001", "vorgangId should be replaced");
        // Verify no EDIFACT special characters
        assert!(!vorgang.contains('+'), "Reference must not contain +");
        assert!(!vorgang.contains(':'), "Reference must not contain :");
        assert!(!vorgang.contains('\''), "Reference must not contain '");
        assert!(!vorgang.contains('?'), "Reference must not contain ?");

        // externeReferenz ends with "referenz" (case-insensitive match via ends_with)
        // but our check is exact: "Referenz" or "referenz"
        // "externeReferenz" ends with "Referenz" → matches the *Referenz rule
        let ext_ref = value["externeReferenz"].as_str().unwrap();
        assert_ne!(ext_ref, "TESTREF", "externeReferenz should be replaced");
    }

    #[test]
    fn test_enhance_dates() {
        let mut value = json!({
            "gueltigAb": "20250401120000+00",
            "gueltigBis": "20260401120000+00"
        });
        let config = make_config(42);
        enhance_entity_value(&mut value, "Prozessdaten", &config, &None);

        let ab = value["gueltigAb"].as_str().unwrap();
        let bis = value["gueltigBis"].as_str().unwrap();

        assert_ne!(ab, "20250401120000+00", "gueltigAb should be replaced");
        assert_ne!(bis, "20260401120000+00", "gueltigBis should be replaced");
        assert_ne!(ab, bis, "gueltigAb and gueltigBis should differ");

        // Check format: CCYYMMDDHHMMSS+TZ
        assert!(ab.len() >= 15, "Date should be in 303 format, got: {ab}");
        assert!(ab.contains('+'), "Date should contain timezone, got: {ab}");
    }

    #[test]
    fn test_extract_edifact_element_id() {
        assert_eq!(extract_edifact_element_id("sts.c556.d9013"), Some("9013"));
        assert_eq!(extract_edifact_element_id("loc.d3227"), Some("3227"));
        assert_eq!(
            extract_edifact_element_id("cav[Z91].c889.d7111"),
            Some("7111")
        );
        assert_eq!(extract_edifact_element_id("nad.c080.d3036"), Some("3036"));
        // Legacy numeric paths should return None
        assert_eq!(extract_edifact_element_id("loc.1.0"), None);
        assert_eq!(extract_edifact_element_id("seq.0"), None);
    }

    #[test]
    fn test_build_code_map_with_definitions() {
        use indexmap::IndexMap;
        use mig_bo4e::definition::{MappingDefinition, MappingMeta};

        // Schema with a code element id=9013 having 2 codes
        let schema = json!({
            "fields": {
                "sg4": {
                    "segments": [{
                        "id": "STS",
                        "elements": [{
                            "type": "code",
                            "id": "9013",
                            "codes": [
                                {"value": "E01"},
                                {"value": "ZW4"}
                            ]
                        }]
                    }]
                }
            }
        });

        // TOML definition mapping d9013 → "transaktionsgrund"
        let mut fields = IndexMap::new();
        fields.insert(
            "sts.c556.d9013".to_string(),
            FieldMapping::Simple("transaktionsgrund".to_string()),
        );

        let def = MappingDefinition {
            meta: MappingMeta {
                entity: "Prozessdaten".to_string(),
                bo4e_type: "Prozessdaten".to_string(),
                companion_type: None,
                source_group: "SG4".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields,
            companion_fields: None,
            complex_handlers: None,
        };

        let code_map = build_code_map(&schema, &[def]);
        // Should be keyed by BO4E field name, not EDIFACT element ID
        assert!(
            code_map.contains_key("transaktionsgrund"),
            "Code map should contain BO4E field name 'transaktionsgrund', got: {:?}",
            code_map.keys().collect::<Vec<_>>()
        );
        assert_eq!(code_map["transaktionsgrund"], vec!["E01", "ZW4"]);
        // Should NOT be keyed by EDIFACT element ID
        assert!(
            !code_map.contains_key("9013"),
            "Code map should NOT contain EDIFACT element ID"
        );
    }

    #[test]
    fn test_enhance_code_variant_selection() {
        // Build a code map with a known BO4E field
        let mut code_map = CodeMap::new();
        code_map.insert(
            "transaktionsgrund".to_string(),
            vec!["E01".to_string(), "ZW4".to_string()],
        );

        // Variant 0 → first code
        let mut value = json!({"transaktionsgrund": "E01"});
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Prozessdaten", &config, &Some(code_map.clone()));
        assert_eq!(
            value["transaktionsgrund"].as_str().unwrap(),
            "E01",
            "Variant 0 should select first code"
        );

        // Variant 1 → second code
        let mut value = json!({"transaktionsgrund": "E01"});
        let config = EnhancerConfig::new(42, 1);
        enhance_entity_value(&mut value, "Prozessdaten", &config, &Some(code_map));
        assert_eq!(
            value["transaktionsgrund"].as_str().unwrap(),
            "ZW4",
            "Variant 1 should select second code"
        );
    }

    #[test]
    fn test_enhance_mapped_message() {
        let mut msg = MappedMessage {
            stammdaten: json!({
                "marktteilnehmer": [{
                    "identifikation": "1234567890128",
                    "marktteilnehmerEdifact": {
                        "nad_qualifier": "MS"
                    }
                }]
            }),
            transaktionen: vec![Transaktion {
                stammdaten: json!({
                    "marktlokation": {
                        "marktlokationsId": "00000000001"
                    },
                    "geschaeftspartner": [{
                        "nachname": "Mustermann",
                        "vorname": "Max",
                        "ort": "Musterstadt",
                        "postleitzahl": "12345",
                        "geschaeftspartnerEdifact": {
                            "nad_qualifier": "Z65"
                        }
                    }]
                }),
                transaktionsdaten: json!({
                    "vorgangId": "GENERATED00001",
                    "gueltigAb": "20250401120000+00"
                }),
            }],
        };

        let config = EnhancerConfig::new(42, 0);
        enhance_mapped_message(&mut msg, &None, &config);

        // Message-level: GLN should be replaced
        let mt = &msg.stammdaten["marktteilnehmer"][0];
        let gln = mt["identifikation"].as_str().unwrap();
        assert_ne!(gln, "1234567890128", "GLN should be replaced");
        assert_eq!(gln.len(), 13, "GLN must be 13 digits");

        // Companion preserved
        assert_eq!(
            mt["marktteilnehmerEdifact"]["nad_qualifier"]
                .as_str()
                .unwrap(),
            "MS",
            "Companion qualifier must be preserved"
        );

        // Transaction stammdaten
        let tx = &msg.transaktionen[0];
        let malo = tx.stammdaten["marktlokation"]["marktlokationsId"]
            .as_str()
            .unwrap();
        assert_ne!(malo, "00000000001", "MaLo ID should be replaced");
        assert_eq!(malo.len(), 11, "MaLo ID must be 11 digits");

        // Geschaeftspartner name replaced
        let gp = &tx.stammdaten["geschaeftspartner"][0];
        let nachname = gp["nachname"].as_str().unwrap();
        assert!(NACHNAMEN.contains(&nachname), "Name must be from seed data");

        // Companion preserved in array element
        assert_eq!(
            gp["geschaeftspartnerEdifact"]["nad_qualifier"]
                .as_str()
                .unwrap(),
            "Z65",
            "Companion qualifier must be preserved in array"
        );

        // Transaktionsdaten
        let vorgang = tx.transaktionsdaten["vorgangId"].as_str().unwrap();
        assert_ne!(vorgang, "GENERATED00001", "vorgangId should be replaced");

        let ab = tx.transaktionsdaten["gueltigAb"].as_str().unwrap();
        assert_ne!(ab, "20250401120000+00", "Date should be replaced");
    }
}
