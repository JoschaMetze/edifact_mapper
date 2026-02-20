---
feature: mig-driven-mapping
epic: 2
title: "PID-Specific Composition Codegen"
depends_on: [mig-driven-mapping/E01]
estimated_tasks: 6
crate: mig-types, automapper-generator
status: in_progress
---

# Epic 2: PID-Specific Composition Codegen

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Extend `automapper-generator` to read AHB XML and generate per-PID composition structs that compose the shared segment group types from Epic 1. Each PID gets a struct that contains only the fields applicable to that PID, with cardinality driven by AHB status. Qualifier-disambiguated segment groups (e.g., SG8 with SEQ+Z01 vs SEQ+Z78) become separate named fields.

**Architecture:** The AHB parser already produces `AhbSchema` with `Vec<Pruefidentifikator>`, each containing `Vec<AhbFieldDefinition>` with segment paths like `"SG2/NAD/C082/3039"`. A new codegen module cross-references these paths against the MIG tree to determine which segment groups, segments, and fields exist for each PID. The output is per-PID Rust structs that compose the shared `Sg*` types. Additionally, a `PidTree` trait is generated for common operations across all PID types.

**Tech Stack:** Rust, automapper-generator schema types (MigSchema, AhbSchema), mig-types shared types from Epic 1

---

## Task 1: AHB Path Analysis — Map AHB Fields to MIG Tree Nodes

**Files:**
- Create: `crates/automapper-generator/src/codegen/pid_type_gen.rs`
- Modify: `crates/automapper-generator/src/codegen/mod.rs`
- Create: `crates/automapper-generator/tests/pid_type_gen_test.rs`

**Step 1: Write the failing test**

Create `crates/automapper-generator/tests/pid_type_gen_test.rs`:

```rust
use automapper_generator::codegen::pid_type_gen;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use std::path::Path;

fn load_utilmd() -> (automapper_generator::schema::mig::MigSchema, automapper_generator::schema::ahb::AhbSchema) {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();
    let ahb = parse_ahb(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();
    (mig, ahb)
}

#[test]
fn test_analyze_pid_structure() {
    let (mig, ahb) = load_utilmd();

    // Pick a specific PID to analyze
    let pid = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();
    let structure = pid_type_gen::analyze_pid_structure(pid, &mig);

    // Should identify which top-level groups are present
    assert!(!structure.groups.is_empty(), "PID should have groups");
    // Should identify SG2 (NAD parties) as present
    assert!(structure.groups.iter().any(|g| g.group_id == "SG2"), "Missing SG2");
    // Should identify SG4 (transaction) as present
    assert!(structure.groups.iter().any(|g| g.group_id == "SG4"), "Missing SG4");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_analyze_pid_structure -- --nocapture`
Expected: FAIL — `pid_type_gen` module doesn't exist

**Step 3: Implement PID structure analysis**

Create `crates/automapper-generator/src/codegen/pid_type_gen.rs`:

