//! End-to-end test: EDIFACT fixture → split → assemble → map → Interchange hierarchy.

use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("example_market_communication_bo4e_transactions")
        .join(name)
}

fn mappings_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mappings/FV2504/UTILMD_Strom")
}

#[test]
fn test_interchange_hierarchy_from_55001_fixture() {
    // Load a real UTILMD fixture
    let fixture = fixture_path("UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi");
    if !fixture.exists() {
        eprintln!("Skipping: fixture not found at {}", fixture.display());
        return;
    }
    let input = std::fs::read_to_string(&fixture).unwrap();

    // Tokenize and split
    let segments = mig_assembly::tokenize::parse_to_segments(input.as_bytes()).unwrap();
    let chunks = mig_assembly::split_messages(segments).unwrap();

    // Should be single message
    assert_eq!(chunks.messages.len(), 1, "Expected 1 message in fixture");

    // Verify UNH fields
    let msg = &chunks.messages[0];
    let (unh_ref, msg_type) = mig_bo4e::model::extract_unh_fields(&msg.unh);
    assert!(!unh_ref.is_empty(), "UNH reference should not be empty");
    assert_eq!(msg_type, "UTILMD");

    // Verify nachrichtendaten extraction
    let nd = mig_bo4e::model::extract_nachrichtendaten(&chunks.envelope);
    assert!(
        nd.get("absenderCode").is_some(),
        "Should extract sender from UNB"
    );
    assert!(
        nd.get("empfaengerCode").is_some(),
        "Should extract recipient from UNB"
    );
    assert!(
        nd.get("interchangeRef").is_some(),
        "Should extract interchange reference from UNB"
    );

    // Test split engine loading and mapping (if mapping dirs exist)
    let msg_dir = mappings_dir().join("message");
    let tx_dir = mappings_dir().join("pid_55001");
    if msg_dir.is_dir() && tx_dir.is_dir() {
        let (msg_engine, tx_engine) =
            mig_bo4e::MappingEngine::load_split(&msg_dir, &tx_dir).unwrap();

        // Load MIG XML and assemble the message for full pipeline test
        let mig_xml = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml");

        if mig_xml.exists() {
            let service =
                mig_assembly::ConversionService::new(&mig_xml, "UTILMD", Some("Strom"), "FV2504")
                    .unwrap();

            // Assemble the message
            let all_segments = msg.all_segments();
            let tree = mig_assembly::assembler::Assembler::new(service.mig())
                .assemble_generic(&all_segments)
                .unwrap();

            // Map with split engines
            let mapped =
                mig_bo4e::MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", true);

            // Verify message-level stammdaten
            assert!(
                mapped.stammdaten.is_object(),
                "Message stammdaten should be an object"
            );

            // Verify at least one transaction
            assert!(
                !mapped.transaktionen.is_empty(),
                "Should have at least one transaction"
            );

            // Build the full Interchange
            let interchange = mig_bo4e::Interchange {
                nachrichtendaten: nd.clone(),
                nachrichten: vec![mig_bo4e::Nachricht {
                    unh_referenz: unh_ref,
                    nachrichten_typ: msg_type,
                    stammdaten: mapped.stammdaten,
                    transaktionen: mapped.transaktionen,
                }],
            };

            // Verify it serializes to valid JSON
            let json = serde_json::to_string_pretty(&interchange).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(parsed["nachrichten"].is_array());
            assert_eq!(parsed["nachrichten"].as_array().unwrap().len(), 1);
            assert_eq!(
                parsed["nachrichten"][0]["nachrichtenTyp"].as_str().unwrap(),
                "UTILMD"
            );
        }
    }
}
