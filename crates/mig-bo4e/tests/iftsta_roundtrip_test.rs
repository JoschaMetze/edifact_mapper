//! IFTSTA full pipeline roundtrip tests.
//!
//! Two structural families:
//! - Family A (MaBiS): PIDs 21000–21005, tx_group = SG4 (EQD-initiated)
//! - Family B (GPKE/WiM): PIDs 21007–21047, tx_group = SG14 (CNI-initiated)
//!
//! Tests: EDIFACT → tokenize → split → assemble → map_interchange
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

use std::path::Path;

// ── Real fixture roundtrip tests ──

#[test]
fn test_iftsta_pid_21039_roundtrip() {
    common::iftsta::run_full_roundtrip("21039");
}

#[test]
fn test_iftsta_pid_21044_roundtrip() {
    common::iftsta::run_full_roundtrip("21044");
}

#[test]
fn test_iftsta_pid_21047_roundtrip() {
    // Skip the S2.1 fixture (different MIG version)
    common::iftsta::run_full_roundtrip_with_skip(
        "21047",
        &["21047_IFSTA_S2.1_ALEXANDE104366BGM.edi"],
    );
}

// ── Family A generated fixture roundtrip tests ──

#[test]
fn test_iftsta_pid_21000_generated_roundtrip() {
    run_generated_roundtrip("21000");
}

#[test]
fn test_iftsta_pid_21001_generated_roundtrip() {
    run_generated_roundtrip("21001");
}

#[test]
fn test_iftsta_pid_21002_generated_roundtrip() {
    run_generated_roundtrip("21002");
}

#[test]
fn test_iftsta_pid_21003_generated_roundtrip() {
    run_generated_roundtrip("21003");
}

#[test]
fn test_iftsta_pid_21004_generated_roundtrip() {
    run_generated_roundtrip("21004");
}

#[test]
fn test_iftsta_pid_21005_generated_roundtrip() {
    run_generated_roundtrip("21005");
}

// ── Family B generated fixture roundtrip tests ──

#[test]
fn test_iftsta_pid_21007_generated_roundtrip() {
    run_generated_roundtrip("21007");
}

#[test]
fn test_iftsta_pid_21009_generated_roundtrip() {
    run_generated_roundtrip("21009");
}

#[test]
fn test_iftsta_pid_21010_generated_roundtrip() {
    run_generated_roundtrip("21010");
}

#[test]
fn test_iftsta_pid_21011_generated_roundtrip() {
    run_generated_roundtrip("21011");
}

#[test]
fn test_iftsta_pid_21012_generated_roundtrip() {
    run_generated_roundtrip("21012");
}

#[test]
fn test_iftsta_pid_21013_generated_roundtrip() {
    run_generated_roundtrip("21013");
}

#[test]
fn test_iftsta_pid_21015_generated_roundtrip() {
    run_generated_roundtrip("21015");
}

#[test]
fn test_iftsta_pid_21018_generated_roundtrip() {
    run_generated_roundtrip("21018");
}

#[test]
fn test_iftsta_pid_21024_generated_roundtrip() {
    run_generated_roundtrip("21024");
}

#[test]
fn test_iftsta_pid_21025_generated_roundtrip() {
    run_generated_roundtrip("21025");
}

#[test]
fn test_iftsta_pid_21026_generated_roundtrip() {
    run_generated_roundtrip("21026");
}

#[test]
fn test_iftsta_pid_21027_generated_roundtrip() {
    run_generated_roundtrip("21027");
}

#[test]
fn test_iftsta_pid_21028_generated_roundtrip() {
    run_generated_roundtrip("21028");
}

#[test]
fn test_iftsta_pid_21029_generated_roundtrip() {
    run_generated_roundtrip("21029");
}

#[test]
fn test_iftsta_pid_21030_generated_roundtrip() {
    run_generated_roundtrip("21030");
}

#[test]
fn test_iftsta_pid_21031_generated_roundtrip() {
    run_generated_roundtrip("21031");
}

#[test]
fn test_iftsta_pid_21032_generated_roundtrip() {
    run_generated_roundtrip("21032");
}

#[test]
fn test_iftsta_pid_21033_generated_roundtrip() {
    run_generated_roundtrip("21033");
}

#[test]
fn test_iftsta_pid_21035_generated_roundtrip() {
    run_generated_roundtrip("21035");
}

#[test]
fn test_iftsta_pid_21036_generated_roundtrip() {
    run_generated_roundtrip("21036");
}

#[test]
fn test_iftsta_pid_21037_generated_roundtrip() {
    run_generated_roundtrip("21037");
}

#[test]
fn test_iftsta_pid_21038_generated_roundtrip() {
    run_generated_roundtrip("21038");
}

#[test]
fn test_iftsta_pid_21039_generated_roundtrip() {
    run_generated_roundtrip("21039");
}

#[test]
fn test_iftsta_pid_21040_generated_roundtrip() {
    run_generated_roundtrip("21040");
}

#[test]
fn test_iftsta_pid_21042_generated_roundtrip() {
    run_generated_roundtrip("21042");
}

#[test]
fn test_iftsta_pid_21043_generated_roundtrip() {
    run_generated_roundtrip("21043");
}

#[test]
fn test_iftsta_pid_21044_generated_roundtrip() {
    run_generated_roundtrip("21044");
}

#[test]
fn test_iftsta_pid_21045_generated_roundtrip() {
    run_generated_roundtrip("21045");
}

#[test]
fn test_iftsta_pid_21047_generated_roundtrip() {
    run_generated_roundtrip("21047");
}

// ── TOML loading test ──

#[test]
fn test_iftsta_all_pids_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;

    let schema_dir = Path::new(common::iftsta::SCHEMA_DIR);
    let msg_dir = common::iftsta::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    // Verify message-level engine loads
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver.clone());
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );

    let all_pids: Vec<&str> = common::iftsta::FAMILY_A_PIDS
        .iter()
        .chain(common::iftsta::FAMILY_B_PIDS.iter())
        .copied()
        .collect();

    for pid in all_pids {
        let tx_dir = common::iftsta::pid_dir(pid);
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
    let fixture = common::iftsta::discover_generated_fixture(pid);
    let Some(fixture) = fixture else {
        eprintln!("PID {pid}: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::iftsta::load_pid_filtered_mig(pid) else {
        eprintln!("PID {pid}: MIG/AHB XML not available -- skipping");
        return;
    };

    let tx_dir = common::iftsta::pid_dir(pid);
    let (msg_engine, tx_engine) = if tx_dir.exists() {
        common::iftsta::load_split_engines(pid)
    } else {
        let msg = common::iftsta::load_message_engine();
        let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
        (msg, tx)
    };

    let tx_group = if common::iftsta::FAMILY_A_PIDS.contains(&pid) {
        "SG4"
    } else {
        "SG14"
    };

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        pid,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        tx_group,
    );
}
