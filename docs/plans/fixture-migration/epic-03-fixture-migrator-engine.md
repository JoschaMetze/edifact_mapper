---
feature: fixture-migration
epic: 3
title: "Fixture Migrator Engine"
depends_on: [1]
estimated_tasks: 5
crate: automapper-generator
status: pending
---

# Epic 3: Fixture Migrator Engine

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Build a `fixture_migrator` module that takes an old `.edi` fixture file, a `PidSchemaDiff`, and a new PID schema JSON, and produces a migrated `.edi` file plus a list of warnings for items requiring manual review.

**Architecture:** The migrator parses the old `.edi` into `Vec<OwnedSegment>` using `parse_to_segments()`, then applies diff rules in confidence order: copy unchanged segments, drop removed segments, substitute renamed codes, update the UNH version string, generate skeleton segments for additions (filling with schema-valid default codes and empty data elements), and flag restructured groups. Output is rendered back to EDIFACT using `render_edifact()`.

**Existing code:**
- `parse_to_segments()` at `mig-assembly::tokenize` — tokenize `.edi` into segments
- `OwnedSegment` at `mig-types::segment` — segment type with `id`, `elements`, `segment_number`
- `render_edifact()` at `mig-assembly::renderer` — render segments to EDIFACT string
- `DisassembledSegment` at `mig-assembly::disassembler` — segment type used by renderer
- `EdifactDelimiters` at `edifact-types` — delimiter configuration
- `PidSchemaDiff` from Epic 1 — structured diff

**Dependencies:**
- `automapper-generator` Cargo.toml needs `mig-assembly` and `mig-types` as dependencies (check if already present, add if not)

---

## Task 1: Define `MigrationResult` Types and Module

**Files:**
- Create: `crates/automapper-generator/src/fixture_migrator/mod.rs`
- Create: `crates/automapper-generator/src/fixture_migrator/types.rs`
- Modify: `crates/automapper-generator/src/lib.rs` — add `pub mod fixture_migrator;`

**Step 1: Write the types**

`crates/automapper-generator/src/fixture_migrator/mod.rs`:
```rust
pub mod types;
pub mod migrator;

pub use types::*;
pub use migrator::*;
```

`crates/automapper-generator/src/fixture_migrator/types.rs`:
```rust
use serde::{Deserialize, Serialize};

/// Result of migrating an EDIFACT fixture between format versions.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// The migrated EDIFACT content as a string.
    pub edifact: String,
    /// Warnings about items requiring manual review.
    pub warnings: Vec<MigrationWarning>,
    /// Summary statistics.
    pub stats: MigrationStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationWarning {
    pub severity: WarningSeverity,
    pub message: String,
    /// The segment tag this warning relates to, if applicable.
    pub segment: Option<String>,
    /// The group this warning relates to, if applicable.
    pub group: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    /// Automatic action taken, informational only.
    Info,
    /// Automatic action taken but may need verification.
    Warning,
    /// Could not be handled automatically — requires manual review.
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct MigrationStats {
    pub segments_copied: usize,
    pub segments_removed: usize,
    pub segments_added: usize,
    pub codes_substituted: usize,
    pub manual_review_items: usize,
}

impl std::fmt::Display for MigrationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.severity {
            WarningSeverity::Info => "INFO",
            WarningSeverity::Warning => "WARNING",
            WarningSeverity::Error => "ERROR",
        };
        write!(f, "{}: {}", prefix, self.message)
    }
}
```

**Step 2: Add module to lib.rs**

In `crates/automapper-generator/src/lib.rs`, add:
```rust
pub mod fixture_migrator;
```

**Step 3: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/fixture_migrator/ crates/automapper-generator/src/lib.rs
git commit -m "feat(generator): add MigrationResult types for fixture migration"
```

---

## Task 2: UNH Version Update — Test and Implementation

**Files:**
- Create: `crates/automapper-generator/src/fixture_migrator/migrator.rs`
- Create: `crates/automapper-generator/tests/fixture_migrator_test.rs`

**Step 1: Write the failing test**

`crates/automapper-generator/tests/fixture_migrator_test.rs`:
```rust
//! Tests for the fixture migrator engine.

