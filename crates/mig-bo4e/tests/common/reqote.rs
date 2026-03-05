//! REQOTE-specific test utilities for mig-bo4e integration tests.
//!
//! Thin wrapper around `MessageTypeConfig` with REQOTE-specific constants.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/REQOTE_MIG_1_3b_20241001.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/REQOTE_AHB_1_0a_Fehlerkorrektur_20250225.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/REQOTE/FV2504";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/REQOTE";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/reqote/pids";

/// REQOTE transaction group — SG27 (LIN-initiated positions).
pub const TX_GROUP: &str = "SG27";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "REQOTE",
    variant: None,
    tx_group: TX_GROUP,
};

pub fn path_resolver() -> PathResolver {
    CONFIG.path_resolver()
}

pub fn message_dir() -> PathBuf {
    CONFIG.message_dir()
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

pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    CONFIG.load_split_engines(pid)
}

pub fn load_message_engine() -> MappingEngine {
    CONFIG.load_message_engine()
}

pub fn run_full_roundtrip(pid: &str) {
    CONFIG.run_full_roundtrip(pid);
}

pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    CONFIG.run_full_roundtrip_with_skip(pid, known_incomplete);
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
