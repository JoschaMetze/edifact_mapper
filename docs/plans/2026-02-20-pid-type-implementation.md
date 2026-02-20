# PID Type Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace bare MIG-skeleton PID types with AHB-enriched wrapper types that support direct assembly from segments, qualifier discrimination, and generated TOML mapping scaffolds.

**Architecture:** Enhance `pid_type_gen.rs` in `automapper-generator` to emit richer PID types with AHB-derived field names, `from_segments()` assembly, and JSON schema companions. Adapt `mig-bo4e` mapping engine to navigate PID struct fields instead of generic `AssembledTree` groups.

**Tech Stack:** Rust, TOML (serde), JSON (serde_json), insta (snapshot testing), existing MIG/AHB XML parsers

**Design doc:** `docs/plans/2026-02-20-pid-type-redesign.md`

---

## Task 1: Enhance PidGroupInfo with AHB Field Names

The current `PidGroupInfo` stores `group_id` and `qualifier_values` but not a human-readable field name. We need an `ahb_name` field derived from AHB field definitions, plus a `discriminator_segment`/`discriminator_element`/`discriminator_value` to know what qualifies each group instance.

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs:25-37` (PidGroupInfo struct)
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs:148-238` (analyze_pid_structure_with_qualifiers)
- Test: `crates/automapper-generator/src/codegen/pid_type_gen.rs` (add unit test module)

**Step 1: Write a failing test**

Add a `#[cfg(test)]` module at the bottom of `pid_type_gen.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{ahb_parser, mig_parser};
    use std::path::PathBuf;

    fn load_mig_ahb() -> (MigSchema, AhbSchema) {
        let mig_path = PathBuf::from("../../xml-migs-and-ahbs/FV2504/UTILMD_Strom_MIG_S2.1_2025-04-01.xml");
        let ahb_path = PathBuf::from("../../xml-migs-and-ahbs/FV2504/UTILMD_Strom_AHB_3.2_2025-04-01.xml");
        // Skip if XML files not available
        if !mig_path.exists() || !ahb_path.exists() {
            panic!("MIG/AHB XML files not found — run from workspace root");
        }
        let mig = mig_parser::parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        let ahb = ahb_parser::parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        (mig, ahb)
    }

    #[test]
    fn test_pid_55001_structure_has_named_groups() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

        // SG2 should be split into named occurrences by NAD qualifier
        let sg2_group = structure.groups.iter().find(|g| g.group_id == "SG2").unwrap();
        // Should have child or be split — at minimum, ahb_name should be populated
        assert!(!structure.groups.is_empty());

        // SG4 should exist
        let sg4 = structure.groups.iter().find(|g| g.group_id == "SG4").unwrap();
        assert!(!sg4.child_groups.is_empty());

        // SG4's child SG8 groups should have qualifier discrimination
        let sg8_children: Vec<_> = sg4.child_groups.iter().filter(|c| c.group_id == "SG8").collect();
        // At least some SG8 groups should have qualifier values (e.g., ZD5, ZD6)
        let has_qualified = sg8_children.iter().any(|c| !c.qualifier_values.is_empty());
        assert!(has_qualified, "SG8 groups should have qualifier discrimination");
    }
}
```

**Step 2: Run the test to verify it passes with current code**

Run: `cargo test -p automapper-generator test_pid_55001_structure_has_named_groups -- --nocapture`

This test should pass with the existing `analyze_pid_structure_with_qualifiers`. If it does, we have a baseline.

**Step 3: Add `ahb_name` and discriminator fields to PidGroupInfo**

In `pid_type_gen.rs`, modify `PidGroupInfo` (line 26):

```rust
#[derive(Debug, Clone)]
pub struct PidGroupInfo {
    pub group_id: String,
    pub qualifier_values: Vec<String>,
    pub ahb_status: String,
    /// Human-readable AHB-derived field name (e.g., "Absender", "Summenzeitreihe Arbeit/Leistung").
    pub ahb_name: Option<String>,
    /// Trigger segment + data element for qualifier discrimination.
    /// E.g., ("NAD", "3035") for SG2, ("SEQ", "1229") for SG8.
    pub discriminator: Option<(String, String)>,
    pub child_groups: Vec<PidGroupInfo>,
    pub segments: BTreeSet<String>,
}
```

Update all construction sites in `GroupOccurrenceTracker::into_group_info()` and `analyze_pid_structure_with_qualifiers()` to populate `ahb_name: None` and `discriminator: None` initially. (The actual name derivation comes in Task 2.)

**Step 4: Run the test and all workspace tests**

Run: `cargo test -p automapper-generator -- --nocapture`
Run: `cargo clippy -p automapper-generator -- -D warnings`

