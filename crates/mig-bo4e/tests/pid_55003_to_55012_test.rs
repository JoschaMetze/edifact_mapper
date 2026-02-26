//! Integration tests for PIDs 55003-55012 TOML mapping definitions.
//!
//! Two test categories:
//! 1. **TOML loading**: Verify `MappingEngine::load_merged()` succeeds for each PID.
//! 2. **Forward mapping**: For each PID with a fixture, run the full EDIFACT->BO4E pipeline.
//!
//! Tests skip gracefully when required files (MIG/AHB XML, fixtures, mappings) are absent.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::assembler::{AssembledSegment, Assembler};
use mig_assembly::disassembler::Disassembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── Paths (relative to the crate root = crates/mig-bo4e) ──

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(std::path::Path::new(SCHEMA_DIR))
}

// ── PID definitions ──

struct PidTestSpec {
    pid: &'static str,
    /// First fixture filename (relative to FIXTURE_DIR). We pick one per PID.
    fixture: &'static str,
    /// Entity keys expected in the transaction stammdaten (after camelCase conversion).
    /// These are entities OTHER than "prozessdaten" and "nachricht" which go to transaktionsdaten.
    tx_stammdaten_keys: &'static [&'static str],
    /// Entity keys expected in transaktionsdaten.
    tx_transaktionsdaten_keys: &'static [&'static str],
}

/// All PIDs 55003-55012 with their expected entity keys.
///
/// Entity placement:
/// - "prozessdaten" and "nachricht" -> transaktionsdaten
/// - everything else -> tx stammdaten
/// - message-level "marktteilnehmer", "nachricht", "kontakt" -> message stammdaten
const PID_SPECS: &[PidTestSpec] = &[
    PidTestSpec {
        pid: "55003",
        fixture: "55003_UTILMD_S2.1_ALEXANDE115345.edi",
        tx_stammdaten_keys: &["geschaeftspartner"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55004",
        fixture: "55004_UTILMD_S2.1_ALEXANDE828552.edi",
        // Fixture only has LOC+Z16 (Marktlokation) — no Z21/Z22 segments
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55005",
        fixture: "55005_UTILMD_S2.1_ALEXANDE862054.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55006",
        fixture: "55006_UTILMD_S2.1_ALEXANDE376464.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55007",
        fixture: "55007_UTILMD_S2.1_ALEXANDE453458.edi",
        // Fixture only has LOC+Z16 (Marktlokation) — no Z21/Z22 segments
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55008",
        fixture: "55008_UTILMD_S2.1_ALEXANDE442150.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55009",
        fixture: "55009_UTILMD_S2.1_ALEXANDE461768.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55010",
        fixture: "55010_UTILMD_S2.1_ALEXANDE551604.edi",
        // Fixture has LOC+Z16 + NAD+Z09 + NAD+VY — no LOC+Z21/Z22
        tx_stammdaten_keys: &["marktlokation", "geschaeftspartner", "ansprechpartner"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55011",
        fixture: "55011_UTILMD_S2.1_ALEXANDE550930.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55012",
        fixture: "55012_UTILMD_S2.1_ALEXANDE631821.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
];

// ── Helper functions ──

fn message_dir() -> PathBuf {
    Path::new(MAPPINGS_BASE).join("message")
}

fn pid_dir(pid: &str) -> PathBuf {
    Path::new(MAPPINGS_BASE).join(format!("pid_{pid}"))
}

// ── TOML loading tests (no fixtures needed) ──

macro_rules! toml_loading_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let msg_dir = message_dir();
            let tx_dir = pid_dir($pid);
            if !msg_dir.exists() {
                eprintln!("Skipping {}: message mappings not found", stringify!($name));
                return;
            }
            if !tx_dir.exists() {
                eprintln!(
                    "Skipping {}: PID {} mappings not found",
                    stringify!($name),
                    $pid
                );
                return;
            }

            let engine = MappingEngine::load_merged(&[&msg_dir, &tx_dir])
                .map(|e| e.with_path_resolver(path_resolver()))
                .unwrap_or_else(|e| panic!("Failed to load merged engine for PID {}: {e}", $pid));

            assert!(
                !engine.definitions().is_empty(),
                "PID {} merged engine should have non-empty definitions",
                $pid
            );

            eprintln!(
                "PID {} TOML loading OK: {} definitions loaded",
                $pid,
                engine.definitions().len()
            );
        }
    };
}

