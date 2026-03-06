//! UTILMD Gas PID 44001 (Anmeldung NN) full pipeline roundtrip test.
//!
//! EDIFACT -> tokenize -> split -> assemble -> map_interchange
//! -> map_interchange_reverse -> disassemble -> render -> byte-identical comparison.
//!
//! Uses SG2/SG3 normalization for generated fixtures where CTA/COM ordering
//! may differ from the reverse mapper output.

mod common;

use common::{bo4e_validation, test_utils, utilmd_gas};
use edifact_types::EdifactDelimiters;
use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;

#[test]
fn test_pid_44001_generated_roundtrip() {
    run_generated_roundtrip("44001");
}

#[test]
fn test_pid_44002_generated_roundtrip() {
    run_generated_roundtrip("44002");
}

// ── SG2/SG3 normalization ──

/// Normalize SG2/SG3 block ordering in rendered EDIFACT.
///
/// Generated fixtures place CTA/COM (SG3) after ALL NAD (SG2) segments,
/// while the reverse mapper places them under the first SG2 rep.
/// This normalizes by moving CTA/COM segments to just after the first NAD.
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
            // Once we hit a non-NAD, non-CTA/COM segment after NADs, insert collected SG3
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
    // Handle edge case: SG3 at very end
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
    let Some(fixture_path) = utilmd_gas::discover_generated_fixture(pid) else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(
        fixture_path.exists(),
        "Generated fixture not found: {fixture_path:?}"
    );

    let Some(filtered_mig) = utilmd_gas::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let tx_dir = utilmd_gas::pid_dir(pid);
    if !utilmd_gas::message_dir().exists() || !tx_dir.exists() {
        eprintln!("Skipping roundtrip for PID {pid}: mapping directories not found");
        return;
    }
    let (msg_engine, tx_engine) = utilmd_gas::load_split_engines(pid);

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
        utilmd_gas::TX_GROUP,
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
        utilmd_gas::TX_GROUP,
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
        utilmd_gas::TX_GROUP,
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
        "PID {pid} ({fixture_name}): segment tags should match after forward->reverse roundtrip"
    );

    // Step 8: Compare full rendered EDIFACT (normalized)
    if original_normalized != reverse_normalized {
        let max_len = orig_norm_segs.len().max(rev_norm_segs.len());
        let mut diff_count = 0;
        for i in 0..max_len {
            let o = orig_norm_segs.get(i).unwrap_or(&"<missing>");
            let r = rev_norm_segs.get(i).unwrap_or(&"<missing>");
            if o != r {
                eprintln!("PID {pid} ({fixture_name}): segment {i} differs:");
                eprintln!("  original: {o}");
                eprintln!("  reversed: {r}");
                diff_count += 1;
            }
        }
        eprintln!(
            "PID {pid} ({fixture_name}): {diff_count} segment(s) differ out of {}",
            orig_norm_segs.len()
        );
    }

    assert_eq!(
        original_normalized, reverse_normalized,
        "PID {pid} ({fixture_name}): full EDIFACT roundtrip should be byte-identical (SG2/SG3 normalized)"
    );

    eprintln!(
        "PID {pid}: {fixture_name} -- roundtrip OK, {} segments byte-identical",
        original_tags.len()
    );
}
