---
feature: multi-message-transaction
epic: 3
title: "Transaction Group Scoping"
depends_on: [multi-message-transaction/E01]
estimated_tasks: 5
crate: mig-bo4e
status: complete
---

# Epic 3: Transaction Group Scoping

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Enable the `MappingEngine` to map definitions against sub-trees scoped to individual transaction groups (SG4 instances in UTILMD). The existing `map_forward()` and `map_all_forward()` operate on a full `AssembledTree`; we need to scope them to work on a single group repetition as if it were its own tree.

**Architecture:** The key insight is that a transaction group (SG4 instance) contains nested groups (SG5, SG6, SG8, SG10, SG12) that become "top-level" groups within the scoped context. We add a method `AssembledGroupInstance::as_assembled_tree()` that creates a virtual `AssembledTree` from one group instance, then the existing `map_all_forward()` works unchanged on it.

Transaction-level TOML definitions use `source_group` paths relative to the transaction group (e.g., `"SG5"` instead of `"SG4.SG5"`). Message-level definitions keep their absolute paths (e.g., `"SG2"`).

**Tech Stack:** Rust, `mig-assembly::assembler` (AssembledTree, AssembledGroup, AssembledGroupInstance), `mig-bo4e::engine` (MappingEngine)

---

## Task 1: Add as_assembled_tree() to AssembledGroupInstance

**Files:**
- Modify: `crates/mig-assembly/src/assembler.rs`

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/mig-assembly/src/assembler.rs`:

```rust
#[test]
fn test_group_instance_as_assembled_tree() {
    // Build an SG4 instance with root segments (IDE, STS) and child groups (SG5)
    let sg5 = AssembledGroup {
        group_id: "SG5".to_string(),
        repetitions: vec![AssembledGroupInstance {
            segments: vec![AssembledSegment {
                tag: "LOC".to_string(),
                elements: vec![vec!["Z16".to_string(), "DE000111222333".to_string()]],
            }],
            child_groups: vec![],
        }],
    };

    let sg4_instance = AssembledGroupInstance {
        segments: vec![
            AssembledSegment {
                tag: "IDE".to_string(),
                elements: vec![vec!["24".to_string(), "TX001".to_string()]],
            },
            AssembledSegment {
                tag: "STS".to_string(),
                elements: vec![vec!["7".to_string()]],
            },
        ],
        child_groups: vec![sg5],
    };

    let sub_tree = sg4_instance.as_assembled_tree();

    // Root segments of sub-tree are the SG4 instance's segments
    assert_eq!(sub_tree.segments.len(), 2);
    assert_eq!(sub_tree.segments[0].tag, "IDE");
    assert_eq!(sub_tree.segments[1].tag, "STS");

    // Groups of sub-tree are the SG4 instance's child groups
    assert_eq!(sub_tree.groups.len(), 1);
    assert_eq!(sub_tree.groups[0].group_id, "SG5");

    // post_group_start marks where root segments end
    assert_eq!(sub_tree.post_group_start, 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-assembly test_group_instance_as_assembled_tree`
Expected: FAIL — no method `as_assembled_tree` on `AssembledGroupInstance`

**Step 3: Write implementation**

Add an `impl` block for `AssembledGroupInstance` in `crates/mig-assembly/src/assembler.rs`:

```rust
impl AssembledGroupInstance {
    /// Create a virtual `AssembledTree` scoped to this group instance.
    ///
    /// The instance's own segments become the tree's root segments,
    /// and its child groups become the tree's groups. This enables
    /// running `MappingEngine::map_all_forward()` on a single
    /// transaction group as if it were a complete message.
    pub fn as_assembled_tree(&self) -> AssembledTree {
        AssembledTree {
            segments: self.segments.clone(),
            groups: self.child_groups.clone(),
            post_group_start: self.segments.len(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-assembly test_group_instance_as_assembled_tree`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-assembly/src/assembler.rs
git commit -m "feat(mig-assembly): add as_assembled_tree() for sub-tree scoping"
```

---

## Task 2: Add map_message_level() to MappingEngine

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`

**Step 1: Write the failing test**

Add to the test module in `crates/mig-bo4e/src/engine.rs`:

```rust
#[test]
fn test_map_message_level_extracts_sg2_only() {
    use mig_assembly::assembler::*;

    // Build a tree with SG2 (message-level) and SG4 (transaction-level)
    let tree = AssembledTree {
        segments: vec![
            AssembledSegment { tag: "UNH".to_string(), elements: vec![vec!["001".to_string()]] },
            AssembledSegment { tag: "BGM".to_string(), elements: vec![vec!["E01".to_string()]] },
        ],
        groups: vec![
            AssembledGroup {
                group_id: "SG2".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "NAD".to_string(),
                        elements: vec![vec!["MS".to_string(), "9900123".to_string()]],
                    }],
                    child_groups: vec![],
                }],
            },
            AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "IDE".to_string(),
                        elements: vec![vec!["24".to_string(), "TX001".to_string()]],
                    }],
                    child_groups: vec![],
                }],
            },
        ],
        post_group_start: 2,
    };

    // Message-level definition maps SG2
    let mut msg_fields = BTreeMap::new();
    msg_fields.insert("nad.0".to_string(), FieldMapping::Simple("marktrolle".to_string()));
    msg_fields.insert("nad.1.0".to_string(), FieldMapping::Simple("rollencodenummer".to_string()));
    let msg_def = MappingDefinition {
        meta: MappingMeta {
            entity: "Marktteilnehmer".to_string(),
            bo4e_type: "Marktteilnehmer".to_string(),
            companion_type: None,
            source_group: "SG2".to_string(),
            source_path: None,
            discriminator: None,
        },
        fields: msg_fields,
        companion_fields: None,
        complex_handlers: None,
    };

    let engine = MappingEngine::from_definitions(vec![msg_def.clone()]);
    let result = engine.map_all_forward(&tree);

    // Should contain Marktteilnehmer from SG2
    assert!(result.get("Marktteilnehmer").is_some());
    let mt = &result["Marktteilnehmer"];
    assert_eq!(mt["marktrolle"].as_str().unwrap(), "MS");
    assert_eq!(mt["rollencodenummer"].as_str().unwrap(), "9900123");
}
```

**Step 2: Run test to verify it passes**

This test should already pass with existing `map_all_forward()` since SG2 definitions work on the full tree. This validates that message-level mapping works with no code changes.

Run: `cargo test -p mig-bo4e test_map_message_level_extracts_sg2_only`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs
git commit -m "test(mig-bo4e): validate message-level mapping on full tree"
```

