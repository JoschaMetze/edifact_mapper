---
feature: multi-message-transaction
epic: 2
title: "Interchange Data Model"
depends_on: []
estimated_tasks: 3
crate: mig-bo4e
status: in_progress
---

# Epic 2: Interchange Data Model

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Define the typed `Interchange`, `Nachricht`, and `Transaktion` structs in `mig-bo4e` that represent the three-level EDIFACT hierarchy. These are the output types of the MIG-driven mapping pipeline when processing complete interchanges.

**Architecture:** Generic-free structs using `serde_json::Value` for the BO4E payload (since the mapping engine already produces JSON). The types live in a new `model` module in `mig-bo4e`. They derive `Serialize`/`Deserialize` for API responses.

**Tech Stack:** Rust, serde, serde_json

---

## Task 1: Create Transaktion Struct

**Files:**
- Create: `crates/mig-bo4e/src/model.rs`
- Modify: `crates/mig-bo4e/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/mig-bo4e/src/model.rs` with:

```rust
//! Output model types for the MIG-driven mapping pipeline.
//!
//! Three-level hierarchy: `Interchange` → `Nachricht` → `Transaktion`
//! matching the EDIFACT structure: UNB/UNZ → UNH/UNT → IDE/SG4.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaktion_serde_roundtrip() {
        let tx = Transaktion {
            stammdaten: serde_json::json!({
                "Marktlokation": { "marktlokationsId": "DE000111222333" }
            }),
            transaktionsdaten: serde_json::json!({
                "vorgangId": "TX001",
                "transaktionsgrund": "E01"
            }),
        };

        let json = serde_json::to_string(&tx).unwrap();
        let de: Transaktion = serde_json::from_str(&json).unwrap();
        assert_eq!(
            de.transaktionsdaten["vorgangId"].as_str().unwrap(),
            "TX001"
        );
        assert!(de.stammdaten["Marktlokation"].is_object());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_transaktion_serde_roundtrip`
Expected: FAIL — `Transaktion` not found

**Step 3: Write implementation**

Add above the `#[cfg(test)]` block:

```rust
use serde::{Deserialize, Serialize};

/// A single transaction within an EDIFACT message.
///
/// In UTILMD, each SG4 group (starting with IDE) is one transaction.
/// Contains the mapped BO4E entities (stammdaten) and process metadata
/// (transaktionsdaten) extracted from the transaction group's root segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaktion {
    /// BO4E entities mapped from this transaction's segment groups.
    /// Keys are entity names (e.g., "Marktlokation", "Messlokation").
    pub stammdaten: serde_json::Value,

    /// Process metadata from the transaction group's root segments
    /// (IDE, STS, DTM in UTILMD). Not mapped to BO4E types.
    pub transaktionsdaten: serde_json::Value,
}
```

Also add the module to `crates/mig-bo4e/src/lib.rs`:

```rust
pub mod model;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_transaktion_serde_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/model.rs crates/mig-bo4e/src/lib.rs
git commit -m "feat(mig-bo4e): add Transaktion output model type"
```

---

## Task 2: Add Nachricht and Interchange Structs

**Files:**
- Modify: `crates/mig-bo4e/src/model.rs`

**Step 1: Write the failing tests**

Add to the test module:

```rust
#[test]
fn test_nachricht_serde_roundtrip() {
    let msg = Nachricht {
        unh_referenz: "00001".to_string(),
        nachrichten_typ: "UTILMD".to_string(),
        stammdaten: serde_json::json!({
            "Marktteilnehmer": [
                { "marktrolle": "MS", "rollencodenummer": "9900123" }
            ]
        }),
        transaktionen: vec![Transaktion {
            stammdaten: serde_json::json!({}),
            transaktionsdaten: serde_json::json!({}),
        }],
    };

    let json = serde_json::to_string(&msg).unwrap();
    let de: Nachricht = serde_json::from_str(&json).unwrap();
    assert_eq!(de.unh_referenz, "00001");
    assert_eq!(de.nachrichten_typ, "UTILMD");
    assert_eq!(de.transaktionen.len(), 1);
}

#[test]
fn test_interchange_serde_roundtrip() {
    let interchange = Interchange {
        nachrichtendaten: serde_json::json!({
            "absender": "9900123456789",
            "empfaenger": "9900987654321"
        }),
        nachrichten: vec![Nachricht {
            unh_referenz: "00001".to_string(),
            nachrichten_typ: "UTILMD".to_string(),
            stammdaten: serde_json::json!({}),
            transaktionen: vec![],
        }],
    };

    let json = serde_json::to_string_pretty(&interchange).unwrap();
    let de: Interchange = serde_json::from_str(&json).unwrap();
    assert_eq!(de.nachrichten.len(), 1);
    assert_eq!(de.nachrichten[0].unh_referenz, "00001");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p mig-bo4e test_nachricht_serde_roundtrip`
