use automapper_generator::codegen::version_config_gen::generate_version_config;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_generate_version_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let output = generate_version_config(&mig).unwrap();

    assert!(output.contains("auto-generated"), "should have header");
    assert!(
        output.contains("pub struct FV2510;"),
        "should have marker struct"
    );
    assert!(
        output.contains("impl VersionConfig for FV2510"),
        "should have impl block"
    );
    assert!(
        output.contains("const VERSION: FormatVersion = FormatVersion::FV2510"),
        "should set VERSION constant"
    );
}

#[test]
fn test_version_config_has_entity_types() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let output = generate_version_config(&mig).unwrap();

    // SG2 exists -> should have Geschaeftspartner
    assert!(
        output.contains("type GeschaeftspartnerMapper"),
        "should have Geschaeftspartner mapper type"
    );

    // Prozessdaten always exists
    assert!(
        output.contains("type ProzessdatenMapper"),
        "should have Prozessdaten mapper type"
    );
}

#[test]
fn test_version_config_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let output = generate_version_config(&mig).unwrap();
    insta::assert_snapshot!("version_config_fv2510", output);
}
