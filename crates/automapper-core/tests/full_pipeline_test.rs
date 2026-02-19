//! Full pipeline integration test: detect version -> parse -> write -> verify.
//!
//! Tests the complete flow from raw EDIFACT bytes through all layers.

use automapper_core::writer::entity_writers::*;
use automapper_core::{
    convert_batch, create_coordinator, detect_format_version, EdifactDocumentWriter, FormatVersion,
};

const FULL_UTILMD: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+GEN0001'\
UNH+GEN0001MSG+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202506190130:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
STS+E01+E01::Z44'\
RFF+Z13:VORGANGS001'\
RFF+Z49:1'\
DTM+Z25:202507010000:303'\
DTM+Z26:202512310000:303'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z17+DE00098765432100000000000000012'\
LOC+Z18+NELO00000000001'\
NAD+Z04+9900999000003::293'\
SEQ+Z03'\
PIA+5+ZAEHLER001'\
RFF+Z19:DE00098765432100000000000000012'\
SEQ+Z18'\
CCI+Z15++Z01'\
FTX+ACB+++Testbemerkung'\
UNT+23+GEN0001MSG'\
UNZ+1+GEN0001'";

#[test]
fn test_full_pipeline_detect_parse_write() {
    // Step 1: Detect format version
    let fv = detect_format_version(FULL_UTILMD).unwrap();
    assert_eq!(fv, FormatVersion::FV2504);

    // Step 2: Parse with coordinator
    let mut coord = create_coordinator(fv).unwrap();
    let transactions = coord.parse(FULL_UTILMD).unwrap();

    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];

    // Step 3: Verify parsed data
    assert_eq!(tx.transaktions_id, "TXID001");
    assert_eq!(tx.absender.mp_id, Some("9900123000002".to_string()));
    assert_eq!(tx.empfaenger.mp_id, Some("9900456000001".to_string()));
    assert_eq!(tx.prozessdaten.transaktionsgrund, Some("E01".to_string()));
    assert_eq!(
        tx.prozessdaten.referenz_vorgangsnummer,
        Some("VORGANGS001".to_string())
    );
    assert_eq!(tx.prozessdaten.bemerkung, Some("Testbemerkung".to_string()));
    assert_eq!(tx.zeitscheiben.len(), 1);
    assert_eq!(tx.zeitscheiben[0].zeitscheiben_id, "1");
    assert_eq!(tx.marktlokationen.len(), 1);
    assert_eq!(tx.messlokationen.len(), 1);
    assert_eq!(tx.netzlokationen.len(), 1);
    assert_eq!(tx.parteien.len(), 1);
    assert_eq!(tx.zaehler.len(), 1);
    assert_eq!(
        tx.zaehler[0].data.zaehlernummer,
        Some("ZAEHLER001".to_string())
    );
    assert!(tx.vertrag.is_some());
    assert_eq!(
        tx.vertrag.as_ref().unwrap().edifact.haushaltskunde,
        Some(true)
    );

    // Step 4: Write back to EDIFACT
    let mut doc = EdifactDocumentWriter::new();
    doc.begin_interchange(
        "9900123000002",
        Some("500"),
        "9900456000001",
        Some("500"),
        "GEN0001",
        "251217",
        "1229",
        true,
    );
    doc.begin_message("GEN0001MSG", "UTILMD:D:11A:UN:S2.1");

    doc.write_segment("BGM", &["E03", "DOC001"]);
    doc.write_segment_with_composites("DTM", &[&["137", "202506190130", "303"]]);
    doc.write_segment("NAD", &["MS", "9900123000002::293"]);
    doc.write_segment("NAD", &["MR", "9900456000001::293"]);
    doc.write_segment("IDE", &["24", &tx.transaktions_id]);
    doc.write_segment("STS", &["E01", "E01::Z44"]);
    doc.write_segment_with_composites("RFF", &[&["Z13", "VORGANGS001"]]);

    for ml in &tx.marktlokationen {
        MarktlokationWriter::write(&mut doc, ml);
    }
    for ml in &tx.messlokationen {
        MesslokationWriter::write(&mut doc, ml);
    }
    for nl in &tx.netzlokationen {
        NetzlokationWriter::write(&mut doc, nl);
    }
    for gp in &tx.parteien {
        GeschaeftspartnerWriter::write(&mut doc, gp);
    }
    for z in &tx.zaehler {
        ZaehlerWriter::write(&mut doc, z);
    }
    if let Some(ref v) = tx.vertrag {
        VertragWriter::write(&mut doc, v);
    }
    doc.write_segment("FTX", &["ACB", "", "", "Testbemerkung"]);

    doc.end_message();
    doc.end_interchange();

    // Step 5: Verify output contains key segments
    let output = doc.output();
    assert!(output.contains("UNA:+.? "), "should have UNA");
    assert!(output.contains("UNB+"), "should have UNB");
    assert!(output.contains("UNH+"), "should have UNH");
    assert!(
        output.contains("LOC+Z16+DE00014545768S0000000000000003054'"),
        "should have MALO"
    );
    assert!(
        output.contains("LOC+Z17+DE00098765432100000000000000012'"),
        "should have MELO"
    );
    assert!(
        output.contains("LOC+Z18+NELO00000000001'"),
        "should have NELO"
    );
    assert!(output.contains("SEQ+Z03'"), "should have Zaehler SEQ");
    assert!(
        output.contains("PIA+5+ZAEHLER001'"),
        "should have Zaehler PIA"
    );
    assert!(output.contains("SEQ+Z18'"), "should have Vertrag SEQ");
    assert!(
        output.contains("CCI+Z15++Z01'"),
        "should have Haushaltskunde"
    );
    assert!(output.contains("UNT+"), "should have UNT");
    assert!(output.contains("UNZ+1+GEN0001'"), "should have UNZ");
}

#[test]
fn test_batch_processing_produces_correct_results() {
    let inputs: Vec<&[u8]> = vec![FULL_UTILMD; 5];
    let results = convert_batch(&inputs, FormatVersion::FV2504);

    assert_eq!(results.len(), 5);
    for result in &results {
        let txs = result.as_ref().unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].transaktions_id, "TXID001");
        assert_eq!(txs[0].marktlokationen.len(), 1);
        assert_eq!(txs[0].zaehler.len(), 1);
    }
}
