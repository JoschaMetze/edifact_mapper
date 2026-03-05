//! REQOTE full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange(SG27)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Generated fixture roundtrips ──

#[test]
fn test_reqote_pid_35001_generated_roundtrip() {
    run_generated_roundtrip("35001");
}

#[test]
fn test_reqote_pid_35002_generated_roundtrip() {
    run_generated_roundtrip("35002");
}

#[test]
fn test_reqote_pid_35003_generated_roundtrip() {
    run_generated_roundtrip("35003");
}

#[test]
fn test_reqote_pid_35004_generated_roundtrip() {
    run_generated_roundtrip("35004");
}

/// Verify all REQOTE PID TOML mappings load successfully.
#[test]
fn test_reqote_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::reqote::SCHEMA_DIR);
    let msg_dir = common::reqote::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = ["35001", "35002", "35003", "35004"];

    for pid in pids {
        let tx_dir = common::reqote::pid_dir(pid);
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

// ── Helpers ──

fn run_generated_roundtrip(pid: &str) {
    let fixture = common::reqote::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::reqote::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let tx_dir = common::reqote::pid_dir(pid);
    let (msg_engine, tx_engine) = if tx_dir.exists() {
        common::reqote::load_split_engines(pid)
    } else {
        let msg = common::reqote::load_message_engine();
        let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
        (msg, tx)
    };

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::reqote::TX_GROUP,
    );
}
