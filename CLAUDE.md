# CLAUDE.md — Project Conventions for edifact-bo4e-automapper

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust port of the C# [edifact_bo4e_automapper](https://github.com/Hochfrequenz/edifact_bo4e_automapper) — bidirectional EDIFACT ↔ BO4E conversion for the German energy market. Goals: batch processing millions of messages with zero-copy parsing, and publishing reusable Rust crates.

C# reference repo: `../edifact_bo4e_automapper/` (commit cee0b09)

## Workspace Structure

11 crates in dependency order:
1. `edifact-types` — zero-dep EDIFACT primitives
2. `edifact-parser` — standalone streaming parser (publishable)
3. `bo4e-extensions` — BO4E companion types for EDIFACT domain data
4. `automapper-core` — coordinators, mappers, builders, writers (legacy pipeline)
5. `automapper-validation` — AHB condition parser/evaluator
6. `automapper-generator` — CLI code generator from MIG/AHB XML
7. `mig-types` — generated typed MIG-tree types (segments, composites, enums, PIDs)
8. `mig-assembly` — MIG-guided EDIFACT tree assembly/disassembly, ConversionService
9. `mig-bo4e` — declarative TOML-based MIG-tree to BO4E mapping engine
10. `automapper-api` — Axum REST + tonic gRPC server (dual API: v1 legacy + v2 MIG-driven)
11. `automapper-web` — Leptos WASM frontend

## Commands

```bash
cargo check --workspace          # Type-check everything
cargo test --workspace           # Run all tests
cargo test -p <crate>            # Run tests for one crate
cargo test -p edifact-parser test_una_detection  # Run a single test
cargo clippy --workspace -- -D warnings  # Lint (warnings are errors)
cargo fmt --all -- --check       # Format check
cargo fmt --all                  # Auto-format
cargo bench -p automapper-core   # Legacy pipeline benchmarks
cargo bench -p mig-assembly      # MIG-driven pipeline benchmarks
cargo build --release --workspace
```

## Architecture

Eleven-crate Cargo workspace under `crates/`, ordered by dependency:

```
edifact-types          Zero-copy EDIFACT primitives (RawSegment<'a>, EdifactDelimiters)
    ↓
edifact-parser         SAX-style streaming parser, EdifactHandler trait, UNA detection
    ↓
bo4e-extensions        WithValidity<T,E> wrapper, *Edifact companion types, LinkRegistry
    ↓                  (depends on external bo4e-rust crate for standard BO4E types)
automapper-core        Coordinators, entity mappers, builders, writers, batch (rayon) [LEGACY]
automapper-generator   CLI: MIG/AHB XML → Rust codegen, claude CLI for conditions
    ↓
mig-types              Generated typed MIG-tree types (segments, composites, enums, PIDs)
mig-assembly           MIG-guided tree assembly/disassembly, ConversionService [NEW]
    ↓
mig-bo4e               TOML-based MIG-tree → BO4E mapping engine [NEW]
    ↓
├── automapper-validation   AHB condition parser/evaluator, EdifactValidator
└── automapper-api          Axum REST + tonic gRPC (dual API: v1 legacy + v2 MIG-driven)
        ↓
    automapper-web          Leptos WASM frontend (served as static files by api)
```

### Key Patterns

**Streaming parser**: `EdifactStreamParser::parse(input, handler)` emits `RawSegment<'a>` references borrowing from the input buffer. Handlers implement `EdifactHandler` trait (on_interchange_start, on_message_start, on_segment, etc.) and return `Control::Continue` or `Control::Stop`.

**Coordinator → Mapper → Builder flow**: `UtilmdCoordinator<V: VersionConfig>` implements `EdifactHandler`, routes segments to entity-specific mappers (MarktlokationMapper, ZaehlerMapper, etc.). Each mapper implements `SegmentHandler` + `EntityWriter` for bidirectional conversion. Builders accumulate state across segments.

**Format version dispatch**: Compile-time generics in the hot path (`VersionConfig` trait with associated types for each mapper), runtime `FormatVersion` enum at the entry point. `create_coordinator(fv)` returns `Box<dyn Coordinator>`.

**Companion types**: `*Edifact` structs (e.g. `MarktlokationEdifact`) store functional domain data that exists in EDIFACT but not in standard BO4E (data quality, cross-references, qualifiers). They do NOT store transport/ordering data — roundtrip ordering is handled by deterministic MIG-derived rules in writers.

**WithValidity<T, E>**: Wraps a standard BO4E object (`T`) with its EDIFACT companion (`E`), a validity period (`Zeitraum`), and optional Zeitscheibe reference.

### MIG-Driven Pipeline (New)

Alternative to `automapper-core`, data-driven from MIG XML schemas rather than hand-coded mappers:

```
EDIFACT bytes → parse_to_segments() → Vec<OwnedSegment>
  → Assembler::assemble_generic() → AssembledTree
  → MappingEngine (TOML definitions) → BO4E JSON
  → MappingEngine (reverse) → AssembledTree
  → Disassembler::disassemble() → Vec<DisassembledSegment>
  → render_edifact() → EDIFACT string
```

**ConversionService** (`mig-assembly::service`): High-level facade that loads a MIG XML, tokenizes EDIFACT, and assembles into `AssembledTree` or JSON. Used by the v2 API.

**MappingEngine** (`mig-bo4e::engine`): Loads declarative TOML mapping files to convert between `AssembledTree` and BO4E JSON. Supports forward (tree→BO4E) and reverse (BO4E→tree) mappings with `HandlerRegistry` for complex cases.

