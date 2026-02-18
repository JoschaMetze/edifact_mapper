//! Roundtrip integration tests: EDIFACT -> BO4E -> EDIFACT.
//!
//! Tests verify that parse -> map -> write produces output matching
//! the original input. For full byte-identical roundtrip, the writer
//! must reproduce all segment details from the parsed data.

use automapper_core::{
    create_coordinator, detect_format_version, Coordinator, EdifactDocumentWriter, FormatVersion,
    UtilmdCoordinator, FV2504,
};
use automapper_core::writer::entity_writers::*;

/// A minimal EDIFACT interchange for roundtrip testing.
///
/// This is intentionally simple to test the roundtrip mechanism.
/// Full roundtrip with real fixture files will be added when all
/// mappers and writers are feature-complete.
const SIMPLE_INTERCHANGE: &[u8] = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'IDE+24+TXID001'LOC+Z16+MALO001'LOC+Z17+MELO001'UNT+5+MSG001'UNZ+1+REF001'";

#[test]
fn test_roundtrip_simple_locations() {
    // Step 1: Parse EDIFACT -> BO4E
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let transactions = coord.parse(SIMPLE_INTERCHANGE).unwrap();
    assert_eq!(transactions.len(), 1);

    let tx = &transactions[0];

    // Step 2: Generate BO4E -> EDIFACT
    let mut doc = EdifactDocumentWriter::new();
    doc.begin_interchange("SENDER", "RECEIVER", "REF001", "251217", "1229");
    doc.begin_message("MSG001", "UTILMD:D:11A:UN:S2.1");

    doc.write_segment("BGM", &["E03", "DOC001"]);
    doc.write_segment("IDE", &["24", &tx.transaktions_id]);

    // Write locations
    for ml in &tx.marktlokationen {
        MarktlokationWriter::write(&mut doc, ml);
    }
    for ml in &tx.messlokationen {
        MesslokationWriter::write(&mut doc, ml);
    }

    doc.end_message();
    doc.end_interchange();

    // Step 3: Verify key segments are present
    let output = doc.output();
    assert!(
        output.contains("LOC+Z16+MALO001'"),
        "output should contain Marktlokation LOC"
    );
    assert!(
        output.contains("LOC+Z17+MELO001'"),
        "output should contain Messlokation LOC"
    );
    assert!(
        output.contains("IDE+24+TXID001'"),
        "output should contain transaction ID"
    );
    // UNH + BGM + IDE + LOC + LOC + UNT = 6
    assert!(
        output.contains("UNT+6+MSG001'"),
        "output should have correct segment count in UNT"
    );
    assert!(
        output.contains("UNZ+1+REF001'"),
        "output should have correct message count in UNZ"
    );
}

#[test]
fn test_roundtrip_detect_version_and_parse() {
    let fv = detect_format_version(SIMPLE_INTERCHANGE).unwrap();
    assert_eq!(fv, FormatVersion::FV2504);

    let mut coord = create_coordinator(fv).unwrap();
    let transactions = coord.parse(SIMPLE_INTERCHANGE).unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].transaktions_id, "TXID001");
}

#[test]
fn test_roundtrip_preserves_location_ids() {
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let transactions = coord.parse(SIMPLE_INTERCHANGE).unwrap();
    let tx = &transactions[0];

    // Verify forward mapping extracted correct IDs
    assert_eq!(
        tx.marktlokationen[0].data.marktlokations_id,
        Some("MALO001".to_string())
    );
    assert_eq!(
        tx.messlokationen[0].data.messlokations_id,
        Some("MELO001".to_string())
    );

    // Generate back and verify IDs survive the roundtrip
    let mut doc = EdifactDocumentWriter::with_delimiters(
        edifact_types::EdifactDelimiters::default(),
        false,
    );
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");
    MarktlokationWriter::write(&mut doc, &tx.marktlokationen[0]);
    MesslokationWriter::write(&mut doc, &tx.messlokationen[0]);
    doc.end_message();
    doc.end_interchange();

    let output = doc.output();
    assert!(output.contains("LOC+Z16+MALO001'"));
    assert!(output.contains("LOC+Z17+MELO001'"));
}
