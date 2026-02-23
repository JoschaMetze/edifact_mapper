---
feature: reverse-endpoint
epic: 3
title: "API Contracts & Handler"
depends_on: [reverse-endpoint/E01, reverse-endpoint/E02]
estimated_tasks: 5
crate: automapper-api
status: in_progress
---

# Epic 3: API Contracts & Handler

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add the `POST /api/v2/reverse` endpoint with request/response contracts, a handler that orchestrates the full reverse pipeline (BO4E JSON → reverse mapping → disassembly → rendering → EDIFACT string), and route registration.

**Architecture:** The handler accepts `ReverseRequest` with an `input` (BO4E JSON at any level), `level` (interchange/nachricht/transaktion), `format_version`, `mode` (edifact/mig-tree), and optional `envelope` overrides. It normalizes the input to an `Interchange`, runs `map_interchange_reverse()` per message to get `AssembledTree`s, then disassembles and renders each into EDIFACT. For `mig-tree` mode, it returns the assembled tree JSON instead.

**Existing code:**
- `convert_v2` handler at `crates/automapper-api/src/routes/convert_v2.rs` — pattern for the reverse handler
- `MigServiceRegistry` at `crates/automapper-api/src/state.rs` — engine/service lookup
- `Disassembler::disassemble()` at `crates/mig-assembly/src/disassembler.rs` — tree → ordered segments
- `render_edifact()` at `crates/mig-assembly/src/renderer.rs` — segments → EDIFACT string
- Route registration at `crates/automapper-api/src/routes/mod.rs:22`

---

## Task 1: Add Reverse API Contracts

**Files:**
- Create: `crates/automapper-api/src/contracts/reverse_v2.rs`
- Modify: `crates/automapper-api/src/contracts/mod.rs`

**Step 1: Create contract types**

Create `crates/automapper-api/src/contracts/reverse_v2.rs`:

```rust
//! V2 reverse conversion request/response types.
//!
//! Accepts BO4E JSON and converts back to EDIFACT.

use serde::{Deserialize, Serialize};

/// Input level for the reverse endpoint.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InputLevel {
    /// Full interchange JSON (nachrichtendaten + nachrichten array).
    Interchange,
    /// Single message JSON (unhReferenz, nachrichtenTyp, stammdaten, transaktionen).
    Nachricht,
    /// Single transaction JSON (stammdaten, transaktionsdaten).
    Transaktion,
}

/// Output mode for the reverse endpoint.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReverseMode {
    /// Return EDIFACT string.
    Edifact,
    /// Return the assembled MIG tree as JSON (debugging).
    MigTree,
}

/// Optional envelope overrides for missing levels.
///
/// When input is `nachricht` or `transaktion`, these values fill in
/// the envelope segments that aren't present in the input.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeOverrides {
    pub absender_code: Option<String>,
    pub empfaenger_code: Option<String>,
    pub nachrichten_typ: Option<String>,
}

/// Request body for `POST /api/v2/reverse`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseV2Request {
    /// The BO4E JSON to convert back to EDIFACT.
    /// Shape depends on `level`.
    pub input: serde_json::Value,

    /// Which level the input represents.
    pub level: InputLevel,

    /// Format version (e.g., "FV2504").
    pub format_version: String,

    /// Output mode: "edifact" or "mig-tree".
    #[serde(default = "default_mode")]
    pub mode: ReverseMode,

    /// Optional envelope overrides for missing levels.
    #[serde(default)]
    pub envelope: Option<EnvelopeOverrides>,
}

fn default_mode() -> ReverseMode {
    ReverseMode::Edifact
}

/// Response body for `POST /api/v2/reverse`.
#[derive(Debug, Clone, Serialize)]
pub struct ReverseV2Response {
    /// The mode used for conversion.
    pub mode: String,

    /// The result: EDIFACT string or MIG tree JSON.
    pub result: serde_json::Value,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,
}
```

**Step 2: Register the module**

Add to `crates/automapper-api/src/contracts/mod.rs`:

```rust
pub mod reverse_v2;
```

**Step 3: Verify compilation**

Run: `cargo check -p automapper-api`
Expected: OK

**Step 4: Commit**