---

## Task 3: Add map_transaction() Using Sub-Tree Scoping

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_map_transaction_scoped_to_sg4_instance() {
    use mig_assembly::assembler::*;

    // Build a tree with SG4 containing SG5 (LOC+Z16)
    let tree = AssembledTree {
        segments: vec![
            AssembledSegment { tag: "UNH".to_string(), elements: vec![vec!["001".to_string()]] },
            AssembledSegment { tag: "BGM".to_string(), elements: vec![vec!["E01".to_string()]] },
        ],
        groups: vec![
            AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![
                        AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![vec!["24".to_string(), "TX001".to_string()]],
                        },
                    ],
                    child_groups: vec![
                        AssembledGroup {
                            group_id: "SG5".to_string(),
                            repetitions: vec![AssembledGroupInstance {
                                segments: vec![AssembledSegment {
                                    tag: "LOC".to_string(),
                                    elements: vec![vec!["Z16".to_string(), "DE000111222333".to_string()]],
                                }],
                                child_groups: vec![],
                            }],
                        },
                    ],
                }],
            },
        ],
        post_group_start: 2,
    };

    // Transaction-level definitions: prozessdaten (root of SG4) + marktlokation (SG5 within SG4)
    let mut proz_fields = BTreeMap::new();
    proz_fields.insert("ide.1".to_string(), FieldMapping::Simple("vorgangId".to_string()));
    let proz_def = MappingDefinition {
        meta: MappingMeta {
            entity: "Prozessdaten".to_string(),
            bo4e_type: "Prozessdaten".to_string(),
            companion_type: None,
            source_group: "".to_string(),  // Root-level within transaction sub-tree
            source_path: None,
            discriminator: None,
        },
        fields: proz_fields,
        companion_fields: None,
        complex_handlers: None,
    };

    let mut malo_fields = BTreeMap::new();
    malo_fields.insert("loc.1".to_string(), FieldMapping::Simple("marktlokationsId".to_string()));
    let malo_def = MappingDefinition {
        meta: MappingMeta {
            entity: "Marktlokation".to_string(),
            bo4e_type: "Marktlokation".to_string(),
            companion_type: None,
            source_group: "SG5".to_string(), // Relative to SG4, not "SG4.SG5"
            source_path: None,
            discriminator: None,
        },
        fields: malo_fields,
        companion_fields: None,
        complex_handlers: None,
    };

    let tx_engine = MappingEngine::from_definitions(vec![proz_def, malo_def]);

    // Scope to the SG4 instance and map
    let sg4 = &tree.groups[0]; // SG4 group
    let sg4_instance = &sg4.repetitions[0];
    let sub_tree = sg4_instance.as_assembled_tree();

    let result = tx_engine.map_all_forward(&sub_tree);

    // Should contain Prozessdaten from SG4 root segments
    assert_eq!(result["Prozessdaten"]["vorgangId"].as_str().unwrap(), "TX001");

    // Should contain Marktlokation from SG5 within SG4
    assert_eq!(
        result["Marktlokation"]["marktlokationsId"].as_str().unwrap(),
        "DE000111222333"
    );
}
```

**Step 2: Run test to verify it passes**

This should pass because `as_assembled_tree()` (from Task 1) creates a tree where the SG4's segments are root segments and SG5 becomes a top-level group. The existing `map_all_forward()` handles both cases.

Run: `cargo test -p mig-bo4e test_map_transaction_scoped_to_sg4_instance`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs
git commit -m "test(mig-bo4e): validate transaction-level mapping via sub-tree scoping"
```

