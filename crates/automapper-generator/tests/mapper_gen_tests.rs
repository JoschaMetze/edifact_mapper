use automapper_generator::codegen::mapper_gen::generate_mapper_stubs;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_generate_mapper_stubs_from_minimal() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Should generate at least:
    // - Prozessdaten mapper (top-level segments)
    // - Geschaeftspartner mapper (SG2)
    // - A mod.rs file
    assert!(
        stubs.len() >= 3,
        "expected at least 3 files, got {}",
        stubs.len()
    );

    // Verify filenames follow the pattern
    for (filename, _) in &stubs {
        assert!(
            filename.starts_with("utilmd_"),
            "filename should start with message type: {}",
            filename
        );
        assert!(
            filename.ends_with(".rs"),
            "filename should end with .rs: {}",
            filename
        );
    }
}

#[test]
fn test_mapper_stub_content() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Find the Geschaeftspartner mapper
    let gp_stub = stubs
        .iter()
        .find(|(name, _)| name.contains("geschaeftspartner"))
        .expect("should have Geschaeftspartner mapper");

    let content = &gp_stub.1;

    // Verify it contains expected elements
    assert!(
        content.contains("auto-generated"),
        "should have auto-generated header"
    );
    assert!(
        content.contains("SegmentHandler"),
        "should implement SegmentHandler"
    );
    assert!(
        content.contains("EntityWriter"),
        "should implement EntityWriter"
    );
    assert!(content.contains("Mapper"), "should implement Mapper");
    assert!(content.contains("Builder"), "should have a Builder");
    assert!(content.contains("NAD"), "should reference NAD segment");
    assert!(
        content.contains("FormatVersion::FV2510"),
        "should reference format version"
    );
}

#[test]
fn test_mapper_stub_snapshot() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Snapshot each generated file
    for (filename, content) in &stubs {
        let snapshot_name = filename.replace('.', "_");
        insta::assert_snapshot!(snapshot_name, content);
    }
}

#[test]
fn test_mod_file_generation() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Find the mod file
    let mod_file = stubs.iter().find(|(name, _)| name.contains("_mod.rs"));
    assert!(mod_file.is_some(), "should generate a mod.rs file");

    let content = &mod_file.unwrap().1;
    assert!(
        content.contains("pub mod"),
        "mod file should have pub mod declarations"
    );
}
