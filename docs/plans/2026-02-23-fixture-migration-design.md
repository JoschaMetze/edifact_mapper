# Format Version Fixture Migration: Systematic Test Data Migration Across EDIFACT Format Versions

**Date**: 2026-02-23
**Scope**: `automapper-generator`, `mig-types`, `mig-bo4e`, `mig-assembly`
**Depends on**: MIG-driven mapping pipeline (Feature 5), PID schema generation

## Problem

When a new EDIFACT format version is released (e.g., FV2504 → FV2510), the MIG/AHB XML schemas change — segments are added or removed, code values change, group structures are renested, and requirement levels shift. All existing test fixture `.edi` files must be updated to the new version's wire format.

Today this is entirely manual: someone diffs the MIG XMLs by eye, rewrites fixtures by hand, and hopes nothing was missed. With 187 PIDs for UTILMD alone and ~1,944 fixture files across message types, this does not scale.

### What Changes Between Format Versions

Comparing PID 55001 across FV2310 (S1.1), FV2404 (S1.1a), and FV2504 (S2.1):

| Aspect | FV2310 (S1.1) | FV2404 (S1.1a) | FV2504 (S2.1) |
|---|---|---|---|
| UNH version | `S1.1` | `S1.1a` | `S2.1` |
| IMD segment | present | present | removed |
| PIA segment | absent | absent | new |
| STS structure | `STS+7++E01` | `STS+7++E01` | `STS+7++E01+ZW4+E03` |
| SEQ qualifiers | Z01, Z03, Z12, Z75 | same | Z79, ZH0, Z01, Z75 |
| NAD qualifiers | Z04, Z05, Z09, DP | same | Z04, Z09 only |
| LOC segments | Z16, Z17 | absent | Z16 only |

These are deep structural changes — not just find-and-replace. The knowledge of what changed lives only in the MIG/AHB XMLs themselves; there is no formal migration changelog.

## Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Diff granularity | PID schema level | PID schemas are the merged MIG+AHB view — diffing them captures both structural and business rule changes in one pass |
| Diff input | PID schema JSON vs PID schema JSON | Already generated, already captures segment tree with discriminators and valid codes |
| AHB condition changes | Separate human-readable report | Fixtures represent concrete instances, not conditional logic — conditions only matter for TOML mapping authors |
| Canonical BO4E format | `.mig.bo.json` extension | Consistent with existing `.bo.json` convention, `.mig.` infix distinguishes pipeline source |
| Fixture generation strategy | Three-phase: bootstrap → TOML dev → golden records | Avoids chicken-and-egg: bootstrap fixtures enable TOML development, TOML mappings enable golden records |

## Architecture

### Three-Phase Migration Pipeline

```
New FV2510 MIG/AHB XMLs land
         │
         ▼
┌─────────────────────────────┐
│ automapper-generator        │
│   generate-mig-types        │  (existing tool)
│   → PID schemas + Rust types│
│   for FV2510                │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ 1a: mig-diff                │
│   old: pid_55001_schema     │  FV2504
│   new: pid_55001_schema     │  FV2510
│   → diff JSON (structural)  │
│   → AHB condition report    │  (for humans)
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ 1b: migrate-fixture         │
│   old .edi + diff + new     │
│   schema → new .edi         │
│   ⚠ warnings for manual     │
│   review items              │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ Phase 2: TOML development   │
│   diff report guides which  │
│   TOMLs to copy/adapt/write │
│   migrated fixtures for     │
│   testing roundtrips        │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│ 3: render-fixture           │
│   canonical .mig.bo.json    │
│   → reverse through FV2510  │
│   TOML mappings             │
│   → golden .edi fixtures    │
└──────────┬──────────────────┘
           │
           ▼
  FV2510 fixtures + mappings
  ready for CI
```

### File Layout After Migration

Current state (FV2504):
```
example_.../UTILMD/FV2504/
  55001_UTILMD_S2.1_ALEXANDE121980.edi          # EDIFACT source
  55001_UTILMD_S2.1_ALEXANDE121980.bo.json      # Legacy BO4E (flat stammdaten[])
  55001_UTILMD_S2.1_ALEXANDE121980.mig.bo.json  # MIG-pipeline BO4E (canonical)
```

After FV2510 migration:
```
example_.../UTILMD/FV2510/
  55001_UTILMD_S2.2_ALEXANDE121980.edi          # Re-rendered from .mig.bo.json
  55001_UTILMD_S2.2_ALEXANDE121980.mig.bo.json  # Same canonical BO4E (or enhanced)
```