```bash
git add crates/automapper-api/src/contracts/reverse_v2.rs crates/automapper-api/src/contracts/mod.rs
git commit -m "feat(api): add reverse endpoint contracts (ReverseV2Request, InputLevel, ReverseMode)"
```

---

## Task 2: Normalize Input to Interchange

**Files:**
- Modify: `crates/automapper-api/src/contracts/reverse_v2.rs`

**Step 1: Write the normalization tests**

Add a `#[cfg(test)] mod tests` block at the bottom of `reverse_v2.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_interchange_level() {
        let input = serde_json::json!({
            "nachrichtendaten": { "absenderCode": "9900123" },
            "nachrichten": [{
                "unhReferenz": "00001",
                "nachrichtenTyp": "UTILMD",
                "stammdaten": {},
                "transaktionen": []
            }]
        });

        let interchange: mig_bo4e::Interchange =
            normalize_to_interchange(&input, &InputLevel::Interchange, None).unwrap();
        assert_eq!(interchange.nachrichten.len(), 1);
        assert_eq!(interchange.nachrichten[0].unh_referenz, "00001");
    }

    #[test]
    fn test_normalize_nachricht_level() {
        let input = serde_json::json!({
            "unhReferenz": "00001",
            "nachrichtenTyp": "UTILMD",
            "stammdaten": { "Marktteilnehmer": [] },
            "transaktionen": [{ "stammdaten": {}, "transaktionsdaten": {} }]
        });

        let interchange =
            normalize_to_interchange(&input, &InputLevel::Nachricht, None).unwrap();
        assert_eq!(interchange.nachrichten.len(), 1);
        assert_eq!(interchange.nachrichten[0].unh_referenz, "00001");
    }

    #[test]
    fn test_normalize_transaktion_level() {
        let input = serde_json::json!({
            "stammdaten": { "Marktlokation": {} },
            "transaktionsdaten": { "pruefidentifikator": "55001" }
        });

        let overrides = EnvelopeOverrides {
            absender_code: Some("9900123".to_string()),
            empfaenger_code: Some("9900456".to_string()),
            nachrichten_typ: Some("UTILMD".to_string()),
        };

        let interchange =
            normalize_to_interchange(&input, &InputLevel::Transaktion, Some(&overrides)).unwrap();
        assert_eq!(interchange.nachrichten.len(), 1);
        assert_eq!(interchange.nachrichten[0].transaktionen.len(), 1);
        assert_eq!(interchange.nachrichten[0].nachrichten_typ, "UTILMD");
    }
}
```

**Step 2: Implement normalization function**

Add before the tests block in `reverse_v2.rs`:

```rust
/// Normalize input JSON to an `Interchange`, wrapping lower-level inputs as needed.
pub fn normalize_to_interchange(
    input: &serde_json::Value,
    level: &InputLevel,
    overrides: Option<&EnvelopeOverrides>,
) -> Result<mig_bo4e::Interchange, String> {
    match level {
        InputLevel::Interchange => {
            serde_json::from_value(input.clone())
                .map_err(|e| format!("Invalid interchange JSON: {e}"))
        }
        InputLevel::Nachricht => {
            let nachricht: mig_bo4e::Nachricht = serde_json::from_value(input.clone())
                .map_err(|e| format!("Invalid nachricht JSON: {e}"))?;

            let nachrichten_typ = overrides
                .and_then(|o| o.nachrichten_typ.clone())
                .unwrap_or_else(|| nachricht.nachrichten_typ.clone());

            // Build default nachrichtendaten
            let mut nd = serde_json::Map::new();
            nd.insert("syntaxKennung".to_string(), serde_json::json!("UNOC"));
            if let Some(ref o) = overrides {
                if let Some(ref s) = o.absender_code {
                    nd.insert("absenderCode".to_string(), serde_json::json!(s));
                }
                if let Some(ref r) = o.empfaenger_code {
                    nd.insert("empfaengerCode".to_string(), serde_json::json!(r));
                }
            }
            nd.insert("interchangeRef".to_string(), serde_json::json!("00000"));

            Ok(mig_bo4e::Interchange {
                nachrichtendaten: serde_json::Value::Object(nd),
                nachrichten: vec![mig_bo4e::Nachricht {
                    nachrichten_typ,
                    ..nachricht
                }],
            })
        }
        InputLevel::Transaktion => {
            let tx: mig_bo4e::Transaktion = serde_json::from_value(input.clone())
                .map_err(|e| format!("Invalid transaktion JSON: {e}"))?;

            let nachrichten_typ = overrides
                .and_then(|o| o.nachrichten_typ.clone())
                .unwrap_or_else(|| "UTILMD".to_string());

            let mut nd = serde_json::Map::new();
            nd.insert("syntaxKennung".to_string(), serde_json::json!("UNOC"));
            if let Some(ref o) = overrides {
                if let Some(ref s) = o.absender_code {
                    nd.insert("absenderCode".to_string(), serde_json::json!(s));
                }
                if let Some(ref r) = o.empfaenger_code {
                    nd.insert("empfaengerCode".to_string(), serde_json::json!(r));
                }
            }
            nd.insert("interchangeRef".to_string(), serde_json::json!("00000"));

            Ok(mig_bo4e::Interchange {
                nachrichtendaten: serde_json::Value::Object(nd),
                nachrichten: vec![mig_bo4e::Nachricht {
                    unh_referenz: "00001".to_string(),
                    nachrichten_typ,
                    stammdaten: serde_json::json!({}),
                    transaktionen: vec![tx],
                }],
            })
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p automapper-api -- contracts::reverse_v2::tests`
Expected: ALL PASS

