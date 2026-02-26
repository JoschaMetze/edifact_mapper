//! Full roundtrip test: EDIFACT → forward → reverse → disassemble → render → compare.
//!
//! Validates that map_interchange() followed by map_interchange_reverse()
//! produces a tree that can be disassembled back to the original EDIFACT.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::{AssembledSegment, Assembler};
use mig_assembly::disassembler::Disassembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/pid_55001";
const MESSAGE_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/message";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(Path::new(SCHEMA_DIR))
}

fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        return None;
    }
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    Some(filter_mig_for_pid(&mig, &numbers))
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(FIXTURE_DIR).join(name)
}

/// Convert an OwnedSegment to an AssembledSegment (for injecting UNH/UNT into trees).
fn owned_to_assembled(seg: &mig_assembly::tokenize::OwnedSegment) -> AssembledSegment {
    AssembledSegment {
        tag: seg.id.clone(),
        elements: seg
            .elements
            .iter()
            .map(|el| el.iter().map(|c| c.to_string()).collect())
            .collect(),
    }
}

/// Full pipeline roundtrip: EDIFACT → tokenize → split → assemble → map_interchange
/// → map_interchange_reverse → disassemble → render → compare with original.
///
/// The mapping engine handles content segments (BGM, DTM, SG2, SG4...) but NOT
/// envelope segments (UNB, UNH, UNT, UNZ). The test reconstructs UNH/UNT on the
/// reversed tree to match the original assembly, just as the API layer does.
#[test]
fn test_forward_reverse_roundtrip_55001() {
    let Some(filtered_mig) = load_pid_filtered_mig("55001") else {
        eprintln!("Skipping: MIG/AHB XML not available");
        return;
    };

    let path = fixture_path("55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {}", path.display());
        return;
    }
    let edifact_input = std::fs::read_to_string(&path).unwrap();

    // Load split engines
    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new(MAPPINGS_DIR);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping: mapping directories not found");
        return;
    }
    let msg_engine = MappingEngine::load(msg_dir)
        .unwrap()
        .with_path_resolver(path_resolver());
    let tx_engine = MappingEngine::load(tx_dir)
        .unwrap()
        .with_path_resolver(path_resolver());

    // Step 1: Tokenize and split
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(
        !chunks.messages.is_empty(),
        "Should have at least one message"
    );

    let msg_chunk = &chunks.messages[0];

    // Assemble with UNH + body + UNT only (no UNB envelope).
    // The mapping engine handles message content; envelope reconstruction
    // is an API-level concern (rebuild_unb/rebuild_unz).
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    // Step 2: Assemble with PID-filtered MIG
    let assembler = Assembler::new(&filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    // Step 3: Forward mapping → MappedMessage
    let mapped =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);

    // Verify we got meaningful forward output
    assert!(
        !mapped.transaktionen.is_empty(),
        "Forward mapping should produce at least one transaction"
    );

    // Step 4: Reverse mapping → AssembledTree (content only, no UNH/UNT)
    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

    // Add UNH to the front of pre-group segments, UNT to post-group.
    // This mirrors what the API layer does with rebuild_unh/rebuild_unt.
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    let unt_assembled = owned_to_assembled(&msg_chunk.unt);

    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;
    reverse_tree.segments.push(unt_assembled);

    // Step 5: Disassemble both trees and render
    let disassembler = Disassembler::new(&filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    // Step 6: Compare segment tags (structural check)
    let original_tags: Vec<&str> = original_dis.iter().map(|s| s.tag.as_str()).collect();
    let reverse_tags: Vec<&str> = reverse_dis.iter().map(|s| s.tag.as_str()).collect();

    assert_eq!(
        original_tags, reverse_tags,
        "Segment tags should match after forward→reverse roundtrip.\nOriginal: {:?}\nReversed: {:?}",
        original_tags, reverse_tags
    );

    // Step 7: Compare full rendered EDIFACT (byte-identical check)
    if original_rendered != reverse_rendered {
        let orig_segs: Vec<&str> = original_rendered.split('\'').collect();
        let rev_segs: Vec<&str> = reverse_rendered.split('\'').collect();
        let max_len = orig_segs.len().max(rev_segs.len());
        for i in 0..max_len {
            let o = orig_segs.get(i).unwrap_or(&"<missing>");
            let r = rev_segs.get(i).unwrap_or(&"<missing>");
            if o != r {
                eprintln!("Segment {i} differs:");
                eprintln!("  original: {o}");
                eprintln!("  reversed: {r}");
            }
        }
    }

    assert_eq!(
        original_rendered, reverse_rendered,
        "Full EDIFACT roundtrip should be byte-identical"
    );
}

