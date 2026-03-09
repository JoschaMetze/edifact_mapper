//! Bulk roundtrip tests for all 119 newly-mapped PIDs.
//!
//! Uses generated fixtures from `fixtures/generated/fv2504/utilmd/`.
//! Full pipeline: EDIFACT → tokenize → split → assemble → map_interchange
//! → map_interchange_reverse → disassemble → render → compare with original.

use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;

mod common;
use common::bo4e_validation;
use common::test_utils;

/// Fixtures with known mapping gaps that prevent byte-identical roundtrip.
const KNOWN_INCOMPLETE: &[&str] = &[];

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
            if !past_sg2 && first_nad_idx.is_some() && !sg3_segs.is_empty() {
                // Insert SG3 segments after first NAD
                let insert_pos = first_nad_idx.unwrap() + 1;
                for (i, sg3) in sg3_segs.drain(..).enumerate() {
                    result.insert(insert_pos + i, sg3);
                }
                past_sg2 = true;
            } else if !past_sg2 && first_nad_idx.is_some() {
                past_sg2 = true;
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

/// Full pipeline roundtrip for a PID using its generated fixture.
fn run_generated_roundtrip(pid: &str) {
    let Some(fixture_path) = test_utils::discover_generated_fixture(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: no generated fixture found");
        return;
    };

    let Some(filtered_mig) = test_utils::load_pid_filtered_mig(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
        return;
    };

    let tx_dir = test_utils::pid_dir(pid);
    if !test_utils::message_dir().exists() || !tx_dir.exists() {
        eprintln!("Skipping roundtrip for PID {pid}: mapping directories not found");
        return;
    }
    let (msg_engine, tx_engine) = test_utils::load_split_engines(pid);

    let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();

    if KNOWN_INCOMPLETE.contains(&fixture_name) {
        eprintln!("PID {pid}: {fixture_name} -- SKIPPED (known incomplete mapping)");
        return;
    }

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

    // Step 3: Forward mapping → MappedMessage
    let mapped =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);

    assert!(
        !mapped.transaktionen.is_empty(),
        "PID {pid} ({fixture_name}): forward mapping should produce at least one transaction"
    );

    // Step 3b: BO4E schema validation
    let mapped_for_validation =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", false);
    bo4e_validation::validate_mapped_message(
        pid,
        fixture_name,
        &msg_engine,
        &tx_engine,
        &mapped_for_validation,
    );

    // Step 4: Reverse mapping → AssembledTree
    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4", Some(&filtered_mig));

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
    let delimiters = edifact_types::EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    // Step 6: Normalize SG2/SG3 ordering (generated fixtures may differ from reverse)
    let original_normalized = normalize_sg2_sg3_ordering(&original_rendered);
    let reverse_normalized = normalize_sg2_sg3_ordering(&reverse_rendered);

    // Step 7: Compare segment tags (using normalized)
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
        "PID {pid} ({fixture_name}): segment tags should match after forward→reverse roundtrip"
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

// ── TOML loading tests ──────────────────────────────────────────────────

macro_rules! toml_loading_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let tx_dir = test_utils::pid_dir($pid);
            if !test_utils::message_dir().exists() || !tx_dir.exists() {
                eprintln!("Skipping {}: mapping dirs not found", stringify!($name));
                return;
            }
            let (msg_engine, tx_engine) = test_utils::load_split_engines($pid);
            eprintln!(
                "PID {} TOML loading OK: {} message + {} transaction definitions",
                $pid,
                msg_engine.definitions().len(),
                tx_engine.definitions().len()
            );
        }
    };
}

macro_rules! roundtrip_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            run_generated_roundtrip($pid);
        }
    };
}

