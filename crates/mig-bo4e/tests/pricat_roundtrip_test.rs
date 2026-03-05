//! PRICAT full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange(SG17)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

use std::path::Path;

// ── Generated fixture roundtrip tests ──

#[test]
fn test_pricat_pid_27001_generated_roundtrip() {
    let fixture =
        Path::new(common::pricat::FIXTURE_DIR).join("generated/27001_PRICAT_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::pricat::load_pid_filtered_mig("27001").unwrap();
    let (msg_engine, tx_engine) = common::pricat::load_split_engines("27001");

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "27001",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::pricat::TX_GROUP,
    );
}

#[test]
fn test_pricat_pid_27002_generated_roundtrip() {
    let fixture =
        Path::new(common::pricat::FIXTURE_DIR).join("generated/27002_PRICAT_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::pricat::load_pid_filtered_mig("27002").unwrap();
    let (msg_engine, tx_engine) = common::pricat::load_split_engines("27002");

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "27002",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::pricat::TX_GROUP,
    );
}

#[test]
fn test_pricat_pid_27003_generated_roundtrip() {
    let fixture =
        Path::new(common::pricat::FIXTURE_DIR).join("generated/27003_PRICAT_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::pricat::load_pid_filtered_mig("27003").unwrap();
    let (msg_engine, tx_engine) = common::pricat::load_split_engines("27003");

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "27003",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::pricat::TX_GROUP,
    );
}

// ── Real fixture roundtrip tests (KNOWN_INCOMPLETE due to DTM+NAD schema mismatch) ──

#[test]
fn test_pricat_pid_27002_roundtrip() {
    common::pricat::run_full_roundtrip_with_skip("27002", &["27002_PRICAT_2.0d_DEV-80749.edi"]);
}

#[test]
fn test_pricat_pid_27003_roundtrip() {
    common::pricat::run_full_roundtrip_with_skip(
        "27003",
        &[
            "27003_PRICAT_2.0d_DEV-80749.edi",
            "27003_PRICAT_2.0d_DEV-90070.edi",
        ],
    );
}

// ── TOML loading verification ──

/// Verify all PRICAT PID TOML mappings load successfully.
#[test]
fn test_pricat_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;

    let schema_dir = Path::new(common::pricat::SCHEMA_DIR);
    let msg_dir = common::pricat::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = ["27001", "27002", "27003"];

    for pid in pids {
        let tx_dir = common::pricat::pid_dir(pid);
        if !tx_dir.exists() {
            eprintln!("PID {pid}: no mapping directory — skipping");
            continue;
        }

        match MappingEngine::load(&tx_dir) {
            Ok(engine) => {
                let engine = engine.with_path_resolver(resolver.clone());
                let count = engine.definitions().len();
                eprintln!("PID {pid}: {count} definitions loaded OK");
            }
            Err(e) => panic!("PID {pid}: failed to load TOML mappings: {e}"),
        }
    }
}
