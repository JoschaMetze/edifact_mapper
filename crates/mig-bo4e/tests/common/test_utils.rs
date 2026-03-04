//! Shared test utilities for mig-bo4e integration tests.
//!
//! Consolidates constants, helper functions, and roundtrip logic used
//! across multiple test files.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::bo4e_validation;

// Re-export owned_to_assembled from mig_assembly for backward compatibility
pub use mig_assembly::assembler::owned_to_assembled;

// ── Generic message type configuration ──

/// Configuration for testing a specific EDIFACT message type (UTILMD, MSCONS, ORDERS, etc.).
///
/// Encapsulates all per-message-type constants and provides methods for
/// common test operations (fixture discovery, engine loading, roundtrip testing).
pub struct MessageTypeConfig {
    pub mig_xml_path: &'static str,
    pub ahb_xml_path: &'static str,
    pub fixture_dir: &'static str,
    pub mappings_base: &'static str,
    pub schema_dir: &'static str,
    pub message_type: &'static str,
    pub variant: Option<&'static str>,
    pub tx_group: &'static str,
}

impl MessageTypeConfig {
    pub fn path_resolver(&self) -> PathResolver {
        PathResolver::from_schema_dir(Path::new(self.schema_dir))
    }

    pub fn message_dir(&self) -> PathBuf {
        Path::new(self.mappings_base).join("message")
    }

    pub fn common_dir(&self) -> PathBuf {
        Path::new(self.mappings_base).join("common")
    }

    pub fn pid_dir(&self, pid: &str) -> PathBuf {
        Path::new(self.mappings_base).join(format!("pid_{pid}"))
    }

    pub fn schema_index(&self, pid: &str) -> PidSchemaIndex {
        let schema_path = Path::new(self.schema_dir).join(format!("pid_{pid}_schema.json"));
        PidSchemaIndex::from_schema_file(&schema_path).unwrap()
    }

    /// Parse MIG/AHB and return a PID-filtered MIG schema.
    pub fn load_pid_filtered_mig(&self, pid_id: &str) -> Option<MigSchema> {
        let mig_path = Path::new(self.mig_xml_path);
        let ahb_path = Path::new(self.ahb_xml_path);
        if !mig_path.exists() || !ahb_path.exists() {
            return None;
        }
        let mig = parse_mig(mig_path, self.message_type, self.variant, "FV2504").ok()?;
        let ahb = parse_ahb(ahb_path, self.message_type, self.variant, "FV2504").ok()?;
        let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
        let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
        Some(filter_mig_for_pid(&mig, &numbers))
    }

