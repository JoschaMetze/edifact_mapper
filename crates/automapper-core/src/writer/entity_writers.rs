//! Entity writers that serialize domain objects back to EDIFACT segments.
//!
//! Each writer knows how to produce the EDIFACT segments for one entity type.
//! They use `EdifactDocumentWriter` to append segments within an open message.

use bo4e_extensions::*;
use chrono::NaiveDateTime;

use super::document_writer::EdifactDocumentWriter;

/// Writes a Marktlokation to EDIFACT segments.
///
/// Produces: LOC+Z16+id' and NAD+DP address segments.
pub struct MarktlokationWriter;

impl MarktlokationWriter {
    /// Writes the LOC+Z16 segment for a Marktlokation.
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Marktlokation, MarktlokationEdifact>,
    ) {
        // LOC+Z16+marktlokationsId'
        if let Some(ref id) = ml.data.marktlokations_id {
            doc.write_segment("LOC", &["Z16", id]);
        }
    }

    /// Writes the NAD+DP address segment if address data is present.
    pub fn write_address(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Marktlokation, MarktlokationEdifact>,
    ) {
        if let Some(ref addr) = ml.data.lokationsadresse {
            let w = doc.segment_writer();
            w.begin_segment("NAD");
            w.add_element("DP");
            w.add_empty_element(); // C082
            w.add_empty_element(); // C058
            w.add_empty_element(); // C080
                                   // C059: street address
            w.begin_composite();
            w.add_component(addr.strasse.as_deref().unwrap_or(""));
            w.add_empty_component(); // 3042_1
            w.add_component(addr.hausnummer.as_deref().unwrap_or(""));
            w.end_composite();
            w.add_element(addr.ort.as_deref().unwrap_or(""));
            w.add_empty_element(); // region
            w.add_element(addr.postleitzahl.as_deref().unwrap_or(""));
            w.add_element(addr.landescode.as_deref().unwrap_or(""));
            w.end_segment();
            doc.message_segment_count_increment();
        }
    }
}

/// Writes a Messlokation to EDIFACT segments.
pub struct MesslokationWriter;

impl MesslokationWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Messlokation, MesslokationEdifact>,
    ) {
        if let Some(ref id) = ml.data.messlokations_id {
            doc.write_segment("LOC", &["Z17", id]);
        }
    }
}

/// Writes a Netzlokation to EDIFACT segments.
pub struct NetzlokationWriter;

impl NetzlokationWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        nl: &WithValidity<Netzlokation, NetzlokationEdifact>,
    ) {
        if let Some(ref id) = nl.data.netzlokations_id {
            doc.write_segment("LOC", &["Z18", id]);
        }
    }
}

/// Writes Geschaeftspartner NAD segments.
pub struct GeschaeftspartnerWriter;

impl GeschaeftspartnerWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        gp: &WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>,
    ) {
        let qualifier = gp.edifact.nad_qualifier.as_deref().unwrap_or("Z04");
        let id = gp.data.name1.as_deref().unwrap_or("");
        doc.write_segment("NAD", &[qualifier, id]);
    }
}

/// Writes a Zaehler to EDIFACT segments.
pub struct ZaehlerWriter;

impl ZaehlerWriter {
    pub fn write(doc: &mut EdifactDocumentWriter, z: &WithValidity<Zaehler, ZaehlerEdifact>) {
        // SEQ+Z03'
        doc.write_segment("SEQ", &["Z03"]);

        // PIA+5+zaehlernummer'
        if let Some(ref nr) = z.data.zaehlernummer {
            doc.write_segment("PIA", &["5", nr]);
        }

        // RFF+Z19:messlokation_ref'
        if let Some(ref melo_ref) = z.edifact.referenz_messlokation {
            doc.write_segment_with_composites("RFF", &[&["Z19", melo_ref]]);
        }

        // RFF+Z14:gateway_ref'
        if let Some(ref gw_ref) = z.edifact.referenz_gateway {
            doc.write_segment_with_composites("RFF", &[&["Z14", gw_ref]]);
        }
    }
}

/// Writes Vertrag data segments.
pub struct VertragWriter;

impl VertragWriter {
    pub fn write(doc: &mut EdifactDocumentWriter, v: &WithValidity<Vertrag, VertragEdifact>) {
        doc.write_segment("SEQ", &["Z18"]);

        if let Some(haushalt) = v.edifact.haushaltskunde {
            let code = if haushalt { "Z01" } else { "Z02" };
            doc.write_segment("CCI", &["Z15", "", code]);
        }

        if let Some(ref va) = v.edifact.versorgungsart {
            doc.write_segment("CCI", &["Z36", "", va]);
        }
    }
}