```rust
//! Code generation for PID-specific composition types.
//!
//! Cross-references AHB field definitions against the MIG tree
//! to determine which segment groups, segments, and fields
//! exist for each PID.

use std::collections::{BTreeMap, BTreeSet};
use crate::schema::ahb::{Pruefidentifikator, AhbFieldDefinition};
use crate::schema::mig::MigSchema;

/// Analyzed structure of a single PID.
#[derive(Debug)]
pub struct PidStructure {
    pub pid_id: String,
    pub beschreibung: String,
    pub kommunikation_von: Option<String>,
    /// Top-level groups present in this PID.
    pub groups: Vec<PidGroupInfo>,
    /// Top-level segments (outside groups) present in this PID.
    pub top_level_segments: Vec<String>,
}

/// Information about a segment group's usage within a PID.
#[derive(Debug)]
pub struct PidGroupInfo {
    pub group_id: String,
    /// Qualifier values that disambiguate this group (e.g., SEQ+Z01).
    /// Empty if the group is not qualifier-disambiguated.
    pub qualifier_values: Vec<String>,
    /// AHB status for this group occurrence ("Muss", "Kann", etc.)
    pub ahb_status: String,
    /// Nested child groups present in this PID's usage.
    pub child_groups: Vec<PidGroupInfo>,
    /// Segments present in this group for this PID.
    pub segments: BTreeSet<String>,
}

/// Analyze which MIG tree nodes a PID uses, based on its AHB field definitions.
pub fn analyze_pid_structure(pid: &Pruefidentifikator, _mig: &MigSchema) -> PidStructure {
    let mut top_level_segments: BTreeSet<String> = BTreeSet::new();
    let mut group_map: BTreeMap<String, PidGroupInfo> = BTreeMap::new();

    for field in &pid.fields {
        let parts: Vec<&str> = field.segment_path.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        // Determine if path starts with a group (SGn) or a segment
        if parts[0].starts_with("SG") {
            let group_id = parts[0].to_string();
            let entry = group_map.entry(group_id.clone()).or_insert_with(|| PidGroupInfo {
                group_id: group_id.clone(),
                qualifier_values: Vec::new(),
                ahb_status: field.ahb_status.clone(),
                child_groups: Vec::new(),
                segments: BTreeSet::new(),
            });

            // If there's a segment in the path, record it
            if parts.len() > 1 && !parts[1].starts_with("SG") {
                entry.segments.insert(parts[1].to_string());
            }

            // Handle nested groups (SG4/SG8/...)
            if parts.len() > 1 && parts[1].starts_with("SG") {
                let child_id = parts[1].to_string();
                if !entry.child_groups.iter().any(|c| c.group_id == child_id) {
                    let mut child_segments = BTreeSet::new();
                    if parts.len() > 2 && !parts[2].starts_with("SG") {
                        child_segments.insert(parts[2].to_string());
                    }
                    entry.child_groups.push(PidGroupInfo {
                        group_id: child_id,
                        qualifier_values: Vec::new(),
                        ahb_status: field.ahb_status.clone(),
                        child_groups: Vec::new(),
                        segments: child_segments,
                    });
                } else if parts.len() > 2 && !parts[2].starts_with("SG") {
                    if let Some(child) = entry.child_groups.iter_mut().find(|c| c.group_id == child_id) {
                        child.segments.insert(parts[2].to_string());
                    }
                }
            }
        } else {
            // Top-level segment (not in a group)
            top_level_segments.insert(parts[0].to_string());
        }
    }

    PidStructure {
        pid_id: pid.id.clone(),
        beschreibung: pid.beschreibung.clone(),
        kommunikation_von: pid.kommunikation_von.clone(),
        groups: group_map.into_values().collect(),
        top_level_segments: top_level_segments.into_iter().collect(),
    }
}
```

Add `pub mod pid_type_gen;` to `crates/automapper-generator/src/codegen/mod.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_analyze_pid_structure -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs crates/automapper-generator/src/codegen/mod.rs crates/automapper-generator/tests/pid_type_gen_test.rs
git commit -m "feat(generator): add PID structure analysis from AHB field definitions"
```

---

## Task 2: Qualifier Disambiguation Detection

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs`
- Modify: `crates/automapper-generator/tests/pid_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `pid_type_gen_test.rs`:

```rust
#[test]
fn test_detect_qualifier_disambiguation() {
    let (mig, ahb) = load_utilmd();

    // Find a PID that has multiple SG8 usages with different SEQ qualifiers
    // (e.g., SEQ+Z01 for MaLo, SEQ+Z78 for Zuordnung)
    let pid = ahb.workflows.iter()
        .find(|w| {
            let structure = pid_type_gen::analyze_pid_structure(w, &mig);
            structure.groups.iter().any(|g| g.group_id == "SG4")
        })
        .expect("Should find a PID with SG4");

    let structure = pid_type_gen::analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

    // Check that SG8 groups under SG4 are disambiguated by qualifier
    let sg4 = structure.groups.iter().find(|g| g.group_id == "SG4").unwrap();
    let sg8_groups: Vec<_> = sg4.child_groups.iter().filter(|g| g.group_id == "SG8").collect();

    // If this PID has multiple SG8 usages, they should have different qualifier_values
    if sg8_groups.len() > 1 {
        let qualifiers: Vec<_> = sg8_groups.iter().map(|g| &g.qualifier_values).collect();
        assert!(qualifiers.windows(2).all(|w| w[0] != w[1]),
            "SG8 groups should be disambiguated by qualifier: {:?}", qualifiers);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_detect_qualifier_disambiguation -- --nocapture`
