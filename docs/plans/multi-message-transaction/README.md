# Multi-Message & Transaction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add multi-message interchange and multi-transaction support to the MIG-driven pipeline, producing a typed `Interchange → Nachricht → Transaktion` hierarchy from EDIFACT files containing one or more UNH/UNT messages, each with one or more transaction groups (SG4 in UTILMD).

**Architecture:** Three-layer extension. Layer 1 (`mig-assembly::tokenize`) splits a flat segment list at UNH/UNT boundaries into per-message chunks. Layer 2 (`mig-bo4e::model`) defines typed output structs (`Interchange`, `Nachricht`, `Transaktion`). Layer 3 (`mig-bo4e::engine`) scopes the existing mapping logic to sub-trees — message-level definitions run against the full tree, transaction-level definitions run against each SG4 instance. The API response changes from a flat object to the hierarchical format.

**Tech Stack:** Rust 2021, serde + serde_json (serialization), mig-assembly (tokenization, assembly), mig-bo4e (TOML mapping engine), automapper-api (Axum REST)

**Design Document:** [2026-02-23-multi-message-transaction-design.md](../2026-02-23-multi-message-transaction-design.md)

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | Segment Splitting | Split `Vec<OwnedSegment>` at UNH/UNT boundaries into per-message chunks | - |
| 2 | Interchange Data Model | Define `Interchange`, `Nachricht`, `Transaktion` typed structs in mig-bo4e | - |
| 3 | Transaction Group Scoping | Extract sub-trees from `AssembledTree` for per-transaction mapping | Epic 1 |
| 4 | TOML Directory Reorganization | Move TOMLs into `message/` and `{pid}/` subdirectories, update loader | Epic 2 |
| 5 | API Integration | Wire up `map_interchange()`, update v2 response format, ConversionService | Epics 1-4 |

---

## Crate → Epic Mapping

| Crate | Epic(s) | Description |
|-------|---------|-------------|
| `mig-assembly` | 1 | `split_messages()` function + `InterchangeChunks` types |
| `mig-bo4e` | 2, 3, 4 | Data model + sub-tree scoping + TOML directory loader |
| `automapper-api` | 5 | v2 response format + ConversionService update |

## Existing Code Reused

| Component | Location | Used In |
|-----------|----------|---------|
| `parse_to_segments()` | `mig-assembly::tokenize` | Epic 1 |
| `Assembler::assemble_generic()` | `mig-assembly::assembler` | Epic 3 |
| `AssembledTree`, `AssembledGroup`, `AssembledGroupInstance` | `mig-assembly::assembler` | Epics 2-3 |
| `MappingEngine::map_all_forward()`, `map_forward()` | `mig-bo4e::engine` | Epic 3 |
| `MappingEngine::load()` | `mig-bo4e::engine` | Epic 4 |
| `ConversionService` | `mig-assembly::service` | Epic 5 |
| `convert_v2` handler | `automapper-api::routes::convert_v2` | Epic 5 |

## Branch Naming

```
feat/multi-message-E1   # Epic 1: segment splitting
feat/multi-message-E2   # Epic 2: interchange data model
feat/multi-message-E3   # Epic 3: transaction group scoping
feat/multi-message-E4   # Epic 4: TOML directory reorganization
feat/multi-message-E5   # Epic 5: API integration
```

## Files in This Plan

1. [Epic 1: Segment Splitting](./epic-01-segment-splitting.md)
2. [Epic 2: Interchange Data Model](./epic-02-interchange-data-model.md)
3. [Epic 3: Transaction Group Scoping](./epic-03-transaction-group-scoping.md)
4. [Epic 4: TOML Directory Reorganization](./epic-04-toml-directory-reorganization.md)
5. [Epic 5: API Integration](./epic-05-api-integration.md)

## Commands Reference

```bash
# Check entire workspace compiles
cargo check --workspace

# Run tests for affected crates
cargo test -p mig-assembly
cargo test -p mig-bo4e
cargo test -p automapper-api

# Run all tests
cargo test --workspace

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```
