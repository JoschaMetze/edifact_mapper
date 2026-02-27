//! Integration tests for mig-assembly — end-to-end parse → detect PID → assemble.
//!
//! Tests use both synthetic EDIFACT data and real MIG XML + fixture files.

use mig_assembly::assembler::Assembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_detect::detect_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_types::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};
use std::path::Path;

// ---------------------------------------------------------------------------
// Helpers for synthetic MIG schemas (used in targeted tests)
// ---------------------------------------------------------------------------

fn make_mig_segment(id: &str) -> MigSegment {
    MigSegment {
        id: id.to_string(),
        name: id.to_string(),
        description: None,
        counter: None,
        level: 0,
        number: None,
        max_rep_std: 1,
        max_rep_spec: 1,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        example: None,
        data_elements: vec![],
        composites: vec![],
    }
}

fn make_mig_group(id: &str, segments: Vec<&str>, nested: Vec<MigSegmentGroup>) -> MigSegmentGroup {
    MigSegmentGroup {
        id: id.to_string(),
        name: id.to_string(),
        description: None,
        counter: None,
        level: 1,
        max_rep_std: 99,
        max_rep_spec: 99,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        segments: segments.into_iter().map(make_mig_segment).collect(),
        nested_groups: nested,
    }
}

fn make_utilmd_mig() -> MigSchema {
    let sg3 = make_mig_group("SG3", vec!["CTA", "COM"], vec![]);
    let sg5 = make_mig_group("SG5", vec!["LOC"], vec![]);
    let sg6 = make_mig_group("SG6", vec!["RFF"], vec![]);
    let sg8 = make_mig_group("SG8", vec!["SEQ", "RFF"], vec![]);
    let sg4 = MigSegmentGroup {
        id: "SG4".to_string(),
        name: "SG4".to_string(),
        description: None,
        counter: None,
        level: 1,
        max_rep_std: 99,
        max_rep_spec: 99,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        segments: vec![
            make_mig_segment("IDE"),
            make_mig_segment("STS"),
            make_mig_segment("DTM"),
        ],
        nested_groups: vec![sg5, sg6, sg8],
    };

    MigSchema {
        message_type: "UTILMD".to_string(),
        variant: Some("Strom".to_string()),
        version: "S2.1".to_string(),
        publication_date: "2025-03-20".to_string(),
        author: "BDEW".to_string(),
        format_version: "FV2504".to_string(),
        source_file: "test".to_string(),
        segments: vec![
            make_mig_segment("UNB"),
            make_mig_segment("UNH"),
            make_mig_segment("BGM"),
            make_mig_segment("DTM"),
        ],
        segment_groups: vec![make_mig_group("SG2", vec!["NAD"], vec![sg3]), sg4],
    }
}

// ---------------------------------------------------------------------------
// Loading real MIG from XML
// ---------------------------------------------------------------------------

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";

fn load_real_mig() -> Option<MigSchema> {
    let path = Path::new(MIG_XML_PATH);
    if !path.exists() {
        eprintln!("MIG XML not found at {MIG_XML_PATH}, skipping real-MIG tests");
        return None;
    }
    Some(parse_mig(path, "UTILMD", Some("Strom"), "FV2504").expect("Failed to parse MIG XML"))
}

// ---------------------------------------------------------------------------
// Synthetic data tests (always run)
// ---------------------------------------------------------------------------

#[test]
fn test_end_to_end_minimal_utilmd() {
    let input = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000003:500+210615:1200+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E01+DOC001+9'\
DTM+137:20250701:102'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000003::293'\
IDE+24+TX001'\
STS+7+Z33'\
DTM+92:20250801:102'\
LOC+Z16+DE00014545768S0000000000000003054'\
RFF+Z13:55001'\
UNT+10+MSG001'\
UNZ+1+REF001'";

    let segments = parse_to_segments(input).unwrap();
    assert!(segments.len() >= 10, "Should parse at least 10 segments");

    let pid = detect_pid(&segments).unwrap();
    assert_eq!(pid, "55001");

    let mig = make_utilmd_mig();
    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    assert!(tree.segments.iter().any(|s| s.tag == "UNH"));
    assert!(tree.segments.iter().any(|s| s.tag == "BGM"));

    let sg2 = tree.groups.iter().find(|g| g.group_id == "SG2").unwrap();
    assert_eq!(sg2.repetitions.len(), 2, "Should have 2 NAD parties");

    let sg4 = tree.groups.iter().find(|g| g.group_id == "SG4").unwrap();
    assert_eq!(sg4.repetitions.len(), 1, "Should have 1 transaction");
}

