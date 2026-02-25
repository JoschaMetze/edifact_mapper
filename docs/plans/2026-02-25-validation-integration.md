# Validation Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire the isolated `automapper-validation` crate into the MIG-driven pipeline so it can validate real EDIFACT messages end-to-end.

**Architecture:** The validator becomes a pure engine accepting pre-parsed `OwnedSegment`s. The API layer orchestrates parsing, PID detection, MIG loading, and assembly — then hands segments + AhbWorkflow + ConditionEvaluator to the validator. Structure diagnostics come from the assembler; condition validation from generated evaluators.

**Tech Stack:** Rust, Axum, mig-assembly, automapper-generator (Claude CLI for condition codegen), insta (snapshots), test-case (parameterized tests)

**Design doc:** `docs/plans/2026-02-25-validation-integration-design.md`

---

### Task 1: Switch EvaluationContext from RawSegment to OwnedSegment

The current `EvaluationContext` holds `&'a [edifact_types::RawSegment<'a>]` which has double-lifetime complexity. The MIG pipeline produces `OwnedSegment`s (from `mig_types::segment`). Switching aligns the validator with the pipeline and simplifies lifetimes.

**Files:**
- Modify: `crates/automapper-validation/Cargo.toml`
- Modify: `crates/automapper-validation/src/eval/context.rs`
- Modify: `crates/automapper-validation/src/eval/evaluator.rs` (ConditionEvaluator trait)
- Modify: `crates/automapper-validation/src/eval/expr_eval.rs` (if it references RawSegment)
- Modify: `crates/automapper-validation/src/eval/mod.rs` (re-exports)

**Step 1: Add mig-types dependency**

In `crates/automapper-validation/Cargo.toml`, add to `[dependencies]`:

```toml
mig-types.workspace = true
```

**Step 2: Update EvaluationContext to use OwnedSegment**

In `crates/automapper-validation/src/eval/context.rs`, replace the entire file. Key changes:
- Import `mig_types::segment::OwnedSegment` instead of `edifact_types::RawSegment`
- Change `segments: &'a [edifact_types::RawSegment<'a>]` → `segments: &'a [OwnedSegment]`
- Update `find_segment()` to return `Option<&'a OwnedSegment>`
- Update `find_segments()` to return `Vec<&'a OwnedSegment>`
- Update `find_segments_with_qualifier()` — OwnedSegment uses `String` not `&str`, so comparisons change from `*v == qualifier` to `v == qualifier`
- Update tests: replace `make_segment()` with `OwnedSegment { id, elements, segment_number: 0 }` — elements become `Vec<Vec<String>>`

```rust
// New context.rs — key struct change:
use mig_types::segment::OwnedSegment;

pub struct EvaluationContext<'a> {
    pub pruefidentifikator: &'a str,
    pub external: &'a dyn ExternalConditionProvider,
    pub segments: &'a [OwnedSegment],
}
```

Helper methods stay the same shape but return `&OwnedSegment` instead of `&RawSegment`. The `find_segments_with_qualifier` filter changes:
```rust
// Old: s.elements.get(element_index).and_then(|e| e.first()).is_some_and(|v| *v == qualifier)
// New: s.elements.get(element_index).and_then(|e| e.first()).is_some_and(|v| v == qualifier)
```

Tests: Replace `make_segment(id, elements)` helper with direct `OwnedSegment` construction:
```rust
fn make_segment(id: &str, elements: Vec<Vec<&str>>) -> OwnedSegment {
    OwnedSegment {
        id: id.to_string(),
        elements: elements.into_iter().map(|e| e.into_iter().map(|c| c.to_string()).collect()).collect(),
        segment_number: 0,
    }
}
```

**Step 3: Update ConditionEvaluator trait signature**

In `crates/automapper-validation/src/eval/evaluator.rs`, the `ConditionEvaluator::evaluate` method takes `&EvaluationContext` which now uses `OwnedSegment`. The trait signature itself doesn't change — it's `fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult`. But any code that accesses `ctx.segments` now gets `&[OwnedSegment]`.

No changes needed to the trait itself or `NoOpExternalProvider`. Just verify it compiles.

**Step 4: Run tests to verify**

```bash
cargo test -p automapper-validation
```

Expected: All 143 tests pass. The context tests use the updated `make_segment` helper. The validator tests use `MockEvaluator` which doesn't inspect segments.

**Step 5: Run clippy and format**

```bash
cargo clippy -p automapper-validation -- -D warnings && cargo fmt --all
```

**Step 6: Commit**

```bash
git add crates/automapper-validation/
git commit -m "refactor(validation): switch EvaluationContext from RawSegment to OwnedSegment

Aligns the validator with the MIG pipeline which produces OwnedSegment.
Removes double-lifetime complexity. mig-types is zero-dep (only serde)."
```

---

### Task 2: Simplify EdifactValidator API to Accept Pre-Parsed Segments

Remove `parse_segments()` (the empty TODO) and `validate_structure()` (the no-op placeholder) from `EdifactValidator`. Change `validate()` to accept pre-parsed `OwnedSegment`s directly. The API layer will handle parsing and PID detection.

**Files:**
- Modify: `crates/automapper-validation/src/validator/validate.rs`
- Modify: `crates/automapper-validation/src/error.rs` (remove Parse variant if unused)
- Modify: `crates/automapper-validation/tests/validator_integration.rs`

**Step 1: Write the failing test for the new API**

In `crates/automapper-validation/tests/validator_integration.rs`, add a test that calls the new simplified `validate()` with `OwnedSegment`s:

```rust
use mig_types::segment::OwnedSegment;

#[test]
fn test_validate_with_owned_segments() {
    let evaluator = ConfigurableEvaluator::new("UTILMD", "FV2504")
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::True);
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let segments = vec![
        OwnedSegment {
            id: "UNH".to_string(),
            elements: vec![vec!["001".to_string(), "UTILMD:D:11A:UN:S2.1".to_string()]],
            segment_number: 1,
        },
        OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MS".to_string(), "9900123".to_string()]],
            segment_number: 2,
        },
    ];

    let workflow = make_workflow(vec![simple_field(
        "SG2/NAD/C082/3039",
        "MP-ID",
        "Muss [182] ∧ [152]",
    )]);

    let report = validator.validate(&segments, &workflow, &external, ValidationLevel::Full);
    assert!(report.is_valid()); // NAD present, condition True → no errors
}
```

Run: `cargo test -p automapper-validation test_validate_with_owned_segments`
Expected: FAIL — `validate()` still takes `&[u8]`.

**Step 2: Rewrite `validate()` in `validate.rs`**

Replace the current `validate()` method (lines 113-156) and remove `parse_segments()` (lines 158-167), `detect_message_type()` (lines 170-180), `detect_pruefidentifikator()` (lines 183-198), and `validate_structure()` (lines 200-210). The new signature:

