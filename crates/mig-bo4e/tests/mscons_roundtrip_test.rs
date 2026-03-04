//! MSCONS full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange(SG5)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

#[test]
fn test_mscons_pid_13017_roundtrip() {
    common::mscons::run_full_roundtrip("13017");
}

/// Verify all MSCONS PID TOML mappings load successfully.
#[test]
fn test_mscons_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::mscons::SCHEMA_DIR);
    let msg_dir = common::mscons::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    // Verify each PID's transaction engine loads
    let pids = [
        "13002", "13003", "13005", "13006", "13007", "13008", "13009", "13010", "13011", "13012",
        "13013", "13014", "13015", "13016", "13017", "13018", "13019", "13020", "13021", "13022",
        "13023", "13024", "13025", "13026", "13027", "13028",
    ];

    for pid in pids {
        let tx_dir = common::mscons::pid_dir(pid);
        if !tx_dir.exists() {
            eprintln!("PID {pid}: no mapping directory — skipping");
            continue;
        }

        let cmn_dir = common::mscons::common_dir();
        if cmn_dir.exists() {
            let idx = common::mscons::schema_index(pid);
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
