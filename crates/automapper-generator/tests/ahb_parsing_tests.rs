use automapper_generator::parsing::ahb_parser::parse_ahb;
use std::path::Path;

#[test]
fn test_parse_minimal_ahb() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema =
        parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").expect("should parse minimal AHB XML");

    assert_eq!(schema.message_type, "UTILMD");
    assert_eq!(schema.variant, Some("Strom".to_string()));
    assert_eq!(schema.version, "2.1");
    assert_eq!(schema.format_version, "FV2510");
}

#[test]
fn test_parse_ahb_workflows() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    assert_eq!(schema.workflows.len(), 2);

    let wf1 = &schema.workflows[0];
    assert_eq!(wf1.id, "55001");
    assert_eq!(wf1.beschreibung, "Lieferantenwechsel");
    assert_eq!(wf1.kommunikation_von, Some("NB an LF".to_string()));
}

#[test]
fn test_parse_ahb_fields() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf1 = &schema.workflows[0];

    // Should capture fields with AHB_Status
    assert!(!wf1.fields.is_empty());

    // Find the D_1001 field
    let d1001 = wf1.fields.iter().find(|f| f.segment_path.contains("1001"));
    assert!(d1001.is_some(), "should find D_1001 field");
    let d1001 = d1001.unwrap();
    assert_eq!(d1001.ahb_status, "X");
    assert_eq!(d1001.codes.len(), 1); // E40
    assert_eq!(d1001.codes[0].value, "E40");

    // Find the D_3035 field (in SG2/NAD)
    let d3035 = wf1.fields.iter().find(|f| f.segment_path.contains("3035"));
    assert!(d3035.is_some(), "should find D_3035 field");
    let d3035 = d3035.unwrap();
    assert_eq!(d3035.codes.len(), 2); // MS and MR

    // Find the D_3039 field (in SG2/NAD/C082)
    let d3039 = wf1.fields.iter().find(|f| f.segment_path.contains("3039"));
    assert!(d3039.is_some(), "should find D_3039 field");
}

#[test]
fn test_parse_ahb_conditional_group() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf1 = &schema.workflows[0];

    // SG8 has a conditional status "[166] AND [2351]" — should be captured as a field
    let sg8_field = wf1
        .fields
        .iter()
        .find(|f| f.segment_path.contains("SG8") && f.ahb_status.contains("[166]"));
    assert!(
        sg8_field.is_some(),
        "should capture SG8 group-level conditional status"
    );

    // D_1245 has "X [931]"
    let d1245 = wf1.fields.iter().find(|f| f.segment_path.contains("1245"));
    assert!(d1245.is_some(), "should find D_1245 field");
    assert_eq!(d1245.unwrap().ahb_status, "X [931]");
}

#[test]
fn test_parse_ahb_bedingungen() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    assert_eq!(schema.bedingungen.len(), 3);
    assert_eq!(schema.bedingungen[0].id, "1");
    assert_eq!(
        schema.bedingungen[0].description,
        "Wenn Aufteilung vorhanden"
    );
    assert_eq!(schema.bedingungen[1].id, "2");
    assert_eq!(schema.bedingungen[2].id, "931");
    assert_eq!(
        schema.bedingungen[2].description,
        "Wenn Zeitformat korrekt ist"
    );
}

#[test]
fn test_parse_ahb_second_workflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf2 = &schema.workflows[1];
    assert_eq!(wf2.id, "55002");
    assert_eq!(wf2.beschreibung, "Ein-/Auszug");
    assert_eq!(wf2.kommunikation_von, Some("LF an NB".to_string()));

    // Should have at least the D_1001 field with "Muss"
    let d1001 = wf2.fields.iter().find(|f| f.segment_path.contains("1001"));
    assert!(d1001.is_some());
    assert_eq!(d1001.unwrap().ahb_status, "Muss");
}

#[test]
fn test_parse_ahb_nonexistent_file() {
    let result = parse_ahb(Path::new("/nonexistent/ahb.xml"), "UTILMD", None, "FV2510");
    assert!(result.is_err());
}
