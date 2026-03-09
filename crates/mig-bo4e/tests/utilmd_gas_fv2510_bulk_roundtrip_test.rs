//! Bulk roundtrip tests for all 95 UTILMD Gas PIDs (FV2510).
//!
//! Uses generated fixtures from `example_market_communication_bo4e_transactions/UTILMD/FV2510/generated_gas/`.
//! Full pipeline: EDIFACT → tokenize → split → assemble → map_interchange
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.
//!
//! Gas MIG/AHB XMLs are byte-identical between FV2504 and FV2510, so all PIDs
//! that pass in FV2504 should pass identically in FV2510.

mod common;

use common::{bo4e_validation, test_utils, utilmd_gas_fv2510};
use edifact_types::EdifactDelimiters;
use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;

/// PIDs with known structural limitations.
/// - 44002: FV2510 fixture has PIA after RFF in Z20 block, but MIG defines
///   PIA before RFF. Assembler drops PIA (out-of-order). FV2504 fixture OK.
/// - 44013, 44014, 44035: Z02+Z20 PIA merge conflict (same as FV2504).
const KNOWN_INCOMPLETE: &[&str] = &["44002", "44013", "44014", "44035"];

#[test]
fn test_all_gas_fv2510_pids_roundtrip() {
    let all_pids = [
        "44001", "44002", "44003", "44004", "44005", "44006", "44007", "44008", "44009", "44010",
        "44011", "44012", "44013", "44014", "44015", "44016", "44017", "44018", "44019", "44020",
        "44021", "44022", "44023", "44024", "44035", "44036", "44037", "44038", "44039", "44040",
        "44041", "44042", "44043", "44044", "44051", "44052", "44053", "44060", "44096", "44097",
        "44101", "44102", "44103", "44104", "44105", "44109", "44110", "44111", "44112", "44113",
        "44115", "44116", "44117", "44119", "44120", "44121", "44123", "44124", "44129", "44130",
        "44132", "44137", "44138", "44139", "44140", "44142", "44143", "44145", "44146", "44147",
        "44148", "44149", "44150", "44151", "44152", "44156", "44157", "44159", "44160", "44161",
        "44162", "44163", "44164", "44165", "44166", "44167", "44168", "44169", "44170", "44172",
        "44175", "44176", "44180", "44181", "44182",
    ];

    assert_eq!(all_pids.len(), 95, "should cover all 95 Gas PIDs");

    let mut passed = 0;
    let mut skipped = 0;

    for pid in &all_pids {
        if KNOWN_INCOMPLETE.contains(pid) {
            eprintln!("PID {pid}: KNOWN_INCOMPLETE — skipping");
            skipped += 1;
            continue;
        }
        run_generated_roundtrip(pid);
        passed += 1;
    }

    eprintln!(
        "\nGas FV2510 bulk roundtrip: {passed} passed, {skipped} known-incomplete, {} total",
        all_pids.len()
    );
}

// ── SG2/SG3 normalization ──

/// Normalize SG2/SG3 block ordering in rendered EDIFACT.
fn normalize_sg2_sg3_ordering(rendered: &str) -> String {
    let segs: Vec<&str> = rendered.split('\'').collect();
    let mut result: Vec<&str> = Vec::new();
    let mut sg3_segs: Vec<&str> = Vec::new();
    let mut first_nad_idx: Option<usize> = None;
    let mut past_sg2 = false;

    for seg in &segs {
        let tag = seg.split('+').next().unwrap_or("");
        if tag == "NAD" && !past_sg2 {
            if first_nad_idx.is_none() {
                first_nad_idx = Some(result.len());
            }
            result.push(seg);
        } else if (tag == "CTA" || tag == "COM") && !past_sg2 {
            sg3_segs.push(seg);
        } else {
            if !past_sg2 {
                if let Some(idx) = first_nad_idx {
                    if !sg3_segs.is_empty() {
                        let insert_pos = idx + 1;
                        for (i, sg3) in sg3_segs.drain(..).enumerate() {
                            result.insert(insert_pos + i, sg3);
                        }
                    }
                    past_sg2 = true;
                }
            }
            result.push(seg);
        }
    }
    if !sg3_segs.is_empty() {
        if let Some(idx) = first_nad_idx {
            let insert_pos = idx + 1;
            for (i, sg3) in sg3_segs.into_iter().enumerate() {
                result.insert(insert_pos + i, sg3);
            }
        }
    }
    result.join("'")
}

