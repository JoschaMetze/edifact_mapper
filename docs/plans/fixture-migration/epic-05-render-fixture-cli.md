---
feature: fixture-migration
epic: 5
title: "render-fixture CLI Subcommand"
depends_on: []
estimated_tasks: 4
crate: automapper-generator
status: in_progress
---

# Epic 5: `render-fixture` CLI Subcommand

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add a `render-fixture` subcommand that takes a canonical `.mig.bo.json` file and renders it through version-specific TOML mappings + MIG/AHB XMLs to produce a golden `.edi` fixture. This is Phase 3 of the migration pipeline — used after TOML mappings exist for the target format version.

**Architecture:** The handler loads the BO4E JSON, the MIG/AHB XMLs (for PID filtering and disassembly), and the TOML mappings (for reverse mapping). It then:
1. Loads message and transaction `MappingEngine`s from split TOML directories
2. Builds a `MappedMessage` from the BO4E JSON
3. Calls `map_interchange_reverse()` to produce an `AssembledTree`
4. Filters MIG by PID's AHB numbers for the `Disassembler`
5. Disassembles the tree into segments
6. Reconstructs UNB/UNH/UNT/UNZ envelope
7. Renders the final EDIFACT string

This reuses the existing reverse pipeline from `mig-bo4e` and `mig-assembly`. The new code is primarily CLI plumbing and the `.mig.bo.json` format contract.

**Existing code:**
- `MappingEngine::load_split()` at `mig-bo4e::engine` — loads message + transaction engines
- `MappingEngine::map_interchange_reverse()` at `mig-bo4e::engine` — two-pass reverse mapping
- `rebuild_unb/unh/unt/unz()` at `mig-bo4e::model` — envelope segment builders
- `Disassembler::disassemble()` at `mig-assembly::disassembler` — tree → segments
- `render_edifact()` at `mig-assembly::renderer` — segments → EDIFACT string
- `parse_mig()` at `automapper-generator::parsing::mig_parser` — load MIG XML
- `parse_ahb()` at `automapper-generator::parsing::ahb_parser` — load AHB XML
- `pid_filter::filter_mig_for_pid()` at `mig-assembly` — filter MIG to PID subset
- `ConversionService` at `mig-assembly::service` — high-level assembly facade

**Dependencies:**
- `automapper-generator` Cargo.toml needs `mig-bo4e` as a dependency (check if already present)

---

## Task 1: Define `.mig.bo.json` Format Contract

**Files:**
- Create: `crates/automapper-generator/src/fixture_renderer/mod.rs`
- Create: `crates/automapper-generator/src/fixture_renderer/types.rs`
- Modify: `crates/automapper-generator/src/lib.rs` — add `pub mod fixture_renderer;`
- Create: `crates/automapper-generator/tests/fixture_renderer_test.rs`

**Step 1: Write the types and test**

The `.mig.bo.json` format mirrors the `MappedMessage` structure from `mig-bo4e::model`:

`crates/automapper-generator/src/fixture_renderer/types.rs`:
```rust
use serde::{Deserialize, Serialize};

/// The canonical BO4E representation stored in `.mig.bo.json` files.
///
/// This is the version-independent business content. Each format version
/// renders this into a different EDIFACT wire format via TOML mappings.
///
/// Structure mirrors `mig_bo4e::model::MappedMessage` with added metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalBo4e {
    /// Metadata about the source fixture.
    pub meta: CanonicalMeta,
    /// Interchange-level data (sender/receiver, date, reference).
    pub nachrichtendaten: serde_json::Value,
    /// Message-level data (UNH reference, type, message-level entities).
    pub nachricht: NachrichtBo4e,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMeta {
    /// PID that this fixture represents.
    pub pid: String,
    /// Message type (e.g., "UTILMD").
    pub message_type: String,
    /// Original format version this was derived from.
    pub source_format_version: String,
    /// Original fixture filename.
    pub source_fixture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NachrichtBo4e {
    /// UNH reference number.
    pub unh_referenz: String,
    /// Message subtype (e.g., "UTILMD:D:11A:UN:S2.1").
    pub nachrichten_typ: String,
    /// Message-level entities (Marktteilnehmer, Ansprechpartner).
    pub stammdaten: serde_json::Value,
    /// Transaction-level data (one per SG4 repetition).
    pub transaktionen: Vec<TransaktionBo4e>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransaktionBo4e {
    /// Transaction-level entities (Marktlokation, Messlokation, etc.).
    pub stammdaten: serde_json::Value,
    /// Process data (Prozessdaten, Nachricht).
    pub transaktionsdaten: serde_json::Value,
}
```

