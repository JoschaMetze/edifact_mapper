//! EDIFACT roundtrip tests for PIDs 55013-55035.
//!
//! Full pipeline: EDIFACT -> tokenize -> split -> assemble -> map_interchange
//! -> map_interchange_reverse -> disassemble -> render -> compare with original.

mod common;
use common::test_utils;

/// Fixtures with non-MIG-compliant segment ordering.
/// The roundtrip normalizes these to MIG-defined order, so byte-identical
/// comparison fails — but the data content is correct.
///
/// DEV-77392-3: ZD7 SG10 has CCI+ZB3 before CCI+Z49/ZF3, contradicting
/// the MIG which defines Steuerkanal (Z49, Nr 00091) before Zugeordnete
/// Marktpartner (ZB3, Nr 00092). All 4 other fixtures follow MIG order.
const KNOWN_WRONG_FIXTURE_ORDER: &[&str] = &["55035_UTILMD_S2.1_DEV-77392-3.edi"];

/// TOML loading test -- verifies all TOML files parse correctly.
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
            eprintln!(
                "PID {} TOML loading OK: {} message + {} transaction definitions",
                $pid,
                msg_engine.definitions().len(),
                tx_engine.definitions().len()
            );
        }
    };
}

// TOML loading tests (all PIDs, no fixture needed)
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

// Full EDIFACT roundtrip tests (PIDs with fixtures)

#[test]
fn test_roundtrip_55013() {
    test_utils::run_full_roundtrip("55013");
}
#[test]
fn test_roundtrip_55014() {
    test_utils::run_full_roundtrip("55014");
}
#[test]
fn test_roundtrip_55015() {
    test_utils::run_full_roundtrip("55015");
}
#[test]
fn test_roundtrip_55016() {
    test_utils::run_full_roundtrip("55016");
}
#[test]
fn test_roundtrip_55017() {
    test_utils::run_full_roundtrip("55017");
}
#[test]
fn test_roundtrip_55018() {
    test_utils::run_full_roundtrip("55018");
}
#[test]
fn test_roundtrip_55035() {
    test_utils::run_full_roundtrip_with_skip("55035", KNOWN_WRONG_FIXTURE_ORDER);
}
// PIDs 55022, 55023, 55024 have no fixture files -- TOML loading tests only.
