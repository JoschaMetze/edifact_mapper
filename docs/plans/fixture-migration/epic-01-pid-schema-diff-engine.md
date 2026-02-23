---
feature: fixture-migration
epic: 1
title: "PID Schema Diff Engine"
depends_on: []
estimated_tasks: 5
crate: automapper-generator
status: in_progress
---

# Epic 1: PID Schema Diff Engine

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Build a `schema_diff` module that compares two PID schema JSONs and produces a structured `PidSchemaDiff` containing added/removed/unchanged segments, changed codes, added/removed groups, restructured groups, and added/removed elements.

**Architecture:** The diff algorithm walks both PID schema JSON trees (as `Vec<SchemaGroup>`) in parallel. It matches groups by `(source_group, qualifier)` pairs, segments by `(tag, element layout)`, and elements/codes by `(index, sub_index)`. Output is a `PidSchemaDiff` struct that serializes to the JSON format defined in the design doc.

**Existing code:**
- `load_pid_schema()` at `crates/automapper-generator/src/codegen/pid_mapping_gen.rs` — loads PID schema JSON into `Vec<SchemaGroup>`
- `SchemaGroup`, `SchemaSegmentInfo`, `SchemaElementInfo`, `SchemaCodeInfo`, `SchemaComponentInfo` — all in `pid_mapping_gen.rs`

---

## Task 1: Define `PidSchemaDiff` Types

**Files:**
- Create: `crates/automapper-generator/src/schema_diff/mod.rs`
- Create: `crates/automapper-generator/src/schema_diff/types.rs`
- Modify: `crates/automapper-generator/src/lib.rs` — add `pub mod schema_diff;`

**Step 1: Write the types**

Create the module structure and diff output types.

`crates/automapper-generator/src/schema_diff/mod.rs`:
```rust
pub mod types;
pub mod differ;

pub use types::*;
pub use differ::*;
```

`crates/automapper-generator/src/schema_diff/types.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidSchemaDiff {
    pub old_version: String,
    pub new_version: String,
    pub message_type: String,
    pub pid: String,
    pub unh_version: Option<VersionChange>,
    pub segments: SegmentDiff,
    pub codes: CodeDiff,
    pub groups: GroupDiff,
    pub elements: ElementDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDiff {
    pub added: Vec<SegmentEntry>,
    pub removed: Vec<SegmentEntry>,
    pub unchanged: Vec<SegmentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEntry {
    pub group: String,
    pub tag: String,
    /// Human-readable context (e.g., "New metering segment in SG8_Z98")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiff {
    pub changed: Vec<CodeChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub segment: String,
    pub element: String,
    pub group: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDiff {
    pub added: Vec<GroupEntry>,
    pub removed: Vec<GroupEntry>,
    pub restructured: Vec<RestructuredGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupEntry {
    pub group: String,
    pub parent: String,
    /// Entry segment with qualifier, e.g., "SEQ+ZH5"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_segment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestructuredGroup {
    pub group: String,
    pub description: String,
    pub manual_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDiff {
    pub added: Vec<ElementChange>,
    pub removed: Vec<ElementChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementChange {
    pub segment: String,
    pub group: String,
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PidSchemaDiff {
    /// Returns true if the diff contains no changes.
    pub fn is_empty(&self) -> bool {
        self.segments.added.is_empty()
            && self.segments.removed.is_empty()
            && self.codes.changed.is_empty()
            && self.groups.added.is_empty()
            && self.groups.removed.is_empty()
            && self.groups.restructured.is_empty()
            && self.elements.added.is_empty()
            && self.elements.removed.is_empty()
    }
}
```

**Step 2: Add the module to lib.rs**

In `crates/automapper-generator/src/lib.rs`, add:
```rust
pub mod schema_diff;
```

**Step 3: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: PASS (types only, no logic yet)

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/schema_diff/ crates/automapper-generator/src/lib.rs
git commit -m "feat(generator): add PidSchemaDiff types for schema comparison"
```

---

## Task 2: Group-Level Diffing — Test and Implementation

**Files:**
- Create: `crates/automapper-generator/src/schema_diff/differ.rs`
- Create: `crates/automapper-generator/tests/schema_diff_test.rs`

**Step 1: Write the failing test**

`crates/automapper-generator/tests/schema_diff_test.rs`:
```rust
//! Tests for PID schema diffing.

