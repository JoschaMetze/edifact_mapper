//! Integration tests for disassembler with real MIG schemas.

use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::tokenize::parse_to_segments;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";

fn load_real_mig() -> Option<MigSchema> {
    let path = Path::new(MIG_XML_PATH);
    if !path.exists() {
        eprintln!("MIG XML not found at {MIG_XML_PATH}, skipping");
        return None;
    }
    Some(parse_mig(path, "UTILMD", Some("Strom"), "FV2504").expect("Failed to parse MIG XML"))
}

#[test]
fn test_disassemble_real_fixture() {
    let Some(mig) = load_real_mig() else { return };

    let fixture_path = Path::new(FIXTURE_DIR).join("55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !fixture_path.exists() {
        eprintln!("Fixture not found, skipping");
        return;
    }

    let content = std::fs::read(&fixture_path).unwrap();
    let segments = parse_to_segments(&content).unwrap();

    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    let disassembler = Disassembler::new(&mig);
    let dis_segments = disassembler.disassemble(&tree);

    // Should produce segments
    assert!(
        !dis_segments.is_empty(),
        "Disassembly should produce segments"
    );

    // All tags should be non-empty
    for seg in &dis_segments {
        assert!(!seg.tag.is_empty(), "Segment tag should be non-empty");
    }

    eprintln!("Disassembled {} segments from fixture", dis_segments.len());
    for seg in &dis_segments {
        eprintln!("  {}", seg.tag);
    }
}

#[test]
fn test_disassemble_preserves_segment_count() {
    let Some(mig) = load_real_mig() else { return };
    let fixture_dir = Path::new(FIXTURE_DIR);
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut total = 0;
    let mut match_count = 0;

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("edi") {
            continue;
        }
        total += 1;

        let content = std::fs::read(&path).unwrap();
        let segments = match parse_to_segments(&content) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let assembler = Assembler::new(&mig);
        let tree = match assembler.assemble_generic(&segments) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let disassembler = Disassembler::new(&mig);
        let dis_segments = disassembler.disassemble(&tree);

        // Count total assembled segments for comparison
        let assembled_count = count_tree_segments(&tree);
        if dis_segments.len() == assembled_count {
            match_count += 1;
        } else {
            let name = path.file_name().unwrap().to_string_lossy();
            eprintln!(
                "{name}: assembled={assembled_count} disassembled={}",
                dis_segments.len()
            );
        }
    }

    eprintln!("\nSegment count match: {match_count}/{total}");
    if total > 0 {
        let rate = match_count as f64 / total as f64;
        assert!(
            rate > 0.9,
            "Segment count match rate too low: {match_count}/{total}"
        );
    }
}

fn count_tree_segments(tree: &mig_assembly::assembler::AssembledTree) -> usize {
    let mut count = tree.segments.len();
    for group in &tree.groups {
        count += count_group_segments(group);
    }
    count
}

fn count_group_segments(group: &mig_assembly::assembler::AssembledGroup) -> usize {
    let mut count = 0;
    for rep in &group.repetitions {
        count += rep.segments.len();
        for child in &rep.child_groups {
            count += count_group_segments(child);
        }
    }
    count
}