/// Writes Prozessdaten to EDIFACT segments.
///
/// MIG segment ordering (from UTILMD_MIG_Strom_S2_1, Counter values):
/// - DTM qualifiers at Counter=0230 (Nr 00021-00034)
/// - STS at Counter=0250 (Nr 00035)
/// - FTX+ACB at Counter=0280 (Nr 00038)
/// - RFF+Z13 at Counter=0350 via SG6 (Nr 00056)
pub struct ProzessdatenWriter;

impl ProzessdatenWriter {
    /// Formats a NaiveDateTime as EDIFACT DTM format code 303 (CCYYMMDDHHmm).
    fn format_dtm(dt: &NaiveDateTime) -> String {
        dt.format("%Y%m%d%H%M").to_string()
    }

    /// Writes a DTM composite segment: DTM+qualifier:value:303'
    fn write_dtm(doc: &mut EdifactDocumentWriter, qualifier: &str, dt: &NaiveDateTime) {
        let value = Self::format_dtm(dt);
        doc.write_segment_with_composites("DTM", &[&[qualifier, &value, "303"]]);
    }

    /// Writes all Prozessdaten segments for a transaction.
    ///
    /// Segment order follows MIG Counter=0230 (DTM dates), then
    /// Counter=0250 (STS), then Counter=0280 (FTX).
    /// RFF+Z13 is written separately via SG6 (Counter=0350).
    pub fn write(doc: &mut EdifactDocumentWriter, pd: &Prozessdaten) {
        // DTM segments (Counter=0230, Nr 00021-00034)
        if let Some(ref dt) = pd.prozessdatum {
            Self::write_dtm(doc, "137", dt);
        }
        if let Some(ref dt) = pd.wirksamkeitsdatum {
            Self::write_dtm(doc, "471", dt);
        }
        if let Some(ref dt) = pd.vertragsbeginn {
            Self::write_dtm(doc, "92", dt);
        }
        if let Some(ref dt) = pd.vertragsende {
            Self::write_dtm(doc, "93", dt);
        }
        if let Some(ref dt) = pd.lieferbeginndatum_in_bearbeitung {
            Self::write_dtm(doc, "Z07", dt);
        }
        if let Some(ref dt) = pd.datum_naechste_bearbeitung {
            Self::write_dtm(doc, "Z08", dt);
        }

        // STS segment (Counter=0250, Nr 00035)
        // STS+7+transaktionsgrund::codelist+ergaenzung'
        if let Some(ref grund) = pd.transaktionsgrund {
            let w = doc.segment_writer();
            w.begin_segment("STS");
            w.add_element("7");
            w.begin_composite();
            w.add_component(grund);
            w.end_composite();
            if let Some(ref erg) = pd.transaktionsgrund_ergaenzung {
                w.begin_composite();
                w.add_component(erg);
                w.end_composite();
            }
            w.end_segment();
            doc.message_segment_count_increment();
        }

        // FTX+ACB segment (Counter=0280, Nr 00038)
        if let Some(ref bemerkung) = pd.bemerkung {
            let w = doc.segment_writer();
            w.begin_segment("FTX");
            w.add_element("ACB");
            w.add_empty_element(); // text subject code
            w.add_empty_element(); // text function code
            w.begin_composite();
            w.add_component(bemerkung);
            w.end_composite();
            w.end_segment();
            doc.message_segment_count_increment();
        }
    }

    /// Writes RFF segments from Prozessdaten (SG6, Counter=0350).
    ///
    /// Called separately from `write()` because SG6 comes after SG5 (LOC)
    /// in MIG counter order.
    pub fn write_references(doc: &mut EdifactDocumentWriter, pd: &Prozessdaten) {
        // RFF+Z13:referenz_vorgangsnummer (Nr 00056)
        if let Some(ref vorgangsnr) = pd.referenz_vorgangsnummer {
            doc.write_segment_with_composites("RFF", &[&["Z13", vorgangsnr]]);
        }
    }
}

/// Writes Zeitscheibe data to EDIFACT segments.
///
/// MIG: SG6 with RFF+Z47 (Verwendungszeitraum der Daten, Nr 00066, Counter=0350)
/// Each Zeitscheibe produces:
/// - RFF+Z47:zeitscheiben_id (Counter=0360)
/// - DTM+Z25:von:303 (Counter=0370, if gueltigkeitszeitraum.von present)
/// - DTM+Z26:bis:303 (Counter=0370, if gueltigkeitszeitraum.bis present)
pub struct ZeitscheibeWriter;

