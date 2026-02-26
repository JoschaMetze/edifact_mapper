//! Integration tests for Bo4eFieldIndex — resolves EDIFACT paths to BO4E paths.

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::error_mapping::Bo4eFieldIndex;
use mig_bo4e::path_resolver::PathResolver;
use std::collections::HashSet;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const MAPPINGS_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/pid_55001";
const MESSAGE_DIR: &str = "../../mappings/FV2504/UTILMD_Strom/message";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(std::path::Path::new(SCHEMA_DIR))
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

fn load_split_engines() -> Option<(MappingEngine, MappingEngine)> {
    let msg_dir = Path::new(MESSAGE_DIR);
    let tx_dir = Path::new(MAPPINGS_DIR);
    if !msg_dir.exists() || !tx_dir.exists() {
        return None;
    }
    MappingEngine::load_split(msg_dir, tx_dir)
        .ok()
        .map(|(m, t)| {
            let r = path_resolver();
            (m.with_path_resolver(r.clone()), t.with_path_resolver(r))
        })
}

#[test]
fn test_marktlokation_path_resolution() {
    let mig = match load_pid_filtered_mig("55001") {
        Some(m) => m,
        None => {
            eprintln!("Skipping: MIG/AHB XML not available");
            return;
        }
    };
    let (msg_engine, tx_engine) = match load_split_engines() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: mapping files not available");
            return;
        }
    };

    let mut all_defs: Vec<_> = msg_engine.definitions().to_vec();
    all_defs.extend(tx_engine.definitions().iter().cloned());

    let index = Bo4eFieldIndex::build(&all_defs, &mig);

    // marktlokation.toml: source_group = "SG4.SG5", loc.1.0 = "marktlokationsId"
    // LOC at SG4/SG5 → element 1 = composite C517 → component 0 = data element 3225
    let result = index.resolve("SG4/SG5/LOC/C517/3225");
    assert_eq!(
        result.as_deref(),
        Some("stammdaten.Marktlokation.marktlokationsId"),
        "Should resolve LOC/C517/3225 to Marktlokation.marktlokationsId"
    );
}

#[test]
fn test_prozessdaten_path_resolution() {
    let mig = match load_pid_filtered_mig("55001") {
        Some(m) => m,
        None => {
            eprintln!("Skipping: MIG/AHB XML not available");
            return;
        }
    };
    let (msg_engine, tx_engine) = match load_split_engines() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: mapping files not available");
            return;
        }
    };

    let mut all_defs: Vec<_> = msg_engine.definitions().to_vec();
    all_defs.extend(tx_engine.definitions().iter().cloned());

    let index = Bo4eFieldIndex::build(&all_defs, &mig);

    // prozessdaten.toml: source_group = "SG4", ide.1 = "vorgangId"
    // IDE at SG4 → element 1 = composite C206 → sub 0 = data element 7402 (Vorgangsnummer)
    let result = index.resolve("SG4/IDE/C206/7402");
    assert_eq!(
        result.as_deref(),
        Some("transaktionsdaten.Prozessdaten.vorgangId"),
        "Should resolve IDE/C206/7402 to Prozessdaten.vorgangId"
    );
}

#[test]
fn test_prefix_match_for_entity_level() {
    let mig = match load_pid_filtered_mig("55001") {
        Some(m) => m,
        None => {
            eprintln!("Skipping: MIG/AHB XML not available");
            return;
        }
    };
    let (msg_engine, tx_engine) = match load_split_engines() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: mapping files not available");
            return;
        }
    };

    let mut all_defs: Vec<_> = msg_engine.definitions().to_vec();
    all_defs.extend(tx_engine.definitions().iter().cloned());

    let index = Bo4eFieldIndex::build(&all_defs, &mig);

    // SG2/NAD/... should prefix-match to Marktteilnehmer
    let result = index.resolve("SG2/NAD/unknown_element");
    assert!(
        result.is_some(),
        "Should prefix-match SG2/NAD to a stammdaten entity"
    );
    let resolved = result.unwrap();
    assert!(
        resolved.starts_with("stammdaten."),
        "SG2 entities should be in stammdaten, got: {resolved}"
    );
}

#[test]
fn test_unknown_path_returns_none() {
    let mig = match load_pid_filtered_mig("55001") {
        Some(m) => m,
        None => {
            eprintln!("Skipping: MIG/AHB XML not available");
            return;
        }
    };
    let (msg_engine, tx_engine) = match load_split_engines() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: mapping files not available");
            return;
        }
    };

    let mut all_defs: Vec<_> = msg_engine.definitions().to_vec();
    all_defs.extend(tx_engine.definitions().iter().cloned());

    let index = Bo4eFieldIndex::build(&all_defs, &mig);

    assert!(index.resolve("SG99/UNKNOWN/9999").is_none());
}
