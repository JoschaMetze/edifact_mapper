# TOML Mapping Centralization Design

## Problem

2,835 TOML mapping files across 187 PIDs. 60% are exact duplicates. When a shared mapping changes (e.g., LOC+Z16 field name), we must update 44 identical files. This is a maintainability burden.

## Solution

Engine-level inheritance with schema-aware filtering. A new `common/` directory holds shared transaction-level templates. PIDs only contain files that differ from common templates. The PID schema JSON filters which common templates apply to each PID.

## Architecture

### Directory Layout

```
mappings/FV2504/UTILMD_Strom/
├── message/           # Message-level (unchanged): nachricht, sg1_nachricht, kontakt, marktteilnehmer
├── common/            # NEW: shared transaction-level templates
│   ├── sg5_z16.toml           # LOC+Z16 Marktlokation
│   ├── sg5_z17.toml           # LOC+Z17 Messlokation
│   ├── sg5_z18.toml           # LOC+Z18 Netzlokation
│   ├── sg5_z19.toml           # LOC+Z19 SteuerbareRessource
│   ├── sg5_z20.toml           # LOC+Z20 TechnischeRessource
│   ├── sg5_z21.toml           # LOC+Z21 Tranche
│   ├── sg5_z22.toml           # LOC+Z22 RuhendeMarktlokation
│   ├── sg5_z15.toml           # LOC+Z15 MaBiS-Zaehlpunkt
│   ├── rff_z13.toml           # SG6 RFF+Z13 Prüfidentifikator
│   └── rff_tn.toml            # SG6 RFF+TN Transaktionsnummer
├── pid_55001/         # Only PID-specific files remain
│   ├── sg4.toml
│   ├── geschaeftspartner.toml
│   └── ...            # SG8/SG10/SG6 files unique to this PID
```

### Engine Changes

New method on `MappingEngine`:

```rust
/// Load transaction-level mappings with common template inheritance.
///
/// 1. Loads all `.toml` from `common_dir`
/// 2. Filters: keeps only definitions whose `source_path` exists in the PID schema
/// 3. Loads all `.toml` from `pid_dir`
/// 4. For each PID definition, if a common definition has matching
///    `(source_group, discriminator)`, replaces the common one (file-level replacement)
/// 5. Merges both sets: common first, then PID additions
pub fn load_with_common(
    common_dir: &Path,
    pid_dir: &Path,
    pid_schema: &PidSchema,
) -> Result<MappingEngine>
```

Updated `load_split`:

```rust
/// Load message + transaction engines, with optional common template directory.
pub fn load_split_with_common(
    message_dir: &Path,
    common_dir: &Path,
    transaction_dir: &Path,
    pid_schema: &PidSchema,
) -> Result<(MappingEngine, MappingEngine)>
```

Schema-aware filter:

```rust
/// Check if a source_path corresponds to an existing group in the PID schema.
fn schema_has_group(schema: &PidSchema, source_path: &str) -> bool {
    // Walk the schema's `fields` tree following the dot-separated path
    // "sg4.sg5_z16" → schema.fields["sg4"]["sg5_z16"] must exist
    // "sg4.sg8_z98.sg10" → schema.fields["sg4"]["sg8_z98"]["sg10"] must exist
}
```

### Override Rules

- **Override key**: `(source_group, discriminator)` — NOT including `entity`
- **File-level replacement**: PID file completely replaces the common version (no field merging)
- **PID-only files**: Files in PID dir with no common match are added as-is
- **Schema filtering**: Common files whose `source_path` is absent from PID schema are skipped

### Backward Compatibility

- Existing `load()`, `load_split()`, `load_merged()` remain unchanged
- New methods are additive
- Tests can migrate incrementally

## Files for `common/`

### Tier 1: Single Variant (8 SG5 files)

All copies are 100% byte-identical. Zero risk.

| File | Entity | Copies | source_path |
|------|--------|--------|-------------|
| `sg5_z16.toml` | Marktlokation | 44 | `sg4.sg5_z16` |
| `sg5_z17.toml` | Messlokation | 20 | `sg4.sg5_z17` |
| `sg5_z18.toml` | Netzlokation | 16 | `sg4.sg5_z18` |
| `sg5_z19.toml` | SteuerbareRessource | 15 | `sg4.sg5_z19` |
| `sg5_z20.toml` | TechnischeRessource | 14 | `sg4.sg5_z20` |
| `sg5_z21.toml` | Tranche | 27 | `sg4.sg5_z21` |
| `sg5_z22.toml` | RuhendeMarktlokation | 6 | `sg4.sg5_z22` |
| `sg5_z15.toml` | MabisZaehlpunkt | 11 | `sg4.sg5_z15` |

### Tier 2: Majority Variant (2 RFF files)

119/53 copies are identical. PIDs with non-majority format keep local override.

| File | Entity | Majority copies | Variant copies |
|------|--------|----------------|----------------|
| `rff_z13.toml` | Prozessdaten | 119 | ~46 (format diffs + real diffs) |
| `rff_tn.toml` | Prozessdaten | 53 | ~23 |

### NOT Centralized

