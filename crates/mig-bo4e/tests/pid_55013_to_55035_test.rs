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
use mig_bo4e::path_resolver::PathResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(std::path::Path::new(SCHEMA_DIR))
}

/// Discover all `.edi` fixture files for a given PID.
fn discover_fixtures(pid: &str) -> Vec<PathBuf> {
    let dir = Path::new(FIXTURE_DIR);
    if !dir.exists() {
        return vec![];
    }
    let prefix = format!("{pid}_");
    let mut fixtures: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(".edi"))
                .unwrap_or(false)
        })
        .collect();
    fixtures.sort();
    fixtures
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

/// Fixtures with known mapping gaps that prevent byte-identical roundtrip.
/// These are legitimate issues to fix later, not test bugs.
const KNOWN_INCOMPLETE: &[&str] = &[];

/// Fixtures with non-MIG-compliant segment ordering.
/// The roundtrip normalizes these to MIG-defined order, so byte-identical
/// comparison fails — but the data content is correct.
///
/// DEV-77392-3: ZD7 SG10 has CCI+ZB3 before CCI+Z49/ZF3, contradicting
/// the MIG which defines Steuerkanal (Z49, Nr 00091) before Zugeordnete
/// Marktpartner (ZB3, Nr 00092). All 4 other fixtures follow MIG order.
const KNOWN_WRONG_FIXTURE_ORDER: &[&str] = &["55035_UTILMD_S2.1_DEV-77392-3.edi"];

/// Full pipeline roundtrip for ALL fixtures of a PID:
/// EDIFACT -> tokenize -> split -> assemble -> map_interchange
/// -> map_interchange_reverse -> disassemble -> render -> compare with original.
///
/// Panics with detailed diagnostics if the roundtrip is not byte-identical.
fn run_full_roundtrip(pid: &str) {
    let fixtures = discover_fixtures(pid);
    if fixtures.is_empty() {
        eprintln!("Skipping roundtrip for PID {pid}: no fixtures found");
        return;
    }

    let Some(filtered_mig) = load_pid_filtered_mig(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
        return;
    };

    let msg_dir = message_dir();
    let tx_dir = pid_dir(pid);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping roundtrip for PID {pid}: mapping directories not found");
        return;
    }
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(path_resolver());
    let tx_engine = MappingEngine::load(&tx_dir)
        .unwrap()
        .with_path_resolver(path_resolver());

    let mut tested = 0;
    let mut skipped = 0;

    for fixture_path in &fixtures {
        let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();

        if KNOWN_INCOMPLETE.contains(&fixture_name) {
            eprintln!("PID {pid}: {fixture_name} -- SKIPPED (known incomplete mapping)");
            skipped += 1;
            continue;
        }

        if KNOWN_WRONG_FIXTURE_ORDER.contains(&fixture_name) {
            eprintln!("PID {pid}: {fixture_name} -- SKIPPED (non-MIG-compliant segment order)");
            skipped += 1;
            continue;
        }

        let edifact_input = std::fs::read_to_string(fixture_path).unwrap();

        // Step 1: Tokenize and split
        let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
        let chunks = split_messages(segments).unwrap();
        assert!(
            !chunks.messages.is_empty(),
            "PID {pid} ({fixture_name}): should have at least one message"
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
            "PID {pid} ({fixture_name}): forward mapping should produce at least one transaction"
        );

        // Step 4: Reverse mapping -> AssembledTree
        let mut reverse_tree =
            MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

        // Add UNH envelope (mapping engine handles content only)
        let unh_assembled = owned_to_assembled(&msg_chunk.unh);
        reverse_tree.segments.insert(0, unh_assembled);
        reverse_tree.post_group_start += 1;

        // Only add UNT if the assembler captured it in the original tree.
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

        // Step 7: Compare full rendered EDIFACT (byte-identical check)
        if original_rendered != reverse_rendered {
            let orig_segs: Vec<&str> = original_rendered.split('\'').collect();
            let rev_segs: Vec<&str> = reverse_rendered.split('\'').collect();
            let max_len = orig_segs.len().max(rev_segs.len());
            let mut diff_count = 0;
            for i in 0..max_len {
                let o = orig_segs.get(i).unwrap_or(&"<missing>");
                let r = rev_segs.get(i).unwrap_or(&"<missing>");
                if o != r {
                    eprintln!("PID {pid} ({fixture_name}): segment {i} differs:");
                    eprintln!("  original: {o}");
                    eprintln!("  reversed: {r}");
                    diff_count += 1;
                }
            }
            eprintln!(
                "PID {pid} ({fixture_name}): {diff_count} segment(s) differ out of {}",
                orig_segs.len()
            );
        }

        assert_eq!(
            original_rendered, reverse_rendered,
            "PID {pid} ({fixture_name}): full EDIFACT roundtrip should be byte-identical"
        );

        eprintln!(
            "PID {pid}: {fixture_name} -- roundtrip OK, {} segments byte-identical",
            original_tags.len()
        );
        tested += 1;
    }

    eprintln!("PID {pid}: {tested} fixtures passed, {skipped} skipped",);
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
                .map(|e| e.with_path_resolver(path_resolver()))
                .unwrap_or_else(|e| panic!("Failed to load message engine: {e}"));
            let tx_engine = MappingEngine::load(&tx_dir)
                .map(|e| e.with_path_resolver(path_resolver()))
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

// Full EDIFACT roundtrip tests (PIDs with fixtures -- tests all available fixtures per PID)

#[test]
fn test_roundtrip_55013() {
    run_full_roundtrip("55013");
}
#[test]
fn test_roundtrip_55014() {
    run_full_roundtrip("55014");
}
#[test]
fn test_roundtrip_55015() {
    run_full_roundtrip("55015");
}
#[test]
fn test_roundtrip_55016() {
    run_full_roundtrip("55016");
}
#[test]
fn test_roundtrip_55017() {
    run_full_roundtrip("55017");
}
#[test]
fn test_roundtrip_55018() {
    run_full_roundtrip("55018");
}
#[test]
fn test_roundtrip_55035() {
    run_full_roundtrip("55035");
}
// PIDs 55022, 55023, 55024 have no fixture files -- TOML loading tests only.
