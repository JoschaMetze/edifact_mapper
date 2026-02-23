---
feature: reverse-endpoint
epic: 2
title: "Envelope Reconstruction"
depends_on: []
estimated_tasks: 3
crate: mig-bo4e
status: in_progress
---

# Epic 2: Envelope Reconstruction

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add helper functions to reconstruct EDIFACT envelope segments (UNB, UNH, UNT, UNZ) from BO4E JSON data. These are the inverse of `extract_nachrichtendaten()` and `extract_unh_fields()` — they take structured data and produce `OwnedSegment`s.

**Architecture:** Four functions in `mig-bo4e::model`:
- `rebuild_unb()` — from `nachrichtendaten` JSON → UNB `OwnedSegment`
- `rebuild_unh()` — from referenz + type → UNH `OwnedSegment`
- `rebuild_unt()` — from segment count + referenz → UNT `OwnedSegment`
- `rebuild_unz()` — from message count + interchange ref → UNZ `OwnedSegment`

For `transaktion`-level input (no envelope in input), defaults are provided: UNB with UNOC:3, placeholder sender/receiver (overridable), current datetime; UNH with sequential ref and UTILMD type (overridable).

**Existing code:**
- `extract_nachrichtendaten()` at `crates/mig-bo4e/src/model.rs:79` — inverse of `rebuild_unb()`
- `extract_unh_fields()` at `crates/mig-bo4e/src/model.rs:72` — inverse of `rebuild_unh()`
- `OwnedSegment` from `mig_types::segment` — the target type for all builders

---

## Task 1: Envelope Builders — Unit Tests

**Files:**
- Modify: `crates/mig-bo4e/src/model.rs` (add tests)

**Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `crates/mig-bo4e/src/model.rs` (after line 240):

```rust
    #[test]
    fn test_rebuild_unb_from_nachrichtendaten() {
        let nd = serde_json::json!({
            "syntaxKennung": "UNOC",
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "datum": "210101",
            "zeit": "1200",
            "interchangeRef": "REF001"
        });

        let unb = rebuild_unb(&nd);
        assert_eq!(unb.id, "UNB");
        // UNB+UNOC:3+sender:500+receiver:500+date:time+ref
        assert_eq!(unb.elements[0], vec!["UNOC", "3"]);
        assert_eq!(unb.elements[1][0], "9900123456789");
        assert_eq!(unb.elements[2][0], "9900987654321");
        assert_eq!(unb.elements[3], vec!["210101", "1200"]);
        assert_eq!(unb.elements[4], vec!["REF001"]);
    }

    #[test]
    fn test_rebuild_unb_defaults() {
        // Empty nachrichtendaten — should produce valid UNB with placeholders
        let nd = serde_json::json!({});
        let unb = rebuild_unb(&nd);
        assert_eq!(unb.id, "UNB");
        assert_eq!(unb.elements[0], vec!["UNOC", "3"]);
    }

    #[test]
    fn test_rebuild_unh() {
        let unh = rebuild_unh("00001", "UTILMD");
        assert_eq!(unh.id, "UNH");
        assert_eq!(unh.elements[0], vec!["00001"]);
        assert_eq!(unh.elements[1][0], "UTILMD");
        assert_eq!(unh.elements[1][1], "D");
        assert_eq!(unh.elements[1][2], "11A");
        assert_eq!(unh.elements[1][3], "UN");
        assert_eq!(unh.elements[1][4], "S2.1");
    }

    #[test]
    fn test_rebuild_unt() {
        let unt = rebuild_unt(25, "00001");
        assert_eq!(unt.id, "UNT");
        assert_eq!(unt.elements[0], vec!["25"]);
        assert_eq!(unt.elements[1], vec!["00001"]);
    }

    #[test]
    fn test_rebuild_unz() {
        let unz = rebuild_unz(1, "REF001");
        assert_eq!(unz.id, "UNZ");
        assert_eq!(unz.elements[0], vec!["1"]);
        assert_eq!(unz.elements[1], vec!["REF001"]);
    }

    #[test]
    fn test_roundtrip_nachrichtendaten_rebuild() {
        // extract_nachrichtendaten() → rebuild_unb() should preserve fields
        let original = OwnedSegment {
            id: "UNB".to_string(),
            elements: vec![
                vec!["UNOC".to_string(), "3".to_string()],
                vec!["9900123456789".to_string(), "500".to_string()],
                vec!["9900987654321".to_string(), "500".to_string()],
                vec!["210101".to_string(), "1200".to_string()],
                vec!["REF001".to_string()],
            ],
            segment_number: 0,
        };

        let nd = extract_nachrichtendaten(&[original]);
        let rebuilt = rebuild_unb(&nd);
        assert_eq!(rebuilt.elements[0], vec!["UNOC", "3"]);
        assert_eq!(rebuilt.elements[1][0], "9900123456789");
        assert_eq!(rebuilt.elements[2][0], "9900987654321");
        assert_eq!(rebuilt.elements[3], vec!["210101", "1200"]);
        assert_eq!(rebuilt.elements[4], vec!["REF001"]);
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p mig-bo4e -- model::tests::test_rebuild`
Expected: FAIL — `rebuild_unb`, `rebuild_unh`, etc. not found

