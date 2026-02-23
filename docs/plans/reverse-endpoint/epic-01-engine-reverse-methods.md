---
feature: reverse-endpoint
epic: 1
title: "Engine Reverse Methods"
depends_on: []
estimated_tasks: 4
crate: mig-bo4e
status: complete
---

# Epic 1: Engine Reverse Methods

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add `map_all_reverse()` to `MappingEngine` that reverses all definitions in a single engine (producing an `AssembledTree`), and `map_interchange_reverse()` that orchestrates the two-pass per-transaction reverse mapping mirroring the forward `map_interchange()`.

**Architecture:** `map_all_reverse()` iterates over definitions, calling the existing `map_reverse()` per definition, then places results into an `AssembledTree` by their `source_group`. `map_interchange_reverse()` is a static method that takes message and transaction engines plus a `MappedMessage`, runs message-level reverse on stammdaten, then for each transaction does two passes (transaktionsdaten → root+SG6, stammdaten → SG5/SG8/SG10/SG12), and merges results into `AssembledGroupInstance` entries within an SG4 `AssembledGroup`.

**Existing code:**
- `MappingEngine::map_reverse()` at `crates/mig-bo4e/src/engine.rs:345` — reverses a single definition
- `MappingEngine::map_all_forward()` at `crates/mig-bo4e/src/engine.rs:729` — pattern for iterating definitions
- `MappingEngine::map_interchange()` at `crates/mig-bo4e/src/engine.rs:817` — forward two-engine pattern to mirror
- `MappingEngine::build_group_from_bo4e()` at `crates/mig-bo4e/src/engine.rs:878` — builds group from BO4E value

---

## Task 1: Add `map_all_reverse()` — Unit Test

**Files:**
- Create: `crates/mig-bo4e/tests/reverse_all_test.rs`

**Step 1: Write the failing test**

```rust
//! Tests for map_all_reverse() — reversing all definitions in an engine.

use mig_bo4e::MappingEngine;
use std::path::PathBuf;

fn mappings_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mappings/FV2504/UTILMD_Strom")
}

#[test]
fn test_map_all_reverse_message_level() {
    let msg_dir = mappings_dir().join("message");
    let msg_engine = MappingEngine::load(&msg_dir).unwrap();

    // Construct a minimal message-level BO4E JSON (Marktteilnehmer in SG2)
    let bo4e = serde_json::json!({
        "Marktteilnehmer": [
            {
                "marktrolle": "MS",
                "rollencodenummer": "9900123456789"
            },
            {
                "marktrolle": "MR",
                "rollencodenummer": "9900987654321"
            }
        ]
    });

    let tree = msg_engine.map_all_reverse(&bo4e);

    // Should produce an AssembledTree with SG2 group containing 2 repetitions
    let sg2 = tree.groups.iter().find(|g| g.group_id == "SG2");
    assert!(sg2.is_some(), "Should have SG2 group");
    let sg2 = sg2.unwrap();
    assert_eq!(sg2.repetitions.len(), 2, "Should have 2 Marktteilnehmer reps");

    // First rep should have NAD segment with MS qualifier
    let rep0 = &sg2.repetitions[0];
    let nad = rep0.segments.iter().find(|s| s.tag == "NAD");
    assert!(nad.is_some(), "First SG2 rep should have NAD");
}

#[test]
fn test_map_all_reverse_transaction_level() {
    let tx_dir = mappings_dir().join("pid_55001");
    let tx_engine = MappingEngine::load(&tx_dir).unwrap();

    // Minimal transaction-level BO4E (Marktlokation in SG5)
    let bo4e = serde_json::json!({
        "Marktlokation": {
            "marktlokationsId": "51238696781"
        }
    });

    let tree = tx_engine.map_all_reverse(&bo4e);

    // Should produce groups including SG5
    let has_groups = !tree.groups.is_empty();
    assert!(has_groups, "Should produce at least one group from reverse mapping");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e --test reverse_all_test`
Expected: FAIL — `map_all_reverse` not found

---

