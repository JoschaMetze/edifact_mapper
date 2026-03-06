//! INVOIC-specific test utilities for mig-bo4e integration tests.
//!
//! 11 PIDs (31001–31011) for German energy market invoice messages.
//! TX_GROUP = "SG26" (LIN-initiated positions, UNS+S trailing).
//! PID 31004 (Storno) has no SG26 — 0 tx_group reps.
//! PID 31002 (NN-Rechnung) has extra SG39 ALC surcharge groups.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/INVOIC_MIG_2.8d_Fehlerkorrektur_20250131.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/INVOIC_AHB_2_5d_Fehlerkorrektur_20250623.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/INVOIC/FV2504";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/INVOIC";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/invoic/pids";

/// INVOIC transaction group — SG26 (LIN-initiated positions).
pub const TX_GROUP: &str = "SG26";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "INVOIC",
    variant: None,
    tx_group: TX_GROUP,
    format_version: "FV2504",
};

pub fn path_resolver() -> PathResolver {
    CONFIG.path_resolver()
}

pub fn message_dir() -> PathBuf {
    CONFIG.message_dir()
}

pub fn common_dir() -> PathBuf {
    CONFIG.common_dir()
}

pub fn pid_dir(pid: &str) -> PathBuf {
    CONFIG.pid_dir(pid)
}

pub fn schema_index(pid: &str) -> PidSchemaIndex {
    CONFIG.schema_index(pid)
}

pub fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    CONFIG.load_pid_filtered_mig(pid_id)
}

pub fn discover_fixtures(pid: &str) -> Vec<PathBuf> {
    CONFIG.discover_fixtures(pid)
}

pub fn load_message_engine() -> MappingEngine {
    CONFIG.load_message_engine()
}

pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    CONFIG.load_split_engines(pid)
}

/// Load engines for a PID, handling the case where no per-PID dir exists.
///
/// When `pid_NNNNN/` exists → standard load (message + common + pid).
/// When only `common/` exists → load common with schema filtering (no pid overrides).
pub fn load_engines_for_pid(pid: &str) -> (MappingEngine, MappingEngine) {
    let tx_dir = CONFIG.pid_dir(pid);
    if tx_dir.exists() {
        CONFIG.load_split_engines(pid)
    } else {
        let cmn_dir = CONFIG.common_dir();
        let resolver = CONFIG.path_resolver();
        let msg = CONFIG.load_message_engine();
        if cmn_dir.exists() {
            let idx = CONFIG.schema_index(pid);
            let mut defs = MappingEngine::load(&cmn_dir)
                .unwrap()
                .with_path_resolver(resolver.clone())
                .definitions()
                .to_vec();
            defs.retain(|d| {
                d.meta
                    .source_path
                    .as_deref()
                    .map(|sp| idx.has_group(sp))
                    .unwrap_or(true)
            });
            let tx = MappingEngine::from_definitions(defs).with_path_resolver(resolver);
            (msg, tx)
        } else {
            let tx = MappingEngine::from_definitions(vec![]);
            (msg, tx)
        }
    }
}

pub fn run_full_roundtrip(pid: &str) {
    run_full_roundtrip_with_skip(pid, &[]);
}

pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    let fixtures = CONFIG.discover_fixtures(pid);
    if fixtures.is_empty() {
        eprintln!("Skipping roundtrip for PID {pid}: no fixtures found");
        return;
    }

    let Some(filtered_mig) = CONFIG.load_pid_filtered_mig(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
        return;
    };

    let (msg_engine, tx_engine) = load_engines_for_pid(pid);

    let mut tested = 0;
    let mut skipped = 0;

    for fixture_path in &fixtures {
        let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();
        if known_incomplete.contains(&fixture_name) {
            eprintln!("PID {pid}: {fixture_name} -- SKIPPED (known incomplete mapping)");
            skipped += 1;
            continue;
        }

        super::test_utils::run_single_fixture_roundtrip_with_tx_group(
            pid,
            fixture_path,
            &filtered_mig,
            &msg_engine,
            &tx_engine,
            TX_GROUP,
        );
        tested += 1;
    }

    assert!(
        tested > 0 || skipped > 0,
        "PID {pid}: expected at least one fixture"
    );
    if skipped > 0 {
        eprintln!("PID {pid}: {tested} tested, {skipped} skipped (known incomplete)");
    }
}

/// Discover generated fixture file for a PID (single file in `generated/` subdir).
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
