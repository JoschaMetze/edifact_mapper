//! Bulk roundtrip tests for all FV2510 message types (non-UTILMD).
//!
//! Tests generated EDIFACT fixtures through the full pipeline:
//! EDIFACT → tokenize → split → assemble → map_interchange
//! → map_interchange_reverse → disassemble → render → byte-identical comparison.
//!
//! UTILMD Strom/Gas FV2510 are tested in their own dedicated test files.

mod common;

use common::test_utils;

// ═══════════════════════════════════════════════════════════════════════
// APERAK — 1 PID (message-only, empty AHB PID)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_aperak_generated_roundtrip() {
    let Some(fixture) = common::aperak_fv2510::discover_generated_fixture() else {
        eprintln!("APERAK FV2510: no generated fixture found -- skipping");
        return;
    };

    let Some(mig) = common::aperak_fv2510::load_pid_filtered_mig() else {
        eprintln!("APERAK FV2510: MIG/AHB XML not available -- skipping");
        return;
    };

    let msg_engine = common::aperak_fv2510::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    test_utils::run_single_fixture_roundtrip_with_tx_group(
        common::aperak_fv2510::PID_FIXTURE,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::aperak_fv2510::TX_GROUP,
    );
}

// ═══════════════════════════════════════════════════════════════════════
// CONTRL — 1 PID (message-only, empty AHB PID)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_contrl_generated_roundtrip() {
    let Some(fixture) = common::contrl_fv2510::discover_generated_fixture() else {
        eprintln!("CONTRL FV2510: no generated fixture found -- skipping");
        return;
    };

    let Some(mig) = common::contrl_fv2510::load_pid_filtered_mig() else {
        eprintln!("CONTRL FV2510: MIG/AHB XML not available -- skipping");
        return;
    };

    let msg_engine = common::contrl_fv2510::load_message_engine();
    let tx_engine = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);

    test_utils::run_single_fixture_roundtrip_with_tx_group(
        common::contrl_fv2510::PID_FIXTURE,
        &fixture,
        &mig,
        &msg_engine,
        &tx_engine,
        common::contrl_fv2510::TX_GROUP,
    );
}

