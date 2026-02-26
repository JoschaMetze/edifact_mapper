# CLAUDE.md — Project Conventions for edifact-bo4e-automapper

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust port of the C# [edifact_bo4e_automapper](https://github.com/Hochfrequenz/edifact_bo4e_automapper) — bidirectional EDIFACT ↔ BO4E conversion for the German energy market. Goals: batch processing millions of messages with zero-copy parsing, and publishing reusable Rust crates.

C# reference repo: `../edifact_bo4e_automapper/` (commit cee0b09)

## Workspace Structure

10 crates in dependency order:
1. `edifact-types` — zero-dep EDIFACT primitives
2. `edifact-parser` — standalone streaming parser (publishable)
3. `bo4e-extensions` — BO4E companion types for EDIFACT domain data
4. `automapper-validation` — AHB condition parser/evaluator
5. `automapper-generator` — CLI code generator from MIG/AHB XML
6. `mig-types` — generated typed MIG-tree types (segments, composites, enums, PIDs)
7. `mig-assembly` — MIG-guided EDIFACT tree assembly/disassembly, ConversionService
8. `mig-bo4e` — declarative TOML-based MIG-tree to BO4E mapping engine
9. `automapper-api` — Axum REST + tonic gRPC server (v2 MIG-driven API)
10. `automapper-web` — Leptos WASM frontend

## Commands

```bash
cargo check --workspace          # Type-check everything
cargo test --workspace           # Run all tests
cargo test -p <crate>            # Run tests for one crate
cargo test -p edifact-parser test_una_detection  # Run a single test
cargo clippy --workspace -- -D warnings  # Lint (warnings are errors)
cargo fmt --all -- --check       # Format check
cargo fmt --all                  # Auto-format
cargo bench -p mig-assembly      # MIG-driven pipeline benchmarks (includes batch)
cargo build --release --workspace

# Schema lookup CLI — inspect PID schemas for TOML mapping authoring
cargo run -p automapper-generator -- schema-lookup --pid 55035                # List all groups
cargo run -p automapper-generator -- schema-lookup --pid 55035 --group sg4.sg8_zf0  # Detail for one group
cargo run -p automapper-generator -- schema-lookup --pid 55035 --group sg4.sg8_zf0 --toml-template  # With TOML template

# Migrate TOML paths from numeric to named EDIFACT ID paths
cargo run -p automapper-generator -- migrate-paths \
  --schema-dir crates/mig-types/src/generated/fv2504/utilmd/pids \
  --mappings-dir mappings/FV2504/UTILMD_Strom --dry-run  # Preview changes
cargo run -p automapper-generator -- migrate-paths \
  --schema-dir crates/mig-types/src/generated/fv2504/utilmd/pids \
  --mappings-dir mappings/FV2504/UTILMD_Strom             # Apply changes
```

## Architecture

Ten-crate Cargo workspace under `crates/`, ordered by dependency:

```
edifact-types          Zero-copy EDIFACT primitives (RawSegment<'a>, EdifactDelimiters)
    ↓
edifact-parser         SAX-style streaming parser, EdifactHandler trait, UNA detection
    ↓
bo4e-extensions        WithValidity<T,E> wrapper, *Edifact companion types, LinkRegistry
    ↓                  (depends on external bo4e-rust crate for standard BO4E types)
automapper-generator   CLI: MIG/AHB XML → Rust codegen, claude CLI for conditions
    ↓
mig-types              Generated typed MIG-tree types (segments, composites, enums, PIDs)
mig-assembly           MIG-guided tree assembly/disassembly, ConversionService
    ↓
mig-bo4e               TOML-based MIG-tree → BO4E mapping engine
    ↓
├── automapper-validation   AHB condition parser/evaluator, EdifactValidator
└── automapper-api          Axum REST + tonic gRPC (v2 MIG-driven API)
        ↓
    automapper-web          Leptos WASM frontend (served as static files by api)
```

### Key Patterns

**Streaming parser**: `EdifactStreamParser::parse(input, handler)` emits `RawSegment<'a>` references borrowing from the input buffer. Handlers implement `EdifactHandler` trait (on_interchange_start, on_message_start, on_segment, etc.) and return `Control::Continue` or `Control::Stop`.