```rust
/// Validate pre-parsed EDIFACT segments against AHB rules.
///
/// The caller is responsible for parsing EDIFACT, detecting PID,
/// and loading the AHB workflow. This method is a pure validation
/// engine that checks conditions and codes.
pub fn validate(
    &self,
    segments: &[OwnedSegment],
    workflow: &AhbWorkflow,
    external: &dyn ExternalConditionProvider,
    level: ValidationLevel,
) -> ValidationReport {
    let mut report = ValidationReport::new(
        self.evaluator.message_type(),
        level,
    )
    .with_format_version(self.evaluator.format_version())
    .with_pruefidentifikator(&workflow.pruefidentifikator);

    let ctx = EvaluationContext::new(
        &workflow.pruefidentifikator,
        external,
        segments,
    );

    // Condition validation (if level >= Conditions)
    if matches!(level, ValidationLevel::Conditions | ValidationLevel::Full) {
        self.validate_conditions(workflow, &ctx, &mut report);
    }

    report
}
```

Add the import at the top of the file:
```rust
use mig_types::segment::OwnedSegment;
```

Also update `validate_conditions` and `validate_field_codes` — they use `ctx.has_segment()` and `ctx.find_segments()` which now return `OwnedSegment` references. The logic is the same; the types are compatible since we check `.id` (now `String`) and `.elements` (now `Vec<Vec<String>>`).

In `validate_field_codes`, the code check changes from:
```rust
// Old: if !code_value.is_empty() && !allowed_codes.contains(code_value)
// New: if !code_value.is_empty() && !allowed_codes.contains(&code_value.as_str())
```

**Step 3: Update existing tests in validate.rs**

All existing tests call `validator.validate(b"", ...)` with `&[u8]`. Change them to:
```rust
// Old: validator.validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
// New: validator.validate(&[], &workflow, &external, ValidationLevel::Conditions)
```

Key changes:
- `&[]` for empty segments instead of `b""`
- `workflow` is no longer `Option` — it's always required (the API layer is responsible for loading it)
- Return type is `ValidationReport` (not `Result`) — parsing errors are the API layer's problem
- Remove tests for `None` workflow (that concept is gone)
- Update `test_report_includes_metadata` to pass a simple workflow

**Step 4: Update integration tests**

In `crates/automapper-validation/tests/validator_integration.rs`, update the `ConfigurableEvaluator` tests to match the new API. The `FixedExternalProvider` and helpers should work unchanged. Add `use mig_types::segment::OwnedSegment;` to the imports.

**Step 5: Run all tests**

```bash
cargo test -p automapper-validation
```

Expected: All tests pass with the new API.

**Step 6: Clean up error.rs**