// ═══════════════════════════════════════════════════════════════════════
// COMDIS — 2 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_comdis_all_pids_roundtrip() {
    let pids = ["29001", "29002"];
    run_all_generated_roundtrips("COMDIS", &pids, &[], |pid| {
        let fixture = common::comdis_fv2510::discover_generated_fixture(pid)?;
        let mig = common::comdis_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::comdis_fv2510::load_split_engines(pid);
        Some((fixture, mig, msg, tx, common::comdis_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// IFTSTA — 35 PIDs (2 families)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_iftsta_all_pids_roundtrip() {
    let pids = [
        "21000", "21001", "21002", "21003", "21004", "21005",
        "21007", "21009", "21010", "21011", "21012", "21013", "21015", "21018",
        "21024", "21025", "21026", "21027", "21028", "21029", "21030", "21031",
        "21032", "21033", "21035", "21036", "21037", "21038", "21039", "21040",
        "21042", "21043", "21044", "21045", "21047",
    ];
    run_all_generated_roundtrips("IFTSTA", &pids, &[], |pid| {
        let fixture = common::iftsta_fv2510::discover_generated_fixture(pid)?;
        let mig = common::iftsta_fv2510::load_pid_filtered_mig(pid)?;
        let tx_dir = common::iftsta_fv2510::pid_dir(pid);
        let (msg, tx) = if tx_dir.exists() {
            common::iftsta_fv2510::load_split_engines(pid)
        } else {
            let msg = common::iftsta_fv2510::load_message_engine();
            let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
            (msg, tx)
        };
        let tx_group = common::iftsta_fv2510::tx_group_for_pid(pid);
        Some((fixture, mig, msg, tx, tx_group))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// INVOIC — 11 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_invoic_all_pids_roundtrip() {
    let pids = [
        "31001", "31002", "31003", "31004", "31005", "31006",
        "31007", "31008", "31009", "31010", "31011",
    ];
    run_all_generated_roundtrips("INVOIC", &pids, &[], |pid| {
        let fixture = common::invoic_fv2510::discover_generated_fixture(pid)?;
        let mig = common::invoic_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::invoic_fv2510::load_engines_for_pid(pid);
        Some((fixture, mig, msg, tx, common::invoic_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// MSCONS — 26 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_mscons_all_pids_roundtrip() {
    let pids = [
        "13002", "13003", "13005", "13006", "13007", "13008", "13009", "13010",
        "13011", "13012", "13013", "13014", "13015", "13016", "13017", "13018",
        "13019", "13020", "13021", "13022", "13023", "13024", "13025", "13026",
        "13027", "13028",
    ];
    run_all_generated_roundtrips("MSCONS", &pids, &[], |pid| {
        let fixture = common::mscons_fv2510::discover_generated_fixture(pid)?;
        let mig = common::mscons_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::mscons_fv2510::load_split_engines(pid);
        Some((fixture, mig, msg, tx, common::mscons_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// ORDCHG — 3 PIDs (message-only)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_ordchg_all_pids_roundtrip() {
    let pids = ["39000", "39001", "39002"];
    run_all_generated_roundtrips("ORDCHG", &pids, &[], |pid| {
        let fixture = common::ordchg_fv2510::discover_generated_fixture(pid)?;
        let mig = common::ordchg_fv2510::load_pid_filtered_mig(pid)?;
        let msg = common::ordchg_fv2510::load_message_engine();
        let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
        Some((fixture, mig, msg, tx, common::ordchg_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// ORDERS — 50 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_orders_all_pids_roundtrip() {
    let pids = [
        "17001", "17002", "17003", "17004", "17005", "17006", "17007", "17008",
        "17009", "17011", "17101", "17102", "17103", "17104", "17110", "17111",
        "17112", "17113", "17114", "17115", "17116", "17117", "17118", "17120",
        "17121", "17122", "17123", "17124", "17125", "17126", "17128", "17129",
        "17130", "17131", "17132", "17133", "17134", "17135", "17201", "17202",
        "17203", "17204", "17205", "17206", "17207", "17208", "17209", "17210",
        "17211", "17301",
    ];
    let known: [&str; 0] = [];
    run_all_generated_roundtrips("ORDERS", &pids, &known, |pid| {
        let fixture = common::orders_fv2510::discover_generated_fixture(pid)?;
        let mig = common::orders_fv2510::load_pid_filtered_mig(pid)?;
        let tx_dir = common::orders_fv2510::pid_dir(pid);
        let (msg, tx) = if tx_dir.exists() {
            common::orders_fv2510::load_split_engines(pid)
        } else {
            let msg = common::orders_fv2510::load_message_engine();
            let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
            (msg, tx)
        };
        Some((fixture, mig, msg, tx, common::orders_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// ORDRSP — 40 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_ordrsp_all_pids_roundtrip() {
    let pids = [
        "19001", "19002", "19003", "19004", "19005", "19006", "19007", "19009",
        "19010", "19011", "19012", "19013", "19014", "19015", "19016", "19101",
        "19102", "19103", "19104", "19110", "19114", "19115", "19116", "19117",
        "19118", "19119", "19120", "19121", "19123", "19124", "19127", "19128",
        "19129", "19130", "19131", "19132", "19133", "19204", "19301", "19302",
    ];
    run_all_generated_roundtrips("ORDRSP", &pids, &[], |pid| {
        let fixture = common::ordrsp_fv2510::discover_generated_fixture(pid)?;
        let mig = common::ordrsp_fv2510::load_pid_filtered_mig(pid)?;
        let tx_dir = common::ordrsp_fv2510::pid_dir(pid);
        let (msg, tx) = if tx_dir.exists() {
            common::ordrsp_fv2510::load_split_engines(pid)
        } else {
            let msg = common::ordrsp_fv2510::load_message_engine();
            let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
            (msg, tx)
        };
        Some((fixture, mig, msg, tx, common::ordrsp_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// PARTIN — 7 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_partin_all_pids_roundtrip() {
    let pids = ["37000", "37001", "37002", "37003", "37004", "37005", "37006"];
    run_all_generated_roundtrips("PARTIN", &pids, &[], |pid| {
        let fixture = common::partin_fv2510::discover_generated_fixture(pid)?;
        let mig = common::partin_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::partin_fv2510::load_engines_for_pid(pid);
        Some((fixture, mig, msg, tx, common::partin_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// PRICAT — 3 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_pricat_all_pids_roundtrip() {
    let pids = ["27001", "27002", "27003"];
    run_all_generated_roundtrips("PRICAT", &pids, &[], |pid| {
        let fixture = common::pricat_fv2510::discover_generated_fixture(pid)?;
        let mig = common::pricat_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::pricat_fv2510::load_split_engines(pid);
        Some((fixture, mig, msg, tx, common::pricat_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// QUOTES — 5 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_quotes_all_pids_roundtrip() {
    let pids = ["15001", "15002", "15003", "15004", "15005"];
    run_all_generated_roundtrips("QUOTES", &pids, &[], |pid| {
        let fixture = common::quotes_fv2510::discover_generated_fixture(pid)?;
        let mig = common::quotes_fv2510::load_pid_filtered_mig(pid)?;
        let tx_dir = common::quotes_fv2510::pid_dir(pid);
        let (msg, tx) = if tx_dir.exists() {
            common::quotes_fv2510::load_split_engines(pid)
        } else {
            let msg = common::quotes_fv2510::load_message_engine();
            let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
            (msg, tx)
        };
        Some((fixture, mig, msg, tx, common::quotes_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// REMADV — 4 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_remadv_all_pids_roundtrip() {
    let pids = ["33001", "33002", "33003", "33004"];
    run_all_generated_roundtrips("REMADV", &pids, &[], |pid| {
        let fixture = common::remadv_fv2510::discover_generated_fixture(pid)?;
        let mig = common::remadv_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::remadv_fv2510::load_split_engines(pid);
        Some((fixture, mig, msg, tx, common::remadv_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// REQOTE — 5 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_reqote_all_pids_roundtrip() {
    let pids = ["35001", "35002", "35003", "35004", "35005"];
    run_all_generated_roundtrips("REQOTE", &pids, &[], |pid| {
        let fixture = common::reqote_fv2510::discover_generated_fixture(pid)?;
        let mig = common::reqote_fv2510::load_pid_filtered_mig(pid)?;
        let tx_dir = common::reqote_fv2510::pid_dir(pid);
        let (msg, tx) = if tx_dir.exists() {
            common::reqote_fv2510::load_split_engines(pid)
        } else {
            let msg = common::reqote_fv2510::load_message_engine();
            let tx = mig_bo4e::engine::MappingEngine::from_definitions(vec![]);
            (msg, tx)
        };
        Some((fixture, mig, msg, tx, common::reqote_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// UTILTS — 8 PIDs
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_utilts_all_pids_roundtrip() {
    let pids = ["25001", "25004", "25005", "25006", "25007", "25008", "25009", "25010"];
    run_all_generated_roundtrips("UTILTS", &pids, &[], |pid| {
        let fixture = common::utilts_fv2510::discover_generated_fixture(pid)?;
        let mig = common::utilts_fv2510::load_pid_filtered_mig(pid)?;
        let (msg, tx) = common::utilts_fv2510::load_split_engines(pid);
        Some((fixture, mig, msg, tx, common::utilts_fv2510::TX_GROUP))
    });
}

// ═══════════════════════════════════════════════════════════════════════
// Real fixture roundtrip tests (from example_market_communication corpus)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_fv2510_comdis_real_fixture_roundtrip() {
    run_all_real_fixture_roundtrips("COMDIS", &["29001", "29002"], &[], |pid, skip| {
        common::comdis_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_iftsta_real_fixture_roundtrip() {
    let pids = [
        "21000", "21001", "21002", "21003", "21004", "21005",
        "21007", "21009", "21010", "21011", "21012", "21013", "21015", "21018",
        "21024", "21025", "21026", "21027", "21028", "21029", "21030", "21031",
        "21032", "21033", "21035", "21036", "21037", "21038", "21039", "21040",
        "21042", "21043", "21044", "21045", "21047",
    ];
    // KNOWN_INCOMPLETE: SG2/SG3 NAD ordering — reverse nests CTA/COM under first NAD rep
    let skip = &["21033_IFTSTA_2.0g_W1693469088XXX.edi"];
    run_all_real_fixture_roundtrips("IFTSTA", &pids, skip, |pid, skip| {
        common::iftsta_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_invoic_real_fixture_roundtrip() {
    let pids = [
        "31001", "31002", "31003", "31004", "31005", "31006",
        "31007", "31008", "31009", "31010", "31011",
    ];
    // KNOWN_INCOMPLETE: NAD C080 d3045 name format code (Z02) not roundtripped
    let skip = &[
        "31002_INVOIC_2.8e_DEV-96031.edi",
        "31004_INVOIC_2.8e_DEV-96031.edi",
        "31009_INVOIC_2.8e_DEV-96031.edi",
    ];
    run_all_real_fixture_roundtrips("INVOIC", &pids, skip, |pid, skip| {
        common::invoic_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_orders_real_fixture_roundtrip() {
    let pids = [
        "17001", "17002", "17003", "17004", "17005", "17006", "17007", "17008",
        "17009", "17011", "17101", "17102", "17103", "17104", "17110", "17111",
        "17112", "17113", "17114", "17115", "17116", "17117", "17118", "17120",
        "17121", "17122", "17123", "17124", "17125", "17126", "17128", "17129",
        "17130", "17131", "17132", "17133", "17134", "17135", "17201", "17202",
        "17203", "17204", "17205", "17206", "17207", "17208", "17209", "17210",
        "17211", "17301",
    ];
    // KNOWN_INCOMPLETE: SG2.SG3 RFF nesting + multi-LIN transaction structures
    let skip = &[
        "17011_ORDERS_1.4b_DEV-96032-2.edi",
        "17011_ORDERS_1.4b_DEV-96032.edi",
        "17134_ORDERS_1.4b_DEV-96032.edi",
    ];
    run_all_real_fixture_roundtrips("ORDERS", &pids, skip, |pid, skip| {
        common::orders_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_ordrsp_real_fixture_roundtrip() {
    let pids = [
        "19001", "19002", "19003", "19004", "19005", "19006", "19007", "19009",
        "19010", "19011", "19012", "19013", "19014", "19015", "19016", "19101",
        "19102", "19103", "19104", "19110", "19114", "19115", "19116", "19117",
        "19118", "19119", "19120", "19121", "19123", "19124", "19127", "19128",
        "19129", "19130", "19131", "19132", "19133", "19204", "19301", "19302",
    ];
    // KNOWN_INCOMPLETE: message-only fixtures with missing DTM/phantom UNS
    let skip = &[
        "19005_ORDRSP_1.4a_ALEXANDE108027.edi",
        "19006_ORDRSP_1.4a_ALEXANDE987528.edi",
    ];
    run_all_real_fixture_roundtrips("ORDRSP", &pids, skip, |pid, skip| {
        common::ordrsp_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_pricat_real_fixture_roundtrip() {
    let pids = ["27001", "27002", "27003"];
    // KNOWN_INCOMPLETE: DTM+NAD schema mismatch — DTM segments not roundtripped
    let skip = &[
        "27002_PRICAT_2.0e_DEV-96034-2.edi",
        "27002_PRICAT_2.0e_DEV-96034-3.edi",
        "27002_PRICAT_2.0e_DEV-96034.edi",
        "27003_PRICAT_2.0e_DEV-96034.edi",
    ];
    run_all_real_fixture_roundtrips("PRICAT", &pids, skip, |pid, skip| {
        common::pricat_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_quotes_real_fixture_roundtrip() {
    let pids = ["15001", "15002", "15003", "15004", "15005"];
    // KNOWN_INCOMPLETE: message-only fixture with missing DTM/phantom UNS
    let skip = &["15005_QUOTES_1.3b_DEV-96035.edi"];
    run_all_real_fixture_roundtrips("QUOTES", &pids, skip, |pid, skip| {
        common::quotes_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_mscons_real_fixture_roundtrip() {
    let pids = [
        "13002", "13003", "13005", "13006", "13007", "13008", "13009", "13010",
        "13011", "13012", "13013", "13014", "13015", "13016", "13017", "13018",
        "13019", "13020", "13021", "13022", "13023", "13024", "13025", "13026",
        "13027", "13028",
    ];
    run_all_real_fixture_roundtrips("MSCONS", &pids, &[], |pid, skip| {
        common::mscons_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_ordchg_real_fixture_roundtrip() {
    let pids = ["39000", "39001", "39002"];
    run_all_real_fixture_roundtrips("ORDCHG", &pids, &[], |pid, skip| {
        common::ordchg_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_partin_real_fixture_roundtrip() {
    let pids = ["37000", "37001", "37002", "37003", "37004", "37005", "37006"];
    run_all_real_fixture_roundtrips("PARTIN", &pids, &[], |pid, skip| {
        common::partin_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_remadv_real_fixture_roundtrip() {
    let pids = ["33001", "33002", "33003", "33004"];
    run_all_real_fixture_roundtrips("REMADV", &pids, &[], |pid, skip| {
        common::remadv_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_reqote_real_fixture_roundtrip() {
    let pids = ["35001", "35002", "35003", "35004", "35005"];
    // KNOWN_INCOMPLETE: multi-LIN transaction structure not fully roundtripped
    let skip = &["35005_REQOTE_1.3c_ALEXANDE142263.edi"];
    run_all_real_fixture_roundtrips("REQOTE", &pids, skip, |pid, skip| {
        common::reqote_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

#[test]
fn test_fv2510_utilts_real_fixture_roundtrip() {
    let pids = ["25001", "25004", "25005", "25006", "25007", "25008", "25009", "25010"];
    run_all_real_fixture_roundtrips("UTILTS", &pids, &[], |pid, skip| {
        common::utilts_fv2510::run_full_roundtrip_with_skip(pid, skip);
    });
}

// ═══════════════════════════════════════════════════════════════════════
// Shared test runners
// ═══════════════════════════════════════════════════════════════════════

use mig_bo4e::engine::MappingEngine;
use mig_types::schema::mig::MigSchema;
use std::path::PathBuf;

/// Run generated fixture roundtrips for all PIDs of a message type.
fn run_all_generated_roundtrips<F>(
    msg_type: &str,
    pids: &[&str],
    known_incomplete: &[&str],
    loader: F,
) where
    F: Fn(&str) -> Option<(PathBuf, MigSchema, MappingEngine, MappingEngine, &'static str)>,
{
    let mut passed = 0;
    let mut skipped_incomplete = 0;
    let mut skipped_missing = 0;

    for pid in pids {
        if known_incomplete.contains(pid) {
            eprintln!("{msg_type} FV2510 PID {pid}: KNOWN_INCOMPLETE — skipping");
            skipped_incomplete += 1;
            continue;
        }

        let Some((fixture, mig, msg_engine, tx_engine, tx_group)) = loader(pid) else {
            eprintln!("{msg_type} FV2510 PID {pid}: fixture/MIG not available — skipping");
            skipped_missing += 1;
            continue;
        };

        test_utils::run_single_fixture_roundtrip_with_tx_group(
            pid,
            &fixture,
            &mig,
            &msg_engine,
            &tx_engine,
            tx_group,
        );

        passed += 1;
    }

    eprintln!(
        "\n{msg_type} FV2510 bulk roundtrip: {passed} passed, \
         {skipped_incomplete} known-incomplete, {skipped_missing} missing, \
         {} total",
        pids.len()
    );

    assert!(
        passed > 0,
        "{msg_type} FV2510: expected at least one PID to pass roundtrip"
    );
}

/// Run real fixture roundtrips for all PIDs of a message type.
///
/// Calls `run_full_roundtrip_with_skip` for each PID via the provided closure.
/// PIDs without real fixtures are silently skipped (the config module handles this).
/// Uses catch_unwind to continue testing after failures and report all at the end.
fn run_all_real_fixture_roundtrips<F>(
    msg_type: &str,
    pids: &[&str],
    known_incomplete_fixtures: &[&str],
    runner: F,
) where
    F: Fn(&str, &[&str]) + std::panic::RefUnwindSafe,
{
    let mut passed = 0;
    let mut failed: Vec<String> = Vec::new();

    for pid in pids {
        match std::panic::catch_unwind(|| {
            runner(pid, known_incomplete_fixtures);
        }) {
            Ok(()) => passed += 1,
            Err(_) => failed.push(pid.to_string()),
        }
    }

    eprintln!(
        "\n{msg_type} FV2510 real fixture roundtrip: {passed} passed, {} failed, {} total PIDs",
        failed.len(),
        pids.len()
    );

    if !failed.is_empty() {
        eprintln!("  Failed PIDs: {}", failed.join(", "));
    }

    assert!(
        failed.is_empty(),
        "{msg_type} FV2510 real fixture roundtrip: {} PIDs failed: {}",
        failed.len(),
        failed.join(", ")
    );
}