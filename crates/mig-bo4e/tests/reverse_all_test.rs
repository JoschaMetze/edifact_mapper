//! Tests for map_all_reverse() — reversing all definitions in an engine.

use mig_bo4e::MappingEngine;
use std::path::PathBuf;

fn mappings_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mappings/FV2504/UTILMD_Strom")
}

#[test]
fn test_map_all_reverse_message_level() {
    let msg_dir = mappings_dir().join("message");
    let msg_engine = MappingEngine::load(&msg_dir).unwrap();

    // Construct a minimal message-level BO4E JSON (Marktteilnehmer in SG2)
    let bo4e = serde_json::json!({
        "Marktteilnehmer": [
            {
                "marktrolle": "MS",
                "rollencodenummer": "9900123456789"
            },
            {
                "marktrolle": "MR",
                "rollencodenummer": "9900987654321"
            }
        ]
    });

    let tree = msg_engine.map_all_reverse(&bo4e);

    // Should produce an AssembledTree with SG2 group containing 2 repetitions
    let sg2 = tree.groups.iter().find(|g| g.group_id == "SG2");
    assert!(sg2.is_some(), "Should have SG2 group");
    let sg2 = sg2.unwrap();
    assert_eq!(sg2.repetitions.len(), 2, "Should have 2 Marktteilnehmer reps");

    // First rep should have NAD segment with MS qualifier
    let rep0 = &sg2.repetitions[0];
    let nad = rep0.segments.iter().find(|s| s.tag == "NAD");
    assert!(nad.is_some(), "First SG2 rep should have NAD");
}

#[test]
fn test_map_all_reverse_transaction_level() {
    let tx_dir = mappings_dir().join("pid_55001");
    let tx_engine = MappingEngine::load(&tx_dir).unwrap();

    // Minimal transaction-level BO4E (Marktlokation in SG5)
    let bo4e = serde_json::json!({
        "Marktlokation": {
            "marktlokationsId": "51238696781"
        }
    });

    let tree = tx_engine.map_all_reverse(&bo4e);

    // Should produce groups including SG5
    let has_groups = !tree.groups.is_empty();
    assert!(has_groups, "Should produce at least one group from reverse mapping");
}