If `ValidationError::Parse` is no longer used (since `validate()` doesn't parse), consider keeping it for potential future use or remove it. Check if anything else references it:

```bash
cargo check -p automapper-validation 2>&1 | head -30
```

If `Parse` variant and `edifact-parser` dep are unused, remove the `Parse` variant from `ValidationError` and `edifact-parser` from `Cargo.toml` dependencies. Keep `edifact-parser` in dev-dependencies if integration tests need it.

**Step 7: Clippy + format + commit**

```bash
cargo clippy -p automapper-validation -- -D warnings && cargo fmt --all
git add crates/automapper-validation/
git commit -m "refactor(validation): simplify EdifactValidator to accept pre-parsed segments

Remove parse_segments() TODO and validate_structure() placeholder.
Validator is now a pure engine: receives OwnedSegments, AhbWorkflow,
and ExternalConditionProvider, returns ValidationReport directly."
```

---

### Task 3: Add MapExternalProvider and CompositeExternalProvider

Add two new `ExternalConditionProvider` implementations for flexible external condition handling.

**Files:**
- Create: `crates/automapper-validation/src/eval/providers.rs`
- Modify: `crates/automapper-validation/src/eval/mod.rs` (add module + re-exports)
- Modify: `crates/automapper-validation/src/lib.rs` (re-export new types)

**Step 1: Write failing tests for MapExternalProvider**

Create `crates/automapper-validation/src/eval/providers.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::ConditionResult;
    use std::collections::HashMap;

    #[test]
    fn test_map_provider_known_condition() {
        let mut map = HashMap::new();
        map.insert("DateKnown".to_string(), true);
        map.insert("MessageSplitting".to_string(), false);
        let provider = MapExternalProvider::new(map);

        assert_eq!(provider.evaluate("DateKnown"), ConditionResult::True);
        assert_eq!(provider.evaluate("MessageSplitting"), ConditionResult::False);
    }

    #[test]
    fn test_map_provider_unknown_condition() {
        let provider = MapExternalProvider::new(HashMap::new());
        assert_eq!(provider.evaluate("anything"), ConditionResult::Unknown);
    }

    #[test]
    fn test_composite_provider_first_known_wins() {
        let mut map1 = HashMap::new();
        map1.insert("A".to_string(), true);
        let p1 = MapExternalProvider::new(map1);

        let mut map2 = HashMap::new();
        map2.insert("B".to_string(), false);
        let p2 = MapExternalProvider::new(map2);

        let composite = CompositeExternalProvider::new(vec![Box::new(p1), Box::new(p2)]);

        assert_eq!(composite.evaluate("A"), ConditionResult::True);   // from p1
        assert_eq!(composite.evaluate("B"), ConditionResult::False);  // from p2
        assert_eq!(composite.evaluate("C"), ConditionResult::Unknown); // neither
    }

    #[test]
    fn test_composite_provider_empty() {
        let composite = CompositeExternalProvider::new(vec![]);
        assert_eq!(composite.evaluate("anything"), ConditionResult::Unknown);
    }
}
```

Run: `cargo test -p automapper-validation test_map_provider`
Expected: FAIL — types don't exist yet.

**Step 2: Implement MapExternalProvider and CompositeExternalProvider**

In the same file `providers.rs`, above the test module:

```rust
//! Pluggable external condition provider implementations.

use std::collections::HashMap;

use super::evaluator::{ConditionResult, ExternalConditionProvider};

/// External condition provider backed by a string→bool map.
///
/// Useful for API callers that supply condition overrides as JSON.
/// Unknown conditions (not in the map) return `ConditionResult::Unknown`.
pub struct MapExternalProvider {
    conditions: HashMap<String, bool>,
}

impl MapExternalProvider {
    pub fn new(conditions: HashMap<String, bool>) -> Self {
        Self { conditions }
    }
}

impl ExternalConditionProvider for MapExternalProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        match self.conditions.get(condition_name) {
            Some(true) => ConditionResult::True,
            Some(false) => ConditionResult::False,
            None => ConditionResult::Unknown,
        }
    }
}

/// Composite provider that chains multiple providers.
///
/// Evaluates each provider in order. Returns the first non-Unknown result.
/// If all providers return Unknown, returns Unknown.
pub struct CompositeExternalProvider {
    providers: Vec<Box<dyn ExternalConditionProvider>>,
}

impl CompositeExternalProvider {
    pub fn new(providers: Vec<Box<dyn ExternalConditionProvider>>) -> Self {
        Self { providers }
    }
}

impl ExternalConditionProvider for CompositeExternalProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        for provider in &self.providers {
            let result = provider.evaluate(condition_name);
            if !result.is_unknown() {
                return result;
            }
        }
        ConditionResult::Unknown
    }
}
```

**Step 3: Wire up module exports**

In `crates/automapper-validation/src/eval/mod.rs`, add:
```rust
pub mod providers;
```
And in the re-exports add:
```rust
pub use providers::{CompositeExternalProvider, MapExternalProvider};
```

In `crates/automapper-validation/src/lib.rs`, add to the eval re-exports:
```rust
pub use eval::{..., CompositeExternalProvider, MapExternalProvider};
```

**Step 4: Run tests**

```bash
cargo test -p automapper-validation
```

Expected: All tests pass including the new provider tests.

**Step 5: Commit**

```bash
cargo clippy -p automapper-validation -- -D warnings && cargo fmt --all
git add crates/automapper-validation/
git commit -m "feat(validation): add MapExternalProvider and CompositeExternalProvider

MapExternalProvider wraps HashMap<String, bool> for API callers.
CompositeExternalProvider chains multiple providers, first non-Unknown wins."
```

---

### Task 4: Define StructureDiagnostic Types in mig-assembly

Add diagnostic types that the assembler can populate during assembly.

**Files:**
- Create: `crates/mig-assembly/src/diagnostic.rs`
- Modify: `crates/mig-assembly/src/lib.rs` (add module + re-exports)

**Step 1: Write tests for diagnostic types**

Create `crates/mig-assembly/src/diagnostic.rs`:

```rust
//! Structure diagnostics emitted during MIG-guided assembly.

use serde::{Deserialize, Serialize};

/// A structure-level issue found during MIG-guided assembly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDiagnostic {
    /// The kind of structural issue.
    pub kind: StructureDiagnosticKind,
    /// Segment identifier involved (e.g., "LOC", "DTM").
    pub segment_id: String,
    /// 0-based segment index in the input where the issue was detected.
    pub position: usize,
    /// Human-readable description of the issue.
    pub message: String,
}

/// Classification of structure diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureDiagnosticKind {
    /// Input segment not expected by the PID-filtered MIG at this position.
    UnexpectedSegment,
    /// MIG declares a mandatory segment that was not found in the input.
    MissingRequiredSegment,
    /// More repetitions of a group/segment than the MIG allows.
    MaxRepetitionsExceeded,
    /// Segment present but qualifier value not recognized by MIG.
    UnrecognizedQualifier,
}

impl std::fmt::Display for StructureDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {} at position {}: {}", self.kind, self.segment_id, self.position, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display() {
        let d = StructureDiagnostic {
            kind: StructureDiagnosticKind::UnexpectedSegment,
            segment_id: "FTX".to_string(),
            position: 5,
            message: "Segment FTX not expected by MIG at this position".to_string(),
        };
        let s = format!("{d}");
        assert!(s.contains("FTX"));
        assert!(s.contains("position 5"));
    }

    #[test]
    fn test_diagnostic_serialization() {
        let d = StructureDiagnostic {
            kind: StructureDiagnosticKind::MissingRequiredSegment,
            segment_id: "UNH".to_string(),
            position: 0,
            message: "Required segment UNH not found".to_string(),
        };
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains("MissingRequiredSegment"));
        let roundtrip: StructureDiagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.kind, StructureDiagnosticKind::MissingRequiredSegment);
    }
}
```

**Step 2: Wire module**

In `crates/mig-assembly/src/lib.rs`, add:
```rust
pub mod diagnostic;
```
And re-export:
```rust
pub use diagnostic::{StructureDiagnostic, StructureDiagnosticKind};
```

**Step 3: Run tests**

```bash
cargo test -p mig-assembly test_diagnostic
```

Expected: PASS.

**Step 4: Commit**

```bash
cargo clippy -p mig-assembly -- -D warnings && cargo fmt --all
git add crates/mig-assembly/src/diagnostic.rs crates/mig-assembly/src/lib.rs
git commit -m "feat(mig-assembly): add StructureDiagnostic types for assembly diagnostics"
```

---

### Task 5: Implement assemble_with_diagnostics

Add a method to `Assembler` that collects diagnostics during assembly. The existing `assemble_generic()` stays unchanged.

**Files:**
- Modify: `crates/mig-assembly/src/assembler.rs`

**Step 1: Write failing test for unconsumed segments**

In the `#[cfg(test)]` module at the bottom of `assembler.rs`:

```rust
use crate::diagnostic::{StructureDiagnostic, StructureDiagnosticKind};

#[test]
fn test_assemble_with_diagnostics_clean_input() {
    let mig = make_mig_schema(vec!["UNH", "BGM", "UNT"], vec![]);
    let segments = vec![
        make_owned_seg("UNH", vec![vec!["001"]]),
        make_owned_seg("BGM", vec![vec!["E01"]]),
        make_owned_seg("UNT", vec![vec!["2", "001"]]),
    ];

    let assembler = Assembler::new(&mig);
    let (tree, diagnostics) = assembler.assemble_with_diagnostics(&segments);

    assert_eq!(tree.segments.len(), 3);
    assert!(diagnostics.is_empty(), "Clean input should produce no diagnostics");
}

#[test]
fn test_assemble_with_diagnostics_unconsumed_segments() {
    // MIG only expects UNH, BGM — but input has UNH, BGM, FTX (extra)
    let mig = make_mig_schema(vec!["UNH", "BGM"], vec![]);
    let segments = vec![
        make_owned_seg("UNH", vec![vec!["001"]]),
        make_owned_seg("BGM", vec![vec!["E01"]]),
        make_owned_seg("FTX", vec![vec!["AAA", "extra text"]]),
    ];

    let assembler = Assembler::new(&mig);
    let (tree, diagnostics) = assembler.assemble_with_diagnostics(&segments);

    assert_eq!(tree.segments.len(), 2); // UNH + BGM consumed
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].kind, StructureDiagnosticKind::UnexpectedSegment);
    assert_eq!(diagnostics[0].segment_id, "FTX");
    assert_eq!(diagnostics[0].position, 2);
}
```

Run: `cargo test -p mig-assembly test_assemble_with_diagnostics`
Expected: FAIL — method doesn't exist.

**Step 2: Implement assemble_with_diagnostics**

Add to `impl<'a> Assembler<'a>` in `assembler.rs`, after `assemble_generic()`:

```rust
/// Assemble segments with diagnostic collection.
///
/// Returns the assembled tree (same as `assemble_generic`) plus a list of
/// diagnostics for segments that weren't consumed or other structural issues.
pub fn assemble_with_diagnostics(
    &self,
    segments: &[OwnedSegment],
) -> (AssembledTree, Vec<StructureDiagnostic>) {
    let mut diagnostics = Vec::new();

    // Run normal assembly (which never fails for well-formed MIGs)
    let tree = match self.assemble_generic(segments) {
        Ok(tree) => tree,
        Err(e) => {
            diagnostics.push(StructureDiagnostic {
                kind: StructureDiagnosticKind::UnexpectedSegment,
                segment_id: String::new(),
                position: 0,
                message: format!("Assembly failed: {e}"),
            });
            return (
                AssembledTree { segments: Vec::new(), groups: Vec::new(), post_group_start: 0 },
                diagnostics,
            );
        }
    };

    // Count consumed segments in the tree
    let consumed = count_segments_in_tree(&tree);

    // Any input segments beyond what was consumed are unexpected
    for (i, seg) in segments.iter().enumerate().skip(consumed) {
        diagnostics.push(StructureDiagnostic {
            kind: StructureDiagnosticKind::UnexpectedSegment,
            segment_id: seg.id.clone(),
            position: i,
            message: format!(
                "Segment '{}' at position {} was not consumed by MIG-guided assembly",
                seg.id, i
            ),
        });
    }

    (tree, diagnostics)
}
```

Add the helper function outside the impl block:

```rust
/// Count total segments captured in an assembled tree (recursively).
fn count_segments_in_tree(tree: &AssembledTree) -> usize {
    let mut count = tree.segments.len();
    for group in &tree.groups {
        for rep in &group.repetitions {
            count += rep.segments.len();
            for child in &rep.child_groups {
                count += count_segments_in_group(child);
            }
        }
    }
    count
}

fn count_segments_in_group(group: &AssembledGroup) -> usize {
    let mut count = 0;
    for rep in &group.repetitions {
        count += rep.segments.len();
        for child in &rep.child_groups {
            count += count_segments_in_group(child);
        }
    }
    count
}
```

Add the import at the top of `assembler.rs`:
```rust
use crate::diagnostic::{StructureDiagnostic, StructureDiagnosticKind};
```

**Step 3: Run tests**

```bash
cargo test -p mig-assembly test_assemble_with_diagnostics
```

Expected: Both tests pass.

**Step 4: Run full test suite**

```bash
cargo test -p mig-assembly
```

Expected: All existing + new tests pass. `assemble_generic()` is unchanged.

**Step 5: Commit**

```bash
cargo clippy -p mig-assembly -- -D warnings && cargo fmt --all
git add crates/mig-assembly/
git commit -m "feat(mig-assembly): add assemble_with_diagnostics for structure validation

Collects StructureDiagnostics for segments not consumed by MIG assembly.
Existing assemble_generic() is unchanged (non-breaking)."
```

---

### Task 6: Add AhbWorkflow Bridge (AhbSchema → AhbWorkflow Conversion)

Create a function to convert the generator's `AhbSchema` types into the validator's `AhbWorkflow` input. This bridges the two crates.

**Files:**
- Create: `crates/automapper-api/src/validation_bridge.rs`
- Modify: `crates/automapper-api/src/lib.rs` (or wherever modules are declared)
- Modify: `crates/automapper-api/Cargo.toml` (add automapper-validation dependency)

**Step 1: Add automapper-validation dependency to API crate**

In `crates/automapper-api/Cargo.toml` under `[dependencies]`:
```toml
automapper-validation.workspace = true
```

Check the workspace `Cargo.toml` has `automapper-validation` in `[workspace.dependencies]`. If not, add it:
```toml
automapper-validation = { path = "crates/automapper-validation" }
```

**Step 2: Write test for the bridge function**

Create `crates/automapper-api/src/validation_bridge.rs`:

```rust
//! Bridge between automapper-generator's AhbSchema and automapper-validation's AhbWorkflow.

use automapper_generator::schema::ahb::{AhbFieldDefinition, AhbSchema, Pruefidentifikator};
use automapper_validation::validator::{AhbCodeRule, AhbFieldRule, AhbWorkflow};

/// Convert an AhbSchema + PID into an AhbWorkflow for the validator.
///
/// Returns `None` if the PID is not found in the schema.
pub fn ahb_workflow_from_schema(schema: &AhbSchema, pid: &str) -> Option<AhbWorkflow> {
    let pruefid = schema.workflows.iter().find(|w| w.id == pid)?;
    Some(AhbWorkflow {
        pruefidentifikator: pid.to_string(),
        description: pruefid.description.clone().unwrap_or_default(),
        communication_direction: pruefid.communication_direction.clone(),
        fields: pruefid
            .fields
            .iter()
            .map(|f| AhbFieldRule {
                segment_path: f.segment_path.clone(),
                name: f.name.clone().unwrap_or_default(),
                ahb_status: f.ahb_status.clone(),
                codes: f
                    .codes
                    .iter()
                    .map(|c| AhbCodeRule {
                        value: c.value.clone(),
                        description: c.name.clone().unwrap_or_default(),
                        ahb_status: c.ahb_status.clone().unwrap_or_else(|| "X".to_string()),
                    })
                    .collect(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use automapper_generator::schema::ahb::{AhbCodeValue, BedingungDefinition};

    fn make_test_schema() -> AhbSchema {
        AhbSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            format_version: "FV2504".to_string(),
            workflows: vec![Pruefidentifikator {
                id: "55001".to_string(),
                description: Some("Anmeldung".to_string()),
                communication_direction: Some("NB an LF".to_string()),
                segment_numbers: vec!["0001".to_string()],
                fields: vec![AhbFieldDefinition {
                    segment_path: "SG2/NAD/3035".to_string(),
                    name: Some("Partnerrolle".to_string()),
                    ahb_status: "Muss".to_string(),
                    mig_number: None,
                    codes: vec![AhbCodeValue {
                        value: "MS".to_string(),
                        name: Some("Absender".to_string()),
                        description: None,
                        ahb_status: Some("X".to_string()),
                    }],
                }],
            }],
            bedingungen: vec![],
        }
    }

    #[test]
    fn test_bridge_known_pid() {
        let schema = make_test_schema();
        let workflow = ahb_workflow_from_schema(&schema, "55001").unwrap();

        assert_eq!(workflow.pruefidentifikator, "55001");
        assert_eq!(workflow.description, "Anmeldung");
        assert_eq!(workflow.fields.len(), 1);
        assert_eq!(workflow.fields[0].segment_path, "SG2/NAD/3035");
        assert_eq!(workflow.fields[0].ahb_status, "Muss");
        assert_eq!(workflow.fields[0].codes.len(), 1);
        assert_eq!(workflow.fields[0].codes[0].value, "MS");
    }

    #[test]
    fn test_bridge_unknown_pid() {
        let schema = make_test_schema();
        assert!(ahb_workflow_from_schema(&schema, "99999").is_none());
    }
}
```

**Step 3: Wire module**

In the appropriate module file for `automapper-api` (likely `src/lib.rs` or create a new module declaration), add:
```rust
pub mod validation_bridge;
```

**Step 4: Run tests**

```bash
cargo test -p automapper-api test_bridge
```

Expected: PASS. Note: the test constructs AHB types directly — check that `AhbFieldDefinition`, `AhbCodeValue`, `Pruefidentifikator` field names match the actual struct definitions in `automapper_generator::schema::ahb`. If field names differ (e.g., `description` might be `beschreibung`), adjust accordingly.

**Step 5: Commit**

```bash
cargo clippy -p automapper-api -- -D warnings && cargo fmt --all
git add crates/automapper-api/
git commit -m "feat(api): add AhbSchema→AhbWorkflow bridge for validation integration"
```

---

### Task 7: Add Validation Contracts and POST /api/v2/validate Endpoint

Add request/response types and the validation endpoint.

**Files:**
- Create: `crates/automapper-api/src/contracts/validate_v2.rs`
- Modify: `crates/automapper-api/src/contracts/mod.rs`
- Create: `crates/automapper-api/src/routes/validate_v2.rs`
- Modify: `crates/automapper-api/src/routes/mod.rs`

**Step 1: Create validation contracts**

Create `crates/automapper-api/src/contracts/validate_v2.rs`:

```rust
//! V2 validation request/response types.

use std::collections::HashMap;

use automapper_validation::validator::{ValidationLevel, ValidationReport};
use serde::{Deserialize, Serialize};

/// Request body for `POST /api/v2/validate`.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ValidateV2Request {
    /// Raw EDIFACT content to validate.
    pub input: String,

    /// Format version (e.g., "FV2504").
    pub format_version: String,

    /// Validation level. Defaults to "full".
    #[serde(default = "default_level")]
    pub level: ValidationLevel,

    /// Optional external condition overrides (condition_name → bool).
    pub external_conditions: Option<HashMap<String, bool>>,
}

fn default_level() -> ValidationLevel {
    ValidationLevel::Full
}

/// Response body for `POST /api/v2/validate`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ValidateV2Response {
    /// The validation report with all issues found.
    #[schema(value_type = Object)]
    pub report: serde_json::Value,

    /// Validation duration in milliseconds.
    pub duration_ms: f64,
}
```

Add to `crates/automapper-api/src/contracts/mod.rs`:
```rust
pub mod validate_v2;
```

Note: `ValidationLevel` needs `Deserialize` derive — check if it already has it in `crates/automapper-validation/src/validator/level.rs`. If not, add `Deserialize` to its derive list.

**Step 2: Create validation route handler**

Create `crates/automapper-api/src/routes/validate_v2.rs`:

```rust
//! V2 validation endpoint.

use std::collections::HashSet;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use automapper_validation::validator::{EdifactValidator, ValidationLevel};
use automapper_validation::eval::{NoOpExternalProvider, MapExternalProvider, CompositeExternalProvider};
use mig_assembly::assembler::Assembler;
use mig_assembly::pid_detect::detect_pid;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;

use crate::contracts::validate_v2::{ValidateV2Request, ValidateV2Response};
use crate::error::ApiError;
use crate::state::AppState;
use crate::validation_bridge::ahb_workflow_from_schema;

pub fn routes() -> Router<AppState> {
    Router::new().route("/validate", post(validate_v2))
}

pub(crate) async fn validate_v2(
    State(state): State<AppState>,
    Json(req): Json<ValidateV2Request>,
) -> Result<Json<ValidateV2Response>, ApiError> {
    let start = std::time::Instant::now();

    // Step 1: Tokenize
    let segments =
        parse_to_segments(req.input.as_bytes()).map_err(|e| ApiError::BadRequest {
            message: format!("EDIFACT parse error: {e}"),
        })?;

    // Step 2: Split into messages and detect PID from first message
    let chunks = mig_assembly::split_messages(segments).map_err(|e| ApiError::BadRequest {
        message: format!("message splitting error: {e}"),
    })?;

    let first_msg = chunks.messages.first().ok_or_else(|| ApiError::BadRequest {
        message: "No messages found in EDIFACT input".to_string(),
    })?;

    let all_segments = first_msg.all_segments();
    let pid = detect_pid(&all_segments).map_err(|e| ApiError::BadRequest {
        message: format!("PID detection error: {e}"),
    })?;

    // Step 3: Load AHB schema and build workflow
    // TODO: detect message variant from UNH instead of hardcoding
    let msg_variant = "UTILMD_Strom";
    let ahb = state
        .mig_registry
        .ahb_schema(&req.format_version, msg_variant)
        .ok_or_else(|| ApiError::BadRequest {
            message: format!(
                "No AHB schema for {}/{}",
                req.format_version, msg_variant
            ),
        })?;

    let workflow = ahb_workflow_from_schema(ahb, &pid).ok_or_else(|| ApiError::BadRequest {
        message: format!("PID {pid} not found in AHB"),
    })?;

    // Step 4: Assemble with diagnostics
    let service = state
        .mig_registry
        .service(&req.format_version)
        .ok_or_else(|| ApiError::BadRequest {
            message: format!("No MIG service for {}", req.format_version),
        })?;

    let ahb_workflow_entry = ahb.workflows.iter().find(|w| w.id == pid);
    let ahb_numbers: HashSet<String> = ahb_workflow_entry
        .map(|w| w.segment_numbers.iter().cloned().collect())
        .unwrap_or_default();

    let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);
    let assembler = Assembler::new(&filtered_mig);
    let (_tree, structure_diagnostics) = assembler.assemble_with_diagnostics(&all_segments);

    // Step 5: Build external provider
    let external: Box<dyn automapper_validation::eval::ExternalConditionProvider> =
        if let Some(ext_map) = req.external_conditions {
            Box::new(MapExternalProvider::new(ext_map))
        } else {
            Box::new(NoOpExternalProvider)
        };

    // Step 6: Create a no-op evaluator for now (generated evaluators come in Task 9)
    // TODO: look up real evaluator from EvaluatorRegistry
    let evaluator = StubEvaluator {
        message_type: "UTILMD".to_string(),
        format_version: req.format_version.clone(),
    };
    let validator = EdifactValidator::new(evaluator);

    // Step 7: Validate
    let mut report = validator.validate(&all_segments, &workflow, external.as_ref(), req.level);

    // Step 8: Add structure diagnostics to report
    for diag in structure_diagnostics {
        report.add_issue(
            automapper_validation::ValidationIssue::new(
                automapper_validation::Severity::Warning,
                automapper_validation::ValidationCategory::Structure,
                automapper_validation::ErrorCodes::UNEXPECTED_SEGMENT,
                diag.message,
            ),
        );
    }

    let report_json = serde_json::to_value(&report).unwrap_or_default();

    Ok(Json(ValidateV2Response {
        report: report_json,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}

/// Temporary stub evaluator until generated evaluators are available.
struct StubEvaluator {
    message_type: String,
    format_version: String,
}

impl automapper_validation::ConditionEvaluator for StubEvaluator {
    fn evaluate(
        &self,
        _condition: u32,
        _ctx: &automapper_validation::EvaluationContext,
    ) -> automapper_validation::ConditionResult {
        automapper_validation::ConditionResult::Unknown
    }
    fn is_external(&self, _condition: u32) -> bool {
        false
    }
    fn message_type(&self) -> &str {
        &self.message_type
    }
    fn format_version(&self) -> &str {
        &self.format_version
    }
}
```

**Step 3: Wire the route**

In `crates/automapper-api/src/routes/mod.rs`, add:
```rust
pub mod validate_v2;
```

And in `api_v2_routes()`:
```rust
pub fn api_v2_routes() -> Router<AppState> {
    Router::new()
        .merge(convert_v2::routes())
        .merge(reverse_v2::routes())
        .merge(validate_v2::routes())
}
```

**Step 4: Ensure ValidationLevel has Deserialize**

Check `crates/automapper-validation/src/validator/level.rs`. If `ValidationLevel` doesn't derive `Deserialize`, add it. Also ensure `ValidationReport` derives `Serialize` (it should already).

**Step 5: Build and test**

```bash
cargo check -p automapper-api
cargo test -p automapper-api
```

Fix any compilation errors. Common issues:
- Missing re-exports (add `AhbWorkflow` etc. to `automapper_validation::validator` public exports)
- Field name mismatches between generator's AHB types and what the bridge expects
- Missing `Serialize`/`Deserialize` derives

**Step 6: Commit**

```bash
cargo clippy -p automapper-api -- -D warnings && cargo fmt --all
git add crates/automapper-api/ crates/automapper-validation/
git commit -m "feat(api): add POST /api/v2/validate endpoint

Validates EDIFACT against AHB rules. Uses stub evaluator (returns Unknown
for all conditions) until generated evaluators are available. Includes
structure diagnostics from assembler in the validation report."
```

---

### Task 8: Add validate Flag to Convert Endpoint

Add an optional `validate` query parameter to the existing `POST /api/v2/convert` endpoint.

**Files:**
- Modify: `crates/automapper-api/src/contracts/convert_v2.rs`
- Modify: `crates/automapper-api/src/routes/convert_v2.rs`

**Step 1: Add validate field to query params**

In `crates/automapper-api/src/contracts/convert_v2.rs`, add to `ConvertV2Query`:

```rust
/// Run validation and include report in response. Defaults to `false`.
pub validate: Option<bool>,
```

Add to `ConvertV2Response`:
```rust
/// Validation report (present when `validate=true`).
#[serde(skip_serializing_if = "Option::is_none")]
#[schema(value_type = Option<Object>)]
pub validation: Option<serde_json::Value>,
```

**Step 2: Wire validation into Bo4e convert handler**

In `crates/automapper-api/src/routes/convert_v2.rs`, in the `ConvertMode::Bo4e` arm, after the mapping produces `interchange`, add:

```rust
let validation = if query.validate.unwrap_or(false) {
    // Reuse the segments and workflow from conversion for validation
    // Use the first message's workflow
    if let Some(first_msg) = chunks.messages.first() {
        let all_segs = first_msg.all_segments();
        let pid = detect_pid(&all_segs).ok();
        let workflow = pid.as_ref().and_then(|p| {
            let ahb = state.mig_registry.ahb_schema(&req.format_version, msg_variant)?;
            crate::validation_bridge::ahb_workflow_from_schema(ahb, p)
        });
        if let Some(wf) = workflow {
            let evaluator = crate::routes::validate_v2::StubEvaluator {
                message_type: "UTILMD".to_string(),
                format_version: req.format_version.clone(),
            };
            let validator = automapper_validation::EdifactValidator::new(evaluator);
            let external = automapper_validation::eval::NoOpExternalProvider;
            let report = validator.validate(
                &all_segs,
                &wf,
                &external,
                automapper_validation::ValidationLevel::Full,
            );
            Some(serde_json::to_value(&report).unwrap_or_default())
        } else {
            None
        }
    } else {
        None
    }
} else {
    None
};
```

Then include `validation` in the response:
```rust
Ok(Json(ConvertV2Response {
    mode: "bo4e".to_string(),
    result: serde_json::to_value(&interchange).unwrap_or_default(),
    duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    validation,
}))
```

Note: Make `StubEvaluator` in `validate_v2.rs` public so convert_v2 can use it, or extract it to a shared module.

**Step 3: Run tests**

```bash
cargo test -p automapper-api
```

**Step 4: Commit**

```bash
cargo clippy -p automapper-api -- -D warnings && cargo fmt --all
git add crates/automapper-api/
git commit -m "feat(api): add validate flag to POST /api/v2/convert

When ?validate=true, includes a ValidationReport in the response
alongside the conversion result."
```

---

### Task 9: Generate and Commit Condition Evaluators

The `GenerateConditions` CLI subcommand already exists (main.rs:385-618). This task runs it and commits the output. **This task requires the Claude API** — it invokes `claude --print --model sonnet` to generate Rust condition logic.

**Files:**
- Create: `crates/automapper-validation/src/generated/mod.rs`
- Create: `crates/automapper-validation/src/generated/fv2504/mod.rs`
- Create: `crates/automapper-validation/src/generated/fv2504/utilmd_strom.rs` (generated)
- Modify: `crates/automapper-validation/src/lib.rs` (add `generated` module)

**Step 1: Create generated directory structure**

```bash
mkdir -p crates/automapper-validation/src/generated/fv2504
```

**Step 2: Run the condition generator**

```bash
cargo run -p automapper-generator -- generate-conditions \
  --ahb-path xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml \
  --message-type UTILMD \
  --format-version FV2504 \
  --output-dir crates/automapper-validation/src/generated/fv2504 \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml
```

This shells out to the Claude CLI for each condition. Review the output. If any conditions fail to generate, the tool records them in metadata and they'll return `Unknown`.

**Step 3: Review generated code**

Read the generated file and verify:
- It imports from `automapper_validation::eval::*`
- The `evaluate()` method uses the correct `EvaluationContext` (with `OwnedSegment`)
- External conditions call `ctx.external.evaluate(name)` properly
- The code compiles: `cargo check -p automapper-validation`

Fix any compilation errors in the generated code manually.

**Step 4: Wire the generated module**

Create `crates/automapper-validation/src/generated/mod.rs`:
```rust
//! Generated condition evaluators.
//!
//! These files are produced by `automapper-generator generate-conditions`
//! and committed to the repository. Do not edit by hand.

pub mod fv2504;
```

Create `crates/automapper-validation/src/generated/fv2504/mod.rs`:
```rust
pub mod utilmd_strom;
```

Add to `crates/automapper-validation/src/lib.rs`:
```rust
pub mod generated;
```

**Step 5: Verify compilation**

```bash
cargo check -p automapper-validation
cargo test -p automapper-validation
```

**Step 6: Commit**

```bash
cargo clippy -p automapper-validation -- -D warnings && cargo fmt --all
git add crates/automapper-validation/src/generated/
git commit -m "feat(validation): add generated condition evaluators for UTILMD FV2504

Generated by automapper-generator generate-conditions from AHB XML.
Implements ConditionEvaluator for UTILMD Strom FV2504."
```

---

### Task 10: Wire Generated Evaluator into API

Replace the `StubEvaluator` in the validate endpoint with the actual generated evaluator.

**Files:**
- Modify: `crates/automapper-api/src/routes/validate_v2.rs`
- Modify: `crates/automapper-api/src/state.rs` (register evaluators at startup)

**Step 1: Register evaluator at startup**

In `crates/automapper-api/src/state.rs`, in `AppState::new()` or `MigServiceRegistry::discover()`, after loading schemas:

```rust
// Register generated condition evaluators
use automapper_validation::eval::EvaluatorRegistry;

// Create a shared evaluator registry
let evaluator_registry = {
    let registry = EvaluatorRegistry::new();
    // Register UTILMD Strom FV2504 evaluator
    let evaluator = automapper_validation::generated::fv2504::utilmd_strom::UtilmdStromFv2504Evaluator::default();
    registry.register(evaluator);
    Arc::new(registry)
};
```

Add `evaluator_registry: Arc<EvaluatorRegistry>` to `AppState`.

**Step 2: Use real evaluator in validate endpoint**

In `validate_v2.rs`, replace the `StubEvaluator` creation with a lookup:

```rust
// Look up condition evaluator from registry
let evaluator_opt = state.evaluator_registry.get("UTILMD", &req.format_version);
// Fall back to stub if no evaluator registered
```

If an evaluator exists, use it. If not, fall back to the stub.

**Step 3: Test**

```bash
cargo test -p automapper-api
```

**Step 4: Commit**

```bash
cargo clippy -p automapper-api -- -D warnings && cargo fmt --all
git add crates/automapper-api/
git commit -m "feat(api): wire generated condition evaluator into validation endpoint

Registers UTILMD Strom FV2504 evaluator at startup. Falls back to
stub evaluator for unregistered message type/format version combos."
```

---

### Task 11: Integration Tests with Real Fixtures

End-to-end tests using real EDIFACT fixtures from the submodule.

**Files:**
- Create: `crates/automapper-api/tests/validate_v2_test.rs`
- Create: `crates/automapper-validation/tests/real_fixture_test.rs`

**Step 1: API integration test**

Create `crates/automapper-api/tests/validate_v2_test.rs`:

```rust
//! Integration tests for POST /api/v2/validate.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;
use common::app;

#[tokio::test]
async fn test_validate_endpoint_returns_report() {
    let fixture_path = "example_market_communication_bo4e_transactions/UTILMD/FV2504";
    let fixture_dir = std::path::Path::new(fixture_path);
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture dir not found at {fixture_path}");
        return;
    }

    // Find a 55001 fixture
    let fixture = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.contains("55001") && name.ends_with(".txt")
        });

    let Some(fixture) = fixture else {
        eprintln!("Skipping: no 55001 fixture found");
        return;
    };

    let edifact = std::fs::read_to_string(fixture.path()).unwrap();

    let body = serde_json::json!({
        "input": edifact,
        "format_version": "FV2504",
        "level": "full"
    });

    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the response contains a report
    assert!(result.get("report").is_some());
    assert!(result.get("duration_ms").is_some());

    // The report should have basic structure
    let report = &result["report"];
    assert!(report.get("message_type").is_some());
    assert!(report.get("level").is_some());
    assert!(report.get("issues").is_some());
}

#[tokio::test]
async fn test_validate_invalid_edifact() {
    let body = serde_json::json!({
        "input": "this is not valid edifact",
        "format_version": "FV2504",
    });

    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v2/validate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 400 for unparseable EDIFACT
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

Note: Adjust the `common::app()` pattern to match how existing API tests create the app instance. Check `crates/automapper-api/tests/api_integration.rs` for the exact pattern used.

**Step 2: Validation crate integration test with real parsed segments**

Create `crates/automapper-validation/tests/real_fixture_test.rs`:

```rust
//! Integration tests with real EDIFACT fixtures.

use mig_types::segment::OwnedSegment;
use automapper_validation::{
    EdifactValidator, ValidationLevel, ConditionResult, EvaluationContext,
    ConditionEvaluator,
};
use automapper_validation::eval::NoOpExternalProvider;
use automapper_validation::validator::{AhbWorkflow, AhbFieldRule};

/// Stub evaluator for integration testing without generated evaluators.
struct AllUnknownEvaluator;

impl ConditionEvaluator for AllUnknownEvaluator {
    fn evaluate(&self, _condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
        ConditionResult::Unknown
    }
    fn is_external(&self, _condition: u32) -> bool { false }
    fn message_type(&self) -> &str { "UTILMD" }
    fn format_version(&self) -> &str { "FV2504" }
}

#[test]
fn test_validate_parsed_55001_fixture() {
    // Load and parse a real fixture
    let fixture_path = "example_market_communication_bo4e_transactions/UTILMD/FV2504";
    let fixture_dir = std::path::Path::new(fixture_path);
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture dir not found");
        return;
    }

    let fixture = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("55001"));

    let Some(fixture) = fixture else {
        eprintln!("Skipping: no 55001 fixture");
        return;
    };

    let edifact = std::fs::read(fixture.path()).unwrap();
    let segments = mig_assembly::tokenize::parse_to_segments(&edifact).unwrap();

    // Create a minimal workflow with a "Muss" field for NAD
    let workflow = AhbWorkflow {
        pruefidentifikator: "55001".to_string(),
        description: "Test".to_string(),
        communication_direction: None,
        fields: vec![AhbFieldRule {
            segment_path: "SG2/NAD/3035".to_string(),
            name: "Partnerrolle".to_string(),
            ahb_status: "Muss".to_string(),
            codes: vec![],
        }],
    };

    let validator = EdifactValidator::new(AllUnknownEvaluator);
    let external = NoOpExternalProvider;

    let report = validator.validate(&segments, &workflow, &external, ValidationLevel::Conditions);

    // Real 55001 fixture should have NAD segments → no missing mandatory errors
    assert!(report.is_valid(), "Report: {:#?}", report);
}
```

**Step 3: Add dev-dependencies**

In `crates/automapper-validation/Cargo.toml` add:
```toml
[dev-dependencies]
mig-assembly = { workspace = true }
```

**Step 4: Run tests**

```bash
cargo test -p automapper-validation test_validate_parsed
cargo test -p automapper-api test_validate
```

**Step 5: Commit**

```bash
cargo clippy --workspace -- -D warnings && cargo fmt --all
git add crates/automapper-api/tests/ crates/automapper-validation/tests/
git commit -m "test(validation): add integration tests with real EDIFACT fixtures

