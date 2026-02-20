# Missing Entity Mappers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the 7 missing entity mappers and writers (SteuerbareRessource, TechnischeRessource, Tranche, MabisZaehlpunkt, Bilanzierung, Produktpaket, Lokationszuordnung) so that the UTILMD roundtrip is complete for all entity types in `UtilmdTransaktion`.

**Architecture:** Each mapper implements `SegmentHandler` + `Builder<T>` following existing patterns. LOC-based entities (4) follow the `NetzlokationMapper` template. SEQ-based entities (3) follow the `VertragMapper`/`ZaehlerMapper` template with context tracking. Writers are added to `entity_writers.rs` and integrated into the coordinator's `write_transaction()` method in MIG Counter/Nr order.

**Tech Stack:** Rust, edifact-types, edifact-parser, bo4e-extensions, automapper-core traits, TDD

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | LOC-Based Entity Mappers | SteuerbareRessource (Z19), TechnischeRessource (Z20), Tranche (Z21), MabisZaehlpunkt (Z15) | - |
| 2 | SEQ-Based Entity Mappers | Produktpaket (Z79), Lokationszuordnung (Z78), Bilanzierung (Z98/Z81) | - |
| 3 | Coordinator Integration & Roundtrip Tests | Register all 7 mappers in coordinator, update write_transaction ordering, extend roundtrip tests | Epic 1, Epic 2 |

---

## Files in This Plan

1. [Epic 1: LOC-Based Entity Mappers](./epic-01-loc-based-mappers.md)
2. [Epic 2: SEQ-Based Entity Mappers](./epic-02-seq-based-mappers.md)
3. [Epic 3: Coordinator Integration & Roundtrip Tests](./epic-03-coordinator-integration.md)

---

## Existing Pattern Reference

**LOC-based mapper template** (`crates/automapper-core/src/mappers/netzlokation.rs`):
- `can_handle`: check `segment.id == "LOC" && segment.get_element(0) == "<qualifier>"`
- `handle`: extract ID from `segment.get_component(1, 0)`
- `Builder<Option<WithValidity<T, E>>>`: return Some if has_data, None otherwise

**SEQ-based mapper template** (`crates/automapper-core/src/mappers/vertrag.rs`):
- `can_handle`: check SEQ + context-dependent subordinate segments
- `handle`: on SEQ set context flag, on subordinate segments extract data
- `Builder<Option<WithValidity<T, E>>>` or `Builder<Vec<WithValidity<T, E>>>`

**Writer template** (`crates/automapper-core/src/writer/entity_writers.rs`):
- Struct with `write(doc: &mut EdifactDocumentWriter, entity: &WithValidity<T, E>)` method

## MIG Segment Ordering (for reference)

**SG5 LOC qualifiers (Counter=0320):**
```
Nr 00048 | Z18 | Netzlokation          ← existing
Nr 00049 | Z16 | Marktlokation         ← existing
Nr 00051 | Z20 | Technische Ressource  ← NEW
Nr 00052 | Z19 | Steuerbare Ressource  ← NEW
Nr 00053 | Z21 | Tranche               ← NEW
Nr 00054 | Z17 | Messlokation          ← existing
Nr 00055 | Z15 | MaBiS-Zählpunkt       ← NEW
```

**SG8 SEQ qualifiers (Counter=0410, relevant subset):**
```
Nr 00074 | Z78 | Lokationszuordnung    ← NEW
Nr 00081 | Z79 | Produktpaket          ← NEW
Nr 00278 | Z62 | Steuerbare Ressource data (SEQ context)
Nr 00291 | Z18 | Vertrag               ← existing
Nr 00311 | Z03 | Zaehler               ← existing
```

**Bilanzierung** maps to SEQ+Z98/Z81 with CCI+Z20 and QTY segments.

## Commands Reference

```bash
cargo check -p automapper-core          # Type-check
cargo test -p automapper-core            # Run all tests
cargo test -p automapper-core --test roundtrip_bo4e_test  # Roundtrip tests
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all -- --check               # Format check
cargo fmt --all                          # Auto-format
cargo test --workspace                   # Full workspace
```
