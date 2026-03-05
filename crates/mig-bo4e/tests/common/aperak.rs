//! APERAK-specific test utilities for mig-bo4e integration tests.
//!
//! APERAK has a single workflow with empty PID in the AHB, but fixtures
//! use "92001" prefix by convention. Custom helpers bridge the gap.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str = "../../xml-migs-and-ahbs/FV2504/APERAK_MIG_2_1i_20240619.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/APERAK_AHB_2_4a_Fehlerkorrektur_20250331.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/APERAK/FV2504";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/APERAK";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/aperak/pids";

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

/// Discover generated fixture file (single file in `generated/` subdir).
pub fn discover_generated_fixture() -> Option<PathBuf> {
    let dir = Path::new(FIXTURE_DIR).join("generated");
    let prefix = format!("{PID_FIXTURE}_");
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
