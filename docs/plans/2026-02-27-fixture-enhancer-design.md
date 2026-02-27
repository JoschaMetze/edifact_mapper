# Fixture Enhancer Design

## Problem

The `generate-fixture` command produces structurally valid EDIFACT fixtures but with obvious placeholder values (`TESTID`, `GENERATED00001`, `1234567890128`, `Mustermann`). These fixtures are useful for structural validation but not for realistic integration testing or diverse code path coverage.

## Solution

A fixture enhancer that takes a generated EDIFACT fixture, forward-maps it to BO4E JSON via the existing TOML mapping engine, replaces placeholder values with realistic German energy market data, then reverse-maps back to EDIFACT. The roundtrip pipeline guarantees structural correctness.

## Pipeline

```
PID Schema JSON
  |
  v
generate_fixture()              raw EDIFACT (placeholder values)
  |
  v
tokenize + assemble(filtered MIG)
  |
  v
map_interchange()               MappedMessage (BO4E JSON with placeholders)
  |
  v
enhance_mapped_message()        MappedMessage (realistic values + varied codes)
  |
  v
map_interchange_reverse()       AssembledTree (enhanced)
  |
  v
disassemble + render            enhanced EDIFACT fixture
```

## Field Recognition

The enhancer identifies fields by their BO4E target name from the TOML mappings. This is more reliable than raw EDIFACT element IDs since the TOML author already assigned semantic names.

### Energy Market ID Generators

Each location/resource type has its own ID format with check digit generation:

| Field name | Type | Format | Example |
|---|---|---|---|
| `marktlokationsId` | MaLo | 11 digits, check digit (mod 10) | `52234567891` |
| `messlokationsId` | MeLo | `DE` + 31 alphanum + check char | `DE0052234567891000000000000000017` |
| `netzlokationsId` | NeLo | 10 digits, `E` prefix | `E5223456789` |
| `steuerbareRessourceId` | SteuRess | `C` + 10 digits | `C5223456789` |
| `technischeRessourceId` | TechRess | `D` + 10 digits | `D5223456789` |
| `tranchenId` | Tranche | 11 digits (like MaLo) | `62234567892` |

LOC qualifier context determines the generator: `LOC+Z16` = MaLo, `LOC+Z17` = MeLo, `LOC+Z18` = NeLo, `LOC+Z19` = SteuRess, `LOC+Z20` = TechRess, `LOC+Z22` = MaLo.

### General Field Patterns

| Pattern | Generator | Example |
|---|---|---|
| `identifikation`, `*Id` (GLN context) | 13-digit GLN with check digit | `9900259000003` |
| `vorgangId`, `*referenz` | Business reference string | `VORG-2025-A7B3C9` |
| `nachname`, `vorname`, `titel`, `anrede` | From name seed data | `Müller`, `Anna`, `Dr.`, `Frau` |
| `strasse`, `hausnummer` | From address seed data | `Berliner Str.`, `42a` |
| `ort`, `postleitzahl`, `land`, `region` | From city seed data | `Hamburg`, `20095`, `DE`, `HH` |
| `*datum`, `gueltigAb`, `gueltigBis` | Date in EDIFACT format (303) | `20250401120000+00` |
| `*_qualifier`, `*Code` (companion) | Untouched (structural) | (unchanged) |
| Code-type fields (from schema) | Sampled from AHB code list | varies by variant |

## Seed Data

Embedded static const arrays compiled into the binary. No external files or dependencies.

**Names:** ~20 German surnames, ~20 first names, titles (`Dr.`, `Prof.`), salutations (`Herr`, `Frau`).

**Addresses:** ~30 coherent tuples of `(strasse, hausnummer, plz, ort, bundesland)`. Kept as tuples so street/PLZ/city/region always belong together geographically.

**GLNs:** ~10 synthetic 13-digit GLNs with valid check digits for party identifiers.

Selection is deterministic: `seed + field_position` indexes into arrays. Different transactions within one fixture get different people/addresses. Same seed reproduces identical output.

## Variant Generation

For diverse test coverage, the enhancer varies code-type fields across fixtures using the PID schema's AHB-filtered code lists.

**Mechanism:** Each code-type field picks `codes[variant_index % codes.len()]` instead of always `codes[0]`. This rotates through valid codes deterministically.

**Constraints:**
- Discriminator codes are never varied (structural: `LOC.d3227=Z16` must stay `Z16`).
- Default-only fields (`target = ""`) are untouched (fixed qualifiers).
- Only fields with a real BO4E target name AND code type get varied.

## CLI Interface

```bash
# Single enhanced fixture
cargo run -p automapper-generator -- generate-fixture \
  --pid-schema pid_55001_schema.json \
  --output /tmp/55001_enhanced.edi \
  --enhance \
  --seed 42 \
  --mig-xml UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-xml UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml

# Multiple variants
cargo run -p automapper-generator -- generate-fixture \
  --pid-schema pid_55001_schema.json \
  --output /tmp/55001.edi \
  --enhance \
  --variants 5 \
  --seed 42 \
  --mig-xml ... --ahb-xml ...
```

With `--variants 5`: produces `55001_v0.edi` through `55001_v4.edi`, each with different realistic data and code combinations. Without `--variants`: one enhanced fixture.

Requires `--mig-xml` and `--ahb-xml` (same as `--validate`). TOML mapping directories are resolved from the format version and message type.

## Implementation

### New Files

| File | Purpose |
|---|---|
| `fixture_generator/enhancer.rs` | `enhance_mapped_message()`, field recognition, value replacement |
| `fixture_generator/seed_data.rs` | Static const arrays: names, addresses, GLNs |
| `fixture_generator/id_generators.rs` | MaLo, MeLo, NeLo, SteuRess, TechRess ID generators with check digits |

### Modified Files

| File | Change |
|---|---|
| `fixture_generator/mod.rs` | Add modules, public `generate_enhanced_fixture()` entry point |
| `main.rs` | Add `--enhance`, `--variants`, `--seed` flags to `GenerateFixture` |

### Entry Point

```rust
fn generate_enhanced_fixture(
    schema: &Value,
    filtered_mig: &MigSchema,
    msg_engine: &MappingEngine,
    tx_engine: &MappingEngine,
    seed: u64,
    variant: usize,
) -> String {
    let edi = generate_fixture(schema);
    let segments = parse_to_segments(edi.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    let msg_segs = /* UNH + body + UNT */;
    let assembler = Assembler::new(&filtered_mig);
    let tree = assembler.assemble_generic(&msg_segs).unwrap();
    let mut mapped = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", true);
    enhance_mapped_message(&mut mapped, schema, seed, variant);
    let reverse_tree = MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");
    // add UNH/UNT, disassemble, render
}
```

## Scope & Constraints

**Does NOT:**
- Modify UNB/UNZ envelope segments (outside mapping engine scope).
- Invent or remove segments (structure stays identical).
- Validate cross-field business rules (validation layer's job).
- Enhance PIDs without TOML mappings (falls back to unenhanced with warning).

**Guarantees:**
- Deterministic: same `(seed, variant)` = same fixture.
- Roundtrip-safe: enhanced fixtures pass MIG assembly roundtrip.
- Structurally valid: all code values from PID schema's AHB-filtered lists.

**v1 limitations:**
- UTILMD only (only message type with TOML mappings).
- German-only seed data (matches energy market domain).
- Enhancement coverage depends on TOML mapping coverage — unmapped segments keep placeholders.
