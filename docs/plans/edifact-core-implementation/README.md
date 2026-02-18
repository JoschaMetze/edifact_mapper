# Edifact Core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the complete EDIFACT parsing, BO4E mapping, and EDIFACT generation pipeline as a Rust workspace, porting the C# edifact_bo4e_automapper core functionality with zero-copy parsing, streaming handler architecture, and bidirectional UTILMD mapping.

**Architecture:** SAX-style streaming parser emits `RawSegment` references that borrow directly from the input buffer. A `UtilmdCoordinator` implements `EdifactHandler` and routes segments to entity-specific mappers (Marktlokation, Zaehler, Geschaeftspartner, etc.) that accumulate state via builders. Writers serialize domain objects back to EDIFACT segments for roundtrip fidelity. Format version dispatch uses trait-based generics with an enum boundary at the runtime entry point.

**Tech Stack:** Rust 2021 edition, Cargo workspace (8 crates), serde + serde_json, chrono, thiserror, rayon, proptest, criterion, insta, test-case

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | Cargo Workspace & Project Setup | Initialize git repo, Cargo workspace, all 8 crate stubs, .gitignore, rustfmt.toml, clippy.toml, git submodules, CLAUDE.md | - |
| 2 | edifact-types Crate | EdifactDelimiters with defaults and UNA parsing, SegmentPosition, RawSegment zero-copy type, Control enum | Epic 1 |
| 3 | edifact-parser: Tokenizer & UNA | UNA header detection, segment tokenizer with release char handling, element/component splitting | Epic 2 |
| 4 | edifact-parser: Streaming Parser & Handler | EdifactHandler trait, EdifactStreamParser::parse(), service segment routing, ParseError, integration tests, proptest fuzzing | Epic 3 |
| 5 | bo4e-extensions Crate | WithValidity wrapper, Zeitraum, all Edifact companion types, DataQuality, UtilmdNachricht/Transaktion containers, Bo4eUri, LinkRegistry | Epic 1 |
| 6 | automapper-core: Traits & Version Dispatch | SegmentHandler, Builder, EntityWriter, Mapper traits, FormatVersion enum, VersionConfig trait, TransactionContext, Coordinator trait, AutomapperError | Epic 4, Epic 5 |
| 7 | automapper-core: UTILMD Forward Mapping | UtilmdCoordinator, MarktlokationMapper, MesslokationMapper, NetzlokationMapper, ZaehlerMapper, GeschaeftspartnerMapper, VertragMapper, ProzessdatenMapper, ZeitscheibeMapper | Epic 6 |
| 8 | automapper-core: Writer, Roundtrip & Batch | EdifactSegmentWriter, EdifactDocumentWriter, entity writers, roundtrip integration tests, convert_batch with rayon, criterion benchmarks | Epic 7 |

---

## Files in This Plan

1. [Epic 1: Cargo Workspace & Project Setup](./epic-01-cargo-workspace-setup.md)
2. [Epic 2: edifact-types Crate](./epic-02-edifact-types.md)
3. [Epic 3: edifact-parser Tokenizer & UNA](./epic-03-parser-tokenizer.md)
4. [Epic 4: edifact-parser Streaming Parser & Handler](./epic-04-parser-streaming.md)
5. [Epic 5: bo4e-extensions Crate](./epic-05-bo4e-extensions.md)
6. [Epic 6: automapper-core Traits & Version Dispatch](./epic-06-core-traits-version-dispatch.md)
7. [Epic 7: automapper-core UTILMD Forward Mapping](./epic-07-utilmd-forward-mapping.md)
8. [Epic 8: automapper-core Writer, Roundtrip & Batch](./epic-08-writer-roundtrip-batch.md)

---

## Test Strategy

- **Unit tests**: `#[cfg(test)]` modules in each crate, TDD red-green-refactor cycle
- **Integration tests**: `tests/` directory with real EDIFACT fixture files from `example_market_communication_bo4e_transactions/` submodule
- **Snapshot tests**: `insta` for JSON serialization output of parsed transactions
- **Property-based tests**: `proptest` for parser robustness (arbitrary byte input must not panic)
- **Parameterized tests**: `test-case` for running the same test over multiple fixture files
- **Roundtrip tests**: parse EDIFACT -> build domain objects -> generate EDIFACT -> compare byte-identical
- **Benchmarks**: `criterion` for parser throughput (bytes/sec) and batch conversion speed

## Commands Reference

```bash
# Check entire workspace compiles
cargo check --workspace

# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p edifact-types
cargo test -p edifact-parser
cargo test -p bo4e-extensions
cargo test -p automapper-core

# Run a specific test
cargo test -p edifact-types test_default_delimiters

# Run clippy lints
cargo clippy --workspace -- -D warnings

# Run formatter
cargo fmt --all -- --check

# Run benchmarks
cargo bench -p automapper-core

# Build release
cargo build --release --workspace
```
