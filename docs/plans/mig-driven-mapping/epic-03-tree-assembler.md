---
feature: mig-driven-mapping
epic: 3
title: "mig-assembly Crate — Tree Assembler"
depends_on: [mig-driven-mapping/E02]
estimated_tasks: 6
crate: mig-assembly
status: in_progress
---

# Epic 3: mig-assembly Crate — Tree Assembler

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Create the `mig-assembly` crate with a two-pass tree assembler. Pass 1 uses the existing `edifact-parser` to tokenize EDIFACT into `Vec<RawSegment>`. Pass 2 is a new recursive descent assembler that consumes the segment list guided by the MIG schema and populates the PID-specific typed tree from `mig-types`.

**Architecture:** The assembler is a pure function: `fn assemble(segments: &[RawSegment], mig: &MigSchema, pid: &str) -> Result<PidTree>`. It maintains a cursor position into the segment slice. At each MIG tree node, it checks whether the current segment matches (tag + qualifier). For repeating groups, it loops until the next segment doesn't match the group's entry segment. PID detection happens before assembly by examining key segments (BGM document type, SG7 process codes).

**Tech Stack:** Rust, edifact-types (RawSegment), edifact-parser (tokenization), mig-types (generated PID types), automapper-generator (MigSchema for runtime schema access)

---

## Task 1: Create mig-assembly Crate Stub

**Files:**
- Create: `crates/mig-assembly/Cargo.toml`
- Create: `crates/mig-assembly/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

**Step 1: Add mig-assembly to workspace**

Add `"crates/mig-assembly"` to workspace members and:

```toml
mig-assembly = { path = "crates/mig-assembly" }
```

**Step 2: Create crate Cargo.toml**

Create `crates/mig-assembly/Cargo.toml`:

```toml
[package]
name = "mig-assembly"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "MIG-guided EDIFACT tree assembly — parse RawSegments into typed MIG trees"

[dependencies]
edifact-types.workspace = true
edifact-parser.workspace = true
mig-types = { path = "../mig-types" }
automapper-generator = { path = "../automapper-generator" }
thiserror.workspace = true
serde.workspace = true

[dev-dependencies]
serde_json.workspace = true
insta.workspace = true
```

**Step 3: Create lib.rs**

Create `crates/mig-assembly/src/lib.rs`:

```rust
//! MIG-guided EDIFACT tree assembly.
//!
//! Two-pass approach:
//! 1. Tokenize EDIFACT into `Vec<RawSegment>` (existing parser)
//! 2. Assemble segments into typed MIG tree guided by MIG schema
//!
//! # Usage
//! ```ignore
//! let segments = parse_edifact(input);
//! let tree: Pid55001 = assemble(&segments, &mig_schema, "55001")?;
//! ```

pub mod assembler;
pub mod error;
pub mod pid_detect;

pub use error::AssemblyError;
```

Create `crates/mig-assembly/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssemblyError {
    #[error("Unexpected segment '{segment_id}' at position {position}, expected one of: {expected:?}")]
    UnexpectedSegment {
        segment_id: String,
        position: usize,
        expected: Vec<String>,
    },

    #[error("Missing mandatory segment '{segment_id}' for PID {pid}")]
    MissingMandatory {
        segment_id: String,
        pid: String,
    },

    #[error("Unknown PID: {0}")]
    UnknownPid(String),

    #[error("PID detection failed: could not determine PID from segments")]
    PidDetectionFailed,

    #[error("Segment cursor out of bounds at position {0}")]
    CursorOutOfBounds(usize),

    #[error("Parse error: {0}")]
    ParseError(String),
}
```

Create stub `crates/mig-assembly/src/assembler.rs`:

```rust
//! Recursive descent assembler — MIG-guided segment consumption.
```

Create stub `crates/mig-assembly/src/pid_detect.rs`:

```rust
//! PID detection from EDIFACT segments.
```

**Step 4: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/ Cargo.toml
git commit -m "feat(mig-assembly): add crate stub for MIG-guided tree assembly"
```

---

## Task 2: PID Detection from Segments

**Files:**
- Modify: `crates/mig-assembly/src/pid_detect.rs`
- Create: `crates/mig-assembly/tests/pid_detect_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-assembly/tests/pid_detect_test.rs`:

```rust
use mig_assembly::pid_detect::detect_pid;
use edifact_types::RawSegment;

#[test]
fn test_detect_pid_from_utilmd_segments() {
    // Construct minimal segments that identify a PID
    // PID is typically determined by:
    // - BGM document type code
    // - STS transaction reason
    // - Process data identifiers in SG7
    // For now, test with a real fixture file

    let input = std::fs::read_to_string(
        "../../example_market_communication_bo4e_transactions/UTILMD/55001.txt"
    );
    // If fixture exists, parse and detect
    if let Ok(content) = input {
        let segments = parse_to_raw_segments(&content);
        let pid = detect_pid(&segments);
        assert!(pid.is_ok(), "Should detect PID from UTILMD fixture");
        // The fixture filename hints at the PID
        assert_eq!(pid.unwrap(), "55001");
    }
}

fn parse_to_raw_segments(input: &str) -> Vec<RawSegment<'_>> {
    // Use edifact-parser to tokenize
    // This is a helper that collects all segments
    todo!("Implement using edifact-parser")
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_detect_pid_from_utilmd_segments -- --nocapture`
Expected: FAIL — `detect_pid` doesn't exist

**Step 3: Implement PID detection**

The PID detection logic examines specific segment values to determine which Pruefidentifikator applies. For UTILMD, the PID is encoded in the process data — typically an STS segment's transaction reason code, or a combination of BGM document name code and process-specific segments.

Implement in `crates/mig-assembly/src/pid_detect.rs`:

```rust
use edifact_types::RawSegment;
use crate::AssemblyError;

/// Detect the PID (Pruefidentifikator) from a list of parsed EDIFACT segments.
///
/// For UTILMD messages, the PID is determined by examining:
/// - BGM document name code (element 1001)
/// - STS transaction reason (element 9015/9013)
/// - Process identifiers in SG7
///
/// Returns the PID as a string (e.g., "55001").
pub fn detect_pid(segments: &[RawSegment<'_>]) -> Result<String, AssemblyError> {
    // Find BGM segment and extract document type
    let bgm = segments.iter().find(|s| s.tag() == "BGM");
    // Find STS segment and extract status code
    let sts = segments.iter().find(|s| s.tag() == "STS");

    // PID detection logic based on combination of BGM + STS values
    // This is message-type-specific; for UTILMD, the mapping is:
    // BGM document code + STS reason -> PID

    match (bgm, sts) {
        (Some(bgm_seg), Some(sts_seg)) => {
            let doc_code = bgm_seg.element(0).and_then(|e| e.component(0));
            let reason = sts_seg.element(1).and_then(|e| e.component(0));
            resolve_utilmd_pid(doc_code, reason)
        }
        _ => Err(AssemblyError::PidDetectionFailed),
    }
}

fn resolve_utilmd_pid(
    doc_code: Option<&str>,
    reason: Option<&str>,
) -> Result<String, AssemblyError> {
    // Map known combinations to PIDs
    // This mapping table is derived from the AHB
    // For now, implement the most common ones; expand as needed
    match (doc_code, reason) {
        // ... populated from AHB analysis
        _ => Err(AssemblyError::PidDetectionFailed),
    }
}
```

Note: The exact PID detection mapping table needs to be derived from the AHB. This may also be generated from the AHB XML in a later refinement. The initial implementation should handle at least the most common PIDs and can be expanded incrementally.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_detect_pid_from_utilmd_segments -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/pid_detect.rs crates/mig-assembly/tests/
git commit -m "feat(mig-assembly): implement PID detection from EDIFACT segments"
```

---

## Task 3: Segment Cursor Abstraction

**Files:**
- Create: `crates/mig-assembly/src/cursor.rs`
- Modify: `crates/mig-assembly/src/lib.rs`

**Step 1: Write the failing test**

Add to `crates/mig-assembly/src/cursor.rs` (inline test module):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_peek_and_advance() {
        let segments = vec!["UNH", "BGM", "NAD", "IDE"];
        let mut cursor = SegmentCursor::new(segments.len());

        assert_eq!(cursor.position(), 0);
        assert!(!cursor.is_exhausted());

        cursor.advance();
        assert_eq!(cursor.position(), 1);

        cursor.advance();
        cursor.advance();
        cursor.advance();
        assert!(cursor.is_exhausted());
    }

    #[test]
    fn test_cursor_remaining() {
        let mut cursor = SegmentCursor::new(5);
        assert_eq!(cursor.remaining(), 5);
        cursor.advance();
        assert_eq!(cursor.remaining(), 4);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_cursor_peek_and_advance -- --nocapture`
