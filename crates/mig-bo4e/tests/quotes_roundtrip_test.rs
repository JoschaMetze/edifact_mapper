//! QUOTES full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange(SG27)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Real fixture roundtrip ──

#[test]
fn test_quotes_pid_15004_roundtrip() {
    common::quotes::run_full_roundtrip("15004");
}

// ── Generated fixture roundtrips ──

#[test]
fn test_quotes_pid_15001_generated_roundtrip() {
    run_generated_roundtrip("15001");
}

#[test]
fn test_quotes_pid_15002_generated_roundtrip() {
    run_generated_roundtrip("15002");
}

#[test]
fn test_quotes_pid_15003_generated_roundtrip() {
    run_generated_roundtrip("15003");
}

#[test]
fn test_quotes_pid_15004_generated_roundtrip() {
    run_generated_roundtrip("15004");
}

/// Verify all QUOTES PID TOML mappings load successfully.
#[test]
fn test_quotes_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::quotes::SCHEMA_DIR);
    let msg_dir = common::quotes::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = ["15001", "15002", "15003", "15004"];

    for pid in pids {
        let tx_dir = common::quotes::pid_dir(pid);
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
    let fixture = common::quotes::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::quotes::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let tx_dir = common::quotes::pid_dir(pid);
    let (msg_engine, tx_engine) = if tx_dir.exists() {
        common::quotes::load_split_engines(pid)
    } else {
        let msg = common::quotes::load_message_engine();
        let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
        (msg, tx)
    };

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::quotes::TX_GROUP,
    );
}
