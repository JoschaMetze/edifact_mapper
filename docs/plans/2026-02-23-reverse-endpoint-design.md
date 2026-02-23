# Reverse Endpoint: BO4E → EDIFACT for MIG-Driven Pipeline

**Date**: 2026-02-23
**Scope**: `mig-bo4e`, `automapper-api`
**Depends on**: Multi-message & transaction support (implemented)

## Problem

The MIG-driven pipeline supports EDIFACT → BO4E conversion via `POST /api/v2/convert`. The reverse direction (BO4E → EDIFACT) is needed for:

1. **Roundtrip validation** — verify `EDIFACT → BO4E → EDIFACT` produces identical output
2. **BO4E-first authoring** — generate EDIFACT messages from BO4E JSON constructed externally

The building blocks exist: `MappingEngine::map_reverse()` converts one definition's BO4E back to segments, `Disassembler` orders segments by MIG structure, and `render_edifact()` produces the EDIFACT string. What's missing is the multi-definition reverse mapping and the API endpoint.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Endpoint | Separate `POST /api/v2/reverse` | Clean separation from forward `/convert` |
| Input levels | Accept Interchange, Nachricht, or Transaktion | Flexible — roundtrip uses Interchange, authoring may use Transaktion |
| PID source | Extract from `transaktionsdaten.pruefidentifikator` | Already present in forward output (RFF+Z13 mapping) |
| Missing envelope | Generate defaults with optional overrides | Allows Transaktion-level input without requiring full wrapper |
| Reverse split | Two passes per transaction (transaktionsdaten + stammdaten) | Explicit mirror of forward split, cleaner separation |
| Output modes | `edifact` (EDIFACT string) or `mig-tree` (assembled tree JSON) | Tree mode useful for debugging |

## API Contract

### Request

```
POST /api/v2/reverse
Content-Type: application/json

{
    "input": <BO4E JSON — Interchange, Nachricht, or Transaktion>,
    "level": "interchange" | "nachricht" | "transaktion",
    "formatVersion": "FV2504",
    "mode": "edifact" | "mig-tree",
    "envelope": {                          // Optional, for missing levels
        "absenderCode": "9900123456789",
        "empfaengerCode": "9900987654321",
        "nachrichtenTyp": "UTILMD"
    }
}
```

### Response

```json
{
    "mode": "edifact",
    "result": "UNA:+.? 'UNB+UNOC:3+...'",
    "duration_ms": 12.5
}
```

Or for `mig-tree` mode:

```json
{
    "mode": "mig-tree",
    "result": { "segments": [...], "groups": [...] },
    "duration_ms": 8.2
}
```

### Input Level Handling

| Level | Input shape | PID source | Envelope |
|-------|------------|------------|----------|
| `interchange` | `{ nachrichtendaten, nachrichten: [{ unhReferenz, nachrichtenTyp, stammdaten, transaktionen }] }` | Each transaction's `transaktionsdaten.pruefidentifikator` | From `nachrichtendaten` |
| `nachricht` | `{ unhReferenz, nachrichtenTyp, stammdaten, transaktionen }` | Each transaction's `transaktionsdaten.pruefidentifikator` | UNB generated (or overrides) |
| `transaktion` | `{ stammdaten, transaktionsdaten }` | `transaktionsdaten.pruefidentifikator` | UNB + UNH generated (or overrides) |

## Reverse Pipeline

### Full flow (Interchange input)

```
Interchange JSON
  │
  ├─ Extract nachrichtendaten → rebuild UNB segment
  │
  ├─ For each Nachricht:
  │    │
  │    ├─ Extract unhReferenz, nachrichtenTyp → rebuild UNH segment
  │    │
  │    ├─ Reverse-map message-level stammdaten (msg_engine):
  │    │    map_all_reverse(nachricht.stammdaten) → SG2/SG3 groups
  │    │
  │    ├─ For each Transaktion:
  │    │    │
  │    │    ├─ Extract PID from transaktionsdaten.pruefidentifikator
  │    │    ├─ Load tx_engine for PID
  │    │    │
  │    │    ├─ Pass 1: Reverse-map transaktionsdaten
  │    │    │    → Root segments (IDE, STS, DTM) + SG6 groups (RFF)
  │    │    │
  │    │    ├─ Pass 2: Reverse-map stammdaten
  │    │    │    → SG5, SG8, SG10, SG12 groups
  │    │    │
  │    │    └─ Merge → AssembledGroupInstance (one SG4 instance)
  │    │
  │    ├─ Collect SG4 instances into AssembledGroup
  │    │
  │    ├─ Build AssembledTree:
  │    │    root segs (UNH, BGM, DTM[137]) + groups (SG2, SG4) + UNT
  │    │
  │    ├─ Disassemble (MIG-ordered) → Vec<DisassembledSegment>
  │    │
  │    └─ Render → EDIFACT string for this message
  │
  └─ Concatenate: UNA + UNB + messages + UNZ → full EDIFACT string
```