Expected: FAIL — `cursor` module doesn't exist

**Step 3: Implement cursor**

Create `crates/mig-assembly/src/cursor.rs`:

```rust
//! Segment cursor for tracking position during MIG-guided assembly.

/// A cursor that tracks position within a segment slice during assembly.
///
/// The cursor is the core state machine of the assembler. It advances
/// through segments as the MIG tree is matched against the input.
pub struct SegmentCursor {
    position: usize,
    total: usize,
}

impl SegmentCursor {
    pub fn new(total: usize) -> Self {
        Self { position: 0, total }
    }

    /// Current position in the segment list.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Number of segments remaining.
    pub fn remaining(&self) -> usize {
        self.total.saturating_sub(self.position)
    }

    /// Whether all segments have been consumed.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.total
    }

    /// Advance the cursor by one segment.
    pub fn advance(&mut self) {
        self.position += 1;
    }

    /// Save the current position for backtracking.
    pub fn save(&self) -> usize {
        self.position
    }

    /// Restore to a previously saved position.
    pub fn restore(&mut self, saved: usize) {
        self.position = saved;
    }
}
```

Add `pub mod cursor;` to `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_cursor -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/cursor.rs crates/mig-assembly/src/lib.rs
git commit -m "feat(mig-assembly): add segment cursor for position tracking during assembly"
```

---

## Task 4: Segment Matching Logic

**Files:**
- Create: `crates/mig-assembly/src/matcher.rs`
- Modify: `crates/mig-assembly/src/lib.rs`

**Step 1: Write the failing test**

Inline tests in `crates/mig-assembly/src/matcher.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_segment_by_tag() {
        // A NAD segment should match MIG node "NAD"
        assert!(matches_segment_tag("NAD", "NAD"));
        assert!(!matches_segment_tag("NAD", "LOC"));
    }

    #[test]
    fn test_match_segment_with_qualifier() {
        // NAD+MS should match when qualifier is "MS"
        // The raw segment's first element (after tag) is the qualifier
        assert!(matches_qualifier("MS", "MS"));
        assert!(!matches_qualifier("MS", "MR"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_match_segment -- --nocapture`
Expected: FAIL — module doesn't exist

**Step 3: Implement matcher**

Create `crates/mig-assembly/src/matcher.rs`:

```rust
//! Segment matching logic for MIG-guided assembly.
//!
//! Determines whether a RawSegment matches a MIG tree node based on
//! segment tag and optional qualifier values.

use edifact_types::RawSegment;

/// Check if a segment's tag matches the expected MIG segment ID.
pub fn matches_segment_tag(segment_tag: &str, expected_tag: &str) -> bool {
    segment_tag.eq_ignore_ascii_case(expected_tag)
}

/// Check if a qualifier value matches the expected qualifier.
pub fn matches_qualifier(actual: &str, expected: &str) -> bool {
    actual.trim() == expected.trim()
}

/// Check if a RawSegment matches a MIG node, optionally checking a qualifier.
///
/// - `segment`: the raw EDIFACT segment
/// - `expected_tag`: the MIG segment ID (e.g., "NAD")
/// - `qualifier_element`: which element index contains the qualifier (typically 0)
/// - `qualifier_component`: which component of that element (typically 0)
/// - `expected_qualifier`: if Some, the qualifier must match
pub fn matches_mig_node(
    segment: &RawSegment<'_>,
    expected_tag: &str,
    qualifier_element: usize,
    qualifier_component: usize,
    expected_qualifier: Option<&str>,
) -> bool {
    if !matches_segment_tag(segment.tag(), expected_tag) {
        return false;
    }
    match expected_qualifier {
        Some(q) => segment
            .element(qualifier_element)
            .and_then(|e| e.component(qualifier_component))
            .map(|actual| matches_qualifier(actual, q))
            .unwrap_or(false),
        None => true,
    }
}
```

Add `pub mod matcher;` to `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_match -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/matcher.rs crates/mig-assembly/src/lib.rs
git commit -m "feat(mig-assembly): add segment matching logic for MIG-guided assembly"
```

---

## Task 5: Recursive Descent Assembler Core

