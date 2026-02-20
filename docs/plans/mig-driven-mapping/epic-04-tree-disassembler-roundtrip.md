---
feature: mig-driven-mapping
epic: 4
title: "mig-assembly — Tree Disassembler & Roundtrip"
depends_on: [mig-driven-mapping/E03]
estimated_tasks: 5
crate: mig-assembly
status: in_progress
---

# Epic 4: mig-assembly — Tree Disassembler & Roundtrip

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Implement the tree disassembler (the reverse of the assembler) that walks an `AssembledTree` in MIG order and emits `Vec<RawSegment>`, plus EDIFACT rendering. Validate roundtrip fidelity: `EDIFACT → assemble → disassemble → render` produces byte-identical output.

**Architecture:** The disassembler mirrors the assembler. It walks the MIG schema tree in order. For each MIG node, it checks whether the assembled tree has data for that node. If yes, it emits the corresponding `RawSegment`. Ordering is correct by construction because we follow the MIG sequence. The renderer converts `Vec<RawSegment>` back to an EDIFACT string using `EdifactDelimiters`.

**Tech Stack:** Rust, edifact-types (RawSegment, EdifactDelimiters), mig-assembly assembler types

---

## Task 1: Disassembler — Generic Tree to Segments

**Files:**
- Create: `crates/mig-assembly/src/disassembler.rs`
- Modify: `crates/mig-assembly/src/lib.rs`
- Create: `crates/mig-assembly/tests/disassembler_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-assembly/tests/disassembler_test.rs`:

```rust
use mig_assembly::assembler::{Assembler, AssembledTree, AssembledSegment, AssembledGroup, AssembledGroupInstance};
use mig_assembly::disassembler::Disassembler;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

fn load_mig() -> automapper_generator::schema::mig::MigSchema {
    parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap()
}

#[test]
fn test_disassemble_roundtrip_segments() {
    let mig = load_mig();

    // Build a minimal assembled tree manually
    let tree = AssembledTree {
        segments: vec![
            AssembledSegment {
                tag: "UNH".to_string(),
                elements: vec![
                    vec!["1".to_string()],
                    vec!["UTILMD".to_string(), "D".to_string(), "11A".to_string(), "UN".to_string(), "S2.1".to_string()],
                ],
            },
            AssembledSegment {
                tag: "BGM".to_string(),
                elements: vec![
                    vec!["E01".to_string()],
                    vec!["MSG001".to_string()],
                    vec!["9".to_string()],
                ],
            },
        ],
        groups: vec![],
    };

    let disassembler = Disassembler::new(&mig);
    let segments = disassembler.disassemble(&tree);

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].tag, "UNH");
    assert_eq!(segments[1].tag, "BGM");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_disassemble_roundtrip_segments -- --nocapture`
Expected: FAIL — `Disassembler` doesn't exist

**Step 3: Implement disassembler**

Create `crates/mig-assembly/src/disassembler.rs`:

```rust
//! Tree disassembler — converts AssembledTree back to ordered segments.
//!
//! Walks the MIG schema tree in order. For each MIG node that has
//! corresponding data in the assembled tree, emits segments in MIG order.

use crate::assembler::{AssembledTree, AssembledSegment, AssembledGroup};
use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

/// Output segment from disassembly (owned data, ready for rendering).
#[derive(Debug, Clone)]
pub struct DisassembledSegment {
    pub tag: String,
    pub elements: Vec<Vec<String>>,
}

pub struct Disassembler<'a> {
    mig: &'a MigSchema,
}

impl<'a> Disassembler<'a> {
    pub fn new(mig: &'a MigSchema) -> Self {
        Self { mig }
    }

    /// Disassemble a tree into ordered segments following MIG sequence.
    pub fn disassemble(&self, tree: &AssembledTree) -> Vec<DisassembledSegment> {
        let mut output = Vec::new();

        // Emit top-level segments in MIG order
        for mig_seg in &self.mig.segments {
            if let Some(seg) = tree.segments.iter().find(|s| s.tag == mig_seg.id) {
                output.push(assembled_to_disassembled(seg));
            }
        }

        // Emit groups in MIG order
        for mig_group in &self.mig.segment_groups {
            if let Some(group) = tree.groups.iter().find(|g| g.group_id == mig_group.id) {
                self.emit_group(group, mig_group, &mut output);
            }
        }

        output
    }

    fn emit_group(
        &self,
        group: &AssembledGroup,
        mig_group: &MigSegmentGroup,
        output: &mut Vec<DisassembledSegment>,
    ) {
        for instance in &group.repetitions {
            // Emit segments within this group instance in MIG order
            for mig_seg in &mig_group.segments {
                if let Some(seg) = instance.segments.iter().find(|s| s.tag == mig_seg.id) {
                    output.push(assembled_to_disassembled(seg));
                }
            }

            // Emit nested groups in MIG order
            for nested_mig in &mig_group.nested_groups {
                if let Some(nested) = instance.child_groups.iter().find(|g| g.group_id == nested_mig.id) {
                    self.emit_group(nested, nested_mig, output);
                }
            }
        }
    }
}

fn assembled_to_disassembled(seg: &AssembledSegment) -> DisassembledSegment {
    DisassembledSegment {
        tag: seg.tag.clone(),
        elements: seg.elements.clone(),
    }
}
```

Add `pub mod disassembler;` to `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_disassemble_roundtrip_segments -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/disassembler.rs crates/mig-assembly/src/lib.rs crates/mig-assembly/tests/
git commit -m "feat(mig-assembly): implement tree disassembler for MIG-ordered segment emission"
```

---

## Task 2: EDIFACT Renderer

**Files:**
- Create: `crates/mig-assembly/src/renderer.rs`
- Modify: `crates/mig-assembly/src/lib.rs`

**Step 1: Write the failing test**

Inline test in `renderer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::disassembler::DisassembledSegment;

    #[test]
    fn test_render_segments_to_edifact() {
        let segments = vec![
            DisassembledSegment {
                tag: "UNH".to_string(),
                elements: vec![
                    vec!["1".to_string()],
                    vec!["UTILMD".to_string(), "D".to_string(), "11A".to_string()],
                ],
            },
            DisassembledSegment {
                tag: "BGM".to_string(),
                elements: vec![
                    vec!["E01".to_string()],
                ],
            },
        ];

        let delimiters = edifact_types::EdifactDelimiters::default();
        let rendered = render_edifact(&segments, &delimiters);

        assert!(rendered.contains("UNH+1+UTILMD:D:11A'"));
        assert!(rendered.contains("BGM+E01'"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_render_segments_to_edifact -- --nocapture`
Expected: FAIL

**Step 3: Implement renderer**

Create `crates/mig-assembly/src/renderer.rs`:

```rust
//! EDIFACT string renderer from disassembled segments.

use crate::disassembler::DisassembledSegment;
use edifact_types::EdifactDelimiters;

/// Render a list of disassembled segments into an EDIFACT string.
pub fn render_edifact(segments: &[DisassembledSegment], delimiters: &EdifactDelimiters) -> String {
    let mut out = String::new();

    for seg in segments {
        out.push_str(&seg.tag);

        for (i, element) in seg.elements.iter().enumerate() {
            out.push(delimiters.element_separator());

            for (j, component) in element.iter().enumerate() {
                if j > 0 {
                    out.push(delimiters.component_separator());
                }
                // Escape release characters in the component value
                for ch in component.chars() {
                    if ch == delimiters.release_character()
                        || ch == delimiters.element_separator()
                        || ch == delimiters.component_separator()
                        || ch == delimiters.segment_terminator()
                    {
                        out.push(delimiters.release_character());
                    }
                    out.push(ch);
                }
            }
        }

        out.push(delimiters.segment_terminator());
    }

    out
}
```

