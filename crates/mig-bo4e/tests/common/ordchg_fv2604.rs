//! ORDCHG FV2604 test utilities.
//!
//! Mirrors FV2510 ORDCHG config with FV2604 paths.
//! 3 PIDs (39000-39002) for order change messages.
//! TX_GROUP = "" (message-only, no transaction groups).

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2604/ORDCHG_MIG_1_1_au\u{00df}erordentliche_20240726.xml";
pub const AHB_XML_PATH: &str = "../../xml-migs-and-ahbs/FV2604/ORDCHG_AHB_1_0a_20241001.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/ORDCHG/FV2604";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2604/ORDCHG";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2604/ordchg/pids";

/// ORDCHG has no transaction group — message-only PIDs.
pub const TX_GROUP: &str = "";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "ORDCHG",
    variant: None,
    tx_group: TX_GROUP,
    format_version: "FV2604",
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

pub fn load_message_engine() -> MappingEngine {
    CONFIG.load_message_engine()
}

pub fn run_full_roundtrip(pid: &str) {
    CONFIG.run_full_roundtrip(pid);
}

pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    CONFIG.run_full_roundtrip_with_skip(pid, known_incomplete);
}

/// Discover generated fixture file for an ORDCHG PID.
pub fn discover_generated_fixture(pid: &str) -> Option<PathBuf> {
    let path = Path::new(FIXTURE_DIR).join(format!("generated/{pid}.edi"));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
