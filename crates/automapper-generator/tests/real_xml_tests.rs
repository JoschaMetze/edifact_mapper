//! Integration tests that parse real MIG/AHB XML files from the submodule.
//! These tests are gated behind the submodule's presence since it
//! may not be available in all build environments.
//!
//! Only supported format versions (FV2504, FV2510) with standard release
//! XML files are expected to parse without error. Unsupported versions,
//! non-UTILMD message types, and special editions (außerordentliche releases,
//! Fehlerkorrektur patches) are logged and skipped.
use std::path::Path;

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;

/// Supported format versions for strict parsing.
const SUPPORTED_VERSIONS: &[&str] = &["FV2504", "FV2510"];

/// Message types whose XML schema the parser was built for.
const SUPPORTED_MESSAGE_TYPES: &[&str] = &["UTILMD"];

/// Filenames containing these substrings are special editions with
/// potentially different XML schemas (missing attributes, different structure).
const SPECIAL_EDITION_MARKERS: &[&str] = &["außerordentliche", "außerordendliche", "Fehlerkorrektur"];

/// Helper to locate the xml-migs-and-ahbs directory.
/// Returns None if the submodule is not initialized.
fn find_xml_submodule() -> Option<std::path::PathBuf> {
    // Walk up from the crate directory to find the workspace root
    let mut dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    for _ in 0..5 {
        let candidate = dir.join("xml-migs-and-ahbs");
        if candidate.is_dir() {
            // Check it's not empty (submodule is initialized)
            if std::fs::read_dir(&candidate).ok()?.next().is_some() {
                return Some(candidate);
            }
        }
        dir = dir.parent()?.to_path_buf();
    }
    None
}

