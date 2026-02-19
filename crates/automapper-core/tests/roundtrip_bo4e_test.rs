//! BO4E roundtrip integration tests: EDIFACT → parse → BO4E → generate → EDIFACT.
//!
//! Tests the complete bidirectional mapping pipeline through the Coordinator API.
//! Segment ordering in generate() follows the MIG XML Counter attributes.
//! See `docs/mig-segment-ordering.md` for the derivation.

use automapper_core::{create_coordinator, detect_format_version, FormatVersion};
use bo4e_extensions::{
    Geschaeftspartner, GeschaeftspartnerEdifact, Marktlokation, MarktlokationEdifact, Messlokation,
    MesslokationEdifact, Nachrichtendaten, Netzlokation, NetzlokationEdifact, Prozessdaten,
    UtilmdNachricht, UtilmdTransaktion, WithValidity, Zaehler, ZaehlerEdifact, Zeitraum,
    Zeitscheibe,
};
use chrono::NaiveDate;
use std::path::Path;

// ---------------------------------------------------------------------------
// 1. Synthetic roundtrip: construct EDIFACT → parse → generate → verify
// ---------------------------------------------------------------------------

/// Minimal EDIFACT message with envelope + one transaction containing
/// a Marktlokation, Messlokation, and Netzlokation.
const MINIMAL_EDIFACT: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002+9900456000001+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202507011330:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z17+MELO001'\
LOC+Z18+NELO001'\
STS+7+E01'\
UNT+11+MSG001'\
UNZ+1+REF001'";

#[test]
fn test_bo4e_roundtrip_synthetic_parse_generate() {
    // Step 1: Parse EDIFACT → BO4E (UtilmdNachricht)
    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(MINIMAL_EDIFACT).unwrap();

    assert_eq!(nachricht.transaktionen.len(), 1);
    assert_eq!(nachricht.dokumentennummer, "DOC001");
    assert_eq!(nachricht.kategorie, Some("E03".to_string()));

    let nd = &nachricht.nachrichtendaten;
    assert_eq!(nd.absender_mp_id, Some("9900123000002".to_string()));
    assert_eq!(nd.empfaenger_mp_id, Some("9900456000001".to_string()));
    assert_eq!(nd.datenaustauschreferenz, Some("REF001".to_string()));
    assert_eq!(nd.nachrichtenreferenz, Some("MSG001".to_string()));

    let tx = &nachricht.transaktionen[0];
    assert_eq!(tx.transaktions_id, "TXID001");
    assert_eq!(tx.marktlokationen.len(), 1);
    assert_eq!(tx.messlokationen.len(), 1);
    assert_eq!(tx.netzlokationen.len(), 1);
    assert_eq!(tx.prozessdaten.transaktionsgrund, Some("E01".to_string()));

    // Step 2: Generate BO4E → EDIFACT
    let output_bytes = coord.generate(&nachricht).unwrap();
    let output = String::from_utf8(output_bytes).unwrap();

    // Step 3: Verify key segments are present in output
    assert!(output.starts_with("UNA:+.? "), "should start with UNA");
    assert!(
        output.contains("UNB+UNOC:3+9900123000002+9900456000001+"),
        "UNB should have sender/recipient"
    );
    assert!(
        output.contains("+REF001'"),
        "UNB should have interchange reference"
    );
    assert!(
        output.contains("UNH+MSG001+UTILMD:D:11A:UN:S2.1'"),
        "UNH should have message ref and type"
    );
    assert!(output.contains("BGM+E03+DOC001'"), "BGM should match");
    assert!(
        output.contains("DTM+137:202507011330:303'"),
        "DTM+137 nachrichtendatum should be present"
    );
    assert!(
        output.contains("NAD+MS+9900123000002::293'"),
        "NAD+MS sender"
    );
    assert!(
        output.contains("NAD+MR+9900456000001::293'"),
        "NAD+MR recipient"
    );
    assert!(output.contains("IDE+24+TXID001'"), "IDE transaction ID");
    assert!(output.contains("STS+7+E01'"), "STS transaktionsgrund");
    assert!(output.contains("LOC+Z18+NELO001'"), "LOC+Z18 netzlokation");
    assert!(
        output.contains("LOC+Z16+DE00014545768S0000000000000003054'"),
        "LOC+Z16 marktlokation"
    );
    assert!(output.contains("LOC+Z17+MELO001'"), "LOC+Z17 messlokation");
    assert!(output.contains("UNT+"), "UNT trailer");
    assert!(output.contains("UNZ+1+REF001'"), "UNZ trailer");
}

