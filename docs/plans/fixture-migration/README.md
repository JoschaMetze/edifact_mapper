# Fixture Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build three tools (`mig-diff`, `migrate-fixture`, `render-fixture`) that systematically migrate EDIFACT test fixture files across format versions, using PID schema diffs and BO4E round-trip bridging.

**Architecture:** Three-phase pipeline, all implemented as subcommands in `automapper-generator`. Phase 1: `mig-diff` compares two PID schema JSONs and produces a structured diff JSON. Phase 1b: `migrate-fixture` consumes old `.edi` + diff + new PID schema to produce a migrated `.edi` with warnings. Phase 3: `render-fixture` takes canonical `.mig.bo.json` and reverse-maps through version-specific TOML mappings to produce golden `.edi` fixtures. A shared `schema_diff` module contains the diffing logic, and a `fixture_gen` module handles EDIFACT segment synthesis.

**Tech Stack:** Rust 2021, serde + serde_json (schema parsing/diff output), clap (CLI), mig-assembly (tokenize, disassemble, render), mig-bo4e (reverse mapping engine), edifact-types (OwnedSegment, EdifactDelimiters)

**Design Document:** [2026-02-23-fixture-migration-design.md](../2026-02-23-fixture-migration-design.md)

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | PID Schema Diff Engine | Core diffing logic that compares two PID schema JSONs and produces a structured diff | - |
| 2 | `mig-diff` CLI Subcommand | CLI entry point for generating diff reports and human-readable AHB condition summaries | Epic 1 |
| 3 | Fixture Migrator Engine | Core logic to parse old `.edi`, apply diff rules, fill new segments from schema, emit warnings | Epic 1 |
| 4 | `migrate-fixture` CLI Subcommand | CLI entry point for migrating `.edi` fixtures between format versions | Epics 1, 3 |
| 5 | `render-fixture` CLI Subcommand | CLI entry point for rendering `.mig.bo.json` through TOML mappings to produce golden `.edi` fixtures | - |

---

## Crate → Epic Mapping

| Crate | Epic(s) | Description |
|-------|---------|-------------|
| `automapper-generator` | 1-5 | All new code: diff engine, migrator, renderer, CLI subcommands |
| `mig-assembly` | 5 | Consumed as library (tokenize, disassemble, render) |
| `mig-bo4e` | 5 | Consumed as library (MappingEngine reverse, envelope reconstruction) |
| `edifact-types` | 3, 5 | Consumed as library (EdifactDelimiters) |
| `mig-types` | 3 | Consumed as library (OwnedSegment) |

## Existing Code Reused

| Component | Location | Used In |
|-----------|----------|---------|
| PID schema JSON loading | `automapper-generator::codegen::pid_mapping_gen` (`load_pid_schema()`, `SchemaGroup`, `SchemaSegmentInfo`, `SchemaElementInfo`) | Epics 1, 3 |
| `parse_to_segments()` | `mig-assembly::tokenize` | Epic 3 |
| `Disassembler::disassemble()` | `mig-assembly::disassembler` | Epic 5 |
| `render_edifact()` | `mig-assembly::renderer` | Epics 3, 5 |
| `MappingEngine::map_interchange_reverse()` | `mig-bo4e::engine` | Epic 5 |
| `rebuild_unb/unh/unt/unz()` | `mig-bo4e::model` | Epic 5 |
| `EdifactDelimiters` | `edifact-types` | Epic 3 |
| `OwnedSegment` | `mig-types::segment` | Epic 3 |
| `parse_mig()`, `parse_ahb()` | `automapper-generator::parsing` | Epic 5 |
| `Pruefidentifikator::segment_numbers` | `automapper-generator::schema::ahb` | Epic 5 |
| CLI `Commands` enum | `automapper-generator::main.rs` | Epics 2, 4, 5 |

## Commands Reference

```bash
# Check entire workspace compiles
cargo check --workspace

# Run tests for automapper-generator
cargo test -p automapper-generator

# Run a single test
cargo test -p automapper-generator test_diff_identical_schemas

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Files in This Plan

1. [Epic 1: PID Schema Diff Engine](./epic-01-pid-schema-diff-engine.md)
2. [Epic 2: `mig-diff` CLI Subcommand](./epic-02-mig-diff-cli.md)
3. [Epic 3: Fixture Migrator Engine](./epic-03-fixture-migrator-engine.md)
4. [Epic 4: `migrate-fixture` CLI Subcommand](./epic-04-migrate-fixture-cli.md)
5. [Epic 5: `render-fixture` CLI Subcommand](./epic-05-render-fixture-cli.md)
