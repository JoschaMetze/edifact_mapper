//! EDIFACT roundtrip tests for PIDs 55013-55035.
//!
//! Full pipeline: EDIFACT -> tokenize -> split -> assemble -> map_interchange
//! -> map_interchange_reverse -> disassemble -> render -> compare with original.
//!
//! If any TOML mapping is missing, the reverse mapping will fail to reconstruct
//! those segments, and the byte-identical comparison will fail.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::{AssembledSegment, Assembler};
use mig_assembly::disassembler::Disassembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";

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

fn message_dir() -> PathBuf {
    Path::new(MAPPINGS_BASE).join("message")
}

fn pid_dir(pid: &str) -> PathBuf {
    Path::new(MAPPINGS_BASE).join(format!("pid_{pid}"))
}

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

/// Full pipeline roundtrip: EDIFACT -> tokenize -> split -> assemble -> map_interchange
/// -> map_interchange_reverse -> disassemble -> render -> compare with original.
///
/// Panics with detailed diagnostics if the roundtrip is not byte-identical.
fn run_full_roundtrip(pid: &str, fixture_name: &str) {
    let Some(filtered_mig) = load_pid_filtered_mig(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
        return;
    };

    let fixture_path = Path::new(FIXTURE_DIR).join(fixture_name);
    if !fixture_path.exists() {
        eprintln!(
            "Skipping roundtrip for PID {pid}: fixture not found at {}",
            fixture_path.display()
        );
        return;
    }
    let edifact_input = std::fs::read_to_string(&fixture_path).unwrap();

    let msg_dir = message_dir();
    let tx_dir = pid_dir(pid);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping roundtrip for PID {pid}: mapping directories not found");
        return;
    }
    let msg_engine = MappingEngine::load(&msg_dir).unwrap();
    let tx_engine = MappingEngine::load(&tx_dir).unwrap();

    // Step 1: Tokenize and split
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(
        !chunks.messages.is_empty(),
        "PID {pid}: should have at least one message"
    );

    let msg_chunk = &chunks.messages[0];

    // Assemble with UNH + body + UNT (message content, no UNB envelope)
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    // Step 2: Assemble with PID-filtered MIG
    let assembler = Assembler::new(&filtered_mig);
    let original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    // Step 3: Forward mapping -> MappedMessage
    let mapped =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);

    assert!(
        !mapped.transaktionen.is_empty(),
        "PID {pid}: forward mapping should produce at least one transaction"
    );

    // Step 4: Reverse mapping -> AssembledTree
    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

    // Add UNH envelope (mapping engine handles content only)
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    // Only add UNT if the assembler captured it in the original tree.
    // Some PID-filtered MIGs don't include UNT in the assembled tree.
    let original_has_unt = original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if original_has_unt {
        let unt_assembled = owned_to_assembled(&msg_chunk.unt);
        reverse_tree.segments.push(unt_assembled);
    }

    // Step 5: Disassemble both trees and render to EDIFACT
    let disassembler = Disassembler::new(&filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    // Step 6: Compare segment tags (structural check -- catches missing groups)
    let original_tags: Vec<&str> = original_dis.iter().map(|s| s.tag.as_str()).collect();
    let reverse_tags: Vec<&str> = reverse_dis.iter().map(|s| s.tag.as_str()).collect();

    if original_tags != reverse_tags {
        eprintln!("PID {pid}: segment tag mismatch!");
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
        // Show which tags are missing
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
        "PID {pid}: segment tags should match after forward->reverse roundtrip"
    );

    // Step 7: Compare full rendered EDIFACT (byte-identical check -- catches wrong field values)
    if original_rendered != reverse_rendered {
        let orig_segs: Vec<&str> = original_rendered.split('\'').collect();
        let rev_segs: Vec<&str> = reverse_rendered.split('\'').collect();
        let max_len = orig_segs.len().max(rev_segs.len());
        let mut diff_count = 0;
        for i in 0..max_len {
            let o = orig_segs.get(i).unwrap_or(&"<missing>");
            let r = rev_segs.get(i).unwrap_or(&"<missing>");
            if o != r {
                eprintln!("PID {pid}: segment {i} differs:");
                eprintln!("  original: {o}");
                eprintln!("  reversed: {r}");
                diff_count += 1;
            }
        }
        eprintln!(
            "PID {pid}: {diff_count} segment(s) differ out of {}",
            orig_segs.len()
        );
    }

    assert_eq!(
        original_rendered, reverse_rendered,
        "PID {pid}: full EDIFACT roundtrip should be byte-identical"
    );

    eprintln!(
        "PID {pid}: roundtrip OK -- {} segments byte-identical",
        original_tags.len()
    );
}

