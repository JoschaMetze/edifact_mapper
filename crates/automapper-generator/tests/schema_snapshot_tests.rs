use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_mig_schema_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    insta::assert_yaml_snapshot!("mig_schema", schema, {
        ".source_file" => "[path]",
    });
}

#[test]
fn test_ahb_schema_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    insta::assert_yaml_snapshot!("ahb_schema", schema, {
        ".source_file" => "[path]",
    });
}

#[test]
fn test_mig_segment_details_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let unh = &schema.segments[0];
    insta::assert_yaml_snapshot!("mig_unh_segment", unh);
}

#[test]
fn test_ahb_workflow_details_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf1 = &schema.workflows[0];
    insta::assert_yaml_snapshot!("ahb_workflow_55001", wf1);
}
