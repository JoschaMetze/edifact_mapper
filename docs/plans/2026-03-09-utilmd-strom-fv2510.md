# UTILMD Strom FV2510 Format Migration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add FV2510 support for UTILMD Strom by copying all 2,831 FV2504 TOML mappings and validating with FV2510 MIG/AHB XMLs and generated fixtures.

**Architecture:** Copy-and-revalidate. FV2510 schemas are structurally identical to FV2504 (same 187 PIDs, same field paths, only `enum` metadata removed). No TOML content changes needed. Engine is already format-version agnostic.

**Tech Stack:** Rust, TOML mappings, MIG/AHB XML, generated EDIFACT fixtures

---

### Task 1: Copy TOML mappings to FV2510

**Files:**
- Source: `mappings/FV2504/UTILMD_Strom/` (2,831 files)
- Create: `mappings/FV2510/UTILMD_Strom/` (identical copy)

**Step 1: Copy the directory tree**

```bash
cp -r mappings/FV2504/UTILMD_Strom mappings/FV2510/UTILMD_Strom
```

**Step 2: Verify file count matches**

```bash
find mappings/FV2504/UTILMD_Strom -name '*.toml' | wc -l
find mappings/FV2510/UTILMD_Strom -name '*.toml' | wc -l
```

Expected: Both should show 2,831 (or current count).

**Step 3: Commit**

```bash
git add mappings/FV2510/UTILMD_Strom/
git commit -m "feat(utilmd-strom): copy FV2504 TOML mappings to FV2510"
```

---

### Task 2: Create FV2510 test config module

**Files:**
- Create: `crates/mig-bo4e/tests/common/utilmd_strom_fv2510.rs`
- Modify: `crates/mig-bo4e/tests/common/mod.rs`

**Step 1: Read existing FV2504 UTILMD Strom test config**

Read `crates/mig-bo4e/tests/common/test_utils.rs` for the existing UTILMD Strom config (the default CONFIG constant and helper functions).

**Step 2: Create FV2510 test config module**

Create `utilmd_strom_fv2510.rs` modeled on the existing `utilmd_gas.rs` and `insrpt.rs` patterns. Key differences from FV2504:

- `mig_xml_path` → FV2510 UTILMD Strom MIG XML
- `ahb_xml_path` → FV2510 UTILMD Strom AHB XML
- `fixture_dir` → FV2510 UTILMD fixture directory
- `mappings_base` → `mappings/FV2510/UTILMD_Strom`
- `schema_dir` → `crates/mig-types/src/generated/fv2510/utilmd/pids`
- `format_version` → `"FV2510"`
- `variant` → `Some("Strom")`
- `tx_group` → `"SG4"`

Include helper functions: `path_resolver()`, `load_pid_filtered_mig()`, `load_split_engines()`, `discover_generated_fixture()`, `message_dir()`, `pid_dir()`.

**Step 3: Register in mod.rs**

Add `pub mod utilmd_strom_fv2510;` to `crates/mig-bo4e/tests/common/mod.rs`.

**Step 4: Verify compilation**

```bash
cargo check -p mig-bo4e --tests
```

**Step 5: Commit**

```bash
git add crates/mig-bo4e/tests/common/utilmd_strom_fv2510.rs crates/mig-bo4e/tests/common/mod.rs
git commit -m "test(utilmd-strom): add FV2510 test config module"
```

---

### Task 3: Create bulk roundtrip test for all 187 PIDs

**Files:**
- Create: `crates/mig-bo4e/tests/utilmd_strom_fv2510_bulk_roundtrip_test.rs`

**Step 1: Read existing FV2504 bulk test as reference**

Read `crates/mig-bo4e/tests/utilmd_strom_bulk_roundtrip_test.rs` (or equivalent FV2504 bulk test) for the pattern including:
- Full PID list (187 PIDs)
- KNOWN_INCOMPLETE list
- SG2/SG3 normalization function
- `run_generated_roundtrip()` helper with full pipeline

**Step 2: Create FV2510 bulk roundtrip test**

Create `utilmd_strom_fv2510_bulk_roundtrip_test.rs` that:
- Imports `utilmd_strom_fv2510` module instead of FV2504 test_utils
- Lists all 187 PIDs
- Uses same KNOWN_INCOMPLETE list as FV2504 (same structural limitations)
- Runs full pipeline: EDIFACT → tokenize → split → assemble → map_interchange → map_interchange_reverse → disassemble → render → compare
- Uses `discover_generated_fixture()` from the FV2510 module
- Includes SG2/SG3 normalization

**Step 3: Run the bulk test**

```bash
cargo test -p mig-bo4e --test utilmd_strom_fv2510_bulk_roundtrip_test -- --nocapture
```

Expected: 180 passed, 7 known-incomplete, 187 total.

**Step 4: Commit**

```bash
git add crates/mig-bo4e/tests/utilmd_strom_fv2510_bulk_roundtrip_test.rs
git commit -m "test(utilmd-strom): add FV2510 bulk roundtrip test for all 187 PIDs"
```

---

### Task 4: Final verification

**Step 1: Run clippy**

```bash
cargo clippy -p mig-bo4e -- -D warnings
```

**Step 2: Run fmt check**

```bash
cargo fmt --all -- --check
```

**Step 3: Run full mig-bo4e test suite**

```bash
cargo test -p mig-bo4e
```

Expected: All tests pass (FV2504 + FV2510).

**Step 4: Fix any issues found**

---

### Task 5: Commit and push

**Step 1: Check git status**

```bash
git status
git log --oneline -5
```

**Step 2: Push**

```bash
git push
```