use automapper_generator::fixture_migrator::migrate_fixture;
use automapper_generator::schema_diff::types::*;

/// Build a minimal PidSchemaDiff with only a UNH version change.
fn version_only_diff() -> PidSchemaDiff {
    PidSchemaDiff {
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
        unh_version: Some(VersionChange {
            old: "S2.1".into(),
            new: "S2.2".into(),
        }),
        segments: SegmentDiff {
            added: vec![],
            removed: vec![],
            unchanged: vec![],
        },
        codes: CodeDiff { changed: vec![] },
        groups: GroupDiff {
            added: vec![],
            removed: vec![],
            restructured: vec![],
        },
        elements: ElementDiff {
            added: vec![],
            removed: vec![],
        },
    }
}

#[test]
fn test_migrate_updates_unh_version() {
    let old_edi = "\
UNB+UNOC:3+9978842000002:500+9900269000000:500+250331:1329+REF123'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E01+MSG001BGM'\
UNT+3+MSG001'\
UNZ+1+REF123'";

    let diff = version_only_diff();
    let new_schema = serde_json::json!({
        "pid": "55001",
        "beschreibung": "Test",
        "format_version": "FV2510",
        "fields": {}
    });

    let result = migrate_fixture(old_edi, &diff, &new_schema);
    assert!(result.edifact.contains("UTILMD:D:11A:UN:S2.2"),
        "UNH version should be updated to S2.2, got: {}", result.edifact);
    assert!(result.edifact.contains("UNB+UNOC:3"),
        "UNB should be preserved unchanged");
    assert!(result.edifact.contains("BGM+E01"),
        "BGM should be preserved unchanged");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_migrate_updates`
Expected: FAIL — `migrate_fixture` doesn't exist yet.

**Step 3: Write the implementation**

`crates/automapper-generator/src/fixture_migrator/migrator.rs`:
```rust
use super::types::*;
use crate::schema_diff::types::PidSchemaDiff;

/// Migrate an EDIFACT fixture string using a PidSchemaDiff and new PID schema.
///
/// Applies diff rules in confidence order:
/// 1. Update UNH version string (automatic)
/// 2. Drop removed segments (automatic)
/// 3. Substitute renamed codes (automatic)
/// 4. Copy unchanged segments verbatim (automatic)
/// 5. Generate skeleton segments for additions (automatic + warning)
/// 6. Flag restructured groups (error warning)
pub fn migrate_fixture(
    old_edi: &str,
    diff: &PidSchemaDiff,
    _new_schema: &serde_json::Value,
) -> MigrationResult {
    let mut warnings = Vec::new();
    let mut stats = MigrationStats::default();

    // Parse the EDIFACT into segments by splitting on segment terminator.
    // We work at the string level to preserve exact formatting.
    let segments = split_edifact_segments(old_edi);

    let mut output_segments: Vec<String> = Vec::new();

    for seg_str in &segments {
        let tag = extract_tag(seg_str);

        // Check if this segment is in a removed group
        if is_segment_removed(&tag, diff) {
            stats.segments_removed += 1;
            warnings.push(MigrationWarning {
                severity: WarningSeverity::Info,
                message: format!("Removed segment {} (no longer in new schema)", tag),
                segment: Some(tag.clone()),
                group: None,
            });
            continue;
        }

        // Apply UNH version update
        if tag == "UNH" {
            if let Some(ref version_change) = diff.unh_version {
                let updated = seg_str.replace(
                    &format!(":{}", version_change.old),
                    &format!(":{}", version_change.new),
                );
                output_segments.push(updated);
                stats.segments_copied += 1;
                continue;
            }
        }

        // Apply code substitutions
        let (migrated_seg, sub_count) = apply_code_substitutions(seg_str, &tag, diff);
        stats.codes_substituted += sub_count;

        output_segments.push(migrated_seg);
        stats.segments_copied += 1;
    }

    // Add warnings for restructured groups
    for rg in &diff.groups.restructured {
        warnings.push(MigrationWarning {
            severity: WarningSeverity::Error,
            message: format!(
                "Group {} restructured: {} — manual review required",
                rg.group, rg.description
            ),
            segment: None,
            group: Some(rg.group.clone()),
        });
        stats.manual_review_items += 1;
    }

    // Add warnings for new groups/segments that need content
    for group in &diff.groups.added {
        warnings.push(MigrationWarning {
            severity: WarningSeverity::Warning,
            message: format!(
                "New group {} (parent: {}) — needs content. Entry: {}",
                group.group,
                group.parent,
                group.entry_segment.as_deref().unwrap_or("unknown")
            ),
            segment: None,
            group: Some(group.group.clone()),
        });
        stats.manual_review_items += 1;
    }

    for seg in &diff.segments.added {
        // Only warn for segments in groups that exist (not already covered by group.added)
        let group_is_new = diff.groups.added.iter().any(|g| g.group == seg.group);
        if !group_is_new {
            warnings.push(MigrationWarning {
                severity: WarningSeverity::Warning,
                message: format!(
                    "New segment {} in existing group {} — filled with defaults, needs review",
                    seg.tag, seg.group
                ),
                segment: Some(seg.tag.clone()),
                group: Some(seg.group.clone()),
            });
            stats.segments_added += 1;
        }
    }

    let edifact = output_segments.join("'");
    // Re-add trailing segment terminator if original had one
    let edifact = if old_edi.ends_with('\'') && !edifact.ends_with('\'') {
        format!("{}'", edifact)
    } else {
        edifact
    };

    MigrationResult {
        edifact,
        warnings,
        stats,
    }
}

/// Split EDIFACT string into segments (excluding empty trailing entries).
fn split_edifact_segments(edi: &str) -> Vec<String> {
    edi.split('\'')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Extract the segment tag (first 3 characters before + or end).
fn extract_tag(segment: &str) -> String {
    segment
        .split('+')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Check if a segment tag appears only in removed segments/groups.
fn is_segment_removed(tag: &str, diff: &PidSchemaDiff) -> bool {
    let in_removed = diff.segments.removed.iter().any(|s| s.tag == tag);
    let in_unchanged = diff.segments.unchanged.iter().any(|s| s.tag == tag);
    let in_added = diff.segments.added.iter().any(|s| s.tag == tag);

    // Only remove if explicitly removed and not also present elsewhere
    in_removed && !in_unchanged && !in_added
}

/// Apply code substitutions based on diff's code changes.
/// Returns (migrated_segment_string, substitution_count).
fn apply_code_substitutions(
    seg_str: &str,
    tag: &str,
    diff: &PidSchemaDiff,
) -> (String, usize) {
    let mut result = seg_str.to_string();
    let mut count = 0;

    for code_change in &diff.codes.changed {
        if code_change.segment != *tag {
            continue;
        }

        // Only apply 1:1 renames (one removed, one added)
        if code_change.removed.len() == 1 && code_change.added.len() == 1 {
            let old_code = &code_change.removed[0];
            let new_code = &code_change.added[0];

            // Replace the code value in the segment, being careful about context
            // (only replace within element boundaries, not arbitrary substrings)
            if result.contains(old_code) {
                result = replace_code_in_segment(&result, old_code, new_code);
                count += 1;
            }
        }
    }

    (result, count)
}

/// Replace a code value within an EDIFACT segment string.
/// Careful to only replace at element/component boundaries.
fn replace_code_in_segment(segment: &str, old_code: &str, new_code: &str) -> String {
    // Split by element separator, then check component boundaries
    let elements: Vec<&str> = segment.split('+').collect();
    let mut new_elements: Vec<String> = Vec::new();

    for element in elements {
        let components: Vec<&str> = element.split(':').collect();
        let new_components: Vec<String> = components
            .iter()
            .map(|comp| {
                if *comp == old_code {
                    new_code.to_string()
                } else {
                    comp.to_string()
                }
            })
            .collect();
        new_elements.push(new_components.join(":"));
    }

    new_elements.join("+")
}
```

**Step 4: Run tests**

Run: `cargo test -p automapper-generator test_migrate_updates`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/fixture_migrator/ crates/automapper-generator/tests/fixture_migrator_test.rs
git commit -m "feat(generator): add fixture migrator with UNH version update"
```

---

## Task 3: Segment Removal and Code Substitution Tests

**Files:**
- Modify: `crates/automapper-generator/tests/fixture_migrator_test.rs`

**Step 1: Write the tests**

Add to `fixture_migrator_test.rs`:
```rust
#[test]
fn test_migrate_removes_dropped_segments() {
    let old_edi = "\
UNB+UNOC:3+SENDER:500+RECEIVER:500+250101:0900+REF1'\
UNH+M1+UTILMD:D:11A:UN:S2.1'\
BGM+E01+M1BGM'\
IMD++Z36+Z13'\
UNT+4+M1'\
UNZ+1+REF1'";

    let mut diff = version_only_diff();
    diff.segments.removed.push(SegmentEntry {
        group: "sg4".into(),
        tag: "IMD".into(),
        context: Some("Removed in S2.2".into()),
    });

    let new_schema = serde_json::json!({
        "pid": "55001",
        "fields": {}
    });

    let result = migrate_fixture(old_edi, &diff, &new_schema);
    assert!(!result.edifact.contains("IMD"),
        "IMD should be removed, got: {}", result.edifact);
    assert!(result.edifact.contains("BGM+E01"),
        "BGM should be preserved");
    assert_eq!(result.stats.segments_removed, 1);
}

#[test]
fn test_migrate_substitutes_renamed_code() {
    let old_edi = "\
UNH+M1+UTILMD:D:11A:UN:S2.1'\
CCI+Z88'\
UNT+2+M1'";

    let mut diff = version_only_diff();
    diff.codes.changed.push(CodeChange {
        segment: "CCI".into(),
        element: "0".into(),
        group: "sg10".into(),
        added: vec!["Z95".into()],
        removed: vec!["Z88".into()],
        context: None,
    });

    let new_schema = serde_json::json!({"pid": "55001", "fields": {}});
    let result = migrate_fixture(old_edi, &diff, &new_schema);

    assert!(result.edifact.contains("CCI+Z95"),
        "Z88 should be renamed to Z95, got: {}", result.edifact);
    assert!(!result.edifact.contains("Z88"),
        "Old code Z88 should not appear");
    assert_eq!(result.stats.codes_substituted, 1);
}

#[test]
fn test_migrate_warns_on_restructured_groups() {
    let old_edi = "UNH+M1+UTILMD:D:11A:UN:S2.1'\nUNT+1+M1'";

    let mut diff = version_only_diff();
    diff.groups.restructured.push(RestructuredGroup {
        group: "sg10".into(),
        description: "Moved from SG8 to SG5".into(),
        manual_review: true,
    });

    let new_schema = serde_json::json!({"pid": "55001", "fields": {}});
    let result = migrate_fixture(old_edi, &diff, &new_schema);

    assert_eq!(result.stats.manual_review_items, 1);
    assert!(result.warnings.iter().any(|w| w.severity == WarningSeverity::Error));
    assert!(result.warnings.iter().any(|w| w.message.contains("sg10")));
}

#[test]
fn test_migrate_warns_on_new_groups() {
    let old_edi = "UNH+M1+UTILMD:D:11A:UN:S2.1'\nUNT+1+M1'";

    let mut diff = version_only_diff();
    diff.groups.added.push(GroupEntry {
        group: "sg8_zh5".into(),
        parent: "sg4".into(),
        entry_segment: Some("SEQ+ZH5".into()),
    });

    let new_schema = serde_json::json!({"pid": "55001", "fields": {}});
    let result = migrate_fixture(old_edi, &diff, &new_schema);

    assert!(result.warnings.iter().any(|w|
        w.severity == WarningSeverity::Warning && w.message.contains("sg8_zh5")
    ));
}
```

**Step 2: Run tests**

Run: `cargo test -p automapper-generator test_migrate_ -- --nocapture`
Expected: All PASS

**Step 3: Commit**

```bash
git add crates/automapper-generator/tests/fixture_migrator_test.rs
git commit -m "test(generator): add segment removal, code substitution, and warning tests"
```

---

## Task 4: Skeleton Segment Generation from Schema

**Files:**
- Create: `crates/automapper-generator/src/fixture_migrator/skeleton.rs`
- Modify: `crates/automapper-generator/src/fixture_migrator/mod.rs`
- Modify: `crates/automapper-generator/src/fixture_migrator/migrator.rs`
- Modify: `crates/automapper-generator/tests/fixture_migrator_test.rs`

**Step 1: Write the failing test**

Add to `fixture_migrator_test.rs`:
```rust
use automapper_generator::fixture_migrator::generate_skeleton_segment;

#[test]
fn test_generate_skeleton_segment_from_schema() {
    // A schema segment definition with one code element and one data element
    let segment_schema = serde_json::json!({
        "id": "MEA",
        "elements": [
            {
                "index": 0,
                "id": "6311",
                "type": "code",
                "codes": [{"value": "AAA", "name": "Test"}],
                "components": []
            },
            {
                "index": 1,
                "id": "6314",
                "type": "data",
                "codes": [],
                "components": []
            }
        ]
    });

    let skeleton = generate_skeleton_segment(&segment_schema);
    assert_eq!(skeleton, "MEA+AAA",
        "Should use first valid code for code elements, omit trailing empty data elements");
}

#[test]
fn test_generate_skeleton_with_composite() {
    let segment_schema = serde_json::json!({
        "id": "LOC",
        "elements": [
            {
                "index": 0,
                "id": "3227",
                "type": "code",
                "codes": [{"value": "Z16", "name": "Marktlokation"}],
                "components": []
            },
            {
                "index": 1,
                "id": "C517",
                "type": "data",
                "composite": "C517",
                "codes": [],
                "components": [
                    {"sub_index": 0, "id": "3225", "type": "data", "codes": []},
                    {"sub_index": 1, "id": "1131", "type": "data", "codes": []}
                ]
            }
        ]
    });

    let skeleton = generate_skeleton_segment(&segment_schema);
    // Code element filled, composite left empty (data elements)
    assert_eq!(skeleton, "LOC+Z16",
        "Should fill code, omit trailing empty composites");
}
```

**Step 2: Write the implementation**

`crates/automapper-generator/src/fixture_migrator/skeleton.rs`:
```rust
/// Generate a skeleton EDIFACT segment string from a PID schema segment definition.
///
/// Code elements are filled with the first valid code value.
/// Data elements are left empty.
/// Trailing empty elements are trimmed.
pub fn generate_skeleton_segment(segment_schema: &serde_json::Value) -> String {
    let tag = segment_schema
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("???");

    let elements = segment_schema
        .get("elements")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Build element values in order
    let mut max_index = 0;
    for el in &elements {
        if let Some(idx) = el.get("index").and_then(|v| v.as_u64()) {
            max_index = max_index.max(idx as usize);
        }
    }

    let mut element_values: Vec<String> = vec![String::new(); max_index + 1];

    for el in &elements {
        let Some(idx) = el.get("index").and_then(|v| v.as_u64()) else {
            continue;
        };
        let idx = idx as usize;

        let el_type = el.get("type").and_then(|v| v.as_str()).unwrap_or("data");
        let components = el.get("components").and_then(|v| v.as_array());

        if let Some(components) = components {
            if !components.is_empty() {
                // Composite element — build component string
                let comp_str = build_composite(components);
                element_values[idx] = comp_str;
                continue;
            }
        }

        // Simple element
        if el_type == "code" {
            if let Some(first_code) = el
                .get("codes")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|c| c.get("value"))
                .and_then(|v| v.as_str())
            {
                element_values[idx] = first_code.to_string();
            }
        }
        // Data elements left as empty string
    }

    // Trim trailing empty elements
    while element_values.last().map(|v| v.is_empty()).unwrap_or(false) {
        element_values.pop();
    }

    if element_values.is_empty() {
        tag.to_string()
    } else {
        format!("{}+{}", tag, element_values.join("+"))
    }
}

fn build_composite(components: &[serde_json::Value]) -> String {
    let mut max_sub = 0;
    for comp in components {
        if let Some(si) = comp.get("sub_index").and_then(|v| v.as_u64()) {
            max_sub = max_sub.max(si as usize);
        }
    }

    let mut comp_values: Vec<String> = vec![String::new(); max_sub + 1];

    for comp in components {
        let Some(si) = comp.get("sub_index").and_then(|v| v.as_u64()) else {
            continue;
        };
        let si = si as usize;
        let comp_type = comp.get("type").and_then(|v| v.as_str()).unwrap_or("data");

        if comp_type == "code" {
            if let Some(first_code) = comp
                .get("codes")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|c| c.get("value"))
                .and_then(|v| v.as_str())
            {
                comp_values[si] = first_code.to_string();
            }
        }
    }

    // Trim trailing empty components
    while comp_values.last().map(|v| v.is_empty()).unwrap_or(false) {
        comp_values.pop();
    }

    comp_values.join(":")
}
```

Update `crates/automapper-generator/src/fixture_migrator/mod.rs`:
```rust
pub mod types;
pub mod migrator;
pub mod skeleton;

pub use types::*;
pub use migrator::*;
pub use skeleton::*;
```

**Step 3: Run tests**

Run: `cargo test -p automapper-generator test_generate_skeleton`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/fixture_migrator/skeleton.rs crates/automapper-generator/src/fixture_migrator/mod.rs crates/automapper-generator/tests/fixture_migrator_test.rs
git commit -m "feat(generator): add skeleton segment generation from PID schema"
```

---

## Task 5: Integration Test with Real Fixture

**Files:**
- Modify: `crates/automapper-generator/tests/fixture_migrator_test.rs`

**Step 1: Write integration test**

Add to `fixture_migrator_test.rs`:
```rust
use std::path::Path;

#[test]
fn test_migrate_real_55001_fixture_with_synthetic_diff() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let fixture_path = base.join(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    let schema_path = base.join(
        "crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json",
    );

    if !fixture_path.exists() || !schema_path.exists() {
        eprintln!("Skipping: fixture or schema not found");
        return;
    }

    let old_edi = std::fs::read_to_string(&fixture_path).unwrap();
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();

    // Create a synthetic diff: just a version bump, no structural changes
    let mut diff = version_only_diff();
    diff.unh_version = Some(VersionChange {
        old: "S2.1".into(),
        new: "S2.2".into(),
    });

    let result = migrate_fixture(&old_edi, &diff, &schema);

    // Verify version was updated
    assert!(result.edifact.contains("S2.2"),
        "Should contain updated version");
    assert!(!result.edifact.contains("S2.1"),
        "Should not contain old version");

    // Verify all non-UNH segments are preserved
    assert!(result.edifact.contains("UNB+UNOC:3"));
    assert!(result.edifact.contains("BGM+E01"));
    assert!(result.edifact.contains("LOC+Z16"));
    assert!(result.edifact.contains("RFF+Z13:55001"));
    assert!(result.edifact.contains("UNT+"));
    assert!(result.edifact.contains("UNZ+"));

    // No warnings for a simple version bump
    let error_warnings: Vec<_> = result.warnings.iter()
        .filter(|w| w.severity == WarningSeverity::Error)
        .collect();
    assert!(error_warnings.is_empty(),
        "Version-only diff should produce no error warnings, got: {:?}", error_warnings);

    eprintln!("Migration stats: copied={}, removed={}, added={}, subs={}",
        result.stats.segments_copied,
        result.stats.segments_removed,
        result.stats.segments_added,
        result.stats.codes_substituted,
    );
}
```

**Step 2: Run test**

Run: `cargo test -p automapper-generator test_migrate_real -- --nocapture`
Expected: PASS

**Step 3: Run full test suite and lint**

Run: `cargo test -p automapper-generator && cargo clippy -p automapper-generator -- -D warnings`
Expected: All PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/tests/fixture_migrator_test.rs
git commit -m "test(generator): add integration test migrating real 55001 fixture"
```