**Files:**
- Modify: `crates/mig-assembly/src/assembler.rs`
- Create: `crates/mig-assembly/tests/assembler_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-assembly/tests/assembler_test.rs`:

```rust
use mig_assembly::assembler::Assembler;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use std::path::Path;

fn load_mig() -> MigSchema {
    parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap()
}

#[test]
fn test_assembler_basic_segments() {
    let mig = load_mig();

    // Minimal UTILMD: UNH + BGM + UNT
    // This tests that the assembler can consume service segments
    let input = "UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01+MSG001+9'UNT+3+1'";

    let segments = edifact_parser::parse_to_segments(input);
    let assembler = Assembler::new(&mig);
    let result = assembler.assemble_generic(&segments);

    assert!(result.is_ok(), "Assembly should succeed for minimal UTILMD: {:?}", result.err());
    let tree = result.unwrap();
    // The generic tree should have captured UNH, BGM, UNT
    assert!(tree.segments.iter().any(|s| s.tag == "UNH"));
    assert!(tree.segments.iter().any(|s| s.tag == "BGM"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_assembler_basic_segments -- --nocapture`
Expected: FAIL — `Assembler` doesn't exist

**Step 3: Implement assembler core**

The assembler uses the MIG schema as a grammar to guide consumption of segments. It implements a recursive descent approach:

Implement in `crates/mig-assembly/src/assembler.rs`:

```rust
//! Recursive descent assembler — MIG-guided segment consumption.
//!
//! The assembler walks the MIG tree structure and consumes matching
//! segments from the input. It produces a generic tree representation
//! that can be converted to typed PID structs.

use crate::cursor::SegmentCursor;
use crate::matcher;
use crate::AssemblyError;
use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};
use edifact_types::RawSegment;

/// A generic assembled tree node (before PID-specific typing).
#[derive(Debug, Clone)]
pub struct AssembledTree {
    pub segments: Vec<AssembledSegment>,
    pub groups: Vec<AssembledGroup>,
}

#[derive(Debug, Clone)]
pub struct AssembledSegment {
    pub tag: String,
    pub elements: Vec<Vec<String>>, // elements[i][j] = component j of element i
}

#[derive(Debug, Clone)]
pub struct AssembledGroup {
    pub group_id: String,
    pub repetitions: Vec<AssembledGroupInstance>,
}

#[derive(Debug, Clone)]
pub struct AssembledGroupInstance {
    pub segments: Vec<AssembledSegment>,
    pub child_groups: Vec<AssembledGroup>,
}

pub struct Assembler<'a> {
    mig: &'a MigSchema,
}

impl<'a> Assembler<'a> {
    pub fn new(mig: &'a MigSchema) -> Self {
        Self { mig }
    }

    /// Assemble segments into a generic tree following MIG structure.
    pub fn assemble_generic(
        &self,
        segments: &[RawSegment<'_>],
    ) -> Result<AssembledTree, AssemblyError> {
        let mut cursor = SegmentCursor::new(segments.len());
        let mut tree = AssembledTree {
            segments: Vec::new(),
            groups: Vec::new(),
        };

        // Process top-level segments
        for mig_seg in &self.mig.segments {
            if cursor.is_exhausted() {
                break;
            }
            if let Some(assembled) = self.try_consume_segment(segments, &mut cursor, mig_seg)? {
                tree.segments.push(assembled);
            }
        }

        // Process segment groups
        for mig_group in &self.mig.segment_groups {
            if cursor.is_exhausted() {
                break;
            }
            if let Some(assembled) = self.try_consume_group(segments, &mut cursor, mig_group)? {
                tree.groups.push(assembled);
            }
        }

        Ok(tree)
    }

    fn try_consume_segment(
        &self,
        segments: &[RawSegment<'_>],
        cursor: &mut SegmentCursor,
        mig_seg: &MigSegment,
    ) -> Result<Option<AssembledSegment>, AssemblyError> {
        if cursor.is_exhausted() {
            return Ok(None);
        }
        let seg = &segments[cursor.position()];
        if matcher::matches_segment_tag(seg.tag(), &mig_seg.id) {
            let assembled = raw_to_assembled(seg);
            cursor.advance();
            Ok(Some(assembled))
        } else {
            Ok(None) // Segment not present (optional)
        }
    }

    fn try_consume_group(
        &self,
        segments: &[RawSegment<'_>],
        cursor: &mut SegmentCursor,
        mig_group: &MigSegmentGroup,
    ) -> Result<Option<AssembledGroup>, AssemblyError> {
        let mut repetitions = Vec::new();
        let entry_segment = mig_group.segments.first()
            .ok_or_else(|| AssemblyError::ParseError(
                format!("Group {} has no segments", mig_group.id)
            ))?;

        // Loop for repeating groups
        while !cursor.is_exhausted() {
            let seg = &segments[cursor.position()];
            if !matcher::matches_segment_tag(seg.tag(), &entry_segment.id) {
                break; // Current segment doesn't match group entry — stop repeating
            }

            let mut instance = AssembledGroupInstance {
                segments: Vec::new(),
                child_groups: Vec::new(),
            };

            // Consume segments within this group instance
            for group_seg in &mig_group.segments {
                if cursor.is_exhausted() {
                    break;
                }
                if let Some(assembled) = self.try_consume_segment(segments, cursor, group_seg)? {
                    instance.segments.push(assembled);
                }
            }

            // Consume nested groups
            for nested in &mig_group.nested_groups {
                if cursor.is_exhausted() {
                    break;
                }
                if let Some(assembled) = self.try_consume_group(segments, cursor, nested)? {
                    instance.child_groups.push(assembled);
                }
            }

            repetitions.push(instance);
        }

        if repetitions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AssembledGroup {
                group_id: mig_group.id.clone(),
                repetitions,
            }))
        }
    }
}

fn raw_to_assembled(seg: &RawSegment<'_>) -> AssembledSegment {
    let mut elements = Vec::new();
    let mut i = 0;
    while let Some(elem) = seg.element(i) {
        let mut components = Vec::new();
        let mut j = 0;
        while let Some(comp) = elem.component(j) {
            components.push(comp.to_string());
            j += 1;
        }
        elements.push(components);
        i += 1;
    }
    AssembledSegment {
        tag: seg.tag().to_string(),
        elements,
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_assembler_basic_segments -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/assembler.rs crates/mig-assembly/tests/
git commit -m "feat(mig-assembly): implement recursive descent assembler core"
```

