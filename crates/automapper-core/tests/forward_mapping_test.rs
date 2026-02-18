//! Integration tests for UTILMD forward mapping (EDIFACT -> BO4E).
//!
//! Tests parse synthetic but realistic EDIFACT messages and verify
//! the resulting UtilmdTransaktion structure.

use automapper_core::{create_coordinator, Coordinator, FormatVersion, UtilmdCoordinator, FV2504};

/// A synthetic UTILMD message with multiple entity types.
const SYNTHETIC_UTILMD: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+GEN0001'\
UNH+GEN0001MSG+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202506190130:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
STS+E01+E01::Z44'\
DTM+137:202507010000:303'\
DTM+471:202508010000:303'\
RFF+Z13:VORGANGS001'\
RFF+Z49:1'\
DTM+Z25:202507010000:303'\
DTM+Z26:202512310000:303'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z17+DE00098765432100000000000000012'\
LOC+Z18+NELO00000000001'\
NAD+Z04+9900999000003::293'\
FTX+ACB+++Testbemerkung'\
UNT+18+GEN0001MSG'\
UNZ+1+GEN0001'";

#[test]
fn test_forward_mapping_synthetic_utilmd() {
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let result = coord.parse(SYNTHETIC_UTILMD).unwrap();

    assert_eq!(result.len(), 1, "should produce one transaction");
    let tx = &result[0];

    // Transaction ID
    assert_eq!(tx.transaktions_id, "TXID001");

    // Absender/Empfaenger
    assert_eq!(tx.absender.mp_id, Some("9900123000002".to_string()));
    assert_eq!(tx.empfaenger.mp_id, Some("9900456000001".to_string()));

    // Prozessdaten
    assert_eq!(tx.prozessdaten.transaktionsgrund, Some("E01".to_string()));
    assert!(tx.prozessdaten.prozessdatum.is_some());
    assert!(tx.prozessdaten.wirksamkeitsdatum.is_some());
    assert_eq!(
        tx.prozessdaten.referenz_vorgangsnummer,
        Some("VORGANGS001".to_string())
    );
    assert_eq!(tx.prozessdaten.bemerkung, Some("Testbemerkung".to_string()));

    // Zeitscheiben
    assert_eq!(tx.zeitscheiben.len(), 1);
    assert_eq!(tx.zeitscheiben[0].zeitscheiben_id, "1");
    assert!(tx.zeitscheiben[0].gueltigkeitszeitraum.is_some());

    // Marktlokation
    assert_eq!(tx.marktlokationen.len(), 1);
    assert_eq!(
        tx.marktlokationen[0].data.marktlokations_id,
        Some("DE00014545768S0000000000000003054".to_string())
    );

    // Messlokation
    assert_eq!(tx.messlokationen.len(), 1);
    assert_eq!(
        tx.messlokationen[0].data.messlokations_id,
        Some("DE00098765432100000000000000012".to_string())
    );

    // Netzlokation
    assert_eq!(tx.netzlokationen.len(), 1);
    assert_eq!(
        tx.netzlokationen[0].data.netzlokations_id,
        Some("NELO00000000001".to_string())
    );

    // Geschaeftspartner
    assert_eq!(tx.parteien.len(), 1);
    assert_eq!(
        tx.parteien[0].edifact.nad_qualifier,
        Some("Z04".to_string())
    );
}

#[test]
fn test_forward_mapping_via_create_coordinator() {
    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let result = coord.parse(SYNTHETIC_UTILMD).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].transaktions_id, "TXID001");
}

#[test]
fn test_forward_mapping_empty_input() {
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let result = coord.parse(b"").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_forward_mapping_nad_sparte() {
    let input = b"UNA:+.? 'UNB+UNOC:3+S+R+D+REF'UNH+M+UTILMD:D:11A:UN:S2.1'BGM+E03+D'NAD+MS+9900123::293'IDE+24+TX1'LOC+Z16+MALO1'UNT+6+M'UNZ+1+REF'";
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let result = coord.parse(input).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].marktlokationen[0].data.sparte,
        Some("STROM".to_string())
    );
}
