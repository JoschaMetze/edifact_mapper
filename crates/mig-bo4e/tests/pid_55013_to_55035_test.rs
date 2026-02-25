//! Integration tests for PIDs 55013-55035 TOML mapping definitions.
//!
//! Two test categories:
//! 1. **TOML loading**: Verify `MappingEngine::load_merged()` succeeds for each PID.
//! 2. **Forward mapping**: For each PID with a fixture, run the full EDIFACT->BO4E pipeline.
//!
//! Tests skip gracefully when required files (MIG/AHB XML, fixtures, mappings) are absent.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use mig_assembly::assembler::Assembler;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── Paths (relative to the crate root = crates/mig-bo4e) ──

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";

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

/// All PIDs 55013-55035 with their expected entity keys.
///
/// Entity placement:
/// - "prozessdaten" and "nachricht" -> transaktionsdaten
/// - everything else -> tx stammdaten
/// - message-level "marktteilnehmer", "nachricht", "kontakt" -> message stammdaten
///
/// PIDs 55022, 55023, 55024 have no fixture files — forward/interchange tests skip gracefully.
const PID_SPECS: &[PidTestSpec] = &[
    PidTestSpec {
        pid: "55013",
        fixture: "55013_UTILMD_S2.1_ALEXANDE982717998.edi",
        // Anmeldung/Zuordnung EOG: LOC+Z16,Z17,Z18,Z19,Z20 + NAD+Z63-Z70 + SEQ groups
        tx_stammdaten_keys: &[
            "marktlokation",
            "messlokation",
            "netzlokation",
            "steuerbareRessource",
            "technischeRessource",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55014",
        fixture: "55014_UTILMD_S2.1_ALEXANDE948259148.edi",
        // Bestaetigung Anmeldung: NAD+Z09,Z04 + SEQ+Z79,ZH0,Z01,Z75
        tx_stammdaten_keys: &[
            "ansprechpartner",
            "enfgDaten",
            "geschaeftspartner",
            "marktlokation",
            "produktpaket",
            "produktpaketPriorisierung",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55015",
        fixture: "55015_UTILMD_S2.1_ALEXANDE665361172.edi",
        // Ablehnung: no LOC/NAD in SG4
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55016",
        fixture: "55016_UTILMD_S2.1_ALEXANDE616133.edi",
        // Abmeldung: LOC+Z16 (Marktlokation)
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55017",
        fixture: "55017_UTILMD_S2.1_ALEXANDE107081.edi",
        // Bestaetigung Abmeldung: SEQ+Z01 → Marktlokation enrichment
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55018",
        fixture: "55018_UTILMD_S2.1_ALEXANDE203211.edi",
        // Ablehnung Abmeldung: no LOC/NAD in SG4
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55022",
        // No fixture file available — forward/interchange tests skip
        fixture: "",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55023",
        // No fixture file available — forward/interchange tests skip
        fixture: "",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55024",
        // No fixture file available — forward/interchange tests skip
        fixture: "",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55035",
        fixture: "55035_UTILMD_S2.1_ALEXANDE195836.edi",
        // Stammdatenaenderung: LOC+Z16,Z17,Z18,Z19,Z20,Z22 + NAD+Z63 + many SEQ groups
        tx_stammdaten_keys: &[
            "marktlokation",
            "messlokation",
            "netzlokation",
            "ruhendeMarktlokation",
            "steuerbareRessource",
            "technischeRessource",
        ],
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

toml_loading_test!(test_toml_loading_55013, "55013");
toml_loading_test!(test_toml_loading_55014, "55014");
toml_loading_test!(test_toml_loading_55015, "55015");
toml_loading_test!(test_toml_loading_55016, "55016");
toml_loading_test!(test_toml_loading_55017, "55017");
toml_loading_test!(test_toml_loading_55018, "55018");
toml_loading_test!(test_toml_loading_55022, "55022");
toml_loading_test!(test_toml_loading_55023, "55023");
toml_loading_test!(test_toml_loading_55024, "55024");
toml_loading_test!(test_toml_loading_55035, "55035");

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

    if spec.fixture.is_empty() {
        eprintln!(
            "Skipping forward test for PID {}: no fixture defined",
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

forward_mapping_test!(test_forward_mapping_55013, "55013");
forward_mapping_test!(test_forward_mapping_55014, "55014");
forward_mapping_test!(test_forward_mapping_55015, "55015");
forward_mapping_test!(test_forward_mapping_55016, "55016");
forward_mapping_test!(test_forward_mapping_55017, "55017");
forward_mapping_test!(test_forward_mapping_55018, "55018");
forward_mapping_test!(test_forward_mapping_55022, "55022");
forward_mapping_test!(test_forward_mapping_55023, "55023");
forward_mapping_test!(test_forward_mapping_55024, "55024");
forward_mapping_test!(test_forward_mapping_55035, "55035");

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

            if spec.fixture.is_empty() {
                eprintln!(
                    "Skipping interchange test for PID {}: no fixture defined",
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

interchange_test!(test_interchange_55013, "55013");
interchange_test!(test_interchange_55014, "55014");
interchange_test!(test_interchange_55015, "55015");
interchange_test!(test_interchange_55016, "55016");
interchange_test!(test_interchange_55017, "55017");
interchange_test!(test_interchange_55018, "55018");
interchange_test!(test_interchange_55022, "55022");
interchange_test!(test_interchange_55023, "55023");
interchange_test!(test_interchange_55024, "55024");
interchange_test!(test_interchange_55035, "55035");