impl ZeitscheibeWriter {
    /// Writes all Zeitscheibe segments for a transaction.
    pub fn write(doc: &mut EdifactDocumentWriter, zeitscheiben: &[Zeitscheibe]) {
        for zs in zeitscheiben {
            // RFF+Z47:zeitscheiben_id
            doc.write_segment_with_composites("RFF", &[&["Z47", &zs.zeitscheiben_id]]);

            // DTM+Z25/Z26 from gueltigkeitszeitraum
            if let Some(ref gz) = zs.gueltigkeitszeitraum {
                if let Some(ref von) = gz.von {
                    let value = von.format("%Y%m%d%H%M").to_string();
                    doc.write_segment_with_composites("DTM", &[&["Z25", &value, "303"]]);
                }
                if let Some(ref bis) = gz.bis {
                    let value = bis.format("%Y%m%d%H%M").to_string();
                    doc.write_segment_with_composites("DTM", &[&["Z26", &value, "303"]]);
                }
            }
        }
    }
}

/// Writes a SteuerbareRessource to EDIFACT segments.
pub struct SteuerbareRessourceWriter;

impl SteuerbareRessourceWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        sr: &WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>,
    ) {
        if let Some(ref id) = sr.data.steuerbare_ressource_id {
            doc.write_segment("LOC", &["Z19", id]);
        }
    }
}

/// Writes a TechnischeRessource to EDIFACT segments.
pub struct TechnischeRessourceWriter;

impl TechnischeRessourceWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        tr: &WithValidity<TechnischeRessource, TechnischeRessourceEdifact>,
    ) {
        if let Some(ref id) = tr.data.technische_ressource_id {
            doc.write_segment("LOC", &["Z20", id]);
        }
    }
}

/// Writes a Tranche to EDIFACT segments.
pub struct TrancheWriter;

impl TrancheWriter {
    pub fn write(doc: &mut EdifactDocumentWriter, t: &WithValidity<Tranche, TrancheEdifact>) {
        if let Some(ref id) = t.data.tranche_id {
            doc.write_segment("LOC", &["Z21", id]);
        }
    }
}

/// Writes a MabisZaehlpunkt to EDIFACT segments.
pub struct MabisZaehlpunktWriter;

impl MabisZaehlpunktWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        mz: &WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>,
    ) {
        if let Some(ref id) = mz.data.zaehlpunkt_id {
            doc.write_segment("LOC", &["Z15", id]);
        }
    }
}

/// Writes a Produktpaket to EDIFACT segments.
pub struct ProduktpaketWriter;

impl ProduktpaketWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        pp: &WithValidity<Produktpaket, ProduktpaketEdifact>,
    ) {
        // SEQ+Z79+produktpaket_id'
        let id = pp.data.produktpaket_id.as_deref().unwrap_or("");
        doc.write_segment("SEQ", &["Z79", id]);

        // PIA+5+produktpaket_name'
        if let Some(ref name) = pp.edifact.produktpaket_name {
            doc.write_segment("PIA", &["5", name]);
        }
    }
}

/// Writes a Lokationszuordnung to EDIFACT segments.
pub struct LokationszuordnungWriter;

impl LokationszuordnungWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        lz: &WithValidity<Lokationszuordnung, LokationszuordnungEdifact>,
    ) {
        // SEQ+Z78'
        doc.write_segment("SEQ", &["Z78"]);

        // RFF+Z18:marktlokations_id'
        if let Some(ref malo_id) = lz.data.marktlokations_id {
            doc.write_segment_with_composites("RFF", &[&["Z18", malo_id]]);
        }

        // RFF+Z19:messlokations_id'
        if let Some(ref melo_id) = lz.data.messlokations_id {
            doc.write_segment_with_composites("RFF", &[&["Z19", melo_id]]);
        }
    }
}

/// Writes Bilanzierung data to EDIFACT segments.
pub struct BilanzierungWriter;