#[test]
fn test_bo4e_roundtrip_synthetic_reparse() {
    // Parse → generate → reparse and compare BO4E structures
    let mut coord1 = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht1 = coord1.parse_nachricht(MINIMAL_EDIFACT).unwrap();

    let output_bytes = coord1.generate(&nachricht1).unwrap();

    let mut coord2 = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht2 = coord2.parse_nachricht(&output_bytes).unwrap();

    // Compare envelope
    assert_eq!(nachricht1.dokumentennummer, nachricht2.dokumentennummer);
    assert_eq!(nachricht1.kategorie, nachricht2.kategorie);
    assert_eq!(
        nachricht1.nachrichtendaten.absender_mp_id,
        nachricht2.nachrichtendaten.absender_mp_id
    );
    assert_eq!(
        nachricht1.nachrichtendaten.empfaenger_mp_id,
        nachricht2.nachrichtendaten.empfaenger_mp_id
    );
    assert_eq!(
        nachricht1.nachrichtendaten.datenaustauschreferenz,
        nachricht2.nachrichtendaten.datenaustauschreferenz
    );
    assert_eq!(
        nachricht1.nachrichtendaten.nachrichtenreferenz,
        nachricht2.nachrichtendaten.nachrichtenreferenz
    );

    // Compare transactions
    assert_eq!(
        nachricht1.transaktionen.len(),
        nachricht2.transaktionen.len()
    );

    let tx1 = &nachricht1.transaktionen[0];
    let tx2 = &nachricht2.transaktionen[0];

    assert_eq!(tx1.transaktions_id, tx2.transaktions_id);
    assert_eq!(tx1.marktlokationen.len(), tx2.marktlokationen.len());
    assert_eq!(tx1.messlokationen.len(), tx2.messlokationen.len());
    assert_eq!(tx1.netzlokationen.len(), tx2.netzlokationen.len());
    assert_eq!(
        tx1.prozessdaten.transaktionsgrund,
        tx2.prozessdaten.transaktionsgrund
    );

    // Compare location IDs
    assert_eq!(
        tx1.marktlokationen[0].data.marktlokations_id,
        tx2.marktlokationen[0].data.marktlokations_id
    );
    assert_eq!(
        tx1.messlokationen[0].data.messlokations_id,
        tx2.messlokationen[0].data.messlokations_id
    );
    assert_eq!(
        tx1.netzlokationen[0].data.netzlokations_id,
        tx2.netzlokationen[0].data.netzlokations_id
    );
}

// ---------------------------------------------------------------------------
// 2. Construct-generate roundtrip: build BO4E → generate → parse → compare
// ---------------------------------------------------------------------------