toml_loading_test!(test_toml_loading_55003, "55003");
toml_loading_test!(test_toml_loading_55004, "55004");
toml_loading_test!(test_toml_loading_55005, "55005");
toml_loading_test!(test_toml_loading_55006, "55006");
toml_loading_test!(test_toml_loading_55007, "55007");
toml_loading_test!(test_toml_loading_55008, "55008");
toml_loading_test!(test_toml_loading_55009, "55009");
toml_loading_test!(test_toml_loading_55010, "55010");
toml_loading_test!(test_toml_loading_55011, "55011");
toml_loading_test!(test_toml_loading_55012, "55012");

// ── Forward mapping tests (need fixtures + MIG/AHB XML) ──

/// Run the full forward mapping pipeline for a single PID spec.
fn run_forward_mapping_test(spec: &PidTestSpec) {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        eprintln!(
            "Skipping forward test for PID {}: MIG/AHB XML not found",
            spec.pid
        );
        return;
    }

    let fixture_path = Path::new(FIXTURE_DIR).join(spec.fixture);
    if !fixture_path.exists() {
        eprintln!(
            "Skipping forward test for PID {}: fixture not found at {}",
            spec.pid,
            fixture_path.display()
        );
        return;
    }

    let msg_dir = message_dir();
    let tx_dir = pid_dir(spec.pid);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!(
            "Skipping forward test for PID {}: mappings not found",
            spec.pid
        );
        return;
    }

    // Step 1: Parse EDIFACT fixture
    let content = std::fs::read(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture for PID {}: {e}", spec.pid));
    let segments = parse_to_segments(&content)
        .unwrap_or_else(|e| panic!("Failed to parse EDIFACT for PID {}: {e}", spec.pid));

    // Step 2: Split messages
    let chunks = mig_assembly::split_messages(segments)
        .unwrap_or_else(|e| panic!("Failed to split messages for PID {}: {e}", spec.pid));
    assert!(
        !chunks.messages.is_empty(),
        "PID {}: fixture should contain at least one message",
        spec.pid
    );

    // Step 3: Detect PID from the first message's body segments
    let msg = &chunks.messages[0];
    let detected_pid = mig_assembly::pid_detect::detect_pid(&msg.body).unwrap_or_else(|e| {
        panic!(
            "Failed to detect PID from fixture for PID {}: {e}",
            spec.pid
        )
    });
    assert_eq!(
        detected_pid, spec.pid,
        "Detected PID should match expected PID"
    );

    // Step 4: Load AHB, filter MIG for this PID
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504")
        .unwrap_or_else(|e| panic!("Failed to parse MIG XML for PID {}: {e}", spec.pid));
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504")
        .unwrap_or_else(|e| panic!("Failed to parse AHB XML for PID {}: {e}", spec.pid));
    let pid_workflow = ahb
        .workflows
        .iter()
        .find(|w| w.id == spec.pid)
        .unwrap_or_else(|| panic!("AHB should contain workflow for PID {}", spec.pid));
    let numbers: HashSet<String> = pid_workflow.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(&mig, &numbers);

    // Step 5: Assemble the message with PID-filtered MIG
    let all_segments = msg.all_segments();
    let assembler = Assembler::new(&filtered_mig);
    let tree = assembler
        .assemble_generic(&all_segments)
        .unwrap_or_else(|e| panic!("Failed to assemble tree for PID {}: {e}", spec.pid));

    // Step 6: Load split engines (message + transaction)
    let (msg_engine, tx_engine) = MappingEngine::load_split(&msg_dir, &tx_dir)
        .unwrap_or_else(|e| panic!("Failed to load split engines for PID {}: {e}", spec.pid));
    let resolver = path_resolver();
    let msg_engine = msg_engine.with_path_resolver(resolver.clone());
    let tx_engine = tx_engine.with_path_resolver(resolver);

    // Step 7: Map with split engines (no code enrichment for basic test)
    let mapped = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", false);

    // Verify message-level stammdaten has marktteilnehmer
    let msg_stammdaten = mapped.stammdaten.as_object().unwrap_or_else(|| {
        panic!(
            "PID {}: message stammdaten should be a JSON object",
            spec.pid
        )
    });
    assert!(
        msg_stammdaten.contains_key("marktteilnehmer"),
        "PID {}: message stammdaten should contain 'marktteilnehmer'",
        spec.pid
    );

    // Verify at least one transaction
    assert!(
        !mapped.transaktionen.is_empty(),
        "PID {}: should have at least one transaction",
        spec.pid
    );

    let tx = &mapped.transaktionen[0];

    // Verify transaktionsdaten has expected keys
    if !tx.transaktionsdaten.is_null() {
        for key in spec.tx_transaktionsdaten_keys {
            assert!(
                tx.transaktionsdaten.get(key).is_some(),
                "PID {}: transaktionsdaten should contain '{key}'",
                spec.pid
            );
        }
    } else if !spec.tx_transaktionsdaten_keys.is_empty() {
        panic!(
            "PID {}: transaktionsdaten is null but expected keys: {:?}",
            spec.pid, spec.tx_transaktionsdaten_keys
        );
    }

    // Verify transaction stammdaten has expected entity keys
    let tx_stamm = tx.stammdaten.as_object().unwrap_or_else(|| {
        panic!(
            "PID {}: transaction stammdaten should be a JSON object",
            spec.pid
        )
    });
    for key in spec.tx_stammdaten_keys {
        assert!(
            tx_stamm.contains_key(*key),
            "PID {}: transaction stammdaten should contain '{key}', got keys: {:?}",
            spec.pid,
            tx_stamm.keys().collect::<Vec<_>>()
        );
    }

    // Print summary for debugging
    let tx_keys: Vec<&String> = tx_stamm.keys().collect();
    let transaktionsdaten_keys: Vec<&str> = if let Some(obj) = tx.transaktionsdaten.as_object() {
        obj.keys().map(|s| s.as_str()).collect()
    } else {
        vec![]
    };
    eprintln!(
        "PID {} forward mapping OK: msg_stammdaten keys={:?}, tx_stammdaten keys={:?}, transaktionsdaten keys={:?}",
        spec.pid,
        msg_stammdaten.keys().collect::<Vec<_>>(),
        tx_keys,
        transaktionsdaten_keys
    );
}

