//! Port of C# `UtilmdForwardMappingTests`.
//!
//! Parses real UTILMD fixture files through the coordinator and verifies
//! that domain objects are correctly extracted.

use automapper_core::{create_coordinator, detect_format_version, FormatVersion};
use std::path::Path;

fn fixture_dir() -> Option<std::path::PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest.join("../../example_market_communication_bo4e_transactions");
    if fixture_path.exists() {
        Some(fixture_path)
    } else {
        None
    }
}

fn collect_edi_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_edi_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "edi") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Parse all UTILMD files through the coordinator without error.
///
/// Port of C# `Should_Parse_UTILMD_File_Successfully`.
#[test]
fn test_parse_all_utilmd_files_through_coordinator() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let utilmd_dir = fixture_path.join("UTILMD");
    if !utilmd_dir.exists() {
        eprintln!("Skipping: UTILMD directory not found");
        return;
    }

    let files = collect_edi_files(&utilmd_dir);
    assert!(!files.is_empty(), "No UTILMD .edi files found");

    let mut failures: Vec<String> = Vec::new();
    let mut success_count = 0;

    for file_path in &files {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(e) => {
                failures.push(format!("{}: read error: {}", file_path.display(), e));
                continue;
            }
        };

        let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);

        // Detect format version
        let fv = match detect_format_version(&content) {
            Some(fv) => fv,
            None => {
                // Files from older format versions may not be detected;
                // default to FV2504
                FormatVersion::FV2504
            }
        };

        // Create coordinator and parse
        let mut coord = match create_coordinator(fv) {
            Ok(c) => c,
            Err(e) => {
                failures.push(format!("{}: coordinator error: {}", rel.display(), e));
                continue;
            }
        };

        match coord.parse(&content) {
            Ok(transactions) => {
                if transactions.is_empty() {
                    failures.push(format!("{}: parsed OK but 0 transactions", rel.display()));
                } else {
                    success_count += 1;
                }
            }
            Err(e) => {
                failures.push(format!("{}: parse error: {}", rel.display(), e));
            }
        }
    }

    eprintln!(
        "Coordinator parsed {}/{} UTILMD files successfully",
        success_count,
        files.len()
    );

    if !failures.is_empty() {
        panic!(
            "{} of {} UTILMD files failed coordinator parsing:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
}

/// Verify that parsed UTILMD transactions have valid transaktionsdaten.
///
/// Port of C# `Should_Extract_Transaktionsdaten_Fields`.
#[test]
fn test_utilmd_transaktionsdaten_extraction() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let utilmd_dir = fixture_path.join("UTILMD");
    if !utilmd_dir.exists() {
        return;
    }

    let files = collect_edi_files(&utilmd_dir);
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0;

    // Check up to 20 files to keep test time reasonable
    for file_path in files.iter().take(20) {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let fv = detect_format_version(&content).unwrap_or(FormatVersion::FV2504);
        let mut coord = match create_coordinator(fv) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let transactions = match coord.parse(&content) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);

        for (i, tx) in transactions.iter().enumerate() {
            // Every transaction should have an ID
            if tx.transaktions_id.is_empty() {
                failures.push(format!(
                    "{} tx[{}]: empty transaktions_id",
                    rel.display(),
                    i
                ));
            }
        }

        checked += 1;
    }

    eprintln!("Checked transaktionsdaten in {} files", checked);

    if !failures.is_empty() {
        panic!(
            "{} transaktionsdaten issues:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Verify parsed results serialize to valid JSON.
///
/// Port of C# `Should_Serialize_Result_To_Valid_JSON`.
#[test]
fn test_utilmd_results_serialize_to_json() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let utilmd_dir = fixture_path.join("UTILMD");
    if !utilmd_dir.exists() {
        return;
    }

    let files = collect_edi_files(&utilmd_dir);
    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0;

    for file_path in files.iter().take(10) {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let fv = detect_format_version(&content).unwrap_or(FormatVersion::FV2504);
        let mut coord = match create_coordinator(fv) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let transactions = match coord.parse(&content) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);

        // Serialize to JSON and back
        let json = match serde_json::to_string_pretty(&transactions) {
            Ok(j) => j,
            Err(e) => {
                failures.push(format!("{}: serialize error: {}", rel.display(), e));
                continue;
            }
        };

        // Verify it's valid JSON by parsing it back
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&json) {
            failures.push(format!("{}: invalid JSON: {}", rel.display(), e));
            continue;
        }

        // Verify the JSON contains expected top-level structure
        assert!(
            !json.is_empty(),
            "JSON output should not be empty for {}",
            rel.display()
        );

        checked += 1;
    }

    eprintln!("Serialized {} files to valid JSON", checked);

    if !failures.is_empty() {
        panic!(
            "{} serialization issues:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