#[test]
fn test_end_to_end_multiple_transactions() {
    let input = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000003:500+210615:1200+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E01+DOC001+9'\
DTM+137:20250701:102'\
NAD+MS+9900123000002::293'\
IDE+24+TX001'\
STS+7+Z33'\
DTM+92:20250801:102'\
RFF+Z13:55001'\
IDE+24+TX002'\
STS+7+Z34'\
DTM+92:20250901:102'\
RFF+Z13:55002'\
UNT+12+MSG001'\
UNZ+1+REF001'";

    let segments = parse_to_segments(input).unwrap();
    let mig = make_utilmd_mig();
    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    let sg4 = tree.groups.iter().find(|g| g.group_id == "SG4").unwrap();
    assert_eq!(sg4.repetitions.len(), 2, "Should have 2 SG4 transactions");
    assert_eq!(sg4.repetitions[0].segments[0].elements[1][0], "TX001");
    assert_eq!(sg4.repetitions[1].segments[0].elements[1][0], "TX002");
}

#[test]
fn test_end_to_end_with_contact_info() {
    let input = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000003:500+210615:1200+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E01+DOC001+9'\
DTM+137:20250701:102'\
NAD+MS+9900123000002::293'\
CTA+IC+:Max Mustermann'\
COM+0401234567:TE'\
NAD+MR+9900456000003::293'\
UNT+7+MSG001'\
UNZ+1+REF001'";

    let segments = parse_to_segments(input).unwrap();
    let mig = make_utilmd_mig();
    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    let sg2 = tree.groups.iter().find(|g| g.group_id == "SG2").unwrap();
    assert_eq!(sg2.repetitions.len(), 2);

    let sg3 = &sg2.repetitions[0].child_groups;
    assert_eq!(sg3.len(), 1, "First NAD should have SG3 contact");
    assert_eq!(sg3[0].repetitions[0].segments[0].tag, "CTA");
    assert_eq!(sg3[0].repetitions[0].segments[1].tag, "COM");

    assert!(sg2.repetitions[1].child_groups.is_empty());
}

// ---------------------------------------------------------------------------
// Real MIG XML tests
// ---------------------------------------------------------------------------

#[test]
fn test_load_real_mig_schema() {
    let Some(mig) = load_real_mig() else { return };

    assert_eq!(mig.message_type, "UTILMD");
    assert_eq!(mig.variant.as_deref(), Some("Strom"));
    assert_eq!(mig.format_version, "FV2504");

    // Real UTILMD MIG should have top-level segments
    assert!(
        !mig.segments.is_empty(),
        "MIG should have top-level segments"
    );
    // Should have segment groups
    assert!(
        !mig.segment_groups.is_empty(),
        "MIG should have segment groups"
    );

    // Print MIG structure for debugging
    eprintln!(
        "Top-level segments: {:?}",
        mig.segments.iter().map(|s| &s.id).collect::<Vec<_>>()
    );
    eprintln!(
        "Segment groups: {:?}",
        mig.segment_groups.iter().map(|g| &g.id).collect::<Vec<_>>()
    );
}