// TOML loading tests for all 119 new PIDs
toml_loading_test!(test_toml_loading_55039, "55039");
toml_loading_test!(test_toml_loading_55040, "55040");
toml_loading_test!(test_toml_loading_55041, "55041");
toml_loading_test!(test_toml_loading_55043, "55043");
toml_loading_test!(test_toml_loading_55044, "55044");
toml_loading_test!(test_toml_loading_55051, "55051");
toml_loading_test!(test_toml_loading_55052, "55052");
toml_loading_test!(test_toml_loading_55053, "55053");
toml_loading_test!(test_toml_loading_55060, "55060");
toml_loading_test!(test_toml_loading_55062, "55062");
toml_loading_test!(test_toml_loading_55063, "55063");
toml_loading_test!(test_toml_loading_55064, "55064");
toml_loading_test!(test_toml_loading_55065, "55065");
toml_loading_test!(test_toml_loading_55066, "55066");
toml_loading_test!(test_toml_loading_55067, "55067");
toml_loading_test!(test_toml_loading_55069, "55069");
toml_loading_test!(test_toml_loading_55070, "55070");
toml_loading_test!(test_toml_loading_55071, "55071");
toml_loading_test!(test_toml_loading_55072, "55072");
toml_loading_test!(test_toml_loading_55073, "55073");
toml_loading_test!(test_toml_loading_55074, "55074");
toml_loading_test!(test_toml_loading_55075, "55075");
toml_loading_test!(test_toml_loading_55076, "55076");
toml_loading_test!(test_toml_loading_55077, "55077");
toml_loading_test!(test_toml_loading_55078, "55078");
toml_loading_test!(test_toml_loading_55080, "55080");
toml_loading_test!(test_toml_loading_55095, "55095");
toml_loading_test!(test_toml_loading_55168, "55168");
toml_loading_test!(test_toml_loading_55169, "55169");
toml_loading_test!(test_toml_loading_55170, "55170");
toml_loading_test!(test_toml_loading_55173, "55173");
toml_loading_test!(test_toml_loading_55177, "55177");
toml_loading_test!(test_toml_loading_55194, "55194");
toml_loading_test!(test_toml_loading_55195, "55195");
toml_loading_test!(test_toml_loading_55196, "55196");
toml_loading_test!(test_toml_loading_55197, "55197");
toml_loading_test!(test_toml_loading_55198, "55198");
toml_loading_test!(test_toml_loading_55199, "55199");
toml_loading_test!(test_toml_loading_55200, "55200");
toml_loading_test!(test_toml_loading_55201, "55201");
toml_loading_test!(test_toml_loading_55202, "55202");
toml_loading_test!(test_toml_loading_55203, "55203");
toml_loading_test!(test_toml_loading_55204, "55204");
toml_loading_test!(test_toml_loading_55205, "55205");
toml_loading_test!(test_toml_loading_55206, "55206");
toml_loading_test!(test_toml_loading_55207, "55207");
toml_loading_test!(test_toml_loading_55208, "55208");
toml_loading_test!(test_toml_loading_55209, "55209");
toml_loading_test!(test_toml_loading_55210, "55210");
toml_loading_test!(test_toml_loading_55211, "55211");
toml_loading_test!(test_toml_loading_55212, "55212");
toml_loading_test!(test_toml_loading_55213, "55213");
toml_loading_test!(test_toml_loading_55214, "55214");
toml_loading_test!(test_toml_loading_55223, "55223");
toml_loading_test!(test_toml_loading_55224, "55224");
toml_loading_test!(test_toml_loading_55227, "55227");
toml_loading_test!(test_toml_loading_55230, "55230");
toml_loading_test!(test_toml_loading_55235, "55235");
toml_loading_test!(test_toml_loading_55236, "55236");
toml_loading_test!(test_toml_loading_55237, "55237");
toml_loading_test!(test_toml_loading_55238, "55238");
toml_loading_test!(test_toml_loading_55239, "55239");
toml_loading_test!(test_toml_loading_55240, "55240");
toml_loading_test!(test_toml_loading_55241, "55241");
toml_loading_test!(test_toml_loading_55242, "55242");
toml_loading_test!(test_toml_loading_55243, "55243");
toml_loading_test!(test_toml_loading_55557, "55557");
toml_loading_test!(test_toml_loading_55559, "55559");
toml_loading_test!(test_toml_loading_55601, "55601");
toml_loading_test!(test_toml_loading_55603, "55603");
toml_loading_test!(test_toml_loading_55605, "55605");
toml_loading_test!(test_toml_loading_55607, "55607");
toml_loading_test!(test_toml_loading_55608, "55608");
toml_loading_test!(test_toml_loading_55609, "55609");
toml_loading_test!(test_toml_loading_55611, "55611");
toml_loading_test!(test_toml_loading_55613, "55613");
toml_loading_test!(test_toml_loading_55614, "55614");
toml_loading_test!(test_toml_loading_55619, "55619");
toml_loading_test!(test_toml_loading_55625, "55625");
toml_loading_test!(test_toml_loading_55627, "55627");
toml_loading_test!(test_toml_loading_55628, "55628");
toml_loading_test!(test_toml_loading_55629, "55629");
toml_loading_test!(test_toml_loading_55630, "55630");
toml_loading_test!(test_toml_loading_55632, "55632");
toml_loading_test!(test_toml_loading_55633, "55633");
toml_loading_test!(test_toml_loading_55634, "55634");
toml_loading_test!(test_toml_loading_55635, "55635");
toml_loading_test!(test_toml_loading_55636, "55636");
toml_loading_test!(test_toml_loading_55638, "55638");
toml_loading_test!(test_toml_loading_55639, "55639");
toml_loading_test!(test_toml_loading_55642, "55642");
toml_loading_test!(test_toml_loading_55644, "55644");
toml_loading_test!(test_toml_loading_55645, "55645");
toml_loading_test!(test_toml_loading_55647, "55647");
toml_loading_test!(test_toml_loading_55652, "55652");
toml_loading_test!(test_toml_loading_55657, "55657");
toml_loading_test!(test_toml_loading_55659, "55659");
toml_loading_test!(test_toml_loading_55660, "55660");
toml_loading_test!(test_toml_loading_55661, "55661");
toml_loading_test!(test_toml_loading_55662, "55662");
toml_loading_test!(test_toml_loading_55663, "55663");
toml_loading_test!(test_toml_loading_55664, "55664");
toml_loading_test!(test_toml_loading_55665, "55665");
toml_loading_test!(test_toml_loading_55666, "55666");
toml_loading_test!(test_toml_loading_55667, "55667");
toml_loading_test!(test_toml_loading_55669, "55669");
toml_loading_test!(test_toml_loading_55670, "55670");
toml_loading_test!(test_toml_loading_55671, "55671");
toml_loading_test!(test_toml_loading_55672, "55672");
toml_loading_test!(test_toml_loading_55673, "55673");
toml_loading_test!(test_toml_loading_55674, "55674");
toml_loading_test!(test_toml_loading_55675, "55675");
toml_loading_test!(test_toml_loading_55684, "55684");
toml_loading_test!(test_toml_loading_55685, "55685");
toml_loading_test!(test_toml_loading_55686, "55686");
toml_loading_test!(test_toml_loading_55687, "55687");
toml_loading_test!(test_toml_loading_55688, "55688");
toml_loading_test!(test_toml_loading_55689, "55689");
toml_loading_test!(test_toml_loading_55690, "55690");

