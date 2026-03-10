//! Test utilities for UTILMD Gas FV2604 message type.
//!
//! Mirrors the FV2510 UTILMD Gas config but points to FV2604 MIG/AHB XMLs,
//! schemas, and mappings.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2604/UTILMD_MIG_Gas_G1_1_au\u{00df}erordentliche_20251211.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2604/UTILMD_AHB_Gas_1_1_au\u{00df}erordentliche_20251211.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2604";
pub const GENERATED_FIXTURE_DIR: &str =
    "../../example_market_communication_bo4e_transactions/UTILMD/FV2604/generated_gas";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2604/UTILMD_Gas";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2604/utilmd/pids";

/// UTILMD Gas transaction group — SG4 (IDE-initiated transactions).
pub const TX_GROUP: &str = "SG4";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "UTILMD",
    variant: Some("Gas"),
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

pub fn discover_generated_fixture(pid: &str) -> Option<PathBuf> {
    let path = Path::new(GENERATED_FIXTURE_DIR).join(format!("{pid}.edi"));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    CONFIG.load_split_engines(pid)
}