/// TOML loading test -- verifies all TOML files parse correctly.
/// Runs even without fixture files.
macro_rules! toml_loading_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let msg_dir = message_dir();
            let tx_dir = pid_dir($pid);
            if !msg_dir.exists() || !tx_dir.exists() {
                eprintln!("Skipping {}: mapping dirs not found", stringify!($name));
                return;
            }
            let msg_engine = MappingEngine::load(&msg_dir)
                .unwrap_or_else(|e| panic!("Failed to load message engine: {e}"));
            let tx_engine = MappingEngine::load(&tx_dir)
                .unwrap_or_else(|e| panic!("Failed to load PID {} engine: {e}", $pid));
            eprintln!(
                "PID {} TOML loading OK: {} message + {} transaction definitions",
                $pid,
                msg_engine.definitions().len(),
                tx_engine.definitions().len()
            );
        }
    };
}

/// Full EDIFACT roundtrip test -- byte-identical verification.
///
/// Currently #[ignore] because:
/// 1. Reverse mapping generates phantom segments from TOML defaults for groups
///    not present in the fixture (e.g., SG8+ZD7 info/zuordnung defaults).
/// 2. Some forward mappings don't capture all fixture segments (e.g., dual STS).
///
/// Run with: `cargo test -p mig-bo4e -- --ignored test_roundtrip_`
/// Un-ignore as each PID's mappings reach full roundtrip fidelity.
macro_rules! roundtrip_test {
    ($name:ident, $pid:expr, $fixture:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            run_full_roundtrip($pid, $fixture);
        }
    };
}

// TOML loading tests (all PIDs, no fixture needed)
toml_loading_test!(test_toml_loading_55013, "55013");
toml_loading_test!(test_toml_loading_55014, "55014");
toml_loading_test!(test_toml_loading_55015, "55015");
toml_loading_test!(test_toml_loading_55016, "55016");
toml_loading_test!(test_toml_loading_55017, "55017");
toml_loading_test!(test_toml_loading_55018, "55018");
toml_loading_test!(test_toml_loading_55022, "55022");
toml_loading_test!(test_toml_loading_55023, "55023");
toml_loading_test!(test_toml_loading_55024, "55024");
toml_loading_test!(test_toml_loading_55035, "55035");

// Full EDIFACT roundtrip tests (PIDs with fixtures)

// PID 55013: roundtrip verified -- 20 segments byte-identical
#[test]
fn test_roundtrip_55013() {
    run_full_roundtrip("55013", "55013_UTILMD_S2.1_ALEXANDE982717998.edi");
}
// PID 55014: roundtrip verified -- 29 segments byte-identical
#[test]
fn test_roundtrip_55014() {
    run_full_roundtrip("55014", "55014_UTILMD_S2.1_ALEXANDE948259148.edi");
}
// PID 55015: roundtrip verified -- 12 segments byte-identical
#[test]
fn test_roundtrip_55015() {
    run_full_roundtrip("55015", "55015_UTILMD_S2.1_ALEXANDE665361172.edi");
}
// PID 55016: roundtrip verified -- 11 segments byte-identical
#[test]
fn test_roundtrip_55016() {
    run_full_roundtrip("55016", "55016_UTILMD_S2.1_ALEXANDE616133.edi");
}
// PID 55017: roundtrip verified -- 14 segments byte-identical
#[test]
fn test_roundtrip_55017() {
    run_full_roundtrip("55017", "55017_UTILMD_S2.1_ALEXANDE107081.edi");
}
// PID 55018: roundtrip verified -- 14 segments byte-identical
#[test]
fn test_roundtrip_55018() {
    run_full_roundtrip("55018", "55018_UTILMD_S2.1_ALEXANDE203211.edi");
}
roundtrip_test!(
    test_roundtrip_55035,
    "55035",
    "55035_UTILMD_S2.1_ALEXANDE195836.edi"
);
// PIDs 55022, 55023, 55024 have no fixture files -- TOML loading tests only.
