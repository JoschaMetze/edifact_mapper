mod common;
use common::utilmd_gas;
use mig_assembly::assembler::Assembler;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use std::fs;

#[test]
fn test_pid_44013_debug() {
    let pid = "44013";
    let fixture_path =
        "/home/claude/github/edifact_mapper/fixtures/generated/fv2504/utilmd/44013.edi";

    let Some(filtered_mig) = utilmd_gas::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available");
        return;
    };

    let (msg_engine, tx_engine) = utilmd_gas::load_split_engines(pid);
    let edifact_input = fs::read_to_string(fixture_path).unwrap();

    // Parse
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    let msg_chunk = &chunks.messages[0];
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    eprintln!("Original segments: {}", msg_segs.len());
    for (i, seg) in msg_segs.iter().enumerate() {
        eprintln!("  {}: {}", i, seg.id);
    }

    // Assemble
    let assembler = Assembler::new(&filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs).unwrap();
    eprintln!("Original tree: {} segments", original_tree.segments.len());
    for seg in &original_tree.segments {
        eprintln!("  tree: {}", seg.tag);
    }

    // Forward
    let mapped =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);
    eprintln!("Mapped transactions: {}", mapped.transaktionen.len());
    if let Some(tx) = mapped.transaktionen.first() {
        let tx_val = serde_json::to_value(tx).unwrap();
        if let Some(obj) = tx_val.as_object() {
            eprintln!("First tx keys: {:?}", obj.keys().collect::<Vec<_>>());
        }
    }

    // Reverse
    let reverse_tree = MappingEngine::map_interchange_reverse(
        &msg_engine,
        &tx_engine,
        &mapped,
        "SG4",
        Some(&filtered_mig),
    );

    eprintln!(
        "Reverse tree segments: {} (no UNH/UNT yet)",
        reverse_tree.segments.len()
    );
    for seg in reverse_tree.segments.iter() {
        eprintln!("  {}", seg.tag);
    }
}