Add `pub mod renderer;` to `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_render_segments -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/renderer.rs crates/mig-assembly/src/lib.rs
git commit -m "feat(mig-assembly): add EDIFACT string renderer from disassembled segments"
```

---

## Task 3: Full Roundtrip Pipeline Function

**Files:**
- Create: `crates/mig-assembly/src/roundtrip.rs`
- Modify: `crates/mig-assembly/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_minimal_utilmd() {
        let input = "UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01+MSG001+9'UNT+3+1'";

        let mig = automapper_generator::parsing::mig_parser::parse_mig(
            std::path::Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
            "UTILMD", Some("Strom"), "FV2504",
        ).unwrap();

        let result = roundtrip(input, &mig);
        assert!(result.is_ok(), "Roundtrip failed: {:?}", result.err());
        // The output should contain all the same segments
        let output = result.unwrap();
        assert!(output.contains("UNH+1+UTILMD"));
        assert!(output.contains("BGM+E01+MSG001+9"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_roundtrip_minimal -- --nocapture`
Expected: FAIL

**Step 3: Implement roundtrip pipeline**

Create `crates/mig-assembly/src/roundtrip.rs`:

```rust
//! Full roundtrip pipeline: EDIFACT → assemble → disassemble → render.

use crate::assembler::Assembler;
use crate::disassembler::Disassembler;
use crate::renderer::render_edifact;
use crate::AssemblyError;
use automapper_generator::schema::mig::MigSchema;
use edifact_types::EdifactDelimiters;

/// Perform a full roundtrip: parse EDIFACT, assemble into tree, disassemble, render back.
///
/// Returns the rendered EDIFACT string. If the roundtrip is perfect,
/// the output should be byte-identical to the input (modulo UNA header).
pub fn roundtrip(input: &str, mig: &MigSchema) -> Result<String, AssemblyError> {
    // Pass 1: tokenize
    let segments = edifact_parser::parse_to_segments(input);

    // Detect delimiters from input (UNA or defaults)
    let delimiters = edifact_types::EdifactDelimiters::from_input(input)
        .unwrap_or_default();

    // Pass 2: assemble
    let assembler = Assembler::new(mig);
    let tree = assembler.assemble_generic(&segments)?;

    // Disassemble
    let disassembler = Disassembler::new(mig);
    let dis_segments = disassembler.disassemble(&tree);

    // Render
    let output = render_edifact(&dis_segments, &delimiters);

    Ok(output)
}
```

Add `pub mod roundtrip;` to `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_roundtrip_minimal -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/roundtrip.rs crates/mig-assembly/src/lib.rs
git commit -m "feat(mig-assembly): add full roundtrip pipeline function"
```

---

## Task 4: Byte-Identical Roundtrip Fixture Tests

**Files:**
- Create: `crates/mig-assembly/tests/roundtrip_test.rs`

**Step 1: Write the test**

Create `crates/mig-assembly/tests/roundtrip_test.rs`:

```rust
use mig_assembly::roundtrip::roundtrip;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_byte_identical_roundtrip_all_fixtures() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();

    let fixture_dir = Path::new("../../example_market_communication_bo4e_transactions/UTILMD");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut byte_identical = 0;
    let mut total = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.extension().map(|e| e == "txt").unwrap_or(false) {
            continue;
        }
        total += 1;
        let content = std::fs::read_to_string(&path).unwrap();

        match roundtrip(&content, &mig) {
            Ok(output) => {
                if output == content {
                    byte_identical += 1;
                } else {
                    let name = path.file_name().unwrap().to_string_lossy();
                    // Find first difference position
                    let diff_pos = content.bytes().zip(output.bytes())
                        .position(|(a, b)| a != b)
                        .unwrap_or(content.len().min(output.len()));
                    failures.push(format!("{name}: first diff at byte {diff_pos}"));
                }
            }
            Err(e) => {
                let name = path.file_name().unwrap().to_string_lossy();
                failures.push(format!("{name}: assembly error: {e}"));
            }
        }
    }

    eprintln!("\nRoundtrip: {byte_identical}/{total} byte-identical");
    if !failures.is_empty() {
        eprintln!("Failures (first 20):");
        for f in failures.iter().take(20) {
            eprintln!("  {f}");
        }
    }

    // This is the key metric — should approach 100%
    let rate = byte_identical as f64 / total as f64;
    eprintln!("Success rate: {:.1}%", rate * 100.0);

    // Start with a realistic threshold, tighten as we fix issues
    assert!(rate > 0.8, "Roundtrip rate too low: {byte_identical}/{total}");
}
```