use automapper_generator::schema_diff::{diff_pid_schemas, DiffInput, PidSchemaDiff};

fn minimal_schema_json(groups: &[(&str, &str, &str)]) -> serde_json::Value {
    // Build a minimal PID schema JSON with specified groups.
    // Each tuple is (field_name, source_group, discriminator_segment:qualifier).
    let mut fields = serde_json::Map::new();
    for (field_name, source_group, disc) in groups {
        let parts: Vec<&str> = disc.split(':').collect();
        let disc_obj = if parts.len() == 2 {
            serde_json::json!({
                "segment": parts[0],
                "element": "3227",
                "values": [parts[1]]
            })
        } else {
            serde_json::Value::Null
        };

        fields.insert(
            field_name.to_string(),
            serde_json::json!({
                "source_group": source_group,
                "discriminator": disc_obj,
                "segments": [],
                "children": null
            }),
        );
    }

    serde_json::json!({
        "pid": "55001",
        "beschreibung": "Test",
        "format_version": "FV2504",
        "fields": fields
    })
}

#[test]
fn test_diff_identical_schemas_has_no_group_changes() {
    let schema = minimal_schema_json(&[
        ("sg5_z16", "SG5", "LOC:Z16"),
        ("sg8_z98", "SG8", "SEQ:Z98"),
    ]);

    let input = DiffInput {
        old_schema: schema.clone(),
        new_schema: schema,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert!(diff.groups.added.is_empty());
    assert!(diff.groups.removed.is_empty());
    assert!(diff.groups.restructured.is_empty());
}

#[test]
fn test_diff_detects_added_group() {
    let old = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16")]);
    let new = minimal_schema_json(&[
        ("sg5_z16", "SG5", "LOC:Z16"),
        ("sg8_zh5", "SG8", "SEQ:ZH5"),
    ]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.groups.added.len(), 1);
    assert_eq!(diff.groups.added[0].group, "sg8_zh5");
}

#[test]
fn test_diff_detects_removed_group() {
    let old = minimal_schema_json(&[
        ("sg5_z16", "SG5", "LOC:Z16"),
        ("sg8_z98", "SG8", "SEQ:Z98"),
    ]);
    let new = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16")]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.groups.removed.len(), 1);
    assert_eq!(diff.groups.removed[0].group, "sg8_z98");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_diff_identical`
Expected: FAIL — `diff_pid_schemas` and `DiffInput` don't exist yet.

**Step 3: Write the implementation**

`crates/automapper-generator/src/schema_diff/differ.rs`:
```rust
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Input for the PID schema diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffInput {
    pub old_schema: serde_json::Value,
    pub new_schema: serde_json::Value,
    pub old_version: String,
    pub new_version: String,
    pub message_type: String,
    pub pid: String,
}

/// Compare two PID schema JSONs and produce a structured diff.
pub fn diff_pid_schemas(input: &DiffInput) -> PidSchemaDiff {
    let old_fields = extract_fields(&input.old_schema);
    let new_fields = extract_fields(&input.new_schema);

    let groups = diff_groups(&old_fields, &new_fields);
    let segments = diff_segments(&old_fields, &new_fields);
    let codes = diff_codes(&old_fields, &new_fields);
    let elements = diff_elements(&old_fields, &new_fields);

    PidSchemaDiff {
        old_version: input.old_version.clone(),
        new_version: input.new_version.clone(),
        message_type: input.message_type.clone(),
        pid: input.pid.clone(),
        unh_version: None, // Populated from MIG metadata, not schema JSON
        segments,
        codes,
        groups,
        elements,
    }
}

/// A flattened representation of a schema group for diffing.
#[derive(Debug, Clone)]
struct FlatGroup {
    field_name: String,
    source_group: String,
    qualifier: Option<String>,
    disc_segment: Option<String>,
    parent: Option<String>,
    segments: Vec<FlatSegment>,
}

#[derive(Debug, Clone)]
struct FlatSegment {
    tag: String,
    elements: Vec<FlatElement>,
}

#[derive(Debug, Clone)]
struct FlatElement {
    index: usize,
    id: String,
    element_type: String,
    codes: Vec<String>,
    components: Vec<FlatComponent>,
}

#[derive(Debug, Clone)]
struct FlatComponent {
    sub_index: usize,
    id: String,
    element_type: String,
    codes: Vec<String>,
}

/// Extract all groups from PID schema JSON into a flat map keyed by field_name.
fn extract_fields(schema: &serde_json::Value) -> BTreeMap<String, FlatGroup> {
    let mut result = BTreeMap::new();
    if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
        flatten_groups(fields, None, &mut result);
    }
    result
}

fn flatten_groups(
    fields: &serde_json::Map<String, serde_json::Value>,
    parent: Option<&str>,
    result: &mut BTreeMap<String, FlatGroup>,
) {
    for (field_name, field_value) in fields {
        let source_group = field_value
            .get("source_group")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let (disc_segment, qualifier) = extract_discriminator(field_value);
        let segments = extract_segments(field_value);

        result.insert(
            field_name.clone(),
            FlatGroup {
                field_name: field_name.clone(),
                source_group,
                qualifier,
                disc_segment,
                parent: parent.map(String::from),
                segments,
            },
        );

        // Recurse into children
        if let Some(children) = field_value.get("children").and_then(|c| c.as_object()) {
            flatten_groups(children, Some(field_name), result);
        }
    }
}

fn extract_discriminator(field: &serde_json::Value) -> (Option<String>, Option<String>) {
    if let Some(disc) = field.get("discriminator") {
        if disc.is_null() {
            return (None, None);
        }
        let segment = disc.get("segment").and_then(|v| v.as_str()).map(String::from);
        let values = disc
            .get("values")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(String::from);
        (segment, values)
    } else {
        (None, None)
    }
}

fn extract_segments(field: &serde_json::Value) -> Vec<FlatSegment> {
    let Some(segments) = field.get("segments").and_then(|s| s.as_array()) else {
        return vec![];
    };

    segments
        .iter()
        .filter_map(|seg| {
            let tag = seg.get("id").and_then(|v| v.as_str())?.to_string();
            let elements = seg
                .get("elements")
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|el| {
                            let index = el.get("index").and_then(|v| v.as_u64())? as usize;
                            let id = el.get("id").and_then(|v| v.as_str())?.to_string();
                            let element_type = el
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("data")
                                .to_string();
                            let codes = extract_code_values(el);
                            let components = extract_components(el);
                            Some(FlatElement {
                                index,
                                id,
                                element_type,
                                codes,
                                components,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(FlatSegment { tag, elements })
        })
        .collect()
}

fn extract_code_values(element: &serde_json::Value) -> Vec<String> {
    element
        .get("codes")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("value").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_components(element: &serde_json::Value) -> Vec<FlatComponent> {
    element
        .get("components")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|comp| {
                    let sub_index =
                        comp.get("sub_index").and_then(|v| v.as_u64())? as usize;
                    let id = comp.get("id").and_then(|v| v.as_str())?.to_string();
                    let element_type = comp
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("data")
                        .to_string();
                    let codes = extract_code_values(comp);
                    Some(FlatComponent {
                        sub_index,
                        id,
                        element_type,
                        codes,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn diff_groups(
    old: &BTreeMap<String, FlatGroup>,
    new: &BTreeMap<String, FlatGroup>,
) -> GroupDiff {
    let old_keys: BTreeSet<&String> = old.keys().collect();
    let new_keys: BTreeSet<&String> = new.keys().collect();

    let added: Vec<GroupEntry> = new_keys
        .difference(&old_keys)
        .map(|k| {
            let g = &new[*k];
            let entry_seg = g.disc_segment.as_ref().map(|seg| {
                if let Some(ref q) = g.qualifier {
                    format!("{}+{}", seg, q)
                } else {
                    seg.clone()
                }
            });
            GroupEntry {
                group: k.to_string(),
                parent: g.parent.clone().unwrap_or_else(|| "root".to_string()),
                entry_segment: entry_seg,
            }
        })
        .collect();

    let removed: Vec<GroupEntry> = old_keys
        .difference(&new_keys)
        .map(|k| {
            let g = &old[*k];
            GroupEntry {
                group: k.to_string(),
                parent: g.parent.clone().unwrap_or_else(|| "root".to_string()),
                entry_segment: None,
            }
        })
        .collect();

    // Detect restructured: same source_group+qualifier but different parent
    let mut restructured = Vec::new();
    for key in old_keys.intersection(&new_keys) {
        let old_g = &old[*key];
        let new_g = &new[*key];
        if old_g.parent != new_g.parent {
            restructured.push(RestructuredGroup {
                group: key.to_string(),
                description: format!(
                    "Parent changed from {:?} to {:?}",
                    old_g.parent, new_g.parent
                ),
                manual_review: true,
            });
        }
    }

    GroupDiff {
        added,
        removed,
        restructured,
    }
}

fn diff_segments(
    old: &BTreeMap<String, FlatGroup>,
    new: &BTreeMap<String, FlatGroup>,
) -> SegmentDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut unchanged = Vec::new();

    let all_keys: BTreeSet<&String> = old.keys().chain(new.keys()).collect();

    for key in &all_keys {
        let old_group = old.get(*key);
        let new_group = new.get(*key);

        match (old_group, new_group) {
            (Some(og), Some(ng)) => {
                // Group exists in both — diff segments within it
                let old_tags: BTreeSet<&str> =
                    og.segments.iter().map(|s| s.tag.as_str()).collect();
                let new_tags: BTreeSet<&str> =
                    ng.segments.iter().map(|s| s.tag.as_str()).collect();

                for tag in old_tags.intersection(&new_tags) {
                    unchanged.push(SegmentEntry {
                        group: key.to_string(),
                        tag: tag.to_string(),
                        context: None,
                    });
                }
                for tag in new_tags.difference(&old_tags) {
                    added.push(SegmentEntry {
                        group: key.to_string(),
                        tag: tag.to_string(),
                        context: Some(format!("New segment in {}", key)),
                    });
                }
                for tag in old_tags.difference(&new_tags) {
                    removed.push(SegmentEntry {
                        group: key.to_string(),
                        tag: tag.to_string(),
                        context: Some(format!("Removed from {}", key)),
                    });
                }
            }
            (None, Some(ng)) => {
                // Entire group is new — all segments are added
                for seg in &ng.segments {
                    added.push(SegmentEntry {
                        group: key.to_string(),
                        tag: seg.tag.clone(),
                        context: Some(format!("New group {}", key)),
                    });
                }
            }
            (Some(og), None) => {
                // Entire group removed — all segments are removed
                for seg in &og.segments {
                    removed.push(SegmentEntry {
                        group: key.to_string(),
                        tag: seg.tag.clone(),
                        context: Some(format!("Removed group {}", key)),
                    });
                }
            }
            (None, None) => unreachable!(),
        }
    }

    SegmentDiff {
        added,
        removed,
        unchanged,
    }
}

fn diff_codes(
    old: &BTreeMap<String, FlatGroup>,
    new: &BTreeMap<String, FlatGroup>,
) -> CodeDiff {
    let mut changed = Vec::new();

    for (key, new_group) in new {
        let Some(old_group) = old.get(key) else {
            continue; // New group — codes already captured in segments.added
        };

        // Match segments by tag
        for new_seg in &new_group.segments {
            let Some(old_seg) = old_group.segments.iter().find(|s| s.tag == new_seg.tag) else {
                continue;
            };

            // Compare elements by index
            for new_el in &new_seg.elements {
                let old_el = old_seg.elements.iter().find(|e| e.index == new_el.index);

                if let Some(old_el) = old_el {
                    // Compare top-level codes
                    let old_codes: BTreeSet<&str> =
                        old_el.codes.iter().map(|s| s.as_str()).collect();
                    let new_codes: BTreeSet<&str> =
                        new_el.codes.iter().map(|s| s.as_str()).collect();

                    let added_codes: Vec<String> = new_codes
                        .difference(&old_codes)
                        .map(|s| s.to_string())
                        .collect();
                    let removed_codes: Vec<String> = old_codes
                        .difference(&new_codes)
                        .map(|s| s.to_string())
                        .collect();

                    if !added_codes.is_empty() || !removed_codes.is_empty() {
                        changed.push(CodeChange {
                            segment: new_seg.tag.clone(),
                            element: new_el.index.to_string(),
                            group: key.clone(),
                            added: added_codes,
                            removed: removed_codes,
                            context: None,
                        });
                    }

                    // Compare component codes
                    for new_comp in &new_el.components {
                        let old_comp = old_el
                            .components
                            .iter()
                            .find(|c| c.sub_index == new_comp.sub_index);
                        if let Some(old_comp) = old_comp {
                            let oc: BTreeSet<&str> =
                                old_comp.codes.iter().map(|s| s.as_str()).collect();
                            let nc: BTreeSet<&str> =
                                new_comp.codes.iter().map(|s| s.as_str()).collect();
                            let ac: Vec<String> =
                                nc.difference(&oc).map(|s| s.to_string()).collect();
                            let rc: Vec<String> =
                                oc.difference(&nc).map(|s| s.to_string()).collect();
                            if !ac.is_empty() || !rc.is_empty() {
                                changed.push(CodeChange {
                                    segment: new_seg.tag.clone(),
                                    element: format!("{}.{}", new_el.index, new_comp.sub_index),
                                    group: key.clone(),
                                    added: ac,
                                    removed: rc,
                                    context: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    CodeDiff { changed }
}

fn diff_elements(
    old: &BTreeMap<String, FlatGroup>,
    new: &BTreeMap<String, FlatGroup>,
) -> ElementDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for (key, new_group) in new {
        let Some(old_group) = old.get(key) else {
            continue;
        };

        for new_seg in &new_group.segments {
            let Some(old_seg) = old_group.segments.iter().find(|s| s.tag == new_seg.tag) else {
                continue;
            };

            let old_indices: BTreeSet<usize> = old_seg.elements.iter().map(|e| e.index).collect();
            let new_indices: BTreeSet<usize> = new_seg.elements.iter().map(|e| e.index).collect();

            for idx in new_indices.difference(&old_indices) {
                let el = new_seg.elements.iter().find(|e| e.index == *idx).unwrap();
                added.push(ElementChange {
                    segment: new_seg.tag.clone(),
                    group: key.clone(),
                    index: *idx,
                    sub_index: None,
                    description: Some(format!("New element {} ({})", el.id, el.element_type)),
                });
            }
            for idx in old_indices.difference(&new_indices) {
                let el = old_seg.elements.iter().find(|e| e.index == *idx).unwrap();
                removed.push(ElementChange {
                    segment: new_seg.tag.clone(),
                    group: key.clone(),
                    index: *idx,
                    sub_index: None,
                    description: Some(format!("Removed element {}", el.id)),
                });
            }

            // Compare components within matching elements
            for new_el in &new_seg.elements {
                let Some(old_el) = old_seg.elements.iter().find(|e| e.index == new_el.index)
                else {
                    continue;
                };
                let old_subs: BTreeSet<usize> =
                    old_el.components.iter().map(|c| c.sub_index).collect();
                let new_subs: BTreeSet<usize> =
                    new_el.components.iter().map(|c| c.sub_index).collect();

                for si in new_subs.difference(&old_subs) {
                    added.push(ElementChange {
                        segment: new_seg.tag.clone(),
                        group: key.clone(),
                        index: new_el.index,
                        sub_index: Some(*si),
                        description: Some("New component".to_string()),
                    });
                }
                for si in old_subs.difference(&new_subs) {
                    removed.push(ElementChange {
                        segment: new_seg.tag.clone(),
                        group: key.clone(),
                        index: new_el.index,
                        sub_index: Some(*si),
                        description: Some("Removed component".to_string()),
                    });
                }
            }
        }
    }

    ElementDiff { added, removed }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p automapper-generator test_diff_`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/schema_diff/ crates/automapper-generator/tests/schema_diff_test.rs
git commit -m "feat(generator): add PID schema diff engine with group-level diffing"
```

---

## Task 3: Segment-Level and Code-Level Diff Tests

**Files:**
- Modify: `crates/automapper-generator/tests/schema_diff_test.rs`

**Step 1: Write the failing tests**

Add to `schema_diff_test.rs`:
```rust
fn schema_with_segments(
    groups: &[(&str, &str, &str, &[(&str, &[(usize, &str, &str, &[&str])])])],
) -> serde_json::Value {
    // groups: [(field_name, source_group, disc, segments)]
    // segments: [(tag, elements)]
    // elements: [(index, id, type, codes)]
    let mut fields = serde_json::Map::new();
    for (field_name, source_group, disc, segments) in groups {
        let parts: Vec<&str> = disc.split(':').collect();
        let disc_obj = if parts.len() == 2 {
            serde_json::json!({
                "segment": parts[0],
                "element": "3227",
                "values": [parts[1]]
            })
        } else {
            serde_json::Value::Null
        };

        let segs: Vec<serde_json::Value> = segments
            .iter()
            .map(|(tag, elems)| {
                let elements: Vec<serde_json::Value> = elems
                    .iter()
                    .map(|(idx, id, etype, codes)| {
                        let code_arr: Vec<serde_json::Value> = codes
                            .iter()
                            .map(|c| serde_json::json!({"value": c, "name": c}))
                            .collect();
                        serde_json::json!({
                            "index": idx,
                            "id": id,
                            "type": etype,
                            "codes": code_arr,
                            "components": []
                        })
                    })
                    .collect();
                serde_json::json!({"id": tag, "elements": elements})
            })
            .collect();

        fields.insert(
            field_name.to_string(),
            serde_json::json!({
                "source_group": source_group,
                "discriminator": disc_obj,
                "segments": segs,
                "children": null
            }),
        );
    }

    serde_json::json!({
        "pid": "55001",
        "beschreibung": "Test",
        "format_version": "FV2504",
        "fields": fields
    })
}

#[test]
fn test_diff_detects_added_segment_within_group() {
    let old = schema_with_segments(&[(
        "sg5_z16",
        "SG5",
        "LOC:Z16",
        &[("LOC", &[(0, "3227", "code", &["Z16"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg5_z16",
        "SG5",
        "LOC:Z16",
        &[
            ("LOC", &[(0, "3227", "code", &["Z16"])]),
            ("MEA", &[(0, "6311", "code", &["AAA"])]),
        ],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.segments.added.len(), 1);
    assert_eq!(diff.segments.added[0].tag, "MEA");
    assert_eq!(diff.segments.unchanged.len(), 1);
    assert_eq!(diff.segments.unchanged[0].tag, "LOC");
}

#[test]
fn test_diff_detects_code_change() {
    let old = schema_with_segments(&[(
        "sg10",
        "SG10",
        "CCI:Z66",
        &[("CCI", &[(0, "7059", "code", &["Z66", "Z88"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg10",
        "SG10",
        "CCI:Z66",
        &[("CCI", &[(0, "7059", "code", &["Z66", "Z95"])])],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.codes.changed.len(), 1);
    assert_eq!(diff.codes.changed[0].added, vec!["Z95"]);
    assert_eq!(diff.codes.changed[0].removed, vec!["Z88"]);
}

#[test]
fn test_diff_detects_added_element() {
    let old = schema_with_segments(&[(
        "sg4",
        "SG4",
        ":",
        &[("STS", &[(0, "9015", "code", &["7"]), (2, "9013", "code", &["E01"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg4",
        "SG4",
        ":",
        &[(
            "STS",
            &[
                (0, "9015", "code", &["7"]),
                (2, "9013", "code", &["E01"]),
                (4, "9013b", "code", &["E03"]),
            ],
        )],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.elements.added.len(), 1);
    assert_eq!(diff.elements.added[0].index, 4);
    assert_eq!(diff.elements.added[0].segment, "STS");
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test -p automapper-generator test_diff_`
Expected: 6 tests PASS (3 new + 3 from Task 2)

**Step 3: Commit**

```bash
git add crates/automapper-generator/tests/schema_diff_test.rs
git commit -m "test(generator): add segment-level and code-level schema diff tests"
```

---

## Task 4: Diff with Real PID Schemas (Integration Test)

**Files:**
- Modify: `crates/automapper-generator/tests/schema_diff_test.rs`

**Step 1: Write the integration test**

Add to `schema_diff_test.rs`:
```rust
use std::path::Path;

#[test]
fn test_diff_real_55001_against_itself() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");

    if !schema_path.exists() {
        eprintln!("Skipping: schema not found at {:?}", schema_path);
        return;
    }

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();

    let input = DiffInput {
        old_schema: schema.clone(),
        new_schema: schema,
        old_version: "FV2504".into(),
        new_version: "FV2504".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert!(
        diff.is_empty(),
        "Diffing a schema against itself should produce no changes, got: {} added groups, {} removed groups, {} code changes, {} added segments, {} removed segments",
        diff.groups.added.len(),
        diff.groups.removed.len(),
        diff.codes.changed.len(),
        diff.segments.added.len(),
        diff.segments.removed.len(),
    );
}

#[test]
fn test_diff_55001_vs_55002_shows_differences() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("crates/mig-types/src/generated/fv2504/utilmd/pids");

    let schema_55001_path = base.join("pid_55001_schema.json");
    let schema_55002_path = base.join("pid_55002_schema.json");

    if !schema_55001_path.exists() || !schema_55002_path.exists() {
        eprintln!("Skipping: schemas not found");
        return;
    }

    let schema_55001: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_55001_path).unwrap()).unwrap();
    let schema_55002: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_55002_path).unwrap()).unwrap();

    let input = DiffInput {
        old_schema: schema_55001,
        new_schema: schema_55002,
        old_version: "FV2504".into(),
        new_version: "FV2504".into(),
        message_type: "UTILMD".into(),
        pid: "55001-vs-55002".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert!(
        !diff.is_empty(),
        "55001 and 55002 should have structural differences"
    );

    // 55002 has more LOC groups (Z17, Z18, Z19, Z20) that 55001 doesn't
    assert!(
        !diff.groups.added.is_empty(),
        "55002 should have groups not in 55001"
    );

    // Print diff summary for manual inspection
    eprintln!("Groups added: {:?}", diff.groups.added.iter().map(|g| &g.group).collect::<Vec<_>>());
    eprintln!("Groups removed: {:?}", diff.groups.removed.iter().map(|g| &g.group).collect::<Vec<_>>());
    eprintln!("Code changes: {}", diff.codes.changed.len());
}
```

**Step 2: Run tests**

Run: `cargo test -p automapper-generator test_diff_real -- --nocapture`
Expected: PASS with diagnostic output showing the cross-PID diff.

**Step 3: Commit**

```bash
git add crates/automapper-generator/tests/schema_diff_test.rs
git commit -m "test(generator): add integration tests diffing real PID schemas"
```

---

## Task 5: JSON Serialization Output

**Files:**
- Modify: `crates/automapper-generator/tests/schema_diff_test.rs`

**Step 1: Write a serialization test**

Add to `schema_diff_test.rs`:
```rust
#[test]
fn test_diff_serializes_to_json() {
    let old = schema_with_segments(&[(
        "sg5_z16",
        "SG5",
        "LOC:Z16",
        &[("LOC", &[(0, "3227", "code", &["Z16"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg5_z16",
        "SG5",
        "LOC:Z16",
        &[
            ("LOC", &[(0, "3227", "code", &["Z16"])]),
            ("MEA", &[(0, "6311", "code", &["AAA"])]),
        ],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    let json = serde_json::to_string_pretty(&diff).unwrap();
    assert!(json.contains("\"old_version\": \"FV2504\""));
    assert!(json.contains("\"new_version\": \"FV2510\""));
    assert!(json.contains("\"tag\": \"MEA\""));

    // Verify round-trip: deserialize back
    let parsed: PidSchemaDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.segments.added.len(), 1);
}
```

**Step 2: Run all diff tests**

Run: `cargo test -p automapper-generator test_diff_ -- --nocapture`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add crates/automapper-generator/tests/schema_diff_test.rs
git commit -m "test(generator): add JSON serialization roundtrip test for PidSchemaDiff"
```
