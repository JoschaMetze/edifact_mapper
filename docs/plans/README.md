# edifact_mapper — Implementation Plans

## Overview

Full Rust port of the C# edifact_bo4e_automapper. Six features, 26 epics.

**Design Document:** [2026-02-18-rust-port-design.md](./2026-02-18-rust-port-design.md)

**C# Reference Repo:** `../edifact_bo4e_automapper/` (at commit cee0b09)

---

## Feature Overview

| # | Feature | Epics | Description | Depends On |
|---|---------|-------|-------------|------------|
| 1 | [edifact-core](./edifact-core-implementation/) | 8 | Workspace, parser, domain types, core mapper engine | - |
| 2 | [validation](./validation-implementation/) | 3 | AHB condition parser, evaluator, validator | Feature 1 |
| 3 | [generator](./generator-implementation/) | 3 | MIG/AHB XML parsing, Rust codegen, Claude CLI | Feature 1 |
| 4 | [web-stack](./web-stack-implementation/) | 3 | Axum REST API, tonic gRPC, Leptos frontend | Feature 1 |
| 5 | [missing-entity-mappers](./missing-entity-mappers/) | 3 | 7 remaining entity mappers/writers for complete roundtrip | Feature 1 |
| 6 | [mig-driven-mapping](./mig-driven-mapping/) | 6 | MIG-driven pipeline: typed trees, TOML mapping, dual API | Features 1, 3 |

## Dependency Graph

```
Feature 1: edifact-core-implementation
    │
    ├──→ Feature 2: validation-implementation     ┐
    ├──→ Feature 3: generator-implementation       ├── parallel
    └──→ Feature 4: web-stack-implementation       ┘
```

Features 2, 3, and 4 are fully independent and can be developed in parallel once Feature 1 is complete.
Feature 6 depends on Features 1 and 3 (uses automapper-generator for MIG parsing).

---

## Crate → Feature Mapping

| Crate | Feature | Epic(s) |
|-------|---------|---------|
| `edifact-types` | 1 | Epic 2 |
| `edifact-parser` | 1 | Epics 3-4 |
| `bo4e-extensions` | 1 | Epic 5 |
| `automapper-core` | 1 | Epics 6-8 |
| `automapper-validation` | 2 | Epics 1-3 |
| `automapper-generator` | 3 | Epics 1-3 |
| `automapper-api` | 4, 6 | Epics 1-2, Epic 6 |
| `automapper-web` | 4 | Epic 3 |
| `mig-types` | 6 | Epics 1-2 |
| `mig-assembly` | 6 | Epics 3-4, 6 |
| `mig-bo4e` | 6 | Epics 5-6 |

## Branch Naming

```
feat/edifact-core-implementation-E1   # Feature 1, Epic 1
feat/edifact-core-implementation-E2   # Feature 1, Epic 2
feat/validation-implementation-E1     # Feature 2, Epic 1
feat/generator-implementation-E1      # Feature 3, Epic 1
feat/web-stack-implementation-E1      # Feature 4, Epic 1
```

## Commands Reference

```bash
# Check all crates compile
cargo check --workspace

# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p edifact-types
cargo test -p edifact-parser
cargo test -p bo4e-extensions
cargo test -p automapper-core

# Run benchmarks
cargo bench -p edifact-parser

# Format
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Build release
cargo build --release --workspace
```