### Two-pass transaction reverse

The forward pipeline splits a transaction's mapped entities into:
- **transaktionsdaten**: entities named "Prozessdaten" or "Nachricht"
- **stammdaten**: all other entities

The reverse mirrors this with two passes:

**Pass 1** — Reverse-map transaktionsdaten:
- Filter definitions where `meta.entity ∈ ["Prozessdaten", "Nachricht"]`
- Run `map_reverse(transaktionsdaten, def)` for each
- Produces: root-level segments (IDE, STS, DTM) from `source_group = ""`
- Produces: SG6 groups (RFF+Z13, RFF+Z60) from `source_group = "SG4.SG6"`

**Pass 2** — Reverse-map stammdaten:
- Filter definitions where `meta.entity ∉ ["Prozessdaten", "Nachricht"]`
- For each entity key in stammdaten, find matching definition(s)
- Run `map_reverse(entity_value, def)` for each
- Produces: SG5 groups (LOC), SG8 groups (SEQ), SG10 groups (CCI/CAV), SG12 groups (NAD)

**Merge**: Combine root segments + all groups into one `AssembledGroupInstance`, which becomes one SG4 repetition.

### Envelope reconstruction

```rust
fn rebuild_unb(nachrichtendaten: &serde_json::Value) -> OwnedSegment {
    // UNB+UNOC:3+absenderCode:qualifier+empfaengerCode:qualifier+datum:zeit+ref
    // Fields from nachrichtendaten JSON
}

fn rebuild_unh(referenz: &str, nachrichten_typ: &str) -> OwnedSegment {
    // UNH+referenz+nachrichtenTyp:D:11A:UN:S2.1
}

fn rebuild_unt(segment_count: usize, referenz: &str) -> OwnedSegment {
    // UNT+count+referenz
}

fn rebuild_unz(message_count: usize, interchange_ref: &str) -> OwnedSegment {
    // UNZ+count+ref
}
```

For `transaktion`-level input, defaults:
- UNB: `UNOC:3`, sender/receiver from overrides or `"PLACEHOLDER"`, current datetime
- UNH: sequential ref `"00001"`, type from overrides or `"UTILMD"`
- UNT: computed after disassembly
- UNZ: message count = 1

## New Engine Methods

### map_all_reverse()

```rust
impl MappingEngine {
    /// Reverse-map a BO4E entity map back to an AssembledTree.
    ///
    /// For each definition:
    /// 1. Look up entity in input by meta.entity name
    /// 2. If entity is array, map each element as separate group repetition
    /// 3. Place results by source_group: "" → root segments, "SGn" → groups
    pub fn map_all_reverse(
        &self,
        entities: &serde_json::Value,
    ) -> AssembledTree
}
```

### map_interchange_reverse()

```rust
impl MappingEngine {
    /// Reverse-map a full MappedMessage back to an AssembledTree.
    ///
    /// Two-engine approach mirroring map_interchange():
    /// - msg_engine handles message-level stammdaten → SG2/SG3
    /// - tx_engine handles per-transaction stammdaten + transaktionsdaten → SG4
    pub fn map_interchange_reverse(
        msg_engine: &MappingEngine,
        tx_engine: &MappingEngine,
        mapped: &MappedMessage,
        transaction_group: &str,
    ) -> AssembledTree
}
```

## Change Summary

| Crate | Change | Size |
|-------|--------|------|
| `mig-bo4e` | `map_all_reverse()` on `MappingEngine` | M |
| `mig-bo4e` | `map_interchange_reverse()` on `MappingEngine` | M |
| `mig-bo4e` | Envelope reconstruction helpers in `model.rs` | S |
| `automapper-api` | `ReverseRequest`, `InputLevel`, `ReverseMode` contracts | S |
| `automapper-api` | `POST /api/v2/reverse` handler | M |
| `automapper-api` | Register route in router | S |

**Unchanged**: Disassembler, renderer, TOML mappings, forward pipeline, assembler.

## Testing Strategy

1. **Unit tests** for `map_all_reverse()` — single entity, multiple entities, array entities
2. **Unit tests** for `map_interchange_reverse()` — single transaction, multi-transaction
3. **Unit tests** for envelope reconstruction — UNB/UNH/UNT/UNZ helpers
4. **Roundtrip test** — `EDIFACT → forward → reverse → EDIFACT` byte comparison
5. **API test** — each input level (interchange, nachricht, transaktion) produces valid EDIFACT
6. **Integration test** with real PID 55001 fixture — full roundtrip via API