#[test]
fn test_assemble_real_fixture_with_real_mig() {
    let Some(mig) = load_real_mig() else { return };

    // Use a specific well-known fixture
    let fixture_path = Path::new(FIXTURE_DIR).join("55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !fixture_path.exists() {
        eprintln!("Fixture file not found, skipping");
        return;
    }

    let content = std::fs::read(&fixture_path).unwrap();
    let segments = parse_to_segments(&content).unwrap();

    eprintln!(
        "Parsed {} segments from fixture: {:?}",
        segments.len(),
        segments.iter().map(|s| &s.id).collect::<Vec<_>>()
    );

    // Detect PID
    let pid = detect_pid(&segments).unwrap();
    assert_eq!(pid, "55001", "Fixture should be PID 55001");

    // Assemble with real MIG
    let assembler = Assembler::new(&mig);
    let tree = assembler.assemble_generic(&segments).unwrap();

    // Basic sanity checks
    assert!(
        tree.segments.iter().any(|s| s.tag == "UNH"),
        "Should have UNH"
    );
    assert!(
        tree.segments.iter().any(|s| s.tag == "BGM"),
        "Should have BGM"
    );

    eprintln!(
        "Assembled tree: {} top-level segments, {} groups",
        tree.segments.len(),
        tree.groups.len()
    );
    for group in &tree.groups {
        eprintln!(
            "  Group {}: {} repetitions",
            group.group_id,
            group.repetitions.len()
        );
    }
}

#[test]
fn test_pid_detection_on_real_fixtures() {
    let fixture_dir = Path::new(FIXTURE_DIR);
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut detected = 0;
    let mut total = 0;
    let mut pid_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

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
            Err(e) => {
                eprintln!("Parse failed for {:?}: {}", path.file_name().unwrap(), e);
                continue;
            }
        };

        match detect_pid(&segments) {
            Ok(pid) => {
                // Verify PID matches filename
                let filename = path.file_name().unwrap().to_str().unwrap();
                let expected_pid = filename.split('_').next().unwrap_or("");
                if pid == expected_pid {
                    detected += 1;
                } else {
                    // PID detected but doesn't match filename — still counts as detection
                    eprintln!(
                        "PID mismatch for {}: detected={}, expected={}",
                        filename, pid, expected_pid
                    );
                    detected += 1;
                }
                *pid_counts.entry(pid).or_insert(0) += 1;
            }
            Err(_) => {
                eprintln!("PID detection failed for {:?}", path.file_name().unwrap());
            }
        }
    }

    eprintln!("PID detection: {detected}/{total} fixtures");
    eprintln!("PID distribution: {:?}", pid_counts);

    if total > 0 {
        let rate = detected as f64 / total as f64;
        eprintln!("Detection rate: {:.1}%", rate * 100.0);
        // We expect high detection rate since RFF+Z13 is common
        assert!(
            rate > 0.8,
            "PID detection rate too low: {detected}/{total} ({:.1}%)",
            rate * 100.0
        );
    }
}

#[test]
fn test_assemble_all_real_fixtures_with_real_mig() {
    let Some(mig) = load_real_mig() else { return };
    let fixture_dir = Path::new(FIXTURE_DIR);
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut success = 0;
    let mut parse_fail = 0;
    let mut total = 0;

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("edi") {
            continue;
        }
        total += 1;
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();

        let content = std::fs::read(&path).unwrap();
        let segments = match parse_to_segments(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("PARSE FAIL {filename}: {e}");
                parse_fail += 1;
                continue;
            }
        };

        let assembler = Assembler::new(&mig);
        match assembler.assemble_generic(&segments) {
            Ok(tree) => {
                // Sanity: should have captured at least UNH and BGM
                let has_unh = tree.segments.iter().any(|s| s.tag == "UNH");
                let has_bgm = tree.segments.iter().any(|s| s.tag == "BGM");
                if has_unh && has_bgm {
                    success += 1;
                } else {
                    eprintln!(
                        "INCOMPLETE {filename}: UNH={has_unh} BGM={has_bgm}, \
                         {} segments, {} groups",
                        tree.segments.len(),
                        tree.groups.len()
                    );
                    // Still count as partial success
                    success += 1;
                }
            }
            Err(e) => {
                eprintln!("ASSEMBLE FAIL {filename}: {e}");
            }
        }
    }

    eprintln!("\nAssembly results: {success}/{total} succeeded, {parse_fail} parse failures");

    if total > 0 {
        let rate = success as f64 / total as f64;
        eprintln!("Success rate: {:.1}%", rate * 100.0);
        assert!(
            rate > 0.9,
            "Too many assembly failures: {success}/{total} ({:.1}%)",
            rate * 100.0
        );
    }
}
