//! Migration comparison tests: old (automapper-core) vs new (mig-assembly + mig-bo4e) pipelines.
//!
//! Runs both pipelines on fixture files and compares their ability to process
//! the same EDIFACT input. Since the MIG-driven pipeline returns a different
//! structure (AssembledTree) than the legacy pipeline (UtilmdTransaktion), we
//! compare structural properties rather than exact equality.

use std::path::Path;

use automapper_core::{create_coordinator, FormatVersion};
use mig_assembly::service::ConversionService;

/// Run the legacy pipeline on EDIFACT input, returning serialized JSON or error.
fn run_legacy_pipeline(input: &str) -> Result<serde_json::Value, String> {
    let mut coordinator = create_coordinator(FormatVersion::FV2504).map_err(|e| e.to_string())?;
    let transactions = coordinator
        .parse(input.as_bytes())
        .map_err(|e| e.to_string())?;
    serde_json::to_value(&transactions).map_err(|e| e.to_string())
}

/// Run the new MIG-driven pipeline on EDIFACT input, returning serialized JSON or error.
fn run_new_pipeline(input: &str, service: &ConversionService) -> Result<serde_json::Value, String> {
    service.convert_to_tree(input).map_err(|e| e.to_string())
}

#[test]
fn test_old_vs_new_pipeline_both_process_fixtures() {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping comparison test");
        return;
    }

    let service = ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504")
        .expect("Failed to create ConversionService");

    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let mut old_success = 0u32;
    let mut new_success = 0u32;
    let mut both_success = 0u32;
    let mut total = 0u32;

    for entry in std::fs::read_dir(fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.extension().map(|e| e == "edi").unwrap_or(false) {
            continue;
        }
        total += 1;

        let content = std::fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy();

        let old_result = run_legacy_pipeline(&content);
        let new_result = run_new_pipeline(&content, &service);

        let old_ok = old_result.is_ok();
        let new_ok = new_result.is_ok();

        if old_ok {
            old_success += 1;
        }
        if new_ok {
            new_success += 1;
        }
        if old_ok && new_ok {
            both_success += 1;
        }

        if !old_ok {
            eprintln!("  OLD FAIL  {name}: {}", old_result.unwrap_err());
        }
        if !new_ok {
            eprintln!("  NEW FAIL  {name}: {}", new_result.unwrap_err());
        }
    }

    eprintln!("\n=== Migration Comparison Results ===");
    eprintln!("Total fixtures:      {total}");
    eprintln!("Legacy success:      {old_success}/{total}");
    eprintln!("MIG-driven success:  {new_success}/{total}");
    eprintln!("Both succeed:        {both_success}/{total}");

    // The new pipeline should successfully parse at least some files
    assert!(
        new_success > 0,
        "New pipeline should succeed on at least one fixture"
    );
    assert!(total > 0, "Should have found at least one fixture file");
}

#[test]
fn test_new_pipeline_tree_has_expected_structure() {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping");
        return;
    }

    let service = ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504")
        .expect("Failed to create ConversionService");

    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    // Take first .edi fixture file
    let first_file = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "edi").unwrap_or(false));

    let first_file = match first_file {
        Some(f) => f,
        None => {
            eprintln!("No .edi fixture files found, skipping");
            return;
        }
    };

    let content = std::fs::read_to_string(first_file.path()).unwrap();
    let tree = service
        .convert_to_assembled_tree(&content)
        .expect("Assembly should succeed");

    // Structural checks on the assembled tree
    assert!(
        !tree.segments.is_empty(),
        "Assembled tree should have segments"
    );

    // Should have at least UNB and UNH in top-level segments
    let segment_tags: Vec<&str> = tree.segments.iter().map(|s| s.tag.as_str()).collect();
    eprintln!("Top-level segments: {:?}", segment_tags);

    assert!(
        segment_tags.contains(&"UNB") || segment_tags.contains(&"UNH"),
        "Should have UNB or UNH in top-level segments"
    );
}

#[test]
fn test_both_pipelines_handle_minimal_edifact() {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping");
        return;
    }

    let service = ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504")
        .expect("Failed to create ConversionService");

    let minimal_input = "UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001+9'UNT+3+MSG001'UNZ+1+REF001'";

    // New pipeline should handle minimal input
    let new_result = run_new_pipeline(minimal_input, &service);
    assert!(
        new_result.is_ok(),
        "New pipeline should handle minimal EDIFACT: {:?}",
        new_result.err()
    );

    // Legacy pipeline may or may not handle minimal input (it's more strict)
    let old_result = run_legacy_pipeline(minimal_input);
    eprintln!(
        "Legacy pipeline on minimal input: {}",
        if old_result.is_ok() {
            "OK"
        } else {
            "FAIL (expected for minimal input)"
        }
    );
}
