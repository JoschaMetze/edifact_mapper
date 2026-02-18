use automapper_generator::codegen::segment_order::extract_ordered_segments;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_extract_ordered_segments() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let ordered = extract_ordered_segments(&schema);

    // Should have 3 segments: UNH (0010), BGM (0020), NAD (0080, in SG2)
    assert_eq!(ordered.len(), 3);

    // Verify ordering by counter
    assert_eq!(ordered[0].segment_id, "UNH");
    assert_eq!(ordered[0].counter, "0010");
    assert!(!ordered[0].is_optional); // M = mandatory
    assert!(ordered[0].group_id.is_none()); // Top-level

    assert_eq!(ordered[1].segment_id, "BGM");
    assert_eq!(ordered[1].counter, "0020");

    assert_eq!(ordered[2].segment_id, "NAD");
    assert_eq!(ordered[2].counter, "0080");
    assert!(!ordered[2].is_optional); // M in spec
    assert_eq!(ordered[2].group_id, Some("SG2".to_string()));
    assert_eq!(ordered[2].group_max_rep, 99);
}

#[test]
fn test_unique_segment_ids() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let ordered = extract_ordered_segments(&schema);
    let unique_ids: Vec<&str> = ordered
        .iter()
        .map(|e| e.segment_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    assert!(unique_ids.contains(&"UNH"));
    assert!(unique_ids.contains(&"BGM"));
    assert!(unique_ids.contains(&"NAD"));
}
