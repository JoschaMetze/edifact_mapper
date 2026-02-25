# Validation Integration Design

**Date:** 2026-02-25
**Status:** Draft
**Scope:** Full integration — all 7 gaps connecting `automapper-validation` to the MIG-driven pipeline

## Context

The `automapper-validation` crate has solid internals (condition expression parser, three-valued logic evaluator, report types — 143 tests), but it's isolated from the rest of the pipeline. Seven gaps prevent it from processing real EDIFACT messages:

1. `parse_segments()` returns empty vec — validator can't process real input
2. `validate_structure()` is a no-op — no MIG schema validation
3. No generated `ConditionEvaluator` implementations for any PID
4. `AhbWorkflow` must be manually constructed — no bridge from `AhbSchema`
5. No validation endpoint in the API
6. No `ExternalConditionProvider` implementations
7. No integration tests with real fixtures

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Codegen model | Generated Rust files, committed | Matches `mig-types` pattern. No runtime LLM dependency. |
| Structure validation | Leverage assembly diagnostics | Assembler already knows MIG rules. Enhance to collect diagnostics. |
| API exposure | Separate endpoint + inline flag | `POST /api/v2/validate` standalone, plus `validate=true` on convert |
| External conditions | Pluggable provider traits + stub | Three-valued logic handles Unknown gracefully. Callers upgrade incrementally. |
| Validator input | Pre-parsed segments (not raw EDIFACT) | Cleaner separation. API orchestrates; validator is a pure engine. |

## Architecture Overview

### Data Flow

```
EDIFACT bytes
  → edifact-parser: parse_to_segments()        → Vec<OwnedSegment>
  → detect_pid() from RFF+Z13                  → PID string
  → MigServiceRegistry: load MIG + AHB         → (MigSchema, AhbSchema)
  → pid_filter::filter_mig_for_pid()           → PID-specific MIG
  → Assembler::assemble_with_diagnostics()     → (AssembledTree, Vec<StructureDiagnostic>)
  → AhbSchema → AhbWorkflow conversion         → AhbWorkflow
  → EvaluatorRegistry::get(msg_type, fv)       → &dyn ConditionEvaluator
  → EdifactValidator::validate()               → ValidationReport
  → serialize                                  → JSON response
```

The validator doesn't re-parse or re-assemble. It receives pre-parsed segments and assembly diagnostics from the shared pipeline. Both the validate and convert endpoints share the same parsing/assembly code path.

### Crate Dependency Changes

- `automapper-validation`: no new crate dependencies (receives diagnostics as `ValidationIssue` from API layer)
- `automapper-api`: adds dependency on `automapper-validation`
- `automapper-generator`: adds `generate-conditions` CLI subcommand
- Generated evaluators live in `crates/automapper-validation/src/generated/` (committed to git)

## Component Details

### 1. Assembly Diagnostics (Gap 2)

Enhance `mig-assembly::Assembler` to collect structure diagnostics during assembly without changing the assembly result.

**New types in `mig-assembly`:**

```rust
/// Diagnostic from assembly — structure-level issue found during MIG-guided assembly
pub struct StructureDiagnostic {
    pub kind: StructureDiagnosticKind,
    pub segment_id: String,          // e.g., "LOC", "DTM"
    pub position: usize,             // segment index in input
    pub message: String,             // human-readable description
}

pub enum StructureDiagnosticKind {
    UnexpectedSegment,       // segment not in PID-filtered MIG
    MissingRequiredSegment,  // MIG says mandatory, not found
    MaxRepetitionsExceeded,  // more instances than MIG allows
    UnrecognizedQualifier,   // qualifier value not in MIG enum
}
```

**New method:**

```rust
impl Assembler {
    /// Like assemble_generic but also returns diagnostics
    pub fn assemble_with_diagnostics(
        &self,
        segments: &[OwnedSegment],
    ) -> (AssembledTree, Vec<StructureDiagnostic>)
}
```

The existing `assemble_generic()` remains unchanged (non-breaking). The API layer converts `StructureDiagnostic` into `ValidationIssue` — the validation crate stays generic and doesn't depend on assembly types.

### 2. Validator API Simplification (Gap 1)

Remove `parse_segments()` from `EdifactValidator` entirely. Change the validator to accept pre-parsed data:

```rust
impl<E: ConditionEvaluator> EdifactValidator<E> {
    /// Validate pre-parsed segments against AHB rules
    pub fn validate(
        &self,
        segments: &[OwnedSegment],
        workflow: &AhbWorkflow,
        external: &dyn ExternalConditionProvider,
        level: ValidationLevel,
    ) -> ValidationReport

    /// Validate conditions only (used when structure diagnostics come from assembler)
    pub fn validate_conditions(
        &self,
        segments: &[OwnedSegment],
        workflow: &AhbWorkflow,
        external: &dyn ExternalConditionProvider,
    ) -> Vec<ValidationIssue>
}
```

The validator becomes a pure validation engine. Parsing, PID detection, MIG loading, and assembly happen in the API layer before calling the validator.

### 3. AhbWorkflow Bridge (Gap 4)

Conversion function from the generator's AHB types to the validator's input types. Lives in the API crate as glue code:

```rust
pub fn ahb_workflow_from_schema(
    schema: &AhbSchema,
    pid: &str,
) -> Option<AhbWorkflow> {
    let pruefid = schema.workflows.iter().find(|p| p.id == pid)?;
    Some(AhbWorkflow {
        pruefidentifikator: pid.to_string(),
        fields: pruefid.fields.iter().map(|f| AhbFieldRule {
            segment_path: f.segment_path.clone(),
            ahb_status: f.ahb_status.clone(),
            allowed_codes: f.codes.iter().map(|c| c.value.clone()).collect(),
        }).collect(),
    })
}
```

### 4. Condition Evaluator Generation (Gap 3)

**New CLI subcommand on `automapper-generator`:**

```bash
cargo run -p automapper-generator -- generate-conditions \
  --ahb-xml xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_*.xml \
  --message-type UTILMD \
  --format-version FV2504 \
  --output-dir crates/automapper-validation/src/generated/fv2504/
```

**Generated output:**

```
crates/automapper-validation/src/generated/
├── mod.rs                      # re-exports, registry population fn
└── fv2504/
    ├── mod.rs
    └── utilmd_strom.rs          # UtilmdStromFv2504Evaluator
```

