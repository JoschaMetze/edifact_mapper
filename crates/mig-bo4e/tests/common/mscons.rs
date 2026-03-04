//! MSCONS-specific test utilities for mig-bo4e integration tests.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::test_utils;

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/MSCONS_MIG_2_4c_außerordentliche_20240726.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/MSCONS_AHB_3_1f_Fehlerkorrektur_20250623.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/MSCONS/FV2504";
pub const GENERATED_FIXTURE_DIR: &str = "../../fixtures/generated/fv2504/mscons";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/MSCONS";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/mscons/pids";

/// MSCONS transaction group — SG5 (NAD+DP delivery point).
pub const TX_GROUP: &str = "SG5";

pub fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(Path::new(SCHEMA_DIR))
}

pub fn message_dir() -> PathBuf {
    Path::new(MAPPINGS_BASE).join("message")
}

pub fn common_dir() -> PathBuf {
    Path::new(MAPPINGS_BASE).join("common")
}

pub fn pid_dir(pid: &str) -> PathBuf {
    Path::new(MAPPINGS_BASE).join(format!("pid_{pid}"))
}

pub fn schema_index(pid: &str) -> PidSchemaIndex {
    let schema_path = Path::new(SCHEMA_DIR).join(format!("pid_{pid}_schema.json"));
    PidSchemaIndex::from_schema_file(&schema_path).unwrap()
}

/// Parse MSCONS MIG/AHB and return a PID-filtered MIG schema.
pub fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        return None;
    }
    let mig = parse_mig(mig_path, "MSCONS", None, "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "MSCONS", None, "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    Some(filter_mig_for_pid(&mig, &numbers))
}

/// Discover all `.edi` fixture files for a given MSCONS PID.
pub fn discover_fixtures(pid: &str) -> Vec<PathBuf> {
    let dir = Path::new(FIXTURE_DIR);
    if !dir.exists() {
        return vec![];
    }
    let prefix = format!("{pid}_");
    let mut fixtures: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(".edi"))
                .unwrap_or(false)
        })
        .collect();
    fixtures.sort();
    fixtures
}

/// Discover generated fixture file for a MSCONS PID.
pub fn discover_generated_fixture(pid: &str) -> Option<PathBuf> {
    let dir = Path::new(FIXTURE_DIR).join("generated");
    let prefix = format!("{pid}_");
    if !dir.exists() {
        return None;
    }
    std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(".edi"))
                .unwrap_or(false)
        })
}

/// Load split engines for MSCONS with common/ inheritance when available.
pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    let msg_dir = message_dir();
    let cmn_dir = common_dir();
    let tx_dir = pid_dir(pid);
    let resolver = path_resolver();
    if cmn_dir.exists() {
        let idx = schema_index(pid);
        let (m, t) =
            MappingEngine::load_split_with_common(&msg_dir, &cmn_dir, &tx_dir, &idx).unwrap();
        (
            m.with_path_resolver(resolver.clone()),
            t.with_path_resolver(resolver),
        )
    } else {
        let (m, t) = MappingEngine::load_split(&msg_dir, &tx_dir).unwrap();
        (
            m.with_path_resolver(resolver.clone()),
            t.with_path_resolver(resolver),
        )
    }
}

/// Full pipeline roundtrip for a MSCONS PID (tx_group = "SG5").
pub fn run_full_roundtrip(pid: &str) {
    run_full_roundtrip_with_skip(pid, &[]);
}

/// Full pipeline roundtrip with fixture skip list.
/// Note: generated fixtures are excluded — they were generated with incomplete
/// schemas (missing SG10) and have malformed segment ordering.
pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    let all_fixtures = discover_fixtures(pid);

    if all_fixtures.is_empty() {
        eprintln!("Skipping roundtrip for PID {pid}: no fixtures found");
        return;
    }

    let Some(filtered_mig) = load_pid_filtered_mig(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
        return;
    };

    let tx_dir = pid_dir(pid);
    if !message_dir().exists() || !tx_dir.exists() {
        eprintln!("Skipping roundtrip for PID {pid}: mapping directories not found");
        return;
    }
    let (msg_engine, tx_engine) = load_split_engines(pid);

    let mut tested = 0;
    let mut skipped = 0;

    for fixture_path in &all_fixtures {
        let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();

        if known_incomplete.contains(&fixture_name) {
            eprintln!("PID {pid}: {fixture_name} -- SKIPPED (known incomplete mapping)");
            skipped += 1;
            continue;
        }

        test_utils::run_single_fixture_roundtrip_with_tx_group(
            pid,
            fixture_path,
            &filtered_mig,
            &msg_engine,
            &tx_engine,
            TX_GROUP,
        );
        tested += 1;
    }

    eprintln!("PID {pid}: {tested} fixtures passed, {skipped} skipped (known incomplete)");
}
