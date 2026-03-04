//! COMDIS full pipeline roundtrip tests.
//!
//! Tests EDIFACT -> tokenize -> split -> assemble -> map_interchange(SG2)
//! -> map_interchange_reverse -> disassemble -> render -> byte-identical comparison.

mod common;

use std::path::Path;

// ── Roundtrip tests ──

#[test]
fn test_comdis_pid_29001_roundtrip() {
    common::comdis::run_full_roundtrip("29001");
}

#[test]
fn test_comdis_pid_29001_generated_roundtrip() {
    let fixture =
        Path::new(common::comdis::FIXTURE_DIR).join("generated/29001_COMDIS_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::comdis::load_pid_filtered_mig("29001").unwrap();
    let (msg_engine, tx_engine) = common::comdis::load_split_engines("29001");

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "29001",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::comdis::TX_GROUP,
    );
}

#[test]
fn test_comdis_pid_29002_generated_roundtrip() {
    let fixture =
        Path::new(common::comdis::FIXTURE_DIR).join("generated/29002_COMDIS_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::comdis::load_pid_filtered_mig("29002").unwrap();
    let (msg_engine, tx_engine) = common::comdis::load_split_engines("29002");

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "29002",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::comdis::TX_GROUP,
    );
}

/// Verify all COMDIS PID TOML mappings load successfully.
#[test]
fn test_comdis_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;

    let schema_dir = Path::new(common::comdis::SCHEMA_DIR);
    let msg_dir = common::comdis::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = ["29001", "29002"];

    for pid in pids {
        let tx_dir = common::comdis::pid_dir(pid);
        if !tx_dir.exists() {
            eprintln!("PID {pid}: no mapping directory -- skipping");
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
