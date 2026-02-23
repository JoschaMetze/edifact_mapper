# Reverse Endpoint Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `POST /api/v2/reverse` endpoint that accepts BO4E JSON (at interchange, nachricht, or transaktion level) and converts it back to an EDIFACT string, completing the bidirectional MIG-driven pipeline.

**Architecture:** Three-layer extension. Layer 1 (`mig-bo4e::engine`) adds `map_all_reverse()` (reverse all definitions in an engine) and `map_interchange_reverse()` (two-pass reverse mapping mirroring the forward `map_interchange()`). Layer 2 (`mig-bo4e::model`) adds envelope reconstruction helpers (UNB/UNH/UNT/UNZ segment builders). Layer 3 (`automapper-api`) adds the `/reverse` endpoint with `ReverseRequest`, `InputLevel`, `ReverseMode` contracts and a handler that orchestrates the full reverse pipeline through disassembly and rendering.

**Tech Stack:** Rust 2021, serde + serde_json (serialization), mig-assembly (disassembly, rendering), mig-bo4e (TOML mapping engine), automapper-api (Axum REST)

**Design Document:** [2026-02-23-reverse-endpoint-design.md](../2026-02-23-reverse-endpoint-design.md)

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | Engine Reverse Methods | Add `map_all_reverse()` and `map_interchange_reverse()` to `MappingEngine` | - |
| 2 | Envelope Reconstruction | Build OwnedSegment helpers for UNB, UNH, UNT, UNZ from JSON/parameters | - |
| 3 | API Contracts & Handler | Add reverse endpoint contracts, handler, and route registration | Epics 1-2 |
| 4 | Roundtrip Integration Test | Full EDIFACT → forward → reverse → EDIFACT byte comparison via API | Epics 1-3 |

---

## Crate → Epic Mapping

| Crate | Epic(s) | Description |
|-------|---------|-------------|
| `mig-bo4e` | 1, 2 | Reverse mapping methods + envelope reconstruction |
| `automapper-api` | 3, 4 | API contracts, handler, route, integration tests |

## Existing Code Reused

| Component | Location | Used In |
|-----------|----------|---------|
| `MappingEngine::map_reverse()` | `mig-bo4e::engine` (line 345) | Epic 1 |
| `MappingEngine::map_all_forward()` | `mig-bo4e::engine` (line 729) | Epic 1 (pattern reference) |
| `MappingEngine::map_interchange()` | `mig-bo4e::engine` (line 817) | Epic 1 (pattern reference) |
| `Interchange`, `Nachricht`, `Transaktion` | `mig-bo4e::model` | Epics 1-3 |
| `extract_nachrichtendaten()` | `mig-bo4e::model` (line 79) | Epic 2 (inverse) |
| `Disassembler::disassemble()` | `mig-assembly::disassembler` | Epic 3 |
| `render_edifact()` | `mig-assembly::renderer` | Epic 3 |
| `ConvertV2Response` | `automapper-api::contracts::convert_v2` | Epic 3 |
| `MigServiceRegistry` | `automapper-api::state` | Epic 3 |
| `convert_v2` handler | `automapper-api::routes::convert_v2` | Epic 3 (pattern reference) |

## Commands Reference

```bash
# Check entire workspace compiles
cargo check --workspace

# Run tests for affected crates
cargo test -p mig-bo4e
cargo test -p automapper-api

# Run all tests
cargo test --workspace

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Files in This Plan

1. [Epic 1: Engine Reverse Methods](./epic-01-engine-reverse-methods.md)
2. [Epic 2: Envelope Reconstruction](./epic-02-envelope-reconstruction.md)
3. [Epic 3: API Contracts & Handler](./epic-03-api-contracts-handler.md)
4. [Epic 4: Roundtrip Integration Test](./epic-04-roundtrip-integration-test.md)