## Task 2: Implement `map_all_reverse()`

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`

**Step 1: Implement `map_all_reverse()`**

Add after `map_all_forward()` (around line 800), before `map_interchange()`:

```rust
    /// Reverse-map a BO4E entity map back to an AssembledTree.
    ///
    /// For each definition:
    /// 1. Look up entity in input by `meta.entity` name
    /// 2. If entity value is an array, map each element as a separate group repetition
    /// 3. Place results by `source_group`: `""` → root segments, `"SGn"` → groups
    ///
    /// This is the inverse of `map_all_forward()`.
    pub fn map_all_reverse(&self, entities: &serde_json::Value) -> AssembledTree {
        let mut root_segments: Vec<AssembledSegment> = Vec::new();
        let mut groups: Vec<AssembledGroup> = Vec::new();

        for def in &self.definitions {
            let entity = &def.meta.entity;

            // Look up entity value: try direct key, also try under companion_type
            let entity_value = entities.get(entity);

            if entity_value.is_none() {
                continue;
            }
            let entity_value = entity_value.unwrap();

            // Determine target group from source_group
            let leaf_group = def
                .meta
                .source_group
                .rsplit('.')
                .next()
                .unwrap_or(&def.meta.source_group);

            if def.meta.source_group.is_empty() {
                // Root-level: reverse into root segments
                let instance = self.map_reverse(entity_value, def);
                root_segments.extend(instance.segments);
            } else if entity_value.is_array() {
                // Array entity: each element becomes a group repetition
                let arr = entity_value.as_array().unwrap();
                let mut reps = Vec::new();
                for item in arr {
                    reps.push(self.map_reverse(item, def));
                }

                // Merge into existing group or create new one
                if let Some(existing) = groups.iter_mut().find(|g| g.group_id == leaf_group) {
                    existing.repetitions.extend(reps);
                } else {
                    groups.push(AssembledGroup {
                        group_id: leaf_group.to_string(),
                        repetitions: reps,
                    });
                }
            } else {
                // Single object: one repetition
                let instance = self.map_reverse(entity_value, def);

                if let Some(existing) = groups.iter_mut().find(|g| g.group_id == leaf_group) {
                    // Merge segments into last repetition's child groups or add new rep
                    existing.repetitions.push(instance);
                } else {
                    groups.push(AssembledGroup {
                        group_id: leaf_group.to_string(),
                        repetitions: vec![instance],
                    });
                }
            }
        }

        AssembledTree {
            segments: root_segments,
            groups,
            post_group_start: 0,
        }
    }
```

Note: This is a starting implementation. The actual merging logic for definitions that share the same entity and source_group will need refinement during testing — definitions like `marktlokation.toml` (SG5) and `marktlokation_info.toml` (SG8 child of SG5) contribute to the same entity but different groups. The implementation should handle:
- Definitions with the same `(entity, source_group)`: merge segments into the same repetition
- Definitions with different `source_group` but same `entity`: place segments in correct nested groups
- Discriminated definitions: only one matches per repetition

**Step 2: Run test to verify it passes**

Run: `cargo test -p mig-bo4e --test reverse_all_test`
Expected: PASS (both tests)

**Step 3: Run full test suite**

Run: `cargo test -p mig-bo4e`
Expected: ALL PASS (no regressions)

**Step 4: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/reverse_all_test.rs
git commit -m "feat(mig-bo4e): add map_all_reverse() for reversing all definitions"
```

---

## Task 3: Add `map_interchange_reverse()` — Unit Test

**Files:**
- Create: `crates/mig-bo4e/tests/interchange_reverse_test.rs`

**Step 1: Write the failing test**

```rust
//! Tests for map_interchange_reverse() — two-pass reverse mapping mirroring forward.

use mig_bo4e::model::{MappedMessage, Transaktion};
use mig_bo4e::MappingEngine;
use std::path::PathBuf;

fn mappings_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mappings/FV2504/UTILMD_Strom")
}

#[test]
fn test_map_interchange_reverse_single_transaction() {
    let msg_dir = mappings_dir().join("message");
    let tx_dir = mappings_dir().join("pid_55001");

    let msg_engine = MappingEngine::load(&msg_dir).unwrap();
    let tx_engine = MappingEngine::load(&tx_dir).unwrap();

    // Build a MappedMessage that mirrors the forward output
    let mapped = MappedMessage {
        stammdaten: serde_json::json!({
            "Marktteilnehmer": [
                { "marktrolle": "MS", "rollencodenummer": "9900123456789" }
            ]
        }),
        transaktionen: vec![Transaktion {
            stammdaten: serde_json::json!({
                "Marktlokation": { "marktlokationsId": "51238696781" }
            }),
            transaktionsdaten: serde_json::json!({
                "kategorie": "E01",
                "pruefidentifikator": "55001"
            }),
        }],
    };

    let tree = MappingEngine::map_interchange_reverse(
        &msg_engine,
        &tx_engine,
        &mapped,
        "SG4",
    );

    // Should have message-level groups (SG2) and transaction group (SG4)
    let sg2 = tree.groups.iter().find(|g| g.group_id == "SG2");
    assert!(sg2.is_some(), "Should have SG2 group from message stammdaten");

    let sg4 = tree.groups.iter().find(|g| g.group_id == "SG4");
    assert!(sg4.is_some(), "Should have SG4 group from transactions");

    let sg4 = sg4.unwrap();
    assert_eq!(sg4.repetitions.len(), 1, "Should have 1 transaction");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e --test interchange_reverse_test`
Expected: FAIL — `map_interchange_reverse` not found

---

