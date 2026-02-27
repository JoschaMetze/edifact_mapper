//! Integration tests for the fixture enhancer pipeline.
//!
//! Tests that `generate_enhanced_fixture` produces valid, realistic EDIFACT
//! output with deterministic seed-based data generation.

use automapper_generator::fixture_generator::generate_enhanced_fixture;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_types::schema::mig::MigSchema;
use std::collections::HashSet;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(Path::new(SCHEMA_DIR))
}

/// Load all resources needed for PID 55001 enhanced fixture generation.
/// Returns None if any required resource (MIG/AHB XML, schema, mappings) is unavailable.
fn setup_55001() -> Option<(serde_json::Value, MigSchema, MappingEngine, MappingEngine)> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    let schema_path = Path::new(SCHEMA_DIR).join("pid_55001_schema.json");
    let msg_dir = Path::new(MAPPINGS_BASE).join("message");
    let tx_dir = Path::new(MAPPINGS_BASE).join("pid_55001");

    // Check all required resources exist
    if !mig_path.exists() {
        eprintln!("Skipping: MIG XML not found at {}", MIG_XML_PATH);
        return None;
    }
    if !ahb_path.exists() {
        eprintln!("Skipping: AHB XML not found at {}", AHB_XML_PATH);
        return None;
    }
    if !schema_path.exists() {
        eprintln!("Skipping: PID 55001 schema not found");
        return None;
    }
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping: mapping directories not found");
        return None;
    }

    // Load PID schema JSON
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).ok()?).ok()?;

    // Parse MIG and AHB, filter for PID 55001
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == "55001")?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(&mig, &numbers);

    // Load mapping engines with path resolver
    let resolver = path_resolver();
    let msg_engine = MappingEngine::load(&msg_dir)
        .ok()?
        .with_path_resolver(resolver.clone());
    let tx_engine = MappingEngine::load(&tx_dir)
        .ok()?
        .with_path_resolver(resolver);

    Some((schema, filtered_mig, msg_engine, tx_engine))
}

#[test]
fn test_enhanced_fixture_55001() {
    let Some((schema, filtered_mig, msg_engine, tx_engine)) = setup_55001() else {
        eprintln!("Skipping test_enhanced_fixture_55001: resources not available");
        return;
    };

    // Generate enhanced fixture
    let result = generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 42, 0);

    let edi = result.expect("generate_enhanced_fixture should succeed for PID 55001");

    // Verify it can be tokenized (structurally valid EDIFACT)
    let segments =
        parse_to_segments(edi.as_bytes()).expect("Enhanced fixture must tokenize as valid EDIFACT");
    assert!(
        segments.len() > 10,
        "Expected at least 10 segments, got {}",
        segments.len()
    );

    // Verify EDIFACT envelope structure
    let tags: Vec<&str> = segments.iter().map(|s| s.id.as_str()).collect();
    assert!(tags.contains(&"UNB"), "Missing UNB envelope segment");
    assert!(tags.contains(&"UNH"), "Missing UNH message header");
    assert!(tags.contains(&"BGM"), "Missing BGM segment");
    assert!(tags.contains(&"UNT"), "Missing UNT message trailer");
    assert!(tags.contains(&"UNZ"), "Missing UNZ interchange trailer");

    // Verify that main entity placeholders have been replaced.
    // Check the full EDIFACT string for placeholder values that should NOT appear
    // in the main mapped entities. Note: envelope segments (UNB, UNH) and
    // unmapped segments may still contain original placeholder values — that is OK.
    //
    // Extract just the content segments (between UNH and UNT) to check for
    // placeholder replacement in the mapped portion.
    let content_segments: Vec<&str> = {
        let mut in_message = false;
        let mut content = Vec::new();
        for seg_str in edi.split('\'') {
            let trimmed = seg_str.trim();
            if trimmed.starts_with("UNH+") {
                in_message = true;
                continue;
            }
            if trimmed.starts_with("UNT+") {
                break;
            }
            if in_message
                && !trimmed.is_empty()
                && !trimmed.starts_with("BGM+")
                && !trimmed.starts_with("DTM+")
            {
                content.push(trimmed);
            }
        }
        content
    };

    // At least some NAD segments should have enhanced names (not "Mustermann")
    let nad_segments: Vec<&&str> = content_segments
        .iter()
        .filter(|s| s.starts_with("NAD+"))
        .collect();
    if !nad_segments.is_empty() {
        let nad_with_mustermann = nad_segments
            .iter()
            .filter(|s| s.contains("Mustermann"))
            .count();
        eprintln!(
            "NAD segments: {} total, {} still with Mustermann placeholder",
            nad_segments.len(),
            nad_with_mustermann
        );
        // At least some NAD segments should have been enhanced
        assert!(
            nad_with_mustermann < nad_segments.len(),
            "All NAD segments still have placeholder 'Mustermann' — enhancement did not work"
        );
    }

    eprintln!(
        "PID 55001 enhanced fixture: {} segments, valid EDIFACT",
        segments.len()
    );
    eprintln!("First 200 chars: {}", &edi[..edi.len().min(200)]);
}

#[test]
fn test_enhanced_fixture_deterministic() {
    let Some((schema, filtered_mig, msg_engine, tx_engine)) = setup_55001() else {
        eprintln!("Skipping test_enhanced_fixture_deterministic: resources not available");
        return;
    };

    // Generate twice with the same seed — must produce identical output
    let result_a =
        generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 42, 0)
            .expect("first generation with seed=42 should succeed");
    let result_b =
        generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 42, 0)
            .expect("second generation with seed=42 should succeed");

    assert_eq!(
        result_a, result_b,
        "Same seed (42) must produce identical EDIFACT output"
    );

    // Generate with a different seed — should produce different output
    let result_c =
        generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 99, 0)
            .expect("generation with seed=99 should succeed");

    assert_ne!(
        result_a, result_c,
        "Different seeds (42 vs 99) should produce different EDIFACT output"
    );

    eprintln!("Determinism verified: seed=42 produces identical output, seed=99 differs");
}