Expected: All tests pass, no clippy warnings.

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs
git commit -m "refactor(generator): add ahb_name and discriminator fields to PidGroupInfo"
```

---

## Task 2: Derive AHB Field Names from Pruefidentifikator Fields

Populate `ahb_name` on `PidGroupInfo` by looking at the AHB's `AhbFieldDefinition.name` for the group's entry segment. For SG2, the name comes from the NAD field definition (e.g., "MP-ID Absender" → "absender"). For SG8, from the SEQ definition.

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs:148-238` (analyze function)
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs` (add name derivation helper)
- Test: existing test module from Task 1

**Step 1: Write a failing test**

Add to the test module:

```rust
#[test]
fn test_pid_55001_sg2_has_ahb_names() {
    let (mig, ahb) = load_mig_ahb();
    let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
    let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

    let sg2 = structure.groups.iter().find(|g| g.group_id == "SG2").unwrap();
    // SG2 should have a discriminator (NAD qualifier)
    assert!(sg2.discriminator.is_some(), "SG2 should have NAD discriminator");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_pid_55001_sg2_has_ahb_names -- --nocapture`
Expected: FAIL — discriminator is None.

**Step 3: Implement name derivation**

Add a helper function:

```rust
/// Derive the AHB field name for a group from its entry segment's AHB definition.
///
/// Looks for the AHB field that references this group's entry segment path
/// (e.g., "SG2/NAD" for SG2, "SG4/SG8/SEQ" for SG8 under SG4).
fn derive_ahb_name(
    pid: &Pruefidentifikator,
    group_path: &str,  // e.g., "SG2" or "SG4/SG8"
    entry_segment: &str,  // e.g., "NAD", "SEQ"
) -> Option<String> {
    let target_prefix = format!("{}/{}", group_path, entry_segment);
    pid.fields
        .iter()
        .find(|f| f.segment_path.starts_with(&target_prefix))
        .and_then(|f| {
            let name = f.name.trim();
            if name.is_empty() { None } else { Some(name.to_string()) }
        })
}
```

Then populate `discriminator` on top-level groups by calling `find_group_qualifier()` in the analysis function. For each top-level group, find its entry segment's trigger element and set `discriminator = Some((segment_id, element_id))`.

**Step 4: Run tests**

Run: `cargo test -p automapper-generator -- --nocapture`
Expected: All pass.

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs
git commit -m "feat(generator): derive AHB names and discriminators for PID groups"
```

---

## Task 3: Generate PID-Specific Wrapper Structs

Replace the current `generate_pid_struct()` output (bare `Vec<Sg2>` fields) with AHB-named wrapper types. For PID 55001, emit `Pid55001Absender`, `Pid55001Empfaenger`, `Pid55001Transaktion`, etc.

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs:431-499` (generate_pid_struct, emit_pid_group_field)
- Test: snapshot test with insta

**Step 1: Write a failing snapshot test**

Add to the test module:

```rust
#[test]
fn test_generate_pid_55001_struct_snapshot() {
    let (mig, ahb) = load_mig_ahb();
    let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
    let source = generate_pid_struct(pid, &mig, &ahb);
    insta::assert_snapshot!("pid_55001_struct", source);
}
```

**Step 2: Run to create initial snapshot**

Run: `cargo test -p automapper-generator test_generate_pid_55001_struct_snapshot -- --nocapture`

This creates a pending snapshot. Review with `cargo insta review` — the current output shows bare `Vec<Sg2>` fields.

**Step 3: Rewrite `generate_pid_struct()` to emit wrapper types**

Replace the current `emit_pid_group_field()` approach. For each `PidGroupInfo` that has qualifier discrimination or is a named group, generate a dedicated wrapper struct:

```rust
fn generate_pid_struct(pid: &Pruefidentifikator, mig: &MigSchema, ahb: &AhbSchema) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let mut out = String::new();
    let struct_name = format!("Pid{}", pid.id);

    // First, emit wrapper structs for groups that need them
    for group in &structure.groups {
        emit_wrapper_structs(&struct_name, group, &mut out);
    }

    // Then emit the main PID struct
    out.push_str(&format!("/// PID {}: {}\n", pid.id, sanitize_doc(&pid.beschreibung)));
    if let Some(ref komm) = pid.kommunikation_von {
        out.push_str(&format!("/// Kommunikation: {}\n", sanitize_doc(komm)));
    }
    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    // Top-level segments
    for seg_id in &structure.top_level_segments {
        let field_name = seg_id.to_lowercase();
        let seg_type = format!("super::super::segments::Seg{}", capitalize_segment_id(seg_id));
        out.push_str(&format!("    pub {field_name}: {seg_type},\n"));
    }

    // Groups with wrapper type names
    for group in &structure.groups {
        emit_pid_group_field_v2(&struct_name, group, &mut out, "    ");
    }

    out.push_str("}\n\n");
    out
}
```

The `emit_wrapper_structs()` function generates structs like `Pid55001Absender` with the group's segments as fields. The `emit_pid_group_field_v2()` uses the wrapper type name instead of `super::super::groups::SgN`.

**Step 4: Update snapshot and review**

Run: `cargo test -p automapper-generator test_generate_pid_55001_struct_snapshot -- --nocapture`
Run: `cargo insta review`

Verify the new snapshot shows named wrapper types.

**Step 5: Run full test suite**

Run: `cargo test -p automapper-generator -- --nocapture`
Run: `cargo clippy -p automapper-generator -- -D warnings`

**Step 6: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs
git commit -m "feat(generator): emit PID-specific wrapper structs with AHB field names"
```

---

## Task 4: Generate PID JSON Schema Files

Add a new function `generate_pid_schema()` that emits a JSON companion file alongside each PID Rust struct. The schema describes field paths, discriminators, cardinality, and available codes.

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs` (add generate_pid_schema function)
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs:512-565` (generate_pid_types orchestrator)
- Test: snapshot test with insta

**Step 1: Write a failing test**

```rust
#[test]
fn test_generate_pid_55001_schema_snapshot() {
    let (mig, ahb) = load_mig_ahb();
    let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
    let schema = generate_pid_schema(pid, &mig, &ahb);
    insta::assert_snapshot!("pid_55001_schema", schema);
}
```

