---
feature: mig-driven-mapping
epic: 6
title: "Integration & Dual API"
depends_on: [mig-driven-mapping/E05]
estimated_tasks: 5
crate: automapper-api, mig-assembly, mig-bo4e
status: complete
---

# Epic 6: Integration & Dual API

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Wire the new MIG-driven pipeline into `automapper-api` as a dual API — consumers can request either the typed MIG tree or BO4E objects. Validate equivalence between the old `automapper-core` path and the new MIG-driven path. Add migration comparison tests and performance benchmarks.

**Architecture:** The API exposes two conversion modes: `mode=mig-tree` returns the assembled tree as JSON, `mode=bo4e` returns BO4E objects (using the TOML mapping engine). Both modes share the same tokenization and assembly pass. The old `automapper-core` pipeline remains available as `mode=legacy` during migration. A comparison test suite verifies that the new and old paths produce equivalent BO4E output for all fixture files.

**Tech Stack:** Rust, axum (REST API), serde_json (JSON serialization), mig-assembly, mig-bo4e, automapper-core (legacy path)

---

## Task 1: Conversion Service Abstraction

**Files:**
- Create: `crates/mig-assembly/src/service.rs`
- Modify: `crates/mig-assembly/src/lib.rs`

**Step 1: Write the failing test**

Create test in `crates/mig-assembly/tests/service_test.rs`:

```rust
use mig_assembly::service::ConversionService;
use std::path::Path;

#[test]
fn test_conversion_service_mig_tree_mode() {
    let service = ConversionService::new(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    ).unwrap();

    let fixture = Path::new("../../example_market_communication_bo4e_transactions/UTILMD");
    if !fixture.exists() {
        return;
    }

    let first_file = std::fs::read_dir(fixture).unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "txt").unwrap_or(false))
        .unwrap();

    let content = std::fs::read_to_string(first_file.path()).unwrap();

    // Convert to MIG tree JSON
    let tree_json = service.convert_to_tree(&content);
    assert!(tree_json.is_ok(), "Tree conversion failed: {:?}", tree_json.err());

    let json = tree_json.unwrap();
    assert!(json.is_object() || json.is_array(), "Expected JSON object/array");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_conversion_service_mig_tree_mode -- --nocapture`
Expected: FAIL

**Step 3: Implement ConversionService**

Create `crates/mig-assembly/src/service.rs`:

```rust
//! High-level conversion service that orchestrates the full pipeline.

use std::path::Path;
use crate::assembler::Assembler;
use crate::AssemblyError;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;

pub struct ConversionService {
    mig: MigSchema,
}

impl ConversionService {
    pub fn new(
        mig_path: &Path,
        message_type: &str,
        variant: Option<&str>,
        format_version: &str,
    ) -> Result<Self, AssemblyError> {
        let mig = parse_mig(mig_path, message_type, variant, format_version)
            .map_err(|e| AssemblyError::ParseError(e.to_string()))?;
        Ok(Self { mig })
    }

    /// Convert EDIFACT input to an assembled tree, serialized as JSON.
    pub fn convert_to_tree(&self, input: &str) -> Result<serde_json::Value, AssemblyError> {
        let segments = edifact_parser::parse_to_segments(input);
        let assembler = Assembler::new(&self.mig);
        let tree = assembler.assemble_generic(&segments)?;
        serde_json::to_value(&tree)
            .map_err(|e| AssemblyError::ParseError(e.to_string()))
    }

    /// Get a reference to the loaded MIG schema.
    pub fn mig(&self) -> &MigSchema {
        &self.mig
    }
}
```

Add `pub mod service;` to `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_conversion_service_mig_tree_mode -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/service.rs crates/mig-assembly/src/lib.rs crates/mig-assembly/tests/
git commit -m "feat(mig-assembly): add ConversionService for high-level pipeline access"
```

---

## Task 2: Dual API Endpoints in automapper-api

**Files:**
- Modify: `crates/automapper-api/src/routes.rs` (or wherever routes are defined)

**Step 1: Write the failing test**

Add integration test for the new endpoint:

