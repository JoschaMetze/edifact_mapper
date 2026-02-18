---
feature: edifact-core-implementation
epic: 1
title: "Cargo Workspace & Project Setup"
depends_on: []
estimated_tasks: 5
crate: workspace
---

# Epic 1: Cargo Workspace & Project Setup

> **For Claude:** Implement each task in sequence. Each task has numbered steps — execute them in order. Every step that says "Run" means execute the command and verify the output matches. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Initialize the complete Cargo workspace with all 8 crate stubs, git repository, configuration files, submodules, and CLAUDE.md so that `cargo check --workspace` passes on an empty but structurally valid project.

**Architecture:** Cargo workspace at the root with 8 member crates under `crates/`. Each crate has a `Cargo.toml` and `src/lib.rs` stub. The workspace uses a shared dependency table (`[workspace.dependencies]`) for version consistency. Git submodules link to external XML schemas and test data.

**Tech Stack:** Rust 2021 edition, Cargo workspace, git submodules

---

## Task 1: Initialize Git Repository and Workspace Root

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `.gitignore`
- Create: `rustfmt.toml`
- Create: `clippy.toml`

**Step 1: Write the failing test**

No test for this task — this is project scaffolding. The validation is `cargo check --workspace`.

**Step 2: Create workspace root Cargo.toml**

Create `Cargo.toml` at the workspace root:

```toml
[workspace]
resolver = "2"
members = [
    "crates/edifact-types",
    "crates/edifact-parser",
    "crates/bo4e-extensions",
    "crates/automapper-core",
    "crates/automapper-validation",
    "crates/automapper-generator",
    "crates/automapper-api",
    "crates/automapper-web",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"
repository = "https://github.com/your-org/edifact-bo4e-automapper"

[workspace.dependencies]
# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "2"

# Parallelism
rayon = "1"

# CLI
clap = { version = "4", features = ["derive"] }

# XML
quick-xml = "0.37"

# Web
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tonic = "0.12"
prost = "0.13"

# Frontend
leptos = "0.7"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Testing
insta = { version = "1", features = ["json"] }
proptest = "1"
criterion = { version = "0.5", features = ["html_reports"] }
test-case = "3"

# Internal crates
edifact-types = { path = "crates/edifact-types" }
edifact-parser = { path = "crates/edifact-parser" }
bo4e-extensions = { path = "crates/bo4e-extensions" }
automapper-core = { path = "crates/automapper-core" }
automapper-validation = { path = "crates/automapper-validation" }
automapper-generator = { path = "crates/automapper-generator" }
automapper-api = { path = "crates/automapper-api" }
automapper-web = { path = "crates/automapper-web" }
```

**Step 3: Create .gitignore**

Create `.gitignore`:

```gitignore
# Rust build artifacts
/target/
**/*.rs.bk

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local

# Profiling
perf.data
perf.data.old
flamegraph.svg
```

**Step 4: Create rustfmt.toml**

Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

**Step 5: Create clippy.toml**

Create `clippy.toml`:

```toml
too-many-arguments-threshold = 8
type-complexity-threshold = 350
```

**Step 6: Verify structure**

Run: `ls -la Cargo.toml .gitignore rustfmt.toml clippy.toml`
Expected: All four files exist.

---

## Task 2: Create All 8 Crate Stubs

**Files:**
- Create: `crates/edifact-types/Cargo.toml`
- Create: `crates/edifact-types/src/lib.rs`
- Create: `crates/edifact-parser/Cargo.toml`
- Create: `crates/edifact-parser/src/lib.rs`
- Create: `crates/bo4e-extensions/Cargo.toml`
- Create: `crates/bo4e-extensions/src/lib.rs`
- Create: `crates/automapper-core/Cargo.toml`
- Create: `crates/automapper-core/src/lib.rs`
- Create: `crates/automapper-validation/Cargo.toml`
- Create: `crates/automapper-validation/src/lib.rs`
- Create: `crates/automapper-generator/Cargo.toml`
- Create: `crates/automapper-generator/src/lib.rs`
- Create: `crates/automapper-api/Cargo.toml`
- Create: `crates/automapper-api/src/lib.rs`
- Create: `crates/automapper-web/Cargo.toml`
- Create: `crates/automapper-web/src/lib.rs`