The `.mig.bo.json` is the version-independent semantic record. Each format version is a rendering of that corpus through version-specific TOML mappings.

## Tool 1a: `mig-diff`

New subcommand in `automapper-generator`.

### CLI Interface

```bash
cargo run -p automapper-generator -- mig-diff \
  --old-schema crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json \
  --new-schema crates/mig-types/src/generated/fv2510/utilmd/pids/pid_55001_schema.json \
  --old-ahb xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_*.xml \
  --new-ahb xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_*.xml \
  --pid 55001 \
  --output docs/migrations/FV2504_to_FV2510/pid_55001_diff.json
```

### Diff Output Format

The diff operates at PID level, comparing two PID schema JSONs. Output is structured JSON:

```json
{
  "old_version": "FV2504",
  "new_version": "FV2510",
  "message_type": "UTILMD",
  "pid": "55001",
  "unh_version": { "old": "S2.1", "new": "S2.2" },
  "segments": {
    "added": [
      {
        "group": "SG8",
        "tag": "MEA",
        "number": "00142",
        "context": "New metering segment in SG8_Z98"
      }
    ],
    "removed": [
      { "group": "SG4", "tag": "IMD", "number": "00067" }
    ],
    "unchanged": [
      { "group": "SG5", "tag": "LOC", "number": "00089" }
    ]
  },
  "codes": {
    "changed": [
      {
        "segment": "CCI",
        "element": "0",
        "group": "SG10",
        "added": ["Z95"],
        "removed": ["Z88"],
        "context": "CCI qualifier in SG10"
      }
    ]
  },
  "groups": {
    "added": [
      {
        "group": "SG8_ZH5",
        "parent": "SG4",
        "entry_segment": "SEQ+ZH5"
      }
    ],
    "removed": [],
    "restructured": [
      {
        "group": "SG10",
        "description": "SG10 moved from under SG8 to under SG5",
        "manual_review": true
      }
    ]
  },
  "elements": {
    "added": [
      {
        "segment": "STS",
        "index": 4,
        "sub_index": 0,
        "description": "New E03 component"
      }
    ],
    "removed": []
  }
}
```

### Categories

- **segments**: Whole segments added/removed/unchanged, identified by AHB Number.
- **codes**: Code values within elements that were added, removed, or renamed.
- **groups**: SG group topology changes — additions, removals, and restructurings. Restructurings are flagged `manual_review: true`.
- **elements**: Composite/element additions or removals within existing segments.

### Supplementary AHB Condition Report

A human-readable markdown report for TOML mapping authors:

```markdown
# AHB Condition Changes: PID 55001 (FV2504 → FV2510)

## Requirement Level Changes
| Segment | Element | Old | New | Note |
|---------|---------|-----|-----|------|
| LOC.1.0 | MaLo-ID | Muss | Muss [1] ∧ [2] | Now conditional |

## New Conditions
- [999] Wenn Sonderfall XYZ ...

## Removed Conditions
- [42] No longer referenced
```

This report is not consumed by `migrate-fixture` — fixtures represent concrete instances, not conditional logic.

### Implementation

The diff algorithm walks both PID schema JSON trees in parallel:

1. **Group-level diff**: Compare top-level field keys (`sg2`, `sg4`, `sg4.sg5_z16`, etc.). Keys present in new but not old → added groups. Keys in old but not new → removed groups.
2. **Segment-level diff**: Within matching groups, compare segment lists by `(tag, number)` pairs.
3. **Code-level diff**: Within matching segments, compare `codes[]` arrays per element.
4. **Element-level diff**: Within matching segments, compare element/composite structure by `(index, sub_index)`.
5. **Restructure detection**: If a segment tag+number appears in a different group path between versions, flag as restructured.

## Tool 1b: `migrate-fixture`

New subcommand in `automapper-generator`. Consumes old `.edi` + diff JSON + new PID schema to produce a migrated `.edi`.

### CLI Interface

```bash
cargo run -p automapper-generator -- migrate-fixture \
  --old-fixture example_.../FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi \
  --diff docs/migrations/FV2504_to_FV2510/pid_55001_diff.json \
  --new-pid-schema crates/mig-types/src/generated/fv2510/utilmd/pids/pid_55001_schema.json \
  --output example_.../FV2510/55001_UTILMD_S2.2_ALEXANDE121980.edi
```

### Migration Rules

The tool applies diff entries in order of confidence:

| Diff category | Action | Confidence |
|---|---|---|
| `segments.unchanged` | Copy segment data verbatim from old fixture | Automatic |
| `segments.removed` | Drop segment from output | Automatic |
| `codes.changed` (1:1 rename) | Substitute old code → new code | Automatic |
| `unh_version` | Update UNH version string | Automatic |
| `segments.added` | Generate from new PID schema with valid default codes, empty data elements | Automatic + warning |
| `elements.added` | Append empty component to existing segment | Automatic + warning |
| `codes.changed` (1:N or N:1) | Use first valid code, warn | Warning |
| `groups.added` | Generate skeleton group from schema | Warning |
| `groups.restructured` | Flag for manual review, attempt best-effort move | Warning |

### Output

The migrated `.edi` file plus a warnings file:

```
55001_UTILMD_S2.2_ALEXANDE121980.edi              # Migrated fixture
55001_UTILMD_S2.2_ALEXANDE121980.edi.warnings.txt # Review items
```

Warnings file example:
```
WARNING: New mandatory segment MEA in SG8_Z98 — filled with schema defaults, needs realistic data
WARNING: Group SG10 restructured — moved segments from SG8 to SG5, verify nesting
WARNING: Code CCI.0 changed Z88→[Z95,Z96] — used Z95, verify correctness
```

## Tool 3: `render-fixture`

New subcommand in `automapper-generator`. Takes canonical `.mig.bo.json` and renders through version-specific TOML mappings to produce golden `.edi` fixtures.

### CLI Interface

```bash
cargo run -p automapper-generator -- render-fixture \
  --bo4e-input example_.../FV2504/55001_UTILMD_S2.1_ALEXANDE121980.mig.bo.json \
  --mappings mappings/FV2510/UTILMD_Strom/ \
  --mig-xml xml-migs-and-ahbs/FV2510/UTILMD_MIG_Strom_*.xml \
  --ahb-xml xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_*.xml \
  --pid 55001 \
  --output example_.../FV2510/55001_UTILMD_S2.2_ALEXANDE121980.edi
```

### Pipeline

1. Load canonical `.mig.bo.json` — the version-independent business content
2. Load FV2510 TOML mappings (message + PID-specific)
3. Reverse-map BO4E JSON → `AssembledTree` via `MappingEngine`
4. Disassemble tree → segments via `Disassembler`
5. Reconstruct UNB/UNZ envelope with correct version metadata
6. Render EDIFACT string

This reuses the existing reverse mapping pipeline in `mig-bo4e` and the disassembler in `mig-assembly`. The new code is primarily CLI plumbing and envelope reconstruction with the correct UNH version identifier.

### Handling New Entities

When a new format version introduces entities or fields that don't exist in the canonical BO4E data:

1. `render-fixture` reports missing required fields as warnings
2. The canonical `.mig.bo.json` is manually enhanced with the new data
3. `render-fixture` is re-run to produce complete golden fixtures

The `.mig.bo.json` evolves over time as new format versions introduce new concepts, but it remains the single source of business scenario truth.

## Artifacts and Storage

| Artifact | Location | Generated by |
|---|---|---|
| PID schemas (new FV) | `crates/mig-types/src/generated/fv2510/` | existing `generate-mig-types` |
| Diff reports | `docs/migrations/FV2504_to_FV2510/` | `mig-diff` |
| AHB condition reports | `docs/migrations/FV2504_to_FV2510/` | `mig-diff` |
| Bootstrap fixtures | `example_.../UTILMD/FV2510/` | `migrate-fixture` |
| TOML mappings | `mappings/FV2510/UTILMD_Strom/` | manual (guided by diff) |
| Canonical BO4E | `example_.../UTILMD/FV2504/*.mig.bo.json` | forward pipeline (once) |
| Golden fixtures | `example_.../UTILMD/FV2510/` | `render-fixture` |

## Testing Strategy

- **mig-diff**: Unit tests comparing hand-crafted schema pairs with known deltas. Property tests verifying diff(A, A) produces empty diff.
- **migrate-fixture**: Integration tests migrating a known FV2504 fixture through a synthetic diff, validating the output parses cleanly against the new PID schema.
- **render-fixture**: Roundtrip test: forward-map FV2504 fixture → `.mig.bo.json` → reverse-map back → compare with original `.edi`. This validates the canonical BO4E captures enough information for lossless re-rendering.

## Change Summary

| Crate | Change | Size |
|---|---|---|
| `automapper-generator` | Add `mig-diff`, `migrate-fixture`, `render-fixture` subcommands | L |
| `mig-types` | No changes (schema generation already exists) | — |
| `mig-assembly` | No changes (disassembler/renderer already exist) | — |
| `mig-bo4e` | No changes (reverse mapping already exists) | — |

All new code lives in `automapper-generator`. The existing pipeline crates are consumed as libraries.
