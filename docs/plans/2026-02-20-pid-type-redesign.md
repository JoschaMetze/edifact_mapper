# PID Type Redesign — AHB-Enriched Generated Types

**Date:** 2026-02-20
**Status:** Approved (brainstorming complete)

## Problem

The current generated PID types (`Pid55001`, etc.) are bare MIG skeletons:

```rust
pub struct Pid55001 {
    pub bgm: SegBgm,
    pub dtm: SegDtm,
    pub unh: SegUnh,
    pub unt: SegUnt,
    pub sg2: Vec<Sg2>,
    pub sg4: Vec<Sg4>,
}
```

They don't use AHB information (qualifier discrimination, field names, cardinality).
The generator already computes this info via `analyze_pid_structure_with_qualifiers()`
but discards it during code generation.

## Design

### 1. PID-Specific Wrapper Types

Generated structs use AHB-derived field names instead of generic `Vec<SgN>`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pid55001 {
    pub unh: SegUnh,
    pub bgm: SegBgm,
    pub dtm: SegDtm,
    pub absender: Pid55001Absender,           // SG2 where NAD+MS
    pub empfaenger: Pid55001Empfaenger,       // SG2 where NAD+MR
    pub transaktionen: Vec<Pid55001Transaktion>, // SG4 (repeating)
    pub unt: SegUnt,
}

pub struct Pid55001Absender {
    pub nad: SegNad,   // NAD+MS
    pub cci: Vec<SegCci>,
    pub com: Vec<SegCom>,
}

pub struct Pid55001Transaktion {
    pub ide: SegIde,
    pub dtm: Vec<SegDtm>,
    pub sts: Option<SegSts>,
    pub marktlokationen: Vec<Pid55001Marktlokation>,  // SG8 where LOC+Z16
    pub zaehler: Vec<Pid55001Zaehler>,                 // SG8 where LOC+Z17
    pub referenzen: Vec<Pid55001Referenz>,              // SG6
    pub marktteilnehmer: Vec<Pid55001TransaktionNad>,   // SG12
}
```

- **Qualifier discrimination**: SG2 split by NAD qualifier (MS→absender, MR→empfaenger)
- **AHB field names**: snake_cased German terms from AHB `name` field
- **Cardinality**: `Option<T>` for 0..1, `T` for exactly 1, `Vec<T>` for 0..n or 1..n

### 2. Serialized PID Schema

JSON companion file generated alongside the Rust struct:

```json
{
  "pid": "55001",
  "beschreibung": "Anmeldung MaLo",
  "kommunikation_von": "NB an LF",
  "format_version": "FV2504",
  "fields": {
    "absender": {
      "source_group": "SG2",
      "discriminator": { "segment": "NAD", "element": [0, 0], "value": "MS" },
      "cardinality": "1",
      "segments": ["NAD", "CCI", "COM"]
    },
    "transaktionen": {
      "source_group": "SG4",
      "discriminator": null,
      "cardinality": "1..n",
      "children": {
        "marktlokationen": {
          "source_group": "SG8",
          "discriminator": { "segment": "LOC", "element": [0, 0], "value": "Z16" },
          "cardinality": "0..n"
        }
      }
    }
  }
}
```

Used by the mapping engine at runtime for dynamic navigation and validation.

### 3. Generated TOML Mapping Scaffolds

Auto-generated from the PID schema with MIG paths pre-filled:

```toml
# AUTO-GENERATED scaffold for Pid55001 → absender
# Fill in "target" fields with BO4E field names.

[meta]
entity = "Marktteilnehmer"
bo4e_type = "Marktteilnehmer"
source_path = "absender"

