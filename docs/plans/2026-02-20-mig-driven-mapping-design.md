# MIG-Driven Mapping Architecture

**Date:** 2026-02-20
**Status:** Proposed

## Motivation

The current hand-coded mapper approach has two compounding problems:

1. **Roundtrip fragility** — achieving byte-identical roundtrips requires bolting ordering metadata onto domain types (`nad_qualifier_order`, `raw_process_dtms`, `entity_loc_order`, `seq_group_order`). These hacks are fragile and leak transport concerns into the domain model.

2. **Maintainability burden** — adding new PIDs or format versions requires writing new mapper code by hand. The MIG and AHB XMLs already contain the structural information needed, but it's not leveraged.

**Core insight:** The MIG XML defines the canonical EDIFACT message grammar. If we parse EDIFACT into a MIG-tree-shaped structure and write it back by walking the same tree, ordering is structurally correct — no metadata hacks needed. The AHB XML defines which subset of the MIG applies per PID, giving us PID-specific typed structures for free.

## Architecture Overview

Three layers, each independently useful:

```
┌─────────────────────────────────────────────────┐
│  Layer 3: BO4E Mapping (optional)               │
│  TOML mapping files + hand-coded complex logic   │
│  MIG-tree ↔ BO4E bidirectional conversion       │
├─────────────────────────────────────────────────┤
│  Layer 2: MIG-Tree Assembly                      │
│  Generated PID-specific types (from MIG/AHB XML) │
│  Two-pass: RawSegments → typed MIG tree          │
│  Write: typed MIG tree → EDIFACT segments        │
├─────────────────────────────────────────────────┤
│  Layer 1: EDIFACT Tokenization (existing)        │
│  Streaming parser → Vec<RawSegment>              │
│  No changes needed                               │
└─────────────────────────────────────────────────┘
```

**Data flow (read):** `EDIFACT bytes → parser → Vec<RawSegment> → assembler(mig) → PidTree → (optional) BO4E`

**Data flow (write):** `BO4E → (optional) PidTree → disassembler(mig) → Vec<RawSegment> → EDIFACT bytes`

**Dual API:** Consumers choose their entry/exit point — either the typed MIG tree or BO4E objects. The MIG tree gives full structural fidelity (roundtrip correctness by construction). BO4E gives interoperability with existing German energy market systems.

## Layer 1: EDIFACT Tokenization (Existing)

The existing `edifact-parser` crate is unchanged. Its streaming parser produces `Vec<RawSegment>` from raw EDIFACT bytes. This is pass 1 of the two-pass approach.

## Layer 2: MIG-Tree Types and Assembly

### 2.1 Shared Segment Group Types (Generated from MIG)

The generator reads MIG XMLs and produces reusable Rust types for segments, composites, data elements, and segment groups. These are shared across all PIDs that reference them.

**Segments:**

```rust
// crates/mig-types/src/utilmd/segments.rs (generated)

/// NAD segment — Name and Address
pub struct SegNad {
    pub d3035_qualifier: NadQualifier,
    pub c082_party_id: Option<Composite082>,
    pub c058_name_and_address: Option<Composite058>,
    pub d3164_city: Option<String>,
    pub d3251_postcode: Option<String>,
    // ... all data elements from the MIG
}

/// LOC segment — Place/Location Identification
pub struct SegLoc {
    pub d3227_qualifier: LocQualifier,
    pub c517_location: Option<Composite517>,
}
```

**Composites:**

```rust
pub struct Composite517 {
    pub d3225_location_id: Option<String>,
    pub d1131_code_list_qualifier: Option<String>,
    pub d3055_code_list_agency: Option<String>,
}
```

**Enums from MIG code lists:**

```rust
pub enum NadQualifier { MS, MR, DP, MO, /* ... */ }
pub enum LocQualifier { Z16, Z17, Z18, Z19, /* ... */ }
pub enum SeqQualifier { Z01, Z02, Z78, /* ... */ }
```

**Segment groups compose segments:**

```rust
// crates/mig-types/src/utilmd/groups.rs (generated)

/// SG2 — Party identification (NAD + optional SG3 contact)
pub struct Sg2Party {
    pub nad: SegNad,
    pub sg3_contacts: Vec<Sg3Contact>,  // 0..n from MIG MaxRep
}

/// SG8 — SEQ-based entity group
pub struct Sg8SeqGroup {
    pub seq: SegSeq,
    pub rff: Vec<SegRff>,
    pub sg9_characteristics: Vec<Sg9Cci>,
    pub sg10_devices: Vec<Sg10Device>,
    // ... child groups per MIG
}
```