**Step 4: Commit**

```bash
git add crates/automapper-api/src/contracts/reverse_v2.rs
git commit -m "feat(api): add normalize_to_interchange() for input level handling"
```

---

## Task 3: Implement Reverse Handler

**Files:**
- Create: `crates/automapper-api/src/routes/reverse_v2.rs`
- Modify: `crates/automapper-api/src/routes/mod.rs`

**Step 1: Create the handler**

Create `crates/automapper-api/src/routes/reverse_v2.rs`:

```rust
//! V2 reverse endpoint: BO4E → EDIFACT.
//!
//! Accepts BO4E JSON at interchange/nachricht/transaktion level
//! and converts back to an EDIFACT string or MIG tree.

use std::collections::HashSet;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;

use crate::contracts::reverse_v2::{
    normalize_to_interchange, ReverseMode, ReverseV2Request, ReverseV2Response,
};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 reverse routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/reverse", post(reverse_v2))
}

/// `POST /api/v2/reverse` — BO4E to EDIFACT reverse conversion.
async fn reverse_v2(
    State(state): State<AppState>,
    Json(req): Json<ReverseV2Request>,
) -> Result<Json<ReverseV2Response>, ApiError> {
    let start = std::time::Instant::now();

    // Step 1: Normalize input to Interchange
    let interchange =
        normalize_to_interchange(&req.input, &req.level, req.envelope.as_ref()).map_err(|e| {
            ApiError::BadRequest {
                message: format!("Input normalization error: {e}"),
            }
        })?;

    // Step 2: Get MIG service for the format version
    let service = state
        .mig_registry
        .service(&req.format_version)
        .ok_or_else(|| ApiError::BadRequest {
            message: format!(
                "No MIG service available for format version '{}'",
                req.format_version
            ),
        })?;

    // TODO: detect message type/variant from nachrichtenTyp
    let msg_variant = "UTILMD_Strom";

    // Step 3: Reconstruct envelope segments
    let unb = mig_bo4e::model::rebuild_unb(&interchange.nachrichtendaten);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let mut all_edifact_parts: Vec<String> = Vec::new();

    // UNA + UNB
    let una_str = format!("UNA{}", delimiters.to_una_string());
    let unb_segments = vec![mig_assembly::disassembler::DisassembledSegment {
        tag: unb.id.clone(),
        elements: unb.elements.clone(),
    }];
    let unb_str = render_edifact(&unb_segments, &delimiters);

    // Step 4: Process each message
    for nachricht in &interchange.nachrichten {
        // Extract PID from first transaction's transaktionsdaten
        let pid = nachricht
            .transaktionen
            .first()
            .and_then(|tx| tx.transaktionsdaten.get("pruefidentifikator"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::BadRequest {
                message: "No pruefidentifikator found in transaktionsdaten".to_string(),
            })?;

        // Look up AHB for PID → segment numbers → filtered MIG
        let ahb = state
            .mig_registry
            .ahb_schema(&req.format_version, msg_variant)
            .ok_or_else(|| ApiError::Internal {
                message: format!(
                    "No AHB schema available for {}/{}",
                    req.format_version, msg_variant
                ),
            })?;

        let workflow = ahb.workflows.iter().find(|w| w.id == pid).ok_or_else(|| {
            ApiError::ConversionError {
                message: format!("PID {pid} not found in AHB"),
            }
        })?;

        let ahb_numbers: HashSet<String> = workflow.segment_numbers.iter().cloned().collect();
        let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);

        // Load split engines
        let (msg_engine, tx_engine) = state
            .mig_registry
            .mapping_engines_split(&req.format_version, msg_variant, pid)
            .ok_or_else(|| ApiError::Internal {
                message: format!(
                    "No mapping engines for {}/{}/pid_{}",
                    req.format_version, msg_variant, pid
                ),
            })?;

        // Build MappedMessage from Nachricht
        let mapped = mig_bo4e::model::MappedMessage {
            stammdaten: nachricht.stammdaten.clone(),
            transaktionen: nachricht.transaktionen.clone(),
        };

        // Reverse map → AssembledTree
        let tree = mig_bo4e::MappingEngine::map_interchange_reverse(
            msg_engine, tx_engine, &mapped, "SG4",
        );

        match req.mode {
            ReverseMode::MigTree => {
                // Return tree JSON for this message
                return Ok(Json(ReverseV2Response {
                    mode: "mig-tree".to_string(),
                    result: serde_json::to_value(&tree).unwrap_or_default(),
                    duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                }));
            }
            ReverseMode::Edifact => {
                // Disassemble tree → ordered segments
                let disassembler = Disassembler::new(&filtered_mig);
                let dis_segments = disassembler.disassemble(&tree);

                // Build UNH + body + UNT
                let unh = mig_bo4e::model::rebuild_unh(
                    &nachricht.unh_referenz,
                    &nachricht.nachrichten_typ,
                );
                let unh_dis = mig_assembly::disassembler::DisassembledSegment {
                    tag: unh.id.clone(),
                    elements: unh.elements.clone(),
                };

                // Segment count = UNH + body segments + UNT
                let seg_count = 1 + dis_segments.len() + 1;
                let unt = mig_bo4e::model::rebuild_unt(seg_count, &nachricht.unh_referenz);
                let unt_dis = mig_assembly::disassembler::DisassembledSegment {
                    tag: unt.id.clone(),
                    elements: unt.elements.clone(),
                };

                let mut msg_segments = vec![unh_dis];
                msg_segments.extend(dis_segments);
                msg_segments.push(unt_dis);

                all_edifact_parts.push(render_edifact(&msg_segments, &delimiters));
            }
        }
    }

    // Step 5: Build UNZ and concatenate
    let interchange_ref = interchange
        .nachrichtendaten
        .get("interchangeRef")
        .and_then(|v| v.as_str())
        .unwrap_or("00000");
    let unz = mig_bo4e::model::rebuild_unz(interchange.nachrichten.len(), interchange_ref);
    let unz_segments = vec![mig_assembly::disassembler::DisassembledSegment {
        tag: unz.id.clone(),
        elements: unz.elements.clone(),
    }];

    let mut full_edifact = una_str;
    full_edifact.push_str(&unb_str);
    for part in &all_edifact_parts {
        full_edifact.push_str(part);
    }
    full_edifact.push_str(&render_edifact(&unz_segments, &delimiters));

    Ok(Json(ReverseV2Response {
        mode: "edifact".to_string(),
        result: serde_json::Value::String(full_edifact),
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}
```