## Task 4: Implement `map_interchange_reverse()`

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`

**Step 1: Implement `map_interchange_reverse()`**

Add after `map_interchange()` (around line 875):

```rust
    /// Reverse-map a `MappedMessage` back to an `AssembledTree`.
    ///
    /// Two-engine approach mirroring `map_interchange()`:
    /// - `msg_engine` handles message-level stammdaten → SG2/SG3 groups
    /// - `tx_engine` handles per-transaction stammdaten + transaktionsdaten → SG4 instances
    ///
    /// For each transaction, performs two passes:
    /// - **Pass 1**: Reverse-map transaktionsdaten (entities named "Prozessdaten" or "Nachricht")
    ///   → root segments (IDE, STS, DTM) + SG6 groups (RFF)
    /// - **Pass 2**: Reverse-map stammdaten (all other entities)
    ///   → SG5, SG8, SG10, SG12 groups
    ///
    /// Results are merged into one `AssembledGroupInstance` per transaction,
    /// collected into an SG4 `AssembledGroup`, then combined with message-level groups.
    pub fn map_interchange_reverse(
        msg_engine: &MappingEngine,
        tx_engine: &MappingEngine,
        mapped: &crate::model::MappedMessage,
        transaction_group: &str,
    ) -> AssembledTree {
        // Step 1: Reverse message-level stammdaten
        let msg_tree = msg_engine.map_all_reverse(&mapped.stammdaten);

        // Step 2: Build SG4 instances from transactions
        let mut sg4_reps: Vec<AssembledGroupInstance> = Vec::new();

        for tx in &mapped.transaktionen {
            // Pass 1: transaktionsdaten — filter to Prozessdaten/Nachricht definitions
            let tx_daten_tree = {
                let prozess_defs: Vec<&MappingDefinition> = tx_engine
                    .definitions
                    .iter()
                    .filter(|d| d.meta.entity == "Prozessdaten" || d.meta.entity == "Nachricht")
                    .collect();

                let mut root_segs = Vec::new();
                let mut child_groups = Vec::new();

                for def in &prozess_defs {
                    let instance = tx_engine.map_reverse(&tx.transaktionsdaten, def);

                    let leaf_group = def
                        .meta
                        .source_group
                        .rsplit('.')
                        .next()
                        .unwrap_or(&def.meta.source_group);

                    if leaf_group.is_empty() || def.meta.source_group.is_empty() {
                        root_segs.extend(instance.segments);
                    } else {
                        if let Some(existing) =
                            child_groups.iter_mut().find(|g: &&mut AssembledGroup| g.group_id == leaf_group)
                        {
                            existing.repetitions.push(instance);
                        } else {
                            child_groups.push(AssembledGroup {
                                group_id: leaf_group.to_string(),
                                repetitions: vec![instance],
                            });
                        }
                    }
                }

                (root_segs, child_groups)
            };

            // Pass 2: stammdaten — filter to non-Prozessdaten/Nachricht definitions
            let stamm_tree = {
                let stamm_defs: Vec<&MappingDefinition> = tx_engine
                    .definitions
                    .iter()
                    .filter(|d| d.meta.entity != "Prozessdaten" && d.meta.entity != "Nachricht")
                    .collect();

                let mut child_groups: Vec<AssembledGroup> = Vec::new();

                for def in &stamm_defs {
                    let entity = &def.meta.entity;
                    let entity_value = tx.stammdaten.get(entity);

                    if entity_value.is_none() {
                        continue;
                    }
                    let entity_value = entity_value.unwrap();

                    let leaf_group = def
                        .meta
                        .source_group
                        .rsplit('.')
                        .next()
                        .unwrap_or(&def.meta.source_group);

                    let instance = tx_engine.map_reverse(entity_value, def);

                    if let Some(existing) =
                        child_groups.iter_mut().find(|g| g.group_id == leaf_group)
                    {
                        existing.repetitions.push(instance);
                    } else {
                        child_groups.push(AssembledGroup {
                            group_id: leaf_group.to_string(),
                            repetitions: vec![instance],
                        });
                    }
                }

                child_groups
            };

            // Merge: root segments from pass 1, child groups from both passes
            let (root_segs, mut pass1_groups) = tx_daten_tree;
            pass1_groups.extend(stamm_tree);

            sg4_reps.push(AssembledGroupInstance {
                segments: root_segs,
                child_groups: pass1_groups,
            });
        }

        // Step 3: Combine message tree with SG4 group
        let mut all_groups = msg_tree.groups;
        if !sg4_reps.is_empty() {
            all_groups.push(AssembledGroup {
                group_id: transaction_group.to_string(),
                repetitions: sg4_reps,
            });
        }

        AssembledTree {
            segments: msg_tree.segments,
            groups: all_groups,
            post_group_start: 0,
        }
    }
```

Note: This implementation accesses `tx_engine.definitions` directly. If `definitions` is private, add a `pub fn definitions(&self) -> &[MappingDefinition]` accessor if one doesn't already exist. Check the existing code — there should already be one from the split loader tests.

**Step 2: Run test to verify it passes**

Run: `cargo test -p mig-bo4e --test interchange_reverse_test`
Expected: PASS

**Step 3: Run full test suite**

Run: `cargo test -p mig-bo4e`
Expected: ALL PASS

**Step 4: Run workspace check**

Run: `cargo check --workspace`
Expected: OK

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/interchange_reverse_test.rs
git commit -m "feat(mig-bo4e): add map_interchange_reverse() for two-pass reverse mapping"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 80 |
| Passed | 80 |
| Failed | 0 |
| Skipped | 0 |

Files tested:
- crates/mig-bo4e/src/engine.rs
- crates/mig-bo4e/tests/reverse_all_test.rs (new — 2 tests)
- crates/mig-bo4e/tests/interchange_reverse_test.rs (new — 1 test)