**Step 1: Create edifact-types crate**

Create `crates/edifact-types/Cargo.toml`:

```toml
[package]
name = "edifact-types"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Shared EDIFACT primitive types — zero dependencies"

[dependencies]
# No dependencies — this is the leaf crate

[dev-dependencies]
```

Create `crates/edifact-types/src/lib.rs`:

```rust
//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.
//!
//! # Types
//!
//! - [`EdifactDelimiters`] — the six delimiter characters (component, element, decimal, release, segment, reserved)
//! - [`SegmentPosition`] — byte offset and segment/message numbering
//! - [`RawSegment`] — zero-copy parsed segment borrowing from the input buffer
//! - [`Control`] — handler flow control (Continue / Stop)
```

**Step 2: Create edifact-parser crate**

Create `crates/edifact-parser/Cargo.toml`:

```toml
[package]
name = "edifact-parser"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Streaming EDIFACT tokenizer and SAX-style parser — standalone, no BO4E dependency"

[dependencies]
edifact-types.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest.workspace = true
test-case.workspace = true
```

Create `crates/edifact-parser/src/lib.rs`:

```rust
//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.
//!
//! This crate provides a standalone EDIFACT parser with no BO4E dependency.
//! It can be used by anyone in the Rust ecosystem for generic EDIFACT parsing.
//!
//! # Architecture
//!
//! The parser uses a SAX-style streaming model:
//! 1. Tokenizer splits raw bytes into segments
//! 2. Parser routes segments to handler callbacks
//! 3. Handler accumulates state as needed
```

**Step 3: Create bo4e-extensions crate**

Create `crates/bo4e-extensions/Cargo.toml`:

```toml
[package]
name = "bo4e-extensions"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "BO4E extension types for EDIFACT mapping — companion structs, WithValidity, LinkRegistry"

[dependencies]
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true

[dev-dependencies]
insta.workspace = true
```

Create `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.
//!
//! Bridges the standard BO4E types with EDIFACT-specific functional domain data.
//! Provides `WithValidity<T, E>` wrappers, companion `*Edifact` structs,
//! and container types like `UtilmdTransaktion`.
```

**Step 4: Create automapper-core crate**

Create `crates/automapper-core/Cargo.toml`:

```toml
[package]
name = "automapper-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Core automapper — coordinators, mappers, builders, writers for EDIFACT/BO4E conversion"

[dependencies]
edifact-types.workspace = true
edifact-parser.workspace = true
bo4e-extensions.workspace = true
thiserror.workspace = true
rayon.workspace = true

[dev-dependencies]
insta.workspace = true
test-case.workspace = true
criterion.workspace = true
serde_json.workspace = true

[[bench]]
name = "parser_throughput"
harness = false
```

Create `crates/automapper-core/src/lib.rs`:

```rust
//! Core automapper library.
//!
//! Coordinators, mappers, builders, and writers for bidirectional
//! EDIFACT <-> BO4E conversion. Supports streaming parsing with
//! format-version-dispatched mappers and parallel batch processing.
```

**Step 5: Create automapper-validation crate**

Create `crates/automapper-validation/Cargo.toml`:

```toml
[package]
name = "automapper-validation"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "AHB validation — condition expression parser and evaluator"

[dependencies]
edifact-types.workspace = true
bo4e-extensions.workspace = true
automapper-core.workspace = true
thiserror.workspace = true

[dev-dependencies]
test-case.workspace = true
```

Create `crates/automapper-validation/src/lib.rs`:

```rust
//! AHB (Anwendungshandbuch) validation for EDIFACT messages.
//!
//! Parses and evaluates condition expressions from AHB business rules.
```

**Step 6: Create automapper-generator crate**

Create `crates/automapper-generator/Cargo.toml`:

```toml
[package]
name = "automapper-generator"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Code generation CLI — reads MIG/AHB XML schemas, generates Rust mapper code"

[dependencies]
automapper-core.workspace = true
automapper-validation.workspace = true
clap.workspace = true
quick-xml.workspace = true
thiserror.workspace = true
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
test-case.workspace = true
```

Create `crates/automapper-generator/src/lib.rs`:

```rust
//! Code generation CLI for EDIFACT mappers.
//!
//! Reads MIG/AHB XML schemas and generates Rust source code
//! for mappers and condition evaluators.
```

**Step 7: Create automapper-api crate**

Create `crates/automapper-api/Cargo.toml`:

```toml
[package]
name = "automapper-api"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "REST + gRPC server for EDIFACT/BO4E conversion"

[dependencies]
automapper-core.workspace = true
automapper-validation.workspace = true
bo4e-extensions.workspace = true
axum.workspace = true
tokio.workspace = true
tonic.workspace = true
prost.workspace = true
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
thiserror.workspace = true

[dev-dependencies]
```

Create `crates/automapper-api/src/lib.rs`:

```rust
//! Axum REST + tonic gRPC server for EDIFACT/BO4E conversion.
//!
//! Serves both HTTP REST and gRPC on the same port.
//! Also serves the Leptos WASM frontend as static files.
```

**Step 8: Create automapper-web crate**

Create `crates/automapper-web/Cargo.toml`:

```toml
[package]
name = "automapper-web"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Leptos WASM frontend for the EDIFACT converter"

[dependencies]
leptos.workspace = true
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
```

Create `crates/automapper-web/src/lib.rs`:

```rust
//! Leptos WASM frontend for the EDIFACT converter.
//!
//! Two-panel converter UI with collapsible detail panels
//! for segment tree, mapping trace, and errors.
```

**Step 9: Run cargo check to verify**

Run: `cargo check --workspace`
Expected: PASS — all 8 crates compile (empty lib.rs stubs)

**Step 10: Commit**

```bash
git add Cargo.toml .gitignore rustfmt.toml clippy.toml crates/
git commit -m "$(cat <<'EOF'
feat(workspace): initialize Cargo workspace with 8 crate stubs

Sets up the full workspace structure with edifact-types, edifact-parser,
bo4e-extensions, automapper-core, automapper-validation, automapper-generator,
automapper-api, and automapper-web. All crates compile as empty stubs.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add Git Submodules

**Files:**
- Create: `.gitmodules`

**Step 1: Add submodules**

```bash
git submodule add https://github.com/Hochfrequenz/xml-migs-and-ahbs.git xml-migs-and-ahbs
git submodule add https://github.com/Hochfrequenz/stammdatenmodell.git stammdatenmodell
git submodule add https://github.com/Hochfrequenz/example_market_communication_bo4e_transactions.git example_market_communication_bo4e_transactions
```

**Step 2: Create symlink for test fixtures**

```bash
mkdir -p tests/fixtures
ln -s ../../example_market_communication_bo4e_transactions tests/fixtures/examples
```

**Step 3: Verify submodules**

Run: `git submodule status`
Expected: Three submodules listed with commit hashes.

**Step 4: Commit**

```bash
git add .gitmodules xml-migs-and-ahbs stammdatenmodell example_market_communication_bo4e_transactions tests/fixtures
git commit -m "$(cat <<'EOF'
feat(submodules): add xml-migs-and-ahbs, stammdatenmodell, and example transactions

These submodules provide MIG/AHB XML schemas for code generation
and real EDIFACT fixture files for integration testing.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Create CLAUDE.md

**Files:**
- Create: `CLAUDE.md`

**Step 1: Write CLAUDE.md**

Create `CLAUDE.md`:

```markdown
# CLAUDE.md — Project Conventions for edifact-bo4e-automapper

## Project Overview

Rust port of the C# edifact_bo4e_automapper. Streaming EDIFACT parser with
bidirectional BO4E mapping for German energy market messages (UTILMD, APERAK).

## Workspace Structure

8 crates in dependency order:
1. `edifact-types` — zero-dep EDIFACT primitives
2. `edifact-parser` — standalone streaming parser (publishable)
3. `bo4e-extensions` — BO4E companion types for EDIFACT domain data
4. `automapper-core` — coordinators, mappers, builders, writers
5. `automapper-validation` — AHB condition parser/evaluator
6. `automapper-generator` — CLI code generator from MIG/AHB XML
7. `automapper-api` — Axum REST + tonic gRPC server
8. `automapper-web` — Leptos WASM frontend

## Commands

```bash
cargo check --workspace          # Type-check everything
cargo test --workspace           # Run all tests
cargo test -p <crate>            # Run tests for one crate
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all -- --check       # Format check
cargo bench -p automapper-core   # Benchmarks
```

## Coding Conventions

- **Error handling**: `thiserror` in all library crates. No `anyhow` in library code.
- **Serialization**: All domain types derive `Serialize, Deserialize`.
- **Lifetimes**: `RawSegment<'a>` borrows from input buffer. Zero-copy hot path.
- **Testing**: TDD — write failing test first, then implement. Use `#[cfg(test)]` modules.
- **Naming**: Rust snake_case for fields/functions. German domain terms preserved (Marktlokation, Zeitscheibe, Geschaeftspartner).
- **Format versions**: `FV2504`, `FV2510` marker types. `VersionConfig` trait for compile-time dispatch.
- **Commits**: Conventional commits (`feat`, `fix`, `refactor`, `test`, `docs`). Include `Co-Authored-By` trailer.

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
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "$(cat <<'EOF'
docs: add CLAUDE.md with project conventions and architecture guide

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Create Generated and Test Directories

**Files:**
- Create: `generated/.gitkeep`
- Create: `tests/integration/.gitkeep`
- Create: `tests/snapshots/.gitkeep`

**Step 1: Create directory structure**

```bash
mkdir -p generated tests/integration tests/snapshots
touch generated/.gitkeep tests/integration/.gitkeep tests/snapshots/.gitkeep
```

**Step 2: Verify full workspace compiles**

Run: `cargo check --workspace`
Expected: PASS

Run: `cargo fmt --all -- --check`
Expected: PASS

Run: `cargo clippy --workspace -- -D warnings`
Expected: PASS

**Step 3: Commit**

```bash
git add generated/ tests/
git commit -m "$(cat <<'EOF'
feat(workspace): add generated/ and tests/ directory structure

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 0 (scaffolding epic — no functional code) |
| Passed | 0 |
| Failed | 0 |
| Skipped | 0 |
| cargo check | PASS |
| cargo fmt --check | PASS |
| cargo clippy | PASS |

Verified:
- All 8 crates compile as empty stubs
- `cargo check --workspace` passes
- `cargo fmt --all -- --check` passes
- `cargo clippy --workspace -- -D warnings` passes
- `cargo test --workspace` passes (0 tests, all 8 crates exercised)
- 3 git submodules added and verified
- Directory structure: `generated/`, `tests/integration/`, `tests/snapshots/`, `tests/fixtures/`

Files created:
- `Cargo.toml` (workspace root)
- `Cargo.lock`
- `.gitignore`
- `rustfmt.toml`
- `clippy.toml`
- `CLAUDE.md`
- `.gitmodules`
- `crates/edifact-types/{Cargo.toml,src/lib.rs}`
- `crates/edifact-parser/{Cargo.toml,src/lib.rs}`
- `crates/bo4e-extensions/{Cargo.toml,src/lib.rs}`
- `crates/automapper-core/{Cargo.toml,src/lib.rs,benches/parser_throughput.rs}`
- `crates/automapper-validation/{Cargo.toml,src/lib.rs}`
- `crates/automapper-generator/{Cargo.toml,src/lib.rs}`
- `crates/automapper-api/{Cargo.toml,src/lib.rs}`
- `crates/automapper-web/{Cargo.toml,src/lib.rs}`
- `generated/.gitkeep`
- `tests/integration/.gitkeep`
- `tests/snapshots/.gitkeep`
- `tests/fixtures/examples` (symlink)