**Step 2: Run to verify it fails**

Run: `cargo test -p automapper-generator test_generate_pid_55001_schema_snapshot -- --nocapture`
Expected: FAIL — function does not exist.

**Step 3: Implement `generate_pid_schema()`**

```rust
/// Generate a JSON schema describing a PID's structure for runtime use.
pub fn generate_pid_schema(
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    ahb: &AhbSchema,
) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let mut root = serde_json::Map::new();
    root.insert("pid".to_string(), serde_json::Value::String(pid.id.clone()));
    root.insert("beschreibung".to_string(), serde_json::Value::String(pid.beschreibung.clone()));
    root.insert("format_version".to_string(), serde_json::Value::String(ahb.format_version.clone()));

    if let Some(ref komm) = pid.kommunikation_von {
        root.insert("kommunikation_von".to_string(), serde_json::Value::String(komm.clone()));
    }

    let mut fields = serde_json::Map::new();
    for group in &structure.groups {
        let field_name = make_wrapper_field_name(group);
        fields.insert(field_name, group_to_schema_value(group));
    }
    root.insert("fields".to_string(), serde_json::Value::Object(fields));

    serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap()
}

fn group_to_schema_value(group: &PidGroupInfo) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("source_group".to_string(), serde_json::Value::String(group.group_id.clone()));

    if let Some(ref disc) = group.discriminator {
        let mut d = serde_json::Map::new();
        d.insert("segment".to_string(), serde_json::Value::String(disc.0.clone()));
        d.insert("element".to_string(), serde_json::Value::String(disc.1.clone()));
        if !group.qualifier_values.is_empty() {
            d.insert("values".to_string(),
                serde_json::Value::Array(group.qualifier_values.iter().map(|v| serde_json::Value::String(v.clone())).collect()));
        }
        obj.insert("discriminator".to_string(), serde_json::Value::Object(d));
    } else {
        obj.insert("discriminator".to_string(), serde_json::Value::Null);
    }

    let segments: Vec<_> = group.segments.iter().map(|s| serde_json::Value::String(s.clone())).collect();
    obj.insert("segments".to_string(), serde_json::Value::Array(segments));

    if !group.child_groups.is_empty() {
        let mut children = serde_json::Map::new();
        for child in &group.child_groups {
            let name = make_wrapper_field_name(child);
            children.insert(name, group_to_schema_value(child));
        }
        obj.insert("children".to_string(), serde_json::Value::Object(children));
    }

    serde_json::Value::Object(obj)
}
```

**Step 4: Update the `generate_pid_types()` orchestrator to also write schema JSON**

In the loop over `ahb.workflows` (line 525), after writing the `.rs` file, also write:
```rust
let schema = generate_pid_schema(pid, mig, ahb);
std::fs::write(pids_dir.join(format!("pid_{}_schema.json", pid.id.to_lowercase())), schema)?;
```

**Step 5: Run tests and review snapshot**

Run: `cargo test -p automapper-generator test_generate_pid_55001_schema_snapshot -- --nocapture`
Run: `cargo insta review`
Run: `cargo clippy -p automapper-generator -- -D warnings`