**Companion types**: `*Edifact` structs (e.g. `MarktlokationEdifact`) store functional domain data that exists in EDIFACT but not in standard BO4E (data quality, cross-references, qualifiers). They do NOT store transport/ordering data — roundtrip ordering is handled by deterministic MIG-derived rules.

**WithValidity<T, E>**: Wraps a standard BO4E object (`T`) with its EDIFACT companion (`E`), a validity period (`Zeitraum`), and optional Zeitscheibe reference.

### MIG-Driven Pipeline

Data-driven from MIG XML schemas using declarative TOML mappings:

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

**API** (`automapper-api`): `POST /api/v2/convert` accepts `mode` parameter:
- `mig-tree` — returns the MIG-assembled tree as JSON
- `bo4e` — assembles + applies TOML mappings → BO4E JSON

## Coding Conventions

- **Error handling**: `thiserror` in all library crates. No `anyhow` in library code.
- **Serialization**: All domain types derive `Serialize, Deserialize`.
- **Lifetimes**: `RawSegment<'a>` borrows from input buffer. Zero-copy hot path.
- **Testing**: TDD — write failing test first, then implement. Use `#[cfg(test)]` modules.
- **Naming**: Rust snake_case for fields/functions. German domain terms preserved (Marktlokation, Zeitscheibe, Geschaeftspartner).
- **Format versions**: `FV2504`, `FV2510` — format version identifiers used across MIG/AHB XML processing and TOML mapping directories.
- **Commits**: Conventional commits (`feat`, `fix`, `refactor`, `test`, `docs`). Include `Co-Authored-By` trailer.
- **`edifact-parser` is standalone**: No BO4E dependency — publishable as a generic EDIFACT parser crate.
- **Generated code**: Output of `automapper-generator` goes to `generated/` and is committed (no build-time codegen).

## Architecture Decisions

