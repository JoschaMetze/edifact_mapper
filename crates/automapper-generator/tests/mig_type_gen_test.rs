use automapper_generator::codegen::mig_type_gen;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

fn load_utilmd_mig() -> automapper_generator::schema::mig::MigSchema {
    parse_mig(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
    )
    .expect("Failed to parse UTILMD MIG XML")
}

#[test]
fn test_generate_enums_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let enums_source = mig_type_gen::generate_enums(&mig);

    // Should contain D3035Qualifier with MS, MR
    assert!(
        enums_source.contains("pub enum D3035Qualifier"),
        "Missing D3035 enum"
    );
    assert!(enums_source.contains("MS"), "Missing MS variant");
    assert!(enums_source.contains("MR"), "Missing MR variant");

    // Should derive standard traits
    assert!(enums_source
        .contains("#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]"));

    // Should have Display impl for roundtrip to string
    assert!(enums_source.contains("impl std::fmt::Display for D3035Qualifier"));

    // Should have FromStr impl for parsing
    assert!(enums_source.contains("impl std::str::FromStr for D3035Qualifier"));

    // Should compile as valid Rust (syntax check via string inspection)
    assert!(!enums_source.contains("TODO"));
}

#[test]
fn test_generate_composites_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let composites_source = mig_type_gen::generate_composites(&mig);

    // Should contain C082 (party identification)
    assert!(
        composites_source.contains("pub struct CompositeC082"),
        "Missing C082"
    );
    // Fields should use Option for conditional elements
    assert!(composites_source.contains("Option<String>"));
    // Fields with code lists should reference the enum type
    assert!(
        composites_source.contains("D3055Qualifier")
            || composites_source.contains("Option<D3055Qualifier>"),
        "Missing D3055Qualifier reference"
    );
    // Should derive Serialize, Deserialize
    assert!(
        composites_source.contains("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]")
    );
}

#[test]
fn test_generate_segments_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let segments_source = mig_type_gen::generate_segments(&mig);

    // Should contain NAD segment
    assert!(
        segments_source.contains("pub struct SegNad"),
        "Missing SegNad"
    );
    // Should contain UNH segment
    assert!(
        segments_source.contains("pub struct SegUnh"),
        "Missing SegUnh"
    );
    // Should contain BGM segment
    assert!(
        segments_source.contains("pub struct SegBgm"),
        "Missing SegBgm"
    );
    // Segments should reference composites
    assert!(
        segments_source.contains("CompositeC082")
            || segments_source.contains("Option<CompositeC082>"),
        "Missing CompositeC082 reference"
    );
    // Segments should have direct data element fields too
    assert!(segments_source.contains("d3035"), "Missing d3035 field");
}

#[test]
fn test_generate_groups_from_utilmd_mig() {
    let mig = load_utilmd_mig();

    let groups_source = mig_type_gen::generate_groups(&mig);

    // Should contain SG2 (party group)
    assert!(
        groups_source.contains("pub struct Sg2"),
        "Missing SG2 group"
    );
    // Groups should reference segments
    assert!(
        groups_source.contains("SegNad"),
        "Missing SegNad reference in groups"
    );
}

#[test]
fn test_generate_mig_types_writes_files() {
    let output_dir = tempfile::tempdir().unwrap();

    mig_type_gen::generate_mig_types(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
        output_dir.path(),
    )
    .unwrap();

    // Should create version/message module structure
    let base = output_dir.path().join("fv2504").join("utilmd");
    assert!(base.join("enums.rs").exists(), "Missing enums.rs");
    assert!(base.join("composites.rs").exists(), "Missing composites.rs");
    assert!(base.join("segments.rs").exists(), "Missing segments.rs");
    assert!(base.join("groups.rs").exists(), "Missing groups.rs");
    assert!(base.join("mod.rs").exists(), "Missing mod.rs");

    // mod.rs should re-export all modules
    let mod_content = std::fs::read_to_string(base.join("mod.rs")).unwrap();
    assert!(mod_content.contains("pub mod enums;"));
    assert!(mod_content.contains("pub mod composites;"));
    assert!(mod_content.contains("pub mod segments;"));
    assert!(mod_content.contains("pub mod groups;"));
}
