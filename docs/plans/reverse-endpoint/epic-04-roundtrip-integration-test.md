---
feature: reverse-endpoint
epic: 4
title: "Roundtrip Integration Test"
depends_on: [reverse-endpoint/E01, reverse-endpoint/E02, reverse-endpoint/E03]
estimated_tasks: 3
crate: mig-bo4e, automapper-api
status: in_progress
---

# Epic 4: Roundtrip Integration Test

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Validate the full roundtrip: `EDIFACT → forward (map_interchange) → reverse (map_interchange_reverse) → EDIFACT` produces byte-identical output. This is the critical acceptance test for the reverse pipeline.

**Architecture:** The test loads a real PID 55001 fixture file, runs the forward pipeline to get BO4E JSON, then feeds it through the reverse pipeline, disassembles, renders, and compares the output EDIFACT to the original. A separate API-level test does the same through the HTTP endpoints.

**Existing code:**
- Forward fixture roundtrip: `crates/mig-bo4e/tests/` contains forward mapping tests for PID 55001
- EDIFACT fixtures: `example_market_communication_bo4e_transactions/UTILMD/FV2504/` or `tests/fixtures/`
- `mig-assembly::roundtrip` module at `crates/mig-assembly/src/roundtrip.rs` — tree-level roundtrip reference

---

## Task 1: Engine-Level Roundtrip Test

**Files:**
- Create: `crates/mig-bo4e/tests/reverse_roundtrip_test.rs`

**Step 1: Write the roundtrip test**

```rust
//! Full roundtrip test: EDIFACT → forward → reverse → compare AssembledTrees.
//!
//! Validates that map_interchange() followed by map_interchange_reverse()
//! produces a tree that can be disassembled back to the original EDIFACT.

use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_assembly::ConversionService;
use mig_bo4e::MappingEngine;
use std::collections::HashSet;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn find_fixture_55001() -> PathBuf {
    let root = project_root();

    // Try common fixture locations
    let candidates = [
        root.join("example_market_communication_bo4e_transactions/UTILMD/FV2504"),
        root.join("tests/fixtures/UTILMD/FV2504"),
    ];

    for dir in &candidates {
        if dir.exists() {
            // Find a file containing "55001"
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains("55001") && name.ends_with(".txt") {
                        return entry.path();
                    }
                }
            }
        }
    }

    panic!("No PID 55001 fixture file found");
}

#[test]
fn test_forward_reverse_roundtrip_55001() {
    let fixture_path = find_fixture_55001();
    let edifact_input = std::fs::read_to_string(&fixture_path).unwrap();

    let root = project_root();

    // Load MIG
    let mig_path =
        root.join("xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml");
    if !mig_path.exists() {
        eprintln!("Skipping test: MIG XML not available");
        return;
    }
    let service = ConversionService::from_mig(&mig_path).unwrap();

    // Load AHB for PID filtering
    let ahb_path =
        root.join("xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml");
    if !ahb_path.exists() {
        eprintln!("Skipping test: AHB XML not available");
        return;
    }
    let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(&ahb_path).unwrap();
    let workflow = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();
    let ahb_numbers: HashSet<String> = workflow.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);

    // Step 1: Tokenize and split
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(!chunks.messages.is_empty(), "Should have at least one message");

    let msg_chunk = &chunks.messages[0];
    let all_segs = msg_chunk.all_segments();

    // Step 2: Assemble
    let assembler = Assembler::new(&filtered_mig);
    let tree = assembler.assemble_generic(&all_segs).unwrap();

    // Step 3: Forward mapping
    let mappings_dir = root.join("mappings/FV2504/UTILMD_Strom");
    let msg_engine = MappingEngine::load(&mappings_dir.join("message")).unwrap();
    let tx_engine = MappingEngine::load(&mappings_dir.join("pid_55001")).unwrap();

    let mapped = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4");

    // Verify we got something meaningful
    assert!(
        !mapped.transaktionen.is_empty(),
        "Forward mapping should produce at least one transaction"
    );

    // Step 4: Reverse mapping
    let reverse_tree =
        MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

    // Step 5: Disassemble and render
    let disassembler = Disassembler::new(&filtered_mig);
    let dis_segments = disassembler.disassemble(&reverse_tree);

    let delimiters = edifact_types::EdifactDelimiters::default();
    let rendered = render_edifact(&dis_segments, &delimiters);

    // Step 6: Compare with original message body (between UNH and UNT)
    // The rendered output is the message body — compare segment by segment
    // for a meaningful diff rather than raw byte comparison.
    assert!(
        !rendered.is_empty(),
        "Reverse pipeline should produce non-empty EDIFACT"
    );

    // Compare segment tags at minimum
    let original_segments: Vec<&str> = all_segs.iter().map(|s| s.id.as_str()).collect();
    let rendered_segments = parse_to_segments(rendered.as_bytes()).unwrap();
    let rendered_tags: Vec<&str> = rendered_segments.iter().map(|s| s.id.as_str()).collect();

    // The reverse should produce the same set of segment tags in the same order
    assert_eq!(
        original_segments, rendered_tags,
        "Segment tags should match after forward→reverse roundtrip.\nOriginal: {:?}\nReversed: {:?}",
        original_segments, rendered_tags
    );

    // For full byte comparison (may need refinements):
    // Compare the actual rendered EDIFACT with the original message body
    let original_rendered = render_edifact(
        &disassembler.disassemble(&tree),
        &delimiters,
    );
    assert_eq!(
        original_rendered, rendered,
        "Full EDIFACT roundtrip should be byte-identical"
    );
}
```