These types are generated once per MIG version and reused across all PIDs.

### 2.2 PID-Specific Composition (Generated from AHB)

Each PID gets a struct that composes the shared segment group types, with field presence and cardinality driven by the AHB.

```rust
// crates/mig-types/src/utilmd/pids/pid55035.rs (generated)

/// PID 55035: Antwort auf GDA verb. MaLo (NB an LF)
pub struct Pid55035 {
    // Message-level (AHB: Muss)
    pub unh: SegUnh,
    pub bgm: SegBgm,

    // SG2 parties — AHB defines which NAD qualifiers appear
    pub sg2_absender: Sg2Party,      // NAD+MS, Muss
    pub sg2_empfaenger: Sg2Party,    // NAD+MR, Muss

    // SG4 transactions (AHB: Muss, repeating)
    pub sg4_vorgaenge: Vec<Pid55035Vorgang>,

    pub unt: SegUnt,
}

/// Transaction-level for PID 55035
pub struct Pid55035Vorgang {
    pub ide: SegIde,                           // Muss
    pub dtm_vorgangsdaten: Vec<SegDtm>,        // Kann
    pub sg6_referenzen: Vec<Sg6Referenz>,       // Kann
    pub sg7_prozessdaten: Vec<Sg7Prozess>,      // Muss

    // SG8 — AHB says this PID has MaLo + Zuordnung groups
    pub sg8_marktlokationen: Vec<Sg8SeqGroup>,  // SEQ+Z01, Muss
    pub sg8_zuordnungen: Vec<Sg8SeqGroup>,      // SEQ+Z78, Kann
    // No sg8_messlokation for this PID — field absent, not Option
}
```

**What the AHB controls per PID:**

- **Field presence:** absent = not applicable for this PID (not `Option`, just missing)
- **Cardinality:** `Muss` → required field, `Kann` → `Option<T>` or `Vec<T>`
- **Valid qualifiers:** e.g., this PID only allows `SEQ+Z01` and `SEQ+Z78`, not `SEQ+Z02`
- **Conditions:** Bedingungen encoded as doc comments or runtime validation

**Qualifier disambiguation:** When the same segment group (e.g., SG8) appears multiple times with different SEQ qualifiers, the generator creates separate named fields. The qualifier value determines which field a parsed segment group populates.

### 2.3 Tree Assembly (Parse)

Two-pass approach — pass 1 (tokenization) already exists. Pass 2 is a new MIG-guided assembly step.

```rust
// crates/mig-assembly/src/lib.rs

/// Pass 1: existing parser (unchanged)
let segments: Vec<RawSegment> = parse_edifact(input);

/// Pass 2: MIG-guided assembly
fn assemble<P: PidTree>(
    segments: &[RawSegment],
    mig: &MigSchema,
    pid: &str,
) -> Result<P, AssemblyError>;

let tree: Pid55035 = assemble(&segments, &mig_utilmd_s21, "55035")?;
```

The assembler works like a recursive descent recognizer:

1. Start at the MIG root, maintain a cursor into the segment list
2. At each MIG node, check if the current segment matches (by tag + qualifier)
3. If it matches, consume it and fill the corresponding struct field
4. For repeating groups, loop until the next segment no longer matches
5. If a `Muss` field has no matching segment, return `AssemblyError`
6. Advance to next MIG sibling

**PID detection:** Before assembly, examine key segments (typically SG7 process data codes) to determine the PID. A dispatch function returns the PID, then assembly proceeds with the correct target type.

### 2.4 Tree Walking (Write)

The writer mirrors the assembler — walk the MIG tree in order, emit `RawSegment`s for populated fields.

```rust
fn disassemble<P: PidTree>(
    tree: &P,
    mig: &MigSchema,
) -> Vec<RawSegment>;

let segments = disassemble(&tree, &mig_utilmd_s21);
let output = render_edifact(&segments, &delimiters);
```

**Roundtrip guarantee:** `parse → assemble → disassemble → render` produces byte-identical output because the MIG tree defines the canonical ordering. The current ordering hacks (`nad_qualifier_order`, `seq_group_order`, `raw_process_dtms`, `entity_loc_order`) become unnecessary.

## Layer 3: BO4E Mapping

### 3.1 Declarative TOML Mappings (the 80%)