---

## Task 6: Integration Test with Real EDIFACT Fixtures

**Files:**
- Create: `crates/mig-assembly/tests/fixture_assembly_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-assembly/tests/fixture_assembly_test.rs`:

```rust
use mig_assembly::assembler::Assembler;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_assemble_real_utilmd_fixture() {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();

    // Find UTILMD fixture files
    let fixture_dir = Path::new("../../example_market_communication_bo4e_transactions/UTILMD");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut success = 0;
    let mut total = 0;

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "txt").unwrap_or(false) {
            total += 1;
            let content = std::fs::read_to_string(&path).unwrap();
            let segments = edifact_parser::parse_to_segments(&content);

            let assembler = Assembler::new(&mig);
            match assembler.assemble_generic(&segments) {
                Ok(tree) => {
                    // Basic sanity: should have at least UNH and UNT
                    assert!(tree.segments.iter().any(|s| s.tag == "UNH"),
                        "Missing UNH in {:?}", path);
                    success += 1;
                }
                Err(e) => {
                    eprintln!("Assembly failed for {:?}: {}", path.file_name().unwrap(), e);
                }
            }
        }
    }

    eprintln!("Assembly: {success}/{total} fixtures succeeded");
    // At least 90% should assemble successfully
    assert!(success as f64 / total as f64 > 0.9,
        "Too many assembly failures: {success}/{total}");
}
```

**Step 2: Run test**

Run: `cargo test -p mig-assembly test_assemble_real_utilmd_fixture -- --nocapture`
Expected: PASS with >90% fixture assembly success rate. Failures indicate edge cases to fix in the assembler.

**Step 3: Fix any edge cases found**

Iterate on the assembler to handle edge cases from real fixtures (unexpected segment ordering, optional groups in unexpected positions, etc.).

**Step 4: Commit**

```bash
git add crates/mig-assembly/tests/ crates/mig-assembly/src/
git commit -m "test(mig-assembly): add fixture integration tests for real UTILMD files"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | ~6 (PID detection, cursor, matcher, assembler core, fixture integration) |
| Fixture success rate | >90% of UTILMD fixtures assemble |
| cargo check --workspace | PASS |
| cargo clippy --workspace | PASS |
