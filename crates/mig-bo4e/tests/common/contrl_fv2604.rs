//! CONTRL FV2604 test utilities.
//!
//! Mirrors FV2510 CONTRL config with FV2604 paths.
//! CONTRL has no TOML mappings subdirectory — files are directly in MAPPINGS_BASE.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2604/CONTRL_MIG_2_0b_au\u{00df}erordentliche_20251211.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2604/CONTRL_AHB_1_0_au\u{00df}erordentliche_20251211.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/CONTRL/FV2604";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2604/CONTRL";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2604/contrl/pids";

/// CONTRL has no transaction group — message-only.
pub const TX_GROUP: &str = "";

/// AHB PID is empty string (CONTRL has no real PID).
const PID_AHB: &str = "";

/// Fixture filenames use "91001" prefix by convention.
pub const PID_FIXTURE: &str = "91001";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "CONTRL",
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

pub fn load_message_engine() -> MappingEngine {
    // CONTRL has no message/ subdirectory — TOML files are directly in MAPPINGS_BASE
    let resolver = CONFIG.path_resolver();
    MappingEngine::load(Path::new(MAPPINGS_BASE))
        .unwrap()
        .with_path_resolver(resolver)
}

/// Discover generated fixture file.
pub fn discover_generated_fixture() -> Option<PathBuf> {
    // Try enhanced fixture first, then basic
    let enhanced = Path::new(FIXTURE_DIR).join("generated/.edi");
    if enhanced.exists() {
        return Some(enhanced);
    }
    let basic = Path::new(FIXTURE_DIR).join("generated/_CONTRL_generated.edi");
    if basic.exists() {
        Some(basic)
    } else {
        None
    }
}
