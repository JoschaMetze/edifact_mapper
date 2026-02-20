# edifact-bo4e-automapper task runner
# Install: cargo install just

set dotenv-load

# List available recipes
default:
    @just --list

# --- Build ---

# Build all crates (debug)
build:
    cargo build --workspace

# Build all crates (release)
build-release:
    cargo build --release --workspace

# Type-check everything without building
check:
    cargo check --workspace

# --- Test ---

# Run all tests
test:
    cargo test --workspace

# Run tests for a specific crate
test-crate crate:
    cargo test -p {{crate}}

# Run a single test by name
test-one crate name:
    cargo test -p {{crate}} {{name}}

# Run tests with output shown
test-verbose:
    cargo test --workspace -- --nocapture

# --- Lint & Format ---

# Run clippy (warnings are errors)
lint:
    cargo clippy --workspace -- -D warnings

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Auto-format all code
fmt:
    cargo fmt --all

# Run all checks (lint + format + test)
ci: lint fmt-check test

# --- Run ---

# Start the API server
serve:
    cargo run -p automapper-api

# Start the API server (release mode)
serve-release:
    cargo run --release -p automapper-api

# Run the code generator CLI
generate *args:
    cargo run -p automapper-generator -- {{args}}

# Generate shared MIG types (enums, composites, segments, groups)
generate-mig-types fv msg mig_xml:
    cargo run -p automapper-generator -- generate-mig-types \
        --mig-path {{mig_xml}} \
        --message-type {{msg}} \
        --format-version {{fv}} \
        --output-dir crates/mig-types/src/generated

# Generate per-PID composition types from AHB + MIG XML
generate-pid-types fv msg mig_xml ahb_xml:
    cargo run -p automapper-generator -- generate-pid-types \
        --mig-path {{mig_xml}} \
        --ahb-path {{ahb_xml}} \
        --message-type {{msg}} \
        --format-version {{fv}} \
        --output-dir crates/mig-types/src/generated

# Generate all MIG + PID types for UTILMD FV2504 (Strom)
generate-utilmd-fv2504:
    just generate-mig-types FV2504 UTILMD \
        xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml
    just generate-pid-types FV2504 UTILMD \
        xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
        xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml

# --- Benchmarks ---

# Run benchmarks
bench:
    cargo bench -p automapper-core

# --- Snapshot Tests ---

# Run snapshot tests
snap-test:
    cargo insta test -p automapper-generator

# Review snapshot changes
snap-review:
    cargo insta review

# --- Utilities ---

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree --workspace

# Count lines of code (requires tokei)
loc:
    tokei crates/
