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

#[test]
fn test_load_split_55013() {
    let base = mappings_dir();
    let message_dir = base.join("message");
    let tx_dir = base.join("pid_55013");

    if !message_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let (msg_engine, tx_engine) = MappingEngine::load_split(&message_dir, &tx_dir).unwrap();

    // Message engine is shared
    assert!(
        msg_engine
            .definitions()
            .iter()
            .any(|d| d.meta.entity == "Marktteilnehmer"),
        "Message engine should have Marktteilnehmer"
    );

    // Transaction engine should have 55013-specific entities
    let tx_defs = tx_engine.definitions();

    // LOC entities
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Marktlokation"),
        "55013 should have Marktlokation"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Messlokation"),
        "55013 should have Messlokation"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Netzlokation"),
        "55013 should have Netzlokation"
    );
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.entity == "SteuerbareRessource"),
        "55013 should have SteuerbareRessource"
    );
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.entity == "TechnischeRessource"),
        "55013 should have TechnischeRessource"
    );

    // SG12 NAD entities
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.entity == "Marktlokationsanschrift"),
        "55013 should have Marktlokationsanschrift (NAD+Z63)"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "KundeDesLf"),
        "55013 should have KundeDesLf (NAD+Z65)"
    );
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.entity == "KorrespondenzanschriftKundeLf"),
        "55013 should have KorrespondenzanschriftKundeLf (NAD+Z66)"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "KundeDesNb"),
        "55013 should have KundeDesNb (NAD+Z67)"
    );
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.entity == "KorrespondenzanschriftKundeNb"),
        "55013 should have KorrespondenzanschriftKundeNb (NAD+Z68)"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Anschlussnehmer"),
        "55013 should have Anschlussnehmer (NAD+Z69)"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Hausverwalter"),
        "55013 should have Hausverwalter (NAD+Z70)"
    );

    // SG9 within SG8_Z98 should enrich Marktlokation
    let malo_defs: Vec<_> = tx_defs
        .iter()
        .filter(|d| d.meta.entity == "Marktlokation")
        .collect();
    assert!(
        malo_defs.len() >= 2,
        "55013 should have at least 2 Marktlokation definitions (SG5 + SG9), got {}",
        malo_defs.len()
    );
    assert!(
        malo_defs
            .iter()
            .any(|d| d.meta.source_group == "SG4.SG8:1.SG9"),
        "55013 should have SG9 definition for Marktlokation Jahresverbrauchsprognose"
    );

    // Verify total definition count (26 TOML files in pid_55013)
    assert_eq!(
        tx_defs.len(),
        26,
        "55013 transaction engine should have exactly 26 definitions"
    );
}