#[test]
fn test_bo4e_roundtrip_construct_and_generate() {
    let dt = NaiveDate::from_ymd_opt(2025, 7, 1)
        .unwrap()
        .and_hms_opt(13, 30, 0)
        .unwrap();

    let nachricht = UtilmdNachricht {
        dokumentennummer: "DOCGEN001".to_string(),
        kategorie: Some("E03".to_string()),
        nachrichtendaten: Nachrichtendaten {
            dokumentennummer: Some("DOCGEN001".to_string()),
            nachrichtenreferenz: Some("MSGGEN001".to_string()),
            absender_mp_id: Some("9900111000001".to_string()),
            empfaenger_mp_id: Some("9900222000002".to_string()),
            erstellungsdatum: Some(dt),
            datenaustauschreferenz: Some("DREF001".to_string()),
            ..Default::default()
        },
        transaktionen: vec![UtilmdTransaktion {
            transaktions_id: "TX_GEN_001".to_string(),
            prozessdaten: Prozessdaten {
                transaktionsgrund: Some("E01".to_string()),
                transaktionsgrund_ergaenzung: Some("Z14".to_string()),
                prozessdatum: Some(dt),
                wirksamkeitsdatum: Some(dt),
                bemerkung: Some("Testbemerkung".to_string()),
                referenz_vorgangsnummer: Some("VG12345".to_string()),
                ..Default::default()
            },
            marktlokationen: vec![WithValidity {
                data: Marktlokation {
                    marktlokations_id: Some("DE00014545768S0000000000000003054".to_string()),
                    ..Default::default()
                },
                edifact: MarktlokationEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            messlokationen: vec![WithValidity {
                data: Messlokation {
                    messlokations_id: Some("DE001234567890MELO".to_string()),
                    ..Default::default()
                },
                edifact: MesslokationEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            netzlokationen: vec![WithValidity {
                data: Netzlokation {
                    netzlokations_id: Some("NELO_GEN_001".to_string()),
                    ..Default::default()
                },
                edifact: NetzlokationEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            zaehler: vec![WithValidity {
                data: Zaehler {
                    zaehlernummer: Some("ZAEHLER_GEN_001".to_string()),
                    ..Default::default()
                },
                edifact: ZaehlerEdifact {
                    referenz_messlokation: Some("DE001234567890MELO".to_string()),
                    ..Default::default()
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            zeitscheiben: vec![Zeitscheibe {
                zeitscheiben_id: "1".to_string(),
                gueltigkeitszeitraum: Some(Zeitraum::new(
                    Some(dt),
                    NaiveDate::from_ymd_opt(2025, 12, 31)
                        .unwrap()
                        .and_hms_opt(0, 0, 0),
                )),
            }],
            parteien: vec![WithValidity {
                data: Geschaeftspartner {
                    name1: Some("Test GmbH".to_string()),
                    ..Default::default()
                },
                edifact: GeschaeftspartnerEdifact {
                    nad_qualifier: Some("Z09".to_string()),
                    ..Default::default()
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            ..Default::default()
        }],
    };

    // Generate EDIFACT
    let coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let output_bytes = coord.generate(&nachricht).unwrap();
    let output = String::from_utf8(output_bytes.clone()).unwrap();

    // Verify all entities appear
    assert!(output.contains("BGM+E03+DOCGEN001'"));
    assert!(output.contains("DTM+137:202507011330:303'"));
    assert!(output.contains("NAD+MS+9900111000001::293'"));
    assert!(output.contains("NAD+MR+9900222000002::293'"));
    assert!(output.contains("IDE+24+TX_GEN_001'"));
    assert!(output.contains("STS+7+E01+Z14'"));
    assert!(output.contains("FTX+ACB+++Testbemerkung'"));
    assert!(output.contains("LOC+Z18+NELO_GEN_001'"));
    assert!(output.contains("LOC+Z16+DE00014545768S0000000000000003054'"));
    assert!(output.contains("LOC+Z17+DE001234567890MELO'"));
    assert!(output.contains("RFF+Z13:VG12345'"));
    assert!(output.contains("RFF+Z47:1'"));
    assert!(output.contains("DTM+Z25:202507011330:303'"));
    assert!(output.contains("DTM+Z26:202512310000:303'"));
    assert!(output.contains("SEQ+Z03'"));
    assert!(output.contains("PIA+5+ZAEHLER_GEN_001'"));
    assert!(output.contains("RFF+Z19:DE001234567890MELO'"));
    assert!(output.contains("NAD+Z09+Test GmbH'"));

    // Verify MIG segment order within the transaction:
    // IDE should come before DTM, DTM before STS, STS before FTX,
    // FTX before LOC, LOC before RFF, RFF before SEQ, SEQ before NAD (SG12)
    let ide_pos = output.find("IDE+24").unwrap();
    // Find the transaction-level DTM+137 (prozessdatum) — it's the one AFTER IDE,
    // distinct from the message-level DTM+137 (Nachrichtendatum, Counter=0030).
    let dtm_137_pos = output[ide_pos..]
        .find("DTM+137:")
        .map(|p| p + ide_pos)
        .unwrap();
    let sts_pos = output.find("STS+7+E01").unwrap();
    let ftx_pos = output.find("FTX+ACB").unwrap();
    let loc_z18_pos = output.find("LOC+Z18").unwrap();
    let loc_z16_pos = output.find("LOC+Z16").unwrap();
    let loc_z17_pos = output.find("LOC+Z17").unwrap();
    let rff_z13_pos = output.find("RFF+Z13").unwrap();
    let rff_z47_pos = output.find("RFF+Z47").unwrap();
    let seq_z03_pos = output.find("SEQ+Z03").unwrap();
    let nad_z09_pos = output.find("NAD+Z09").unwrap();

    // MIG Counter ordering assertions
    assert!(ide_pos < dtm_137_pos, "IDE (0190) before DTM (0230)");
    assert!(dtm_137_pos < sts_pos, "DTM (0230) before STS (0250)");
    assert!(sts_pos < ftx_pos, "STS (0250) before FTX (0280)");
    assert!(ftx_pos < loc_z18_pos, "FTX (0280) before LOC (0320)");
    // Within SG5: Z18 (Nr 00048) before Z16 (Nr 00049) before Z17 (Nr 00054)
    assert!(
        loc_z18_pos < loc_z16_pos,
        "LOC+Z18 (Nr 48) before LOC+Z16 (Nr 49)"
    );
    assert!(
        loc_z16_pos < loc_z17_pos,
        "LOC+Z16 (Nr 49) before LOC+Z17 (Nr 54)"
    );
    // SG6 after SG5
    assert!(loc_z17_pos < rff_z13_pos, "LOC (0320) before RFF (0350)");
    assert!(
        rff_z13_pos < rff_z47_pos,
        "RFF+Z13 (Nr 56) before RFF+Z47 (Nr 66)"
    );
    // SG8 after SG6
    assert!(rff_z47_pos < seq_z03_pos, "RFF (0350) before SEQ (0410)");
    // SG12 after SG8
    assert!(seq_z03_pos < nad_z09_pos, "SEQ (0410) before NAD (0570)");

    // Reparse and compare
    let mut coord2 = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht2 = coord2.parse_nachricht(&output_bytes).unwrap();

    assert_eq!(nachricht2.dokumentennummer, "DOCGEN001");
    assert_eq!(nachricht2.transaktionen.len(), 1);
    let tx2 = &nachricht2.transaktionen[0];
    assert_eq!(tx2.transaktions_id, "TX_GEN_001");
    assert_eq!(tx2.marktlokationen.len(), 1);
    assert_eq!(tx2.messlokationen.len(), 1);
    assert_eq!(tx2.netzlokationen.len(), 1);
    assert_eq!(tx2.zaehler.len(), 1);
    assert_eq!(
        tx2.marktlokationen[0].data.marktlokations_id,
        Some("DE00014545768S0000000000000003054".to_string())
    );
    assert_eq!(tx2.prozessdaten.transaktionsgrund, Some("E01".to_string()));
}

// ---------------------------------------------------------------------------
// 3. Multi-transaction roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_bo4e_roundtrip_multi_transaction() {
    let nachricht = UtilmdNachricht {
        dokumentennummer: "DOCMULTI".to_string(),
        kategorie: Some("E03".to_string()),
        nachrichtendaten: Nachrichtendaten {
            dokumentennummer: Some("DOCMULTI".to_string()),
            nachrichtenreferenz: Some("MSGMULTI".to_string()),
            absender_mp_id: Some("SENDER".to_string()),
            empfaenger_mp_id: Some("RECEIVER".to_string()),
            datenaustauschreferenz: Some("DREF".to_string()),
            ..Default::default()
        },
        transaktionen: vec![
            UtilmdTransaktion {
                transaktions_id: "TX1".to_string(),
                marktlokationen: vec![WithValidity {
                    data: Marktlokation {
                        marktlokations_id: Some("MALO_TX1".to_string()),
                        ..Default::default()
                    },
                    edifact: MarktlokationEdifact::default(),
                    gueltigkeitszeitraum: None,
                    zeitscheibe_ref: None,
                }],
                ..Default::default()
            },
            UtilmdTransaktion {
                transaktions_id: "TX2".to_string(),
                marktlokationen: vec![WithValidity {
                    data: Marktlokation {
                        marktlokations_id: Some("MALO_TX2".to_string()),
                        ..Default::default()
                    },
                    edifact: MarktlokationEdifact::default(),
                    gueltigkeitszeitraum: None,
                    zeitscheibe_ref: None,
                }],
                ..Default::default()
            },
        ],
    };

    let coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let output_bytes = coord.generate(&nachricht).unwrap();
    let output = String::from_utf8(output_bytes.clone()).unwrap();

    // Both transactions should appear
    assert!(output.contains("IDE+24+TX1'"));
    assert!(output.contains("IDE+24+TX2'"));
    assert!(output.contains("LOC+Z16+MALO_TX1'"));
    assert!(output.contains("LOC+Z16+MALO_TX2'"));

    // TX1 should come before TX2
    let tx1_pos = output.find("IDE+24+TX1'").unwrap();
    let tx2_pos = output.find("IDE+24+TX2'").unwrap();
    assert!(tx1_pos < tx2_pos, "TX1 before TX2");

    // Reparse
    let mut coord2 = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht2 = coord2.parse_nachricht(&output_bytes).unwrap();
    assert_eq!(nachricht2.transaktionen.len(), 2);
    assert_eq!(nachricht2.transaktionen[0].transaktions_id, "TX1");
    assert_eq!(nachricht2.transaktionen[1].transaktions_id, "TX2");
}

// ---------------------------------------------------------------------------
// 4. FV2510 roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_bo4e_roundtrip_fv2510() {
    let nachricht = UtilmdNachricht {
        dokumentennummer: "DOC2510".to_string(),
        kategorie: Some("E03".to_string()),
        nachrichtendaten: Nachrichtendaten {
            dokumentennummer: Some("DOC2510".to_string()),
            nachrichtenreferenz: Some("MSG2510".to_string()),
            absender_mp_id: Some("SENDER".to_string()),
            empfaenger_mp_id: Some("RECEIVER".to_string()),
            datenaustauschreferenz: Some("DREF".to_string()),
            ..Default::default()
        },
        transaktionen: vec![UtilmdTransaktion {
            transaktions_id: "TX_2510".to_string(),
            ..Default::default()
        }],
    };

    let coord = create_coordinator(FormatVersion::FV2510).unwrap();
    let output_bytes = coord.generate(&nachricht).unwrap();
    let output = String::from_utf8(output_bytes).unwrap();

    // Should use S2.2 message type
    assert!(
        output.contains("UTILMD:D:11A:UN:S2.2'"),
        "FV2510 should produce S2.2 message type"
    );
}

// ---------------------------------------------------------------------------
// 5. Fixture regression: parse → generate → no panics
// ---------------------------------------------------------------------------

fn fixture_dir() -> Option<std::path::PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest.join("../../example_market_communication_bo4e_transactions");
    if fixture_path.exists() {
        Some(fixture_path)
    } else {
        None
    }
}

fn collect_edi_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_edi_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "edi") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Parse all UTILMD fixtures → generate → no panics.
///
/// This is a regression test: it verifies that generate() does not panic
/// on any real-world UTILMD file, regardless of whether the output is
/// byte-identical (it won't be — not all entity types have writers yet).
#[test]
fn test_bo4e_roundtrip_fixture_regression() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let utilmd_dir = fixture_path.join("UTILMD");
    if !utilmd_dir.exists() {
        eprintln!("Skipping: UTILMD directory not found");
        return;
    }

    let files = collect_edi_files(&utilmd_dir);
    assert!(!files.is_empty(), "No UTILMD .edi files found");

    let mut generate_ok = 0;
    let mut generate_fail: Vec<String> = Vec::new();

    for file_path in &files {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);

        let fv = detect_format_version(&content).unwrap_or(FormatVersion::FV2504);
        let mut coord = match create_coordinator(fv) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let nachricht = match coord.parse_nachricht(&content) {
            Ok(n) => n,
            Err(_) => continue,
        };

        // Generate should not panic or error
        match coord.generate(&nachricht) {
            Ok(output_bytes) => {
                // Basic sanity: output should be non-empty and valid UTF-8
                assert!(
                    !output_bytes.is_empty(),
                    "generate() produced empty output for {}",
                    rel.display()
                );
                assert!(
                    std::str::from_utf8(&output_bytes).is_ok(),
                    "generate() produced invalid UTF-8 for {}",
                    rel.display()
                );
                generate_ok += 1;
            }
            Err(e) => {
                generate_fail.push(format!("{}: {}", rel.display(), e));
            }
        }
    }

    eprintln!(
        "BO4E roundtrip: {}/{} UTILMD files generated successfully",
        generate_ok,
        files.len()
    );

    if !generate_fail.is_empty() {
        panic!(
            "{} of {} files failed generate():\n{}",
            generate_fail.len(),
            files.len(),
            generate_fail.join("\n")
        );
    }
}

/// Parse → generate → reparse fixture files and compare key fields.
///
/// This is the structural equality test: after parse→generate→reparse,
/// the transaction IDs, location IDs, and entity counts should match.
#[test]
fn test_bo4e_roundtrip_fixture_reparse() {
    let fixture_path = match fixture_dir() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: fixture submodule not initialized");
            return;
        }
    };

    let utilmd_dir = fixture_path.join("UTILMD");
    if !utilmd_dir.exists() {
        eprintln!("Skipping: UTILMD directory not found");
        return;
    }

    let files = collect_edi_files(&utilmd_dir);
    let mut reparse_ok = 0;
    let mut reparse_fail: Vec<String> = Vec::new();

    for file_path in &files {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel = file_path.strip_prefix(&fixture_path).unwrap_or(file_path);

        let fv = detect_format_version(&content).unwrap_or(FormatVersion::FV2504);
        let mut coord1 = match create_coordinator(fv) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let nachricht1 = match coord1.parse_nachricht(&content) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let output_bytes = match coord1.generate(&nachricht1) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let mut coord2 = match create_coordinator(fv) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let nachricht2 = match coord2.parse_nachricht(&output_bytes) {
            Ok(n) => n,
            Err(e) => {
                reparse_fail.push(format!("{}: reparse error: {}", rel.display(), e));
                continue;
            }
        };

        // Compare transaction count
        if nachricht1.transaktionen.len() != nachricht2.transaktionen.len() {
            reparse_fail.push(format!(
                "{}: transaction count mismatch: {} vs {}",
                rel.display(),
                nachricht1.transaktionen.len(),
                nachricht2.transaktionen.len()
            ));
            continue;
        }

        let mut file_ok = true;
        for (i, (tx1, tx2)) in nachricht1
            .transaktionen
            .iter()
            .zip(nachricht2.transaktionen.iter())
            .enumerate()
        {
            if tx1.transaktions_id != tx2.transaktions_id {
                reparse_fail.push(format!(
                    "{} tx[{}]: transaktions_id '{}' vs '{}'",
                    rel.display(),
                    i,
                    tx1.transaktions_id,
                    tx2.transaktions_id
                ));
                file_ok = false;
            }

            // Compare location counts
            if tx1.marktlokationen.len() != tx2.marktlokationen.len() {
                reparse_fail.push(format!(
                    "{} tx[{}]: marktlokationen count {} vs {}",
                    rel.display(),
                    i,
                    tx1.marktlokationen.len(),
                    tx2.marktlokationen.len()
                ));
                file_ok = false;
            }

            // Compare marktlokation IDs
            for (j, (ml1, ml2)) in tx1
                .marktlokationen
                .iter()
                .zip(tx2.marktlokationen.iter())
                .enumerate()
            {
                if ml1.data.marktlokations_id != ml2.data.marktlokations_id {
                    reparse_fail.push(format!(
                        "{} tx[{}].malo[{}]: '{}' vs '{}'",
                        rel.display(),
                        i,
                        j,
                        ml1.data.marktlokations_id.as_deref().unwrap_or(""),
                        ml2.data.marktlokations_id.as_deref().unwrap_or("")
                    ));
                    file_ok = false;
                }
            }
        }

        if file_ok {
            reparse_ok += 1;
        }
    }

    eprintln!(
        "BO4E reparse: {}/{} UTILMD files roundtripped successfully",
        reparse_ok,
        files.len()
    );

    if !reparse_fail.is_empty() {
        panic!(
            "{} reparse issues:\n{}",
            reparse_fail.len(),
            reparse_fail.join("\n")
        );
    }
}
