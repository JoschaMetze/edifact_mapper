//! INSRPT full pipeline roundtrip tests.
//!
//! 8 PIDs (23001–23012) for German energy market fault/inspection reporting (WiM).
//! TX_GROUP = "SG3" (DOC-initiated document groups, no UNS).
//!
//! Tests: EDIFACT → tokenize → split → assemble → map_interchange(SG3)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Generated fixture roundtrip tests ──

#[test]
fn test_insrpt_pid_23001_generated_roundtrip() {
    run_generated_roundtrip("23001");
}

#[test]
fn test_insrpt_pid_23003_generated_roundtrip() {
    run_generated_roundtrip("23003");
}

#[test]
fn test_insrpt_pid_23004_generated_roundtrip() {
    run_generated_roundtrip("23004");
}

#[test]
fn test_insrpt_pid_23005_generated_roundtrip() {
    run_generated_roundtrip("23005");
}

#[test]
fn test_insrpt_pid_23008_generated_roundtrip() {
    run_generated_roundtrip("23008");
}

#[test]
fn test_insrpt_pid_23009_generated_roundtrip() {
    run_generated_roundtrip("23009");
}

#[test]
fn test_insrpt_pid_23011_generated_roundtrip() {
    run_generated_roundtrip("23011");
}

#[test]
fn test_insrpt_pid_23012_generated_roundtrip() {
    run_generated_roundtrip("23012");
}

// ── TOML loading test ──

#[test]
fn test_insrpt_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::insrpt::SCHEMA_DIR);
    let msg_dir = common::insrpt::message_dir();
    let cmn_dir = common::insrpt::common_dir();
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
        "23001", "23003", "23004", "23005", "23008", "23009", "23011", "23012",
    ];

    for pid in pids {
        let tx_dir = common::insrpt::pid_dir(pid);
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
    let fixture = common::insrpt::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::insrpt::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let (msg_engine, tx_engine) = common::insrpt::load_engines_for_pid(pid);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::insrpt::TX_GROUP,
    );
}
