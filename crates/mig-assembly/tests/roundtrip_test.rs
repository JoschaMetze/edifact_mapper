//! Roundtrip tests using real EDIFACT fixtures.
//!
//! Validates: EDIFACT → assemble → disassemble → render
//!
//! The roundtrip preserves all segments the assembler can capture. The
//! assembler's coverage depends on MIG variant matching — some complex MIG
//! schemas have multiple variants of the same group (e.g., SG4 for different
//! PIDs) that the assembler can only partially match. Full byte-identical
//! roundtrip requires unified MIG group variants.

use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::roundtrip::roundtrip;
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
fn test_roundtrip_single_fixture() {
    let Some(mig) = load_real_mig() else { return };

    let fixture_path = Path::new(FIXTURE_DIR).join("55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !fixture_path.exists() {
        eprintln!("Fixture not found, skipping");
        return;
    }

    let content = std::fs::read(&fixture_path).unwrap();
    let result = roundtrip(&content, &mig);
    assert!(result.is_ok(), "Roundtrip failed: {:?}", result.err());

    let output = result.unwrap();
    let input_str = String::from_utf8_lossy(&content);

    if output == input_str.as_ref() {
        eprintln!("Byte-identical roundtrip!");
    } else {
        let diff_pos = input_str
            .bytes()
            .zip(output.bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(input_str.len().min(output.len()));
        eprintln!(
            "First diff at byte {diff_pos}, input_len={}, output_len={}",
            input_str.len(),
            output.len()
        );
    }
}

/// Validate that the disassembler → renderer correctly reconstructs the segments
/// that the assembler captured. This tests the disassembler/renderer fidelity
/// independent of assembler coverage.
#[test]
fn test_roundtrip_assembled_segments_preserved() {
    let Some(mig) = load_real_mig() else { return };
    let fixture_dir = Path::new(FIXTURE_DIR);
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let delimiters = edifact_types::EdifactDelimiters::default();
    let mut total = 0;
    let mut perfect = 0;

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

        // Disassemble and render
        let disassembler = Disassembler::new(&mig);
        let dis_segments = disassembler.disassemble(&tree);
        let rendered = render_edifact(&dis_segments, &delimiters);

        // Re-parse the rendered output
        let re_segments = match parse_to_segments(rendered.as_bytes()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Each disassembled segment should match the re-parsed segment
        if dis_segments.len() == re_segments.len() {
            let all_match = dis_segments
                .iter()
                .zip(re_segments.iter())
                .all(|(dis, re)| dis.tag == re.id && dis.elements == re.elements);
            if all_match {
                perfect += 1;
            }
        }
    }

    let rate = if total > 0 {
        perfect as f64 / total as f64
    } else {
        1.0
    };
    eprintln!(
        "\nDisassembler roundtrip fidelity: {perfect}/{total} ({:.1}%)",
        rate * 100.0
    );

    // The disassembler/renderer should perfectly preserve all captured segments
    assert!(
        rate > 0.95,
        "Disassembler roundtrip fidelity too low: {perfect}/{total} ({:.1}%)",
        rate * 100.0
    );
}

/// Measure the assembler coverage rate — what fraction of input segments
/// are captured by assembly + disassembly.
#[test]
fn test_roundtrip_coverage_all_fixtures() {
    let Some(mig) = load_real_mig() else { return };
    let fixture_dir = Path::new(FIXTURE_DIR);
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut total_input_segments = 0;
    let mut total_output_segments = 0;
    let mut file_count = 0;
    let mut roundtrip_ok = 0;

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("edi") {
            continue;
        }
        file_count += 1;

        let content = std::fs::read(&path).unwrap();
        let segments = match parse_to_segments(&content) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let input_count = segments.len();

        if let Ok(output) = roundtrip(&content, &mig) {
            let re_segments = parse_to_segments(output.as_bytes()).unwrap_or_default();
            total_input_segments += input_count;
            total_output_segments += re_segments.len();
            roundtrip_ok += 1;
        }
    }

    let coverage = if total_input_segments > 0 {
        total_output_segments as f64 / total_input_segments as f64
    } else {
        0.0
    };

    eprintln!("\nRoundtrip stats:");
    eprintln!("  Files processed: {roundtrip_ok}/{file_count}");
    eprintln!("  Input segments:  {total_input_segments}");
    eprintln!("  Output segments: {total_output_segments}");
    eprintln!("  Coverage: {:.1}%", coverage * 100.0);

    // All files should at least roundtrip without errors
    assert_eq!(roundtrip_ok, file_count, "Some files failed to roundtrip");

    // Segment coverage: assembler captures header segments (UNB, UNH, BGM, DTM)
    // plus some group content. Coverage improves as assembler handles more MIG variants.
    assert!(
        coverage > 0.3,
        "Segment coverage too low: {:.1}%",
        coverage * 100.0
    );
}