Expected: FAIL — `analyze_pid_structure_with_qualifiers` doesn't exist

**Step 3: Implement qualifier detection**

Add `analyze_pid_structure_with_qualifiers` to `pid_type_gen.rs`. This enhanced version examines the AHB code values within SEQ/NAD/LOC segments to determine qualifier values that disambiguate repeated groups. For each group occurrence, find the qualifying segment (typically the first segment in the group that has a fixed code value), extract its value, and attach it to `PidGroupInfo.qualifier_values`.

Key logic: scan `AhbFieldDefinition.codes` for entries where `ahb_status` is `"X"` — these are the fixed qualifier values for that PID.

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_detect_qualifier_disambiguation -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs crates/automapper-generator/tests/pid_type_gen_test.rs
git commit -m "feat(generator): detect qualifier disambiguation for repeated segment groups"
```

---

## Task 3: PID Struct Generation

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs`
- Modify: `crates/automapper-generator/tests/pid_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `pid_type_gen_test.rs`:

```rust
#[test]
fn test_generate_pid_struct() {
    let (mig, ahb) = load_utilmd();
    let pid = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();

    let pid_source = pid_type_gen::generate_pid_struct(pid, &mig, &ahb);

    // Should generate a struct named Pid55001
    assert!(pid_source.contains("pub struct Pid55001"), "Missing Pid55001 struct");
    // Should have doc comment with description
    assert!(pid_source.contains(&pid.beschreibung));
    // Should compose shared group types
    assert!(pid_source.contains("Sg2") || pid_source.contains("Sg4"));
    // Should derive standard traits
    assert!(pid_source.contains("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"));
    // Muss fields should not be Option
    // Kann fields should be Option or Vec
    // Should contain transaction-level sub-struct if SG4 is present
    if pid_source.contains("sg4") {
        assert!(pid_source.contains("pub struct Pid55001Vorgang") || pid_source.contains("Vec<Sg4>"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_pid_struct -- --nocapture`
Expected: FAIL — `generate_pid_struct` doesn't exist

**Step 3: Implement PID struct generation**

Add to `pid_type_gen.rs`:

```rust
/// Generate a Rust struct for a specific PID that composes shared segment group types.
pub fn generate_pid_struct(
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    ahb: &AhbSchema,
) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let mut out = String::new();

    let struct_name = format!("Pid{}", pid.id);

    out.push_str(&format!("/// PID {}: {}\n", pid.id, pid.beschreibung));
    if let Some(ref komm) = pid.kommunikation_von {
        out.push_str(&format!("/// Kommunikation: {komm}\n"));
    }
    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    // Top-level segments
    for seg_id in &structure.top_level_segments {
        let field_name = seg_id.to_lowercase();
        let seg_type = format!("super::segments::Seg{}", capitalize_segment_id(seg_id));
        out.push_str(&format!("    pub {field_name}: {seg_type},\n"));
    }

    // Top-level groups
    for group in &structure.groups {
        emit_pid_group_field(group, &mut out, "    ");
    }

    out.push_str("}\n\n");

    // Generate transaction-level sub-structs for repeating groups with children
    // (e.g., Pid55001Vorgang for SG4 contents)
    for group in &structure.groups {
        if !group.child_groups.is_empty() {
            emit_pid_transaction_struct(&struct_name, group, &mut out);
        }
    }

    out
}

fn emit_pid_group_field(group: &PidGroupInfo, out: &mut String, indent: &str) {
    let is_muss = group.ahb_status.contains("Muss") || group.ahb_status.contains("X");
    let group_type = format!("super::groups::Sg{}", group.group_id.trim_start_matches("SG"));

    // Generate field name from group + qualifier
    let field_name = if group.qualifier_values.is_empty() {
        group.group_id.to_lowercase()
    } else {
        format!("{}_{}", group.group_id.to_lowercase(),
                group.qualifier_values.join("_").to_lowercase())
    };

    if is_muss {
        out.push_str(&format!("{indent}pub {field_name}: Vec<{group_type}>,\n"));
    } else {
        out.push_str(&format!("{indent}pub {field_name}: Vec<{group_type}>,\n"));
    }
}

fn emit_pid_transaction_struct(parent_name: &str, group: &PidGroupInfo, out: &mut String) {
    // Generates e.g. Pid55001Sg4 for the SG4 transaction contents
    let struct_name = format!("{parent_name}{}", group.group_id);

    out.push_str(&format!("/// Transaction-level structure for {} in {}\n", group.group_id, parent_name));
    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    for child in &group.child_groups {
        emit_pid_group_field(child, out, "    ");
    }

    out.push_str("}\n\n");
}
```

Helper `capitalize_segment_id` can be imported from `mig_type_gen` or moved to a shared utility.

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_pid_struct -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs crates/automapper-generator/tests/pid_type_gen_test.rs
git commit -m "feat(generator): generate PID-specific composition structs from AHB"
```

---

## Task 4: PidTree Trait Generation

**Files:**
- Modify: `crates/mig-types/src/lib.rs`
- Create: `crates/mig-types/src/traits.rs`

**Step 1: Write the failing test**

Create `crates/mig-types/tests/pid_tree_trait_test.rs`:

```rust
use mig_types::traits::PidTree;

// This test verifies the trait exists and has the expected methods
#[test]
fn test_pid_tree_trait_exists() {
    // PidTree should be object-safe for dynamic dispatch
    fn _assert_object_safe(_: &dyn PidTree) {}
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-types test_pid_tree_trait_exists`
Expected: FAIL — `traits` module doesn't exist

**Step 3: Implement PidTree trait**

Create `crates/mig-types/src/traits.rs`:

```rust
//! Common traits for MIG tree types.

/// Trait implemented by all generated PID-specific tree types.
///
/// Provides common operations that work across any PID structure:
/// identification, metadata access, and serialization.
pub trait PidTree: std::fmt::Debug + Send + Sync {
    /// The PID identifier (e.g., "55001").
    fn pid_id(&self) -> &str;

    /// Human-readable description of this PID.
    fn beschreibung(&self) -> &str;

    /// Communication direction (e.g., "NB an LF").
    fn kommunikation_von(&self) -> Option<&str>;

    /// The message type (e.g., "UTILMD").
    fn message_type(&self) -> &str;

    /// The format version (e.g., "FV2504").
    fn format_version(&self) -> &str;
}
```

Add `pub mod traits;` to `crates/mig-types/src/lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-types test_pid_tree_trait_exists`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-types/src/traits.rs crates/mig-types/src/lib.rs crates/mig-types/tests/
git commit -m "feat(mig-types): add PidTree trait for common operations across PID types"
```

---

## Task 5: Full PID File Generation + Write to Disk

**Files:**
- Modify: `crates/automapper-generator/src/codegen/pid_type_gen.rs`
- Modify: `crates/automapper-generator/tests/pid_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `pid_type_gen_test.rs`:

```rust
#[test]
fn test_generate_pid_types_writes_files() {
    let output_dir = tempfile::tempdir().unwrap();
    let (mig, ahb) = load_utilmd();

    pid_type_gen::generate_pid_types(
        &mig,
        &ahb,
        "FV2504",
        output_dir.path(),
    ).unwrap();

    let pids_dir = output_dir.path().join("fv2504").join("utilmd").join("pids");
    assert!(pids_dir.exists(), "pids/ directory should exist");

    // Should generate a file for each PID
    let pid_files: Vec<_> = std::fs::read_dir(&pids_dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
        .collect();

    // Should have at least some PID files (187 PIDs in UTILMD Strom)
    assert!(pid_files.len() > 10, "Expected many PID files, got {}", pid_files.len());

    // Should have a mod.rs
    assert!(pids_dir.join("mod.rs").exists(), "Missing pids/mod.rs");

    // mod.rs should declare all PID modules
    let mod_content = std::fs::read_to_string(pids_dir.join("mod.rs")).unwrap();
    assert!(mod_content.contains("pub mod pid55001") || mod_content.contains("pub mod pid_55001"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_pid_types_writes_files -- --nocapture`
Expected: FAIL — `generate_pid_types` doesn't exist

**Step 3: Implement PID file generation orchestrator**

Add to `pid_type_gen.rs`:

```rust
use std::path::Path;
use crate::schema::ahb::AhbSchema;
use crate::error::GeneratorError;

/// Generate all PID composition type files for a given AHB and write to disk.
///
/// Creates: {output_dir}/{fv_lower}/{msg_lower}/pids/pid_{id}.rs + mod.rs
pub fn generate_pid_types(
    mig: &MigSchema,
    ahb: &AhbSchema,
    format_version: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError> {
    let fv_lower = format_version.to_lowercase();
    let msg_lower = ahb.message_type.to_lowercase();
    let pids_dir = output_dir.join(&fv_lower).join(&msg_lower).join("pids");
    std::fs::create_dir_all(&pids_dir)?;

    let mut mod_entries = Vec::new();

    for pid in &ahb.workflows {
        let source = generate_pid_struct(pid, mig, ahb);
        let filename = format!("pid_{}.rs", pid.id.to_lowercase());
        let module_name = format!("pid_{}", pid.id.to_lowercase());

        let full_source = format!(
            "//! Auto-generated PID {} types.\n//! {}\n//! Do not edit manually.\n\n\
             use serde::{{Serialize, Deserialize}};\n\n{source}",
            pid.id, pid.beschreibung
        );

        std::fs::write(pids_dir.join(&filename), full_source)?;
        mod_entries.push(module_name);
    }

    // Write mod.rs
    let mut mod_rs = String::from("//! Per-PID composition types.\n//! Do not edit manually.\n\n");
    for module in &mod_entries {
        mod_rs.push_str(&format!("pub mod {module};\n"));
    }
    std::fs::write(pids_dir.join("mod.rs"), mod_rs)?;

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_pid_types_writes_files -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/pid_type_gen.rs crates/automapper-generator/tests/pid_type_gen_test.rs
git commit -m "feat(generator): add PID type file generation and disk writer"
```

---

## Task 6: Generate and Compile Real PID Types

**Files:**
- Modify: `crates/mig-types/src/generated/fv2504/utilmd/mod.rs`
- Generate: `crates/mig-types/src/generated/fv2504/utilmd/pids/*.rs`

**Step 1: Run the generator to produce PID types**

Use a test or CLI command to generate PID types into `crates/mig-types/src/generated/fv2504/utilmd/pids/`.

**Step 2: Update mod.rs to include pids module**

Update `crates/mig-types/src/generated/fv2504/utilmd/mod.rs`:

```rust
pub mod enums;
pub mod composites;
pub mod segments;
pub mod groups;
pub mod pids;
```

**Step 3: Fix compilation errors**

Run: `cargo check -p mig-types`
Expected: PASS (may require iteration — generated PID structs must correctly reference shared types)

**Step 4: Run cargo clippy**

Run: `cargo clippy -p mig-types -- -D warnings`
Expected: PASS (add `#[allow]` attributes in codegen as needed for generated code)

**Step 5: Write a smoke test**

Create `crates/mig-types/tests/pid_compile_test.rs`:

```rust
#[test]
fn test_pid_types_instantiable() {
    // Verify a few PID types can be referenced and are real types
    let _: Option<mig_types::generated::fv2504::utilmd::pids::pid_55001::Pid55001> = None;
}
```

Run: `cargo test -p mig-types test_pid_types_instantiable`
Expected: PASS

**Step 6: Commit generated code**

```bash
git add crates/mig-types/
git commit -m "feat(mig-types): generate and commit PID-specific composition types for UTILMD FV2504"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | ~5 (structure analysis, qualifier disambiguation, struct gen, file write, compile smoke) |
| Generated PID files | ~187 (one per UTILMD Strom PID) |
| cargo check --workspace | PASS |
| cargo clippy --workspace | PASS |
