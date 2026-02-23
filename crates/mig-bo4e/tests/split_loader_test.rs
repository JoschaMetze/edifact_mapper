use mig_bo4e::engine::MappingEngine;
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
fn test_load_split_message_and_transaction() {
    let base = mappings_dir();
    let message_dir = base.join("message");
    let tx_dir = base.join("pid_55001");

    if !message_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let (msg_engine, tx_engine) = MappingEngine::load_split(&message_dir, &tx_dir).unwrap();

    // Message engine should have Marktteilnehmer, Nachricht, Kontakt
    let msg_defs = msg_engine.definitions();
    assert!(
        msg_defs.iter().any(|d| d.meta.entity == "Marktteilnehmer"),
        "Message engine should have Marktteilnehmer"
    );
    assert!(
        msg_defs.iter().any(|d| d.meta.entity == "Nachricht"),
        "Message engine should have Nachricht"
    );
    assert!(
        msg_defs.iter().any(|d| d.meta.entity == "Kontakt"),
        "Message engine should have Kontakt"
    );

    // Transaction engine should have Prozessdaten, Marktlokation, etc.
    let tx_defs = tx_engine.definitions();
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Prozessdaten"),
        "Transaction engine should have Prozessdaten"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Marktlokation"),
        "Transaction engine should have Marktlokation"
    );

    // Transaction engine should NOT have Marktteilnehmer (that's message-level)
    assert!(
        !tx_defs.iter().any(|d| d.meta.entity == "Marktteilnehmer"),
        "Transaction engine should not have Marktteilnehmer"
    );
}

#[test]
fn test_load_split_55002() {
    let base = mappings_dir();
    let message_dir = base.join("message");
    let tx_dir = base.join("pid_55002");

    if !message_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let (msg_engine, tx_engine) = MappingEngine::load_split(&message_dir, &tx_dir).unwrap();

    // Message engine is shared — same as 55001
    assert!(
        msg_engine
            .definitions()
            .iter()
            .any(|d| d.meta.entity == "Marktteilnehmer"),
        "Message engine should have Marktteilnehmer"
    );

    // Transaction engine should have 55002-specific entities
    assert!(
        tx_engine
            .definitions()
            .iter()
            .any(|d| d.meta.entity == "Messlokation"),
        "55002 transaction engine should have Messlokation"
    );
    assert!(
        tx_engine
            .definitions()
            .iter()
            .any(|d| d.meta.entity == "Netzlokation"),
        "55002 transaction engine should have Netzlokation"
    );
}