Expected: FAIL — `Nachricht` not found

**Step 3: Write implementation**

Add above `Transaktion`:

```rust
/// A complete EDIFACT interchange (UNB...UNZ) containing one or more messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interchange {
    /// Service segment data extracted from UNA/UNB/UNZ.
    /// Contains absender, empfaenger, interchange reference, etc.
    pub nachrichtendaten: serde_json::Value,

    /// One entry per UNH/UNT message pair in the interchange.
    pub nachrichten: Vec<Nachricht>,
}

/// A single EDIFACT message (UNH...UNT) within an interchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nachricht {
    /// UNH message reference number (first element of UNH segment).
    pub unh_referenz: String,

    /// Message type identifier from UNH (e.g., "UTILMD", "ORDERS").
    pub nachrichten_typ: String,

    /// Message-level BO4E entities (e.g., Marktteilnehmer from SG2).
    /// Mapped from definitions with `level = "message"` or from `message/` TOML directory.
    pub stammdaten: serde_json::Value,

    /// One entry per transaction group within this message
    /// (SG4 in UTILMD, each starting with IDE).
    pub transaktionen: Vec<Transaktion>,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p mig-bo4e test_nachricht_serde_roundtrip && cargo test -p mig-bo4e test_interchange_serde_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/model.rs
git commit -m "feat(mig-bo4e): add Nachricht and Interchange output model types"
```

---

## Task 3: Add Helper Constructors and Re-exports

**Files:**
- Modify: `crates/mig-bo4e/src/model.rs`
- Modify: `crates/mig-bo4e/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_extract_unh_fields() {
    use mig_types::segment::OwnedSegment;

    let unh = OwnedSegment {
        id: "UNH".to_string(),
        elements: vec![
            vec!["MSG001".to_string()],
            vec!["UTILMD".to_string(), "D".to_string(), "11A".to_string(), "UN".to_string(), "S2.1".to_string()],
        ],
        segment_number: 0,
    };

    let (referenz, typ) = extract_unh_fields(&unh);
    assert_eq!(referenz, "MSG001");
    assert_eq!(typ, "UTILMD");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_extract_unh_fields`
Expected: FAIL — `extract_unh_fields` not found

**Step 3: Write implementation**

Add to `model.rs`:

```rust
use mig_types::segment::OwnedSegment;

/// Extract message reference and message type from a UNH segment.
pub fn extract_unh_fields(unh: &OwnedSegment) -> (String, String) {
    let referenz = unh.get_element(0).to_string();
    let typ = unh.get_component(1, 0).to_string();
    (referenz, typ)
}
```

Add re-export to `crates/mig-bo4e/src/lib.rs`:

```rust
pub use model::{Interchange, Nachricht, Transaktion};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_extract_unh_fields`
Expected: PASS

Run: `cargo check --workspace`
Expected: OK

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/model.rs crates/mig-bo4e/src/lib.rs
git commit -m "feat(mig-bo4e): add UNH field extraction and re-export model types"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 67 |
| Passed | 66 |
| Failed | 0 |
| Skipped | 1 |

New tests added (4):
- `model::tests::test_transaktion_serde_roundtrip` — Transaktion serde roundtrip
- `model::tests::test_nachricht_serde_roundtrip` — Nachricht serde roundtrip
- `model::tests::test_interchange_serde_roundtrip` — Interchange serde roundtrip
- `model::tests::test_extract_unh_fields` — UNH field extraction helper

Files changed:
- `crates/mig-bo4e/src/model.rs` (new)
- `crates/mig-bo4e/src/lib.rs` (modified)