```rust
#[tokio::test]
async fn test_convert_endpoint_mig_tree_mode() {
    // Create test app with the new route
    let app = create_test_app();

    let body = serde_json::json!({
        "input": "UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01+MSG001+9'UNT+3+1'",
        "mode": "mig-tree",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(axum::http::Request::builder()
            .method("POST")
            .uri("/api/v2/convert")
            .header("content-type", "application/json")
            .body(serde_json::to_string(&body).unwrap().into())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap()
    ).unwrap();

    // Response should contain the assembled tree
    assert!(body.get("tree").is_some() || body.get("segments").is_some());
}

#[tokio::test]
async fn test_convert_endpoint_bo4e_mode() {
    let app = create_test_app();

    let body = serde_json::json!({
        "input": "UNA:+.? 'UNH+1+UTILMD:D:11A:UN:S2.1'BGM+E01+MSG001+9'UNT+3+1'",
        "mode": "bo4e",
        "format_version": "FV2504"
    });

    let response = app
        .oneshot(axum::http::Request::builder()
            .method("POST")
            .uri("/api/v2/convert")
            .header("content-type", "application/json")
            .body(serde_json::to_string(&body).unwrap().into())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-api test_convert_endpoint -- --nocapture`
Expected: FAIL

**Step 3: Implement dual API endpoint**

Add a new `/api/v2/convert` endpoint that accepts a `mode` parameter:
- `mode=mig-tree`: runs tokenize → assemble → serialize tree as JSON
- `mode=bo4e`: runs tokenize → assemble → TOML mapping → serialize BO4E as JSON
- `mode=legacy`: uses the existing `automapper-core` pipeline

```rust
#[derive(Deserialize)]
struct ConvertRequest {
    input: String,
    mode: String, // "mig-tree", "bo4e", "legacy"
    format_version: String,
}

async fn convert_v2(
    State(state): State<AppState>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match req.mode.as_str() {
        "mig-tree" => {
            let service = state.conversion_service(&req.format_version)?;
            let tree = service.convert_to_tree(&req.input)
                .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
            Ok(Json(serde_json::json!({ "mode": "mig-tree", "tree": tree })))
        }
        "bo4e" => {
            let service = state.conversion_service(&req.format_version)?;
            let tree = service.convert_to_tree(&req.input)
                .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
            let bo4e = state.mapping_engine.to_bo4e(&tree)
                .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
            Ok(Json(serde_json::json!({ "mode": "bo4e", "result": bo4e })))
        }
        "legacy" => {
            // Use existing automapper-core pipeline
            // ... existing code path ...
            todo!("Wire to existing pipeline")
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-api test_convert_endpoint -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-api/
git commit -m "feat(api): add dual API v2 endpoint supporting mig-tree and bo4e modes"
```

---

## Task 3: Migration Comparison Tests

**Files:**
- Create: `crates/mig-bo4e/tests/migration_comparison_test.rs`

**Step 1: Write the comparison test**

This test runs both the old (`automapper-core`) and new (`mig-assembly` + `mig-bo4e`) pipelines on every fixture file and compares the BO4E output.

```rust
use std::path::Path;

#[test]
fn test_old_vs_new_pipeline_equivalence() {
    let fixture_dir = Path::new("../../example_market_communication_bo4e_transactions/UTILMD");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut equivalent = 0;
    let mut different = 0;
    let mut total = 0;

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.extension().map(|e| e == "txt").unwrap_or(false) {
            continue;
        }
        total += 1;

        let content = std::fs::read_to_string(&path).unwrap();

        // Old pipeline (automapper-core)
        let old_result = run_legacy_pipeline(&content);
        // New pipeline (mig-assembly + mig-bo4e)
        let new_result = run_new_pipeline(&content);

        match (old_result, new_result) {
            (Ok(old_bo4e), Ok(new_bo4e)) => {
                if old_bo4e == new_bo4e {
                    equivalent += 1;
                } else {
                    different += 1;
                    let name = path.file_name().unwrap().to_string_lossy();
                    eprintln!("DIFF in {name}: old and new BO4E output differ");
                }
            }
            (Err(e), _) => {
                eprintln!("Old pipeline failed for {:?}: {e}", path.file_name().unwrap());
            }
            (_, Err(e)) => {
                eprintln!("New pipeline failed for {:?}: {e}", path.file_name().unwrap());
            }
        }
    }

    eprintln!("\nEquivalence: {equivalent}/{total} identical, {different} different");

    // Start with a loose threshold — tighten as mappings improve
    assert!(equivalent > 0, "At least some files should produce equivalent output");
}

fn run_legacy_pipeline(input: &str) -> Result<serde_json::Value, String> {
    // Wire to automapper-core coordinator → writers pipeline
    // Serialize output as JSON for comparison
    todo!("Wire to legacy pipeline")
}

fn run_new_pipeline(input: &str) -> Result<serde_json::Value, String> {
    // Wire to mig-assembly → mig-bo4e pipeline
    // Serialize output as JSON for comparison
    todo!("Wire to new pipeline")
}
```

