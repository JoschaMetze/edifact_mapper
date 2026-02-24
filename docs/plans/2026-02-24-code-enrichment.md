# Code Enrichment for Companion Fields — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automatically enrich companion field code values with human-readable meanings from the PID schema JSON.

**Architecture:** New `code_lookup` module parses PID schema JSON into a lookup table keyed by `(source_path, segment_tag, element_index, component_index)`. The `MappingEngine` optionally holds this lookup and uses it during `extract_companion_fields()` to emit `{"code": "Z15", "meaning": "Ja"}` instead of plain `"Z15"`. Reverse mapping accepts both formats.

**Tech Stack:** Rust, serde_json, existing mig-bo4e crate

---

### Task 1: Add `CodeLookup` type and schema parser

**Files:**
- Create: `crates/mig-bo4e/src/code_lookup.rs`
- Modify: `crates/mig-bo4e/src/lib.rs:17` (add `pub mod code_lookup;`)

**Step 1: Write the failing test**

In `crates/mig-bo4e/src/code_lookup.rs`, add the module with a test that parses a real PID schema:

```rust
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
                // group_key is e.g. "sg2", "sg4"
                Self::walk_group(group_key, group_value, &mut entries);
            }
        }
        Self { entries }
    }

    /// Look up the meaning for a code value at the given position.
    pub fn lookup(
        &self,
        source_path: &str,
        segment_tag: &str,
        element_index: usize,
        component_index: usize,
    ) -> Option<&str> {
        // This method is unused for now — the engine uses `is_code_field` + `meaning_for`
        let key = (
            source_path.to_string(),
            segment_tag.to_uppercase(),
            element_index,
            component_index,
        );
        // Not used directly, but keeping for potential future use
        let _ = self.entries.get(&key);
        None
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
            segment_tag.to_uppercase(),
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
            segment_tag.to_uppercase(),
            element_index,
            component_index,
        );
        self.entries
            .get(&key)
            .and_then(|meanings| meanings.get(value))
            .map(|s| s.as_str())
    }

    /// Walk a group node recursively, collecting code entries.
    ///
    /// `path_prefix` is the dotted source_path built so far (e.g., "sg4.sg8_z01").
    fn walk_group(
        path_prefix: &str,
        group: &Value,
        entries: &mut HashMap<CodeLookupKey, CodeMeanings>,
    ) {
        // Process segments at this level
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

        // Recurse into children
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
            let element_index = element
                .get("index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            // Check if this is a simple element (no composite) with codes
            if let Some("code") = element.get("type").and_then(|v| v.as_str()) {
                if let Some(codes) = element.get("codes").and_then(|c| c.as_array()) {
                    let meanings = Self::extract_codes(codes);
                    if !meanings.is_empty() {
                        let key = (
                            source_path.to_string(),
                            segment_tag.to_string(),
                            element_index,
                            0, // simple elements have implicit component index 0
                        );
                        entries.insert(key, meanings);
                    }
                }
            }

            // Check composite components
            if let Some(components) = element.get("components").and_then(|c| c.as_array()) {
                for component in components {
                    if let Some("code") = component.get("type").and_then(|v| v.as_str()) {
                        let sub_index = component
                            .get("sub_index")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                            as usize;

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

        // CCI element 2 component 0 in sg4.sg8_z01.sg10 should be a code field
        // (Haushaltskunde: Z15/Z18)
        assert!(lookup.is_code_field("sg4.sg8_z01.sg10", "CCI", 2, 0));
        assert_eq!(
            lookup.meaning_for("sg4.sg8_z01.sg10", "CCI", 2, 0, "Z15"),
            Some("Haushaltskunde gem. EnWG")
        );
        assert_eq!(
            lookup.meaning_for("sg4.sg8_z01.sg10", "CCI", 2, 0, "Z18"),
            Some("Kein Haushaltskunde gem. EnWG")
        );

        // CCI element 0 in sg4.sg8_z79.sg10 should be a code field (Z66 = Produkteigenschaft)
        assert!(lookup.is_code_field("sg4.sg8_z79.sg10", "CCI", 0, 0));
        assert_eq!(
            lookup.meaning_for("sg4.sg8_z79.sg10", "CCI", 0, 0, "Z66"),
            Some("Produkteigenschaft")
        );

        // CAV element 0 component 0 in sg4.sg8_z79.sg10 should be a code field
        assert!(lookup.is_code_field("sg4.sg8_z79.sg10", "CAV", 0, 0));

        // CAV element 0 component 3 in sg4.sg8_z79.sg10 should NOT be a code field (type=data)
        assert!(!lookup.is_code_field("sg4.sg8_z79.sg10", "CAV", 0, 3));

        // LOC element 0 in sg4.sg5_z16 should NOT be a code field directly
        // (it has data type for MaLo ID), but LOC qualifier codes exist at sg4 level
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
```

