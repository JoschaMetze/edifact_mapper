//! BO4E roundtrip integration tests: EDIFACT → parse → BO4E → generate → EDIFACT.
//!
//! Tests the complete bidirectional mapping pipeline through the Coordinator API.
//! Segment ordering in generate() follows the MIG XML Counter attributes.
//! See `docs/mig-segment-ordering.md` for the derivation.

use automapper_core::{create_coordinator, detect_format_version, FormatVersion};
use bo4e_extensions::{
    Bilanzierung, BilanzierungEdifact, Geschaeftspartner, GeschaeftspartnerEdifact,
    Lokationszuordnung, LokationszuordnungEdifact, MabisZaehlpunkt, MabisZaehlpunktEdifact,
    Marktlokation, MarktlokationEdifact, Messlokation, MesslokationEdifact, Nachrichtendaten,
    Netzlokation, NetzlokationEdifact, Produktpaket, ProduktpaketEdifact, Prozessdaten,
    SteuerbareRessource, SteuerbareRessourceEdifact, TechnischeRessource,
    TechnischeRessourceEdifact, Tranche, TrancheEdifact, UtilmdNachricht, UtilmdTransaktion,
    Vertrag, VertragEdifact, WithValidity, Zaehler, ZaehlerEdifact, Zeitraum, Zeitscheibe,
};
use chrono::NaiveDate;
use std::path::Path;

// ---------------------------------------------------------------------------
// 1. Synthetic roundtrip: construct EDIFACT → parse → generate → verify
// ---------------------------------------------------------------------------

/// Minimal EDIFACT message with envelope + one transaction containing
/// all LOC-based entity types.
const MINIMAL_EDIFACT: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002+9900456000001+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202507011330:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
LOC+Z18+NELO001'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z20+TECRES001'\
LOC+Z19+STRES001'\
LOC+Z21+TRANCHE001'\
LOC+Z17+MELO001'\
LOC+Z15+MABIS001'\
STS+7++E01'\
UNT+16+MSG001'\
UNZ+1+REF001'";