[fields]
"nad.0" = { target = "", default = "MS" }
"nad.1.0" = { target = "" }
"nad.1.2" = { target = "" }
```

Developer fills in `target` (BO4E field name) and optional `enum_map`.
Scaffold regeneration preserves existing human-written `target`/`enum_map` values.

### 4. Direct PID Assembly (No AssembledTree Intermediate)

Pipeline:

```
EDIFACT bytes
  → parse_to_segments() → Vec<OwnedSegment>
  → detect_pid() → "55001"              (existing pid_detect.rs)
  → Pid55001::from_segments(&segments)   (generated per PID)
  → TOML mapping engine → BO4E JSON
```

Each PID type gets a generated `from_segments()` that walks segments once
using `SegmentCursor`, consuming into typed fields with qualifier discrimination:

```rust
impl Pid55001 {
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, AssemblyError> {
        let mut cursor = SegmentCursor::new(segments);

        let unh = SegUnh::consume(&mut cursor)?;
        let bgm = SegBgm::consume(&mut cursor)?;
        let dtm = SegDtm::consume(&mut cursor)?;

        // SG2: discriminate by NAD qualifier
        let mut absender = None;
        let mut empfaenger = None;
        while cursor.peek_is("NAD") {
            let nad = SegNad::consume(&mut cursor)?;
            match nad.qualifier() {
                "MS" => absender = Some(Pid55001Absender { nad, .. }),
                "MR" => empfaenger = Some(Pid55001Empfaenger { nad, .. }),
                _ => {}
            }
        }

        // SG4: repeating transactions
        let mut transaktionen = Vec::new();
        while cursor.peek_is("IDE") {
            transaktionen.push(Pid55001Transaktion::consume(&mut cursor)?);
        }

        let unt = SegUnt::consume(&mut cursor)?;
        Ok(Self { unh, bgm, dtm, absender, empfaenger, transaktionen, unt })
    }
}
```

The existing `detect_pid()` (three-tier: RFF+Z13, BGM+STS, BGM-only) stays as-is.

### 5. TOML Mapping with PID Types

Mapping files reference PID struct field paths instead of generic group indices:

```toml
[meta]
entity = "Marktteilnehmer"
bo4e_type = "Marktteilnehmer"
source_path = "absender"          # field on Pid55001 (not "SG2")

[fields]
"nad.0" = "marktrolle"
"nad.1.0" = "rollencodenummer"

[fields."nad.1.2"]
target = "rollencodetyp"
enum_map = { "293" = "BDEW", "332" = "DVGW", "500" = "GLN" }
```

Directory structure becomes PID-specific:

```
mappings/FV2504/UTILMD_Strom/
  pid_55001/
    absender.toml
    empfaenger.toml
    transaktion_prozessdaten.toml
    transaktion_marktlokation.toml
    transaktion_referenz.toml
  pid_55102/
    ...
```

The mapping engine receives segments from a typed PID field rather than
navigating `AssembledTree` groups. The segment-level extraction logic
(`extract_from_instance`) is reused with minimal adaptation.

### 6. Generated Outputs & Testing

**Per PID, the generator produces:**

1. **Rust struct** — `Pid55001` with AHB-named fields, `from_segments()` impl, `PidTree` trait impl
2. **JSON schema** — `pid_55001_schema.json` with field paths, discriminators, codes
3. **TOML scaffolds** — one per entity path, pre-filled MIG paths

**Generator input:** MIG XML + AHB XML (both already loaded).

**Testing — three layers:**

1. **Generator snapshot tests** (insta): Verify generated Rust code and JSON schema.
2. **PID assembly tests**: Parse real EDIFACT fixtures → `from_segments()` → verify typed fields.
3. **Roundtrip via TOML mapping**: PID → mapping → BO4E → reverse → PID → EDIFACT.

### 7. Migration & Coexistence

- The existing `AssembledTree` pipeline stays for `mode: "mig-tree"` in the API
- PID-direct pipeline is additive — new `mode: "pid"` or enhanced `mode: "bo4e"`
- Current TOML mappings (`source_group`) continue working until migrated to `source_path`
- Generator enhancement is incremental: enrich `analyze_pid_structure_with_qualifiers()` output
