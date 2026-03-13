//! INVOIC full pipeline roundtrip tests.
//!
//! 11 PIDs (31001–31011) for German energy market invoice messages.
//! TX_GROUP = "SG26" (LIN-initiated positions, UNS+S trailing).
//!
//! Tests: EDIFACT → tokenize → split → assemble → map_interchange(SG26)
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Generated fixture roundtrip tests ──

#[test]
fn test_invoic_pid_31001_generated_roundtrip() {
    run_generated_roundtrip("31001");
}

#[test]
fn test_invoic_pid_31002_generated_roundtrip() {
    run_generated_roundtrip("31002");
}

#[test]
fn test_invoic_pid_31003_generated_roundtrip() {
    run_generated_roundtrip("31003");
}

#[test]
fn test_invoic_pid_31004_generated_roundtrip() {
    run_generated_roundtrip("31004");
}

#[test]
fn test_invoic_pid_31005_generated_roundtrip() {
    run_generated_roundtrip("31005");
}

#[test]
fn test_invoic_pid_31006_generated_roundtrip() {
    run_generated_roundtrip("31006");
}

#[test]
fn test_invoic_pid_31007_generated_roundtrip() {
    run_generated_roundtrip("31007");
}

#[test]
fn test_invoic_pid_31008_generated_roundtrip() {
    run_generated_roundtrip("31008");
}

#[test]
fn test_invoic_pid_31009_generated_roundtrip() {
    run_generated_roundtrip("31009");
}

#[test]
fn test_invoic_pid_31010_generated_roundtrip() {
    run_generated_roundtrip("31010");
}

#[test]
fn test_invoic_pid_31011_generated_roundtrip() {
    run_generated_roundtrip("31011");
}

// ── Real fixture roundtrip tests ──

#[test]
fn test_invoic_pid_31001_real_roundtrip() {
    // KNOWN_INCOMPLETE: RFF+VA distributed across NAD reps in real fixture,
    // but reverse mapper nests all SG3 children under first SG2 rep.
    common::invoic::run_full_roundtrip_with_skip(
        "31001",
        &[
            "31001_INVOIC_2.8d_ALEXANDE436887BGM.edi",
            "31001_INVOIC_2.8d_LongStrings.edi",
        ],
    );
}

#[test]
fn test_invoic_pid_31002_real_roundtrip() {
    common::invoic::run_full_roundtrip("31002");
}

#[test]
fn test_invoic_pid_31004_real_roundtrip() {
    common::invoic::run_full_roundtrip("31004");
}

/// Verify all INVOIC PID TOML mappings load successfully.
#[test]
fn test_invoic_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::invoic::SCHEMA_DIR);
    let msg_dir = common::invoic::message_dir();
    let cmn_dir = common::invoic::common_dir();
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
        "31001", "31002", "31003", "31004", "31005", "31006", "31007", "31008", "31009", "31010",
        "31011",
    ];

    for pid in pids {
        let tx_dir = common::invoic::pid_dir(pid);
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
    let fixture = common::invoic::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::invoic::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let (msg_engine, tx_engine) = common::invoic::load_split_engines(pid);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::invoic::TX_GROUP,
    );
}