---

## Task 2: Implement Envelope Builders

**Files:**
- Modify: `crates/mig-bo4e/src/model.rs`

**Step 1: Add the builder functions**

Add before the `#[cfg(test)]` block in `crates/mig-bo4e/src/model.rs`:

```rust
/// Rebuild a UNB (interchange header) segment from nachrichtendaten JSON.
///
/// This is the inverse of `extract_nachrichtendaten()`.
/// Fields not present in the JSON get sensible defaults (UNOC:3, "500" qualifier).
pub fn rebuild_unb(nachrichtendaten: &serde_json::Value) -> OwnedSegment {
    let syntax = nachrichtendaten
        .get("syntaxKennung")
        .and_then(|v| v.as_str())
        .unwrap_or("UNOC");
    let sender = nachrichtendaten
        .get("absenderCode")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let receiver = nachrichtendaten
        .get("empfaengerCode")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let datum = nachrichtendaten
        .get("datum")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let zeit = nachrichtendaten
        .get("zeit")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let interchange_ref = nachrichtendaten
        .get("interchangeRef")
        .and_then(|v| v.as_str())
        .unwrap_or("00000");

    OwnedSegment {
        id: "UNB".to_string(),
        elements: vec![
            vec![syntax.to_string(), "3".to_string()],
            vec![sender.to_string(), "500".to_string()],
            vec![receiver.to_string(), "500".to_string()],
            vec![datum.to_string(), zeit.to_string()],
            vec![interchange_ref.to_string()],
        ],
        segment_number: 0,
    }
}

/// Rebuild a UNH (message header) segment from reference number and message type.
///
/// Produces: `UNH+referenz+typ:D:11A:UN:S2.1`
pub fn rebuild_unh(referenz: &str, nachrichten_typ: &str) -> OwnedSegment {
    OwnedSegment {
        id: "UNH".to_string(),
        elements: vec![
            vec![referenz.to_string()],
            vec![
                nachrichten_typ.to_string(),
                "D".to_string(),
                "11A".to_string(),
                "UN".to_string(),
                "S2.1".to_string(),
            ],
        ],
        segment_number: 0,
    }
}

/// Rebuild a UNT (message trailer) segment.
///
/// Produces: `UNT+count+referenz`
/// `segment_count` includes UNH and UNT themselves.
pub fn rebuild_unt(segment_count: usize, referenz: &str) -> OwnedSegment {
    OwnedSegment {
        id: "UNT".to_string(),
        elements: vec![
            vec![segment_count.to_string()],
            vec![referenz.to_string()],
        ],
        segment_number: 0,
    }
}

/// Rebuild a UNZ (interchange trailer) segment.
///
/// Produces: `UNZ+count+ref`
pub fn rebuild_unz(message_count: usize, interchange_ref: &str) -> OwnedSegment {
    OwnedSegment {
        id: "UNZ".to_string(),
        elements: vec![
            vec![message_count.to_string()],
            vec![interchange_ref.to_string()],
        ],
        segment_number: 0,
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test -p mig-bo4e -- model::tests::test_rebuild`
Expected: ALL PASS

Run: `cargo test -p mig-bo4e -- model::tests::test_roundtrip_nachrichtendaten`
Expected: PASS

**Step 3: Run full test suite**

Run: `cargo test -p mig-bo4e`
Expected: ALL PASS

**Step 4: Commit**

```bash
git add crates/mig-bo4e/src/model.rs
git commit -m "feat(mig-bo4e): add envelope reconstruction helpers (UNB/UNH/UNT/UNZ)"
```

---

## Task 3: Export Builders from `mig-bo4e`

**Files:**
- Modify: `crates/mig-bo4e/src/lib.rs`

**Step 1: Verify exports**

Check that `rebuild_unb`, `rebuild_unh`, `rebuild_unt`, `rebuild_unz` are accessible from `mig_bo4e::model`. The functions are `pub` in `model.rs` which is `pub mod model` in `lib.rs`, so they should be accessible as `mig_bo4e::model::rebuild_unb()` etc.

If `model` is not already `pub mod`, update `lib.rs` to export it. Check current exports:

```bash
grep 'pub mod' crates/mig-bo4e/src/lib.rs
```

**Step 2: Add re-exports if needed**

If the module is already public (it should be from Epic 2 of multi-message), no changes are needed. If not, add `pub mod model;` to `lib.rs`.

**Step 3: Verify with workspace check**

Run: `cargo check --workspace`
Expected: OK

**Step 4: Commit (only if changes were made)**

```bash
git add crates/mig-bo4e/src/lib.rs
git commit -m "chore(mig-bo4e): ensure model envelope builders are exported"
```
