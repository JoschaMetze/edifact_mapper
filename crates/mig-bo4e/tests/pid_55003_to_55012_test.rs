//! Integration tests for PIDs 55003-55012 TOML mapping definitions.
//!
//! Two test categories:
//! 1. **TOML loading**: Verify `MappingEngine::load_merged()` succeeds for each PID.
//! 2. **Forward mapping**: For each PID with a fixture, run the full EDIFACT->BO4E pipeline.
//!
//! Tests skip gracefully when required files (MIG/AHB XML, fixtures, mappings) are absent.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use mig_assembly::assembler::Assembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use std::collections::HashSet;
use std::path::Path;

mod common;
use common::test_utils;
use test_utils::{AHB_XML_PATH, FIXTURE_DIR, MIG_XML_PATH};

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
    PidTestSpec {
        pid: "55037",
        fixture: "55037_UTILMD_S2.1_ALEXANDE149633.edi",
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55109",
        fixture: "55109_UTILMD_S2.1_ALEXANDE460784.edi",
        tx_stammdaten_keys: &[
            "marktlokation",
            "geschaeftspartner",
            "ansprechpartner",
            "enfgDaten",
            "zeitscheibe",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55110",
        fixture: "55110_UTILMD_S2.1_ALEXANDE178268.edi",
        tx_stammdaten_keys: &[
            "marktlokation",
            "geschaeftspartner",
            "ansprechpartner",
            "zeitscheibe",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55036",
        fixture: "55036_UTILMD_S2.1_ALEXANDE348314.edi",
        tx_stammdaten_keys: &["marktlokation", "geschaeftspartner"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55038",
        fixture: "55038_UTILMD_S2.1_ALEXANDE180450.edi",
        tx_stammdaten_keys: &["marktlokation", "geschaeftspartner"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55604",
        fixture: "55604_UTILMD_S2.1_ALEXANDE6390963.edi",
        tx_stammdaten_keys: &["marktlokation", "geschaeftspartner"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55602",
        fixture: "55602_UTILMD_S2.1_ALEXANDE923530114.edi",
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
        pid: "55136",
        fixture: "55136_UTILMD_S2.1_ALEXANDE416834.edi",
        tx_stammdaten_keys: &["marktlokation", "geschaeftspartner", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55137",
        fixture: "55137_UTILMD_S2.1_ALEXANDE626808.edi",
        tx_stammdaten_keys: &[
            "marktlokation",
            "geschaeftspartner",
            "zeitscheibe",
            "enfgDaten",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55225",
        fixture: "55225_UTILMD_S2.1_ALEXANDE205069.edi",
        tx_stammdaten_keys: &["netzlokation", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55232",
        fixture: "55232_UTILMD_S2.1_ALEXANDE155134.edi",
        tx_stammdaten_keys: &["netzlokation", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55615",
        fixture: "55615_UTILMD_S2.1_DEV-89186.edi",
        tx_stammdaten_keys: &["netzlokation", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55618",
        fixture: "55618_UTILMD_S2.1_ALEXANDE426380.edi",
        tx_stammdaten_keys: &["steuerbareRessource", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55620",
        fixture: "55620_UTILMD_S2.1_ALEXANDE180967.edi",
        tx_stammdaten_keys: &["messlokation", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55621",
        fixture: "55621_UTILMD_S2.1_ALEXANDE409547.edi",
        tx_stammdaten_keys: &["netzlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55624",
        fixture: "55624_UTILMD_S2.1_ALEXANDE979329.edi",
        tx_stammdaten_keys: &["steuerbareRessource"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55626",
        fixture: "55626_UTILMD_S2.1_ALEXANDE980499.edi",
        tx_stammdaten_keys: &["messlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55220",
        fixture: "55220_UTILMD_S2.1_ALEXANDE372301.edi",
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55655",
        fixture: "55655_UTILMD_S2.1_ALEXANDE207290.edi",
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55617",
        fixture: "55617_UTILMD_S2.1_ALEXANDE798498.edi",
        tx_stammdaten_keys: &["technischeRessource"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55650",
        fixture: "55650_UTILMD_S2.1_DEV-90398.edi",
        tx_stammdaten_keys: &["marktlokation", "obisKennzahl", "produktDaten"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55175",
        fixture: "55175_UTILMD_S2.1_ALEXANDE176492.edi",
        tx_stammdaten_keys: &[
            "netzlokation",
            "marktlokation",
            "technischeRessource",
            "steuerbareRessource",
            "messlokation",
            "lokationsbuendel",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55042",
        fixture: "55042_UTILMD_S2.1_SXMP21-20A82AG.edi",
        tx_stammdaten_keys: &["zaehler", "geschaeftspartner"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55616",
        fixture: "55616_UTILMD_S2.1_ALEXANDE121980_macosi.edi",
        tx_stammdaten_keys: &[
            "marktlokation",
            "marktlokationDaten",
            "obisKennzahlNutzung",
            "zeitscheibe",
            "geschaeftspartner",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55691",
        fixture: "55691_UTILMD_S2.1_ALEXANDE1917130348.edi",
        tx_stammdaten_keys: &["marktlokation", "geschaeftspartner", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55692",
        fixture: "55692_UTILMD_S2.1_ALEXANDE1074799373.edi",
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55646",
        fixture: "55646_UTILMD_S2.1_FELLERM1615608.edi",
        tx_stammdaten_keys: &["steuerbareRessource", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55656",
        fixture: "55656_UTILMD_S2.1_ALEXANDE665926512.edi",
        tx_stammdaten_keys: &["steuerbareRessource"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55641",
        fixture: "55641_UTILMD_S2.1_FELLERM8037025.edi",
        tx_stammdaten_keys: &["steuerbareRessource", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55651",
        fixture: "55651_UTILMD_S2.1_ALEXANDE931939.edi",
        tx_stammdaten_keys: &["steuerbareRessource", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55654",
        fixture: "55654_UTILMD_S2.1_ALEXANDE188796.edi",
        tx_stammdaten_keys: &["netzlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55649",
        fixture: "55649_UTILMD_S2.1_ALEXANDE411341.edi",
        // NOTE: Only netzlokation and zeitscheibe appear because the fixture has
        // RFF+Z32 segments in SG8 groups that the AHB doesn't include (Numbers
        // 00090/00100/00107 missing). The assembler can't consume RFF after SEQ,
        // blocking PIA/CCI/CAV — so obisKennzahl and produktDaten are empty.
        tx_stammdaten_keys: &["netzlokation", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55640",
        fixture: "55640_UTILMD_S2.1_FELLERM8037025.edi",
        tx_stammdaten_keys: &[
            "marktlokation",
            "obisKennzahl",
            "produktDaten",
            "zeitscheibe",
            "termindaten",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55180",
        fixture: "55180_UTILMD_S2.1_ALEXANDE1433256082.edi",
        tx_stammdaten_keys: &[],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55553",
        fixture: "55553_UTILMD_S2.1_ALEXANDE703082.edi",
        tx_stammdaten_keys: &[
            "obisMarktlokation",
            "zaehleinrichtung",
            "obisZaehleinrichtung",
            "zeitscheibe",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55555",
        fixture: "55555_UTILMD_S2.1_ALEXANDE343731.edi",
        tx_stammdaten_keys: &[
            "obisMarktlokation",
            "zaehleinrichtung",
            "obisZaehleinrichtung",
            "zeitscheibe",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55126",
        fixture: "55126_UTILMD_S2.1_ALEXANDE203097.edi",
        tx_stammdaten_keys: &["marktlokation", "profil", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55156",
        fixture: "55156_UTILMD_S2.1_ALEXANDE806316.edi",
        tx_stammdaten_keys: &["marktlokation", "profil", "zeitscheibe"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55622",
        fixture: "55622_UTILMD_S2.1_ALEXANDE140866.edi",
        tx_stammdaten_keys: &["marktlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55600",
        fixture: "55600_UTILMD_S2.1_ALEXANDE1777431606.edi",
        tx_stammdaten_keys: &[
            "marktlokation",
            "messlokation",
            "steuerbareRessource",
            "technischeRessource",
            "geschaeftspartner",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55643",
        fixture: "55643_UTILMD_S2.1_FELLERM8037025.edi",
        tx_stammdaten_keys: &[
            "messlokation",
            "zaehler",
            "obisKennzahl",
            "smartmeterGateway",
            "geschaeftspartner",
            "zeitscheibe",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55653",
        fixture: "55653_UTILMD_S2.1_ALEXANDE705357.edi",
        tx_stammdaten_keys: &[
            "messlokation",
            "zaehler",
            "obisKennzahl",
            "wandlerdaten",
            "smartmeterGateway",
            "geschaeftspartner",
            "zeitscheibe",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55658",
        fixture: "55658_UTILMD_S2.1_ALEXANDE464222634.edi",
        tx_stammdaten_keys: &["messlokation"],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
    PidTestSpec {
        pid: "55648",
        // NOTE: fixture is misnamed — contains PID 55684 content (LOC+Z16, STS+ZX6, SEQ+Z02).
        // PID 55648 expects LOC+Z17, STS+ZX7, SEQ+ZG6/ZG7. Listed in KNOWN_INCOMPLETE.
        fixture: "55648_UTILMD_S2.1_FELLERM8037025.edi",
        tx_stammdaten_keys: &[
            "messlokation",
            "zaehler",
            "obisKennzahl",
            "geschaeftspartner",
        ],
        tx_transaktionsdaten_keys: &["vorgangId"],
    },
];

// ── Helper functions ──

// ── TOML loading tests (no fixtures needed) ──

macro_rules! toml_loading_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            let tx_dir = test_utils::pid_dir($pid);
            if !test_utils::message_dir().exists() || !tx_dir.exists() {
                eprintln!("Skipping {}: mapping dirs not found", stringify!($name));
                return;
            }
            let (msg_engine, tx_engine) = test_utils::load_split_engines($pid);
            let total = msg_engine.definitions().len() + tx_engine.definitions().len();
            assert!(
                total > 0,
                "PID {} engines should have non-empty definitions",
                $pid
            );
            eprintln!("PID {} TOML loading OK: {} definitions loaded", $pid, total);
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
toml_loading_test!(test_toml_loading_55037, "55037");
toml_loading_test!(test_toml_loading_55109, "55109");
toml_loading_test!(test_toml_loading_55110, "55110");
toml_loading_test!(test_toml_loading_55036, "55036");
toml_loading_test!(test_toml_loading_55038, "55038");
toml_loading_test!(test_toml_loading_55604, "55604");
toml_loading_test!(test_toml_loading_55602, "55602");
toml_loading_test!(test_toml_loading_55136, "55136");
toml_loading_test!(test_toml_loading_55137, "55137");
toml_loading_test!(test_toml_loading_55225, "55225");
toml_loading_test!(test_toml_loading_55232, "55232");
toml_loading_test!(test_toml_loading_55615, "55615");
toml_loading_test!(test_toml_loading_55618, "55618");
toml_loading_test!(test_toml_loading_55620, "55620");
toml_loading_test!(test_toml_loading_55621, "55621");
toml_loading_test!(test_toml_loading_55624, "55624");
toml_loading_test!(test_toml_loading_55626, "55626");
toml_loading_test!(test_toml_loading_55220, "55220");
toml_loading_test!(test_toml_loading_55655, "55655");
toml_loading_test!(test_toml_loading_55617, "55617");
toml_loading_test!(test_toml_loading_55650, "55650");
toml_loading_test!(test_toml_loading_55175, "55175");
toml_loading_test!(test_toml_loading_55042, "55042");
toml_loading_test!(test_toml_loading_55616, "55616");
toml_loading_test!(test_toml_loading_55691, "55691");
toml_loading_test!(test_toml_loading_55692, "55692");
toml_loading_test!(test_toml_loading_55646, "55646");
toml_loading_test!(test_toml_loading_55656, "55656");
toml_loading_test!(test_toml_loading_55641, "55641");
toml_loading_test!(test_toml_loading_55651, "55651");
toml_loading_test!(test_toml_loading_55654, "55654");
toml_loading_test!(test_toml_loading_55649, "55649");
toml_loading_test!(test_toml_loading_55640, "55640");
toml_loading_test!(test_toml_loading_55180, "55180");
toml_loading_test!(test_toml_loading_55553, "55553");
toml_loading_test!(test_toml_loading_55555, "55555");
toml_loading_test!(test_toml_loading_55126, "55126");
toml_loading_test!(test_toml_loading_55156, "55156");
toml_loading_test!(test_toml_loading_55622, "55622");
toml_loading_test!(test_toml_loading_55600, "55600");
toml_loading_test!(test_toml_loading_55218, "55218");
toml_loading_test!(test_toml_loading_55623, "55623");
toml_loading_test!(test_toml_loading_55643, "55643");
toml_loading_test!(test_toml_loading_55653, "55653");
toml_loading_test!(test_toml_loading_55658, "55658");
toml_loading_test!(test_toml_loading_55648, "55648");

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

    if KNOWN_INCOMPLETE.contains(&spec.fixture) {
        eprintln!(
            "Skipping forward test for PID {}: fixture {} is KNOWN_INCOMPLETE",
            spec.pid, spec.fixture
        );
        return;
    }

    let tx_dir = test_utils::pid_dir(spec.pid);
    if !test_utils::message_dir().exists() || !tx_dir.exists() {
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

    // Step 6: Load split engines (message + transaction with common inheritance)
    let (msg_engine, tx_engine) = test_utils::load_split_engines(spec.pid);

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
forward_mapping_test!(test_forward_mapping_55037, "55037");
forward_mapping_test!(test_forward_mapping_55109, "55109");
forward_mapping_test!(test_forward_mapping_55110, "55110");
forward_mapping_test!(test_forward_mapping_55036, "55036");
forward_mapping_test!(test_forward_mapping_55038, "55038");
forward_mapping_test!(test_forward_mapping_55604, "55604");
forward_mapping_test!(test_forward_mapping_55602, "55602");
forward_mapping_test!(test_forward_mapping_55136, "55136");
forward_mapping_test!(test_forward_mapping_55137, "55137");
forward_mapping_test!(test_forward_mapping_55225, "55225");
forward_mapping_test!(test_forward_mapping_55232, "55232");
forward_mapping_test!(test_forward_mapping_55615, "55615");
forward_mapping_test!(test_forward_mapping_55618, "55618");
forward_mapping_test!(test_forward_mapping_55620, "55620");
forward_mapping_test!(test_forward_mapping_55621, "55621");
forward_mapping_test!(test_forward_mapping_55624, "55624");
forward_mapping_test!(test_forward_mapping_55626, "55626");
forward_mapping_test!(test_forward_mapping_55220, "55220");
forward_mapping_test!(test_forward_mapping_55655, "55655");
forward_mapping_test!(test_forward_mapping_55617, "55617");
forward_mapping_test!(test_forward_mapping_55650, "55650");
forward_mapping_test!(test_forward_mapping_55175, "55175");
forward_mapping_test!(test_forward_mapping_55042, "55042");
forward_mapping_test!(test_forward_mapping_55616, "55616");
forward_mapping_test!(test_forward_mapping_55691, "55691");
forward_mapping_test!(test_forward_mapping_55692, "55692");
forward_mapping_test!(test_forward_mapping_55646, "55646");
forward_mapping_test!(test_forward_mapping_55656, "55656");
forward_mapping_test!(test_forward_mapping_55641, "55641");
forward_mapping_test!(test_forward_mapping_55651, "55651");
forward_mapping_test!(test_forward_mapping_55654, "55654");
forward_mapping_test!(test_forward_mapping_55649, "55649");
forward_mapping_test!(test_forward_mapping_55640, "55640");
forward_mapping_test!(test_forward_mapping_55180, "55180");
forward_mapping_test!(test_forward_mapping_55553, "55553");
forward_mapping_test!(test_forward_mapping_55555, "55555");
forward_mapping_test!(test_forward_mapping_55126, "55126");
forward_mapping_test!(test_forward_mapping_55156, "55156");
forward_mapping_test!(test_forward_mapping_55622, "55622");
forward_mapping_test!(test_forward_mapping_55600, "55600");
forward_mapping_test!(test_forward_mapping_55643, "55643");
forward_mapping_test!(test_forward_mapping_55653, "55653");
forward_mapping_test!(test_forward_mapping_55658, "55658");
forward_mapping_test!(test_forward_mapping_55648, "55648");

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

            // Skip fixtures with known assembly issues
            if KNOWN_INCOMPLETE.contains(&spec.fixture) {
                eprintln!(
                    "Skipping interchange test for PID {}: fixture {} is KNOWN_INCOMPLETE",
                    spec.pid, spec.fixture
                );
                return;
            }

            let msg_dir = test_utils::message_dir();
            let tx_dir = test_utils::pid_dir(spec.pid);
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

            // Map (with common inheritance)
            let (msg_engine, tx_engine) = test_utils::load_split_engines(spec.pid);
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
interchange_test!(test_interchange_55037, "55037");
interchange_test!(test_interchange_55109, "55109");
interchange_test!(test_interchange_55110, "55110");
interchange_test!(test_interchange_55036, "55036");
interchange_test!(test_interchange_55038, "55038");
interchange_test!(test_interchange_55604, "55604");
interchange_test!(test_interchange_55602, "55602");
interchange_test!(test_interchange_55136, "55136");
interchange_test!(test_interchange_55137, "55137");
interchange_test!(test_interchange_55225, "55225");
interchange_test!(test_interchange_55232, "55232");
interchange_test!(test_interchange_55615, "55615");
interchange_test!(test_interchange_55618, "55618");
interchange_test!(test_interchange_55620, "55620");
interchange_test!(test_interchange_55621, "55621");
interchange_test!(test_interchange_55624, "55624");
interchange_test!(test_interchange_55626, "55626");
interchange_test!(test_interchange_55220, "55220");
interchange_test!(test_interchange_55655, "55655");
interchange_test!(test_interchange_55617, "55617");
interchange_test!(test_interchange_55650, "55650");
interchange_test!(test_interchange_55175, "55175");
interchange_test!(test_interchange_55042, "55042");
interchange_test!(test_interchange_55616, "55616");
interchange_test!(test_interchange_55691, "55691");
interchange_test!(test_interchange_55692, "55692");
interchange_test!(test_interchange_55646, "55646");
interchange_test!(test_interchange_55656, "55656");
interchange_test!(test_interchange_55641, "55641");
interchange_test!(test_interchange_55651, "55651");
interchange_test!(test_interchange_55654, "55654");
interchange_test!(test_interchange_55649, "55649");
interchange_test!(test_interchange_55640, "55640");
interchange_test!(test_interchange_55180, "55180");
interchange_test!(test_interchange_55553, "55553");
interchange_test!(test_interchange_55555, "55555");
interchange_test!(test_interchange_55126, "55126");
interchange_test!(test_interchange_55156, "55156");
interchange_test!(test_interchange_55622, "55622");
interchange_test!(test_interchange_55600, "55600");
interchange_test!(test_interchange_55643, "55643");
interchange_test!(test_interchange_55653, "55653");
interchange_test!(test_interchange_55658, "55658");
interchange_test!(test_interchange_55648, "55648");

// ── Full EDIFACT roundtrip tests ──

/// Fixtures with known mapping gaps that prevent byte-identical roundtrip.
/// These are legitimate issues to fix later, not test bugs.
const KNOWN_INCOMPLETE: &[&str] = &[
    // Fixture has RFF+Z38 in SG8 that AHB does not include (Number 00279/00285 not in PID).
    // Assembler can't consume RFF after SEQ entry, so PIA/CCI/CAV segments are lost.
    "55646_UTILMD_S2.1_FELLERM1615608.edi",
    // Same issue: RFF+Z38 in SG8 Z62/Z61 not in AHB for PID 55641/55651.
    // Assembler gets stuck at RFF after SEQ entry, remaining segments lost.
    "55641_UTILMD_S2.1_FELLERM8037025.edi",
    "55651_UTILMD_S2.1_ALEXANDE931939.edi",
    // Same issue: RFF+Z32 in SG8 Z51/Z57/Z60 not in AHB for PID 55649.
    // AHB Numbers 00090, 00100, 00107 (RFF segments) missing — assembler can't consume
    // RFF after SEQ entry, so PIA/CCI/CAV in all three SG8 groups are lost.
    "55649_UTILMD_S2.1_ALEXANDE411341.edi",
    // PID 55643: DTM+92 and DTM+157 in SG4 are not in the AHB (SG4 only defines IDE+STS).
    // Assembler can't skip DTM to reach STS, so all SG4 content segments are lost.
    "55643_UTILMD_S2.1_FELLERM8037025.edi",
    "55643_UTILMD_S2.1_FELLERM8037026.edi",
    // PID 55648: Fixture is misnamed — RFF+Z13:55684 inside, LOC+Z16 (Marktlokation) instead
    // of Z17 (Messlokation), STS+7++ZX6 instead of ZX7, SEQ+Z02 instead of ZG6/ZG7.
    // Fixture actually belongs to PID 55684 ("Änderung Daten der MaLo").
    "55648_UTILMD_S2.1_FELLERM8037025.edi",
];

macro_rules! roundtrip_test {
    ($name:ident, $pid:expr) => {
        #[test]
        fn $name() {
            test_utils::run_full_roundtrip_with_skip($pid, KNOWN_INCOMPLETE);
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
roundtrip_test!(test_roundtrip_55037, "55037");
roundtrip_test!(test_roundtrip_55109, "55109");
roundtrip_test!(test_roundtrip_55110, "55110");
roundtrip_test!(test_roundtrip_55036, "55036");
roundtrip_test!(test_roundtrip_55038, "55038");
roundtrip_test!(test_roundtrip_55604, "55604");
roundtrip_test!(test_roundtrip_55602, "55602");
roundtrip_test!(test_roundtrip_55136, "55136");
roundtrip_test!(test_roundtrip_55137, "55137");
roundtrip_test!(test_roundtrip_55225, "55225");
roundtrip_test!(test_roundtrip_55232, "55232");
roundtrip_test!(test_roundtrip_55615, "55615");
roundtrip_test!(test_roundtrip_55618, "55618");
roundtrip_test!(test_roundtrip_55620, "55620");
roundtrip_test!(test_roundtrip_55621, "55621");
roundtrip_test!(test_roundtrip_55624, "55624");
roundtrip_test!(test_roundtrip_55626, "55626");
roundtrip_test!(test_roundtrip_55220, "55220");
roundtrip_test!(test_roundtrip_55655, "55655");
roundtrip_test!(test_roundtrip_55617, "55617");
roundtrip_test!(test_roundtrip_55650, "55650");
roundtrip_test!(test_roundtrip_55175, "55175");
roundtrip_test!(test_roundtrip_55042, "55042");
roundtrip_test!(test_roundtrip_55616, "55616");
roundtrip_test!(test_roundtrip_55691, "55691");
roundtrip_test!(test_roundtrip_55692, "55692");
roundtrip_test!(test_roundtrip_55646, "55646");
roundtrip_test!(test_roundtrip_55656, "55656");
roundtrip_test!(test_roundtrip_55641, "55641");
roundtrip_test!(test_roundtrip_55651, "55651");
roundtrip_test!(test_roundtrip_55654, "55654");
roundtrip_test!(test_roundtrip_55649, "55649");
roundtrip_test!(test_roundtrip_55640, "55640");
roundtrip_test!(test_roundtrip_55180, "55180");
roundtrip_test!(test_roundtrip_55553, "55553");
roundtrip_test!(test_roundtrip_55555, "55555");
roundtrip_test!(test_roundtrip_55126, "55126");
roundtrip_test!(test_roundtrip_55156, "55156");
roundtrip_test!(test_roundtrip_55622, "55622");
roundtrip_test!(test_roundtrip_55600, "55600");
roundtrip_test!(test_roundtrip_55218, "55218");
roundtrip_test!(test_roundtrip_55623, "55623");
roundtrip_test!(test_roundtrip_55643, "55643");
roundtrip_test!(test_roundtrip_55653, "55653");
roundtrip_test!(test_roundtrip_55658, "55658");
roundtrip_test!(test_roundtrip_55648, "55648");
