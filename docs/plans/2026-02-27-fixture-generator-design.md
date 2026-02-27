# Synthetic Fixture Generator: Generate EDIFACT Test Fixtures from PID Schemas

**Date**: 2026-02-27
**Scope**: `automapper-generator`
**Depends on**: PID schema generation (existing)

## Problem

187 PIDs exist in the FV2504 UTILMD AHB, but only 65 have `.edi` fixture files. This blocks TOML mapping development and roundtrip testing for 122 PIDs. Creating fixtures manually is tedious and error-prone.

## Solution

A new `generate-fixture` subcommand in `automapper-generator` that reads a PID schema JSON and produces a structurally valid `.edi` file with:
- Correct EDIFACT envelope (UNB/UNH/UNT/UNZ)
- All PID-specific segments in MIG-defined order
- Valid code values (first AHB-filtered code from schema)
- Type-aware placeholder data values

## CLI Interface

```bash
cargo run -p automapper-generator -- generate-fixture \
  --pid-schema crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55043_schema.json \
  --output example_market_communication_bo4e_transactions/UTILMD/FV2504/55043_UTILMD_S2.1_GENERATED.edi
```

**Input**: PID schema JSON (sole input — no MIG/AHB XML needed at runtime).
**Output**: Complete `.edi` file.

## Generation Algorithm

1. Emit UNB with synthetic envelope (sender/receiver GLNs, timestamp, control ref)
2. Emit UNH with message type + version from schema metadata
3. Emit root_segments (BGM, DTM) using skeleton generator
4. Walk `fields` tree depth-first, emitting one instance per group variant:
   - SG2 → NAD + nested SG3
   - SG4 → IDE, DTM, STS + nested SG5/SG6/SG8/SG12
   - SG8 → SEQ + nested SG9/SG10 (CCI, CAV)
5. Compute segment count (UNH through UNT inclusive)
6. Emit UNT + UNZ

Each segment produced by the existing `generate_skeleton_segment()` function, extended with type-aware placeholders for data elements.

## Type-Aware Placeholders

Driven by EDIFACT data element IDs from the schema:

| Data Element ID | Meaning | Placeholder |
|---|---|---|
| 3039 | Party identification | `1234567890128` (GLN) |
| 3225 | Location ID | `DE0012345678901234567890123456789012` (MaLo) |
| 2380 | Date/time | `20250401` / `202504011200` |
| 3036 | Party name | `Mustermann` |
| 3042 | Street address | `Musterstrasse 1` |
| 3164 | City | `Musterstadt` |
| 3251 | Postal code | `12345` |
| fallback | Other data elements | `X` |

## Implementation

| File | Change |
|---|---|
| `src/fixture_generator.rs` | Core `generate_fixture()` — schema tree walker + segment emitter |
| `src/fixture_generator/placeholders.rs` | Data element ID → placeholder value mapping |
| `src/main.rs` | `GenerateFixture` CLI variant |

## Testing

1. Generate fixture for PID 55001, verify it parses through assembly pipeline
2. Generate fixture for uncovered PID (55043), verify structural validity
