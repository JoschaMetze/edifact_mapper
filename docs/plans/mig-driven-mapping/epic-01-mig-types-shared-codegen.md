---
feature: mig-driven-mapping
epic: 1
title: "mig-types Crate & Shared Segment Codegen"
depends_on: []
estimated_tasks: 7
crate: mig-types, automapper-generator
status: in_progress
---

# Epic 1: mig-types Crate & Shared Segment Codegen

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Create the `mig-types` crate and extend `automapper-generator` with a new codegen backend that reads MIG XML and emits shared Rust types — segment structs, composite structs, code enums, and segment group structs — into `mig-types/src/generated/`.

**Architecture:** The generator already parses MIG XML into `MigSchema` (see `automapper-generator::parsing::mig_parser`). A new codegen module `mig_type_gen` walks the `MigSchema` tree and emits Rust source files. Types are organized per message type and format version: `mig_types::generated::fv2504::utilmd::*`. Shared segment group types are reused across all PIDs.

**Tech Stack:** Rust, quick-xml (already in generator), automapper-generator schema types, serde (for derive on generated types)

---

## Task 1: Create mig-types Crate Stub

**Files:**
- Create: `crates/mig-types/Cargo.toml`
- Create: `crates/mig-types/src/lib.rs`
- Modify: `Cargo.toml` (workspace root — add member + dependency)

**Step 1: Add mig-types to workspace Cargo.toml**

Add `"crates/mig-types"` to the `[workspace] members` array. Add to `[workspace.dependencies]`:

```toml
mig-types = { path = "crates/mig-types" }
```

**Step 2: Create crate Cargo.toml**

Create `crates/mig-types/Cargo.toml`:

```toml
[package]
name = "mig-types"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Generated MIG-tree types for EDIFACT messages — shared segments, composites, enums, and PID-specific compositions"

[dependencies]
serde = { workspace = true, features = ["derive"] }

[dev-dependencies]
serde_json.workspace = true
```

**Step 3: Create lib.rs**

Create `crates/mig-types/src/lib.rs`:

```rust
//! Generated MIG-tree types for EDIFACT messages.
//!
//! Types are organized by format version and message type:
//! - `generated::fv2504::utilmd` — UTILMD types for FV2504
//!
//! Each message type module contains:
//! - `segments` — segment structs (SegNad, SegLoc, etc.)
//! - `composites` — composite data element structs
//! - `enums` — code list enums (NadQualifier, LocQualifier, etc.)
//! - `groups` — segment group structs (Sg2Party, Sg8SeqGroup, etc.)
//! - `pids` — per-PID composition structs

pub mod generated;
```

Create `crates/mig-types/src/generated/mod.rs`:

```rust
//! Auto-generated types from MIG/AHB XML schemas.
//! Do not edit manually — regenerate with `automapper-generator generate-mig-types`.
```

