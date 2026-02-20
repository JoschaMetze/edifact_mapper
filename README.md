# edifact-bo4e-automapper

Bidirectional EDIFACT <-> BO4E conversion for the German energy market, written in Rust.

Rust port of the C# [edifact_bo4e_automapper](https://github.com/Hochfrequenz/edifact_bo4e_automapper). Designed for batch processing millions of messages with zero-copy parsing.

## Crates

| Crate | Description |
|-------|-------------|
| `edifact-types` | Zero-dependency EDIFACT primitives (`RawSegment`, `EdifactDelimiters`) |
| `edifact-parser` | Standalone SAX-style streaming parser (publishable independently) |
| `bo4e-extensions` | BO4E companion types, `WithValidity<T,E>` wrapper, `LinkRegistry` |
| `automapper-core` | Coordinators, entity mappers, builders, writers, batch processing |
| `automapper-validation` | AHB condition parser and evaluator, EDIFACT message validator |
| `automapper-generator` | CLI tool: generates Rust mapper code from MIG/AHB XML schemas |
| `automapper-api` | Axum REST API + tonic gRPC server |
| `automapper-web` | Leptos WASM frontend |

## Requirements

- Rust 1.75+ (MSRV)
- Git submodules (test fixtures and XML schemas)

## Getting Started

```bash
# Clone with submodules
git clone --recurse-submodules <repo-url>
cd edifact_mapper

# Or initialize submodules after cloning
git submodule update --init --recursive
```

### With Nix (recommended)

A `flake.nix` provides all dependencies — Rust toolchain, `just`, `cargo-insta`, `cargo-nextest`, `trunk`, `tokei`, and WASM targets.

```bash
# Enter the dev shell (one-time setup pulls everything)
nix develop

# Or with direnv (auto-activates on cd)
direnv allow
```

### Without Nix

Install manually:
- [Rust 1.75+](https://rustup.rs/)
- [just](https://github.com/casey/just) (optional, for convenience commands)
- [cargo-insta](https://insta.rs/) (optional, for snapshot tests)

```bash
# Build all crates
cargo build --workspace

# Run the API server
cargo run -p automapper-api
```

## Quick Reference (just)

```bash
just              # List all commands
just build        # Build all crates
just test         # Run all tests
just lint         # Clippy (warnings = errors)
just fmt          # Auto-format
just ci           # Lint + format check + test
just serve        # Start the API server
just bench        # Run benchmarks
just snap-test    # Run snapshot tests
just snap-review  # Review snapshot changes

just generate-utilmd-fv2504                   # Regenerate all UTILMD FV2504 types
just generate-mig-types FV2504 UTILMD mig.xml # Generate shared MIG types
just generate-pid-types FV2504 UTILMD mig.xml ahb.xml # Generate PID types
```

## Testing

```bash
# Run all tests (333 across the workspace)
cargo test --workspace

# Run tests for a specific crate
cargo test -p edifact-parser
cargo test -p automapper-core
cargo test -p automapper-validation

# Run a single test by name
cargo test -p edifact-parser test_una_detection

# Lint (warnings are errors)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --all -- --check

# Benchmarks (parser throughput)
cargo bench -p automapper-core
```

### Snapshot Tests

The `automapper-generator` crate uses [insta](https://insta.rs/) for snapshot testing of generated code:

```bash
cargo insta test -p automapper-generator    # Run snapshot tests
cargo insta review                          # Review snapshot changes
```

## Usage

### As a Library

Parse an EDIFACT message and convert to BO4E:

```rust
use edifact_parser::EdifactStreamParser;
use automapper_core::{create_coordinator, FormatVersion};

// Create a version-specific coordinator
let coordinator = create_coordinator(FormatVersion::FV2504);

// Parse EDIFACT input (zero-copy, streaming)
let result = coordinator.process(edifact_input)?;

// result contains Vec<UtilmdTransaktion> with BO4E objects
```

### REST API

```bash
# Start the server
cargo run -p automapper-api

# Convert EDIFACT to BO4E
curl -X POST http://localhost:3000/api/v1/convert \
  -H "Content-Type: application/json" \
  -d '{"input": "UNB+...", "direction": "edifact_to_bo4e", "format_version": "FV2504"}'

# Inspect EDIFACT structure
curl -X POST http://localhost:3000/api/v1/inspect \
  -H "Content-Type: application/json" \
  -d '{"input": "UNB+..."}'

# List available coordinators
curl http://localhost:3000/api/v1/coordinators

# Health check
curl http://localhost:3000/health
```

### gRPC

Proto definitions are in `proto/`. Services:

- `TransformService` — `ConvertEdifactToBo4e`, `ConvertBo4eToEdifact` (unary + streaming)
- `InspectionService` — `InspectEdifact`, `ListCoordinators`

### Code Generator CLI

Generate typed Rust code from MIG/AHB XML schemas:

```bash
# Generate shared MIG types (enums, composites, segments, groups)
cargo run -p automapper-generator -- generate-mig-types \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --message-type UTILMD --format-version FV2504 \
  --output-dir crates/mig-types/src/generated

# Generate per-PID composition types (requires MIG + AHB)
cargo run -p automapper-generator -- generate-pid-types \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-path xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml \
  --message-type UTILMD --format-version FV2504 \
  --output-dir crates/mig-types/src/generated

# Generate mapper stubs + coordinator + VersionConfig
cargo run -p automapper-generator -- generate-mappers \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-path xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml \
  --message-type UTILMD --format-version FV2504 \
  --output-dir generated/

# Generate condition evaluators from AHB rules (calls Claude API)
cargo run -p automapper-generator -- generate-conditions \
  --ahb-path xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml \
  --message-type UTILMD --format-version FV2504 \
  --output-dir generated/ --dry-run

# Validate generated code against BO4E schema
cargo run -p automapper-generator -- validate-schema \
  --stammdatenmodell-path stammdatenmodell/ \
  --generated-dir generated/

# Or use just shortcuts:
just generate-utilmd-fv2504  # Regenerate all UTILMD FV2504 MIG + PID types
```

## Architecture

```
EDIFACT input
    |
    v
EdifactStreamParser  (zero-copy SAX-style tokenizer)
    |
    v
UtilmdCoordinator<V>  (routes segments to entity mappers)
    |
    +-> MarktlokationMapper   (LOC+Z16)
    +-> MesslokationMapper    (LOC+Z17)
    +-> NetzlokationMapper    (LOC+Z08)
    +-> ZaehlerMapper         (SEQ+Z03)
    +-> GeschaeftspartnerMapper (NAD)
    +-> VertragMapper          (PAT)
    +-> ProzessdatenMapper     (STS, DTM, RFF, FTX)
    +-> ZeitscheibeMapper      (SEQ+Z98)
    |
    v
WithValidity<BO4E, Edifact>  (domain objects)
    |
    v
DocumentWriter  (BO4E -> EDIFACT roundtrip)
```

Format versions (`FV2504`, `FV2510`) are dispatched at compile time via the `VersionConfig` trait for zero-cost abstraction in the hot path.

## License

MIT