`crates/automapper-generator/src/fixture_renderer/mod.rs`:
```rust
pub mod types;
pub mod renderer;

pub use types::*;
pub use renderer::*;
```

`crates/automapper-generator/tests/fixture_renderer_test.rs`:
```rust
//! Tests for the fixture renderer (BO4E → EDIFACT via TOML mappings).

use automapper_generator::fixture_renderer::CanonicalBo4e;

#[test]
fn test_canonical_bo4e_roundtrip_serialization() {
    let canonical = CanonicalBo4e {
        meta: automapper_generator::fixture_renderer::CanonicalMeta {
            pid: "55001".into(),
            message_type: "UTILMD".into(),
            source_format_version: "FV2504".into(),
            source_fixture: "55001_UTILMD_S2.1_ALEXANDE121980.edi".into(),
        },
        nachrichtendaten: serde_json::json!({
            "absender": "9978842000002",
            "empfaenger": "9900269000000",
            "erstellungsdatum": "250331:1329",
            "referenznummer": "ALEXANDE121980"
        }),
        nachricht: automapper_generator::fixture_renderer::NachrichtBo4e {
            unh_referenz: "ALEXANDE951842".into(),
            nachrichten_typ: "UTILMD:D:11A:UN:S2.1".into(),
            stammdaten: serde_json::json!({
                "Marktteilnehmer": [
                    {"marktrolle": "MS", "rollencodenummer": "9978842000002"},
                    {"marktrolle": "MR", "rollencodenummer": "9900269000000"}
                ]
            }),
            transaktionen: vec![
                automapper_generator::fixture_renderer::TransaktionBo4e {
                    stammdaten: serde_json::json!({
                        "Marktlokation": [{"malo_id": "12345678900"}]
                    }),
                    transaktionsdaten: serde_json::json!({
                        "Prozessdaten": [{"pruefidentifikator": "55001"}]
                    }),
                },
            ],
        },
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&canonical).unwrap();
    assert!(json.contains("55001"));
    assert!(json.contains("ALEXANDE121980"));

    // Deserialize back
    let parsed: CanonicalBo4e = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.meta.pid, "55001");
    assert_eq!(parsed.nachricht.transaktionen.len(), 1);
}
```

**Step 2: Add module to lib.rs**

In `crates/automapper-generator/src/lib.rs`:
```rust
pub mod fixture_renderer;
```

**Step 3: Run test**

Run: `cargo test -p automapper-generator test_canonical_bo4e`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/fixture_renderer/ crates/automapper-generator/src/lib.rs crates/automapper-generator/tests/fixture_renderer_test.rs
git commit -m "feat(generator): add CanonicalBo4e types for .mig.bo.json format"
```

---

## Task 2: Implement `render_fixture()` Core Function

**Files:**
- Create: `crates/automapper-generator/src/fixture_renderer/renderer.rs`
- Modify: `crates/automapper-generator/tests/fixture_renderer_test.rs`
- Possibly modify: `crates/automapper-generator/Cargo.toml` — add `mig-bo4e` dependency

**Step 1: Check and update Cargo.toml**

First check if `mig-bo4e` and `mig-assembly` are already dependencies:

Run: `grep -E "mig-bo4e|mig-assembly" crates/automapper-generator/Cargo.toml`

If not present, add them:
```toml
[dependencies]
mig-assembly = { path = "../mig-assembly" }
mig-bo4e = { path = "../mig-bo4e" }
```

**Step 2: Write the failing test**

Add to `fixture_renderer_test.rs`:
```rust
use automapper_generator::fixture_renderer::{render_fixture, RenderInput};
use std::path::Path;

