use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_parse_minimal_mig() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema =
        parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").expect("should parse minimal MIG XML");

    assert_eq!(schema.message_type, "UTILMD");
    assert_eq!(schema.variant, Some("Strom".to_string()));
    assert_eq!(schema.version, "S2.1");
    assert_eq!(schema.author, "BDEW");
    assert_eq!(schema.format_version, "FV2510");
}

#[test]
fn test_parse_mig_segments() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Should have 2 top-level segments: UNH, BGM
    assert_eq!(schema.segments.len(), 2);

    let unh = &schema.segments[0];
    assert_eq!(unh.id, "UNH");
    assert_eq!(unh.name, "Nachrichtenkopfsegment");
    assert_eq!(unh.counter, Some("0010".to_string()));
    assert_eq!(unh.status_std, Some("M".to_string()));
    assert_eq!(unh.example, Some("UNH+1+UTILMD:D:11A:UN:S2.1".to_string()));

    // UNH should have 1 data element (D_0062) and 1 composite (C_S009)
    assert_eq!(unh.data_elements.len(), 1);
    assert_eq!(unh.composites.len(), 1);

    let de_0062 = &unh.data_elements[0];
    assert_eq!(de_0062.id, "0062");
    assert_eq!(de_0062.format_std, Some("an..14".to_string()));
    assert_eq!(de_0062.position, 0);

    let s009 = &unh.composites[0];
    assert_eq!(s009.id, "S009");
    assert_eq!(s009.position, 1); // After the data element
    assert_eq!(s009.data_elements.len(), 2); // D_0065 and D_0052

    let de_0065 = &s009.data_elements[0];
    assert_eq!(de_0065.id, "0065");
    assert_eq!(de_0065.codes.len(), 1);
    assert_eq!(de_0065.codes[0].value, "UTILMD");
    assert_eq!(de_0065.codes[0].name, "Stammdaten");
}

#[test]
fn test_parse_mig_segment_groups() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Should have 1 segment group: SG2
    assert_eq!(schema.segment_groups.len(), 1);

    let sg2 = &schema.segment_groups[0];
    assert_eq!(sg2.id, "SG2");
    assert_eq!(sg2.counter, Some("0070".to_string()));
    assert_eq!(sg2.max_rep_spec, 99);
    assert_eq!(sg2.status_spec, Some("M".to_string()));

    // SG2 should contain NAD segment
    assert_eq!(sg2.segments.len(), 1);
    let nad = &sg2.segments[0];
    assert_eq!(nad.id, "NAD");

    // NAD should have 1 data element (D_3035) and 1 composite (C_C082)
    assert_eq!(nad.data_elements.len(), 1);
    assert_eq!(nad.composites.len(), 1);

    let d_3035 = &nad.data_elements[0];
    assert_eq!(d_3035.codes.len(), 2); // MS and MR
    assert_eq!(d_3035.codes[0].value, "MS");
    assert_eq!(d_3035.codes[1].value, "MR");

    let c082 = &nad.composites[0];
    assert_eq!(c082.data_elements.len(), 3); // 3039, 1131, 3055
}

#[test]
fn test_parse_mig_bgm_codes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let bgm = &schema.segments[1];
    assert_eq!(bgm.id, "BGM");
    assert_eq!(bgm.composites.len(), 1);

    let c002 = &bgm.composites[0];
    assert_eq!(c002.id, "C002");
    assert_eq!(c002.data_elements[0].codes[0].value, "E40");
}

#[test]
fn test_parse_mig_nonexistent_file() {
    let result = parse_mig(Path::new("/nonexistent/file.xml"), "UTILMD", None, "FV2510");
    assert!(result.is_err());
}