**Step 2: Register the route module and add to v2 routes**

Update `crates/automapper-api/src/routes/mod.rs`:

Add `pub mod reverse_v2;` to the module declarations.

Update `api_v2_routes()`:

```rust
pub fn api_v2_routes() -> Router<AppState> {
    Router::new()
        .merge(convert_v2::routes())
        .merge(reverse_v2::routes())
}
```

**Step 3: Verify compilation**

Run: `cargo check -p automapper-api`
Expected: OK (may need import adjustments)

**Step 4: Commit**

```bash
git add crates/automapper-api/src/routes/reverse_v2.rs crates/automapper-api/src/routes/mod.rs
git commit -m "feat(api): add POST /api/v2/reverse handler for BO4E → EDIFACT"
```

---

## Task 4: Add Unit Tests for Handler

**Files:**
- Modify: `crates/automapper-api/src/routes/reverse_v2.rs` (or create test file)

**Step 1: Write handler test using axum test utilities**

Check the existing test pattern in `crates/automapper-api/tests/`. If there are integration tests, add a new test file `crates/automapper-api/tests/reverse_v2_test.rs`:

```rust
//! Integration tests for POST /api/v2/reverse.

// This test requires the full AppState with loaded MIG schemas and mapping engines.
// It mirrors the structure of existing convert_v2 integration tests.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// The test setup depends on how AppState is constructed in existing tests.
// Follow the same pattern as the convert_v2 tests.

#[tokio::test]
async fn test_reverse_transaktion_level_produces_edifact() {
    // Setup: construct AppState (follow existing test pattern)
    // ...

    let request_body = serde_json::json!({
        "input": {
            "stammdaten": {
                "Marktlokation": { "marktlokationsId": "51238696781" }
            },
            "transaktionsdaten": {
                "pruefidentifikator": "55001",
                "kategorie": "E01"
            }
        },
        "level": "transaktion",
        "formatVersion": "FV2504",
        "mode": "edifact",
        "envelope": {
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "nachrichtenTyp": "UTILMD"
        }
    });

    // Send request to /api/v2/reverse
    // Assert status 200
    // Assert response contains EDIFACT string with UNA, UNB, UNH, UNT, UNZ
    // Assert the EDIFACT contains "LOC" segment (from Marktlokation reverse mapping)
}

#[tokio::test]
async fn test_reverse_mig_tree_mode() {
    // Similar setup but with mode: "mig-tree"
    // Assert response.mode == "mig-tree"
    // Assert response.result has "segments" and "groups" keys
}
```