- Parser is SAX-style streaming (matches C# EdifactStreamParser)
- Handler trait has default no-op methods — implementors override what they need
- MIG-driven pipeline: data-driven assembly from MIG XML + declarative TOML mappings
- TOML mapping engine handles both forward (tree->BO4E) and reverse (BO4E->tree) conversion
- Roundtrip fidelity: parse -> assemble -> disassemble -> render must produce byte-identical output

## Test Data

- `example_market_communication_bo4e_transactions/` — real EDIFACT fixture files (submodule)
- `xml-migs-and-ahbs/` — MIG/AHB XML schemas (submodule)
- `stammdatenmodell/` — BO4E data model reference (submodule)
- `tests/fixtures/` — symlinks to submodule data

## Dependencies on C# Reference

Reference C# repo: see design doc for architectural mapping.
Key correspondences:
- `EdifactStreamParser.cs` -> `edifact-parser` crate
- MIG XML schemas -> `mig-assembly` crate (data-driven assembly replaces hand-coded coordinators/mappers)
- TOML mapping definitions -> `mig-bo4e` crate (declarative mappings replace hand-coded entity mappers/writers)

## Implementation Status

5 features (23 epics) implemented. ~6,000 LOC, 260+ tests.

| Crate | Tests | Notes |
|-------|-------|-------|
| edifact-types | 29 | Delimiter parsing, segment construction |
| edifact-parser | 37 | Tokenizer, UNA detection, property tests (proptest) |
| bo4e-extensions | 29 | WithValidity, LinkRegistry, companion types |
| automapper-validation | 143 | Condition parser, evaluator, validator |
| automapper-generator | 1 | Snapshot tests (insta) |
| mig-types | 6 | Generated typed MIG-tree types |
| mig-assembly | 40+ | Assembler, disassembler, roundtrip, ConversionService |
| mig-bo4e | 65+ | TOML mapping engine, roundtrip, PID 55001/55002 mapping tests |
| automapper-api | 12 | REST v2, gRPC, integration tests |
| automapper-web | 7 | Type serialization, API contract tests |

### Implementation Learnings

- **Snapshot testing with insta** works well for codegen output — use `cargo insta test` then `cargo insta review`.
- **Property testing with proptest** catches edge cases in tokenizer/delimiter parsing that unit tests miss.
- **Parameterized tests with test-case** reduce boilerplate for mapping tests with multiple segment qualifiers.
- **TOML mapping files** are the primary way to add new PID support — one file per MIG group, declarative field mappings.
- **Writer segment ordering** is deterministic from MIG rules — no need to store ordering metadata on companion types.
- **AHB condition parser** uses recursive descent with Unicode operators (`∧`, `∨`, `⊻`, `¬`). Three-valued logic (True/False/Unknown) handles missing data gracefully.
- **gRPC streaming** uses tonic's `Streaming<T>` for both request and response sides. Proto files live in `proto/`.
- **Leptos WASM frontend** communicates via REST to the Axum backend. The `static/` directory holds build output served as a fallback route.

### PID-Specific Assembly via AHB-MIG Number Linkage

The MIG XML defines ALL possible segments/groups for a message type (e.g., UTILMD has two SG4 variants, dozens of SG8 variants, many SG12 variants). The AHB for a specific PID references exactly which subset to use. The link between them is the `Number` attribute on `S_*` segment elements — it's the same value in both MIG and AHB XMLs.

**How to add a new PID to the MIG-driven pipeline:**

1. **Get AHB segment numbers**: Parse the AHB XML for the PID → `Pruefidentifikator.segment_numbers` gives you all `Number` values (e.g., PID 55001 has 30 Numbers).
2. **Filter MIG**: Call `pid_filter::filter_mig_for_pid(mig, ahb_numbers)` — this produces a PID-specific MIG with no ambiguous duplicate groups.
3. **Assemble**: The existing `Assembler::assemble_generic()` works unchanged on the filtered MIG, capturing all nested groups correctly.
4. **Roundtrip**: `EDIFACT → tokenize → assemble(filtered_mig) → disassemble → render` is byte-identical for well-formed fixtures.

**Key details:**
- Transport segments (UNA, UNB, UNZ) are always kept — the AHB only covers message content (UNH→UNT).
- The filter works recursively: it resolves ambiguity at all nesting levels (SG4 variants, SG8 variants within SG4, SG10 variants within SG8, etc.).
- Entry segment `Number` determines group selection — if the first segment's Number isn't in the AHB set, the entire group variant is dropped.
- The assembled tree preserves the full group hierarchy: `SG4 → [SG5, SG6, SG8 → [SG10], SG12]`.

### Process: Creating TOML Mappings for a New PID

Follow this step-by-step process when adding BO4E mappings for a new PID.

**Step 1: Read the PID schema JSON (or use schema-lookup CLI)**
```bash
# Primary reference — always start here
cat crates/mig-types/src/generated/fv2504/utilmd/pids/pid_NNNNN_schema.json

# Or use the schema-lookup CLI for structured output:
cargo run -p automapper-generator -- schema-lookup --pid NNNNN              # List all groups
cargo run -p automapper-generator -- schema-lookup --pid NNNNN --group sg4.sg8_zf0  # Detail with EDIFACT ID paths
cargo run -p automapper-generator -- schema-lookup --pid NNNNN --group sg4.sg8_zf0 --toml-template  # Pre-filled TOML
```
This is the single source of truth. It tells you:
- Which SG groups exist (sg5_z16, sg8_z98, sg12_z04, etc.)
- What segments each group contains (LOC, SEQ, CCI, CAV, NAD, RFF, etc.)
- Element indices and component sub-indices for field paths
- EDIFACT ID paths (e.g., `loc.c517.d3225` for LOC element 1, component 0)
- Discriminator codes (LOC qualifier Z16/Z17, SEQ qualifier Z98/ZD7, etc.)
- Which codes are AHB-filtered (only valid codes for this PID)

**Step 2: Check for a reference PID with similar structure**
```bash
ls mappings/FV2504/UTILMD_Strom/
# Compare with existing PIDs — 55001 (Anmeldung) and 55002 (Bestätigung) are references
diff <(python3 -c "import json; d=json.load(open('...pid_55001_schema.json')); print('\n'.join(sorted(d['fields'].keys())))") \
     <(python3 -c "import json; d=json.load(open('...pid_NNNNN_schema.json')); print('\n'.join(sorted(d['fields'].keys())))")
```
If a similar PID already has mappings, copy and adapt rather than starting from scratch.

**Step 3: Generate scaffolds (optional starting point)**
```bash
cargo run -p automapper-generator -- generate-toml-scaffolds \
  --mig-xml xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-xml xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml \
  --message-type UTILMD --variant Strom --format-version FV2504 \
  --pid NNNNN --output-dir mappings/FV2504/UTILMD_Strom
```
Scaffolds are a starting point — they need manual review and refinement (entity names, field names, companion_fields).

**Step 4: Map the group hierarchy to entities**

Read the schema's `fields` object and create one TOML file per group. Use this mapping:

| Schema group | TOML pattern | Entity name source |
|---|---|---|
| `sg2` | `marktteilnehmer.toml` | Always "Marktteilnehmer" |
| `sg2.sg3_ic` | `ansprechpartner.toml` or `kontakt.toml` | CTA function code |
| `sg4` (root segments) | `prozessdaten.toml` | IDE/DTM/STS = "Prozessdaten" |
| `sg4.sg5_zNN` | `{entity}.toml` | LOC code description in schema |
| `sg4.sg6` | `prozessdaten_rff_{qual}.toml` | RFF qualifier, merges into "Prozessdaten" |
| `sg4.sg8_zXX` | `{entity}_info.toml` | SEQ code → parent LOC entity |
| `sg4.sg8_zXX.sg10` | `{entity}_zuordnung.toml` | CCI/CAV → parent LOC entity companion |
| `sg4.sg12_zNN` | `geschaeftspartner.toml` | Always "Geschaeftspartner" — see NAD entity reuse pattern below |

**Step 5: Write each TOML file**

For each group, consult the schema to determine:
1. **Element indices**: Schema `elements[].index` → TOML path prefix (e.g., `loc.1.0`)
2. **Component sub-indices**: Schema `components[].sub_index` → TOML path suffix
3. **Codes vs data**: `type: "code"` with single value → use `default`; `type: "data"` → map to a field name
4. **Discriminators**: If multiple groups share the same `source_group`, add a `discriminator`
5. **companion_fields**: CCI/CAV/RFF segments in _zuordnung and _info files usually go in `[companion_fields]`

**Step 6: Check for a fixture file to test with**
```bash
ls example_market_communication_bo4e_transactions/UTILMD/FV2504/*NNNNN*
```
If a fixture exists, write a full EDIFACT roundtrip test. If not, write at least a TOML loading test.

**Step 7: Write EDIFACT roundtrip tests (REQUIRED)**

TOML loading tests and JSON key presence tests are NOT sufficient. They pass even when half the mappings are missing. Every PID with a fixture file MUST have a full pipeline roundtrip test:

```
EDIFACT → tokenize → split → assemble(PID-filtered MIG)
  → map_interchange(forward) → map_interchange_reverse
  → disassemble → render → compare with original EDIFACT
```

This is the only test that catches:
- Missing TOML files (segments not consumed → not reconstructed)
- Wrong field paths (values extracted incorrectly → different on roundtrip)
- Phantom segments from defaults (reverse generates segments not in original)

Reference implementation: `crates/mig-bo4e/tests/reverse_roundtrip_test.rs` (PID 55001 — passes byte-identical). Use `run_full_roundtrip()` from `pid_55013_to_55035_test.rs` as the reusable pattern.

**Step 8: Verify**
```bash
cargo test -p mig-bo4e -- --nocapture  # all mapping tests
cargo clippy -p mig-bo4e -- -D warnings
```

### TOML Mapping Guidelines

When creating TOML mapping files for new PIDs, follow these rules:

**Entity naming — derive from PID schema descriptions, not guesswork:**
- Always read the PID schema JSON (`crates/mig-types/src/generated/fv2504/utilmd/pids/pid_NNNNN_schema.json`) before naming entities.
- LOC qualifier codes map to specific location types: Z16=Marktlokation, Z17=Messlokation, Z18=Netzlokation, Z19=SteuerbareRessource, Z20=TechnischeRessource, Z22=RuhendeMarktlokation.
- SEQ groups describe their parent location's properties — name the entity after the location, not the property type.
- The schema `beschreibung` and segment `name` fields contain the canonical German descriptions — use these to determine entity names.

**Entity design — reuse BO4E types, don't invent new ones:**
- Map to existing BO4E core types (Marktlokation, Messlokation, Netzlokation, etc.) and their `*Edifact` companion types.
- SEQ/CCI/CAV "info" and "zuordnung" groups are NOT separate entities — they enrich their parent LOC entity. Use `entity = "Marktlokation"` (not `entity = "MarktlokationInfo"`).
- RFF groups with different qualifiers (Z13, TN, Z60) that describe the same concept merge into one entity (e.g., all into `Prozessdaten`) using discriminators, not separate entity types.
- **NAD/SG12 segments**: ALL NAD qualifiers (Z04, Z09, Z63–Z70, etc.) map to `Geschaeftspartner` — see **NAD/SG12 entity reuse** section below. Do NOT create per-qualifier types like `KundeDesLf` or `Marktlokationsanschrift`.
- Reference the C# project's `*Edifact` companion types to understand entity boundaries. If the C# code stores a field on `MarktlokationEdifact`, use `companion_fields` in the TOML, not a new entity.

**One TOML file = one source group instance:**
- Each file maps exactly one group in the MIG tree (SG5, SG8, SG10, etc.).
- Multiple files can share the same `entity` name — `deep_merge_insert()` combines their outputs.
- This enables per-group reuse across PIDs via the `generate-pid-mappings` tool.
- `[fields]` section is REQUIRED even if empty — omitting it causes a TOML parse error.

**companion_fields for EDIFACT-specific data:**
- Core BO4E fields go in `[fields]`.
- EDIFACT-only data (qualifiers, references, transport info) goes in `[companion_fields]` with a `companion_type` in `[meta]`.
- Only set `companion_type` on files that have `[companion_fields]` — it's unused without them.

**Discriminators for multi-instance groups:**
- When multiple TOML files map the same source group (e.g., multiple RFFs in SG6), each needs a `discriminator` (e.g., `RFF.0.0=Z13`) to select the right instance.
- When looking up definitions in tests, use `(entity, source_group)` pairs — `definition_for_entity()` alone is ambiguous when multiple files share an entity name.

**Field path convention — EDIFACT ID paths (preferred):**
- **Preferred**: EDIFACT ID paths — `loc.c517.d3225` (composite + component), `loc.d3227` (simple element), `cav[Z91].c889.d7111` (with qualifier).
- **Legacy**: Numeric paths — `loc.1.0`, `loc.0`, `cav[Z91].0.1`. Still supported but all existing TOML files have been migrated to EDIFACT ID paths.
- Both styles work in the same TOML file. EDIFACT ID paths are resolved to numeric at load time via `PathResolver`.
- **PathResolver is required** on all MappingEngine instances: `MappingEngine::load(dir)?.with_path_resolver(resolver)`. Use `PathResolver::from_schema_dir(path)` to load all PID schemas at once.
- Discriminators use EDIFACT IDs: `LOC.d3227=Z16`, `STS.c556.d9013=E01`. Resolved to 3-part numeric format (`LOC.0.0=Z16`) at load time.
- Duplicate EDIFACT IDs use ordinal suffixes: `c556_2` (2nd occurrence of C556 in segment), `d3036_2` (2nd occurrence of 3036 in composite).
- The `schema-lookup` CLI with `--toml-template` generates TOML using EDIFACT ID paths.
- The `migrate-paths` CLI converts numeric paths to named: `cargo run -p automapper-generator -- migrate-paths --schema-dir ... --mappings-dir ...`
- Empty EDIFACT values (empty string components) are omitted from BO4E JSON output — only non-empty values are included.
- The reverse mapper pads intermediate empty elements automatically, so omitting empty values from BO4E is safe for roundtrip.

**PID-specific STS mapping — verify schema per PID:**
- STS segment structure varies significantly between PIDs (e.g., 55001 uses `STS+7++E01+ZW4+E03` for Transaktionsgrund, while 55002 uses `STS+E01+<status>+<pruefschritt>:<ebd>::<ref>` for Antwort-Status).
- Always check the PID schema JSON for the actual STS element/composite layout before writing STS field mappings.
- DTM qualifiers also vary: 55001 has DTM+92 and DTM+93, 55002 only has DTM+93.

**NAD/SG12 entity reuse — single Geschaeftspartner type for ALL NAD qualifiers:**

The C# reference uses ONE `GeschaeftspartnerMapper` for all NAD qualifiers (Z04, Z09, Z48, Z50, Z63–Z70, DP, VN, KN, Z25). Follow this pattern — do NOT create per-qualifier BO4E types.

- **All SG12/NAD segments** → `entity = "Geschaeftspartner"`, `bo4e_type = "Geschaeftspartner"`, `companion_type = "GeschaeftspartnerEdifact"`.
- **NAD qualifier** stored in `companion_fields`: `"nad.d3035" = "nad_qualifier"` — this is what distinguishes Z65 (KundeDesLf) from Z66 (KorrespondenzanschriftKundeLf) etc.
- **One TOML file per PID** for all SG12 groups: map the superset of all NAD fields (C082 ID, C080 name, C058 additional info, C059 address, d3164 city, d3251 postal code, d3207 country, C819 region). Fields not present in a specific NAD instance are auto-omitted.
- **No discriminator needed**: when a PID has multiple SG12 reps (e.g., 55013 has 7), omitting the discriminator makes the engine auto-produce an array of Geschaeftspartner objects. Each array element captures only the non-empty fields for that qualifier.
- **Reverse mapping**: each array element becomes a separate SG12 rep. The `nad_qualifier` companion field writes the qualifier code (Z65, Z66, etc.) back to `NAD.d3035`.
- **Anti-pattern**: Do NOT create types like `KundeDesLf`, `KorrespondenzanschriftKundeLf`, `Marktlokationsanschrift`, `Anschlussnehmer`, `Hausverwalter` — these are all structurally identical NAD segments differing only by qualifier code.
- **Scope**: This applies to informative NAD qualifiers (Z63–Z70), business party NADs (Z04, Z09), and extended NADs (Z03, Z05, Z07, Z08, Z25, Z26, DP, VN, KN, etc.). ALL are Geschaeftspartner.
- **`deep_merge_insert` caveat**: Do NOT use discriminators with same entity name — the engine merges same-named entities, causing data loss when fields overlap. Instead, omit discriminators to get array output.

Example TOML (covers all SG12 variants in one file):
```toml
[meta]
entity = "Geschaeftspartner"
bo4e_type = "Geschaeftspartner"
companion_type = "GeschaeftspartnerEdifact"
source_group = "SG4.SG12"
source_path = "sg4.sg12"

[fields]
"nad.c082.d3039" = "identifikation"
"nad.c080.d3036" = "nachname"
"nad.c080.d3036_2" = "vorname"
"nad.c080.d3036_3" = "titel"
"nad.c080.d3036_5" = "anrede"
"nad.c058.d3124" = "zusatzinfo"
"nad.c059.d3042" = "strasse"
"nad.c059.d3042_3" = "hausnummer"
"nad.d3164" = "ort"
"nad.d3251" = "postleitzahl"
"nad.d3207" = "land"
"nad.c819.d3229" = "region"

[companion_fields]
"nad.d3035" = "nad_qualifier"
"nad.c082.d1131" = "codelist_code"
"nad.c082.d3055" = "codepflege_code"
"nad.c080.d3036_4" = "name_format_code"
```

**File naming convention:**
- `{entity}.toml` — primary LOC/base mapping (e.g., `marktlokation.toml`)
- `{entity}_info.toml` — SG8 SEQ group enrichment (e.g., `marktlokation_info.toml`)
- `{entity}_zuordnung.toml` — SG10 CCI/CAV attribute mapping (e.g., `marktlokation_zuordnung.toml`)
- `{entity}_rff_{qualifier}.toml` — discriminated RFF mappings (e.g., `prozessdaten_rff_z13.toml`)

**Known generator bug — entity filename collisions:**
- The `generate-pid-mappings` tool uses entity-based filenames. When SG5 and SG8 both map to the same entity (e.g., both → "Marktlokation"), the SG8 file collides with the existing SG5 file and gets silently skipped.
- Always manually verify that every group in the PID schema JSON has a TOML file. Compare the schema's `fields` tree against the TOML directory listing.
- SG6 RFF files are also commonly missed by the generator — check for them explicitly.
- After running the generator, count TOML files vs schema groups to catch gaps.

## Implementation Plans

Detailed task-level plans in `docs/plans/` — 5 features, 23 epics:
- Feature 1: `edifact-core-implementation/` (8 epics) — foundation
- Feature 2: `validation-implementation/` (3 epics) — AHB conditions & validation
- Feature 3: `generator-implementation/` (3 epics) — MIG/AHB XML codegen
- Feature 4: `web-stack-implementation/` (3 epics) — REST, gRPC, WASM frontend
- Feature 5: `mig-driven-mapping/` (6 epics) — MIG-driven pipeline, typed trees, TOML mapping, dual API

Design document: `docs/plans/2026-02-18-rust-port-design.md`