---

## Task 4: Add map_interchange() Method

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_map_interchange_produces_full_hierarchy() {
    use mig_assembly::assembler::*;

    // Build a tree with SG2 (message-level) and SG4 with two repetitions (two transactions)
    let tree = AssembledTree {
        segments: vec![
            AssembledSegment { tag: "UNH".to_string(), elements: vec![vec!["001".to_string()]] },
            AssembledSegment { tag: "BGM".to_string(), elements: vec![vec!["E01".to_string()]] },
        ],
        groups: vec![
            AssembledGroup {
                group_id: "SG2".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "NAD".to_string(),
                        elements: vec![vec!["MS".to_string(), "9900123".to_string()]],
                    }],
                    child_groups: vec![],
                }],
            },
            AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![
                    AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![vec!["24".to_string(), "TX001".to_string()]],
                        }],
                        child_groups: vec![],
                    },
                    AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![vec!["24".to_string(), "TX002".to_string()]],
                        }],
                        child_groups: vec![],
                    },
                ],
            },
        ],
        post_group_start: 2,
    };

    // Message-level definitions
    let mut msg_fields = BTreeMap::new();
    msg_fields.insert("nad.0".to_string(), FieldMapping::Simple("marktrolle".to_string()));
    let msg_defs = vec![MappingDefinition {
        meta: MappingMeta {
            entity: "Marktteilnehmer".to_string(),
            bo4e_type: "Marktteilnehmer".to_string(),
            companion_type: None,
            source_group: "SG2".to_string(),
            source_path: None,
            discriminator: None,
        },
        fields: msg_fields,
        companion_fields: None,
        complex_handlers: None,
    }];

    // Transaction-level definitions
    let mut tx_fields = BTreeMap::new();
    tx_fields.insert("ide.1".to_string(), FieldMapping::Simple("vorgangId".to_string()));
    let tx_defs = vec![MappingDefinition {
        meta: MappingMeta {
            entity: "Prozessdaten".to_string(),
            bo4e_type: "Prozessdaten".to_string(),
            companion_type: None,
            source_group: "".to_string(),
            source_path: None,
            discriminator: None,
        },
        fields: tx_fields,
        companion_fields: None,
        complex_handlers: None,
    }];

    let msg_engine = MappingEngine::from_definitions(msg_defs);
    let tx_engine = MappingEngine::from_definitions(tx_defs);

    let result = MappingEngine::map_interchange(
        &msg_engine,
        &tx_engine,
        &tree,
        "SG4",
    );

    // Message-level stammdaten
    assert!(result.stammdaten["Marktteilnehmer"].is_object());
    assert_eq!(result.stammdaten["Marktteilnehmer"]["marktrolle"].as_str().unwrap(), "MS");

    // Two transactions
    assert_eq!(result.transaktionen.len(), 2);
    assert_eq!(result.transaktionen[0].transaktionsdaten["vorgangId"].as_str().unwrap(), "TX001");
    assert_eq!(result.transaktionen[1].transaktionsdaten["vorgangId"].as_str().unwrap(), "TX002");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_map_interchange_produces_full_hierarchy`
Expected: FAIL — `map_interchange` not found

**Step 3: Write implementation**

Add to `impl MappingEngine` in `crates/mig-bo4e/src/engine.rs`:

```rust
    /// Map an assembled tree into message-level and transaction-level results.
    ///
    /// - `msg_engine`: MappingEngine loaded with message-level definitions (SG2, SG3, root segments)
    /// - `tx_engine`: MappingEngine loaded with transaction-level definitions (relative to SG4)
    /// - `tree`: The assembled tree for one message
    /// - `transaction_group`: The group ID that represents transactions (e.g., "SG4")
    ///
    /// Returns a `Nachricht`-like structure with message stammdaten and per-transaction results.
    pub fn map_interchange(
        msg_engine: &MappingEngine,
        tx_engine: &MappingEngine,
        tree: &AssembledTree,
        transaction_group: &str,
    ) -> crate::model::MappedMessage {
        // Map message-level entities
        let stammdaten = msg_engine.map_all_forward(tree);

        // Find the transaction group and map each repetition
        let transaktionen = tree
            .groups
            .iter()
            .find(|g| g.group_id == transaction_group)
            .map(|sg| {
                sg.repetitions
                    .iter()
                    .map(|instance| {
                        let sub_tree = instance.as_assembled_tree();
                        let tx_result = tx_engine.map_all_forward(&sub_tree);

                        // Split: "Prozessdaten" entity goes into transaktionsdaten, rest into stammdaten
                        let mut stammdaten = serde_json::Map::new();
                        let mut transaktionsdaten = serde_json::Value::Null;

                        if let Some(obj) = tx_result.as_object() {
                            for (key, value) in obj {
                                if key == "Prozessdaten" || key == "Nachricht" {
                                    // Merge into transaktionsdaten
                                    if transaktionsdaten.is_null() {
                                        transaktionsdaten = value.clone();
                                    } else if let (Some(existing), Some(new_map)) =
                                        (transaktionsdaten.as_object_mut(), value.as_object())
                                    {
                                        for (k, v) in new_map {
                                            existing.entry(k.clone()).or_insert(v.clone());
                                        }
                                    }
                                } else {
                                    stammdaten.insert(key.clone(), value.clone());
                                }
                            }
                        }

                        crate::model::Transaktion {
                            stammdaten: serde_json::Value::Object(stammdaten),
                            transaktionsdaten,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        crate::model::MappedMessage {
            stammdaten,
            transaktionen,
        }
    }
```

Also add the `MappedMessage` type to `crates/mig-bo4e/src/model.rs`:

```rust
/// Intermediate result from mapping a single message's tree.
///
/// Contains message-level stammdaten and per-transaction results.
/// Used by `MappingEngine::map_interchange()` before wrapping into `Nachricht`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappedMessage {
    /// Message-level BO4E entities (e.g., Marktteilnehmer from SG2).
    pub stammdaten: serde_json::Value,

    /// Per-transaction results (one per SG4 instance).
    pub transaktionen: Vec<Transaktion>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_map_interchange_produces_full_hierarchy`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/src/model.rs
git commit -m "feat(mig-bo4e): add map_interchange() with transaction group scoping"
```

---

## Task 5: Verify Existing Single-Transaction Mapping Still Works

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs` (tests only)

**Step 1: Write a regression test**

```rust
#[test]
fn test_map_interchange_single_transaction_backward_compat() {
    use mig_assembly::assembler::*;

    // Single SG4 with SG5 — the common case for current PID 55001 fixtures
    let tree = AssembledTree {
        segments: vec![
            AssembledSegment { tag: "UNH".to_string(), elements: vec![vec!["001".to_string()]] },
            AssembledSegment { tag: "BGM".to_string(), elements: vec![vec!["E01".to_string(), "DOC001".to_string()]] },
        ],
        groups: vec![
            AssembledGroup {
                group_id: "SG2".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "NAD".to_string(),
                        elements: vec![vec!["MS".to_string(), "9900123".to_string()]],
                    }],
                    child_groups: vec![],
                }],
            },
            AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "IDE".to_string(),
                        elements: vec![vec!["24".to_string(), "TX001".to_string()]],
                    }],
                    child_groups: vec![AssembledGroup {
                        group_id: "SG5".to_string(),
                        repetitions: vec![AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "LOC".to_string(),
                                elements: vec![vec!["Z16".to_string(), "DE000111222333".to_string()]],
                            }],
                            child_groups: vec![],
                        }],
                    }],
                }],
            },
        ],
        post_group_start: 2,
    };

    // Empty message engine (no message-level defs for this test)
    let msg_engine = MappingEngine::from_definitions(vec![]);

    // Transaction defs
    let mut tx_fields = BTreeMap::new();
    tx_fields.insert("ide.1".to_string(), FieldMapping::Simple("vorgangId".to_string()));
    let mut malo_fields = BTreeMap::new();
    malo_fields.insert("loc.1".to_string(), FieldMapping::Simple("marktlokationsId".to_string()));

    let tx_engine = MappingEngine::from_definitions(vec![
        MappingDefinition {
            meta: MappingMeta {
                entity: "Prozessdaten".to_string(),
                bo4e_type: "Prozessdaten".to_string(),
                companion_type: None,
                source_group: "".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: tx_fields,
            companion_fields: None,
            complex_handlers: None,
        },
        MappingDefinition {
            meta: MappingMeta {
                entity: "Marktlokation".to_string(),
                bo4e_type: "Marktlokation".to_string(),
                companion_type: None,
                source_group: "SG5".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: malo_fields,
            companion_fields: None,
            complex_handlers: None,
        },
    ]);

    let result = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4");

    assert_eq!(result.transaktionen.len(), 1);
    assert_eq!(
        result.transaktionen[0].transaktionsdaten["vorgangId"].as_str().unwrap(),
        "TX001"
    );
    assert_eq!(
        result.transaktionen[0].stammdaten["Marktlokation"]["marktlokationsId"].as_str().unwrap(),
        "DE000111222333"
    );
}
```

**Step 2: Run tests**

Run: `cargo test -p mig-bo4e test_map_interchange`
Expected: ALL PASS

Run: `cargo test --workspace`
Expected: ALL PASS

**Step 3: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs
git commit -m "test(mig-bo4e): add backward compat regression test for single-transaction mapping"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 593 |
| Passed | 593 |
| Failed | 0 |
| Skipped | 0 |
| New tests added | 5 |

New tests:
- `assembler::tests::test_group_instance_as_assembled_tree` — validates sub-tree creation from group instance
- `engine::tests::test_map_message_level_extracts_sg2_only` — message-level SG2 mapping on full tree
- `engine::tests::test_map_transaction_scoped_to_sg4_instance` — transaction-level mapping via sub-tree scoping
- `engine::tests::test_map_interchange_produces_full_hierarchy` — full map_interchange with message + transaction levels
- `engine::tests::test_map_interchange_single_transaction_backward_compat` — backward compat for single-transaction case

Files modified:
- `crates/mig-assembly/src/assembler.rs` — added `AssembledGroupInstance::as_assembled_tree()`
- `crates/mig-bo4e/src/engine.rs` — added `MappingEngine::map_interchange()` + 5 tests
- `crates/mig-bo4e/src/model.rs` — added `MappedMessage` type
- `crates/mig-bo4e/src/lib.rs` — exported `MappedMessage`
