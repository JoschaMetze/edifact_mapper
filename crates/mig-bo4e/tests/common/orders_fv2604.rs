//! ORDERS FV2604 test utilities.
//!
//! Mirrors FV2510 ORDERS config with FV2604 paths.
//! 50 PIDs for order messages. TX_GROUP = "SG29" (LIN-initiated positions).
//! UNS+S goes after tx group.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str = "../../xml-migs-and-ahbs/FV2604/ORDERS_MIG_1_4b_20250401.xml";
pub const AHB_XML_PATH: &str = "../../xml-migs-and-ahbs/FV2604/ORDERS_AHB_1_1a_20251001.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/ORDERS/FV2604";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2604/ORDERS";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2604/orders/pids";

/// ORDERS transaction group — SG29 (LIN-initiated positions).
pub const TX_GROUP: &str = "SG29";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "ORDERS",
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

/// Discover generated fixture file for an ORDERS PID.
pub fn discover_generated_fixture(pid: &str) -> Option<PathBuf> {
    let path = Path::new(FIXTURE_DIR).join(format!("generated/{pid}.edi"));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
