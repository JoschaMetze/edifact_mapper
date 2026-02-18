use automapper_generator::codegen::coordinator_gen::generate_coordinator;
use automapper_generator::codegen::segment_order::extract_ordered_segments;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_generate_coordinator() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();

    assert!(output.contains("auto-generated"));
    assert!(output.contains("UtilmdStromCoordinatorFV2510"));
    assert!(output.contains("dispatch_segment"));
    assert!(output.contains("segment_write_order"));
}

#[test]
fn test_coordinator_segment_dispatch() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();

    // Should have dispatch entries for UNH, BGM, NAD
    assert!(output.contains("\"UNH\""));
    assert!(output.contains("\"BGM\""));
    assert!(output.contains("\"NAD\""));
}

#[test]
fn test_coordinator_write_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();

    // Write order should list segments with their counters
    assert!(output.contains("\"UNH\", // 0010"));
    assert!(output.contains("\"BGM\", // 0020"));
    assert!(output.contains("\"NAD\", // 0080"));
}

#[test]
fn test_coordinator_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();
    insta::assert_snapshot!("coordinator_fv2510", output);
}