#[test]
fn test_render_fixture_produces_valid_edifact() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let mig_path = base.join(
        "xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    let ahb_path = base.join(
        "xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml",
    );
    let msg_mappings = base.join("mappings/FV2504/UTILMD_Strom/message");
    let tx_mappings = base.join("mappings/FV2504/UTILMD_Strom/pid_55001");

    if !mig_path.exists() || !ahb_path.exists() || !msg_mappings.exists() || !tx_mappings.exists()
    {
        eprintln!("Skipping: required files not found");
        return;
    }

    // First, create canonical BO4E by forward-mapping a real fixture
    let fixture_path = base.join(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if !fixture_path.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }

    let input = RenderInput {
        mig_xml_path: mig_path,
        ahb_xml_path: ahb_path,
        message_mappings_dir: msg_mappings,
        transaction_mappings_dir: tx_mappings,
        message_type: "UTILMD".into(),
        variant: Some("Strom".into()),
        format_version: "FV2504".into(),
        pid: "55001".into(),
    };

    // For now, test that the function exists and can load all required resources.
    // Full roundtrip test will use a .mig.bo.json produced by forward mapping.
    let result = render_fixture(&fixture_path, &input);
    assert!(result.is_ok(), "render_fixture should succeed: {:?}", result.err());

    let edifact = result.unwrap();
    assert!(edifact.contains("UNB+"), "Should have UNB segment");
    assert!(edifact.contains("UNH+"), "Should have UNH segment");
    assert!(edifact.contains("UNT+"), "Should have UNT segment");
    assert!(edifact.contains("UNZ+"), "Should have UNZ segment");
    eprintln!("Rendered EDIFACT ({} bytes):\n{}", edifact.len(), &edifact[..edifact.len().min(500)]);
}
```

**Step 3: Write the implementation**

`crates/automapper-generator/src/fixture_renderer/renderer.rs`:

```rust
use crate::error::GeneratorError;
use crate::parsing::{ahb_parser, mig_parser};
use std::path::{Path, PathBuf};

/// Input configuration for rendering a fixture.
pub struct RenderInput {
    pub mig_xml_path: PathBuf,
    pub ahb_xml_path: PathBuf,
    pub message_mappings_dir: PathBuf,
    pub transaction_mappings_dir: PathBuf,
    pub message_type: String,
    pub variant: Option<String>,
    pub format_version: String,
    pub pid: String,
}

