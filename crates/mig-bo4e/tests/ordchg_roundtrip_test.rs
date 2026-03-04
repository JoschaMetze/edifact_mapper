//! ORDCHG full pipeline roundtrip tests.
//!
//! Tests EDIFACT -> tokenize -> split -> assemble -> map_interchange("")
//! -> map_interchange_reverse -> disassemble -> render -> byte-identical comparison.
//!
//! All 3 ORDCHG PIDs are message-only (no transaction group).

mod common;

use std::path::Path;

// ── Roundtrip tests (all generated fixtures) ──

#[test]
fn test_ordchg_pid_39000_generated_roundtrip() {
    let fixture =
        Path::new(common::ordchg::FIXTURE_DIR).join("generated/39000_ORDCHG_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::ordchg::load_pid_filtered_mig("39000").unwrap();
    let msg_engine = common::ordchg::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "39000",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::ordchg::TX_GROUP,
    );
}

#[test]
fn test_ordchg_pid_39001_generated_roundtrip() {
    let fixture =
        Path::new(common::ordchg::FIXTURE_DIR).join("generated/39001_ORDCHG_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::ordchg::load_pid_filtered_mig("39001").unwrap();
    let msg_engine = common::ordchg::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "39001",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::ordchg::TX_GROUP,
    );
}

#[test]
fn test_ordchg_pid_39002_generated_roundtrip() {
    let fixture =
        Path::new(common::ordchg::FIXTURE_DIR).join("generated/39002_ORDCHG_generated.edi");
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let mig = common::ordchg::load_pid_filtered_mig("39002").unwrap();
    let msg_engine = common::ordchg::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        "39002",
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::ordchg::TX_GROUP,
    );
}

/// Verify all ORDCHG PID TOML mappings load successfully.
#[test]
fn test_ordchg_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;

    let schema_dir = Path::new(common::ordchg::SCHEMA_DIR);
    let msg_dir = common::ordchg::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = ["39000", "39001", "39002"];

    for pid in pids {
        let tx_dir = common::ordchg::pid_dir(pid);
        if tx_dir.exists() {
            match MappingEngine::load(&tx_dir) {
                Ok(engine) => {
                    let engine = engine.with_path_resolver(resolver.clone());
                    let count = engine.definitions().len();
                    eprintln!("PID {pid}: {count} definitions loaded OK");
                }
                Err(e) => panic!("PID {pid}: failed to load TOML mappings: {e}"),
            }
        } else {
            eprintln!("PID {pid}: no PID-specific directory (message-only) -- OK");
        }
    }
}
