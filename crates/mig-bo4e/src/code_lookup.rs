//! Code enrichment lookup — maps EDIFACT companion field codes to human-readable meanings.
//!
//! Built from PID schema JSON files. Used by the mapping engine to automatically
//! enrich companion field values during forward mapping (EDIFACT → BO4E).

use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Lookup key: (source_path, segment_tag, element_index, component_index).
///
/// `source_path` matches the TOML `source_path` field (e.g., "sg4.sg8_z01.sg10").
/// `segment_tag` is uppercase (e.g., "CCI", "CAV").
pub type CodeLookupKey = (String, String, usize, usize);

/// Maps EDIFACT code values to their human-readable meanings.
/// E.g., "Z15" → "Haushaltskunde gem. EnWG".
pub type CodeMeanings = BTreeMap<String, String>;

/// Complete code lookup table built from a PID schema JSON.
#[derive(Debug, Clone, Default)]
pub struct CodeLookup {
    entries: HashMap<CodeLookupKey, CodeMeanings>,
}

impl CodeLookup {
    /// Build a CodeLookup from a PID schema JSON file.
    pub fn from_schema_file(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let schema: Value = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::from_schema_value(&schema))
    }

    /// Build a CodeLookup from an already-parsed PID schema JSON value.
    pub fn from_schema_value(schema: &Value) -> Self {
        let mut entries = HashMap::new();
        if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
            for (group_key, group_value) in fields {
                Self::walk_group(group_key, group_value, &mut entries);
            }
        }
        // Root-level segments (BGM, DTM, etc.) use empty source_path.
        if let Some(root_segments) = schema.get("root_segments").and_then(|s| s.as_array()) {
            for segment in root_segments {
                let seg_id = segment
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_uppercase();
                Self::process_segment("", &seg_id, segment, &mut entries);
            }
        }
        Self { entries }
    }

    /// Check if a companion field at the given position is a code-type field.
    pub fn is_code_field(
        &self,
        source_path: &str,
        segment_tag: &str,
        element_index: usize,
        component_index: usize,
    ) -> bool {
        let key = (
            source_path.to_string(),
            segment_tag.to_string(),
            element_index,
            component_index,
        );
        self.entries.contains_key(&key)
    }

    /// Get the human-readable meaning for a code value at the given position.
    /// Returns `None` if the position is not a code field or the value is unknown.
    pub fn meaning_for(
        &self,
        source_path: &str,
        segment_tag: &str,
        element_index: usize,
        component_index: usize,
        value: &str,
    ) -> Option<&str> {
        let key = (
            source_path.to_string(),
            segment_tag.to_string(),
            element_index,
            component_index,
        );
        self.entries
            .get(&key)
            .and_then(|meanings| meanings.get(value))
            .map(|s| s.as_str())
    }

    /// Walk a group node recursively, collecting code entries.
    fn walk_group(
        path_prefix: &str,
        group: &Value,
        entries: &mut HashMap<CodeLookupKey, CodeMeanings>,
    ) {
        if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
            for segment in segments {
                let seg_id = segment
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_uppercase();
                Self::process_segment(path_prefix, &seg_id, segment, entries);
            }
        }
        if let Some(children) = group.get("children").and_then(|c| c.as_object()) {
            for (child_key, child_value) in children {
                let child_path = format!("{}.{}", path_prefix, child_key);
                Self::walk_group(&child_path, child_value, entries);
            }
        }
    }

    /// Process a single segment, collecting code entries for its elements/components.
    fn process_segment(
        source_path: &str,
        segment_tag: &str,
        segment: &Value,
        entries: &mut HashMap<CodeLookupKey, CodeMeanings>,
    ) {
        let Some(elements) = segment.get("elements").and_then(|e| e.as_array()) else {
            return;
        };
        for element in elements {
            let element_index = element.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

            // Simple element (no composite) with codes
            if let Some("code") = element.get("type").and_then(|v| v.as_str()) {
                if let Some(codes) = element.get("codes").and_then(|c| c.as_array()) {
                    let meanings = Self::extract_codes(codes);
                    if !meanings.is_empty() {
                        let key = (
                            source_path.to_string(),
                            segment_tag.to_string(),
                            element_index,
                            0,
                        );
                        entries.insert(key, meanings);
                    }
                }
            }

            // Composite components
            if let Some(components) = element.get("components").and_then(|c| c.as_array()) {
                for component in components {
                    if let Some("code") = component.get("type").and_then(|v| v.as_str()) {
                        let sub_index = component
                            .get("sub_index")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize;
                        if let Some(codes) = component.get("codes").and_then(|c| c.as_array()) {
                            let meanings = Self::extract_codes(codes);
                            if !meanings.is_empty() {
                                let key = (
                                    source_path.to_string(),
                                    segment_tag.to_string(),
                                    element_index,
                                    sub_index,
                                );
                                entries.insert(key, meanings);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Extract code value→name mappings from a JSON codes array.
    fn extract_codes(codes: &[Value]) -> CodeMeanings {
        let mut meanings = BTreeMap::new();
        for code in codes {
            if let (Some(value), Some(name)) = (
                code.get("value").and_then(|v| v.as_str()),
                code.get("name").and_then(|v| v.as_str()),
            ) {
                meanings.insert(value.to_string(), name.to_string());
            }
        }
        meanings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pid_55001_schema() {
        let schema_path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json"
        ));
        if !schema_path.exists() {
            eprintln!("Skipping: PID schema not found");
            return;
        }

        let lookup = CodeLookup::from_schema_file(schema_path).unwrap();

        // CCI element 2 component 0 in sg4.sg8_z01.sg10 — Haushaltskunde codes
        assert!(lookup.is_code_field("sg4.sg8_z01.sg10", "CCI", 2, 0));
        assert_eq!(
            lookup.meaning_for("sg4.sg8_z01.sg10", "CCI", 2, 0, "Z15"),
            Some("Haushaltskunde gem. EnWG")
        );
        assert_eq!(
            lookup.meaning_for("sg4.sg8_z01.sg10", "CCI", 2, 0, "Z18"),
            Some("Kein Haushaltskunde gem. EnWG")
        );

        // CCI element 0 in sg4.sg8_z79.sg10 — Produkteigenschaft
        assert!(lookup.is_code_field("sg4.sg8_z79.sg10", "CCI", 0, 0));
        assert_eq!(
            lookup.meaning_for("sg4.sg8_z79.sg10", "CCI", 0, 0, "Z66"),
            Some("Produkteigenschaft")
        );

        // CAV element 0 component 0 — code field
        assert!(lookup.is_code_field("sg4.sg8_z79.sg10", "CAV", 0, 0));

        // CAV element 0 component 3 — data field, NOT a code
        assert!(!lookup.is_code_field("sg4.sg8_z79.sg10", "CAV", 0, 3));

        // LOC element 1 — data field
        assert!(!lookup.is_code_field("sg4.sg5_z16", "LOC", 1, 0));
    }

    #[test]
    fn test_from_inline_schema() {
        let schema = serde_json::json!({
            "fields": {
                "sg4": {
                    "children": {
                        "sg8_test": {
                            "children": {
                                "sg10": {
                                    "segments": [{
                                        "id": "CCI",
                                        "elements": [{
                                            "index": 2,
                                            "components": [{
                                                "sub_index": 0,
                                                "type": "code",
                                                "codes": [
                                                    {"value": "A1", "name": "Alpha"},
                                                    {"value": "B2", "name": "Beta"}
                                                ]
                                            }]
                                        }]
                                    }],
                                    "source_group": "SG10"
                                }
                            },
                            "segments": [],
                            "source_group": "SG8"
                        }
                    },
                    "segments": [],
                    "source_group": "SG4"
                }
            }
        });

        let lookup = CodeLookup::from_schema_value(&schema);

        assert!(lookup.is_code_field("sg4.sg8_test.sg10", "CCI", 2, 0));
        assert_eq!(
            lookup.meaning_for("sg4.sg8_test.sg10", "CCI", 2, 0, "A1"),
            Some("Alpha")
        );
        assert_eq!(
            lookup.meaning_for("sg4.sg8_test.sg10", "CCI", 2, 0, "B2"),
            Some("Beta")
        );
        assert_eq!(
            lookup.meaning_for("sg4.sg8_test.sg10", "CCI", 2, 0, "XX"),
            None
        );
        assert!(!lookup.is_code_field("sg4.sg8_test.sg10", "CCI", 0, 0));
    }
}