/// Render an EDIFACT fixture from a source .edi file through the full
/// forward → reverse roundtrip pipeline.
///
/// This validates that the TOML mappings can produce a complete EDIFACT message.
/// In Phase 3 usage, this will take a .mig.bo.json as input instead.
pub fn render_fixture(
    source_edi_path: &Path,
    input: &RenderInput,
) -> Result<String, GeneratorError> {
    // 1. Parse MIG and AHB
    let mig = mig_parser::parse_mig(
        &input.mig_xml_path,
        &input.message_type,
        input.variant.as_deref(),
        &input.format_version,
    )?;

    let ahb = ahb_parser::parse_ahb(
        &input.ahb_xml_path,
        &input.message_type,
        input.variant.as_deref(),
        &input.format_version,
    )?;

    // 2. Find PID in AHB
    let pid_def = ahb
        .workflows
        .iter()
        .find(|w| w.id == input.pid)
        .ok_or_else(|| GeneratorError::Validation {
            message: format!("PID {} not found in AHB", input.pid),
        })?;

    // 3. Filter MIG for PID
    let filtered_mig =
        mig_assembly::pid_filter::filter_mig_for_pid(&mig, &pid_def.segment_numbers);

    // 4. Read and tokenize source EDIFACT
    let edi_bytes = std::fs::read(source_edi_path)?;
    let segments = mig_assembly::tokenize::parse_to_segments(&edi_bytes)
        .map_err(|e| GeneratorError::Validation {
            message: format!("Failed to tokenize EDIFACT: {}", e),
        })?;

    // 5. Split into interchange chunks
    let chunks = mig_assembly::tokenize::split_messages(segments)
        .map_err(|e| GeneratorError::Validation {
            message: format!("Failed to split messages: {}", e),
        })?;

    // 6. Load mapping engines
    let (msg_engine, tx_engine) = mig_bo4e::engine::MappingEngine::load_split(
        &input.message_mappings_dir,
        &input.transaction_mappings_dir,
    )
    .map_err(|e| GeneratorError::Validation {
        message: format!("Failed to load TOML mappings: {}", e),
    })?;

    // 7. Assemble and forward-map each message, then reverse-map back
    let assembler = mig_assembly::assembler::Assembler::new(&filtered_mig);
    let disassembler = mig_assembly::disassembler::Disassembler::new(&filtered_mig);

    let mut all_rendered_messages = Vec::new();

    for msg_chunk in &chunks.messages {
        // Assemble message content (UNH body, excluding UNH/UNT themselves)
        let tree = assembler
            .assemble_generic(&msg_chunk.body)
            .map_err(|e| GeneratorError::Validation {
                message: format!("Assembly failed: {}", e),
            })?;

        // Forward map to BO4E
        let mapped = mig_bo4e::engine::MappingEngine::map_interchange(
            &msg_engine,
            &tx_engine,
            &tree,
            "SG4",
        );

        // Reverse map back to tree
        let reverse_tree = mig_bo4e::engine::MappingEngine::map_interchange_reverse(
            &msg_engine,
            &tx_engine,
            &mapped,
            "SG4",
        );

        // Disassemble to segments
        let disassembled = disassembler.disassemble(&reverse_tree);

        // Reconstruct UNH/UNT
        let unh = mig_bo4e::model::rebuild_unh(
            &msg_chunk.unh.get_element(0),
            &msg_chunk.unh.elements.get(1)
                .map(|e| e.join(":"))
                .unwrap_or_default(),
        );
        let segment_count = disassembled.len() + 2; // +UNH +UNT
        let unt = mig_bo4e::model::rebuild_unt(
            segment_count,
            &msg_chunk.unh.get_element(0),
        );

        // Render message segments
        let delimiters = edifact_types::EdifactDelimiters::default();
        let mut msg_segments = vec![mig_assembly::disassembler::DisassembledSegment {
            tag: unh.id.clone(),
            elements: unh.elements.clone(),
        }];
        msg_segments.extend(disassembled);
        msg_segments.push(mig_assembly::disassembler::DisassembledSegment {
            tag: unt.id.clone(),
            elements: unt.elements.clone(),
        });

        let rendered = mig_assembly::renderer::render_edifact(&msg_segments, &delimiters);
        all_rendered_messages.push(rendered);
    }

    // Reconstruct UNB and UNZ
    let unb_str = if let Some(ref env) = chunks.envelope.first() {
        let delimiters = edifact_types::EdifactDelimiters::default();
        let unb_seg = mig_assembly::disassembler::DisassembledSegment {
            tag: env.id.clone(),
            elements: env.elements.clone(),
        };
        mig_assembly::renderer::render_edifact(&[unb_seg], &delimiters)
    } else {
        String::new()
    };

    let unz_str = if let Some(ref unz) = chunks.unz {
        let delimiters = edifact_types::EdifactDelimiters::default();
        let unz_seg = mig_assembly::disassembler::DisassembledSegment {
            tag: unz.id.clone(),
            elements: unz.elements.clone(),
        };
        mig_assembly::renderer::render_edifact(&[unz_seg], &delimiters)
    } else {
        String::new()
    };

    // Combine all parts
    let mut result = String::new();
    result.push_str(&unb_str);
    for msg in &all_rendered_messages {
        result.push_str(msg);
    }
    result.push_str(&unz_str);

    Ok(result)
}
```

**Step 4: Run test**

Run: `cargo test -p automapper-generator test_render_fixture -- --nocapture`
Expected: PASS (or skip if fixtures not available)

Note: This test may need adjustments based on the exact API signatures of `mig_assembly` and `mig_bo4e`. The implementing engineer should resolve import issues and type mismatches as they arise — the logic is correct, but exact method signatures may differ slightly. Check the existing reverse endpoint code at `crates/automapper-api/src/routes/reverse_v2.rs` for the reference pattern.

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/fixture_renderer/renderer.rs crates/automapper-generator/Cargo.toml crates/automapper-generator/tests/fixture_renderer_test.rs
git commit -m "feat(generator): add render_fixture() for BO4E roundtrip rendering"
```

---

## Task 3: Add `RenderFixture` CLI Subcommand

**Files:**
- Modify: `crates/automapper-generator/src/main.rs`

**Step 1: Add the CLI variant and handler**

Add to `Commands` enum:
```rust
/// Render an EDIFACT fixture from a source .edi through forward+reverse mapping.
/// Phase 3: validates TOML mappings can produce complete EDIFACT output.
RenderFixture {
    /// Path to the source .edi fixture (or .mig.bo.json in Phase 3).
    #[arg(long)]
    source: PathBuf,

    /// Path to the MIG XML file.
    #[arg(long)]
    mig_xml: PathBuf,

    /// Path to the AHB XML file.
    #[arg(long)]
    ahb_xml: PathBuf,

    /// Path to message-level TOML mappings directory.
    #[arg(long)]
    message_mappings: PathBuf,

    /// Path to transaction-level TOML mappings directory.
    #[arg(long)]
    transaction_mappings: PathBuf,

    /// Message type (e.g., "UTILMD").
    #[arg(long)]
    message_type: String,

    /// Message variant (e.g., "Strom", "Gas").
    #[arg(long)]
    variant: Option<String>,

    /// Format version (e.g., "FV2504").
    #[arg(long)]
    format_version: String,

    /// PID identifier (e.g., "55001").
    #[arg(long)]
    pid: String,

    /// Output path for the rendered .edi file.
    #[arg(long)]
    output: PathBuf,
},
```

