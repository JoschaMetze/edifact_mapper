//! Verifies that generated code is syntactically valid Rust.
//! We don't compile it with cargo (that would require automapper-core to exist),
//! but we verify it passes basic structural checks.

use automapper_generator::codegen::coordinator_gen::generate_coordinator;
use automapper_generator::codegen::mapper_gen::generate_mapper_stubs;
use automapper_generator::codegen::segment_order::extract_ordered_segments;
use automapper_generator::codegen::version_config_gen::generate_version_config;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

/// Checks that a string is valid Rust syntax by verifying structural balance.
fn assert_valid_rust_syntax(code: &str, context: &str) {
    // Basic structural checks
    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "{}: mismatched braces (open={}, close={})",
        context, open_braces, close_braces
    );

    let open_parens = code.matches('(').count();
    let close_parens = code.matches(')').count();
    assert_eq!(
        open_parens, close_parens,
        "{}: mismatched parentheses",
        context
    );

    // Check for common Rust patterns
    assert!(
        code.contains("fn ") || code.contains("pub mod") || code.contains("struct "),
        "{}: should contain Rust definitions",
        context
    );

    // No unterminated strings (basic check)
    let quote_count = code.matches('"').count();
    assert_eq!(
        quote_count % 2,
        0,
        "{}: odd number of quotes ({})",
        context,
        quote_count
    );
}

#[test]
fn test_all_generated_code_is_valid_rust() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    // Check mapper stubs
    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();
    for (filename, content) in &stubs {
        assert_valid_rust_syntax(content, filename);
    }

    // Check version config
    let vc = generate_version_config(&mig).unwrap();
    assert_valid_rust_syntax(&vc, "version_config");

    // Check coordinator
    let coord = generate_coordinator(&mig, &ordered).unwrap();
    assert_valid_rust_syntax(&coord, "coordinator");
}

#[test]
fn test_generated_code_consistency() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Generate twice — output should be identical (deterministic generation)
    let stubs_1 = generate_mapper_stubs(&mig, &ahb).unwrap();
    let stubs_2 = generate_mapper_stubs(&mig, &ahb).unwrap();

    assert_eq!(stubs_1.len(), stubs_2.len());
    for (a, b) in stubs_1.iter().zip(stubs_2.iter()) {
        assert_eq!(a.0, b.0, "filenames should match");
        assert_eq!(a.1, b.1, "content should be identical across runs");
    }

    let vc1 = generate_version_config(&mig).unwrap();
    let vc2 = generate_version_config(&mig).unwrap();
    assert_eq!(vc1, vc2, "version config should be deterministic");
}