    /// Discover all `.edi` fixture files for a given PID from the example corpus.
    pub fn discover_fixtures(&self, pid: &str) -> Vec<PathBuf> {
        let dir = Path::new(self.fixture_dir);
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

    /// Load split engines with common/ inheritance when available.
    pub fn load_split_engines(&self, pid: &str) -> (MappingEngine, MappingEngine) {
        let msg_dir = self.message_dir();
        let cmn_dir = self.common_dir();
        let tx_dir = self.pid_dir(pid);
        let resolver = self.path_resolver();
        if cmn_dir.exists() {
            let idx = self.schema_index(pid);
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

    /// Load message-only engine (for PIDs without a transaction group).
    pub fn load_message_engine(&self) -> MappingEngine {
        let msg_dir = self.message_dir();
        let resolver = self.path_resolver();
        MappingEngine::load(&msg_dir)
            .unwrap()
            .with_path_resolver(resolver)
    }

    /// Full pipeline roundtrip for a PID, testing ALL available fixtures.
    pub fn run_full_roundtrip(&self, pid: &str) {
        self.run_full_roundtrip_with_skip(pid, &[]);
    }

    /// Full pipeline roundtrip with an explicit list of fixture filenames to skip.
    pub fn run_full_roundtrip_with_skip(&self, pid: &str, known_incomplete: &[&str]) {
        let fixtures = self.discover_fixtures(pid);
        if fixtures.is_empty() {
            eprintln!("Skipping roundtrip for PID {pid}: no fixtures found");
            return;
        }

        let Some(filtered_mig) = self.load_pid_filtered_mig(pid) else {
            eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
            return;
        };

        let tx_dir = self.pid_dir(pid);
        if !self.message_dir().exists() {
            eprintln!("Skipping roundtrip for PID {pid}: message directory not found");
            return;
        }

        // For PIDs without a tx dir, use an empty tx engine
        let (msg_engine, tx_engine) = if tx_dir.exists() {
            self.load_split_engines(pid)
        } else {
            let msg = self.load_message_engine();
            let tx = MappingEngine::from_definitions(vec![]);
            (msg, tx)
        };

        let mut tested = 0;
        let mut skipped = 0;

        for fixture_path in &fixtures {
            let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();

            if known_incomplete.contains(&fixture_name) {
                eprintln!("PID {pid}: {fixture_name} -- SKIPPED (known incomplete mapping)");
                skipped += 1;
                continue;
            }

            run_single_fixture_roundtrip_with_tx_group(
                pid,
                fixture_path,
                &filtered_mig,
                &msg_engine,
                &tx_engine,
                self.tx_group,
            );
            tested += 1;
        }

        eprintln!("PID {pid}: {tested} fixtures passed, {skipped} skipped (known incomplete)");
    }
}

// ── UTILMD-specific constants (backward compatibility) ──

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
pub const GENERATED_FIXTURE_DIR: &str = "../../fixtures/generated/fv2504/utilmd";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

const UTILMD_CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "UTILMD",
    variant: Some("Strom"),
    tx_group: "SG4",
};

pub fn path_resolver() -> PathResolver {
    UTILMD_CONFIG.path_resolver()
}

pub fn message_dir() -> PathBuf {
    UTILMD_CONFIG.message_dir()
}

pub fn common_dir() -> PathBuf {
    UTILMD_CONFIG.common_dir()
}

pub fn pid_dir(pid: &str) -> PathBuf {
    UTILMD_CONFIG.pid_dir(pid)
}

pub fn schema_index(pid: &str) -> PidSchemaIndex {
    UTILMD_CONFIG.schema_index(pid)
}

pub fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    UTILMD_CONFIG.load_pid_filtered_mig(pid_id)
}

pub fn discover_fixtures(pid: &str) -> Vec<PathBuf> {
    UTILMD_CONFIG.discover_fixtures(pid)
}

/// Discover generated fixture file for a PID (single file: `{pid}.edi`).
pub fn discover_generated_fixture(pid: &str) -> Option<PathBuf> {
    let path = Path::new(GENERATED_FIXTURE_DIR).join(format!("{pid}.edi"));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    UTILMD_CONFIG.load_split_engines(pid)
}

pub fn run_full_roundtrip(pid: &str) {
    UTILMD_CONFIG.run_full_roundtrip(pid);
}

pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    UTILMD_CONFIG.run_full_roundtrip_with_skip(pid, known_incomplete);
}

/// Run the roundtrip pipeline for a single fixture file.
pub fn run_single_fixture_roundtrip(
    pid: &str,
    fixture_path: &Path,
    filtered_mig: &MigSchema,
    msg_engine: &MappingEngine,
    tx_engine: &MappingEngine,
) {
    run_single_fixture_roundtrip_with_tx_group(
        pid,
        fixture_path,
        filtered_mig,
        msg_engine,
        tx_engine,
        "SG4",
    );
}

// ── Shared roundtrip pipeline ──