**Step 4: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-types/ Cargo.toml
git commit -m "feat(mig-types): add crate stub for generated MIG-tree types"
```

---

## Task 2: Code Enum Generation

**Files:**
- Create: `crates/automapper-generator/src/codegen/mig_type_gen.rs`
- Modify: `crates/automapper-generator/src/codegen/mod.rs`
- Create: `crates/automapper-generator/tests/mig_type_gen_test.rs`

**Step 1: Write the failing test**

Create `crates/automapper-generator/tests/mig_type_gen_test.rs`:

```rust
use automapper_generator::codegen::mig_type_gen;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_generate_enums_from_utilmd_mig() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    ).unwrap();

    let enums_source = mig_type_gen::generate_enums(&mig);

    // Should contain NadQualifier with MS, MR
    assert!(enums_source.contains("pub enum D3035Qualifier"), "Missing D3035 enum");
    assert!(enums_source.contains("MS"), "Missing MS variant");
    assert!(enums_source.contains("MR"), "Missing MR variant");

    // Should derive standard traits
    assert!(enums_source.contains("#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]"));

    // Should have Display impl for roundtrip to string
    assert!(enums_source.contains("impl std::fmt::Display for D3035Qualifier"));

    // Should have FromStr impl for parsing
    assert!(enums_source.contains("impl std::str::FromStr for D3035Qualifier"));

    // Should compile as valid Rust (syntax check via string inspection)
    assert!(!enums_source.contains("TODO"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_enums_from_utilmd_mig -- --nocapture`
Expected: FAIL — `mig_type_gen` module doesn't exist

**Step 3: Implement enum generation**

Create `crates/automapper-generator/src/codegen/mig_type_gen.rs`:

```rust
//! Code generation for MIG-tree Rust types.
//!
//! Reads `MigSchema` and emits Rust source code for:
//! - Code enums (one per data element with defined codes)
//! - Composite structs
//! - Segment structs
//! - Segment group structs

use std::collections::BTreeMap;
use crate::schema::mig::{MigSchema, MigSegment, MigSegmentGroup, MigComposite, MigDataElement};
use crate::schema::common::CodeDefinition;

/// Collect all data elements with codes from the entire MIG tree.
/// Returns a map of element_id -> Vec<CodeDefinition>, deduplicated.
fn collect_code_elements(mig: &MigSchema) -> BTreeMap<String, Vec<CodeDefinition>> {
    let mut result: BTreeMap<String, Vec<CodeDefinition>> = BTreeMap::new();

    fn visit_data_element(de: &MigDataElement, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        if !de.codes.is_empty() {
            let entry = result.entry(de.id.clone()).or_default();
            for code in &de.codes {
                if !entry.iter().any(|c| c.value == code.value) {
                    entry.push(code.clone());
                }
            }
        }
    }

    fn visit_composite(comp: &MigComposite, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        for de in &comp.data_elements {
            visit_data_element(de, result);
        }
    }

    fn visit_segment(seg: &MigSegment, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        for de in &seg.data_elements {
            visit_data_element(de, result);
        }
        for comp in &seg.composites {
            visit_composite(comp, result);
        }
    }

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, Vec<CodeDefinition>>) {
        for seg in &group.segments {
            visit_segment(seg, result);
        }
        for nested in &group.nested_groups {
            visit_group(nested, result);
        }
    }

    for seg in &mig.segments {
        visit_segment(seg, &mut result);
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }

    result
}

/// Sanitize a code value into a valid Rust identifier.
/// E.g., "Z01" stays "Z01", "293" becomes "_293", spaces become underscores.
fn sanitize_variant_name(value: &str) -> String {
    let trimmed = value.trim();
    let mut name = String::new();
    for ch in trimmed.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            name.push('_');
        }
    }
    if name.is_empty() {
        return "Empty".to_string();
    }
    if name.chars().next().unwrap().is_ascii_digit() {
        name = format!("_{name}");
    }
    name
}

