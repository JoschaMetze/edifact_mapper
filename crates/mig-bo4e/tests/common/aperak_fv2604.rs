//! APERAK FV2604 test utilities.
//!
//! Mirrors FV2510 APERAK config with FV2604 paths.
//! APERAK has a single workflow with empty PID in the AHB, but fixtures
//! use "92001" prefix by convention.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str = "../../xml-migs-and-ahbs/FV2604/APERAK_MIG_2_1i_20240619.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2604/APERAK_AHB_1_0_Fehlerkorrektur_20250930.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/APERAK/FV2604";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2604/APERAK";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2604/aperak/pids";

/// APERAK has no transaction group — message-only.
pub const TX_GROUP: &str = "";

/// AHB PID is empty string (APERAK has no real PID).
const PID_AHB: &str = "";

/// Fixture filenames use "92001" prefix by convention.
pub const PID_FIXTURE: &str = "92001";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "APERAK",
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

pub fn schema_index() -> PidSchemaIndex {
    CONFIG.schema_index(PID_AHB)
}

pub fn load_pid_filtered_mig() -> Option<MigSchema> {
    CONFIG.load_pid_filtered_mig(PID_AHB)
}

pub fn discover_fixtures() -> Vec<PathBuf> {
    CONFIG.discover_fixtures(PID_FIXTURE)
}

pub fn load_message_engine() -> MappingEngine {
    CONFIG.load_message_engine()
}

/// Discover generated fixture file (FV2604 uses `_APERAK_generated.edi`).
pub fn discover_generated_fixture() -> Option<PathBuf> {
    let path = Path::new(FIXTURE_DIR).join("generated/_APERAK_generated.edi");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