/// Run the roundtrip pipeline for a single fixture file with a configurable transaction group.
///
/// EDIFACT → tokenize → split → assemble → map_interchange
/// → map_interchange_reverse → disassemble → render → compare with original.
pub fn run_single_fixture_roundtrip_with_tx_group(
    pid: &str,
    fixture_path: &Path,
    filtered_mig: &MigSchema,
    msg_engine: &MappingEngine,
    tx_engine: &MappingEngine,
    tx_group: &str,
) {
    let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();

    let edifact_input = std::fs::read_to_string(fixture_path).unwrap();

    // Step 1: Tokenize and split
    let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
    let chunks = split_messages(segments).unwrap();
    assert!(
        !chunks.messages.is_empty(),
        "PID {pid} ({fixture_name}): should have at least one message"
    );

    let msg_chunk = &chunks.messages[0];

    // Assemble with UNH + body + UNT only (no UNB envelope).
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    // Step 2: Assemble with PID-filtered MIG
    let assembler = Assembler::new(filtered_mig);
    let mut original_tree = assembler.assemble_generic(&msg_segs).unwrap();

    // Step 3: Forward mapping → MappedMessage
    let mapped =
        MappingEngine::map_interchange(msg_engine, tx_engine, &original_tree, tx_group, true);

    // Only assert non-empty transactions if the tree has the tx group
    let tree_has_tx_group = original_tree.groups.iter().any(|g| g.group_id == tx_group);
    if tree_has_tx_group {
        assert!(
            !mapped.transaktionen.is_empty(),
            "PID {pid} ({fixture_name}): forward mapping should produce at least one transaction"
        );
    }

    // Step 3b: BO4E schema validation (non-fatal — warns about unknown field names)
    let mapped_for_validation =
        MappingEngine::map_interchange(msg_engine, tx_engine, &original_tree, tx_group, false);
    bo4e_validation::validate_mapped_message(
        pid,
        fixture_name,
        msg_engine,
        tx_engine,
        &mapped_for_validation,
    );

    // Step 4: Reverse mapping → AssembledTree (content only, no UNH/UNT)
    let mut reverse_tree =
        MappingEngine::map_interchange_reverse(msg_engine, tx_engine, &mapped, tx_group);

    // Add UNH to the front of pre-group segments, UNT to post-group.
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    // UNT may end up in inter_group_segments (when trailing after the last group).
    // Strip UNT from inter_group_segments in both trees to avoid duplication,
    // and always add it as a root segment.
    for segs in original_tree.inter_group_segments.values_mut() {
        segs.retain(|s| s.tag != "UNT");
    }
    let original_has_unt_in_root =
        original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if !original_has_unt_in_root {
        let unt_assembled_orig = owned_to_assembled(&msg_chunk.unt);
        original_tree.segments.push(unt_assembled_orig);
    }

    for segs in reverse_tree.inter_group_segments.values_mut() {
        segs.retain(|s| s.tag != "UNT");
    }
    let unt_assembled = owned_to_assembled(&msg_chunk.unt);
    reverse_tree.segments.push(unt_assembled);

    // Step 5: Disassemble both trees and render
    let disassembler = Disassembler::new(filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let original_dis = disassembler.disassemble(&original_tree);
    let original_rendered = render_edifact(&original_dis, &delimiters);

    let reverse_dis = disassembler.disassemble(&reverse_tree);
    let reverse_rendered = render_edifact(&reverse_dis, &delimiters);

    // Step 6: Compare segment tags (structural check)
    let original_tags: Vec<&str> = original_dis.iter().map(|s| s.tag.as_str()).collect();
    let reverse_tags: Vec<&str> = reverse_dis.iter().map(|s| s.tag.as_str()).collect();

    if original_tags != reverse_tags {
        eprintln!("PID {pid} ({fixture_name}): segment tag mismatch!");
        eprintln!(
            "  original ({} segs): {:?}",
            original_tags.len(),
            original_tags
        );
        eprintln!(
            "  reversed ({} segs): {:?}",
            reverse_tags.len(),
            reverse_tags
        );
        for (i, tag) in original_tags.iter().enumerate() {
            if reverse_tags.get(i) != Some(tag) {
                eprintln!(
                    "  first difference at position {i}: original={tag}, reversed={}",
                    reverse_tags.get(i).unwrap_or(&"<missing>")
                );
                break;
            }
        }
    }

    assert_eq!(
        original_tags, reverse_tags,
        "PID {pid} ({fixture_name}): segment tags should match after forward→reverse roundtrip"
    );

    // Step 7: Compare full rendered EDIFACT (byte-identical check)
    if original_rendered != reverse_rendered {
        let orig_segs: Vec<&str> = original_rendered.split('\'').collect();
        let rev_segs: Vec<&str> = reverse_rendered.split('\'').collect();
        let max_len = orig_segs.len().max(rev_segs.len());
        let mut diff_count = 0;
        for i in 0..max_len {
            let o = orig_segs.get(i).unwrap_or(&"<missing>");
            let r = rev_segs.get(i).unwrap_or(&"<missing>");
            if o != r {
                eprintln!("PID {pid} ({fixture_name}): segment {i} differs:");
                eprintln!("  original: {o}");
                eprintln!("  reversed: {r}");
                diff_count += 1;
            }
        }
        eprintln!(
            "PID {pid} ({fixture_name}): {diff_count} segment(s) differ out of {}",
            orig_segs.len()
        );
    }

    assert_eq!(
        original_rendered, reverse_rendered,
        "PID {pid} ({fixture_name}): full EDIFACT roundtrip should be byte-identical"
    );

    eprintln!(
        "PID {pid}: {fixture_name} -- roundtrip OK, {} segments byte-identical",
        original_tags.len()
    );
}