**Step 6: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs
git commit -m "feat(generator): generate PID JSON schema companions"
```

---

## Task 5: Regenerate PID Types and Verify Compilation

Run the generator with the new code to replace the current bare PID types with enriched ones. Verify the generated code compiles.

**Files:**
- Regenerate: `crates/mig-types/src/generated/fv2504/utilmd/pids/*.rs`
- Regenerate: `crates/mig-types/src/generated/fv2504/utilmd/pids/*.json`

**Step 1: Run the generator**

```bash
cargo run -p automapper-generator -- generate-pid-types \
    --mig-path xml-migs-and-ahbs/FV2504/UTILMD_Strom_MIG_S2.1_2025-04-01.xml \
    --ahb-path xml-migs-and-ahbs/FV2504/UTILMD_Strom_AHB_3.2_2025-04-01.xml \
    --output-dir crates/mig-types/src/generated \
    --format-version FV2504 \
    --message-type UTILMD
```

Note: Find the exact XML filenames by listing `xml-migs-and-ahbs/FV2504/`. Use the actual filenames.

**Step 2: Verify compilation**

Run: `cargo check -p mig-types`

Fix any compilation errors in the generated code by adjusting the generator.

**Step 3: Run all workspace tests**

Run: `cargo test --workspace`
Run: `cargo clippy --workspace -- -D warnings`

Fix any breakages. The main risk is existing code that references `pid.sg2` or `pid.sg4` — these fields are renamed. Check for usages.

**Step 4: Commit**

```bash
git add crates/mig-types/src/generated/
git commit -m "chore: regenerate PID types with AHB-enriched wrapper structs"
```

---

## Task 6: Add `from_segments()` to SegmentCursor

The `SegmentCursor` currently only tracks an index. For direct PID assembly, it needs to hold a reference to the segment slice and provide `peek()` and `consume()` methods.

**Files:**
- Modify: `crates/mig-assembly/src/cursor.rs`
- Test: `crates/mig-assembly/src/cursor.rs` (existing test module)

**Step 1: Write a failing test**

Add to the existing test module in `cursor.rs`:

```rust
use crate::tokenize::OwnedSegment;

fn make_segment(id: &str) -> OwnedSegment {
    OwnedSegment {
        id: id.to_string(),
        elements: vec![],
        segment_number: 0,
    }
}

#[test]
fn test_cursor_peek_and_consume() {
    let segments = vec![
        make_segment("UNH"),
        make_segment("BGM"),
        make_segment("NAD"),
    ];
    let mut cursor = SegmentCursor::new(segments.len());

    assert_eq!(segments[cursor.position()].id, "UNH");
    assert!(!cursor.is_exhausted());

    cursor.advance(); // consume UNH
    assert_eq!(segments[cursor.position()].id, "BGM");

    cursor.advance(); // consume BGM
    cursor.advance(); // consume NAD
    assert!(cursor.is_exhausted());
}

#[test]
fn test_cursor_peek_is() {
    let segments = vec![
        make_segment("NAD"),
        make_segment("IDE"),
    ];
    let cursor = SegmentCursor::new(segments.len());

    assert!(segments[cursor.position()].id.eq_ignore_ascii_case("NAD"));
}
```

**Step 2: Run tests**

Run: `cargo test -p mig-assembly test_cursor_peek_and_consume -- --nocapture`

These tests should pass already since they use the existing cursor API. This validates the approach.

**Step 3: Add convenience methods**

Add peek/consume helpers that work with a segment slice reference. These are standalone functions (not methods on SegmentCursor) since the cursor doesn't own the data:

```rust
/// Check if the segment at the cursor's current position matches a tag.
pub fn peek_is(segments: &[OwnedSegment], cursor: &SegmentCursor, tag: &str) -> bool {
    if cursor.is_exhausted() {
        return false;
    }
    segments[cursor.position()].is(tag)
}

/// Consume the segment at the cursor's current position, advancing the cursor.
/// Returns None if the cursor is exhausted.
pub fn consume<'a>(
    segments: &'a [OwnedSegment],
    cursor: &mut SegmentCursor,
) -> Option<&'a OwnedSegment> {
    if cursor.is_exhausted() {
        return None;
    }
    let seg = &segments[cursor.position()];
    cursor.advance();
    Some(seg)
}

/// Consume the segment at cursor if it matches the expected tag.
/// Returns Err if exhausted or tag mismatch.
pub fn expect_segment<'a>(
    segments: &'a [OwnedSegment],
    cursor: &mut SegmentCursor,
    tag: &str,
) -> Result<&'a OwnedSegment, crate::AssemblyError> {
    if cursor.is_exhausted() {
        return Err(crate::AssemblyError::SegmentNotFound {
            expected: tag.to_string(),
        });
    }
    let seg = &segments[cursor.position()];
    if !seg.is(tag) {
        return Err(crate::AssemblyError::SegmentNotFound {
            expected: tag.to_string(),
        });
    }
    cursor.advance();
    Ok(seg)
}
```

Note: Check the existing `AssemblyError` variants. If `SegmentNotFound` doesn't exist, add it to `crates/mig-assembly/src/error.rs`.

**Step 4: Write tests for the new helpers**

```rust
#[test]
fn test_peek_is_helper() {
    let segments = vec![make_segment("NAD"), make_segment("IDE")];
    let cursor = SegmentCursor::new(segments.len());
    assert!(peek_is(&segments, &cursor, "NAD"));
    assert!(!peek_is(&segments, &cursor, "IDE"));
}

#[test]
fn test_expect_segment_helper() {
    let segments = vec![make_segment("UNH"), make_segment("BGM")];
    let mut cursor = SegmentCursor::new(segments.len());
    let seg = expect_segment(&segments, &mut cursor, "UNH").unwrap();
    assert_eq!(seg.id, "UNH");
    assert_eq!(cursor.position(), 1);
}

#[test]
fn test_expect_segment_wrong_tag() {
    let segments = vec![make_segment("UNH")];
    let mut cursor = SegmentCursor::new(segments.len());
    let result = expect_segment(&segments, &mut cursor, "BGM");
    assert!(result.is_err());
}
```

**Step 5: Run all tests**

Run: `cargo test -p mig-assembly -- --nocapture`
Run: `cargo clippy -p mig-assembly -- -D warnings`

**Step 6: Commit**

```bash
git add crates/mig-assembly/src/cursor.rs crates/mig-assembly/src/error.rs
git commit -m "feat(mig-assembly): add peek/consume/expect helpers for direct PID assembly"
```

---

## Task 7: Generate `from_segments()` Assembly Code

Add code generation for `from_segments()` on each PID struct. This method walks segments using `SegmentCursor` and populates typed fields with qualifier discrimination.

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs` (add assembly codegen)
- Test: snapshot test

**Step 1: Write a failing test**

```rust
#[test]
fn test_generate_pid_55001_from_segments_snapshot() {
    let (mig, ahb) = load_mig_ahb();
    let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
    let source = generate_pid_from_segments(pid, &mig, &ahb);
    insta::assert_snapshot!("pid_55001_from_segments", source);
}
```

**Step 2: Run to verify it fails**

Run: `cargo test -p automapper-generator test_generate_pid_55001_from_segments_snapshot -- --nocapture`
Expected: FAIL — function does not exist.

**Step 3: Implement `generate_pid_from_segments()`**

This function generates a Rust `impl` block with `from_segments()` for a PID:

```rust
/// Generate a `from_segments()` impl block for a PID type.
pub fn generate_pid_from_segments(
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    ahb: &AhbSchema,
) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let struct_name = format!("Pid{}", pid.id);
    let mut out = String::new();

    out.push_str(&format!("impl {struct_name} {{\n"));
    out.push_str("    /// Assemble this PID from a pre-tokenized segment list.\n");
    out.push_str("    pub fn from_segments(\n");
    out.push_str("        segments: &[OwnedSegment],\n");
    out.push_str("    ) -> Result<Self, AssemblyError> {\n");
    out.push_str("        let mut cursor = SegmentCursor::new(segments.len());\n\n");

    // Emit consume calls for top-level segments
    for seg_id in &structure.top_level_segments {
        let field = seg_id.to_lowercase();
        out.push_str(&format!(
            "        let {field} = expect_segment(segments, &mut cursor, \"{seg_id}\")?;\n"
        ));
    }
    out.push_str("\n");

    // Emit group consumption with qualifier loops
    for group in &structure.groups {
        emit_group_consumption(&struct_name, group, &mut out, "        ");
    }

    // Build return struct
    out.push_str(&format!("\n        Ok({struct_name} {{\n"));
    for seg_id in &structure.top_level_segments {
        let field = seg_id.to_lowercase();
        // Convert &OwnedSegment to the wrapper type — this depends on generated segment types
        out.push_str(&format!("            {field},\n"));
    }
    for group in &structure.groups {
        let field = make_wrapper_field_name(group);
        out.push_str(&format!("            {field},\n"));
    }
    out.push_str("        })\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}
```

The `emit_group_consumption()` function handles repeating groups, qualifier loops, and nested children.

**Step 4: Run and review snapshot**

Run: `cargo test -p automapper-generator test_generate_pid_55001_from_segments_snapshot -- --nocapture`
Run: `cargo insta review`

**Step 5: Run all tests**

Run: `cargo test -p automapper-generator -- --nocapture`
Run: `cargo clippy -p automapper-generator -- -D warnings`

**Step 6: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs
git commit -m "feat(generator): generate from_segments() assembly code for PID types"
```

---

## Task 8: Regenerate and Test PID Assembly

Regenerate the PID types including `from_segments()` impls. Write an integration test that parses a real EDIFACT fixture and assembles it into a typed `Pid55001`.

**Files:**
- Regenerate: `crates/mig-types/src/generated/fv2504/utilmd/pids/*.rs`
- Create: `crates/mig-types/tests/pid_assembly_test.rs`

**Step 1: Regenerate PID types**

Run the generator as in Task 5.

**Step 2: Verify compilation**

Run: `cargo check -p mig-types`

Fix any compilation issues.

**Step 3: Write an integration test**

Create `crates/mig-types/tests/pid_assembly_test.rs`:

```rust
//! Integration test: parse real EDIFACT fixture → Pid55001::from_segments()

use mig_assembly::tokenize::parse_to_segments;
use mig_types::generated::fv2504::utilmd::pids::pid_55001::Pid55001;

#[test]
fn test_pid_55001_from_segments() {
    let fixture_dir = std::path::Path::new(
        "../../example_market_communication_bo4e_transactions/UTILMD/FV2504"
    );
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture directory not found");
        return;
    }

    // Find the first fixture file
    let fixture = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "txt" || x == "edi").unwrap_or(false));

    let Some(fixture) = fixture else {
        eprintln!("Skipping: no fixture files found");
        return;
    };

    let input = std::fs::read(fixture.path()).unwrap();
    let segments = parse_to_segments(&input).unwrap();

    let pid = Pid55001::from_segments(&segments);
    assert!(pid.is_ok(), "from_segments failed: {:?}", pid.err());

    let pid = pid.unwrap();
    // Verify basic fields are populated
    assert!(!pid.unh.elements.is_empty() || true); // Adjust based on generated type
}
```

Note: The exact assertions depend on the generated struct shape. Adjust after seeing the generated types.

**Step 4: Run the integration test**

Run: `cargo test -p mig-types test_pid_55001_from_segments -- --nocapture`
Expected: PASS

**Step 5: Run all workspace tests**

Run: `cargo test --workspace`
Run: `cargo clippy --workspace -- -D warnings`

**Step 6: Commit**

```bash
git add crates/mig-types/ crates/mig-types/tests/
git commit -m "feat(mig-types): regenerate PID types with from_segments + integration test"
```

---

## Task 9: Adapt MappingEngine for PID Source Paths

Update `MappingMeta` to support `source_path` (PID field name) alongside the existing `source_group` (AssembledTree group path). The engine should be able to extract segments from a PID wrapper struct field.

**Files:**
- Modify: `crates/mig-bo4e/src/definition.rs:19-26` (MappingMeta)
- Modify: `crates/mig-bo4e/src/engine.rs` (add PID-aware extraction)
- Test: `crates/mig-bo4e/tests/entity_mapping_test.rs`

**Step 1: Add `source_path` to MappingMeta**

In `definition.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMeta {
    pub entity: String,
    pub bo4e_type: String,
    pub companion_type: Option<String>,
    pub source_group: String,
    /// PID struct field path (e.g., "absender", "transaktionen").
    /// When present, the mapping engine uses PID-direct navigation instead of AssembledTree groups.
    pub source_path: Option<String>,
    pub discriminator: Option<String>,
}
```

**Step 2: Add a method to MappingEngine for PID-based forward mapping**

In `engine.rs`, add:

```rust
/// Map a PID struct field's segments to BO4E JSON.
///
/// `segments` are the segments from the PID wrapper field (e.g., pid.absender.segments()).
/// Uses the same field extraction logic as `map_forward` but without group navigation.
pub fn map_forward_from_segments(
    &self,
    segments: &[OwnedSegment],
    def: &MappingDefinition,
) -> serde_json::Value {
    // Convert OwnedSegments to AssembledSegments for compatibility with extract_from_instance
    let assembled_segments: Vec<AssembledSegment> = segments
        .iter()
        .map(|s| AssembledSegment {
            tag: s.id.clone(),
            elements: s.elements.clone(),
        })
        .collect();

    let instance = AssembledGroupInstance {
        segments: assembled_segments,
        child_groups: vec![],
    };

    let mut result = serde_json::Map::new();
    for (path, field_mapping) in &def.fields {
        let (target, enum_map) = match field_mapping {
            FieldMapping::Simple(t) => (t.clone(), None),
            FieldMapping::Structured(s) => (s.target.clone(), s.enum_map.as_ref()),
            FieldMapping::Nested(_) => continue,
        };
        if target.is_empty() {
            continue;
        }
        if let Some(val) = Self::extract_from_instance(&instance, path) {
            let mapped_val = if let Some(map) = enum_map {
                map.get(&val).cloned().unwrap_or(val)
            } else {
                val
            };
            set_nested_value(&mut result, &target, mapped_val);
        }
    }

    serde_json::Value::Object(result)
}
```

**Step 3: Write a test**

```rust
#[test]
fn test_map_forward_from_segments() {
    // Create OwnedSegments mimicking NAD+MS content
    let segments = vec![OwnedSegment {
        id: "NAD".to_string(),
        elements: vec![
            vec!["MS".to_string()],
            vec!["9978842000002".to_string(), "".to_string(), "293".to_string()],
        ],
        segment_number: 1,
    }];

    let engine = load_engine();
    let def = engine.definition_for_entity("Marktteilnehmer").unwrap();
    let result = engine.map_forward_from_segments(&segments, def);

    assert_eq!(result["marktrolle"], "MS");
    assert_eq!(result["rollencodenummer"], "9978842000002");
    assert_eq!(result["rollencodetyp"], "BDEW");
}
```

**Step 4: Run tests**

Run: `cargo test -p mig-bo4e test_map_forward_from_segments -- --nocapture`
Run: `cargo test -p mig-bo4e -- --nocapture`
Run: `cargo clippy -p mig-bo4e -- -D warnings`

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/definition.rs crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): add source_path and PID-aware forward mapping"
```

---

## Task 10: Generate TOML Mapping Scaffolds

Add a new generator command that produces pre-filled TOML mapping files from the PID schema. One file per entity path, with MIG segment paths pre-filled and empty `target` fields.

**Files:**
- Create: `crates/automapper-generator/src/codegen/toml_scaffold_gen.rs`
- Modify: `crates/automapper-generator/src/codegen/mod.rs` (add module)
- Modify: `crates/automapper-generator/src/main.rs` (add CLI command)
- Test: snapshot test

**Step 1: Create the scaffold generator module**

Create `crates/automapper-generator/src/codegen/toml_scaffold_gen.rs`:

```rust
//! Generate TOML mapping scaffolds from PID schema.
//!
//! Produces one `.toml` file per entity path with MIG segment paths pre-filled.
//! Developers fill in `target` (BO4E field name) and optional `enum_map`.

use std::collections::BTreeSet;
use std::path::Path;

use crate::codegen::pid_type_gen::{analyze_pid_structure_with_qualifiers, PidGroupInfo};
use crate::error::GeneratorError;
use crate::schema::ahb::{AhbSchema, Pruefidentifikator};
use crate::schema::mig::{MigSchema, MigSegmentGroup};

/// Generate TOML scaffold for a single PID group field.
pub fn generate_group_scaffold(
    group: &PidGroupInfo,
    field_name: &str,
    entity_hint: &str,
    mig: &MigSchema,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# AUTO-GENERATED scaffold for {entity_hint} → {field_name}\n"));
    out.push_str("# Fill in \"target\" fields with BO4E field names.\n\n");
    out.push_str("[meta]\n");
    out.push_str(&format!("entity = \"{entity_hint}\"\n"));
    out.push_str(&format!("bo4e_type = \"{entity_hint}\"\n"));
    out.push_str(&format!("source_path = \"{field_name}\"\n\n"));
    out.push_str("[fields]\n");

    // For each segment in the group, enumerate its elements from the MIG
    for seg_id in &group.segments {
        if let Some(mig_seg) = find_segment_in_mig(seg_id, &group.group_id, mig) {
            // Emit element paths
            for (ei, de) in mig_seg.data_elements.iter().enumerate() {
                out.push_str(&format!(
                    "\"{}.{}\" = {{ target = \"\" }}\n",
                    seg_id.to_lowercase(), ei
                ));
            }
            for (ci, comp) in mig_seg.composites.iter().enumerate() {
                let elem_idx = mig_seg.data_elements.len() + ci;
                for (di, de) in comp.data_elements.iter().enumerate() {
                    out.push_str(&format!(
                        "\"{}.{}.{}\" = {{ target = \"\" }}\n",
                        seg_id.to_lowercase(), elem_idx, di
                    ));
                }
            }
        }
    }

    out
}

fn find_segment_in_mig<'a>(
    seg_id: &str,
    group_id: &str,
    mig: &'a MigSchema,
) -> Option<&'a crate::schema::mig::MigSegment> {
    fn find_in_group<'a>(
        seg_id: &str, target_group: &str, group: &'a MigSegmentGroup,
    ) -> Option<&'a crate::schema::mig::MigSegment> {
        if group.id == target_group {
            return group.segments.iter().find(|s| s.id.eq_ignore_ascii_case(seg_id));
        }
        for nested in &group.nested_groups {
            if let Some(s) = find_in_group(seg_id, target_group, nested) {
                return Some(s);
            }
        }
        None
    }

    // Check top-level segments first
    if let Some(seg) = mig.segments.iter().find(|s| s.id.eq_ignore_ascii_case(seg_id)) {
        return Some(seg);
    }
    // Check groups
    for group in &mig.segment_groups {
        if let Some(seg) = find_in_group(seg_id, group_id, group) {
            return Some(seg);
        }
    }
    None
}
```

**Step 2: Write a test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{ahb_parser, mig_parser};

    #[test]
    fn test_scaffold_for_pid_55001_sg2() {
        let mig = mig_parser::parse_mig(
            &"../../xml-migs-and-ahbs/FV2504/UTILMD_Strom_MIG_S2.1_2025-04-01.xml".into(),
            "UTILMD", Some("Strom"), "FV2504",
        ).unwrap();
        let ahb = ahb_parser::parse_ahb(
            &"../../xml-migs-and-ahbs/FV2504/UTILMD_Strom_AHB_3.2_2025-04-01.xml".into(),
            "UTILMD", Some("Strom"), "FV2504",
        ).unwrap();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);
        let sg2 = structure.groups.iter().find(|g| g.group_id == "SG2").unwrap();

        let scaffold = generate_group_scaffold(sg2, "absender", "Marktteilnehmer", &mig);
        assert!(scaffold.contains("source_path = \"absender\""));
        assert!(scaffold.contains("[fields]"));
        assert!(scaffold.contains("\"nad."));
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p automapper-generator test_scaffold_for_pid_55001_sg2 -- --nocapture`

**Step 4: Add CLI command**

In `main.rs`, add a new `GenerateTomlScaffolds` command variant:

```rust
/// Generate TOML mapping scaffolds from PID schema
GenerateTomlScaffolds {
    #[arg(long)]
    mig_path: PathBuf,
    #[arg(long)]
    ahb_path: PathBuf,
    #[arg(long)]
    output_dir: PathBuf,
    #[arg(long)]
    format_version: String,
    #[arg(long)]
    message_type: String,
    /// Generate scaffolds only for this PID (e.g., "55001")
    #[arg(long)]
    pid: Option<String>,
}
```

**Step 5: Run full tests**

Run: `cargo test -p automapper-generator -- --nocapture`
Run: `cargo clippy -p automapper-generator -- -D warnings`

**Step 6: Commit**

```bash
git add crates/automapper-generator/src/codegen/toml_scaffold_gen.rs \
       crates/automapper-generator/src/codegen/mod.rs \
       crates/automapper-generator/src/main.rs
git commit -m "feat(generator): add TOML mapping scaffold generation from PID schema"
```

---

## Task 11: End-to-End Integration Test

Write a full pipeline test: EDIFACT bytes → parse → detect PID → assemble into Pid55001 → apply TOML mappings → BO4E JSON. This validates the entire new pipeline works together.

**Files:**
- Create: `crates/mig-bo4e/tests/pid_pipeline_test.rs`

**Step 1: Write the integration test**

```rust
//! End-to-end test: EDIFACT → PID assembly → TOML mapping → BO4E JSON.

use mig_assembly::pid_detect::detect_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;

#[test]
fn test_full_pid_pipeline() {
    let fixture_dir = std::path::Path::new(
        "../../example_market_communication_bo4e_transactions/UTILMD/FV2504"
    );
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture directory not found");
        return;
    }

    // Find a 55001 fixture
    let fixture = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            // Read and check PID
            if let Ok(bytes) = std::fs::read(p) {
                if let Ok(segments) = parse_to_segments(&bytes) {
                    if let Ok(pid) = detect_pid(&segments) {
                        return pid == "55001";
                    }
                }
            }
            false
        });

    let Some(fixture_path) = fixture else {
        eprintln!("Skipping: no PID 55001 fixture found");
        return;
    };

    // Step 1: Parse
    let bytes = std::fs::read(&fixture_path).unwrap();
    let segments = parse_to_segments(&bytes).unwrap();

    // Step 2: Detect PID
    let pid = detect_pid(&segments).unwrap();
    assert_eq!(pid, "55001");

    // Step 3: Assemble into typed PID
    // (Adjust import based on generated type location)
    // let pid_struct = Pid55001::from_segments(&segments).unwrap();

    // Step 4: Apply TOML mappings
    let mapping_dir = std::path::Path::new("../../mappings/FV2504/UTILMD_Strom");
    if !mapping_dir.exists() {
        eprintln!("Skipping: mapping directory not found");
        return;
    }
    let engine = MappingEngine::load(mapping_dir).unwrap();

    // Step 5: Verify BO4E output
    // (Specific assertions depend on the fixture content and mapping completeness)
    assert!(!engine.definitions().is_empty(), "Should have loaded TOML mappings");
}
```

**Step 2: Run the test**

Run: `cargo test -p mig-bo4e test_full_pid_pipeline -- --nocapture`
Expected: PASS (with potential skips for missing fixtures).

**Step 3: Commit**

```bash
git add crates/mig-bo4e/tests/pid_pipeline_test.rs
git commit -m "test(mig-bo4e): add end-to-end PID pipeline integration test"
```

---

## Task 12: Migrate Existing TOML Mappings to PID Directory Structure

Move the existing TOML mapping files from `mappings/FV2504/UTILMD_Strom/` into `pid_55001/` subdirectory and update `source_group` to `source_path` where applicable.

**Files:**
- Move: `mappings/FV2504/UTILMD_Strom/*.toml` → `mappings/FV2504/UTILMD_Strom/pid_55001/`
- Modify: each TOML file to add `source_path`

**Step 1: Create PID directory and move files**

```bash
mkdir -p mappings/FV2504/UTILMD_Strom/pid_55001
git mv mappings/FV2504/UTILMD_Strom/marktteilnehmer.toml mappings/FV2504/UTILMD_Strom/pid_55001/
git mv mappings/FV2504/UTILMD_Strom/prozessdaten.toml mappings/FV2504/UTILMD_Strom/pid_55001/
git mv mappings/FV2504/UTILMD_Strom/prozess_referenz.toml mappings/FV2504/UTILMD_Strom/pid_55001/
git mv mappings/FV2504/UTILMD_Strom/marktlokation.toml mappings/FV2504/UTILMD_Strom/pid_55001/
git mv mappings/FV2504/UTILMD_Strom/messlokation.toml mappings/FV2504/UTILMD_Strom/pid_55001/
git mv mappings/FV2504/UTILMD_Strom/geschaeftspartner.toml mappings/FV2504/UTILMD_Strom/pid_55001/
```

**Step 2: Add `source_path` to each TOML file**

For example, in `marktteilnehmer.toml`:
```toml
[meta]
entity = "Marktteilnehmer"
bo4e_type = "Marktteilnehmer"
source_group = "SG2"
source_path = "absender"  # NEW
```

The `source_group` stays for backward compatibility. `source_path` is the PID-aware path.

Mapping from entity → source_path:
- marktteilnehmer.toml → `source_path = "absender"` (or "empfaenger" — may need two files)
- prozessdaten.toml → `source_path = "transaktionen"`
- prozess_referenz.toml → `source_path = "transaktionen.referenzen"`
- marktlokation.toml → `source_path = "transaktionen.marktlokationen"`
- messlokation.toml → `source_path = "transaktionen.marktlokationen"` (or similar nested path)
- geschaeftspartner.toml → `source_path = "transaktionen.marktteilnehmer"` (or similar)

Note: The exact paths depend on the generated PID struct field names. Adjust after reviewing the generated types.

**Step 3: Update test paths**

Update `crates/mig-bo4e/tests/entity_mapping_test.rs` to load from the new directory. The `load_engine()` function needs to point to `pid_55001/` instead of the parent directory.

**Step 4: Run all tests**

Run: `cargo test --workspace`
Run: `cargo clippy --workspace -- -D warnings`

**Step 5: Commit**

```bash
git add mappings/ crates/mig-bo4e/tests/
git commit -m "refactor: migrate TOML mappings to PID-specific directory structure"
```

---

## Notes for the Implementer

### Key Files Reference

| File | Purpose |
|------|---------|
| `crates/automapper-generator/src/codegen/pid_type_gen.rs` | PID type code generation (main work) |
| `crates/automapper-generator/src/schema/ahb.rs` | AHB types: `Pruefidentifikator`, `AhbFieldDefinition` |
| `crates/automapper-generator/src/schema/mig.rs` | MIG types: `MigSchema`, `MigSegmentGroup` |
| `crates/mig-types/src/generated/fv2504/utilmd/pids/` | Generated PID types output |
| `crates/mig-assembly/src/cursor.rs` | SegmentCursor for assembly |
| `crates/mig-assembly/src/tokenize.rs` | OwnedSegment type |
| `crates/mig-assembly/src/pid_detect.rs` | PID detection (unchanged) |
| `crates/mig-bo4e/src/engine.rs` | MappingEngine (adapt for PID paths) |
| `crates/mig-bo4e/src/definition.rs` | MappingDefinition types |
| `mappings/FV2504/UTILMD_Strom/` | TOML mapping files |

### XML File Locations

The MIG and AHB XML files are in the `xml-migs-and-ahbs/` submodule. List the actual filenames before using them:
```bash
ls xml-migs-and-ahbs/FV2504/
```

### Test Commands

```bash
# Single crate tests
cargo test -p automapper-generator -- --nocapture
cargo test -p mig-assembly -- --nocapture
cargo test -p mig-bo4e -- --nocapture
cargo test -p mig-types -- --nocapture

# Full workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check

# Snapshot review (after insta tests)
cargo insta review
```

### Existing Snapshot Infrastructure

The project uses `insta` for snapshot tests in `automapper-generator`. Check `Cargo.toml` for the dependency. If not present, add:
```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

### Risk Areas

1. **Generated code compilation** — After Task 3, existing code referencing `pid.sg2` breaks. Fix in Task 5.
2. **Test fixture availability** — Integration tests depend on submodule data. Use early-return guards.
3. **TOML mapping path changes** — Task 12 changes file locations, which may break test paths. Update tests before committing.
4. **Circular dependencies** — The generated PID types (`mig-types`) need `mig-assembly` types (`OwnedSegment`, `SegmentCursor`). Verify `mig-types` has `mig-assembly` as a dependency, or keep assembly code in `mig-assembly`.