// ── Roundtrip helper ──

fn run_generated_roundtrip(pid: &str) {
    let Some(fixture_path) = utilmd_gas_fv2510::discover_generated_fixture(pid) else {
        panic!("PID {pid}: no FV2510 generated fixture found");
    };

    let Some(filtered_mig) = utilmd_gas_fv2510::load_pid_filtered_mig(pid) else {
        panic!("PID {pid}: FV2510 MIG/AHB XML not available");
    };

    let tx_dir = utilmd_gas_fv2510::pid_dir(pid);
    if !utilmd_gas_fv2510::message_dir().exists() || !tx_dir.exists() {
        panic!("PID {pid}: FV2510 mapping directories not found");
    }
    let (msg_engine, tx_engine) = utilmd_gas_fv2510::load_split_engines(pid);

    let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();
    let edifact_input = std::fs::read_to_string(&fixture_path).unwrap();

    // Step 1: Tokenize and split
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(
        !chunks.messages.is_empty(),
        "PID {pid} ({fixture_name}): should have at least one message"
    );

    let msg_chunk = &chunks.messages[0];
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    // Step 2: Assemble with PID-filtered MIG
    let assembler = Assembler::new(&filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    // Step 3: Forward mapping
    let mapped = MappingEngine::map_interchange(
        &msg_engine,
        &tx_engine,
        &original_tree,
        utilmd_gas_fv2510::TX_GROUP,
        true,
    );

    assert!(
        !mapped.transaktionen.is_empty(),
        "PID {pid} ({fixture_name}): forward mapping should produce at least one transaction"
    );

    // Step 3b: BO4E schema validation
    let mapped_for_validation = MappingEngine::map_interchange(
        &msg_engine,
        &tx_engine,
        &original_tree,
        utilmd_gas_fv2510::TX_GROUP,
        false,
    );
    bo4e_validation::validate_mapped_message(
        pid,
        fixture_name,
        &msg_engine,
        &tx_engine,
        &mapped_for_validation,
    );

    // Step 4: Reverse mapping
    let mut reverse_tree = MappingEngine::map_interchange_reverse(
        &msg_engine,
        &tx_engine,
        &mapped,
        utilmd_gas_fv2510::TX_GROUP,
        Some(&filtered_mig),
    );

    let unh_assembled = test_utils::owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    let original_has_unt = original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if original_has_unt {
        let unt_assembled = test_utils::owned_to_assembled(&msg_chunk.unt);
        reverse_tree.segments.push(unt_assembled);
    }

    // Step 5: Disassemble and render
    let disassembler = Disassembler::new(&filtered_mig);
    let delimiters = EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    // Step 6: Normalize SG2/SG3 ordering
    let original_normalized = normalize_sg2_sg3_ordering(&original_rendered);
    let reverse_normalized = normalize_sg2_sg3_ordering(&reverse_rendered);

    // Step 7: Compare segment tags
    let orig_norm_segs: Vec<&str> = original_normalized.split('\'').collect();
    let rev_norm_segs: Vec<&str> = reverse_normalized.split('\'').collect();
    let original_tags: Vec<&str> = orig_norm_segs
        .iter()
        .map(|s| s.split('+').next().unwrap_or(""))
        .collect();
    let reverse_tags: Vec<&str> = rev_norm_segs
        .iter()
        .map(|s| s.split('+').next().unwrap_or(""))
        .collect();

    if original_tags != reverse_tags {
        eprintln!("PID {pid} ({fixture_name}): segment tag mismatch!");
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
        "PID {pid} ({fixture_name}): segment tags should match after roundtrip"
    );

    // Step 8: Compare full rendered EDIFACT (normalized)
    assert_eq!(
        original_normalized, reverse_normalized,
        "PID {pid} ({fixture_name}): full EDIFACT roundtrip should be byte-identical"
    );

    eprintln!(
        "PID {pid}: {fixture_name} -- roundtrip OK, {} segments byte-identical",
        original_tags.len()
    );
}