Each generated file contains:
1. A struct implementing `ConditionEvaluator` with `evaluate()` dispatching to per-condition methods
2. A `register()` function that adds the evaluator to `EvaluatorRegistry`
3. A list of external condition IDs (conditions the evaluator can't resolve locally)

**Condition implementation:** The existing `claude_generator.rs` sends AHB condition descriptions to the Claude API and gets back Rust code snippets. For data-inspection conditions, the generated code uses `EvaluationContext::find_segment()` and `find_segments_with_qualifier()`. For external conditions, the code calls `ctx.external.evaluate(condition_id)`.

**Metadata tracking:** Each condition gets a SHA-256 hash of its AHB description stored in a `.conditions.json` sidecar. Re-running the generator only regenerates conditions whose description changed (staleness detection).

**Bootstrap:** Generate evaluators for PIDs 55001 and 55002 first — we have fixtures and TOML mappings for both.

### 5. External Condition Providers (Gap 6)

**Provider traits in `automapper-validation`:**

```rust
/// Base trait — evaluate arbitrary external conditions by ID
pub trait ExternalConditionProvider: Send + Sync {
    fn evaluate(&self, condition_id: u32, ctx: &EvaluationContext) -> ConditionResult;
    fn supported_conditions(&self) -> &[u32];
}

/// Role-based conditions (e.g., "Is MP a Lieferant?")
pub trait MarktpartnerRoleProvider: Send + Sync {
    fn has_role(&self, mp_id: &str, role: &str) -> ConditionResult;
}

/// Product configuration conditions (Chapter 6 Codeliste)
pub trait ProductConfigurationProvider: Send + Sync {
    fn has_supplier_payment_recipient(&self, product_code: &str) -> ConditionResult;
    fn is_property_code_allowed(&self, product_code: &str, property_code: &str) -> ConditionResult;
    fn is_value_in_range(&self, product_code: &str, value: &str) -> ConditionResult;
}
```

**Implementations:**

- `UnknownExternalProvider`: Returns `ConditionResult::Unknown` for everything. Default when no provider configured.
- `MapExternalProvider`: Wraps `HashMap<u32, bool>` — used for API-supplied overrides.
- `CompositeExternalProvider`: Wraps multiple providers, checks each in order. First non-Unknown result wins.

Three-valued logic makes the stub safe — validation still works, reporting Info-level "could not evaluate condition [X]" instead of hard errors. Callers upgrade incrementally by implementing specific providers.

### 6. API Endpoints (Gap 5)

**Standalone — `POST /api/v2/validate`:**

```rust
#[derive(Deserialize)]
pub struct ValidateRequest {
    pub edifact: String,
    pub format_version: String,        // e.g., "FV2504"
    pub message_variant: String,       // e.g., "Strom"
    pub level: Option<ValidationLevel>, // default: Full
    pub external_conditions: Option<HashMap<u32, bool>>,
}

#[derive(Serialize)]
pub struct ValidateResponse {
    pub report: ValidationReport,
}
```

**Inline on convert:**

```rust
// Added to existing ConvertRequest
pub validate: Option<bool>,  // default: false

// Added to ConvertResponse
pub validation: Option<ValidationReport>,  // present when validate=true
```

**Shared pipeline:** Both endpoints share a `ValidationPipeline` helper:

```rust
pub struct ValidationPipeline {
    registry: Arc<MigServiceRegistry>,
    evaluator_registry: Arc<EvaluatorRegistry>,
}

impl ValidationPipeline {
    pub fn validate(
        &self,
        edifact: &str,
        format_version: &str,
        message_variant: &str,
        level: ValidationLevel,
        external_conditions: Option<HashMap<u32, bool>>,
    ) -> Result<ValidationReport, ApiError>
}
```

This encapsulates: parse → detect PID → load MIG/AHB → filter → assemble with diagnostics → build AhbWorkflow → get evaluator → validate → combine structure diagnostics + AHB issues into final report.

## Testing Strategy

### Unit Tests (in `automapper-validation`)

- Existing 143 tests — keep unchanged
- New tests for provider types: `UnknownExternalProvider`, `CompositeExternalProvider`, `MapExternalProvider`
- New tests for simplified `validate()` API accepting pre-parsed segments

### Assembly Diagnostic Tests (in `mig-assembly`)

- `assemble_with_diagnostics()` with PID 55001/55002 fixtures → empty diagnostics
- Crafted malformed input (missing LOC, extra DTM, wrong qualifier) → verify diagnostic kinds

### Validation Integration Tests (in `automapper-validation/tests/`)

- Parse real EDIFACT fixtures with `edifact-parser`, pass to validator with generated evaluators
- Known-good messages → clean reports
- Intentionally broken messages → specific errors

### API Integration Tests (in `automapper-api`)

- `POST /api/v2/validate` with real fixtures → 200 + clean report
- `POST /api/v2/validate` with bad EDIFACT → 200 + report with errors
- `POST /api/v2/convert?validate=true` → response includes both result and report
- External conditions override: supply map, verify it affects evaluation

### Snapshot Tests (with `insta`)

- Snapshot full `ValidationReport` JSON for PID 55001 and 55002 fixtures
- Catches regressions in report structure and content

## Implementation Order

### Epic 1: Validator Core Simplification
1. Remove `parse_segments()` from EdifactValidator
2. Change `validate()` to accept pre-parsed segments + AhbWorkflow
3. Add `validate_conditions()` method
4. Update existing unit tests for new API

### Epic 2: Assembly Diagnostics
5. Define `StructureDiagnostic` types in `mig-assembly`
6. Implement `assemble_with_diagnostics()`
7. Add diagnostic tests with malformed input
8. Test with PID 55001/55002 well-formed fixtures → empty diagnostics

### Epic 3: External Condition Providers
9. Define provider traits (`ExternalConditionProvider`, `MarktpartnerRoleProvider`, `ProductConfigurationProvider`)
10. Implement `UnknownExternalProvider`, `MapExternalProvider`, `CompositeExternalProvider`
11. Unit tests for all providers

### Epic 4: Condition Evaluator Generation
12. Add `generate-conditions` CLI subcommand to `automapper-generator`
13. Generate evaluators for PID 55001 (UTILMD Strom FV2504)
14. Generate evaluators for PID 55002
15. Commit generated code, verify it compiles and tests pass

### Epic 5: AhbWorkflow Bridge & Pipeline
16. Implement `ahb_workflow_from_schema()` conversion
17. Build `ValidationPipeline` shared helper
18. Validation integration tests with real fixtures

### Epic 6: API Endpoints
19. Add `POST /api/v2/validate` endpoint
20. Add `validate` flag to existing convert endpoint
21. API integration tests
22. Snapshot tests for validation reports