/// PID 55002 roundtrip test — 36 segments byte-identical.
#[test]
fn test_forward_reverse_roundtrip_55002() {
    let Some(filtered_mig) = load_pid_filtered_mig("55002") else {
        eprintln!("Skipping: MIG/AHB XML not available");
        return;
    };

    let path = fixture_path("55002_UTILMD_S2.1_ALEXANDE104683.edi");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {}", path.display());
        return;
    }
    let edifact_input = std::fs::read_to_string(&path).unwrap();

    let pid_55002_dir = Path::new("../../mappings/FV2504/UTILMD_Strom/pid_55002");
    let msg_dir = Path::new(MESSAGE_DIR);
    if !msg_dir.exists() || !pid_55002_dir.exists() {
        eprintln!("Skipping: mapping directories not found");
        return;
    }
    let msg_engine = MappingEngine::load(msg_dir)
        .unwrap()
        .with_path_resolver(path_resolver());
    let tx_engine = MappingEngine::load(pid_55002_dir)
        .unwrap()
        .with_path_resolver(path_resolver());

    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(
        !chunks.messages.is_empty(),
        "Should have at least one message"
    );

    let msg_chunk = &chunks.messages[0];
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    let assembler = Assembler::new(&filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    let mapped =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);
    assert!(
        !mapped.transaktionen.is_empty(),
        "Forward mapping should produce at least one transaction"
    );

    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    let original_has_unt = original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if original_has_unt {
        let unt_assembled = owned_to_assembled(&msg_chunk.unt);
        reverse_tree.segments.push(unt_assembled);
    }

    let disassembler = Disassembler::new(&filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    let original_tags: Vec<&str> = original_dis.iter().map(|s| s.tag.as_str()).collect();
    let reverse_tags: Vec<&str> = reverse_dis.iter().map(|s| s.tag.as_str()).collect();

    if original_tags != reverse_tags {
        eprintln!("PID 55002: segment tag mismatch!");
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
        "PID 55002: segment tags should match"
    );

    if original_rendered != reverse_rendered {
        let orig_segs: Vec<&str> = original_rendered.split('\'').collect();
        let rev_segs: Vec<&str> = reverse_rendered.split('\'').collect();
        let max_len = orig_segs.len().max(rev_segs.len());
        let mut diff_count = 0;
        for i in 0..max_len {
            let o = orig_segs.get(i).unwrap_or(&"<missing>");
            let r = rev_segs.get(i).unwrap_or(&"<missing>");
            if o != r {
                eprintln!("PID 55002: segment {i} differs:");
                eprintln!("  original: {o}");
                eprintln!("  reversed: {r}");
                diff_count += 1;
            }
        }
        eprintln!(
            "PID 55002: {diff_count} segment(s) differ out of {}",
            orig_segs.len()
        );
    }

    assert_eq!(
        original_rendered, reverse_rendered,
        "PID 55002: full EDIFACT roundtrip should be byte-identical"
    );

    eprintln!(
        "PID 55002: roundtrip OK -- {} segments byte-identical",
        original_tags.len()
    );
}

/// Roundtrip structural check with a second fixture file.
///
/// The DEV fixture has additional unmapped name/address fields (long text
/// components in NAD segments) that are lost during forward mapping. This
/// test verifies structural equivalence (segment tags match) rather than
/// byte-identical content, as roundtrip only preserves mapped fields.
#[test]
fn test_forward_reverse_roundtrip_55001_dev_fixture() {
    let Some(filtered_mig) = load_pid_filtered_mig("55001") else {
        eprintln!("Skipping: MIG/AHB XML not available");
        return;
    };

    let path = fixture_path("55001_UTILMD_S2.1_DEV-86943-2.edi");
    if !path.exists() {
        eprintln!("Skipping: fixture not found at {}", path.display());
        return;
    }
    let edifact_input = std::fs::read_to_string(&path).unwrap();

    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new(MAPPINGS_DIR);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping: mapping directories not found");
        return;
    }
    let msg_engine = MappingEngine::load(msg_dir)
        .unwrap()
        .with_path_resolver(path_resolver());
    let tx_engine = MappingEngine::load(tx_dir)
        .unwrap()
        .with_path_resolver(path_resolver());

    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(!chunks.messages.is_empty());

    let msg_chunk = &chunks.messages[0];
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    let assembler = Assembler::new(&filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    // Forward → Reverse
    let mapped =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);
    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

    // Add UNH/UNT envelope
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    let unt_assembled = owned_to_assembled(&msg_chunk.unt);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;
    reverse_tree.segments.push(unt_assembled);

    // Disassemble and compare structural equivalence
    let disassembler = Disassembler::new(&filtered_mig);

    let original_dis = disassembler.disassemble(&original_tree);
    let reverse_dis = disassembler.disassemble(&reverse_tree);

    let original_tags: Vec<&str> = original_dis.iter().map(|s| s.tag.as_str()).collect();
    let reverse_tags: Vec<&str> = reverse_dis.iter().map(|s| s.tag.as_str()).collect();

    assert_eq!(
        original_tags, reverse_tags,
        "Segment tags should match for DEV fixture.\nOriginal: {:?}\nReversed: {:?}",
        original_tags, reverse_tags
    );
}