macro_rules! forward_mapping_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let spec = PID_SPECS
                .iter()
                .find(|s| s.pid == $pid)
                .unwrap_or_else(|| panic!("No PidTestSpec defined for PID {}", $pid));
            run_forward_mapping_test(spec);
        }
    };
}

forward_mapping_test!(test_forward_mapping_55003, "55003");
forward_mapping_test!(test_forward_mapping_55004, "55004");
forward_mapping_test!(test_forward_mapping_55005, "55005");
forward_mapping_test!(test_forward_mapping_55006, "55006");
forward_mapping_test!(test_forward_mapping_55007, "55007");
forward_mapping_test!(test_forward_mapping_55008, "55008");
forward_mapping_test!(test_forward_mapping_55009, "55009");
forward_mapping_test!(test_forward_mapping_55010, "55010");
forward_mapping_test!(test_forward_mapping_55011, "55011");
forward_mapping_test!(test_forward_mapping_55012, "55012");

// ── Interchange-level integration test (builds full Interchange struct) ──

macro_rules! interchange_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let spec = PID_SPECS
                .iter()
                .find(|s| s.pid == $pid)
                .unwrap_or_else(|| panic!("No PidTestSpec defined for PID {}", $pid));

            let mig_path = Path::new(MIG_XML_PATH);
            let ahb_path = Path::new(AHB_XML_PATH);
            if !mig_path.exists() || !ahb_path.exists() {
                eprintln!(
                    "Skipping interchange test for PID {}: MIG/AHB XML not found",
                    spec.pid
                );
                return;
            }

            let fixture_path = Path::new(FIXTURE_DIR).join(spec.fixture);
            if !fixture_path.exists() {
                eprintln!(
                    "Skipping interchange test for PID {}: fixture not found",
                    spec.pid
                );
                return;
            }

            let msg_dir = message_dir();
            let tx_dir = pid_dir(spec.pid);
            if !msg_dir.exists() || !tx_dir.exists() {
                eprintln!(
                    "Skipping interchange test for PID {}: mappings not found",
                    spec.pid
                );
                return;
            }

            // Parse and split
            let content = std::fs::read(&fixture_path).unwrap();
            let segments = parse_to_segments(&content).unwrap();
            let chunks = mig_assembly::split_messages(segments).unwrap();
            let msg = &chunks.messages[0];

            // Extract UNH fields and nachrichtendaten
            let (unh_ref, msg_type) = mig_bo4e::model::extract_unh_fields(&msg.unh);
            assert!(
                !unh_ref.is_empty(),
                "PID {}: UNH reference should not be empty",
                spec.pid
            );
            assert_eq!(
                msg_type, "UTILMD",
                "PID {}: message type should be UTILMD",
                spec.pid
            );

            let nd = mig_bo4e::model::extract_nachrichtendaten(&chunks.envelope);
            assert!(
                nd.get("absenderCode").is_some(),
                "PID {}: should extract sender from UNB",
                spec.pid
            );

            // Load MIG, filter, assemble
            let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
            let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
            let pid_workflow = ahb.workflows.iter().find(|w| w.id == spec.pid).unwrap();
            let numbers: HashSet<String> = pid_workflow.segment_numbers.iter().cloned().collect();
            let filtered_mig = filter_mig_for_pid(&mig, &numbers);

            let all_segments = msg.all_segments();
            let tree = Assembler::new(&filtered_mig)
                .assemble_generic(&all_segments)
                .unwrap();

            // Map
            let (msg_engine, tx_engine) = MappingEngine::load_split(&msg_dir, &tx_dir).unwrap();
            let resolver = path_resolver();
            let msg_engine = msg_engine.with_path_resolver(resolver.clone());
            let tx_engine = tx_engine.with_path_resolver(resolver);
            let mapped =
                MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", false);

            // Build full Interchange
            let interchange = mig_bo4e::Interchange {
                nachrichtendaten: nd,
                nachrichten: vec![mig_bo4e::Nachricht {
                    unh_referenz: unh_ref,
                    nachrichten_typ: msg_type,
                    stammdaten: mapped.stammdaten,
                    transaktionen: mapped.transaktionen,
                }],
            };

            // Verify JSON serialization roundtrip
            let json = serde_json::to_string_pretty(&interchange).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(parsed["nachrichten"].is_array());
            assert_eq!(parsed["nachrichten"].as_array().unwrap().len(), 1);
            assert_eq!(
                parsed["nachrichten"][0]["nachrichtenTyp"].as_str().unwrap(),
                "UTILMD"
            );
            assert!(
                parsed["nachrichten"][0]["transaktionen"].is_array(),
                "PID {}: nachrichten[0] should have transaktionen array",
                spec.pid
            );
            assert!(
                !parsed["nachrichten"][0]["transaktionen"]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                "PID {}: transaktionen should not be empty",
                spec.pid
            );

            eprintln!(
                "PID {} interchange test OK: {} bytes JSON",
                spec.pid,
                json.len()
            );
        }
    };
}