**Step 2: Implement the wiring functions**

Connect both pipelines and run the comparison.

**Step 3: Run test**

Run: `cargo test -p mig-bo4e test_old_vs_new_pipeline_equivalence -- --nocapture`
Expected: PASS with threshold check. Differences identify mapping gaps to close.

**Step 4: Commit**

```bash
git add crates/mig-bo4e/tests/
git commit -m "test(mig-bo4e): add migration comparison tests between old and new pipelines"
```

---

## Task 4: Performance Benchmarks

**Files:**
- Create: `crates/mig-assembly/benches/assembly_throughput.rs`

**Step 1: Create benchmark**

Create `crates/mig-assembly/benches/assembly_throughput.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mig_assembly::assembler::Assembler;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

fn bench_assembly(c: &mut Criterion) {
    let mig = parse_mig(
        Path::new("../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"),
        "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();

    let fixture_dir = Path::new("../../example_market_communication_bo4e_transactions/UTILMD");
    if !fixture_dir.exists() {
        return;
    }

    // Collect a few fixture files for benchmarking
    let fixtures: Vec<(String, String)> = std::fs::read_dir(fixture_dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "txt").unwrap_or(false))
        .take(5)
        .map(|e| {
            let name = e.path().file_stem().unwrap().to_string_lossy().to_string();
            let content = std::fs::read_to_string(e.path()).unwrap();
            (name, content)
        })
        .collect();

    let mut group = c.benchmark_group("assembly");
    for (name, content) in &fixtures {
        let segments = edifact_parser::parse_to_segments(content);
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &segments,
            |b, segments| {
                let assembler = Assembler::new(&mig);
                b.iter(|| assembler.assemble_generic(segments).unwrap());
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_assembly);
criterion_main!(benches);
```

Add to `crates/mig-assembly/Cargo.toml`:

```toml
[[bench]]
name = "assembly_throughput"
harness = false

[dev-dependencies]
criterion.workspace = true
```

**Step 2: Run benchmarks**

Run: `cargo bench -p mig-assembly`
Expected: Benchmark results showing assembly throughput.

**Step 3: Commit**

```bash
git add crates/mig-assembly/benches/ crates/mig-assembly/Cargo.toml
git commit -m "bench(mig-assembly): add assembly throughput benchmarks"
```

---

## Task 5: Update CLAUDE.md and Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `docs/plans/README.md`

**Step 1: Update CLAUDE.md**

Add a section about the new crates and dual API to `CLAUDE.md`:

- Add `mig-types`, `mig-assembly`, `mig-bo4e` to the workspace structure
- Document the MIG-driven architecture as an alternative to automapper-core
- Add commands for the new crates
- Reference the design document

**Step 2: Update plans README**

Add Feature 6 (mig-driven-mapping) to the feature table in `docs/plans/README.md`.

**Step 3: Commit**

```bash
git add CLAUDE.md docs/plans/README.md
git commit -m "docs: update CLAUDE.md and plans README for MIG-driven mapping architecture"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 11 new (3 service + 5 API v2 + 3 migration comparison) |
| Passed | 11 |
| Failed | 0 |
| Skipped | 0 |
| Workspace total | 717 |
| API modes | 3 (mig-tree, bo4e, legacy) |
| Migration comparison | 108/108 fixtures: both pipelines succeed |
| cargo check --workspace | PASS |
| cargo clippy --workspace | PASS |
| cargo fmt --check | PASS |

Files tested:
- crates/mig-assembly/tests/service_test.rs (3 tests)
- crates/automapper-api/tests/convert_v2_test.rs (5 tests)
- crates/mig-bo4e/tests/migration_comparison_test.rs (3 tests)
- crates/mig-assembly/benches/assembly_throughput.rs (benchmarks compile, 3 groups)

## Migration Completion Criteria

The migration from `automapper-core` to the MIG-driven pipeline is complete when:

1. **Roundtrip fidelity** — `mig-assembly` roundtrip rate matches or exceeds `automapper-core` (currently 99.4%)
2. **BO4E equivalence** — `mig-bo4e` output matches `automapper-core` output for >95% of fixtures
3. **Performance** — `mig-assembly` throughput is within 2x of `automapper-core` (acceptable trade-off for structural correctness)
4. **API stability** — dual API endpoints work correctly for all three modes
5. **No regressions** — all existing `automapper-core` tests still pass