Add the handler:
```rust
Commands::RenderFixture {
    source,
    mig_xml,
    ahb_xml,
    message_mappings,
    transaction_mappings,
    message_type,
    variant,
    format_version,
    pid,
    output,
} => {
    use crate::fixture_renderer::{render_fixture, RenderInput};

    let input = RenderInput {
        mig_xml_path: mig_xml,
        ahb_xml_path: ahb_xml,
        message_mappings_dir: message_mappings,
        transaction_mappings_dir: transaction_mappings,
        message_type,
        variant,
        format_version,
        pid: pid.clone(),
    };

    let edifact = render_fixture(&source, &input)?;

    std::fs::create_dir_all(output.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(&output, &edifact)?;

    println!("Rendered fixture: {} ({} bytes)", output.display(), edifact.len());
    println!("PID: {}", pid);
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/automapper-generator/src/main.rs
git commit -m "feat(generator): add render-fixture CLI subcommand"
```

---

## Task 4: Generate `.mig.bo.json` from Existing Fixture (Canonical BO4E Bootstrap)

**Files:**
- Modify: `crates/automapper-generator/src/main.rs`
- Modify: `crates/automapper-generator/tests/fixture_renderer_test.rs`

This task adds a `generate-canonical-bo4e` subcommand that forward-maps an existing `.edi` fixture and writes the `.mig.bo.json` — bootstrapping the canonical test corpus from existing FV2504 fixtures.

**Step 1: Add the CLI variant**

Add to `Commands` enum:
```rust
/// Generate a canonical .mig.bo.json from an existing .edi fixture.
/// Bootstraps the version-independent test corpus.
GenerateCanonicalBo4e {
    /// Path to the source .edi fixture.
    #[arg(long)]
    source: PathBuf,

    /// Path to the MIG XML file.
    #[arg(long)]
    mig_xml: PathBuf,

    /// Path to the AHB XML file.
    #[arg(long)]
    ahb_xml: PathBuf,

    /// Path to message-level TOML mappings directory.
    #[arg(long)]
    message_mappings: PathBuf,

    /// Path to transaction-level TOML mappings directory.
    #[arg(long)]
    transaction_mappings: PathBuf,

    /// Message type (e.g., "UTILMD").
    #[arg(long)]
    message_type: String,

    /// Message variant (e.g., "Strom").
    #[arg(long)]
    variant: Option<String>,

    /// Format version (e.g., "FV2504").
    #[arg(long)]
    format_version: String,

    /// PID identifier (e.g., "55001").
    #[arg(long)]
    pid: String,

    /// Output path for the .mig.bo.json file. If omitted, writes alongside the source .edi.
    #[arg(long)]
    output: Option<PathBuf>,
},
```