// Full roundtrip tests for all 119 new PIDs
roundtrip_test!(test_roundtrip_55039, "55039");
roundtrip_test!(test_roundtrip_55040, "55040");
roundtrip_test!(test_roundtrip_55041, "55041");
roundtrip_test!(test_roundtrip_55043, "55043");
roundtrip_test!(test_roundtrip_55044, "55044");
roundtrip_test!(test_roundtrip_55051, "55051");
roundtrip_test!(test_roundtrip_55052, "55052");
roundtrip_test!(test_roundtrip_55053, "55053");
roundtrip_test!(test_roundtrip_55060, "55060");
roundtrip_test!(test_roundtrip_55062, "55062");
roundtrip_test!(test_roundtrip_55063, "55063");
roundtrip_test!(test_roundtrip_55064, "55064");
roundtrip_test!(test_roundtrip_55065, "55065");
roundtrip_test!(test_roundtrip_55066, "55066");
roundtrip_test!(test_roundtrip_55067, "55067");
roundtrip_test!(test_roundtrip_55069, "55069");
roundtrip_test!(test_roundtrip_55070, "55070");
roundtrip_test!(test_roundtrip_55071, "55071");
roundtrip_test!(test_roundtrip_55072, "55072");
roundtrip_test!(test_roundtrip_55073, "55073");
roundtrip_test!(test_roundtrip_55074, "55074");
roundtrip_test!(test_roundtrip_55075, "55075");
roundtrip_test!(test_roundtrip_55076, "55076");
roundtrip_test!(test_roundtrip_55077, "55077");
roundtrip_test!(test_roundtrip_55078, "55078");
roundtrip_test!(test_roundtrip_55080, "55080");
roundtrip_test!(test_roundtrip_55095, "55095");
roundtrip_test!(test_roundtrip_55168, "55168");
roundtrip_test!(test_roundtrip_55169, "55169");
roundtrip_test!(test_roundtrip_55170, "55170");
roundtrip_test!(test_roundtrip_55173, "55173");
roundtrip_test!(test_roundtrip_55177, "55177");
roundtrip_test!(test_roundtrip_55194, "55194");
roundtrip_test!(test_roundtrip_55195, "55195");
roundtrip_test!(test_roundtrip_55196, "55196");
roundtrip_test!(test_roundtrip_55197, "55197");
roundtrip_test!(test_roundtrip_55198, "55198");
roundtrip_test!(test_roundtrip_55199, "55199");
roundtrip_test!(test_roundtrip_55200, "55200");
roundtrip_test!(test_roundtrip_55201, "55201");
roundtrip_test!(test_roundtrip_55202, "55202");
roundtrip_test!(test_roundtrip_55203, "55203");
roundtrip_test!(test_roundtrip_55204, "55204");
roundtrip_test!(test_roundtrip_55205, "55205");
roundtrip_test!(test_roundtrip_55206, "55206");
roundtrip_test!(test_roundtrip_55207, "55207");
roundtrip_test!(test_roundtrip_55208, "55208");
roundtrip_test!(test_roundtrip_55209, "55209");
roundtrip_test!(test_roundtrip_55210, "55210");
roundtrip_test!(test_roundtrip_55211, "55211");
roundtrip_test!(test_roundtrip_55212, "55212");
roundtrip_test!(test_roundtrip_55213, "55213");
roundtrip_test!(test_roundtrip_55214, "55214");
roundtrip_test!(test_roundtrip_55223, "55223");
roundtrip_test!(test_roundtrip_55224, "55224");
roundtrip_test!(test_roundtrip_55227, "55227");
roundtrip_test!(test_roundtrip_55230, "55230");
roundtrip_test!(test_roundtrip_55235, "55235");
roundtrip_test!(test_roundtrip_55236, "55236");
roundtrip_test!(test_roundtrip_55237, "55237");
roundtrip_test!(test_roundtrip_55238, "55238");
roundtrip_test!(test_roundtrip_55239, "55239");
roundtrip_test!(test_roundtrip_55240, "55240");
roundtrip_test!(test_roundtrip_55241, "55241");
roundtrip_test!(test_roundtrip_55242, "55242");
roundtrip_test!(test_roundtrip_55243, "55243");
roundtrip_test!(test_roundtrip_55557, "55557");
roundtrip_test!(test_roundtrip_55559, "55559");
roundtrip_test!(test_roundtrip_55601, "55601");
roundtrip_test!(test_roundtrip_55603, "55603");
roundtrip_test!(test_roundtrip_55605, "55605");
roundtrip_test!(test_roundtrip_55607, "55607");
roundtrip_test!(test_roundtrip_55608, "55608");
roundtrip_test!(test_roundtrip_55609, "55609");
roundtrip_test!(test_roundtrip_55611, "55611");
roundtrip_test!(test_roundtrip_55613, "55613");
roundtrip_test!(test_roundtrip_55614, "55614");
roundtrip_test!(test_roundtrip_55619, "55619");
roundtrip_test!(test_roundtrip_55625, "55625");
roundtrip_test!(test_roundtrip_55627, "55627");
roundtrip_test!(test_roundtrip_55628, "55628");
roundtrip_test!(test_roundtrip_55629, "55629");
roundtrip_test!(test_roundtrip_55630, "55630");
roundtrip_test!(test_roundtrip_55632, "55632");
roundtrip_test!(test_roundtrip_55633, "55633");
roundtrip_test!(test_roundtrip_55634, "55634");
roundtrip_test!(test_roundtrip_55635, "55635");
roundtrip_test!(test_roundtrip_55636, "55636");
roundtrip_test!(test_roundtrip_55638, "55638");
roundtrip_test!(test_roundtrip_55639, "55639");
roundtrip_test!(test_roundtrip_55642, "55642");
roundtrip_test!(test_roundtrip_55644, "55644");
roundtrip_test!(test_roundtrip_55645, "55645");
roundtrip_test!(test_roundtrip_55647, "55647");
roundtrip_test!(test_roundtrip_55652, "55652");
roundtrip_test!(test_roundtrip_55657, "55657");
roundtrip_test!(test_roundtrip_55659, "55659");
roundtrip_test!(test_roundtrip_55660, "55660");
roundtrip_test!(test_roundtrip_55661, "55661");
roundtrip_test!(test_roundtrip_55662, "55662");
roundtrip_test!(test_roundtrip_55663, "55663");
roundtrip_test!(test_roundtrip_55664, "55664");
roundtrip_test!(test_roundtrip_55665, "55665");
roundtrip_test!(test_roundtrip_55666, "55666");
roundtrip_test!(test_roundtrip_55667, "55667");
roundtrip_test!(test_roundtrip_55669, "55669");
roundtrip_test!(test_roundtrip_55670, "55670");
roundtrip_test!(test_roundtrip_55671, "55671");
roundtrip_test!(test_roundtrip_55672, "55672");
roundtrip_test!(test_roundtrip_55673, "55673");
roundtrip_test!(test_roundtrip_55674, "55674");
roundtrip_test!(test_roundtrip_55675, "55675");
roundtrip_test!(test_roundtrip_55684, "55684");
roundtrip_test!(test_roundtrip_55685, "55685");
roundtrip_test!(test_roundtrip_55686, "55686");
roundtrip_test!(test_roundtrip_55687, "55687");
roundtrip_test!(test_roundtrip_55688, "55688");
roundtrip_test!(test_roundtrip_55689, "55689");
roundtrip_test!(test_roundtrip_55690, "55690");