API endpoint test validates 55001 fixture via POST /api/v2/validate.
Validation crate test parses real fixture and runs condition validation."
```

---

### Task 12: Snapshot Tests for Validation Reports

Capture validation report structure using `insta` snapshots.

**Files:**
- Modify: `crates/automapper-validation/Cargo.toml` (add insta dev-dep)
- Create: `crates/automapper-validation/tests/snapshot_test.rs`

**Step 1: Add insta dependency**

In `crates/automapper-validation/Cargo.toml`:
```toml
[dev-dependencies]
insta = { workspace = true, features = ["json"] }
```

**Step 2: Write snapshot test**

Create `crates/automapper-validation/tests/snapshot_test.rs`:

```rust
//! Snapshot tests for ValidationReport serialization.

use automapper_validation::{
    EdifactValidator, ValidationLevel, ConditionResult, EvaluationContext,
    ConditionEvaluator, Severity, ValidationCategory, ErrorCodes,
};
use automapper_validation::eval::NoOpExternalProvider;
use automapper_validation::validator::{AhbWorkflow, AhbFieldRule, AhbCodeRule};
use mig_types::segment::OwnedSegment;

struct TestEvaluator {
    results: std::collections::HashMap<u32, ConditionResult>,
}

impl ConditionEvaluator for TestEvaluator {
    fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
        self.results.get(&condition).copied().unwrap_or(ConditionResult::Unknown)
    }
    fn is_external(&self, _: u32) -> bool { false }
    fn message_type(&self) -> &str { "UTILMD" }
    fn format_version(&self) -> &str { "FV2504" }
}