#[test]
fn test_envelope_fields_roundtrip() {
    let edifact = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202507011330:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TX001'\
UNT+8+MSG001'\
UNZ+1+REF001'";

    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(edifact).unwrap();

    let nd = &nachricht.nachrichtendaten;
    assert_eq!(nd.absender_unb_qualifier.as_deref(), Some("500"));
    assert_eq!(nd.empfaenger_unb_qualifier.as_deref(), Some("500"));
    assert_eq!(nd.unb_datum.as_deref(), Some("251217"));
    assert_eq!(nd.unb_zeit.as_deref(), Some("1229"));
    assert!(nd.explicit_una);
    assert_eq!(nd.nachrichtentyp.as_deref(), Some("UTILMD:D:11A:UN:S2.1"));
    assert_eq!(
        nd.erstellungsdatum
            .unwrap()
            .format("%Y%m%d%H%M")
            .to_string(),
        "202507011330"
    );
}

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
    assert_eq!(tx.steuerbare_ressourcen.len(), 1);
    assert_eq!(tx.technische_ressourcen.len(), 1);
    assert_eq!(tx.tranchen.len(), 1);
    assert_eq!(tx.mabis_zaehlpunkte.len(), 1);
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
    assert!(output.contains("STS+7++E01'"), "STS transaktionsgrund");
    assert!(output.contains("LOC+Z18+NELO001'"), "LOC+Z18 netzlokation");
    assert!(
        output.contains("LOC+Z16+DE00014545768S0000000000000003054'"),
        "LOC+Z16 marktlokation"
    );
    assert!(
        output.contains("LOC+Z20+TECRES001'"),
        "LOC+Z20 technische_ressource"
    );
    assert!(
        output.contains("LOC+Z19+STRES001'"),
        "LOC+Z19 steuerbare_ressource"
    );
    assert!(output.contains("LOC+Z21+TRANCHE001'"), "LOC+Z21 tranche");
    assert!(output.contains("LOC+Z17+MELO001'"), "LOC+Z17 messlokation");
    assert!(
        output.contains("LOC+Z15+MABIS001'"),
        "LOC+Z15 mabis_zaehlpunkt"
    );
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
        tx1.steuerbare_ressourcen.len(),
        tx2.steuerbare_ressourcen.len()
    );
    assert_eq!(
        tx1.technische_ressourcen.len(),
        tx2.technische_ressourcen.len()
    );
    assert_eq!(tx1.tranchen.len(), tx2.tranchen.len());
    assert_eq!(tx1.mabis_zaehlpunkte.len(), tx2.mabis_zaehlpunkte.len());
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
            steuerbare_ressourcen: vec![WithValidity {
                data: SteuerbareRessource {
                    steuerbare_ressource_id: Some("STRES_GEN_001".to_string()),
                },
                edifact: SteuerbareRessourceEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            technische_ressourcen: vec![WithValidity {
                data: TechnischeRessource {
                    technische_ressource_id: Some("TECRES_GEN_001".to_string()),
                },
                edifact: TechnischeRessourceEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            tranchen: vec![WithValidity {
                data: Tranche {
                    tranche_id: Some("TRANCHE_GEN_001".to_string()),
                },
                edifact: TrancheEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            mabis_zaehlpunkte: vec![WithValidity {
                data: MabisZaehlpunkt {
                    zaehlpunkt_id: Some("MABIS_GEN_001".to_string()),
                },
                edifact: MabisZaehlpunktEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            bilanzierung: Some(WithValidity {
                data: Bilanzierung {
                    bilanzkreis: Some("11YN20---------Z".to_string()),
                    regelzone: None,
                    bilanzierungsgebiet: None,
                },
                edifact: BilanzierungEdifact {
                    jahresverbrauchsprognose: Some(12345.67),
                    temperatur_arbeit: None,
                    seq_qualifier: None,
                    seq_sub_id: None,
                    raw_qty: Vec::new(),
                    raw_segments: Vec::new(),
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }),
            produktpakete: vec![WithValidity {
                data: Produktpaket {
                    produktpaket_id: Some("PP_GEN_001".to_string()),
                },
                edifact: ProduktpaketEdifact {
                    produktpaket_name: Some("Grundversorgung".to_string()),
                    seq_qualifier: None,
                    raw_pia: None,
                    raw_cci_cav: Vec::new(),
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            lokationszuordnungen: vec![WithValidity {
                data: Lokationszuordnung {
                    marktlokations_id: Some("MALO_LZ_001".to_string()),
                    messlokations_id: Some("MELO_LZ_001".to_string()),
                },
                edifact: LokationszuordnungEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            vertrag: Some(WithValidity {
                data: Vertrag::default(),
                edifact: VertragEdifact {
                    haushaltskunde: Some(true),
                    versorgungsart: None,
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }),
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
    assert!(output.contains("STS+7++E01+Z14'"));
    assert!(output.contains("FTX+ACB+++Testbemerkung'"));
    assert!(output.contains("LOC+Z18+NELO_GEN_001'"));
    assert!(output.contains("LOC+Z16+DE00014545768S0000000000000003054'"));
    assert!(output.contains("LOC+Z17+DE001234567890MELO'"));
    assert!(output.contains("RFF+Z13:VG12345'"));
    assert!(output.contains("RFF+Z47:1'"));
    assert!(output.contains("DTM+Z25:202507011330:303'"));
    assert!(output.contains("DTM+Z26:202512310000:303'"));
    assert!(output.contains("SEQ+Z03'"));
    // PIA+5 is NOT written in Z03 groups — it belongs to Z02 groups
    assert!(output.contains("RFF+Z19:DE001234567890MELO'"));
    assert!(output.contains("NAD+Z09+Test GmbH'"));

    // New LOC segments
    assert!(output.contains("LOC+Z20+TECRES_GEN_001'"));
    assert!(output.contains("LOC+Z19+STRES_GEN_001'"));
    assert!(output.contains("LOC+Z21+TRANCHE_GEN_001'"));
    assert!(output.contains("LOC+Z15+MABIS_GEN_001'"));

    // New SEQ segments
    assert!(output.contains("SEQ+Z78'"));
    assert!(output.contains("RFF+Z18:MALO_LZ_001'"));
    assert!(output.contains("RFF+Z19:MELO_LZ_001'"));
    assert!(output.contains("SEQ+Z79+PP_GEN_001'"));
    assert!(output.contains("PIA+5+Grundversorgung'"));
    assert!(output.contains("SEQ+Z98'"));
    assert!(output.contains("CCI+Z20++11YN20---------Z'"));
    assert!(output.contains("QTY+Z09:12345.67'"));

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
    let sts_pos = output.find("STS+7++E01").unwrap();
    let ftx_pos = output.find("FTX+ACB").unwrap();
    let loc_z18_pos = output.find("LOC+Z18").unwrap();
    let loc_z16_pos = output.find("LOC+Z16").unwrap();
    let loc_z20_pos = output.find("LOC+Z20").unwrap();
    let loc_z19_pos = output.find("LOC+Z19").unwrap();
    let loc_z21_pos = output.find("LOC+Z21").unwrap();
    let loc_z17_pos = output.find("LOC+Z17").unwrap();
    let loc_z15_pos = output.find("LOC+Z15").unwrap();
    let rff_z13_pos = output.find("RFF+Z13").unwrap();
    let rff_z47_pos = output.find("RFF+Z47").unwrap();
    let seq_z78_pos = output.find("SEQ+Z78").unwrap();
    let seq_z79_pos = output.find("SEQ+Z79").unwrap();
    let seq_z98_pos = output.find("SEQ+Z98").unwrap();
    let seq_z03_pos = output.find("SEQ+Z03").unwrap();
    let nad_z09_pos = output.find("NAD+Z09").unwrap();

    // MIG Counter ordering assertions
    assert!(ide_pos < dtm_137_pos, "IDE (0190) before DTM (0230)");
    assert!(dtm_137_pos < sts_pos, "DTM (0230) before STS (0250)");
    assert!(sts_pos < ftx_pos, "STS (0250) before FTX (0280)");
    assert!(ftx_pos < loc_z18_pos, "FTX (0280) before LOC (0320)");
    // Full SG5 LOC MIG ordering: Z18 < Z16 < Z20 < Z19 < Z21 < Z17 < Z15
    assert!(
        loc_z18_pos < loc_z16_pos,
        "LOC+Z18 (Nr 48) before LOC+Z16 (Nr 49)"
    );
    assert!(
        loc_z16_pos < loc_z20_pos,
        "LOC+Z16 (Nr 49) before LOC+Z20 (Nr 51)"
    );
    assert!(
        loc_z20_pos < loc_z19_pos,
        "LOC+Z20 (Nr 51) before LOC+Z19 (Nr 52)"
    );
    assert!(
        loc_z19_pos < loc_z21_pos,
        "LOC+Z19 (Nr 52) before LOC+Z21 (Nr 53)"
    );
    assert!(
        loc_z21_pos < loc_z17_pos,
        "LOC+Z21 (Nr 53) before LOC+Z17 (Nr 54)"
    );
    assert!(
        loc_z17_pos < loc_z15_pos,
        "LOC+Z17 (Nr 54) before LOC+Z15 (Nr 55)"
    );
    // SG6 after SG5
    assert!(loc_z15_pos < rff_z13_pos, "LOC (0320) before RFF (0350)");
    assert!(
        rff_z13_pos < rff_z47_pos,
        "RFF+Z13 (Nr 56) before RFF+Z47 (Nr 66)"
    );
    // SG8 after SG6 (fallback ordering without seq_group_order):
    // Z79 (Produktpaket) → Z78 (Lokationszuordnung) → Z03 (Zaehler) → Z98 (Bilanzierung)
    assert!(rff_z47_pos < seq_z79_pos, "RFF (0350) before SEQ (0410)");
    assert!(
        seq_z79_pos < seq_z78_pos,
        "SEQ+Z79 before SEQ+Z78 (fallback order)"
    );
    assert!(
        seq_z78_pos < seq_z03_pos,
        "SEQ+Z78 before SEQ+Z03 (fallback order)"
    );
    assert!(
        seq_z03_pos < seq_z98_pos,
        "SEQ+Z03 before SEQ+Z98 (fallback order)"
    );
    // SG12 after SG8
    assert!(seq_z98_pos < nad_z09_pos, "SEQ (0410) before NAD (0570)");

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
    assert_eq!(tx2.steuerbare_ressourcen.len(), 1);
    assert_eq!(tx2.technische_ressourcen.len(), 1);
    assert_eq!(tx2.tranchen.len(), 1);
    assert_eq!(tx2.mabis_zaehlpunkte.len(), 1);
    assert!(tx2.bilanzierung.is_some());
    assert_eq!(tx2.produktpakete.len(), 1);
    assert_eq!(tx2.lokationszuordnungen.len(), 1);
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

// ---------------------------------------------------------------------------
// Normalization helpers for byte-identical comparison
// ---------------------------------------------------------------------------

/// Normalize an EDIFACT message for deterministic comparison.
///
/// Port of C# `RoundtripTestHelper.NormalizeEdifact`:
/// 1. Strip CR/LF between segments
/// 2. Split into individual segments
/// 3. Sort consecutive SEQ groups with the same qualifier by their first RFF
/// 4. Rejoin with segment terminator (no newlines)
fn normalize_edifact(input: &[u8]) -> String {
    let input_str = String::from_utf8_lossy(input);

    // Split on segment terminator, trim whitespace, drop empty
    let segments: Vec<&str> = input_str
        .split('\'')
        .map(|s| s.trim_matches(|c: char| c == '\r' || c == '\n' || c == ' '))
        .filter(|s| !s.is_empty())
        .collect();

    // Normalize SEQ group ordering: consecutive SEQ groups with the same
    // qualifier (e.g. multiple SEQ+Z03 blocks) can appear in any order per MIG.
    // Sort them by the content of their first RFF segment for determinism.
    let segments = normalize_seq_group_order(segments);

    // Rejoin: each segment terminated by '
    let mut result = String::new();
    for seg in &segments {
        result.push_str(seg);
        result.push('\'');
    }
    result
}

/// Identify consecutive SEQ groups with the same qualifier and sort them
/// by their first RFF value, matching the C# NormalizeSeqGroupOrder logic.
fn normalize_seq_group_order(segments: Vec<&str>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;

    while i < segments.len() {
        // Check if this is the start of a SEQ segment
        if !segments[i].starts_with("SEQ+") {
            result.push(segments[i].to_string());
            i += 1;
            continue;
        }

        // Extract the SEQ qualifier (e.g. "Z03" from "SEQ+Z03")
        let qualifier = segments[i]
            .strip_prefix("SEQ+")
            .unwrap_or("")
            .split('+')
            .next()
            .unwrap_or("");

        // Collect all consecutive SEQ groups with the same qualifier.
        // A SEQ group = the SEQ segment + all following non-SEQ segments
        // until the next SEQ, LOC, IDE, NAD (SG12), UNT, or UNZ.
        let mut groups: Vec<Vec<String>> = Vec::new();

        while i < segments.len()
            && segments[i].starts_with("SEQ+")
            && segments[i]
                .strip_prefix("SEQ+")
                .unwrap_or("")
                .starts_with(qualifier)
        {
            let mut group = vec![segments[i].to_string()];
            i += 1;

            // Collect child segments of this SEQ group
            while i < segments.len()
                && !segments[i].starts_with("SEQ+")
                && !segments[i].starts_with("LOC+")
                && !segments[i].starts_with("IDE+")
                && !segments[i].starts_with("NAD+")
                && !segments[i].starts_with("UNT+")
                && !segments[i].starts_with("UNZ+")
            {
                group.push(segments[i].to_string());
                i += 1;
            }

            groups.push(group);
        }

        if groups.len() > 1 {
            // Sort groups by their first RFF segment content (if any)
            groups.sort_by(|a, b| {
                let rff_a = a
                    .iter()
                    .find(|s| s.starts_with("RFF+"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let rff_b = b
                    .iter()
                    .find(|s| s.starts_with("RFF+"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                rff_a.cmp(rff_b)
            });
        }

        for group in groups {
            result.extend(group);
        }
    }

    result
}

/// Show a segment-level diff between two normalized EDIFACT strings.
fn edifact_diff(original: &str, generated: &str) -> String {
    let orig_segs: Vec<&str> = original.split('\'').filter(|s| !s.is_empty()).collect();
    let gen_segs: Vec<&str> = generated.split('\'').filter(|s| !s.is_empty()).collect();

    let mut diff = String::new();
    let max_len = orig_segs.len().max(gen_segs.len());

    for i in 0..max_len {
        let o = orig_segs.get(i).unwrap_or(&"<missing>");
        let g = gen_segs.get(i).unwrap_or(&"<missing>");
        if o != g {
            diff.push_str(&format!("  seg[{:3}] orig: {}'\n", i, o));
            diff.push_str(&format!("  seg[{:3}]  gen: {}'\n", i, g));
        }
    }

    if diff.is_empty() && orig_segs.len() != gen_segs.len() {
        diff.push_str(&format!(
            "  segment count: orig={} gen={}\n",
            orig_segs.len(),
            gen_segs.len()
        ));
    }

    diff
}

// ---------------------------------------------------------------------------
// 5. Fixture byte-identical roundtrip: parse → generate → normalize → compare
// ---------------------------------------------------------------------------

/// Parse UTILMD fixtures → generate → compare normalized EDIFACT byte-for-byte.
///
/// Port of C# `UtilmdRoundtripTests.Roundtrip_Should_Produce_Identical_EDIFACT`.
/// Both original and generated output are normalized (strip newlines, sort
/// SEQ groups with the same qualifier) before comparison.
#[test]
fn test_bo4e_roundtrip_fixture_byte_identical() {
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

    let mut identical = 0;
    let mut failures: Vec<String> = Vec::new();

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

        let output_bytes = match coord.generate(&nachricht) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let normalized_original = normalize_edifact(&content);
        let normalized_generated = normalize_edifact(&output_bytes);

        if normalized_original == normalized_generated {
            identical += 1;
        } else {
            let diff = edifact_diff(&normalized_original, &normalized_generated);
            failures.push(format!("{}:\n{}", rel.display(), diff));

            // Collect first-diff segment info for diagnostics
            let orig_segs: Vec<&str> = normalized_original
                .split('\'')
                .filter(|s| !s.is_empty())
                .collect();
            let gen_segs: Vec<&str> = normalized_generated
                .split('\'')
                .filter(|s| !s.is_empty())
                .collect();
            for (i, (o, g)) in orig_segs.iter().zip(gen_segs.iter()).enumerate() {
                if o != g {
                    let o_tag = o.split('+').next().unwrap_or("?");
                    let _g_tag = g.split('+').next().unwrap_or("?");
                    eprintln!(
                        "FIRST_DIFF|{}|{}|seg[{}]|orig:{}|gen:{}",
                        rel.display(),
                        o_tag,
                        i,
                        o,
                        g
                    );
                    break;
                }
            }
        }
    }

    eprintln!(
        "BO4E byte-identical roundtrip: {}/{} UTILMD files match",
        identical,
        files.len()
    );

    // Known unfixable failures:
    // - 2 Latin-1 encoded files (parser converts to UTF-8, losing original encoding)
    // - 1 water CCI format with CCI/CAV before NAD at transaction level
    // - 1 multi-MaLo with per-MaLo SEQ group references (complex structural issue)
    let known_failures = 4;
    if failures.len() > known_failures {
        let shown = failures.len().min(5);
        panic!(
            "{} of {} files differ after roundtrip (expected at most {}).\nFirst {}:\n{}",
            failures.len(),
            files.len(),
            known_failures,
            shown,
            failures[..shown].join("\n")
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Fixture regression: parse → generate → no panics
// ---------------------------------------------------------------------------

/// Parse all UTILMD fixtures → generate → no panics.
///
/// Regression test: verifies that generate() does not panic on any
/// real-world UTILMD file.
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

// ---------------------------------------------------------------------------
// 7. Envelope writer preserves UNB qualifiers, date/time, UNA, UNH type
// ---------------------------------------------------------------------------

#[test]
fn test_envelope_writer_preserves_qualifiers() {
    // No UNA in original
    let edifact = b"UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TX001'\
UNT+6+MSG001'\
UNZ+1+REF001'";

    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(edifact).unwrap();
    let output = String::from_utf8(coord.generate(&nachricht).unwrap()).unwrap();

    // No UNA (original had none)
    assert!(
        output.starts_with("UNB+"),
        "should NOT start with UNA when original had none"
    );
    // UNB should preserve :500 qualifiers and date/time
    assert!(
        output.contains("UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'"),
        "UNB should preserve qualifiers and date/time, got: {}",
        output
    );
    // UNH should preserve original message type
    assert!(
        output.contains("UNH+MSG001+UTILMD:D:11A:UN:S2.1'"),
        "UNH should preserve message type"
    );
}

#[test]
fn test_envelope_writer_preserves_una() {
    let edifact = b"UNA:+.? '\
UNB+UNOC:3+SENDER+RECEIVER+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
IDE+24+TX001'\
UNT+4+MSG001'\
UNZ+1+REF001'";

    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(edifact).unwrap();
    let output = String::from_utf8(coord.generate(&nachricht).unwrap()).unwrap();

    assert!(
        output.starts_with("UNA:+.? '"),
        "should start with UNA when original had one"
    );
}

// ---------------------------------------------------------------------------
// 8. Fixture structural roundtrip: parse → generate → reparse → compare fields
// ---------------------------------------------------------------------------

/// Parse → generate → reparse fixture files and compare key fields.
///
/// Structural equality test: after parse→generate→reparse,
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

            // Compare all entity counts
            let entity_checks = [
                (
                    "marktlokationen",
                    tx1.marktlokationen.len(),
                    tx2.marktlokationen.len(),
                ),
                (
                    "messlokationen",
                    tx1.messlokationen.len(),
                    tx2.messlokationen.len(),
                ),
                (
                    "netzlokationen",
                    tx1.netzlokationen.len(),
                    tx2.netzlokationen.len(),
                ),
                (
                    "steuerbare_ressourcen",
                    tx1.steuerbare_ressourcen.len(),
                    tx2.steuerbare_ressourcen.len(),
                ),
                (
                    "technische_ressourcen",
                    tx1.technische_ressourcen.len(),
                    tx2.technische_ressourcen.len(),
                ),
                ("tranchen", tx1.tranchen.len(), tx2.tranchen.len()),
                (
                    "mabis_zaehlpunkte",
                    tx1.mabis_zaehlpunkte.len(),
                    tx2.mabis_zaehlpunkte.len(),
                ),
                ("zaehler", tx1.zaehler.len(), tx2.zaehler.len()),
                (
                    "produktpakete",
                    tx1.produktpakete.len(),
                    tx2.produktpakete.len(),
                ),
                (
                    "lokationszuordnungen",
                    tx1.lokationszuordnungen.len(),
                    tx2.lokationszuordnungen.len(),
                ),
                ("parteien", tx1.parteien.len(), tx2.parteien.len()),
            ];
            for (name, c1, c2) in entity_checks {
                if c1 != c2 {
                    reparse_fail.push(format!(
                        "{} tx[{}]: {} count {} vs {}",
                        rel.display(),
                        i,
                        name,
                        c1,
                        c2
                    ));
                    file_ok = false;
                }
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

// ---------------------------------------------------------------------------
// Raw DTM roundtrip: format code and timezone preservation
// ---------------------------------------------------------------------------

#[test]
fn test_raw_dtm_roundtrip() {
    // DTM with timezone suffix (?+00) and format 102 (date-only)
    let edifact = b"UNA:+.? '\
UNB+UNOC:3+SENDER:500+RECEIVER:500+250331:1329+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202503311329?+00:303'\
NAD+MS+SENDER::293'\
NAD+MR+RECEIVER::293'\
IDE+24+TX001'\
DTM+92:20220624:102'\
STS+7++E01'\
UNT+9+MSG001'\
UNZ+1+REF001'";

    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(edifact).unwrap();
    let output = String::from_utf8(coord.generate(&nachricht).unwrap()).unwrap();

    // Message-level DTM+137 should preserve raw format including ?+00
    assert!(
        output.contains("DTM+137:202503311329?+00:303'"),
        "DTM+137 raw not preserved in: {}",
        output
    );
    // Transaction-level DTM+92 should preserve format 102
    assert!(
        output.contains("DTM+92:20220624:102'"),
        "DTM+92 format 102 not preserved in: {}",
        output
    );
}

// ---------------------------------------------------------------------------
// TEMPORARY: Diff pattern analysis for roundtrip failure triage
// ---------------------------------------------------------------------------

/// Extract the segment type identifier from a segment string.
/// E.g. "IMD++Z36+Z13" -> "IMD", "SEQ+Z01" -> "SEQ+Z01", "CCI+Z30++Z07" -> "CCI"
/// For SEQ and LOC segments, include the qualifier for finer-grained analysis.
fn extract_seg_id(seg: &str) -> String {
    let seg = seg.trim();
    if seg.is_empty() || seg == "<missing>" {
        return seg.to_string();
    }

    // Get the segment tag (first 3 chars before +)
    let tag = seg.split('+').next().unwrap_or(seg);

    // For SEQ, LOC, NAD, DTM, RFF — include the qualifier for more detail
    match tag {
        "SEQ" | "LOC" | "NAD" | "DTM" | "RFF" => {
            // Take tag + first qualifier element
            let parts: Vec<&str> = seg.split('+').collect();
            if parts.len() >= 2 {
                format!("{}+{}", parts[0], parts[1])
            } else {
                tag.to_string()
            }
        }
        _ => tag.to_string(),
    }
}

/// Analyze diff patterns across all UTILMD fixture files.
///
/// For each file that fails byte-identical roundtrip, collect:
/// 1. The FIRST differing segment (original side) — to understand what triggers divergence
/// 2. ALL differing segments — to understand the full scope of missing/reordered segments
///
/// Output: aggregate counts of segment types appearing in diffs.
#[test]
fn test_analyze_diff_patterns() {
    use std::collections::HashMap;

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

    let mut identical = 0usize;
    let mut total_differing = 0usize;
    let mut parse_errors = 0usize;
    let mut generate_errors = 0usize;

    // First-diff segment analysis: which segment type is the FIRST to diverge?
    let mut first_diff_orig_seg: HashMap<String, usize> = HashMap::new();
    // Which segment was generated instead?
    let mut first_diff_gen_seg: HashMap<String, usize> = HashMap::new();
    // First-diff as a pair: "orig_seg -> gen_seg"
    let mut first_diff_pair: HashMap<String, usize> = HashMap::new();

    // All-diff segment analysis: count every segment type that appears wrong
    let mut all_diff_orig_seg: HashMap<String, usize> = HashMap::new();
    let mut all_diff_gen_seg: HashMap<String, usize> = HashMap::new();

    // Segments present in original but missing from generated (extra in original)
    let mut missing_from_generated: HashMap<String, usize> = HashMap::new();
    // Segments present in generated but missing from original (extra in generated)
    let mut extra_in_generated: HashMap<String, usize> = HashMap::new();

    // Segment ordering issues: segments present in both but at different positions
    let mut ordering_issues: HashMap<String, usize> = HashMap::new();

    for file_path in &files {
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let fv = detect_format_version(&content).unwrap_or(FormatVersion::FV2504);
        let mut coord = match create_coordinator(fv) {
            Ok(c) => c,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };

        let nachricht = match coord.parse_nachricht(&content) {
            Ok(n) => n,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };

        let output_bytes = match coord.generate(&nachricht) {
            Ok(b) => b,
            Err(_) => {
                generate_errors += 1;
                continue;
            }
        };

        let normalized_original = normalize_edifact(&content);
        let normalized_generated = normalize_edifact(&output_bytes);

        if normalized_original == normalized_generated {
            identical += 1;
            continue;
        }

        total_differing += 1;

        let orig_segs: Vec<&str> = normalized_original
            .split('\'')
            .filter(|s| !s.is_empty())
            .collect();
        let gen_segs: Vec<&str> = normalized_generated
            .split('\'')
            .filter(|s| !s.is_empty())
            .collect();

        // Collect first diff
        let max_len = orig_segs.len().max(gen_segs.len());
        let mut found_first = false;
        for i in 0..max_len {
            let o = orig_segs.get(i).copied().unwrap_or("<missing>");
            let g = gen_segs.get(i).copied().unwrap_or("<missing>");
            if o != g {
                let o_id = extract_seg_id(o);
                let g_id = extract_seg_id(g);

                if !found_first {
                    *first_diff_orig_seg.entry(o_id.clone()).or_insert(0) += 1;
                    *first_diff_gen_seg.entry(g_id.clone()).or_insert(0) += 1;
                    *first_diff_pair
                        .entry(format!("{} -> {}", o_id, g_id))
                        .or_insert(0) += 1;
                    found_first = true;
                }

                *all_diff_orig_seg.entry(o_id).or_insert(0) += 1;
                *all_diff_gen_seg.entry(g_id).or_insert(0) += 1;
            }
        }

        // Analyze missing/extra segments by comparing multisets
        let mut orig_seg_counts: HashMap<String, usize> = HashMap::new();
        let mut gen_seg_counts: HashMap<String, usize> = HashMap::new();

        for s in &orig_segs {
            let id = extract_seg_id(s);
            *orig_seg_counts.entry(id).or_insert(0) += 1;
        }
        for s in &gen_segs {
            let id = extract_seg_id(s);
            *gen_seg_counts.entry(id).or_insert(0) += 1;
        }

        // Segments in original but not (enough) in generated
        for (seg_id, &orig_count) in &orig_seg_counts {
            let gen_count = gen_seg_counts.get(seg_id).copied().unwrap_or(0);
            if orig_count > gen_count {
                *missing_from_generated.entry(seg_id.clone()).or_insert(0) +=
                    orig_count - gen_count;
            }
        }

        // Segments in generated but not (enough) in original
        for (seg_id, &gen_count) in &gen_seg_counts {
            let orig_count = orig_seg_counts.get(seg_id).copied().unwrap_or(0);
            if gen_count > orig_count {
                *extra_in_generated.entry(seg_id.clone()).or_insert(0) += gen_count - orig_count;
            }
        }

        // Ordering issues: segments that appear equally often but at different positions
        // (present in both orig and gen with same count, but file still differs)
        let _orig_set: std::collections::HashSet<&str> = orig_segs.iter().copied().collect();
        let _gen_set: std::collections::HashSet<&str> = gen_segs.iter().copied().collect();

        // Check segments that exist in both but might be at wrong positions
        for (seg_id, &orig_count) in &orig_seg_counts {
            let gen_count = gen_seg_counts.get(seg_id).copied().unwrap_or(0);
            if orig_count == gen_count && orig_count > 0 {
                // Same count — check if any instance is at a different position
                // (simplified: if segment appears in diff at all, it's an ordering issue)
                let appears_in_diff = (0..max_len).any(|i| {
                    let o = orig_segs.get(i).copied().unwrap_or("<missing>");
                    let g = gen_segs.get(i).copied().unwrap_or("<missing>");
                    o != g && (extract_seg_id(o) == *seg_id || extract_seg_id(g) == *seg_id)
                });
                if appears_in_diff {
                    *ordering_issues.entry(seg_id.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    // Sort helper
    let sort_desc = |map: &HashMap<String, usize>| -> Vec<(String, usize)> {
        let mut v: Vec<_> = map.iter().map(|(k, &v)| (k.clone(), v)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    };

    // Print results
    eprintln!("\n{}", "=".repeat(60));
    eprintln!("ROUNDTRIP DIFF PATTERN ANALYSIS");
    eprintln!("{}", "=".repeat(60));
    eprintln!(
        "Total files: {}, Identical: {}, Differing: {}, Parse errors: {}, Generate errors: {}",
        files.len(),
        identical,
        total_differing,
        parse_errors,
        generate_errors
    );

    eprintln!("\n--- FIRST DIFF: Original segment type (what was expected) ---");
    eprintln!("(Shows which segment type is the first to differ in each file)");
    for (seg, count) in sort_desc(&first_diff_orig_seg).iter().take(30) {
        eprintln!("  {:40} {:>5} files", seg, count);
    }

    eprintln!("\n--- FIRST DIFF: Generated segment type (what we produced instead) ---");
    for (seg, count) in sort_desc(&first_diff_gen_seg).iter().take(30) {
        eprintln!("  {:40} {:>5} files", seg, count);
    }

    eprintln!("\n--- FIRST DIFF PAIRS: original -> generated ---");
    eprintln!("(Shows the most common 'expected X but got Y' patterns)");
    for (pair, count) in sort_desc(&first_diff_pair).iter().take(30) {
        eprintln!("  {:50} {:>5} files", pair, count);
    }

    eprintln!("\n--- SEGMENTS MISSING FROM GENERATED (top 20) ---");
    eprintln!("(Segments present in original but absent/fewer in generated output)");
    for (seg, count) in sort_desc(&missing_from_generated).iter().take(20) {
        eprintln!("  {:40} {:>5} total missing instances", seg, count);
    }

    eprintln!("\n--- EXTRA SEGMENTS IN GENERATED (top 20) ---");
    eprintln!("(Segments in generated that are not in original or appear too often)");
    for (seg, count) in sort_desc(&extra_in_generated).iter().take(20) {
        eprintln!("  {:40} {:>5} total extra instances", seg, count);
    }

    eprintln!("\n--- ORDERING ISSUES (top 20) ---");
    eprintln!("(Segments that appear same # of times but at wrong positions — files affected)");
    for (seg, count) in sort_desc(&ordering_issues).iter().take(20) {
        eprintln!("  {:40} {:>5} files", seg, count);
    }

    eprintln!("\n--- ALL DIFF: Original-side segment types (top 20) ---");
    eprintln!("(Total count of each segment type appearing at a diff position in original)");
    for (seg, count) in sort_desc(&all_diff_orig_seg).iter().take(20) {
        eprintln!("  {:40} {:>5} diff instances", seg, count);
    }

    eprintln!("\n--- ALL DIFF: Generated-side segment types (top 20) ---");
    for (seg, count) in sort_desc(&all_diff_gen_seg).iter().take(20) {
        eprintln!("  {:40} {:>5} diff instances", seg, count);
    }

    // Don't fail the test — this is analysis only
    eprintln!("\n[Analysis complete — this test does not assert, it only reports.]");
}
