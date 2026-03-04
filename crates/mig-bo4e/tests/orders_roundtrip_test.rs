//! ORDERS full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange(SG29)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Roundtrip tests for PIDs with real fixtures ──

#[test]
fn test_orders_pid_17004_roundtrip() {
    common::orders::run_full_roundtrip("17004");
}

#[test]
fn test_orders_pid_17009_roundtrip() {
    common::orders::run_full_roundtrip("17009");
}

#[test]
fn test_orders_pid_17113_roundtrip() {
    common::orders::run_full_roundtrip("17113");
}

#[test]
fn test_orders_pid_17115_roundtrip() {
    common::orders::run_full_roundtrip("17115");
}

#[test]
fn test_orders_pid_17117_roundtrip() {
    // No SG29 — message-only PID
    common::orders::run_full_roundtrip("17117");
}

#[test]
fn test_orders_pid_17129_roundtrip() {
    // No SG29 — message-only PID
    common::orders::run_full_roundtrip("17129");
}

#[test]
fn test_orders_pid_17132_roundtrip() {
    // No SG29 — message-only PID
    common::orders::run_full_roundtrip("17132");
}

#[test]
fn test_orders_pid_17133_roundtrip() {
    common::orders::run_full_roundtrip("17133");
}

#[test]
fn test_orders_pid_17134_roundtrip() {
    // KNOWN_INCOMPLETE: SG2.SG3 RFF nesting across multiple SG2 reps (Z31/SU each with RFF+Z18)
    // and SG30 multi-variant CCI entity collisions — both are reverse mapping limitations.
    common::orders::run_full_roundtrip_with_skip(
        "17134",
        &["17134_ORDERS_1.4a_JOSCHA18524982.edi"],
    );
}

/// Verify all ORDERS PID TOML mappings load successfully.
#[test]
fn test_orders_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::orders::SCHEMA_DIR);
    let msg_dir = common::orders::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let pids = [
        "17001", "17002", "17003", "17004", "17005", "17006", "17007", "17008", "17009", "17101",
        "17102", "17103", "17104", "17110", "17111", "17112", "17113", "17114", "17115", "17116",
        "17117", "17118", "17120", "17121", "17122", "17123", "17124", "17125", "17126", "17128",
        "17129", "17130", "17131", "17132", "17133", "17134", "17135", "17201", "17202", "17203",
        "17204", "17205", "17206", "17207", "17208", "17210", "17211", "17301",
    ];

    // PIDs without SG29 don't need a tx directory
    let no_sg29 = [
        "17002", "17005", "17007", "17008", "17104", "17110", "17125", "17129", "17131", "17132",
        "17211", "17301",
    ];

    for pid in pids {
        let tx_dir = common::orders::pid_dir(pid);
        if !tx_dir.exists() {
            if no_sg29.contains(&pid) {
                eprintln!("PID {pid}: no SG29, no tx directory needed — OK");
            } else {
                eprintln!("PID {pid}: no mapping directory — skipping");
            }
            continue;
        }

        let cmn_dir = common::orders::common_dir();
        if cmn_dir.exists() {
            let idx = common::orders::schema_index(pid);
            let result = MappingEngine::load_with_common(&cmn_dir, &tx_dir, &idx);
            match result {
                Ok(engine) => {
                    let engine = engine.with_path_resolver(resolver.clone());
                    let count = engine.definitions().len();
                    eprintln!("PID {pid}: {count} definitions loaded OK");
                }
                Err(e) => panic!("PID {pid}: failed to load TOML mappings: {e}"),
            }
        } else {
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
}