impl BilanzierungWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        b: &WithValidity<Bilanzierung, BilanzierungEdifact>,
    ) {
        // SEQ+Z98'
        doc.write_segment("SEQ", &["Z98"]);

        // CCI+Z20++bilanzkreis'
        if let Some(ref bk) = b.data.bilanzkreis {
            doc.write_segment("CCI", &["Z20", "", bk]);
        }

        // CCI+Z21++regelzone'
        if let Some(ref rz) = b.data.regelzone {
            doc.write_segment("CCI", &["Z21", "", rz]);
        }

        // CCI+Z22++bilanzierungsgebiet'
        if let Some(ref bg) = b.data.bilanzierungsgebiet {
            doc.write_segment("CCI", &["Z22", "", bg]);
        }

        // QTY+Z09:jahresverbrauchsprognose'
        if let Some(jvp) = b.edifact.jahresverbrauchsprognose {
            let value = format!("{jvp}");
            doc.write_segment_with_composites("QTY", &[&["Z09", &value]]);
        }

        // QTY+265:temperatur_arbeit'
        if let Some(ta) = b.edifact.temperatur_arbeit {
            let value = format!("{ta}");
            doc.write_segment_with_composites("QTY", &[&["265", &value]]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::edifact_types::EdifactDelimiters;
    use chrono::NaiveDate;

    #[test]
    fn test_marktlokation_writer_loc() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let ml = WithValidity {
            data: Marktlokation {
                marktlokations_id: Some("DE00014545768S0000000000000003054".to_string()),
                ..Default::default()
            },
            edifact: MarktlokationEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        MarktlokationWriter::write(&mut doc, &ml);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("LOC+Z16+DE00014545768S0000000000000003054'"));
    }

    #[test]
    fn test_messlokation_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let ml = WithValidity {
            data: Messlokation {
                messlokations_id: Some("MELO001".to_string()),
                ..Default::default()
            },
            edifact: MesslokationEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        MesslokationWriter::write(&mut doc, &ml);
        doc.end_message();
        doc.end_interchange();

        assert!(doc.output().contains("LOC+Z17+MELO001'"));
    }

    #[test]
    fn test_zaehler_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let z = WithValidity {
            data: Zaehler {
                zaehlernummer: Some("ZAEHLER001".to_string()),
                ..Default::default()
            },
            edifact: ZaehlerEdifact {
                referenz_messlokation: Some("MELO001".to_string()),
                ..Default::default()
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        ZaehlerWriter::write(&mut doc, &z);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z03'"));
        assert!(output.contains("PIA+5+ZAEHLER001'"));
        assert!(output.contains("RFF+Z19:MELO001'"));
    }

    #[test]
    fn test_vertrag_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let v = WithValidity {
            data: Vertrag::default(),
            edifact: VertragEdifact {
                haushaltskunde: Some(true),
                versorgungsart: Some("ZD0".to_string()),
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        VertragWriter::write(&mut doc, &v);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z18'"));
        assert!(output.contains("CCI+Z15++Z01'"));
        assert!(output.contains("CCI+Z36++ZD0'"));
    }

    #[test]
    fn test_prozessdaten_writer_dtm_and_sts() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let dt = NaiveDate::from_ymd_opt(2025, 7, 1)
            .unwrap()
            .and_hms_opt(13, 30, 0)
            .unwrap();
        let pd = Prozessdaten {
            transaktionsgrund: Some("E01".to_string()),
            transaktionsgrund_ergaenzung: Some("Z01".to_string()),
            prozessdatum: Some(dt),
            wirksamkeitsdatum: Some(dt),
            vertragsbeginn: Some(dt),
            ..Default::default()
        };

        ProzessdatenWriter::write(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("DTM+137:202507011330:303'"));
        assert!(output.contains("DTM+471:202507011330:303'"));
        assert!(output.contains("DTM+92:202507011330:303'"));
        assert!(output.contains("STS+7+E01+Z01'"));
    }

    #[test]
    fn test_prozessdaten_writer_ftx_and_rff() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let pd = Prozessdaten {
            bemerkung: Some("Test Bemerkung".to_string()),
            referenz_vorgangsnummer: Some("VG001".to_string()),
            ..Default::default()
        };

        ProzessdatenWriter::write(&mut doc, &pd);
        ProzessdatenWriter::write_references(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("FTX+ACB+++Test Bemerkung'"));
        assert!(output.contains("RFF+Z13:VG001'"));
    }

    #[test]
    fn test_prozessdaten_writer_empty() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let pd = Prozessdaten::default();
        ProzessdatenWriter::write(&mut doc, &pd);
        ProzessdatenWriter::write_references(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        // No DTM, STS, FTX, or RFF segments should be written
        assert!(!output.contains("DTM"));
        assert!(!output.contains("STS"));
        assert!(!output.contains("FTX"));
        assert!(!output.contains("RFF"));
    }

    #[test]
    fn test_zeitscheibe_writer_single() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let von = NaiveDate::from_ymd_opt(2025, 7, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let bis = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let zs = vec![Zeitscheibe {
            zeitscheiben_id: "1".to_string(),
            gueltigkeitszeitraum: Some(Zeitraum::new(Some(von), Some(bis))),
        }];

        ZeitscheibeWriter::write(&mut doc, &zs);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("RFF+Z47:1'"));
        assert!(output.contains("DTM+Z25:202507010000:303'"));
        assert!(output.contains("DTM+Z26:202512310000:303'"));
    }

    #[test]
    fn test_zeitscheibe_writer_multiple() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let zs = vec![
            Zeitscheibe {
                zeitscheiben_id: "1".to_string(),
                gueltigkeitszeitraum: None,
            },
            Zeitscheibe {
                zeitscheiben_id: "2".to_string(),
                gueltigkeitszeitraum: None,
            },
        ];

        ZeitscheibeWriter::write(&mut doc, &zs);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("RFF+Z47:1'"));
        assert!(output.contains("RFF+Z47:2'"));
    }

    #[test]
    fn test_zeitscheibe_writer_empty() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        ZeitscheibeWriter::write(&mut doc, &[]);
        doc.end_message();
        doc.end_interchange();

        // No RFF or DTM segments should be written
        let output = doc.output();
        assert!(!output.contains("RFF"));
    }

    #[test]
    fn test_steuerbare_ressource_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let sr = WithValidity {
            data: SteuerbareRessource {
                steuerbare_ressource_id: Some("STRES001".to_string()),
            },
            edifact: SteuerbareRessourceEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        SteuerbareRessourceWriter::write(&mut doc, &sr);
        doc.end_message();
        doc.end_interchange();

        assert!(doc.output().contains("LOC+Z19+STRES001'"));
    }

    #[test]
    fn test_technische_ressource_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let tr = WithValidity {
            data: TechnischeRessource {
                technische_ressource_id: Some("TECRES001".to_string()),
            },
            edifact: TechnischeRessourceEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        TechnischeRessourceWriter::write(&mut doc, &tr);
        doc.end_message();
        doc.end_interchange();

        assert!(doc.output().contains("LOC+Z20+TECRES001'"));
    }

    #[test]
    fn test_tranche_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let t = WithValidity {
            data: Tranche {
                tranche_id: Some("TRANCHE001".to_string()),
            },
            edifact: TrancheEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        TrancheWriter::write(&mut doc, &t);
        doc.end_message();
        doc.end_interchange();

        assert!(doc.output().contains("LOC+Z21+TRANCHE001'"));
    }

    #[test]
    fn test_mabis_zaehlpunkt_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let mz = WithValidity {
            data: MabisZaehlpunkt {
                zaehlpunkt_id: Some("MABIS001".to_string()),
            },
            edifact: MabisZaehlpunktEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        MabisZaehlpunktWriter::write(&mut doc, &mz);
        doc.end_message();
        doc.end_interchange();

        assert!(doc.output().contains("LOC+Z15+MABIS001'"));
    }

    #[test]
    fn test_produktpaket_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let pp = WithValidity {
            data: Produktpaket {
                produktpaket_id: Some("PP001".to_string()),
            },
            edifact: ProduktpaketEdifact {
                produktpaket_name: Some("Grundversorgung".to_string()),
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        ProduktpaketWriter::write(&mut doc, &pp);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z79+PP001'"));
        assert!(output.contains("PIA+5+Grundversorgung'"));
    }

    #[test]
    fn test_lokationszuordnung_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let lz = WithValidity {
            data: Lokationszuordnung {
                marktlokations_id: Some("MALO001".to_string()),
                messlokations_id: Some("MELO001".to_string()),
            },
            edifact: LokationszuordnungEdifact::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        LokationszuordnungWriter::write(&mut doc, &lz);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z78'"));
        assert!(output.contains("RFF+Z18:MALO001'"));
        assert!(output.contains("RFF+Z19:MELO001'"));
    }

    #[test]
    fn test_bilanzierung_writer() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let b = WithValidity {
            data: Bilanzierung {
                bilanzkreis: Some("11YN20---------Z".to_string()),
                regelzone: None,
                bilanzierungsgebiet: None,
            },
            edifact: BilanzierungEdifact {
                jahresverbrauchsprognose: Some(12345.67),
                temperatur_arbeit: None,
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        };

        BilanzierungWriter::write(&mut doc, &b);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("SEQ+Z98'"));
        assert!(output.contains("CCI+Z20++11YN20---------Z'"));
        assert!(output.contains("QTY+Z09:12345.67'"));
    }
}
