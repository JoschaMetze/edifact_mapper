use mig_assembly::service::ConversionService;
use std::path::Path;

#[test]
fn test_conversion_service_from_mig_file() {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping");
        return;
    }

    let service = ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504");
    assert!(
        service.is_ok(),
        "Failed to create service: {:?}",
        service.err()
    );

    let service = service.unwrap();
    assert_eq!(service.mig().message_type, "UTILMD");
    assert_eq!(service.mig().format_version, "FV2504");
}

#[test]
fn test_conversion_service_mig_tree_mode() {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping");
        return;
    }

    let service = ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();

    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

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

    // Convert to MIG tree JSON
    let tree_json = service.convert_to_tree(&content);
    assert!(
        tree_json.is_ok(),
        "Tree conversion failed: {:?}",
        tree_json.err()
    );

    let json = tree_json.unwrap();
    assert!(
        json.is_object(),
        "Expected JSON object, got: {}",
        json.to_string().chars().take(200).collect::<String>()
    );

    // Should have segments and groups fields
    assert!(json.get("segments").is_some(), "Missing 'segments' key");
    assert!(json.get("groups").is_some(), "Missing 'groups' key");
}

#[test]
fn test_conversion_service_assembled_tree_mode() {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping");
        return;
    }

    let service = ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();

    // Minimal EDIFACT input
    let input = "UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001+9'UNT+3+MSG001'UNZ+1+REF001'";

    let tree = service.convert_to_assembled_tree(input);
    assert!(tree.is_ok(), "Assembly failed: {:?}", tree.err());

    let tree = tree.unwrap();
    assert!(!tree.segments.is_empty(), "Expected at least one segment");
}
