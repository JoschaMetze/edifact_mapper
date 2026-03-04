//! EDIFACT roundtrip tests for APERAK and CONTRL message types.
//!
//! These are non-UTILMD messages without PID-based filtering.
//! Uses the full MIG directly with `map_all_forward`/`map_all_reverse`
//! (not `map_interchange` which splits message/transaction levels).

use mig_assembly::assembler::{owned_to_assembled, Assembler, AssemblerConfig};
use mig_assembly::disassembler::Disassembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use std::path::Path;

// ── CONTRL constants ──

const CONTRL_MIG_XML: &str =
    "../../xml-migs-and-ahbs/FV2504/CONTRL_MIG_2_0b_außerordentliche_20240726.xml";
const CONTRL_FIXTURE_DIR: &str =
    "../../example_market_communication_bo4e_transactions/CONTRL/FV2504";
const CONTRL_MAPPINGS: &str = "../../mappings/FV2504/CONTRL";
const CONTRL_SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/contrl/pids";

// ── APERAK constants ──

const APERAK_MIG_XML: &str = "../../xml-migs-and-ahbs/FV2504/APERAK_MIG_2_1i_20240619.xml";
const APERAK_FIXTURE_DIR: &str =
    "../../example_market_communication_bo4e_transactions/APERAK/FV2504";
const APERAK_MAPPINGS: &str = "../../mappings/FV2504/APERAK";
const APERAK_SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/aperak/pids";