**Step 2: Run test**

Run: `cargo test -p mig-assembly test_byte_identical_roundtrip_all_fixtures -- --nocapture`
Expected: Passes threshold check. Failures identify edge cases to fix.

**Step 3: Iterate on assembler/disassembler to improve rate**

Fix issues found by the fixture tests. Common issues:
- Segments with repeating data elements
- Trailing empty elements being trimmed
- Release character handling in values
- UNA header handling
- Service segments (UNB, UNZ) vs message segments (UNH, UNT)

**Step 4: Commit**

```bash
git add crates/mig-assembly/tests/roundtrip_test.rs crates/mig-assembly/src/
git commit -m "test(mig-assembly): add byte-identical roundtrip fixture tests"
```

---

## Task 5: Snapshot Tests for Tree Structure

**Files:**
- Create: `crates/mig-assembly/tests/snapshot_test.rs`

**Step 1: Write snapshot test**

Create `crates/mig-assembly/tests/snapshot_test.rs`:

```rust
use mig_assembly::assembler::Assembler;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_assembled_tree_snapshot() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();

    // Use a small known fixture
    let fixture_path = Path::new("../../example_market_communication_bo4e_transactions/UTILMD");
    let first_fixture = std::fs::read_dir(fixture_path).unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "txt").unwrap_or(false))
        .expect("Need at least one UTILMD fixture");

    let content = std::fs::read_to_string(first_fixture.path()).unwrap();
    let segments = edifact_parser::parse_to_segments(&content);

    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    // Snapshot the tree structure (groups + segment counts, not full data)
    let summary = summarize_tree(&tree);
    insta::assert_snapshot!(summary);
}

fn summarize_tree(tree: &mig_assembly::assembler::AssembledTree) -> String {
    let mut out = String::new();
    out.push_str(&format!("Top-level segments: {}\n", tree.segments.len()));
    for seg in &tree.segments {
        out.push_str(&format!("  {}: {} elements\n", seg.tag, seg.elements.len()));
    }
    out.push_str(&format!("Groups: {}\n", tree.groups.len()));
    for group in &tree.groups {
        out.push_str(&format!("  {}: {} repetitions\n", group.group_id, group.repetitions.len()));
        for (i, rep) in group.repetitions.iter().enumerate() {
            out.push_str(&format!("    rep[{i}]: {} segments, {} child groups\n",
                rep.segments.len(), rep.child_groups.len()));
        }
    }
    out
}
```

**Step 2: Run to generate snapshot**

Run: `cargo test -p mig-assembly test_assembled_tree_snapshot -- --nocapture`
Then: `cargo insta review` to accept the snapshot.

**Step 3: Commit**

```bash
git add crates/mig-assembly/tests/snapshot_test.rs crates/mig-assembly/tests/snapshots/
git commit -m "test(mig-assembly): add snapshot test for assembled tree structure"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | ~5 (disassembler, renderer, roundtrip pipeline, byte-identical fixtures, snapshots) |
| Byte-identical roundtrip rate | >80% (target: approach 100% over iterations) |
| cargo check --workspace | PASS |
| cargo clippy --workspace | PASS |
