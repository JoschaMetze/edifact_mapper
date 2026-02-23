---
feature: multi-message-transaction
epic: 4
title: "TOML Directory Reorganization"
depends_on: [multi-message-transaction/E02]
estimated_tasks: 4
crate: mig-bo4e
status: complete
---

# Epic 4: TOML Directory Reorganization

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Reorganize the TOML mapping directory structure to separate message-level definitions (SG2 Marktteilnehmer, SG3 Ansprechpartner, root-level Nachricht) from transaction-level definitions (everything inside SG4). Add a loader that reads both directories and produces separate `MappingEngine` instances for message-level and transaction-level mapping.

**Architecture:** Current structure is `mappings/FV2504/UTILMD_Strom/pid_{NNNNN}/*.toml` (flat). New structure adds a `message/` subdirectory for shared message-level mappings. The engine gains a `load_split()` method that returns `(MappingEngine, MappingEngine)` — message-level and transaction-level.

The message-level TOMLs are shared across PIDs within the same message type (all UTILMD PIDs share the same SG2/SG3 structure). Transaction-level TOMLs remain per-PID.

**Tech Stack:** Rust, std::fs, mig-bo4e::engine::MappingEngine

---

## Task 1: Create Message-Level TOML Directory

**Files:**
- Create: `mappings/FV2504/UTILMD_Strom/message/marktteilnehmer.toml`
- Create: `mappings/FV2504/UTILMD_Strom/message/ansprechpartner.toml`
- Create: `mappings/FV2504/UTILMD_Strom/message/nachricht.toml`

**Step 1: Identify message-level mappings**

Check both PID directories for files mapping SG2, SG3, or root-level segments:

For PID 55001:
- `marktteilnehmer.toml` — `source_group = "SG2"` (message-level)
- `ansprechpartner.toml` — `source_group = "SG2.SG3"` (message-level)
- `kontakt.toml` — `source_group = "SG2.SG3"` (message-level)
- `nachricht.toml` — `source_group = ""` (root-level, message-level)

For PID 55002:
- `marktteilnehmer.toml` — `source_group = "SG2"` (message-level)
- `ansprechpartner.toml` — `source_group = "SG2.SG3"` (message-level)
- `nachricht.toml` — `source_group = ""` (root-level, message-level)

These map to the same SG2/SG3 structure across all PIDs within UTILMD Strom.

**Step 2: Create message directory and copy shared TOMLs**

```bash
mkdir -p mappings/FV2504/UTILMD_Strom/message
# Copy from pid_55001 as the reference (55002 has identical content for these)
cp mappings/FV2504/UTILMD_Strom/pid_55001/marktteilnehmer.toml mappings/FV2504/UTILMD_Strom/message/
cp mappings/FV2504/UTILMD_Strom/pid_55001/ansprechpartner.toml mappings/FV2504/UTILMD_Strom/message/
cp mappings/FV2504/UTILMD_Strom/pid_55001/nachricht.toml mappings/FV2504/UTILMD_Strom/message/
```

If PID 55001 has a `kontakt.toml`, also copy it:

```bash
cp mappings/FV2504/UTILMD_Strom/pid_55001/kontakt.toml mappings/FV2504/UTILMD_Strom/message/ 2>/dev/null || true
```

**Step 3: Remove message-level TOMLs from PID directories**

Remove the duplicated files from both PID directories:

```bash
# PID 55001
rm mappings/FV2504/UTILMD_Strom/pid_55001/marktteilnehmer.toml
rm mappings/FV2504/UTILMD_Strom/pid_55001/ansprechpartner.toml
rm mappings/FV2504/UTILMD_Strom/pid_55001/nachricht.toml
rm mappings/FV2504/UTILMD_Strom/pid_55001/kontakt.toml 2>/dev/null || true

# PID 55002
rm mappings/FV2504/UTILMD_Strom/pid_55002/marktteilnehmer.toml
rm mappings/FV2504/UTILMD_Strom/pid_55002/ansprechpartner.toml
rm mappings/FV2504/UTILMD_Strom/pid_55002/nachricht.toml
```

**Step 4: Verify the resulting directory structure**

```
mappings/FV2504/UTILMD_Strom/
├── message/
│   ├── marktteilnehmer.toml
│   ├── ansprechpartner.toml
│   ├── kontakt.toml          (if exists)
│   └── nachricht.toml
├── pid_55001/
│   ├── prozessdaten.toml
│   ├── marktlokation.toml
│   ├── marktlokation_daten.toml
│   └── ... (transaction-level only)
├── pid_55002/
│   ├── prozessdaten.toml
│   ├── marktlokation.toml
│   └── ... (transaction-level only)
```

**Step 5: Commit**

```bash
git add mappings/
git commit -m "refactor(mappings): separate message-level TOMLs into message/ directory"
```

---

