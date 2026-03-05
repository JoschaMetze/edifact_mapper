//! PARTIN full pipeline roundtrip tests.
//!
//! 7 PIDs (37000–37006) for market participant master data.
//! TX_GROUP = "SG4" (NAD-initiated party data, after UNS+D).
//!
//! Tests: EDIFACT → tokenize → split → assemble → map_interchange(SG4)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Generated fixture roundtrip tests ──

#[test]
fn test_partin_pid_37000_generated_roundtrip() {
    run_generated_roundtrip("37000");
}

#[test]
fn test_partin_pid_37001_generated_roundtrip() {
    run_generated_roundtrip("37001");
}

#[test]
fn test_partin_pid_37002_generated_roundtrip() {
    run_generated_roundtrip("37002");
}

#[test]
fn test_partin_pid_37003_generated_roundtrip() {
    run_generated_roundtrip("37003");
}

#[test]
fn test_partin_pid_37004_generated_roundtrip() {
    run_generated_roundtrip("37004");
}

#[test]
fn test_partin_pid_37005_generated_roundtrip() {
    run_generated_roundtrip("37005");
}

#[test]
fn test_partin_pid_37006_generated_roundtrip() {
    run_generated_roundtrip("37006");
}

/// Verify all PARTIN PID TOML mappings load successfully.
#[test]
fn test_partin_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::partin::SCHEMA_DIR);
    let msg_dir = common::partin::message_dir();
    let cmn_dir = common::partin::common_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    // Verify common engine loads
    if cmn_dir.exists() {
        let cmn_engine = MappingEngine::load(&cmn_dir)
            .unwrap()
            .with_path_resolver(resolver.clone());
        let count = cmn_engine.definitions().len();
        eprintln!("Common: {count} definitions loaded OK");
    }

    let pids = [
        "37000", "37001", "37002", "37003", "37004", "37005", "37006",
    ];

    for pid in pids {
        let tx_dir = common::partin::pid_dir(pid);
        if !tx_dir.exists() {
            eprintln!("PID {pid}: no per-PID directory (using common only)");
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
    let fixture = common::partin::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::partin::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let (msg_engine, tx_engine) = common::partin::load_engines_for_pid(pid);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::partin::TX_GROUP,
    );
}