interchange_test!(test_interchange_55003, "55003");
interchange_test!(test_interchange_55004, "55004");
interchange_test!(test_interchange_55005, "55005");
interchange_test!(test_interchange_55006, "55006");
interchange_test!(test_interchange_55007, "55007");
interchange_test!(test_interchange_55008, "55008");
interchange_test!(test_interchange_55009, "55009");
interchange_test!(test_interchange_55010, "55010");
interchange_test!(test_interchange_55011, "55011");
interchange_test!(test_interchange_55012, "55012");

// ── Full EDIFACT roundtrip tests ──

/// Discover all `.edi` fixture files for a given PID.
fn discover_fixtures(pid: &str) -> Vec<PathBuf> {
    let dir = Path::new(FIXTURE_DIR);
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

fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        return None;
    }
    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let ahb = parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").ok()?;
    let pid = ahb.workflows.iter().find(|w| w.id == pid_id)?;
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    Some(filter_mig_for_pid(&mig, &numbers))
}

fn owned_to_assembled(seg: &mig_assembly::tokenize::OwnedSegment) -> AssembledSegment {
    AssembledSegment {
        tag: seg.id.clone(),
        elements: seg
            .elements
            .iter()
            .map(|el| el.iter().map(|c| c.to_string()).collect())
            .collect(),
    }
}