/// Run a full roundtrip for a non-UTILMD message type.
///
/// No PID filtering — uses the full MIG directly.
/// Uses `map_all_forward`/`map_all_reverse` (flat TOML directory, no message/transaction split).
fn run_roundtrip(
    message_type: &str,
    mig_xml_path: &str,
    mapping_dir: &str,
    schema_dir: &str,
    fixture_path: &str,
    skip_unknown: bool,
) {
    let mig_path = Path::new(mig_xml_path);
    if !mig_path.exists() {
        eprintln!("Skipping {message_type}: MIG XML not found at {mig_xml_path}");
        return;
    }

    let fixture = Path::new(fixture_path);
    if !fixture.exists() {
        eprintln!("Skipping {message_type}: fixture not found at {fixture_path}");
        return;
    }

    let mapping_path = Path::new(mapping_dir);
    if !mapping_path.exists() {
        eprintln!("Skipping {message_type}: mapping dir not found at {mapping_dir}");
        return;
    }

    let fixture_name = fixture.file_name().unwrap().to_str().unwrap();

    // Step 1: Parse MIG (no variant for CONTRL/APERAK)
    let mig = parse_mig(mig_path, message_type, None, "FV2504")
        .unwrap_or_else(|e| panic!("{message_type}: failed to parse MIG: {e}"));

    // Step 2: Load engine with PathResolver
    let schema_path = Path::new(schema_dir);
    let resolver = if schema_path.exists() {
        PathResolver::from_schema_dir(schema_path)
    } else {
        eprintln!("Warning: schema dir not found at {schema_dir}, using empty resolver");
        PathResolver::from_schema(&serde_json::json!({}))
    };

    let engine = MappingEngine::load(mapping_path)
        .unwrap_or_else(|e| panic!("{message_type}: failed to load mappings: {e}"))
        .with_path_resolver(resolver);

    // Step 3: Tokenize and split
    let edifact_input = std::fs::read_to_string(fixture).unwrap();
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(
        !chunks.messages.is_empty(),
        "{message_type} ({fixture_name}): should have at least one message"
    );

    let msg_chunk = &chunks.messages[0];

    // Build message segments: UNH + body + UNT
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    // Step 4: Assemble with full MIG
    let assembler = Assembler::with_config(
        &mig,
        AssemblerConfig {
            skip_unknown_segments: skip_unknown,
        },
    );
    let mut original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    eprintln!(
        "{message_type} ({fixture_name}): assembled {} root segments, {} groups",
        original_tree.segments.len(),
        original_tree.groups.len()
    );

    // Step 5: Forward map
    let json = engine.map_all_forward(&original_tree);
    eprintln!(
        "{message_type} ({fixture_name}): forward mapped {} entities",
        json.as_object().map(|o| o.len()).unwrap_or(0)
    );

    // Step 6: Reverse map
    let mut reverse_tree = engine.map_all_reverse(&json);

    // Re-insert UNH at position 0 (mapping engine doesn't handle service segments)
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    // UNT may end up in inter_group_segments (trailing after last group).
    // Strip from inter_group in both trees and always add from msg_chunk.unt.
    for segs in original_tree.inter_group_segments.values_mut() {
        segs.retain(|s| s.tag != "UNT");
    }
    let original_has_unt_in_root =
        original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if !original_has_unt_in_root {
        let unt_orig = owned_to_assembled(&msg_chunk.unt);
        original_tree.segments.push(unt_orig);
    }

    for segs in reverse_tree.inter_group_segments.values_mut() {
        segs.retain(|s| s.tag != "UNT");
    }
    let unt_assembled = owned_to_assembled(&msg_chunk.unt);
    reverse_tree.segments.push(unt_assembled);

    // Step 7: Disassemble and render
    let disassembler = Disassembler::new(&mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    // Step 8: Compare segment tags (structural check)
    let original_tags: Vec<&str> = original_dis.iter().map(|s| s.tag.as_str()).collect();
    let reverse_tags: Vec<&str> = reverse_dis.iter().map(|s| s.tag.as_str()).collect();

    if original_tags != reverse_tags {
        eprintln!("{message_type} ({fixture_name}): segment tag mismatch!");
        eprintln!(
            "  original ({} segs): {:?}",
            original_tags.len(),
            original_tags
        );
        eprintln!(
            "  reversed ({} segs): {:?}",
            reverse_tags.len(),
            reverse_tags
        );
        for (i, tag) in original_tags.iter().enumerate() {
            if reverse_tags.get(i) != Some(tag) {
                eprintln!(
                    "  first difference at position {i}: original={tag}, reversed={}",
                    reverse_tags.get(i).unwrap_or(&"<missing>")
                );
                break;
            }
        }
    }

    assert_eq!(
        original_tags, reverse_tags,
        "{message_type} ({fixture_name}): segment tags should match after roundtrip"
    );

    // Step 9: Compare full EDIFACT (byte-identical check)
    if original_rendered != reverse_rendered {
        let orig_segs: Vec<&str> = original_rendered.split('\'').collect();
        let rev_segs: Vec<&str> = reverse_rendered.split('\'').collect();
        let max_len = orig_segs.len().max(rev_segs.len());
        let mut diff_count = 0;
        for i in 0..max_len {
            let o = orig_segs.get(i).unwrap_or(&"<missing>");
            let r = rev_segs.get(i).unwrap_or(&"<missing>");
            if o != r {
                eprintln!("{message_type} ({fixture_name}): segment {i} differs:");
                eprintln!("  original: {o}");
                eprintln!("  reversed: {r}");
                diff_count += 1;
                if diff_count >= 5 {
                    eprintln!("  ... (truncated)");
                    break;
                }
            }
        }
    }

    assert_eq!(
        original_rendered, reverse_rendered,
        "{message_type} ({fixture_name}): EDIFACT output should be byte-identical after roundtrip"
    );

    eprintln!(
        "{message_type} ({fixture_name}): roundtrip OK ({} segments)",
        original_tags.len()
    );
}

// ── CONTRL tests ──

#[test]
fn test_contrl_generated_roundtrip() {
    let fixture = format!("{CONTRL_FIXTURE_DIR}/generated/_CONTRL_generated.edi");
    run_roundtrip(
        "CONTRL",
        CONTRL_MIG_XML,
        CONTRL_MAPPINGS,
        CONTRL_SCHEMA_DIR,
        &fixture,
        false,
    );
}

// ── APERAK tests ──

#[test]
fn test_aperak_generated_roundtrip() {
    let fixture = format!("{APERAK_FIXTURE_DIR}/generated/_APERAK_generated.edi");
    run_roundtrip(
        "APERAK",
        APERAK_MIG_XML,
        APERAK_MAPPINGS,
        APERAK_SCHEMA_DIR,
        &fixture,
        false,
    );
}

#[test]
fn test_aperak_joscha_roundtrip() {
    let fixture = format!("{APERAK_FIXTURE_DIR}/92001_APERAK_2.1f_JOSCHA60014103.edi");
    run_roundtrip(
        "APERAK",
        APERAK_MIG_XML,
        APERAK_MAPPINGS,
        APERAK_SCHEMA_DIR,
        &fixture,
        false,
    );
}

#[test]
fn test_aperak_dev99155_roundtrip() {
    let fixture = format!("{APERAK_FIXTURE_DIR}/92001_APERAK_2.1i_DEV-99155.edi");
    run_roundtrip(
        "APERAK",
        APERAK_MIG_XML,
        APERAK_MAPPINGS,
        APERAK_SCHEMA_DIR,
        &fixture,
        false,
    );
}

#[test]
fn test_aperak_dev99155_2_roundtrip() {
    let fixture = format!("{APERAK_FIXTURE_DIR}/92001_APERAK_2.1i_DEV-99155-2.edi");
    run_roundtrip(
        "APERAK",
        APERAK_MIG_XML,
        APERAK_MAPPINGS,
        APERAK_SCHEMA_DIR,
        &fixture,
        false,
    );
}
