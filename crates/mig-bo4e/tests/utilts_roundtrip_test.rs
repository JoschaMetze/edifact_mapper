//! UTILTS full pipeline roundtrip tests.
//!
//! 8 PIDs in 4 structural families, all sharing tx_group = "SG5" (IDE+24 entry).
//! - Berechnungsformel (PID 25001): LOC+172, STS+Z23, Zeitscheiben, 2 SG8 variants
//! - Übersicht (PIDs 25004, 25006, 25007): DTM+157, DTM+293, STS+Z36, SG8+SG9
//! - Ausgerollt (PIDs 25005, 25008, 25009): LOC+Z09, DTM+Z34/Z35, SG8 with high reps
//! - Antwort (PID 25010): STS+E01+FTX+ACB, no SG8
//!
//! Tests: EDIFACT → tokenize → split → assemble → map_interchange
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

use std::path::Path;

// ── Real fixture roundtrip tests (PID 25005 only) ──

#[test]
fn test_utilts_pid_25005_roundtrip() {
    common::utilts::run_full_roundtrip("25005");
}

// ── Generated fixture roundtrip tests (all 8 PIDs) ──

#[test]
fn test_utilts_pid_25001_generated_roundtrip() {
    run_generated_roundtrip("25001");
}

#[test]
fn test_utilts_pid_25004_generated_roundtrip() {
    run_generated_roundtrip("25004");
}

#[test]
fn test_utilts_pid_25005_generated_roundtrip() {
    run_generated_roundtrip("25005");
}

#[test]
fn test_utilts_pid_25006_generated_roundtrip() {
    run_generated_roundtrip("25006");
}

#[test]
fn test_utilts_pid_25007_generated_roundtrip() {
    run_generated_roundtrip("25007");
}

#[test]
fn test_utilts_pid_25008_generated_roundtrip() {
    run_generated_roundtrip("25008");
}

#[test]
fn test_utilts_pid_25009_generated_roundtrip() {
    run_generated_roundtrip("25009");
}

#[test]
fn test_utilts_pid_25010_generated_roundtrip() {
    run_generated_roundtrip("25010");
}

// ── TOML loading test ──

#[test]
fn test_utilts_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;

    let schema_dir = Path::new(common::utilts::SCHEMA_DIR);
    let msg_dir = common::utilts::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let all_pids = [
        "25001", "25004", "25005", "25006", "25007", "25008", "25009", "25010",
    ];

    for pid in all_pids {
        let tx_dir = common::utilts::pid_dir(pid);
        if !tx_dir.exists() {
            eprintln!("PID {pid}: no mapping directory -- skipping TOML load check");
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

// ── Helper ──

fn run_generated_roundtrip(pid: &str) {
    let fixture = common::utilts::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::utilts::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let tx_dir = common::utilts::pid_dir(pid);
    let (msg_engine, tx_engine) = if tx_dir.exists() {
        common::utilts::load_split_engines(pid)
    } else {
        let msg = common::utilts::load_message_engine();
        let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
        (msg, tx)
    };

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::utilts::TX_GROUP,
    );
}
