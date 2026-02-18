//! Integration tests that parse real MIG/AHB XML files from the submodule.
//! These tests are gated behind the submodule's presence since it
//! may not be available in all build environments.
use std::path::Path;

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;

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
                        panic!("Failed to parse MIG {}: {}", file_name, e);
                    }
                }
            }
        }
    }

    if parsed_count > 0 {
        eprintln!("Parsed {} MIG files successfully", parsed_count);
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
                        panic!("Failed to parse AHB {}: {}", file_name, e);
                    }
                }
            }
        }
    }

    if parsed_count > 0 {
        eprintln!("Parsed {} AHB files successfully", parsed_count);
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

    // Find a UTILMD MIG file and verify qualifier capture
    let mut found = false;
    for entry in std::fs::read_dir(&xml_dir).unwrap().flatten() {
        let fv_path = entry.path();
        if !fv_path.is_dir() {
            continue;
        }
        let fv_name = fv_path.file_name().unwrap().to_string_lossy().to_string();

        for file_entry in std::fs::read_dir(&fv_path).unwrap().flatten() {
            let file_name = file_entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("UTILMD_MIG_Strom") && file_name.ends_with(".xml") {
                let schema =
                    parse_mig(&file_entry.path(), "UTILMD", Some("Strom"), &fv_name).unwrap();

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
                    "Captured {} total code values from UTILMD MIG",
                    all_codes.len()
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
