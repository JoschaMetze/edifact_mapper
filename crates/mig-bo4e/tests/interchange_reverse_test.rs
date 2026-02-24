//! Tests for map_interchange_reverse() — two-pass reverse mapping mirroring forward.

use mig_bo4e::model::{MappedMessage, Transaktion};
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
fn test_map_interchange_reverse_single_transaction() {
    let msg_dir = mappings_dir().join("message");
    let tx_dir = mappings_dir().join("pid_55001");

    let msg_engine = MappingEngine::load(&msg_dir).unwrap();
    let tx_engine = MappingEngine::load(&tx_dir).unwrap();

    // Build a MappedMessage that mirrors the forward output
    let mapped = MappedMessage {
        stammdaten: serde_json::json!({
            "marktteilnehmer": [
                { "marktrolle": "MS", "rollencodenummer": "9900123456789" }
            ]
        }),
        transaktionen: vec![Transaktion {
            stammdaten: serde_json::json!({
                "marktlokation": { "marktlokationsId": "51238696781" }
            }),
            transaktionsdaten: serde_json::json!({
                "kategorie": "E01",
                "pruefidentifikator": "55001"
            }),
        }],
    };

    let tree = MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

    // Should have message-level groups (SG2) and transaction group (SG4)
    let sg2 = tree.groups.iter().find(|g| g.group_id == "SG2");
    assert!(
        sg2.is_some(),
        "Should have SG2 group from message stammdaten"
    );

    let sg4 = tree.groups.iter().find(|g| g.group_id == "SG4");
    assert!(sg4.is_some(), "Should have SG4 group from transactions");

    let sg4 = sg4.unwrap();
    assert_eq!(sg4.repetitions.len(), 1, "Should have 1 transaction");
}