#[test]
fn test_snapshot_clean_report() {
    let evaluator = TestEvaluator { results: std::collections::HashMap::new() };
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let segments = vec![
        OwnedSegment { id: "NAD".to_string(), elements: vec![vec!["MS".to_string()]], segment_number: 1 },
    ];

    let workflow = AhbWorkflow {
        pruefidentifikator: "55001".to_string(),
        description: "Anmeldung".to_string(),
        communication_direction: Some("NB an LF".to_string()),
        fields: vec![AhbFieldRule {
            segment_path: "SG2/NAD/3035".to_string(),
            name: "Partnerrolle".to_string(),
            ahb_status: "Muss".to_string(),
            codes: vec![],
        }],
    };

    let report = validator.validate(&segments, &workflow, &external, ValidationLevel::Full);
    let json = serde_json::to_value(&report).unwrap();
    insta::assert_json_snapshot!("clean_report", json);
}

#[test]
fn test_snapshot_report_with_errors() {
    let mut results = std::collections::HashMap::new();
    results.insert(182, ConditionResult::True);
    results.insert(152, ConditionResult::True);
    let evaluator = TestEvaluator { results };
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    // Empty segments — mandatory fields will be missing
    let segments: Vec<OwnedSegment> = vec![];

    let workflow = AhbWorkflow {
        pruefidentifikator: "55001".to_string(),
        description: "Anmeldung".to_string(),
        communication_direction: None,
        fields: vec![
            AhbFieldRule {
                segment_path: "SG2/NAD/C082/3039".to_string(),
                name: "MP-ID des MSB".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            },
            AhbFieldRule {
                segment_path: "DTM/C507/2380".to_string(),
                name: "Datum".to_string(),
                ahb_status: "X".to_string(),
                codes: vec![],
            },
        ],
    };

    let report = validator.validate(&segments, &workflow, &external, ValidationLevel::Full);
    let json = serde_json::to_value(&report).unwrap();
    insta::assert_json_snapshot!("report_with_errors", json);
}
```

**Step 3: Generate snapshots**

```bash
cargo insta test -p automapper-validation -- snapshot
cargo insta review
```

Accept the snapshots after reviewing them.

**Step 4: Run tests**

```bash
cargo test -p automapper-validation -- snapshot
```

Expected: PASS (snapshots match).

**Step 5: Commit**

```bash
cargo fmt --all
git add crates/automapper-validation/
git commit -m "test(validation): add snapshot tests for ValidationReport JSON structure"
```

---

### Task 13: Final Workspace Verification

Run the full workspace test suite and clippy to make sure everything integrates cleanly.

**Files:** None (verification only)

**Step 1: Full workspace check**

```bash
cargo check --workspace
```

**Step 2: Full workspace tests**

```bash
cargo test --workspace --exclude automapper-web
```

**Step 3: Clippy**

```bash
cargo clippy --workspace -- -D warnings
```

**Step 4: Format check**

```bash
cargo fmt --all -- --check
```

**Step 5: Fix any issues and commit**

If any tests fail or clippy warnings appear, fix them. Final commit:

```bash
git add -A
git commit -m "chore: fix workspace-wide clippy and test issues from validation integration"
```

---

## Summary

| Task | Epic | Description | Key Files |
|------|------|-------------|-----------|
| 1 | Core | Switch EvaluationContext to OwnedSegment | `eval/context.rs`, `Cargo.toml` |
| 2 | Core | Simplify EdifactValidator API | `validator/validate.rs` |
| 3 | Providers | Add Map + Composite external providers | `eval/providers.rs` |
| 4 | Diagnostics | Define StructureDiagnostic types | `mig-assembly/diagnostic.rs` |
| 5 | Diagnostics | Implement assemble_with_diagnostics | `mig-assembly/assembler.rs` |
| 6 | Bridge | AhbSchema→AhbWorkflow conversion | `api/validation_bridge.rs` |
| 7 | API | POST /api/v2/validate endpoint | `routes/validate_v2.rs`, contracts |
| 8 | API | validate flag on convert endpoint | `routes/convert_v2.rs` |
| 9 | Codegen | Generate condition evaluators | `generated/fv2504/utilmd_strom.rs` |
| 10 | API | Wire generated evaluator into API | `state.rs`, `validate_v2.rs` |
| 11 | Tests | Integration tests with real fixtures | `tests/validate_v2_test.rs` |
| 12 | Tests | Snapshot tests for reports | `tests/snapshot_test.rs` |
| 13 | Verify | Full workspace verification | — |

**Dependencies:** Tasks 1→2 (sequential), 3 (after 1), 4→5 (sequential), 6 (after 2), 7 (after 2+3+5+6), 8 (after 7), 9 (after 1, can parallel with 3-8), 10 (after 7+9), 11 (after 10), 12 (after 2), 13 (last).

**Critical path:** 1 → 2 → 7 → 9 → 10 → 11 → 13

**Parallelizable:** Tasks 3, 4-5, 6, 12 can all run in parallel after Task 2 completes.
