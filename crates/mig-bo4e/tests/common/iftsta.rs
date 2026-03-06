//! IFTSTA-specific test utilities for mig-bo4e integration tests.
//!
//! Two structural families:
//! - **Family A (MaBiS)**: PIDs 21000–21005, tx_group = "SG4" (EQD-initiated)
//! - **Family B (GPKE/WiM)**: PIDs 21007–21047, tx_group = "SG14" (CNI-initiated)

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

// ── Paths (relative to crate root = crates/mig-bo4e) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/IFTSTA_MIG_2_0f_Fehlerkorrektur_20250225.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/IFTSTA_AHB_2_0g_Fehlerkorrektur_20250225.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/IFTSTA/FV2504";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/IFTSTA";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/iftsta/pids";

// ── Family A: MaBiS (PIDs 21000–21005), tx_group = SG4 ──

const MABIS_CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "IFTSTA",
    variant: None,
    tx_group: "SG4",
    format_version: "FV2504",
};

// ── Family B: GPKE/WiM (PIDs 21007–21047), tx_group = SG14 ──

const GPKE_CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "IFTSTA",
    variant: None,
    tx_group: "SG14",
    format_version: "FV2504",
};

/// Family A PIDs (MaBiS): 21000–21005
pub const FAMILY_A_PIDS: &[&str] = &["21000", "21001", "21002", "21003", "21004", "21005"];

/// Family B PIDs (GPKE/WiM): 21007–21047
pub const FAMILY_B_PIDS: &[&str] = &[
    "21007", "21009", "21010", "21011", "21012", "21013", "21015", "21018", "21024", "21025",
    "21026", "21027", "21028", "21029", "21030", "21031", "21032", "21033", "21035", "21036",
    "21037", "21038", "21039", "21040", "21042", "21043", "21044", "21045", "21047",
];

fn config_for_pid(pid: &str) -> &'static MessageTypeConfig {
    if FAMILY_A_PIDS.contains(&pid) {
        &MABIS_CONFIG
    } else {
        &GPKE_CONFIG
    }
}

pub fn path_resolver() -> PathResolver {
    GPKE_CONFIG.path_resolver()
}

pub fn message_dir() -> PathBuf {
    GPKE_CONFIG.message_dir()
}

pub fn common_dir() -> PathBuf {
    GPKE_CONFIG.common_dir()
}

pub fn pid_dir(pid: &str) -> PathBuf {
    config_for_pid(pid).pid_dir(pid)
}

pub fn schema_index(pid: &str) -> PidSchemaIndex {
    config_for_pid(pid).schema_index(pid)
}

pub fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    config_for_pid(pid_id).load_pid_filtered_mig(pid_id)
}

pub fn discover_fixtures(pid: &str) -> Vec<PathBuf> {
    config_for_pid(pid).discover_fixtures(pid)
}

/// Discover generated fixture file for an IFTSTA PID.
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

pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    config_for_pid(pid).load_split_engines(pid)
}

pub fn load_message_engine() -> MappingEngine {
    GPKE_CONFIG.load_message_engine()
}

pub fn run_full_roundtrip(pid: &str) {
    config_for_pid(pid).run_full_roundtrip(pid);
}

pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    config_for_pid(pid).run_full_roundtrip_with_skip(pid, known_incomplete);
}
