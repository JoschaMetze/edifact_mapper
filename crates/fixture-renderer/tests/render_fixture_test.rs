//! Integration tests for the fixture renderer pipeline.

use std::path::Path;

use fixture_renderer::{generate_canonical_bo4e, render_fixture, RenderInput};

fn base_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn make_input_55001() -> Option<RenderInput> {
    let base = base_dir();

    let mig_path =
        base.join("xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml");
    let ahb_path =
        base.join("xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml");
    let msg_mappings = base.join("mappings/FV2504/UTILMD_Strom/message");
    let tx_mappings = base.join("mappings/FV2504/UTILMD_Strom/pid_55001");

    if !mig_path.exists() || !ahb_path.exists() || !msg_mappings.exists() || !tx_mappings.exists()
    {
        return None;
    }

    Some(RenderInput {
        mig_xml_path: mig_path,
        ahb_xml_path: ahb_path,
        message_mappings_dir: msg_mappings,
        transaction_mappings_dir: tx_mappings,
        message_type: "UTILMD".into(),
        variant: Some("Strom".into()),
        format_version: "FV2504".into(),
        pid: "55001".into(),
    })
}

fn fixture_path_55001() -> Option<std::path::PathBuf> {
    let p = base_dir().join(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

#[test]
fn test_render_fixture_produces_valid_edifact() {
    let input = match make_input_55001() {
        Some(i) => i,
        None => {
            eprintln!("Skipping: required XML/mapping files not found");
            return;
        }
    };
    let fixture = match fixture_path_55001() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture not found");
            return;
        }
    };

    let result = render_fixture(&fixture, &input);
    assert!(
        result.is_ok(),
        "render_fixture should succeed: {:?}",
        result.err()
    );

    let edifact = result.unwrap();
    assert!(edifact.contains("UNA"), "Should have UNA service string");
    assert!(edifact.contains("UNB+"), "Should have UNB segment");
    assert!(edifact.contains("UNH+"), "Should have UNH segment");
    assert!(edifact.contains("UNT+"), "Should have UNT segment");
    assert!(edifact.contains("UNZ+"), "Should have UNZ segment");

    // Should contain UTILMD message type
    assert!(
        edifact.contains("UTILMD"),
        "Should contain UTILMD message type"
    );

    eprintln!(
        "Rendered EDIFACT ({} bytes):\n{}",
        edifact.len(),
        &edifact[..edifact.len().min(500)]
    );
}

#[test]
fn test_generate_canonical_bo4e_from_fixture() {
    let input = match make_input_55001() {
        Some(i) => i,
        None => {
            eprintln!("Skipping: required XML/mapping files not found");
            return;
        }
    };
    let fixture = match fixture_path_55001() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture not found");
            return;
        }
    };

    let result = generate_canonical_bo4e(&fixture, &input);
    assert!(
        result.is_ok(),
        "generate_canonical_bo4e should succeed: {:?}",
        result.err()
    );

    let canonical = result.unwrap();
    assert_eq!(canonical.meta.pid, "55001");
    assert_eq!(canonical.meta.message_type, "UTILMD");
    assert_eq!(canonical.meta.source_format_version, "FV2504");
    assert!(!canonical.nachricht.unh_referenz.is_empty());
    assert!(!canonical.nachricht.transaktionen.is_empty());

    // Verify it can roundtrip through JSON
    let json = serde_json::to_string_pretty(&canonical).unwrap();
    let parsed: automapper_generator::fixture_renderer::CanonicalBo4e =
        serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.meta.pid, "55001");

    eprintln!("Canonical BO4E JSON ({} bytes)", json.len());
    eprintln!("{}", &json[..json.len().min(500)]);
}

#[test]
fn test_roundtrip_generate_then_render() {
    let input = match make_input_55001() {
        Some(i) => i,
        None => {
            eprintln!("Skipping: required XML/mapping files not found");
            return;
        }
    };
    let fixture = match fixture_path_55001() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture not found");
            return;
        }
    };

    // Step 1: Generate canonical BO4E
    let canonical = generate_canonical_bo4e(&fixture, &input).unwrap();
    assert_eq!(canonical.meta.pid, "55001");

    // Step 2: Render back to EDIFACT
    let edifact = render_fixture(&fixture, &input).unwrap();
    assert!(edifact.contains("UNB+"));
    assert!(edifact.contains("UNH+"));
    assert!(edifact.contains("UTILMD"));
    assert!(edifact.contains("UNT+"));
    assert!(edifact.contains("UNZ+"));

    eprintln!("Roundtrip complete: canonical BO4E -> EDIFACT ({} bytes)", edifact.len());
}