Note: This test depends on `automapper-generator` for AHB parsing. Check that `automapper-generator` is in `mig-bo4e`'s dev-dependencies. If not, this test should be moved to `automapper-api` tests or a separate integration test crate. Adjust the test location based on what compiles.

**Step 2: Run the test**

Run: `cargo test -p mig-bo4e --test reverse_roundtrip_test -- --nocapture`
Expected: PASS (or adjust based on any segment ordering differences)

**Step 3: Commit**

```bash
git add crates/mig-bo4e/tests/reverse_roundtrip_test.rs
git commit -m "test(mig-bo4e): add EDIFACT forward→reverse roundtrip test for PID 55001"
```

---

## Task 2: Fix Roundtrip Differences

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs` (if needed)

This task handles any discrepancies found during Task 1. Common issues:

1. **Segment ordering**: `map_all_reverse()` may produce groups in definition order rather than MIG order. The disassembler should handle reordering, but verify.

2. **Empty vs missing components**: The forward mapper omits empty strings; the reverse mapper may not pad them correctly. Check `map_reverse()` padding logic (lines 533-555 in engine.rs).

3. **Array vs single object**: When forward produces a JSON array for an entity (multiple reps), the reverse must produce multiple group repetitions.

4. **Discriminator handling**: Multiple definitions with the same entity/source_group but different discriminators (e.g., multiple RFFs in SG6) need correct reverse routing.

5. **Nested group paths**: Definitions with `source_group = "SG4.SG8"` need the reverse to place results as child groups of the right parent. The current `map_all_reverse()` uses the leaf group name.

**Step 1: Analyze test failures**

Run the roundtrip test with `--nocapture` and compare original vs reversed segment-by-segment.

**Step 2: Fix identified issues**

Each fix should be minimal and targeted:
- If group ordering is wrong, ensure `map_all_reverse()` preserves definition order or sorts by source_group depth.
- If nested groups aren't placed correctly, enhance the group placement logic in `map_all_reverse()`.
- If discriminated definitions produce duplicate reps, add discriminator-aware merging.

**Step 3: Re-run tests**

Run: `cargo test -p mig-bo4e`
Expected: ALL PASS

**Step 4: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs
git commit -m "fix(mig-bo4e): fix reverse mapping for roundtrip fidelity"
```

---

## Task 3: API-Level Roundtrip Test

**Files:**
- Create or modify: `crates/automapper-api/tests/reverse_roundtrip_test.rs`

**Step 1: Write the API roundtrip test**

This test calls the forward endpoint, takes the BO4E output, feeds it to the reverse endpoint, and compares:

```rust
//! API-level roundtrip test: POST /convert (bo4e) → POST /reverse (edifact).

// Follow the existing API test pattern from crates/automapper-api/tests/

#[tokio::test]
async fn test_api_forward_reverse_roundtrip() {
    // 1. Load EDIFACT fixture
    // 2. POST /api/v2/convert with mode: "bo4e" → get Interchange JSON
    // 3. POST /api/v2/reverse with level: "interchange", mode: "edifact" → get EDIFACT string
    // 4. Compare segment structure of output with original
    //
    // Note: Full byte-identical comparison may differ in envelope segments
    // (UNB date/time, interchange ref) since those are reconstructed from JSON.
    // Compare message body segments (between UNH and UNT) for fidelity.
}
```

**Step 2: Run test**

Run: `cargo test -p automapper-api --test reverse_roundtrip_test -- --nocapture`
Expected: PASS

**Step 3: Run full workspace tests**

Run: `cargo test --workspace`
Expected: ALL PASS

Run: `cargo clippy --workspace -- -D warnings`
Expected: OK

**Step 4: Commit**

```bash
git add crates/automapper-api/tests/
git commit -m "test(api): add API-level forward→reverse roundtrip integration test"
```