/// Generate Rust enum definitions for all data elements that have code lists.
pub fn generate_enums(mig: &MigSchema) -> String {
    let code_elements = collect_code_elements(mig);
    let mut out = String::new();

    out.push_str("//! Auto-generated code enums from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("use serde::{Serialize, Deserialize};\n\n");

    for (element_id, codes) in &code_elements {
        let enum_name = format!("D{element_id}Qualifier");

        // Derive block
        out.push_str("#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub enum {enum_name} {{\n"));

        for code in codes {
            let variant = sanitize_variant_name(&code.value);
            if let Some(desc) = &code.description {
                if !desc.is_empty() {
                    out.push_str(&format!("    /// {desc}\n"));
                }
            }
            out.push_str(&format!("    {variant},\n"));
        }

        // Unknown variant for forward compatibility
        out.push_str("    /// Unrecognized code value\n");
        out.push_str("    Unknown(String),\n");
        out.push_str("}\n\n");

        // Display impl
        out.push_str(&format!("impl std::fmt::Display for {enum_name} {{\n"));
        out.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
        out.push_str("        match self {\n");
        for code in codes {
            let variant = sanitize_variant_name(&code.value);
            let raw = code.value.trim();
            out.push_str(&format!("            Self::{variant} => write!(f, \"{raw}\"),\n"));
        }
        out.push_str("            Self::Unknown(s) => write!(f, \"{}\", s),\n");
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // FromStr impl
        out.push_str(&format!("impl std::str::FromStr for {enum_name} {{\n"));
        out.push_str("    type Err = std::convert::Infallible;\n\n");
        out.push_str("    fn from_str(s: &str) -> Result<Self, Self::Err> {\n");
        out.push_str("        Ok(match s.trim() {\n");
        for code in codes {
            let variant = sanitize_variant_name(&code.value);
            let raw = code.value.trim();
            out.push_str(&format!("            \"{raw}\" => Self::{variant},\n"));
        }
        out.push_str("            other => Self::Unknown(other.to_string()),\n");
        out.push_str("        })\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");
    }

    out
}
```

Add `pub mod mig_type_gen;` to `crates/automapper-generator/src/codegen/mod.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_enums_from_utilmd_mig -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/mig_type_gen.rs crates/automapper-generator/src/codegen/mod.rs crates/automapper-generator/tests/mig_type_gen_test.rs
git commit -m "feat(generator): add code enum generation from MIG data elements"
```

---

## Task 3: Composite Struct Generation

**Files:**
- Modify: `crates/automapper-generator/src/codegen/mig_type_gen.rs`
- Modify: `crates/automapper-generator/tests/mig_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `mig_type_gen_test.rs`:

```rust
#[test]
fn test_generate_composites_from_utilmd_mig() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    ).unwrap();

    let composites_source = mig_type_gen::generate_composites(&mig);

    // Should contain C082 (party identification)
    assert!(composites_source.contains("pub struct CompositeC082"), "Missing C082");
    // Should contain C517 (location identification)
    assert!(composites_source.contains("pub struct CompositeC517"), "Missing C517");
    // Fields should use Option for conditional elements
    assert!(composites_source.contains("Option<String>"));
    // Fields with code lists should reference the enum type
    assert!(composites_source.contains("D3055Qualifier") || composites_source.contains("Option<D3055Qualifier>"));
    // Should derive Serialize, Deserialize
    assert!(composites_source.contains("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_composites_from_utilmd_mig -- --nocapture`
Expected: FAIL — `generate_composites` doesn't exist

**Step 3: Implement composite generation**

Add to `mig_type_gen.rs`:

```rust
/// Collect all unique composites from the MIG tree.
/// Returns a map of composite_id -> MigComposite (first occurrence wins).
fn collect_composites(mig: &MigSchema) -> BTreeMap<String, MigComposite> {
    let mut result: BTreeMap<String, MigComposite> = BTreeMap::new();

    fn visit_segment(seg: &MigSegment, result: &mut BTreeMap<String, MigComposite>) {
        for comp in &seg.composites {
            result.entry(comp.id.clone()).or_insert_with(|| comp.clone());
        }
    }

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, MigComposite>) {
        for seg in &group.segments {
            visit_segment(seg, result);
        }
        for nested in &group.nested_groups {
            visit_group(nested, result);
        }
    }

    for seg in &mig.segments {
        visit_segment(seg, &mut result);
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }
    result
}

/// Determine the Rust type for a data element field.
fn data_element_type(de: &MigDataElement, code_elements: &BTreeMap<String, Vec<CodeDefinition>>) -> String {
    if code_elements.contains_key(&de.id) {
        format!("D{}Qualifier", de.id)
    } else {
        "String".to_string()
    }
}

/// Determine if a data element is optional based on status.
fn is_optional(status_spec: &Option<String>, status_std: &Option<String>) -> bool {
    let status = status_spec.as_deref().or(status_std.as_deref()).unwrap_or("C");
    matches!(status, "C" | "O" | "N" | "D")
}

/// Generate Rust struct definitions for all composites in the MIG.
pub fn generate_composites(mig: &MigSchema) -> String {
    let composites = collect_composites(mig);
    let code_elements = collect_code_elements(mig);
    let mut out = String::new();

    out.push_str("//! Auto-generated composite structs from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("use serde::{Serialize, Deserialize};\n");
    out.push_str("use super::enums::*;\n\n");

    for (comp_id, comp) in &composites {
        let struct_name = format!("Composite{comp_id}");

        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));

        for de in &comp.data_elements {
            let field_name = format!("d{}", de.id);
            let base_type = data_element_type(de, &code_elements);
            let optional = is_optional(&de.status_spec, &de.status_std);

            if let Some(name) = &de.description.as_ref().or(Some(&de.name)) {
                if !name.is_empty() {
                    out.push_str(&format!("    /// {name}\n"));
                }
            }
            if optional {
                out.push_str(&format!("    pub {field_name}: Option<{base_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {base_type},\n"));
            }
        }

        out.push_str("}\n\n");
    }

    out
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_composites_from_utilmd_mig -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/mig_type_gen.rs crates/automapper-generator/tests/mig_type_gen_test.rs
git commit -m "feat(generator): add composite struct generation from MIG composites"
```

---

## Task 4: Segment Struct Generation