## Task 2: Add load_split() to MappingEngine

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`

**Step 1: Write the failing test**

Add a new integration test file `crates/mig-bo4e/tests/split_loader_test.rs`:

```rust
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
fn test_load_split_message_and_transaction() {
    let base = mappings_dir();
    let message_dir = base.join("message");
    let tx_dir = base.join("pid_55001");

    let (msg_engine, tx_engine) = MappingEngine::load_split(&message_dir, &tx_dir).unwrap();

    // Message engine should have Marktteilnehmer, Nachricht, Ansprechpartner
    let msg_defs = msg_engine.definitions();
    assert!(
        msg_defs.iter().any(|d| d.meta.entity == "Marktteilnehmer"),
        "Message engine should have Marktteilnehmer"
    );
    assert!(
        msg_defs.iter().any(|d| d.meta.entity == "Nachricht"),
        "Message engine should have Nachricht"
    );

    // Transaction engine should have Prozessdaten, Marktlokation, etc.
    let tx_defs = tx_engine.definitions();
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Prozessdaten"),
        "Transaction engine should have Prozessdaten"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Marktlokation"),
        "Transaction engine should have Marktlokation"
    );

    // Transaction engine should NOT have Marktteilnehmer (that's message-level)
    assert!(
        !tx_defs.iter().any(|d| d.meta.entity == "Marktteilnehmer"),
        "Transaction engine should not have Marktteilnehmer"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e --test split_loader_test`
Expected: FAIL — `load_split` not found

**Step 3: Write implementation**

Add to `impl MappingEngine` in `crates/mig-bo4e/src/engine.rs`:

```rust
    /// Load message-level and transaction-level TOML mappings from separate directories.
    ///
    /// Returns `(message_engine, transaction_engine)` where:
    /// - `message_engine` maps SG2/SG3/root-level definitions (shared across PIDs)
    /// - `transaction_engine` maps SG4+ definitions (PID-specific)
    pub fn load_split(
        message_dir: &Path,
        transaction_dir: &Path,
    ) -> Result<(Self, Self), MappingError> {
        let msg_engine = Self::load(message_dir)?;
        let tx_engine = Self::load(transaction_dir)?;
        Ok((msg_engine, tx_engine))
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e --test split_loader_test`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/split_loader_test.rs
git commit -m "feat(mig-bo4e): add load_split() for message/transaction TOML directories"
```

---

## Task 3: Update Existing Tests to Use New Directory Layout

**Files:**
- Modify: existing tests in `crates/mig-bo4e/tests/` that load from `pid_55001/` or `pid_55002/`

**Step 1: Identify affected tests**

Search for tests that load `mappings/FV2504/UTILMD_Strom/pid_55001` or `pid_55002` and expect message-level definitions to be present (Marktteilnehmer, Nachricht, Ansprechpartner).

Run: `cargo test -p mig-bo4e -- --nocapture 2>&1 | grep -i "FAIL\|error\|not found"`

**Step 2: Fix tests**

For tests that previously loaded from `pid_55001/` and expected Marktteilnehmer:
- Change them to use `load_split()` instead of `load()`
- Or if they only test transaction-level mappings, no change needed (they just won't find message-level entities, which is correct)

For tests that need the full set of definitions (both message and transaction):
- Load both engines and combine their definitions:

```rust
let (msg_engine, tx_engine) = MappingEngine::load_split(&msg_dir, &tx_dir)?;
// For backward compat in tests that need all defs:
let mut all_defs = msg_engine.definitions().to_vec();
all_defs.extend(tx_engine.definitions().iter().cloned());
let combined_engine = MappingEngine::from_definitions(all_defs);
```

**Step 3: Run all tests**

Run: `cargo test -p mig-bo4e`
Expected: ALL PASS

**Step 4: Commit**

```bash
git add crates/mig-bo4e/tests/ mappings/
git commit -m "fix(mig-bo4e): update tests for message/transaction directory split"
```

---

## Task 4: Update MIG Registry to Use Split Loading

**Files:**
- Modify: `crates/automapper-api/src/state.rs` (or wherever `MigRegistry` loads mapping engines)

**Step 1: Find where mapping engines are loaded**

Search for `mapping_engine_for_pid` or `MappingEngine::load` in `crates/automapper-api/`.

**Step 2: Update to load message-level directory alongside PID directory**

The registry should load `message/` once per message variant and cache it. When `mapping_engine_for_pid()` is called, it returns both engines (or a combined view).

Option A: Return a tuple from the registry:

```rust
pub fn mapping_engines_for_pid(
    &self,
    format_version: &str,
    variant: &str,
    pid: &str,
) -> Option<(&MappingEngine, &MappingEngine)> {
    // Returns (message_engine, transaction_engine)
}
```

Option B: Keep the existing `mapping_engine_for_pid` for backward compat but also expose a split variant. This is left flexible for the implementer since the API handler (Epic 5) will call this directly.

**Step 3: Verify**

Run: `cargo test -p automapper-api`
Expected: PASS (existing tests may need minor updates)

Run: `cargo check --workspace`
Expected: OK

**Step 4: Commit**

```bash
git add crates/automapper-api/
git commit -m "feat(api): update MIG registry for split message/transaction loading"
```
