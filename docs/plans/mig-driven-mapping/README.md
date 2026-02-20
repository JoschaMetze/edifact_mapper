# MIG-Driven Mapping Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the hand-coded mapper approach with a MIG/AHB-driven architecture where EDIFACT is parsed into generated PID-specific typed trees, with optional BO4E mapping via declarative TOML definitions.

**Architecture:** Three-layer design. Layer 1 (existing `edifact-parser`) tokenizes EDIFACT into `Vec<RawSegment>`. Layer 2 (new `mig-types` + `mig-assembly`) uses code-generated PID-specific Rust types — derived from MIG/AHB XMLs — as the intermediate representation, with a recursive descent assembler (parse) and tree walker (write). Layer 3 (new `mig-bo4e`) provides optional BO4E conversion via hybrid TOML declarative mappings + hand-coded complex logic. Roundtrip fidelity is structural (MIG tree ordering) rather than metadata-hacked.

**Tech Stack:** Rust 2021, quick-xml (MIG/AHB parsing — already exists), serde + toml (mapping files), edifact-parser (tokenization), existing `MigSchema`/`AhbSchema` types from `automapper-generator`

**Design Document:** [2026-02-20-mig-driven-mapping-design.md](../2026-02-20-mig-driven-mapping-design.md)

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | mig-types Crate & Shared Segment Codegen | New crate + generator backend that reads MIG XML and emits shared Rust types (segments, composites, enums, segment groups) | - |
| 2 | PID-Specific Composition Codegen | Generator backend that reads AHB XML and emits per-PID composition structs from shared building blocks | Epic 1 |
| 3 | mig-assembly Crate — Tree Assembler | Two-pass assembler: `Vec<RawSegment>` + MIG schema → typed PID tree | Epic 2 |
| 4 | mig-assembly — Tree Disassembler & Roundtrip | Tree walker that emits `Vec<RawSegment>` from typed tree + roundtrip validation | Epic 3 |
| 5 | mig-bo4e Crate — TOML Mapping Engine | Declarative TOML mapping loader + bidirectional MIG-tree ↔ BO4E engine | Epic 4 |
| 6 | Integration & Dual API | Wire into automapper-api, dual API endpoints, migration validation tests | Epic 5 |

---

## New Crate → Epic Mapping

| Crate | Epic(s) | Description |
|-------|---------|-------------|
| `mig-types` | 1, 2 | Generated PID-specific types + shared segment group types |
| `mig-assembly` | 3, 4 | Tree assembler (parse) + disassembler (write) |
| `mig-bo4e` | 5 | TOML mapping engine + hand-coded complex mappings |

## Existing Code Reused

| Component | Location | Used In |
|-----------|----------|---------|
| `MigSchema`, `MigSegment`, `MigSegmentGroup`, etc. | `automapper-generator::schema::mig` | Epics 1-4 |
| `AhbSchema`, `Pruefidentifikator`, `AhbFieldDefinition` | `automapper-generator::schema::ahb` | Epic 2 |
| `parse_mig()`, `parse_ahb()` | `automapper-generator::parsing` | Epics 1-2 |
| `extract_ordered_segments()` | `automapper-generator::codegen::segment_order` | Epic 4 |
| `edifact-parser` streaming parser | `edifact-parser` | Epic 3 |
| `RawSegment`, `EdifactDelimiters` | `edifact-types` | Epics 3-4 |

## Branch Naming

```
feat/mig-driven-mapping-E1   # Epic 1: mig-types + shared codegen
feat/mig-driven-mapping-E2   # Epic 2: PID-specific codegen
feat/mig-driven-mapping-E3   # Epic 3: tree assembler
feat/mig-driven-mapping-E4   # Epic 4: tree disassembler + roundtrip
feat/mig-driven-mapping-E5   # Epic 5: TOML mapping engine
feat/mig-driven-mapping-E6   # Epic 6: integration + dual API
```

## Files in This Plan

1. [Epic 1: mig-types Crate & Shared Segment Codegen](./epic-01-mig-types-shared-codegen.md)
2. [Epic 2: PID-Specific Composition Codegen](./epic-02-pid-specific-codegen.md)
3. [Epic 3: mig-assembly Crate — Tree Assembler](./epic-03-tree-assembler.md)
4. [Epic 4: mig-assembly — Tree Disassembler & Roundtrip](./epic-04-tree-disassembler-roundtrip.md)
5. [Epic 5: mig-bo4e Crate — TOML Mapping Engine](./epic-05-toml-mapping-engine.md)
6. [Epic 6: Integration & Dual API](./epic-06-integration-dual-api.md)

## Commands Reference

```bash
# Check entire workspace compiles
cargo check --workspace

# Run all tests
cargo test --workspace

# Run tests for new crates
cargo test -p mig-types
cargo test -p mig-assembly
cargo test -p mig-bo4e

# Regenerate all UTILMD FV2504 types (MIG + PID)
just generate-utilmd-fv2504

# Or manually:
# Generate shared MIG types (enums, composites, segments, groups)
cargo run -p automapper-generator -- generate-mig-types \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --message-type UTILMD --format-version FV2504 \
  --output-dir crates/mig-types/src/generated

# Generate per-PID composition types
cargo run -p automapper-generator -- generate-pid-types \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-path xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml \
  --message-type UTILMD --format-version FV2504 \
  --output-dir crates/mig-types/src/generated

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```