**Files:**
- Modify: `crates/automapper-generator/src/codegen/mig_type_gen.rs`
- Modify: `crates/automapper-generator/tests/mig_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `mig_type_gen_test.rs`:

```rust
#[test]
fn test_generate_segments_from_utilmd_mig() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    ).unwrap();

    let segments_source = mig_type_gen::generate_segments(&mig);

    // Should contain NAD segment
    assert!(segments_source.contains("pub struct SegNad"), "Missing SegNad");
    // Should contain LOC segment
    assert!(segments_source.contains("pub struct SegLoc"), "Missing SegLoc");
    // Should contain IDE segment
    assert!(segments_source.contains("pub struct SegIde"), "Missing SegIde");
    // Segments should reference composites
    assert!(segments_source.contains("CompositeC082") || segments_source.contains("Option<CompositeC082>"));
    // Segments should have direct data element fields too
    assert!(segments_source.contains("d3035"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_segments_from_utilmd_mig -- --nocapture`
Expected: FAIL — `generate_segments` doesn't exist

**Step 3: Implement segment generation**

Add to `mig_type_gen.rs`:

```rust
/// Collect all unique segments from the MIG tree.
fn collect_segments(mig: &MigSchema) -> BTreeMap<String, MigSegment> {
    let mut result: BTreeMap<String, MigSegment> = BTreeMap::new();

    fn visit_group(group: &MigSegmentGroup, result: &mut BTreeMap<String, MigSegment>) {
        for seg in &group.segments {
            result.entry(seg.id.clone()).or_insert_with(|| seg.clone());
        }
        for nested in &group.nested_groups {
            visit_group(nested, result);
        }
    }

    for seg in &mig.segments {
        result.entry(seg.id.clone()).or_insert_with(|| seg.clone());
    }
    for group in &mig.segment_groups {
        visit_group(group, &mut result);
    }
    result
}

/// Generate Rust struct definitions for all segments in the MIG.
pub fn generate_segments(mig: &MigSchema) -> String {
    let segments = collect_segments(mig);
    let code_elements = collect_code_elements(mig);
    let mut out = String::new();

    out.push_str("//! Auto-generated segment structs from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("use serde::{Serialize, Deserialize};\n");
    out.push_str("use super::enums::*;\n");
    out.push_str("use super::composites::*;\n\n");

    for (seg_id, seg) in &segments {
        let struct_name = format!("Seg{}", capitalize_segment_id(seg_id));

        if let Some(name) = &seg.description.as_ref().or(Some(&seg.name)) {
            out.push_str(&format!("/// {} segment — {}\n", seg_id, name));
        }
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));

        // Direct data elements
        for de in &seg.data_elements {
            let field_name = format!("d{}", de.id);
            let base_type = data_element_type(de, &code_elements);
            let optional = is_optional(&de.status_spec, &de.status_std);

            if optional {
                out.push_str(&format!("    pub {field_name}: Option<{base_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {base_type},\n"));
            }
        }

        // Composites
        for comp in &seg.composites {
            let field_name = format!("c{}", comp.id.to_lowercase());
            let comp_type = format!("Composite{}", comp.id);
            let optional = is_optional(&comp.status_spec, &comp.status_std);

            if optional {
                out.push_str(&format!("    pub {field_name}: Option<{comp_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {comp_type},\n"));
            }
        }

        out.push_str("}\n\n");
    }

    out
}

/// Capitalize a segment ID for struct naming: "NAD" -> "Nad", "UNH" -> "Unh"
fn capitalize_segment_id(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let rest: String = chars.map(|c| c.to_ascii_lowercase()).collect();
            format!("{}{}", first.to_ascii_uppercase(), rest)
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_segments_from_utilmd_mig -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/mig_type_gen.rs crates/automapper-generator/tests/mig_type_gen_test.rs
git commit -m "feat(generator): add segment struct generation from MIG segments"
```

---

## Task 5: Segment Group Generation

**Files:**
- Modify: `crates/automapper-generator/src/codegen/mig_type_gen.rs`
- Modify: `crates/automapper-generator/tests/mig_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `mig_type_gen_test.rs`:

```rust
#[test]
fn test_generate_groups_from_utilmd_mig() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    ).unwrap();

    let groups_source = mig_type_gen::generate_groups(&mig);

    // Should contain SG2 (party group)
    assert!(groups_source.contains("pub struct Sg2"), "Missing SG2 group");
    // Should contain SG4 (transaction group)
    assert!(groups_source.contains("pub struct Sg4"), "Missing SG4 group");
    // Should contain SG8 (SEQ entity group)
    assert!(groups_source.contains("pub struct Sg8"), "Missing SG8 group");
    // Groups should reference segments
    assert!(groups_source.contains("SegNad") || groups_source.contains("SegIde"));
    // Groups with max_rep > 1 children should use Vec
    assert!(groups_source.contains("Vec<"));
    // Nested groups should reference child group types
    assert!(groups_source.contains("Sg3") || groups_source.contains("Sg9") || groups_source.contains("Sg10"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_groups_from_utilmd_mig -- --nocapture`
Expected: FAIL — `generate_groups` doesn't exist

**Step 3: Implement segment group generation**

Add to `mig_type_gen.rs`:

```rust
/// Generate Rust struct definitions for all segment groups in the MIG.
/// Groups compose segment types and nested group types.
pub fn generate_groups(mig: &MigSchema) -> String {
    let mut out = String::new();

    out.push_str("//! Auto-generated segment group structs from MIG XML.\n");
    out.push_str("//! Do not edit manually.\n\n");
    out.push_str("use serde::{Serialize, Deserialize};\n");
    out.push_str("use super::segments::*;\n\n");

    fn emit_group(group: &MigSegmentGroup, out: &mut String) {
        let struct_name = format!("Sg{}", group.id.trim_start_matches("SG"));

        if !group.name.is_empty() {
            out.push_str(&format!("/// {} — {}\n", group.id, group.name));
        }
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {struct_name} {{\n"));

        // Segments in this group
        for seg in &group.segments {
            let field_name = seg.id.to_lowercase();
            let seg_type = format!("Seg{}", capitalize_segment_id(&seg.id));
            let optional = is_optional(&seg.status_spec, &seg.status_std);
            let repeating = seg.max_rep() > 1;

            if repeating {
                out.push_str(&format!("    pub {field_name}: Vec<{seg_type}>,\n"));
            } else if optional {
                out.push_str(&format!("    pub {field_name}: Option<{seg_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {field_name}: {seg_type},\n"));
            }
        }

        // Nested groups
        for nested in &group.nested_groups {
            let nested_name = format!("sg{}", nested.id.trim_start_matches("SG").to_lowercase());
            let nested_type = format!("Sg{}", nested.id.trim_start_matches("SG"));
            let repeating = nested.max_rep_spec > 1 || nested.max_rep_std > 1;
            let optional = is_optional(&nested.status_spec, &nested.status_std);

            if repeating {
                out.push_str(&format!("    pub {nested_name}: Vec<{nested_type}>,\n"));
            } else if optional {
                out.push_str(&format!("    pub {nested_name}: Option<{nested_type}>,\n"));
            } else {
                out.push_str(&format!("    pub {nested_name}: {nested_type},\n"));
            }
        }

        out.push_str("}\n\n");

        // Recurse into nested groups
        for nested in &group.nested_groups {
            emit_group(nested, out);
        }
    }

    for group in &mig.segment_groups {
        emit_group(group, &mut out);
    }

    out
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_groups_from_utilmd_mig -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/mig_type_gen.rs crates/automapper-generator/tests/mig_type_gen_test.rs
git commit -m "feat(generator): add segment group struct generation from MIG groups"
```

---

## Task 6: Full Module File Generation + Write to Disk

**Files:**
- Modify: `crates/automapper-generator/src/codegen/mig_type_gen.rs`
- Modify: `crates/automapper-generator/src/codegen/mod.rs`
- Modify: `crates/automapper-generator/tests/mig_type_gen_test.rs`

**Step 1: Write the failing test**

Add to `mig_type_gen_test.rs`:

```rust
#[test]
fn test_generate_mig_types_writes_files() {
    let output_dir = tempfile::tempdir().unwrap();

    mig_type_gen::generate_mig_types(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD",
        Some("Strom"),
        "FV2504",
        output_dir.path(),
    ).unwrap();

    // Should create version/message module structure
    let base = output_dir.path().join("fv2504").join("utilmd");
    assert!(base.join("enums.rs").exists(), "Missing enums.rs");
    assert!(base.join("composites.rs").exists(), "Missing composites.rs");
    assert!(base.join("segments.rs").exists(), "Missing segments.rs");
    assert!(base.join("groups.rs").exists(), "Missing groups.rs");
    assert!(base.join("mod.rs").exists(), "Missing mod.rs");

    // mod.rs should re-export all modules
    let mod_content = std::fs::read_to_string(base.join("mod.rs")).unwrap();
    assert!(mod_content.contains("pub mod enums;"));
    assert!(mod_content.contains("pub mod composites;"));
    assert!(mod_content.contains("pub mod segments;"));
    assert!(mod_content.contains("pub mod groups;"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-generator test_generate_mig_types_writes_files -- --nocapture`
Expected: FAIL — `generate_mig_types` doesn't exist

**Step 3: Implement file generation orchestrator**

Add to `mig_type_gen.rs`:

```rust
use std::path::Path;
use crate::parsing::mig_parser::parse_mig;
use crate::error::GeneratorError;

/// Generate all MIG type files for a given MIG XML and write them to disk.
///
/// Creates the directory structure:
///   {output_dir}/{fv_lower}/{msg_lower}/enums.rs
///   {output_dir}/{fv_lower}/{msg_lower}/composites.rs
///   {output_dir}/{fv_lower}/{msg_lower}/segments.rs
///   {output_dir}/{fv_lower}/{msg_lower}/groups.rs
///   {output_dir}/{fv_lower}/{msg_lower}/mod.rs
pub fn generate_mig_types(
    mig_path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError> {
    let mig = parse_mig(mig_path, message_type, variant, format_version)?;

    let fv_lower = format_version.to_lowercase();
    let msg_lower = message_type.to_lowercase();
    let base_dir = output_dir.join(&fv_lower).join(&msg_lower);
    std::fs::create_dir_all(&base_dir)?;

    std::fs::write(base_dir.join("enums.rs"), generate_enums(&mig))?;
    std::fs::write(base_dir.join("composites.rs"), generate_composites(&mig))?;
    std::fs::write(base_dir.join("segments.rs"), generate_segments(&mig))?;
    std::fs::write(base_dir.join("groups.rs"), generate_groups(&mig))?;

    let mod_rs = format!(
        "//! Generated UTILMD types for {format_version}.\n\
         //! Do not edit manually.\n\n\
         pub mod enums;\n\
         pub mod composites;\n\
         pub mod segments;\n\
         pub mod groups;\n"
    );
    std::fs::write(base_dir.join("mod.rs"), mod_rs)?;

    // Write parent mod.rs files
    let fv_mod = format!("pub mod {msg_lower};\n");
    std::fs::write(output_dir.join(&fv_lower).join("mod.rs"), fv_mod)?;

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-generator test_generate_mig_types_writes_files -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/codegen/mig_type_gen.rs crates/automapper-generator/tests/mig_type_gen_test.rs
git commit -m "feat(generator): add file-writing orchestrator for MIG type generation"
```

---

## Task 7: Generate and Compile Real UTILMD Types

**Files:**
- Modify: `crates/mig-types/src/generated/mod.rs`
- Generate: `crates/mig-types/src/generated/fv2504/utilmd/*.rs`

**Step 1: Run the generator against real MIG XML**

Run: `cargo run -p automapper-generator -- generate-mig-types --mig xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml --output crates/mig-types/src/generated/ --format-version FV2504 --message-type UTILMD --variant Strom`

(If the CLI subcommand doesn't exist yet, add it to `main.rs` first, or run the generation via a test.)

Alternatively, create a test in `mig-types`:

```rust
// crates/mig-types/tests/compile_test.rs
#[test]
fn test_generated_types_compile() {
    // This test passing means the generated code compiles
    use mig_types::generated::fv2504::utilmd::enums::*;
    use mig_types::generated::fv2504::utilmd::composites::*;
    use mig_types::generated::fv2504::utilmd::segments::*;
    use mig_types::generated::fv2504::utilmd::groups::*;
}
```

**Step 2: Update generated/mod.rs to include the version module**

Update `crates/mig-types/src/generated/mod.rs`:

```rust
//! Auto-generated types from MIG/AHB XML schemas.
//! Do not edit manually — regenerate with `automapper-generator generate-mig-types`.

pub mod fv2504;
```

**Step 3: Fix any compilation errors in generated code**

Run: `cargo check -p mig-types`
Expected: PASS (may require iteration to fix edge cases in codegen)

**Step 4: Run cargo clippy**

Run: `cargo clippy -p mig-types -- -D warnings`
Expected: PASS (generated code may need `#[allow]` attributes — add them in codegen)

**Step 5: Commit generated code**

```bash
git add crates/mig-types/
git commit -m "feat(mig-types): generate and commit shared UTILMD types for FV2504"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | ~5 (enum gen, composite gen, segment gen, group gen, file write) |
| cargo check --workspace | PASS |
| cargo clippy --workspace | PASS |

Generated files:
- `crates/mig-types/src/generated/fv2504/utilmd/enums.rs`
- `crates/mig-types/src/generated/fv2504/utilmd/composites.rs`
- `crates/mig-types/src/generated/fv2504/utilmd/segments.rs`
- `crates/mig-types/src/generated/fv2504/utilmd/groups.rs`
- `crates/mig-types/src/generated/fv2504/utilmd/mod.rs`
- `crates/mig-types/src/generated/fv2504/mod.rs`
- `crates/mig-types/src/generated/mod.rs`
