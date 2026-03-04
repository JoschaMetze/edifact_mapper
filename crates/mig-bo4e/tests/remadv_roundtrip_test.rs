//! REMADV full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange(SG5)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Roundtrip tests for PIDs with real fixtures ──

#[test]
fn test_remadv_pid_33001_roundtrip() {
    common::remadv::run_full_roundtrip("33001");
}

#[test]
fn test_remadv_pid_33002_roundtrip() {
    common::remadv::run_full_roundtrip("33002");
}

#[test]
fn test_remadv_pid_33003_roundtrip() {
    common::remadv::run_full_roundtrip("33003");
}

#[test]
fn test_remadv_pid_33004_roundtrip() {
    common::remadv::run_full_roundtrip("33004");
}

/// Verify all REMADV PID TOML mappings load successfully.
#[test]
fn test_remadv_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::remadv::SCHEMA_DIR);
    let msg_dir = common::remadv::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = ["33001", "33002", "33003", "33004"];

    for pid in pids {
        let tx_dir = common::remadv::pid_dir(pid);
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
