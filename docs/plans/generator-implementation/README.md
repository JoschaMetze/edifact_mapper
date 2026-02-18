# Generator Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the `automapper-generator` crate -- a CLI tool that reads MIG/AHB XML schema files (from the `xml-migs-and-ahbs/` submodule), generates Rust mapper stubs and condition evaluator code into `generated/`, and optionally shells out to the `claude` CLI for AI-assisted condition implementation. Ports the C# `Automapper.Generator` project to idiomatic Rust.

**Architecture:** The generator is a `clap`-based CLI binary (`crates/automapper-generator/src/main.rs`) with three subcommands: `generate-mappers`, `generate-conditions`, and `validate-schema`. It uses `quick-xml` for SAX-style XML parsing of MIG/AHB files into in-memory schema representations (`MigSchema`, `AhbSchema`). Code generation uses `askama` templates to produce Rust source files. AI-assisted condition generation shells out to the `claude` CLI (no SDK dependency). All output goes to `generated/` and is committed to the repo -- no build-time codegen.

**Tech Stack:** Rust 2021 edition, clap 4.x (CLI), quick-xml 0.37 (XML parsing), askama 0.12 (templates), thiserror 2.x (errors), serde + serde_json (metadata serialization), insta (snapshot tests), tempfile (test fixtures)

**Depends On:** Feature 1 (edifact-core-implementation) -- uses types from `edifact-types`, `bo4e-extensions`, and `automapper-core`

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | MIG/AHB XML Schema Parsing | `MigSchema`, `AhbSchema` structs, `quick-xml` parsers, `GeneratorError`, unit tests against real XML | - |
| 2 | Rust Mapper Code Generation | `clap` CLI, `askama` templates, `GenerateMappers` subcommand, generate mapper stubs + `VersionConfig` impls + coordinator registration, snapshot tests with `insta` | Epic 1 |
| 3 | AI-Assisted Condition Generation | `ClaudeConditionGenerator`, prompt building, `GenerateConditions` + `ValidateSchema` subcommands, batch generation, incremental mode, mock-based tests | Epic 1, Epic 2 |

---

## Files in This Plan

1. [Epic 1: MIG/AHB XML Schema Parsing](./epic-01-mig-ahb-xml-parsing.md)
2. [Epic 2: Rust Mapper Code Generation](./epic-02-rust-mapper-codegen.md)
3. [Epic 3: AI-Assisted Condition Generation](./epic-03-ai-condition-generation.md)

---

## Crate Layout

```
crates/automapper-generator/
├── Cargo.toml
├── src/
│   ├── main.rs              # clap CLI entry point
│   ├── lib.rs               # Re-exports for testing
│   ├── error.rs             # GeneratorError with thiserror
│   ├── schema/
│   │   ├── mod.rs
│   │   ├── mig.rs           # MigSchema, MigSegment, MigComposite, MigDataElement, etc.
│   │   ├── ahb.rs           # AhbSchema, Pruefidentifikator, AhbRule, BedingungDefinition, etc.
│   │   └── common.rs        # Cardinality, EdifactFormat, CodeDefinition
│   ├── parsing/
│   │   ├── mod.rs
│   │   ├── mig_parser.rs    # parse_mig() using quick-xml
│   │   └── ahb_parser.rs    # parse_ahb() using quick-xml
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── mapper_gen.rs    # Mapper stub generation
│   │   ├── coordinator_gen.rs  # Coordinator registration code
│   │   ├── version_config_gen.rs  # VersionConfig impl generation
│   │   └── templates/       # askama .rs.txt template files
│   │       ├── mapper_stub.rs.txt
│   │       ├── version_config.rs.txt
│   │       └── coordinator_registration.rs.txt
│   ├── conditions/
│   │   ├── mod.rs
│   │   ├── claude_generator.rs   # ClaudeConditionGenerator
│   │   ├── prompt.rs             # Prompt building
│   │   ├── condition_types.rs    # GeneratedCondition, ConfidenceLevel
│   │   └── metadata.rs          # Condition metadata for incremental regen
│   └── validation/
│       ├── mod.rs
│       └── schema_validator.rs   # ValidateSchema subcommand logic
├── templates/                    # askama template directory (referenced by build)
│   ├── mapper_stub.txt
│   ├── version_config.txt
│   ├── coordinator_registration.txt
│   └── condition_evaluator.txt
└── tests/
    ├── mig_parsing_tests.rs
    ├── ahb_parsing_tests.rs
    ├── codegen_tests.rs
    └── snapshots/                # insta snapshot files
```

## Test Strategy

- **Unit tests**: `#[cfg(test)]` modules for each parser and generator module
- **Integration tests**: Parse real MIG/AHB XML files from `xml-migs-and-ahbs/` submodule
- **Snapshot tests**: `insta` for generated Rust code output verification
- **Mock tests**: Mock the `claude` CLI with a shell script that returns canned JSON responses
- **Compilation tests**: Generated code is checked with `cargo check` in CI

## Commands Reference

```bash
# Check the generator crate compiles
cargo check -p automapper-generator

# Run generator tests
cargo test -p automapper-generator

# Run the generator CLI
cargo run -p automapper-generator -- generate-mappers \
    --mig-path xml-migs-and-ahbs/FV2510/UTILMD_MIG_Strom_S2_1.xml \
    --ahb-path xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_2_1.xml \
    --output-dir generated/ \
    --format-version FV2510 \
    --message-type UTILMD

cargo run -p automapper-generator -- generate-conditions \
    --ahb-path xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_2_1.xml \
    --output-dir generated/ \
    --format-version FV2510 \
    --message-type UTILMD \
    --incremental

cargo run -p automapper-generator -- validate-schema \
    --stammdatenmodell-path stammdatenmodell/ \
    --generated-dir generated/

# Update insta snapshots
cargo insta test -p automapper-generator
cargo insta review
```