**Dual API** (`automapper-api`): `POST /api/v2/convert` accepts `mode` parameter:
- `mig-tree` — returns the MIG-assembled tree as JSON
- `bo4e` — assembles + applies TOML mappings → BO4E JSON
- `legacy` — uses the `automapper-core` pipeline (backward compatible)

## Coding Conventions

- **Error handling**: `thiserror` in all library crates. No `anyhow` in library code.
- **Serialization**: All domain types derive `Serialize, Deserialize`.
- **Lifetimes**: `RawSegment<'a>` borrows from input buffer. Zero-copy hot path.
- **Testing**: TDD — write failing test first, then implement. Use `#[cfg(test)]` modules.
- **Naming**: Rust snake_case for fields/functions. German domain terms preserved (Marktlokation, Zeitscheibe, Geschaeftspartner).
- **Format versions**: `FV2504`, `FV2510` marker types. `VersionConfig` trait for compile-time dispatch.
- **Commits**: Conventional commits (`feat`, `fix`, `refactor`, `test`, `docs`). Include `Co-Authored-By` trailer.
- **`edifact-parser` is standalone**: No BO4E dependency — publishable as a generic EDIFACT parser crate.
- **Generated code**: Output of `automapper-generator` goes to `generated/` and is committed (no build-time codegen).

## Architecture Decisions

- Parser is SAX-style streaming (matches C# EdifactStreamParser)
- Handler trait has default no-op methods — implementors override what they need
- Coordinator routes segments to registered mappers
- Each mapper handles specific segment qualifiers (LOC+Z16 -> MarktlokationMapper)
- Writers reverse the mapping (domain -> EDIFACT segments)
- Roundtrip fidelity: parse -> map -> write must produce byte-identical output

## Test Data

- `example_market_communication_bo4e_transactions/` — real EDIFACT fixture files (submodule)
- `xml-migs-and-ahbs/` — MIG/AHB XML schemas (submodule)
- `stammdatenmodell/` — BO4E data model reference (submodule)
- `tests/fixtures/` — symlinks to submodule data

## Dependencies on C# Reference

Reference C# repo: see design doc for architectural mapping.
Key correspondences:
- `EdifactStreamParser.cs` -> `edifact-parser` crate
- `CoordinatorBase.cs` -> `automapper-core::coordinator` module
- `ISegmentHandler.cs` -> `automapper-core::traits::SegmentHandler`
- `IBuilder.cs` -> `automapper-core::traits::Builder`
- `IEntityWriter.cs` -> `automapper-core::traits::EntityWriter`
- `MarktlokationMapper.cs` -> `automapper-core::mappers::marktlokation`
- `MarktlokationWriter.cs` -> `automapper-core::writers::marktlokation`

## Implementation Status

5 features (23 epics) implemented. ~17,000 LOC, 350+ tests.

| Crate | Tests | Notes |
|-------|-------|-------|
| edifact-types | 29 | Delimiter parsing, segment construction |
| edifact-parser | 37 | Tokenizer, UNA detection, property tests (proptest) |
| bo4e-extensions | 29 | WithValidity, LinkRegistry, companion types |
| automapper-core | 94 | 8 entity mappers, writers, roundtrip, batch |
| automapper-validation | 143 | Condition parser, evaluator, validator |
| automapper-generator | 1 | Snapshot tests (insta) |
| mig-types | 6 | Generated typed MIG-tree types |
| mig-assembly | 40+ | Assembler, disassembler, roundtrip, ConversionService |
| mig-bo4e | 27+ | TOML mapping engine, migration comparison tests |
| automapper-api | 12 | REST v1/v2, gRPC, integration tests |
| automapper-web | 0 | WASM components |

### Implementation Learnings

- **Snapshot testing with insta** works well for codegen output — use `cargo insta test` then `cargo insta review`.
- **Property testing with proptest** catches edge cases in tokenizer/delimiter parsing that unit tests miss.
- **Parameterized tests with test-case** reduce boilerplate for mapper tests with multiple segment qualifiers.
- **Entity mappers follow a consistent pattern**: implement `SegmentHandler` + `Builder<T>` for forward mapping, `EntityWriter` for reverse. New mappers can copy an existing one as a template.
- **Writer segment ordering** is deterministic from MIG rules — no need to store ordering metadata on companion types.
- **AHB condition parser** uses recursive descent with Unicode operators (`∧`, `∨`, `⊻`, `¬`). Three-valued logic (True/False/Unknown) handles missing data gracefully.
- **gRPC streaming** uses tonic's `Streaming<T>` for both request and response sides. Proto files live in `proto/`.
- **Leptos WASM frontend** communicates via REST to the Axum backend. The `static/` directory holds build output served as a fallback route.

## Implementation Plans

Detailed task-level plans in `docs/plans/` — 5 features, 23 epics:
- Feature 1: `edifact-core-implementation/` (8 epics) — foundation
- Feature 2: `validation-implementation/` (3 epics) — AHB conditions & validation
- Feature 3: `generator-implementation/` (3 epics) — MIG/AHB XML codegen
- Feature 4: `web-stack-implementation/` (3 epics) — REST, gRPC, WASM frontend
- Feature 5: `mig-driven-mapping/` (6 epics) — MIG-driven pipeline, typed trees, TOML mapping, dual API

Design document: `docs/plans/2026-02-18-rust-port-design.md`