Note: The exact test setup depends on how existing API tests are structured. Check `crates/automapper-api/tests/` for the pattern. The test should mirror the existing `convert_v2` integration test structure.

**Step 2: Run tests**

Run: `cargo test -p automapper-api`
Expected: ALL PASS

**Step 3: Commit**

```bash
git add crates/automapper-api/tests/
git commit -m "test(api): add integration tests for POST /api/v2/reverse"
```

---

## Task 5: Wire Up Remaining Imports

**Files:**
- Modify: `crates/automapper-api/Cargo.toml` (if needed)
- Modify: `crates/automapper-api/src/routes/reverse_v2.rs` (if needed)

**Step 1: Verify all dependencies**

The reverse handler uses:
- `mig_bo4e::model::rebuild_unb`, `rebuild_unh`, `rebuild_unt`, `rebuild_unz`
- `mig_bo4e::MappingEngine::map_interchange_reverse`
- `mig_assembly::disassembler::Disassembler`
- `mig_assembly::renderer::render_edifact`
- `edifact_types::EdifactDelimiters`

Check that `edifact-types` is in `automapper-api`'s `Cargo.toml` dependencies. If not, add it.

**Step 2: Run workspace checks**

Run: `cargo check --workspace`
Expected: OK

Run: `cargo clippy --workspace -- -D warnings`
Expected: OK (fix any warnings)

Run: `cargo fmt --all -- --check`
Expected: OK (fix any formatting)

**Step 3: Commit if needed**

```bash
git add crates/automapper-api/
git commit -m "fix(api): wire up reverse endpoint dependencies"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 15 |
| Passed | 15 |
| Failed | 0 |
| Skipped | 0 |

New tests added:
- `crates/automapper-api/src/contracts/reverse_v2.rs` — 8 unit tests (normalization, deserialization)
- `crates/automapper-api/tests/reverse_v2_test.rs` — 7 integration tests (contract validation, handler modes, roundtrip)