| Pattern | Reason |
|---------|--------|
| `sg4.toml` | 36 variants — PID-specific STS/DTM |
| `geschaeftspartner.toml` | 27 variants — PID-specific NAD fields |
| SG8 info files | ~98% unique content |
| SG10 zuordnung files | ~98% unique content |

### Message-Level Cleanup

Delete `_99_nachricht.toml`, `_99_kontakt.toml`, `_99_marktteilnehmer.toml` from all PID dirs (118+ PIDs). These are message-level definitions already in `message/` — they're loaded into the tx_engine where they never match SG4 groups (inert).

Also delete non-prefixed message-level copies (`nachricht.toml`, `kontakt.toml`, `marktteilnehmer.toml`) from early PIDs (55001-55038) that have their own copies.

## Callers to Update

### Production Code

| File | Current | Change |
|------|---------|--------|
| `fixture-renderer/src/renderer.rs:80` | `load_split()` | `load_split_with_common()` |
| `fixture-renderer/src/renderer.rs:222` | `load_split()` | `load_split_with_common()` |
| `automapper-api/src/state.rs:177` | `load_merged()` | Use `load_with_common()` |
| `automapper-api/src/state.rs:334` | `load_merged()` | Use `load_with_common()` |
| `automapper-generator/src/main.rs:1435` | `load()` | `load_with_common()` |

### Test Code

| File | Call sites |
|------|-----------|
| `mig-bo4e/tests/pid_bulk_roundtrip_test.rs` | Main roundtrip harness — update helper |
| `mig-bo4e/tests/reverse_roundtrip_test.rs` | `load()` for msg + tx |
| `mig-bo4e/tests/pid_55013_to_55035_test.rs` | `run_full_roundtrip()` helper |
| `mig-bo4e/tests/pid_55003_to_55012_test.rs` | Multiple |
| `mig-bo4e/tests/split_loader_test.rs` | `load_split()` |
| `mig-bo4e/tests/mapping_files_test.rs` | `load()` and `load_merged()` |
| + ~15 more test files | Various |

## Implementation Phases

### Phase 1: Engine Changes (no file moves)

**Goal**: Add `load_with_common()` and `load_split_with_common()` without changing existing behavior.

1. Add `PidSchema` type (or reuse existing schema type) with `has_group(source_path: &str) -> bool`
2. Implement `load_with_common(common_dir, pid_dir, pid_schema) -> Result<MappingEngine>`
3. Implement `load_split_with_common(message_dir, common_dir, tx_dir, pid_schema) -> Result<(MappingEngine, MappingEngine)>`
4. Write unit tests for the override mechanism:
   - Common def filtered by schema (source_path absent)
   - Common def included when schema has matching group
   - PID def overrides common def with same `(source_group, discriminator)`
   - PID-only defs pass through unchanged
5. Verify: all 1,421 existing tests pass (no behavioral change)

### Phase 2: Create `common/` Directory

**Goal**: Add shared files without removing any PID copies yet.

1. Create `mappings/FV2504/UTILMD_Strom/common/`
2. Write 10 canonical TOML files (8 SG5 + 2 RFF) — use majority variant content
3. Write integration test: for each PID with a fixture, load with common/ and verify identical BO4E output vs loading without common/ (both should deep_merge_insert identically since PID copies still exist)
4. Verify: all tests pass

### Phase 3: Delete Redundant PID Copies

**Goal**: Remove files from PID dirs that are identical to common/ versions.

1. For each common/ file, identify all PID copies that are semantically identical:
   - Byte-identical copies: delete immediately
   - Formatting-only variants (field order, inline vs expanded TOML): normalize to canonical format, then delete
2. PIDs with semantically different content keep their local copy (override)
3. Run roundtrip tests after each batch to verify
4. Expected deletions: ~315 files (145 SG5 + 170 RFF)

### Phase 4: Delete Message-Level Duplicates

**Goal**: Remove inert message-level definitions from PID dirs.

1. Delete `_99_nachricht.toml` from all PID dirs that have it (~118 files)
2. Delete `_99_kontakt.toml` from all PID dirs (~118 files)
3. Delete `_99_marktteilnehmer.toml` from all PID dirs (~118 files)
4. Delete non-prefixed message copies from early PIDs (~60 files)
5. Run full test suite to verify no regression

Expected deletions: ~414 files

### Phase 5: Update Callers

**Goal**: Migrate production and test callers to use common/ directory.

1. Update `run_full_roundtrip()` test helper to use `load_split_with_common()`
2. Update `pid_bulk_roundtrip_test.rs` to pass common_dir
3. Update `fixture-renderer` to accept and pass common_dir
4. Update `automapper-api` state initialization
5. Update `automapper-generator` main.rs
6. Update remaining test files incrementally
7. Run full test suite
8. Run `cargo clippy --workspace -- -D warnings`
9. Run `cargo fmt --all -- --check`

## Expected Outcome

| Metric | Before | After |
|--------|--------|-------|
| Total TOML files | 2,835 | ~2,106 |
| Files in common/ | 0 | 10 |
| Files eliminated | — | ~729 |
| Reduction | — | 25.7% |
| Test count | 1,421 | 1,421+ |

The 25.7% reduction focuses on the highest-value targets (single-variant files and inert duplicates). Further centralization of SG8/SG10 files is possible in future if more patterns standardize.
