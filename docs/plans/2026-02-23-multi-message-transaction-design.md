# Multi-Message & Transaction Support for MIG-Driven Pipeline

**Date**: 2026-02-23
**Scope**: `mig-assembly`, `mig-bo4e`, `automapper-api`, `mappings/`

## Problem

The MIG-driven pipeline currently processes a single EDIFACT message and returns a single flat BO4E result. Real EDIFACT files can contain:

1. **Multiple messages** (multiple UNH/UNT pairs within one UNB/UNZ interchange)
2. **Multiple transactions** within a single message (multiple top-level repeating groups, e.g. SG4 in UTILMD, each starting with an IDE segment)

The C# reference project models this as: `Interchange → Nachricht → Transaktion`. The legacy Rust pipeline (`automapper-core`) already handles this via `UtilmdNachricht` with `Vec<UtilmdTransaktion>`. The MIG-driven pipeline needs equivalent support.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Target pipeline | MIG-driven only | Legacy automapper-core already has this |
| Output model | Typed Rust structs | Compile-time safety, serialize to JSON for API |
| Type location | `mig-bo4e` crate | Output of the mapping engine, close to consumers |
| Message splitting | Split segments before assembly | Simple, each message gets independent `AssembledTree` |
| Transaction detection | MIG-group based | Top-level repeating group (SG4) = transaction. MIG-driven, not hardcoded segment names |
| TOML level separation | Separate directories | `message/` vs `{pid}/` subdirectories |
| API compatibility | Breaking change on v2 | v2 is new/internal, just change the format |

## Data Model

Three new structs in `mig-bo4e::model`:

```rust
/// Full EDIFACT interchange (UNB...UNZ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interchange {
    /// Service segment data (UNA delimiters, UNB sender/receiver, UNZ control)
    pub nachrichtendaten: Nachrichtendaten,
    /// One entry per UNH/UNT pair
    pub nachrichten: Vec<Nachricht>,
}

/// Single EDIFACT message (UNH...UNT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nachricht {
    /// UNH message reference number
    pub unh_referenz: String,
    /// Message type identifier (e.g. "UTILMD", "ORDERS")
    pub nachrichten_typ: String,
    /// Message-level entities (SG2 Marktteilnehmer, SG3 Ansprechpartner)
    pub stammdaten: serde_json::Value,
    /// One entry per top-level transaction group (SG4 in UTILMD)
    pub transaktionen: Vec<Transaktion>,
}

/// Single transaction within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaktion {
    /// BO4E entities from TOML mappings (Marktlokation, Messlokation, etc.)
    pub stammdaten: serde_json::Value,
    /// Process metadata (IDE, STS, DTM — segments in the transaction group root)
    pub transaktionsdaten: serde_json::Value,
}
```

`Nachrichtendaten` already exists in `bo4e-extensions` for UNA/UNB/UNZ service data.

## Pipeline Changes

### 1. Segment Splitting (`mig-assembly::tokenize`)

New types and function to split tokenized segments at UNH/UNT boundaries:

```rust
pub struct MessageChunk {
    pub unh: OwnedSegment,
    pub segments: Vec<OwnedSegment>,  // Between UNH and UNT (exclusive)
    pub unt: OwnedSegment,
}

pub struct InterchangeChunks {
    pub pre_message: Vec<OwnedSegment>,   // UNA, UNB
    pub messages: Vec<MessageChunk>,
    pub post_message: Vec<OwnedSegment>,  // UNZ
}

pub fn split_messages(segments: Vec<OwnedSegment>) -> InterchangeChunks
```

Walks the flat `Vec<OwnedSegment>`, accumulates segments into `MessageChunk`s at UNH/UNT boundaries. The existing `parse_to_segments()` is unchanged — `split_messages` is a post-processing step.

### 2. ConversionService (`mig-assembly::service`)

New method that processes the full interchange:

```rust
impl ConversionService {
    pub fn convert_interchange(&self, input: &[u8]) -> Result<InterchangeResult> {
        let segments = parse_to_segments(input)?;
        let chunks = split_messages(segments);

        let messages = chunks.messages.iter().map(|msg| {
            let pid = detect_pid(&msg.segments)?;
            let filtered_mig = self.get_filtered_mig(pid)?;
            let tree = Assembler::new(&filtered_mig)
                .assemble_generic(&msg.all_segments())?;
            Ok(MessageResult { pid, tree, unh: msg.unh.clone(), unt: msg.unt.clone() })
        }).collect::<Result<Vec<_>>>()?;

        Ok(InterchangeResult {
            pre_message: chunks.pre_message,
            messages,
            post_message: chunks.post_message,
        })
    }
}
```

Each message within an interchange gets independent PID detection and MIG filtering. A single file could contain messages with different PIDs.

### 3. Transaction Group Scoping (`mig-bo4e::engine`)

The `MappingEngine` gains a higher-level method:

```rust
impl MappingEngine {
    pub fn map_interchange(
        &self,
        interchange_result: &InterchangeResult,
        message_defs: &[MappingDefinition],
        transaction_defs: &[MappingDefinition],
        transaction_group_name: &str,  // "SG4" for UTILMD
    ) -> Interchange {
        let nachrichtendaten = extract_nachrichtendaten(&interchange_result.pre_message);

        let nachrichten = interchange_result.messages.iter().map(|msg| {
            // Map message-level entities (SG2/SG3) against full tree
            let stammdaten = self.map_definitions(&msg.tree, message_defs);

            // Each instance of the transaction group = one Transaktion
            let transaction_groups = msg.tree.groups.iter()
                .filter(|g| g.name == transaction_group_name);

            let transaktionen = transaction_groups.map(|sg4| {
                let sub_tree = sg4.as_assembled_tree();
                let stammdaten = self.map_definitions(&sub_tree, transaction_defs);
                let transaktionsdaten = extract_transaktionsdaten(&sub_tree);
                Transaktion { stammdaten, transaktionsdaten }
            }).collect();

            Nachricht {
                unh_referenz: extract_unh_ref(&msg.unh),
                nachrichten_typ: extract_msg_type(&msg.unh),
                transaktionen,
                stammdaten,
            }
        }).collect();

        Interchange { nachrichtendaten, nachrichten }
    }
}
```

The existing `map_forward()` and `map_definitions()` work unchanged — they just operate on different tree scopes (full tree for message-level, sub-tree for transaction-level).

`transaction_group_name` is derived from the MIG structure: the top-level repeating group that contains the business content. For UTILMD this is `"SG4"`. For other message types it could differ — the MIG XML defines this.

### 4. TOML Directory Reorganization

```
mappings/FV2504/UTILMD_Strom/
├── message/                          # Message-level (applied once per Nachricht)
│   ├── marktteilnehmer.toml          # SG2 NAD+MS/MR
│   └── ansprechpartner.toml          # SG3 CTA/COM
├── 55001/                            # Transaction-level (applied per SG4)
│   ├── prozessdaten.toml             # SG4 root: IDE, STS, DTM
│   ├── marktlokation.toml            # SG5 LOC+Z16
│   ├── marktlokation_info.toml       # SG8 SEQ
│   ├── marktlokation_zuordnung.toml  # SG10 CCI/CAV
│   └── ...
├── 55002/
│   ├── prozessdaten.toml
│   └── ...
```

The engine loads `message/*.toml` as message-level definitions and `{pid}/*.toml` as transaction-level definitions. Existing TOML files move to PID subdirectories (no content changes, just file moves).

### 5. API Response Format (`automapper-api`)

The v2 `/api/v2/convert` response becomes:

```json
{
  "mode": "bo4e",
  "result": {
    "nachrichtendaten": {
      "una": ":+.? '",
      "absender": "9900123456789",
      "empfaenger": "9900987654321"
    },
    "nachrichten": [
      {
        "unh_referenz": "00001",
        "nachrichten_typ": "UTILMD",
        "stammdaten": {
          "Marktteilnehmer": [...]
        },
        "transaktionen": [
          {
            "stammdaten": {
              "Marktlokation": [...],
              "Messlokation": [...],
              "Netzlokation": [...]
            },
            "transaktionsdaten": {
              "transaktions_id": "T001",
              "kategorie": "E01",
              "transaktionsgrund": "E03"
            }
          }
        ]
      }
    ]
  }
}
```

For single-message, single-transaction files (the common case), both `nachrichten` and `transaktionen` are 1-element arrays.

## Change Summary

| Crate | Change | Size |
|-------|--------|------|
| `mig-assembly` | `split_messages()`, `InterchangeChunks`, `MessageChunk` types | S |
| `mig-assembly` | `convert_interchange()` on `ConversionService` | M |
| `mig-bo4e` | `Interchange`, `Nachricht`, `Transaktion` structs | S |
| `mig-bo4e` | `map_interchange()` on `MappingEngine` | M |
| `mig-bo4e` | Sub-tree scoping for transaction groups (`as_assembled_tree()`) | M |
| `mig-bo4e` | TOML loader: `message/` + `{pid}/` directory support | S |
| `automapper-api` | v2 convert handler: hierarchical response | S |
| `mappings/` | Move SG2/SG3 TOMLs to `message/`, existing PID TOMLs to `{pid}/` | S |

**Unchanged**: `edifact-parser`, `edifact-types`, `bo4e-extensions`, assembler/disassembler internals, TOML mapping file content.

## Testing Strategy

1. **Unit tests** for `split_messages()` — single message, multi-message, empty input
2. **Unit tests** for sub-tree scoping — extract SG4 instances from assembled tree
3. **Integration tests** with UTILMD fixtures — single-message files produce 1-element arrays
4. **Multi-message test** — construct or find a multi-message UTILMD fixture, verify correct splitting
5. **Roundtrip test** — `Interchange` serializes to JSON and back correctly
6. **API test** — v2 endpoint returns the new hierarchical format

## C# Reference Correspondence

| C# Concept | Rust MIG-Driven Equivalent |
|------------|---------------------------|
| `Nachricht<T>` | `mig_bo4e::model::Nachricht` |
| `Transaktion<T>` | `mig_bo4e::model::Transaktion` |
| `Nachrichtendaten` (ExpandoObject) | `bo4e_extensions::Nachrichtendaten` (already exists) |
| `UtilmdTransaktionsdaten` | `transaktionsdaten: serde_json::Value` (dynamic, from TOML mappings) |
| `CoordinatorBase.OnMessageStart/End` | `split_messages()` + per-message assembly |
| `GetNachrichtResult()` | `MappingEngine::map_interchange()` |
| IDE detection in coordinator switch | MIG-group based: top-level repeating group = transaction |