**Step 2: Register the module**

In `crates/mig-bo4e/src/lib.rs`, add `pub mod code_lookup;` after line 17 (after `pub mod engine;`).

**Step 3: Run tests to verify**

Run: `cargo test -p mig-bo4e code_lookup`
Expected: PASS — both `test_parse_pid_55001_schema` and `test_from_inline_schema` pass.

**Step 4: Commit**

```bash
git add crates/mig-bo4e/src/code_lookup.rs crates/mig-bo4e/src/lib.rs
git commit -m "feat(mig-bo4e): add code_lookup module for PID schema code enrichment"
```

---

### Task 2: Integrate CodeLookup into MappingEngine forward mapping

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs:19-22` (add `code_lookup` field to `MappingEngine`)
- Modify: `crates/mig-bo4e/src/engine.rs:24-94` (add constructors/builders)
- Modify: `crates/mig-bo4e/src/engine.rs:242-259` (`extract_companion_fields`)

This task has three sub-parts:

**Step 1: Add `code_lookup` field and builder method**

In `crates/mig-bo4e/src/engine.rs`, modify the `MappingEngine` struct (line 19):

```rust
pub struct MappingEngine {
    definitions: Vec<MappingDefinition>,
    segment_structure: Option<SegmentStructure>,
    code_lookup: Option<crate::code_lookup::CodeLookup>,
}
```

Update all constructors to initialize `code_lookup: None`:
- `load()` (line 43)
- `load_merged()` (line 73)
- `from_definitions()` (line 81)

Add a builder method after `with_segment_structure()` (after line 94):

```rust
/// Attach a code lookup for enriching companion field values.
///
/// When set, companion fields that map to code-type elements in the PID schema
/// are emitted as `{"code": "Z15", "meaning": "Ja"}` objects instead of plain strings.
pub fn with_code_lookup(mut self, cl: crate::code_lookup::CodeLookup) -> Self {
    self.code_lookup = Some(cl);
    self
}
```

**Step 2: Modify `extract_companion_fields` to enrich code values**

Replace `extract_companion_fields` (lines 243-259) with a version that accepts `&self` and checks the code lookup:

```rust
/// Extract companion_fields into a nested object within the result.
///
/// When a `code_lookup` is configured, code-type fields are emitted as
/// `{"code": "Z15", "meaning": "Ja"}` objects. Data-type fields remain plain strings.
fn extract_companion_fields(
    &self,
    instance: &AssembledGroupInstance,
    def: &MappingDefinition,
    result: &mut serde_json::Map<String, serde_json::Value>,
) {
    if let Some(ref companion_fields) = def.companion_fields {
        let companion_key = def.meta.companion_type.as_deref().unwrap_or("_companion");
        let mut companion_result = serde_json::Map::new();

        for (path, field_mapping) in companion_fields {
            let (target, enum_map) = match field_mapping {
                FieldMapping::Simple(t) => (t.clone(), None),
                FieldMapping::Structured(s) => (s.target.clone(), s.enum_map.as_ref()),
                FieldMapping::Nested(_) => continue,
            };
            if target.is_empty() {
                continue;
            }
            if let Some(val) = Self::extract_from_instance(instance, path) {
                let mapped_val = if let Some(map) = enum_map {
                    map.get(&val).cloned().unwrap_or(val)
                } else {
                    val
                };

                // Enrich code fields with meaning from PID schema
                if let (Some(ref code_lookup), Some(ref source_path)) =
                    (&self.code_lookup, &def.meta.source_path)
                {
                    let (seg_tag, _qualifier) = parse_tag_qualifier(
                        path.split('.').next().unwrap_or(""),
                    );
                    let parts: Vec<&str> = path.split('.').collect();
                    let (element_idx, component_idx) =
                        Self::parse_element_component(&parts[1..]);

                    if code_lookup.is_code_field(
                        source_path,
                        &seg_tag,
                        element_idx,
                        component_idx,
                    ) {
                        let meaning = code_lookup
                            .meaning_for(
                                source_path,
                                &seg_tag,
                                element_idx,
                                component_idx,
                                &mapped_val,
                            )
                            .map(|m| serde_json::Value::String(m.to_string()))
                            .unwrap_or(serde_json::Value::Null);

                        let enriched = serde_json::json!({
                            "code": mapped_val,
                            "meaning": meaning,
                        });
                        set_nested_value_json(
                            &mut companion_result,
                            &target,
                            enriched,
                        );
                        continue;
                    }
                }

                set_nested_value(&mut companion_result, &target, mapped_val);
            }
        }

        if !companion_result.is_empty() {
            result.insert(
                companion_key.to_string(),
                serde_json::Value::Object(companion_result),
            );
        }
    }
}
```

**Step 3: Add helper methods**

Add a `parse_element_component` helper to `MappingEngine` impl (private):

```rust
/// Parse element and component indices from path parts after the segment tag.
/// E.g., ["2"] → (2, 0), ["0", "3"] → (0, 3), ["1", "0"] → (1, 0)
fn parse_element_component(parts: &[&str]) -> (usize, usize) {
    if parts.is_empty() {
        return (0, 0);
    }
    let element_idx = parts[0].parse::<usize>().unwrap_or(0);
    let component_idx = if parts.len() > 1 {
        parts[1].parse::<usize>().unwrap_or(0)
    } else {
        0
    };
    (element_idx, component_idx)
}
```

Add a `set_nested_value_json` free function (after `set_nested_value`):

```rust
/// Like `set_nested_value` but accepts a `serde_json::Value` instead of a `String`.
fn set_nested_value_json(
    map: &mut serde_json::Map<String, serde_json::Value>,
    path: &str,
    val: serde_json::Value,
) {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() == 1 {
        map.insert(parts[0].to_string(), val);
        return;
    }
    let mut current = map;
    for part in &parts[..parts.len() - 1] {
        let entry = current
            .entry(part.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        current = entry.as_object_mut().expect("expected object in path");
    }
    current.insert(parts.last().unwrap().to_string(), val);
}
```

**Step 4: Update call sites**

The `extract_companion_fields` method signature changes from `fn(instance, def, result)` (static) to `fn(&self, instance, def, result)` (takes `&self`). Update the two call sites in `map_forward()`:
- Line 228: `Self::extract_companion_fields(...)` → `self.extract_companion_fields(...)`
- Line 236: `Self::extract_companion_fields(...)` → `self.extract_companion_fields(...)`

**Step 5: Write unit test**

Add a test in `engine.rs` `mod tests`:

```rust
#[test]
fn test_extract_companion_fields_with_code_enrichment() {
    use crate::code_lookup::CodeLookup;
    use mig_assembly::assembler::*;

    // Build a minimal schema with a code field at CCI element 2 component 0
    let schema = serde_json::json!({
        "fields": {
            "sg4": {
                "children": {
                    "sg8_z01": {
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
                                                {"value": "Z15", "name": "Haushaltskunde"},
                                                {"value": "Z18", "name": "Kein Haushaltskunde"}
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

    let code_lookup = CodeLookup::from_schema_value(&schema);

    // Build a tree with CCI+++Z15
    let tree = AssembledTree {
        segments: vec![],
        groups: vec![AssembledGroup {
            group_id: "SG4".to_string(),
            repetitions: vec![AssembledGroupInstance {
                segments: vec![],
                child_groups: vec![AssembledGroup {
                    group_id: "SG8".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![],
                        child_groups: vec![AssembledGroup {
                            group_id: "SG10".to_string(),
                            repetitions: vec![AssembledGroupInstance {
                                segments: vec![AssembledSegment {
                                    tag: "CCI".to_string(),
                                    elements: vec![
                                        vec![],      // element 0 empty
                                        vec![],      // element 1 empty
                                        vec!["Z15".to_string()], // element 2
                                    ],
                                }],
                                child_groups: vec![],
                            }],
                        }],
                    }],
                }],
            }],
        }],
        post_group_start: 0,
    };

    // Build a definition with companion_fields
    let mut companion_fields = BTreeMap::new();
    companion_fields.insert(
        "cci.2".to_string(),
        FieldMapping::Simple("haushaltskunde".to_string()),
    );

    let def = MappingDefinition {
        meta: MappingMeta {
            entity: "Marktlokation".to_string(),
            bo4e_type: "Marktlokation".to_string(),
            companion_type: Some("MarktlokationEdifact".to_string()),
            source_group: "SG4.SG8.SG10".to_string(),
            source_path: Some("sg4.sg8_z01.sg10".to_string()),
            discriminator: None,
        },
        fields: BTreeMap::new(),
        companion_fields: Some(companion_fields),
        complex_handlers: None,
    };

    // Without code lookup — plain string
    let engine_plain = MappingEngine::from_definitions(vec![]);
    let bo4e_plain = engine_plain.map_forward(&tree, &def, 0);
    assert_eq!(
        bo4e_plain["MarktlokationEdifact"]["haushaltskunde"].as_str(),
        Some("Z15"),
        "Without code lookup, should be plain string"
    );

    // With code lookup — enriched object
    let engine_enriched =
        MappingEngine::from_definitions(vec![]).with_code_lookup(code_lookup);
    let bo4e_enriched = engine_enriched.map_forward(&tree, &def, 0);
    let hk = &bo4e_enriched["MarktlokationEdifact"]["haushaltskunde"];
    assert_eq!(hk["code"].as_str(), Some("Z15"));
    assert_eq!(hk["meaning"].as_str(), Some("Haushaltskunde"));
}
```

**Step 6: Run tests**

Run: `cargo test -p mig-bo4e -- test_extract_companion_fields_with_code_enrichment`
Expected: PASS

Run: `cargo test -p mig-bo4e`
Expected: All 92 existing tests still pass (no behavioral change without code_lookup)

**Step 7: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs
git commit -m "feat(mig-bo4e): integrate CodeLookup into MappingEngine forward mapping"
```

---

### Task 3: Update reverse mapping to accept enriched code objects

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs:430-505` (reverse mapping companion section)

**Step 1: Write the failing test**

Add a test in `engine.rs` `mod tests`:

```rust
#[test]
fn test_reverse_mapping_accepts_enriched_companion() {
    // Reverse mapping should accept both plain string and enriched object format
    let mut companion_fields = BTreeMap::new();
    companion_fields.insert(
        "cci.2".to_string(),
        FieldMapping::Simple("haushaltskunde".to_string()),
    );

    let def = MappingDefinition {
        meta: MappingMeta {
            entity: "Test".to_string(),
            bo4e_type: "Test".to_string(),
            companion_type: Some("TestEdifact".to_string()),
            source_group: "SG4".to_string(),
            source_path: None,
            discriminator: None,
        },
        fields: BTreeMap::new(),
        companion_fields: Some(companion_fields),
        complex_handlers: None,
    };

    let engine = MappingEngine::from_definitions(vec![]);

    // Test 1: Plain string format (backward compat)
    let bo4e_plain = serde_json::json!({
        "TestEdifact": {
            "haushaltskunde": "Z15"
        }
    });
    let instance_plain = engine.map_reverse(&bo4e_plain, &def);
    assert_eq!(instance_plain.segments[0].elements[2], vec!["Z15"]);

    // Test 2: Enriched object format
    let bo4e_enriched = serde_json::json!({
        "TestEdifact": {
            "haushaltskunde": {
                "code": "Z15",
                "meaning": "Haushaltskunde gem. EnWG"
            }
        }
    });
    let instance_enriched = engine.map_reverse(&bo4e_enriched, &def);
    assert_eq!(instance_enriched.segments[0].elements[2], vec!["Z15"]);
}
```

**Step 2: Modify `populate_field` to handle enriched objects**

In `crates/mig-bo4e/src/engine.rs`, modify `populate_field()` (line 615-627):

```rust
pub fn populate_field(
    &self,
    bo4e_value: &serde_json::Value,
    target_field: &str,
    _source_path: &str,
) -> Option<String> {
    let parts: Vec<&str> = target_field.split('.').collect();
    let mut current = bo4e_value;
    for part in &parts {
        current = current.get(part)?;
    }
    // Handle enriched code objects: {"code": "Z15", "meaning": "..."}
    if let Some(code) = current.get("code").and_then(|v| v.as_str()) {
        return Some(code.to_string());
    }
    current.as_str().map(|s| s.to_string())
}
```

**Step 3: Run tests**

Run: `cargo test -p mig-bo4e -- test_reverse_mapping_accepts_enriched_companion`
Expected: PASS

Run: `cargo test -p mig-bo4e`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs
git commit -m "feat(mig-bo4e): reverse mapping accepts enriched {code, meaning} objects"
```

---

### Task 4: Update integration tests for code enrichment

**Files:**
- Modify: `crates/mig-bo4e/tests/map_all_forward_test.rs` (companion assertions)
- Modify: `crates/mig-bo4e/tests/roundtrip_55001_test.rs` (if assertions check companion strings)

**Step 1: Update `map_all_forward_test.rs`**

The test at line 150-154 currently asserts:
```rust
assert_eq!(
    pp_companion.get("merkmalCode").and_then(|v| v.as_str()),
    Some("Z66"),
    ...
);
```

This will still pass when no code_lookup is loaded (the existing `load_engine()` doesn't load a schema). No changes needed to existing tests — they continue to test the non-enriched path.

**Step 2: Add a new integration test with code enrichment enabled**

Add a new test in `map_all_forward_test.rs` that loads the engine WITH a code lookup:

```rust
#[test]
fn test_map_all_forward_55001_with_code_enrichment() {
    // ... (same setup as test_map_all_forward_55001) ...
    // But additionally load the PID schema for code lookup

    let schema_path = Path::new("../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");
    if !schema_path.exists() {
        eprintln!("Skipping: PID schema not found");
        return;
    }

    // Load engine with code lookup
    let code_lookup = mig_bo4e::code_lookup::CodeLookup::from_schema_file(schema_path).unwrap();
    let engine = MappingEngine::load_merged(&[msg_dir, tx_dir])
        .unwrap()
        .with_code_lookup(code_lookup);

    let result = engine.map_all_forward(&tree);

    // Companion fields should now be enriched objects
    let malo = result.get("Marktlokation").unwrap();
    let malo_companion = malo.get("MarktlokationEdifact").unwrap();
    let hk = malo_companion.get("haushaltskunde").unwrap();
    assert!(hk.is_object(), "haushaltskunde should be an enriched object");
    assert!(hk.get("code").is_some(), "should have code field");
    assert!(hk.get("meaning").is_some(), "should have meaning field");

    let pp = result.get("Produktpaket").unwrap();
    let pp_companion = pp.get("ProduktpaketEdifact").unwrap();
    let mc = pp_companion.get("merkmalCode").unwrap();
    assert_eq!(mc.get("code").and_then(|v| v.as_str()), Some("Z66"));
    assert_eq!(
        mc.get("meaning").and_then(|v| v.as_str()),
        Some("Produkteigenschaft")
    );

    // Data-type fields should still be plain strings
    let pe = pp_companion.get("produkteigenschaftCode");
    if let Some(pe) = pe {
        assert!(pe.is_string(), "data-type companion field should remain a plain string");
    }
}
```

**Step 3: Run all tests**

Run: `cargo test -p mig-bo4e`
Expected: All tests pass including the new enrichment integration test.

**Step 4: Commit**

```bash
git add crates/mig-bo4e/tests/map_all_forward_test.rs
git commit -m "test(mig-bo4e): add integration test for code-enriched companion fields"
```

---

### Task 5: Lint, format, and final verification

**Step 1: Run full test suite**

Run: `cargo test -p mig-bo4e`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -p mig-bo4e -- -D warnings`
Expected: No warnings

**Step 3: Run fmt**

Run: `cargo fmt --all -- --check`
Expected: No formatting issues

**Step 4: Final commit if any fixups needed**

```bash
git add -A && git commit -m "chore(mig-bo4e): lint and format fixes"
```