#[test]
fn test_parse_real_mig_files() {
    let xml_dir = match find_xml_submodule() {
        Some(dir) => dir,
        None => {
            eprintln!("SKIPPED: xml-migs-and-ahbs submodule not found or empty");
            return;
        }
    };

    // Try to find MIG XML files in any format version directory
    let mut parsed_count = 0;
    let mut skipped_count = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&xml_dir).unwrap() {
        let entry = entry.unwrap();
        let fv_path = entry.path();
        if !fv_path.is_dir() {
            continue;
        }

        let fv_name = fv_path.file_name().unwrap().to_string_lossy().to_string();
        if !fv_name.starts_with("FV") {
            continue;
        }

        for file_entry in std::fs::read_dir(&fv_path).unwrap() {
            let file_entry = file_entry.unwrap();
            let file_name = file_entry.file_name().to_string_lossy().to_string();

            if file_name.contains("_MIG_") && file_name.ends_with(".xml") {
                let path = file_entry.path();
                // Infer message type from filename
                let msg_type = file_name.split('_').next().unwrap_or("UNKNOWN");
                let variant = if file_name.contains("Strom") {
                    Some("Strom")
                } else if file_name.contains("Gas") {
                    Some("Gas")
                } else {
                    None
                };

                let is_special = SPECIAL_EDITION_MARKERS
                    .iter()
                    .any(|marker| file_name.contains(marker));
                let is_supported = SUPPORTED_VERSIONS.contains(&fv_name.as_str())
                    && SUPPORTED_MESSAGE_TYPES.contains(&msg_type)
                    && !is_special;

                match parse_mig(&path, msg_type, variant, &fv_name) {
                    Ok(schema) => {
                        assert!(
                            !schema.version.is_empty(),
                            "MIG version should not be empty"
                        );
                        assert!(
                            !schema.segments.is_empty() || !schema.segment_groups.is_empty(),
                            "MIG should have segments or groups"
                        );
                        parsed_count += 1;
                        eprintln!(
                            "  OK: {} {} {} — {} segments, {} groups",
                            schema.message_type,
                            schema.variant.as_deref().unwrap_or(""),
                            fv_name,
                            schema.segments.len(),
                            schema.segment_groups.len()
                        );
                    }
                    Err(e) => {
                        if is_supported {
                            failures.push(format!("{}: {}", file_name, e));
                        } else {
                            skipped_count += 1;
                            eprintln!("  SKIP (unsupported): {}", file_name);
                        }
                    }
                }
            }
        }
    }

    eprintln!(
        "MIG: {} parsed, {} skipped (unsupported), {} failed",
        parsed_count,
        skipped_count,
        failures.len()
    );

    if !failures.is_empty() {
        panic!(
            "{} supported MIG files failed:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn test_parse_real_ahb_files() {
    let xml_dir = match find_xml_submodule() {
        Some(dir) => dir,
        None => {
            eprintln!("SKIPPED: xml-migs-and-ahbs submodule not found or empty");
            return;
        }
    };

    let mut parsed_count = 0;
    let mut skipped_count = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&xml_dir).unwrap() {
        let entry = entry.unwrap();
        let fv_path = entry.path();
        if !fv_path.is_dir() {
            continue;
        }

        let fv_name = fv_path.file_name().unwrap().to_string_lossy().to_string();
        if !fv_name.starts_with("FV") {
            continue;
        }

        for file_entry in std::fs::read_dir(&fv_path).unwrap() {
            let file_entry = file_entry.unwrap();
            let file_name = file_entry.file_name().to_string_lossy().to_string();

            if file_name.contains("_AHB_") && file_name.ends_with(".xml") {
                let path = file_entry.path();
                let msg_type = file_name.split('_').next().unwrap_or("UNKNOWN");
                let variant = if file_name.contains("Strom") {
                    Some("Strom")
                } else if file_name.contains("Gas") {
                    Some("Gas")
                } else {
                    None
                };

                let is_special = SPECIAL_EDITION_MARKERS
                    .iter()
                    .any(|marker| file_name.contains(marker));
                let is_supported = SUPPORTED_VERSIONS.contains(&fv_name.as_str())
                    && SUPPORTED_MESSAGE_TYPES.contains(&msg_type)
                    && !is_special;

                match parse_ahb(&path, msg_type, variant, &fv_name) {
                    Ok(schema) => {
                        assert!(
                            !schema.version.is_empty(),
                            "AHB version should not be empty"
                        );
                        parsed_count += 1;
                        eprintln!(
                            "  OK: {} {} {} — {} workflows, {} conditions",
                            schema.message_type,
                            schema.variant.as_deref().unwrap_or(""),
                            fv_name,
                            schema.workflows.len(),
                            schema.bedingungen.len()
                        );
                    }
                    Err(e) => {
                        if is_supported {
                            failures.push(format!("{}: {}", file_name, e));
                        } else {
                            skipped_count += 1;
                            eprintln!("  SKIP (unsupported): {}", file_name);
                        }
                    }
                }
            }
        }
    }

    eprintln!(
        "AHB: {} parsed, {} skipped (unsupported), {} failed",
        parsed_count,
        skipped_count,
        failures.len()
    );

    if !failures.is_empty() {
        panic!(
            "{} supported AHB files failed:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn test_mig_captures_all_qualifiers() {
    let xml_dir = match find_xml_submodule() {
        Some(dir) => dir,
        None => {
            eprintln!("SKIPPED: xml-migs-and-ahbs submodule not found or empty");
            return;
        }
    };

    // Find a UTILMD MIG file from a supported version and verify qualifier capture
    let mut found = false;
    for fv in SUPPORTED_VERSIONS {
        let fv_path = xml_dir.join(fv);
        if !fv_path.exists() {
            continue;
        }

        for file_entry in std::fs::read_dir(&fv_path).unwrap().flatten() {
            let file_name = file_entry.file_name().to_string_lossy().to_string();
            // Skip special editions (außerordentliche/Fehlerkorrektur)
            let is_special = SPECIAL_EDITION_MARKERS
                .iter()
                .any(|marker| file_name.contains(marker));
            if file_name.starts_with("UTILMD_MIG_Strom") && file_name.ends_with(".xml") && !is_special {
                let schema =
                    parse_mig(&file_entry.path(), "UTILMD", Some("Strom"), fv).unwrap();

                // Collect all code values across all segments (recursively)
                let mut all_codes = Vec::new();
                collect_codes_from_segments(&schema.segments, &mut all_codes);
                for group in &schema.segment_groups {
                    collect_codes_from_group(group, &mut all_codes);
                }

                assert!(
                    !all_codes.is_empty(),
                    "should capture code values from UTILMD MIG"
                );
                eprintln!(
                    "Captured {} total code values from UTILMD MIG ({})",
                    all_codes.len(),
                    fv
                );
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
}

fn collect_codes_from_segments(
    segments: &[automapper_generator::schema::mig::MigSegment],
    codes: &mut Vec<String>,
) {
    for seg in segments {
        for de in &seg.data_elements {
            for code in &de.codes {
                codes.push(format!("{}/{}: {}", seg.id, de.id, code.value));
            }
        }
        for comp in &seg.composites {
            for de in &comp.data_elements {
                for code in &de.codes {
                    codes.push(format!("{}/{}/{}: {}", seg.id, comp.id, de.id, code.value));
                }
            }
        }
    }
}

fn collect_codes_from_group(
    group: &automapper_generator::schema::mig::MigSegmentGroup,
    codes: &mut Vec<String>,
) {
    collect_codes_from_segments(&group.segments, codes);
    for nested in &group.nested_groups {
        collect_codes_from_group(nested, codes);
    }
}
