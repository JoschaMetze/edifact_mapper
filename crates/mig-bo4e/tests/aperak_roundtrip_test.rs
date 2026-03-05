//! APERAK full pipeline roundtrip tests.
//!
//! Tests EDIFACT → tokenize → split → assemble → map_interchange
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.

mod common;

// ── Real fixture roundtrips ──

#[test]
fn test_aperak_92001_roundtrip() {
    run_real_fixture_roundtrips();
}

// ── Generated fixture roundtrip ──

#[test]
fn test_aperak_92001_generated_roundtrip() {
    run_generated_roundtrip();
}

/// Verify all APERAK TOML mappings load successfully.
#[test]
fn test_aperak_toml_loading() {
    use mig_bo4e::engine::MappingEngine;
    use mig_bo4e::path_resolver::PathResolver;
    use std::path::Path;

    let schema_dir = Path::new(common::aperak::SCHEMA_DIR);
    let msg_dir = common::aperak::message_dir();
    let resolver = PathResolver::from_schema_dir(schema_dir);

    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(resolver);
    assert!(
        !msg_engine.definitions().is_empty(),
        "Message engine should have definitions"
    );
    eprintln!(
        "APERAK: {} message definitions loaded OK",
        msg_engine.definitions().len()
    );
}

// ── Helpers ──

fn run_real_fixture_roundtrips() {
    let fixtures = common::aperak::discover_fixtures();
    if fixtures.is_empty() {
        eprintln!("APERAK: no fixtures found -- skipping");
        return;
    }

    let Some(mig) = common::aperak::load_pid_filtered_mig() else {
        eprintln!("APERAK: MIG/AHB XML not available -- skipping");
        return;
    };

    let msg_engine = common::aperak::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    for fixture in &fixtures {
        common::test_utils::run_single_fixture_roundtrip_with_tx_group(
            common::aperak::PID_FIXTURE,
            fixture,
            &mig,
            &msg_engine,
            &tx_engine,
            common::aperak::TX_GROUP,
        );
    }
}

fn run_generated_roundtrip() {
    let Some(fixture) = common::aperak::discover_generated_fixture() else {
        eprintln!("APERAK: no generated fixture found -- skipping");
        return;
    };
    assert!(fixture.exists(), "Generated fixture not found: {fixture:?}");

    let Some(mig) = common::aperak::load_pid_filtered_mig() else {
        eprintln!("APERAK: MIG/AHB XML not available -- skipping");
        return;
    };

    let msg_engine = common::aperak::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    common::test_utils::run_single_fixture_roundtrip_with_tx_group(
        common::aperak::PID_FIXTURE,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::aperak::TX_GROUP,
    );
}