One TOML file per entity type, defining simple field-to-field mappings:

```toml
# mappings/marktlokation.toml

[meta]
entity = "Marktlokation"
bo4e_type = "bo4e::Marktlokation"
companion_type = "MarktlokationEdifact"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z01'"

# Simple 1:1 field mappings
[fields]
"loc.c517.d3225" = "marktlokations_id"
"loc.d3227" = { target = "lokationstyp", transform = "loc_qualifier_to_type" }
"rff.c506.d1154" = { target = "referenz_id", when = "rff.c506.d1153 == 'Z19'" }

# Nested group mappings
[fields."sg9_characteristics"]
"cci.c240.d7037" = "characteristic_code"
"cav.c889.d7111" = "characteristic_value"

# Fields that go to the companion type
[companion_fields]
"dtm.c507.d2380" = { target = "gueltig_ab", when = "dtm.c507.d2005 == '157'" }
"dtm.c507.d2380" = { target = "gueltig_bis", when = "dtm.c507.d2005 == '158'" }
```

TOML files are hand-maintained and version-controlled. They encode domain knowledge about how EDIFACT concepts map to BO4E. Validation at test time ensures all paths exist in the MIG schema and all target fields exist in the BO4E types.

### 3.2 Hand-Coded Complex Mappings (the 20%)

For cases the TOML cannot express — cross-references between groups, conditional aggregation, multi-entity relationships:

```rust
// crates/mig-bo4e/src/complex/lokationszuordnung.rs

pub fn map_zuordnung_references(
    sg8: &Sg8SeqGroup,
    ctx: &MappingContext,
) -> Result<LokationszuordnungEdifact> {
    // Complex logic that genuinely needs code
}
```

Complex handlers are registered by name and referenced from the TOML files when needed.

### 3.3 Mapping Engine

```rust
// Load TOML definitions once
let mappings = load_mappings("mappings/")?;

// Forward: MIG tree → BO4E
let bo4e = mappings.to_bo4e(&pid_tree)?;

// Reverse: BO4E → MIG tree
let pid_tree = mappings.from_bo4e(&bo4e, pid)?;
```

## Crate Structure

```
crates/
  edifact-types/          # unchanged
  edifact-parser/         # unchanged
  bo4e-extensions/        # unchanged (companion types reused)
  automapper-generator/   # extended with new codegen backends
  automapper-validation/  # unchanged (AHB conditions still useful)

  # New crates:
  mig-types/              # generated PID structs + shared segment group types
  mig-assembly/           # two-pass assembler + disassembler (tree ↔ segments)
  mig-bo4e/               # TOML mapping engine + hand-coded complex mappings

  automapper-core/        # existing — gradually deprecated
  automapper-api/         # updated to expose dual API
  automapper-web/         # updated for dual API
```

**Generator extensions in `automapper-generator`:**

- `gen_mig_types` — reads MIG XML → emits shared segment/group/enum types
- `gen_pid_types` — reads AHB XML → emits per-PID composition structs
- Output goes to `mig-types/src/generated/`

**Format version handling:** Each MIG version produces types under a version module: `mig_types::fv2504::utilmd::Pid55035`. The existing `VersionConfig` pattern selects which generated module to use.

## Migration Path

The new architecture is built alongside the existing code. Both paths coexist during migration.

1. **Phase 1:** Build `mig-types` + `mig-assembly` + `mig-bo4e`. Both old and new paths work independently.
2. **Phase 2:** Add roundtrip tests comparing old path vs new path output. Validate identical results.
3. **Phase 3:** Switch `automapper-api` to the new path. Old `automapper-core` mappers become dead code.
4. **Phase 4:** Remove old mappers once confidence is high.

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Intermediate representation | Fully generated static types per PID | Maximum compile-time safety and IDE support; can simplify to semi-generic later if compile times suffer |
| Type reuse | Shared segment group types, composed per PID | Mirrors MIG structure; avoids duplication across ~187 PIDs |
| Parse strategy | Two-pass (tokenize then assemble) | Reuses existing parser; clean separation of concerns |
| BO4E integration | Dual API — MIG tree and BO4E are both first-class | Maximum flexibility for consumers |
| Mapping format | Hybrid: TOML for simple fields, Rust for complex logic | Pragmatic — 80% of mappings are trivial 1:1, 20% need real code |
| Mapping file format | TOML | Clean separation from generated code; easy to review and diff |
