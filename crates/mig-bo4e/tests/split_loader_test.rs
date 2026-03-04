use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use std::path::PathBuf;

const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(std::path::Path::new(SCHEMA_DIR))
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
fn test_load_split_message_and_transaction() {
    let base = mappings_dir();
    let message_dir = base.join("message");
    let tx_dir = base.join("pid_55001");

    if !message_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let (msg_engine, tx_engine) = MappingEngine::load_split(&message_dir, &tx_dir).unwrap();
    let resolver = path_resolver();
    let msg_engine = msg_engine.with_path_resolver(resolver.clone());
    let tx_engine = tx_engine.with_path_resolver(resolver);

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
    let resolver = path_resolver();
    let msg_engine = msg_engine.with_path_resolver(resolver.clone());
    let tx_engine = tx_engine.with_path_resolver(resolver);

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
    let common_dir = base.join("common");
    let tx_dir = base.join("pid_55013");

    if !message_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let resolver = path_resolver();
    let (msg_engine, tx_engine) = if common_dir.exists() {
        let schema_path = std::path::Path::new(SCHEMA_DIR).join("pid_55013_schema.json");
        let schema_index = PidSchemaIndex::from_schema_file(&schema_path).unwrap();
        let (m, t) = MappingEngine::load_split_with_common(
            &message_dir,
            &common_dir,
            &tx_dir,
            &schema_index,
        )
        .unwrap();
        (
            m.with_path_resolver(resolver.clone()),
            t.with_path_resolver(resolver),
        )
    } else {
        let (m, t) = MappingEngine::load_split(&message_dir, &tx_dir).unwrap();
        (
            m.with_path_resolver(resolver.clone()),
            t.with_path_resolver(resolver),
        )
    };

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

    // SG12 NAD: unified Geschaeftspartner entity (all NAD qualifiers Z63-Z70)
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Geschaeftspartner"),
        "55013 should have Geschaeftspartner (unified NAD SG12)"
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
            .any(|d| d.meta.source_group == "SG4.SG8.SG9"),
        "55013 should have SG9 definition for Marktlokation Jahresverbrauchsprognose (source_path-based resolution)"
    );

    // Verify total definition count: 20 PID files + 2 common RFF (z13, tn)
    // Common SG5 files are overridden by PID's own z16-z20 definitions
    assert_eq!(
        tx_defs.len(),
        22,
        "55013 transaction engine should have exactly 22 definitions (20 PID + 2 common RFF)"
    );
}

#[test]
fn test_load_with_common_55002() {
    let base = mappings_dir();
    let common_dir = base.join("common");
    let tx_dir = base.join("pid_55002");

    if !common_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let schema_path = std::path::Path::new(SCHEMA_DIR).join("pid_55002_schema.json");
    let schema_index = PidSchemaIndex::from_schema_file(&schema_path).unwrap();

    let tx_engine = MappingEngine::load_with_common(&common_dir, &tx_dir, &schema_index)
        .unwrap()
        .with_path_resolver(path_resolver());

    // Should have LOC entities from common/ templates
    let tx_defs = tx_engine.definitions();
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Marktlokation"),
        "Should have Marktlokation from common or PID"
    );
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Messlokation"),
        "Should have Messlokation from common or PID"
    );

    // Should NOT have LOC entities for groups not in 55002 schema
    // (55002 has z16-z20, z22 but schema-aware filter handles this)

    // Should have RFF+Z13 Prozessdaten (discriminator resolved to numeric by PathResolver)
    assert!(
        tx_defs.iter().any(|d| d.meta.entity == "Prozessdaten"
            && d.meta
                .discriminator
                .as_deref()
                .map(|disc| disc.contains("Z13"))
                .unwrap_or(false)),
        "Should have RFF+Z13 Prozessdaten"
    );
}

#[test]
fn test_load_with_common_schema_filter() {
    let base = mappings_dir();
    let common_dir = base.join("common");
    let tx_dir = base.join("pid_55001");

    if !common_dir.exists() || !tx_dir.exists() {
        eprintln!("mappings/ dirs not found, skipping");
        return;
    }

    let schema_path = std::path::Path::new(SCHEMA_DIR).join("pid_55001_schema.json");
    let schema_index = PidSchemaIndex::from_schema_file(&schema_path).unwrap();

    let tx_engine = MappingEngine::load_with_common(&common_dir, &tx_dir, &schema_index)
        .unwrap()
        .with_path_resolver(path_resolver());

    let tx_defs = tx_engine.definitions();

    // 55001 has sg5_z16 and sg5_z22 but NOT sg5_z17/z18/z19/z20/z15
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.source_path.as_deref() == Some("sg4.sg5_z16")),
        "Should have sg5_z16 (exists in 55001 schema)"
    );
    assert!(
        tx_defs
            .iter()
            .any(|d| d.meta.source_path.as_deref() == Some("sg4.sg5_z22")),
        "Should have sg5_z22 (exists in 55001 schema)"
    );
    assert!(
        !tx_defs
            .iter()
            .any(|d| d.meta.source_path.as_deref() == Some("sg4.sg5_z17")),
        "Should NOT have sg5_z17 (not in 55001 schema)"
    );
    assert!(
        !tx_defs
            .iter()
            .any(|d| d.meta.source_path.as_deref() == Some("sg4.sg5_z18")),
        "Should NOT have sg5_z18 (not in 55001 schema)"
    );
}