Add the handler (implementation reads the .edi, forward-maps to BO4E, wraps in `CanonicalBo4e`, and writes JSON):
```rust
Commands::GenerateCanonicalBo4e {
    source,
    mig_xml,
    ahb_xml,
    message_mappings,
    transaction_mappings,
    message_type,
    variant,
    format_version,
    pid,
    output,
} => {
    use crate::fixture_renderer::types::*;

    // Parse MIG/AHB, filter for PID, tokenize, assemble, forward-map
    let mig = crate::parsing::mig_parser::parse_mig(
        &mig_xml, &message_type, variant.as_deref(), &format_version,
    )?;
    let ahb = crate::parsing::ahb_parser::parse_ahb(
        &ahb_xml, &message_type, variant.as_deref(), &format_version,
    )?;

    let pid_def = ahb.workflows.iter().find(|w| w.id == pid)
        .ok_or_else(|| crate::error::GeneratorError::Validation {
            message: format!("PID {} not found in AHB", pid),
        })?;

    let filtered_mig =
        mig_assembly::pid_filter::filter_mig_for_pid(&mig, &pid_def.segment_numbers);

    let edi_bytes = std::fs::read(&source)?;
    let segments = mig_assembly::tokenize::parse_to_segments(&edi_bytes)
        .map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("Tokenize failed: {}", e),
        })?;

    let chunks = mig_assembly::tokenize::split_messages(segments)
        .map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("Split failed: {}", e),
        })?;

    // Extract envelope data
    let nachrichtendaten = mig_bo4e::model::extract_nachrichtendaten(&chunks.envelope);

    let (msg_engine, tx_engine) = mig_bo4e::engine::MappingEngine::load_split(
        &message_mappings, &transaction_mappings,
    ).map_err(|e| crate::error::GeneratorError::Validation {
        message: format!("Load mappings failed: {}", e),
    })?;

    let assembler = mig_assembly::assembler::Assembler::new(&filtered_mig);

    // Process first message (for single-message interchanges)
    let msg = chunks.messages.first()
        .ok_or_else(|| crate::error::GeneratorError::Validation {
            message: "No messages found in interchange".into(),
        })?;

    let (unh_ref, nachrichten_typ) = mig_bo4e::model::extract_unh_fields(&msg.unh);

    let tree = assembler.assemble_generic(&msg.body)
        .map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("Assembly failed: {}", e),
        })?;

    let mapped = mig_bo4e::engine::MappingEngine::map_interchange(
        &msg_engine, &tx_engine, &tree, "SG4",
    );

    // Build CanonicalBo4e
    let canonical = CanonicalBo4e {
        meta: CanonicalMeta {
            pid: pid.clone(),
            message_type: message_type.clone(),
            source_format_version: format_version.clone(),
            source_fixture: source.file_name().unwrap().to_string_lossy().to_string(),
        },
        nachrichtendaten,
        nachricht: NachrichtBo4e {
            unh_referenz: unh_ref,
            nachrichten_typ,
            stammdaten: mapped.stammdaten.clone(),
            transaktionen: mapped.transaktionen.iter().map(|tx| {
                TransaktionBo4e {
                    stammdaten: tx.stammdaten.clone(),
                    transaktionsdaten: tx.transaktionsdaten.clone(),
                }
            }).collect(),
        },
    };

    let json = serde_json::to_string_pretty(&canonical)?;

    let output_path = output.unwrap_or_else(|| {
        source.with_extension("mig.bo.json")
    });

    std::fs::write(&output_path, &json)?;
    println!("Wrote canonical BO4E: {} ({} bytes)", output_path.display(), json.len());
}
```

**Step 2: Write integration test**

Add to `fixture_renderer_test.rs`:
```rust
#[test]
fn test_canonical_bo4e_structure_from_real_fixture() {
    // This test verifies the CanonicalBo4e types can represent real data.
    // The actual generation is tested via CLI in the integration test above.

    let canonical_json = r#"{
        "meta": {
            "pid": "55001",
            "message_type": "UTILMD",
            "source_format_version": "FV2504",
            "source_fixture": "55001_UTILMD_S2.1_test.edi"
        },
        "nachrichtendaten": {
            "absender": "9978842000002",
            "empfaenger": "9900269000000"
        },
        "nachricht": {
            "unh_referenz": "MSG001",
            "nachrichten_typ": "UTILMD:D:11A:UN:S2.1",
            "stammdaten": {
                "Marktteilnehmer": [
                    {"marktrolle": "MS"}
                ]
            },
            "transaktionen": [
                {
                    "stammdaten": {
                        "Marktlokation": [{"malo_id": "12345678900"}]
                    },
                    "transaktionsdaten": {
                        "Prozessdaten": [{"pruefidentifikator": "55001"}]
                    }
                }
            ]
        }
    }"#;

    let canonical: CanonicalBo4e = serde_json::from_str(canonical_json).unwrap();
    assert_eq!(canonical.meta.pid, "55001");
    assert_eq!(canonical.nachricht.transaktionen.len(), 1);
}
```

**Step 3: Verify it compiles and tests pass**

Run: `cargo check -p automapper-generator && cargo test -p automapper-generator test_canonical_bo4e`
Expected: PASS

**Step 4: Run full test suite and lint**

Run: `cargo test -p automapper-generator && cargo clippy -p automapper-generator -- -D warnings`
Expected: All PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/main.rs crates/automapper-generator/src/fixture_renderer/ crates/automapper-generator/tests/fixture_renderer_test.rs
git commit -m "feat(generator): add generate-canonical-bo4e and render-fixture CLI subcommands"
```