/// Full pipeline roundtrip for ALL fixtures of a PID:
/// EDIFACT -> tokenize -> split -> assemble -> map_interchange
/// -> map_interchange_reverse -> disassemble -> render -> compare with original.
fn run_full_roundtrip(pid: &str) {
    let fixtures = discover_fixtures(pid);
    if fixtures.is_empty() {
        eprintln!("Skipping roundtrip for PID {pid}: no fixtures found");
        return;
    }

    let Some(filtered_mig) = load_pid_filtered_mig(pid) else {
        eprintln!("Skipping roundtrip for PID {pid}: MIG/AHB XML not available");
        return;
    };

    let msg_dir = message_dir();
    let tx_dir = pid_dir(pid);
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping roundtrip for PID {pid}: mapping directories not found");
        return;
    }
    let msg_engine = MappingEngine::load(&msg_dir)
        .unwrap()
        .with_path_resolver(path_resolver());
    let tx_engine = MappingEngine::load(&tx_dir)
        .unwrap()
        .with_path_resolver(path_resolver());

    for fixture_path in &fixtures {
        let fixture_name = fixture_path.file_name().unwrap().to_str().unwrap();
        let edifact_input = std::fs::read_to_string(fixture_path).unwrap();

        // Step 1: Tokenize and split
        let segments = parse_to_segments(edifact_input.as_bytes()).unwrap();
        let chunks = mig_assembly::split_messages(segments).unwrap();
        assert!(
            !chunks.messages.is_empty(),
            "PID {pid} ({fixture_name}): should have at least one message"
        );

        let msg_chunk = &chunks.messages[0];

        // Assemble with UNH + body + UNT (message content, no UNB envelope)
        let mut msg_segs = vec![msg_chunk.unh.clone()];
        msg_segs.extend(msg_chunk.body.iter().cloned());
        msg_segs.push(msg_chunk.unt.clone());

        // Step 2: Assemble with PID-filtered MIG
        let assembler = Assembler::new(&filtered_mig);
        let original_tree = assembler.assemble_generic(&msg_segs).unwrap();

        // Step 3: Forward mapping -> MappedMessage
        let mapped =
            MappingEngine::map_interchange(&msg_engine, &tx_engine, &original_tree, "SG4", true);

        assert!(
            !mapped.transaktionen.is_empty(),
            "PID {pid} ({fixture_name}): forward mapping should produce at least one transaction"
        );

        // Step 4: Reverse mapping -> AssembledTree
        let mut reverse_tree =
            MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4");

        // Add UNH envelope (mapping engine handles content only)
        let unh_assembled = owned_to_assembled(&msg_chunk.unh);
        reverse_tree.segments.insert(0, unh_assembled);
        reverse_tree.post_group_start += 1;

        // Only add UNT if the assembler captured it in the original tree.
        let original_has_unt =
            original_tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
        if original_has_unt {
            let unt_assembled = owned_to_assembled(&msg_chunk.unt);
            reverse_tree.segments.push(unt_assembled);
        }

        // Step 5: Disassemble both trees and render to EDIFACT
        let disassembler = Disassembler::new(&filtered_mig);
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
            "PID {pid} ({fixture_name}): segment tags should match after forward->reverse roundtrip"
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

    eprintln!(
        "PID {pid}: all {} fixtures passed",
        fixtures.len()
    );
}

macro_rules! roundtrip_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            run_full_roundtrip($pid);
        }
    };
}

roundtrip_test!(test_roundtrip_55003, "55003");
roundtrip_test!(test_roundtrip_55004, "55004");
roundtrip_test!(test_roundtrip_55005, "55005");
roundtrip_test!(test_roundtrip_55006, "55006");
roundtrip_test!(test_roundtrip_55007, "55007");
roundtrip_test!(test_roundtrip_55008, "55008");
roundtrip_test!(test_roundtrip_55009, "55009");
roundtrip_test!(test_roundtrip_55010, "55010");
roundtrip_test!(test_roundtrip_55011, "55011");
roundtrip_test!(test_roundtrip_55012, "55012");
