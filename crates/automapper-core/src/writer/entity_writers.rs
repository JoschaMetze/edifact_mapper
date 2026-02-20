//! Entity writers that serialize domain objects back to EDIFACT segments.
//!
//! Each writer knows how to produce the EDIFACT segments for one entity type.
//! They use `EdifactDocumentWriter` to append segments within an open message.

use std::collections::HashMap;

use bo4e_extensions::*;

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
        if let Some(ref raw) = ml.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = ml.data.marktlokations_id {
            doc.write_segment("LOC", &["Z16", id]);
        }
    }

    /// Writes the NAD+DP address segment if address data is present.
    pub fn write_address(
        doc: &mut EdifactDocumentWriter,
        ml: &WithValidity<Marktlokation, MarktlokationEdifact>,
    ) {
        // Use raw NAD for roundtrip fidelity if available
        if !ml.edifact.raw_nad_address.is_empty() {
            for raw in &ml.edifact.raw_nad_address {
                doc.write_raw_segment(raw);
            }
        } else if let Some(ref addr) = ml.data.lokationsadresse {
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
        if let Some(ref raw) = ml.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = ml.data.messlokations_id {
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
        if let Some(ref raw) = nl.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = nl.data.netzlokations_id {
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
        // Use raw NAD for roundtrip fidelity if available
        if let Some(ref raw) = gp.edifact.raw_nad {
            doc.write_raw_segment(raw);
        } else {
            let qualifier = gp.edifact.nad_qualifier.as_deref().unwrap_or("Z04");
            let id = gp.data.name1.as_deref().unwrap_or("");
            doc.write_segment("NAD", &[qualifier, id]);
        }
        // RFF segments following this NAD party (e.g. RFF+Z18, RFF+Z01, RFF+Z19)
        for raw_rff in &gp.edifact.raw_rffs {
            doc.write_raw_segment(raw_rff);
        }
    }
}

/// Writes a Zaehler to EDIFACT segments.
pub struct ZaehlerWriter;

impl ZaehlerWriter {
    pub fn write(doc: &mut EdifactDocumentWriter, z: &WithValidity<Zaehler, ZaehlerEdifact>) {
        // SEQ+Z03+sub_id'
        let sub_id = z.edifact.seq_sub_id.as_deref().unwrap_or("");
        doc.write_segment("SEQ", &["Z03", sub_id]);

        // RFF+Z19:messlokation_ref'
        if let Some(ref melo_ref) = z.edifact.referenz_messlokation {
            doc.write_segment_with_composites("RFF", &[&["Z19", melo_ref]]);
        }

        // RFF+Z14:gateway_ref'
        if let Some(ref gw_ref) = z.edifact.referenz_gateway {
            doc.write_segment_with_composites("RFF", &[&["Z14", gw_ref]]);
        }

        // QTY segments (Z33, Z34, 31 etc.)
        for raw in &z.edifact.raw_qty {
            doc.write_raw_segment(raw);
        }

        // CCI/CAV segments (device characteristics)
        for raw in &z.edifact.raw_cci_cav {
            doc.write_raw_segment(raw);
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
    /// MIG-ordered process-level DTM qualifiers (Counter=0230, before STS).
    const PROCESS_DTM_ORDER: &[&str] = &[
        "137", "92", "93", "471", "154", "Z05", "76", "157", "158", "159", "Z01", "Z06", "Z07",
        "Z08", "Z10", "Z25", "Z26", "Z42", "Z43", "Z51", "Z52", "Z53",
    ];

    /// MIG-ordered reference-section DTM qualifiers (SG6, after RFF).
    const REFERENCE_DTM_ORDER: &[&str] = &["155", "752", "672", "Z20", "Z21", "Z09", "Z22"];

    /// Write a DTM from the raw_dtm HashMap (preferred) or fallback to a NaiveDateTime value.
    fn write_dtm_from_raw(
        doc: &mut EdifactDocumentWriter,
        qualifier: &str,
        raw_dtm: &HashMap<String, String>,
    ) {
        if let Some(raw) = raw_dtm.get(qualifier) {
            let composite = format!("{}:{}", qualifier, raw);
            doc.write_segment("DTM", &[&composite]);
        }
    }

    /// Write a DTM, trying raw_dtm first, then falling back to a typed NaiveDateTime.
    fn write_dtm_with_fallback(
        doc: &mut EdifactDocumentWriter,
        qualifier: &str,
        raw_dtm: &HashMap<String, String>,
        typed: Option<chrono::NaiveDateTime>,
    ) {
        if let Some(raw) = raw_dtm.get(qualifier) {
            let composite = format!("{}:{}", qualifier, raw);
            doc.write_segment("DTM", &[&composite]);
        } else if let Some(dt) = typed {
            let formatted = dt.format("%Y%m%d%H%M").to_string();
            let composite = format!("{}:{}:303", qualifier, formatted);
            doc.write_segment("DTM", &[&composite]);
        }
    }

    /// Writes all Prozessdaten segments for a transaction.
    ///
    /// Segment order follows MIG Counter=0230 (DTM dates), then
    /// Counter=0250 (STS), then Counter=0280 (FTX).
    /// RFF+Z13 is written separately via SG6 (Counter=0350).
    pub fn write(doc: &mut EdifactDocumentWriter, pd: &Prozessdaten) {
        // IMD segments (between IDE and DTM in MIG order)
        for raw in &pd.raw_imd {
            doc.write_raw_segment(raw);
        }

        // Process-level DTM segments (Counter=0230)
        // Prefer raw_process_dtms (preserves ordering, duplicates, and release chars).
        // Fall back to PROCESS_DTM_ORDER with raw_dtm HashMap for manually constructed data.
        if !pd.raw_process_dtms.is_empty() {
            for raw in &pd.raw_process_dtms {
                doc.write_raw_segment(raw);
            }
        } else {
            for qualifier in Self::PROCESS_DTM_ORDER {
                match *qualifier {
                    "137" => {
                        Self::write_dtm_with_fallback(doc, "137", &pd.raw_dtm, pd.prozessdatum)
                    }
                    "471" => {
                        Self::write_dtm_with_fallback(doc, "471", &pd.raw_dtm, pd.wirksamkeitsdatum)
                    }
                    "92" => {
                        Self::write_dtm_with_fallback(doc, "92", &pd.raw_dtm, pd.vertragsbeginn)
                    }
                    "93" => Self::write_dtm_with_fallback(doc, "93", &pd.raw_dtm, pd.vertragsende),
                    _ => Self::write_dtm_from_raw(doc, qualifier, &pd.raw_dtm),
                }
            }
        }

        // STS+7 segment (Counter=0250, Nr 00035)
        if let Some(ref grund) = pd.transaktionsgrund {
            let w = doc.segment_writer();
            w.begin_segment("STS");
            w.add_element("7");
            w.add_empty_element(); // empty element 1 per S2.1 format
            w.add_element(grund);
            if let Some(ref erg) = pd.transaktionsgrund_ergaenzung {
                w.add_element(erg);
            }
            if let Some(ref befr) = pd.transaktionsgrund_ergaenzung_befristete_anmeldung {
                if pd.transaktionsgrund_ergaenzung.is_none() {
                    w.add_empty_element();
                }
                w.add_element(befr);
            }
            w.end_segment();
            doc.message_segment_count_increment();
        }

        // STS+E01/Z17/Z18/Z35 (stored as raw for roundtrip)
        for raw in &pd.sts_raw {
            doc.write_raw_segment(raw);
        }

        // FTX segments (Counter=0280, Nr 00038)
        // Use raw FTX for roundtrip fidelity if available
        if !pd.raw_ftx.is_empty() {
            for raw in &pd.raw_ftx {
                doc.write_raw_segment(raw);
            }
        } else if let Some(ref bemerkung) = pd.bemerkung {
            let w = doc.segment_writer();
            w.begin_segment("FTX");
            w.add_element("ACB");
            w.add_empty_element();
            w.add_empty_element();
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
        // RFF segments in MIG order (SG6 Counter=0350)
        if let Some(ref v) = pd.referenz_vorgangsnummer {
            doc.write_segment_with_composites("RFF", &[&["Z13", v]]);
        }
        if let Some(ref v) = pd.vorgangsnummer {
            doc.write_segment_with_composites("RFF", &[&["AGI", v]]);
        }
        if let Some(ref v) = pd.referenz_transaktions_id {
            doc.write_segment_with_composites("RFF", &[&["TN", v]]);
        }
        if let Some(ref v) = pd.rff_z42 {
            doc.write_segment_with_composites("RFF", &[&["Z42", v]]);
        }
        if let Some(ref v) = pd.rff_z43 {
            doc.write_segment_with_composites("RFF", &[&["Z43", v]]);
        }
        if let Some(ref v) = pd.rff_z39 {
            doc.write_segment_with_composites("RFF", &[&["Z39", v]]);
        }
        if let Some(ref v) = pd.rff_z60 {
            doc.write_segment_with_composites("RFF", &[&["Z60", v]]);
        }
        if let Some(ref v) = pd.anfrage_referenz_aav {
            doc.write_segment_with_composites("RFF", &[&["AAV", v]]);
        }
        if let Some(ref v) = pd.referenz_vorgangsnummer_acw {
            doc.write_segment_with_composites("RFF", &[&["ACW", v]]);
        }
        if let Some(ref v) = pd.rff_z18 {
            if v.is_empty() {
                doc.write_segment("RFF", &["Z18"]);
            } else {
                doc.write_segment_with_composites("RFF", &[&["Z18", v]]);
            }
        }
        // Zeitscheibe reference blocks: each RFF (Z49/Z50/Z53/Z47) followed by DTM+Z25/Z26
        for zs_ref in &pd.zeitscheibe_refs {
            doc.write_raw_segment(&zs_ref.raw_rff);
            for dtm_raw in &zs_ref.raw_dtms {
                doc.write_raw_segment(dtm_raw);
            }
        }
        // RFF+Z31: Lokationsbuendel reference (after zeitscheibe refs)
        if let Some(ref v) = pd.rff_z31 {
            doc.write_segment("RFF", &[&format!("Z31{}", v)]);
        }
        if let Some(ref v) = pd.rff_z01 {
            doc.write_segment("RFF", &[&format!("Z01{}", v)]);
        }

        // Reference-section DTMs (SG6, after RFF) — MIG order
        for qualifier in Self::REFERENCE_DTM_ORDER {
            Self::write_dtm_from_raw(doc, qualifier, &pd.raw_dtm);
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
        if let Some(ref raw) = sr.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = sr.data.steuerbare_ressource_id {
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
        if let Some(ref raw) = tr.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = tr.data.technische_ressource_id {
            doc.write_segment("LOC", &["Z20", id]);
        }
    }
}

/// Writes a Tranche to EDIFACT segments.
pub struct TrancheWriter;

impl TrancheWriter {
    pub fn write(doc: &mut EdifactDocumentWriter, t: &WithValidity<Tranche, TrancheEdifact>) {
        if let Some(ref raw) = t.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = t.data.tranche_id {
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
        if let Some(ref raw) = mz.edifact.raw_loc {
            doc.write_raw_segment(raw);
        } else if let Some(ref id) = mz.data.zaehlpunkt_id {
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
        // SEQ+Z79/ZH0+produktpaket_id'
        let qualifier = pp.edifact.seq_qualifier.as_deref().unwrap_or("Z79");
        let id = pp.data.produktpaket_id.as_deref().unwrap_or("");
        doc.write_segment("SEQ", &[qualifier, id]);

        // PIA+5+produktpaket_name:typ' — use raw for roundtrip fidelity
        if let Some(ref raw) = pp.edifact.raw_pia {
            doc.write_raw_segment(raw);
        } else if let Some(ref name) = pp.edifact.produktpaket_name {
            doc.write_segment("PIA", &["5", name]);
        }
        // CCI/CAV segments in Z79 group
        for raw in &pp.edifact.raw_cci_cav {
            doc.write_raw_segment(raw);
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
        // SEQ+Z78+sub_id'
        let sub_id = lz.edifact.zuordnungstyp.as_deref().unwrap_or("");
        doc.write_segment("SEQ", &["Z78", sub_id]);

        // Use raw RFFs for roundtrip fidelity if available
        if !lz.edifact.raw_rffs.is_empty() {
            for raw in &lz.edifact.raw_rffs {
                doc.write_raw_segment(raw);
            }
        } else {
            if let Some(ref malo_id) = lz.data.marktlokations_id {
                doc.write_segment_with_composites("RFF", &[&["Z18", malo_id]]);
            }
            if let Some(ref melo_id) = lz.data.messlokations_id {
                doc.write_segment_with_composites("RFF", &[&["Z19", melo_id]]);
            }
        }
    }
}

/// Writes SEQ group data to EDIFACT segments.
///
/// Handles all SEQ group types (Z45, Z71, Z21, Z08, Z01, Z20, and generic).
pub struct SeqGroupWriter;

impl SeqGroupWriter {
    /// Writes a CCI entry as a segment.
    fn write_cci(doc: &mut EdifactDocumentWriter, cci: &bo4e_extensions::CciEntry) {
        let w = doc.segment_writer();
        w.begin_segment("CCI");
        w.add_element(cci.qualifier.as_deref().unwrap_or(""));
        w.add_element(cci.additional_qualifier.as_deref().unwrap_or(""));
        if let Some(ref code) = cci.characteristic_code {
            w.begin_composite();
            w.add_component(code);
            w.end_composite();
        }
        w.end_segment();
        doc.message_segment_count_increment();
    }

    /// Writes a CAV segment with a simple value.
    fn write_cav(doc: &mut EdifactDocumentWriter, value: &str) {
        doc.write_segment("CAV", &[value]);
    }

    /// Writes CCI/CAV segments. Uses raw storage for roundtrip fidelity when available,
    /// otherwise falls back to interleaved CCI/CAV pairs from structured data.
    fn write_cci_cav(
        doc: &mut EdifactDocumentWriter,
        raw_cci_cav: &[String],
        ccis: &[bo4e_extensions::CciEntry],
        cavs: &[String],
    ) {
        if !raw_cci_cav.is_empty() {
            for raw in raw_cci_cav {
                doc.write_raw_segment(raw);
            }
            return;
        }
        // Fallback: interleaved CCI/CAV pairs from structured data
        let mut cav_idx = 0;
        for cci in ccis {
            Self::write_cci(doc, cci);
            let is_z99 = cci.qualifier.as_deref() == Some("Z99");
            if !is_z99 {
                if let Some(cav_val) = cavs.get(cav_idx) {
                    Self::write_cav(doc, cav_val);
                    cav_idx += 1;
                }
            }
        }
        while cav_idx < cavs.len() {
            Self::write_cav(doc, &cavs[cav_idx]);
            cav_idx += 1;
        }
    }

    /// Writes a SEQ+Z45 group.
    pub fn write_z45(doc: &mut EdifactDocumentWriter, group: &bo4e_extensions::SeqZ45Group) {
        // SEQ+Z45+zeitscheibe_ref'
        let w = doc.segment_writer();
        w.begin_segment("SEQ");
        w.add_element("Z45");
        if let Some(ref zs) = group.zeitscheibe_ref {
            w.add_element(zs);
        }
        w.end_segment();
        doc.message_segment_count_increment();

        // Use raw_cci_cav for all body segments (preserves original order of PIA/QTY/CCI/CAV)
        if !group.raw_cci_cav.is_empty() {
            for raw in &group.raw_cci_cav {
                doc.write_raw_segment(raw);
            }
        } else {
            // Fallback: structured fields
            if group.artikel_id.is_some() || group.artikel_id_typ.is_some() {
                let w = doc.segment_writer();
                w.begin_segment("PIA");
                w.add_element("Z02");
                w.begin_composite();
                w.add_component(group.artikel_id.as_deref().unwrap_or(""));
                w.add_component(group.artikel_id_typ.as_deref().unwrap_or(""));
                w.end_composite();
                w.end_segment();
                doc.message_segment_count_increment();
            }
            if let Some(ref raw) = group.wandlerfaktor {
                doc.write_segment("QTY", &[raw]);
            }
            if let Some(ref raw) = group.vorkommastelle {
                doc.write_segment("QTY", &[raw]);
            }
            if let Some(ref raw) = group.nachkommastelle {
                doc.write_segment("QTY", &[raw]);
            }
            Self::write_cci_cav(doc, &[], &group.cci_segments, &group.cav_segments);
        }
    }

    /// Writes a SEQ+Z71 group.
    pub fn write_z71(doc: &mut EdifactDocumentWriter, group: &bo4e_extensions::SeqZ71Group) {
        let w = doc.segment_writer();
        w.begin_segment("SEQ");
        w.add_element("Z71");
        if let Some(ref zs) = group.zeitscheibe_ref {
            w.add_element(zs);
        }
        w.end_segment();
        doc.message_segment_count_increment();

        // Use raw_cci_cav for all body segments (preserves original order)
        if !group.raw_cci_cav.is_empty() {
            for raw in &group.raw_cci_cav {
                doc.write_raw_segment(raw);
            }
        } else {
            if group.artikel_id.is_some() || group.artikel_id_typ.is_some() {
                let w = doc.segment_writer();
                w.begin_segment("PIA");
                w.add_element("Z02");
                w.begin_composite();
                w.add_component(group.artikel_id.as_deref().unwrap_or(""));
                w.add_component(group.artikel_id_typ.as_deref().unwrap_or(""));
                w.end_composite();
                w.end_segment();
                doc.message_segment_count_increment();
            }
            Self::write_cci_cav(doc, &[], &group.cci_segments, &group.cav_segments);
        }
    }

    /// Writes a SEQ+Z21 group.
    pub fn write_z21(doc: &mut EdifactDocumentWriter, group: &bo4e_extensions::SeqZ21Group) {
        let w = doc.segment_writer();
        w.begin_segment("SEQ");
        w.add_element("Z21");
        if let Some(ref zs) = group.zeitscheibe_ref {
            w.add_element(zs);
        }
        w.end_segment();
        doc.message_segment_count_increment();

        // Use raw_cci_cav for all body segments (preserves original order)
        if !group.raw_cci_cav.is_empty() {
            for raw in &group.raw_cci_cav {
                doc.write_raw_segment(raw);
            }
        } else {
            for rff_raw in &group.rff_segments {
                doc.write_raw_segment(rff_raw);
            }
            Self::write_cci_cav(doc, &[], &group.cci_segments, &group.cav_segments);
        }
    }

    /// Writes a SEQ+Z08 group.
    pub fn write_z08(doc: &mut EdifactDocumentWriter, group: &bo4e_extensions::SeqZ08Group) {
        let w = doc.segment_writer();
        w.begin_segment("SEQ");
        w.add_element("Z08");
        if let Some(ref zs) = group.zeitscheibe_ref {
            w.add_element(zs);
        }
        w.end_segment();
        doc.message_segment_count_increment();

        // Use raw_cci_cav for all body segments (preserves original order)
        if !group.raw_cci_cav.is_empty() {
            for raw in &group.raw_cci_cav {
                doc.write_raw_segment(raw);
            }
        } else {
            for rff_raw in &group.rff_segments {
                doc.write_raw_segment(rff_raw);
            }
            Self::write_cci_cav(doc, &[], &group.cci_segments, &group.cav_segments);
        }
    }

    /// Writes a SEQ+Z01 group.
    pub fn write_z01(doc: &mut EdifactDocumentWriter, group: &bo4e_extensions::SeqZ01Group) {
        let w = doc.segment_writer();
        w.begin_segment("SEQ");
        w.add_element("Z01");
        if let Some(ref zs) = group.zeitscheibe_ref {
            w.add_element(zs);
        }
        w.end_segment();
        doc.message_segment_count_increment();

        // Use raw_cci_cav for all body segments (preserves original order of RFF/QTY/CCI/CAV)
        if !group.raw_cci_cav.is_empty() {
            for raw in &group.raw_cci_cav {
                doc.write_raw_segment(raw);
            }
        } else {
            // Fallback: separate lists
            for rff_raw in &group.rff_segments {
                doc.write_raw_segment(rff_raw);
            }
            for qty_raw in &group.qty_segments {
                doc.write_segment("QTY", &[qty_raw]);
            }
            Self::write_cci_cav(doc, &[], &group.cci_segments, &group.cav_segments);
        }
    }

    /// Writes a SEQ+Z20 group.
    pub fn write_z20(doc: &mut EdifactDocumentWriter, group: &bo4e_extensions::SeqZ20Group) {
        let w = doc.segment_writer();
        w.begin_segment("SEQ");
        w.add_element("Z20");
        if let Some(ref zs) = group.zeitscheibe_ref {
            w.add_element(zs);
        }
        w.end_segment();
        doc.message_segment_count_increment();

        // Use raw_cci_cav for all body segments (preserves original order)
        if !group.raw_cci_cav.is_empty() {
            for raw in &group.raw_cci_cav {
                doc.write_raw_segment(raw);
            }
        } else {
            for rff_raw in &group.rff_segments {
                doc.write_raw_segment(rff_raw);
            }
            for pia_raw in &group.pia_segments {
                doc.write_raw_segment(pia_raw);
            }
            Self::write_cci_cav(doc, &[], &group.cci_segments, &group.cav_segments);
        }
    }

    /// Writes a generic SEQ group (raw segment replay).
    pub fn write_generic(
        doc: &mut EdifactDocumentWriter,
        group: &bo4e_extensions::GenericSeqGroup,
    ) {
        for raw in &group.raw_segments {
            doc.write_raw_segment(raw);
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
        // SEQ+Z98 (or Z81, preserved from original) with optional sub-ID
        let qualifier = b.edifact.seq_qualifier.as_deref().unwrap_or("Z98");
        let sub_id = b.edifact.seq_sub_id.as_deref().unwrap_or("");
        doc.write_segment("SEQ", &[qualifier, sub_id]);

        // Use raw_segments for roundtrip fidelity (preserves CCI/CAV/QTY order)
        if !b.edifact.raw_segments.is_empty() {
            for raw in &b.edifact.raw_segments {
                doc.write_raw_segment(raw);
            }
        } else {
            // Fallback: write parsed fields
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

            // QTY segments
            if let Some(jvp) = b.edifact.jahresverbrauchsprognose {
                let value = format!("{jvp}");
                doc.write_segment_with_composites("QTY", &[&["Z09", &value]]);
            }
            if let Some(ta) = b.edifact.temperatur_arbeit {
                let value = format!("{ta}");
                doc.write_segment_with_composites("QTY", &[&["265", &value]]);
            }
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
        assert!(output.contains("RFF+Z19:MELO001'"));
        // PIA+5 is NOT written in Z03 groups — it belongs to Z02 groups
        assert!(!output.contains("PIA+5"));
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
        let mut raw_dtm = HashMap::new();
        raw_dtm.insert("137".to_string(), "202507011330:303".to_string());
        raw_dtm.insert("471".to_string(), "202507011330:303".to_string());
        raw_dtm.insert("92".to_string(), "202507011330:303".to_string());
        let pd = Prozessdaten {
            transaktionsgrund: Some("E01".to_string()),
            transaktionsgrund_ergaenzung: Some("Z01".to_string()),
            prozessdatum: Some(dt),
            wirksamkeitsdatum: Some(dt),
            vertragsbeginn: Some(dt),
            raw_dtm,
            ..Default::default()
        };

        ProzessdatenWriter::write(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("DTM+137:202507011330:303'"));
        assert!(output.contains("DTM+471:202507011330:303'"));
        assert!(output.contains("DTM+92:202507011330:303'"));
        assert!(output.contains("STS+7++E01+Z01'"));
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
    fn test_prozessdaten_writer_sts_s21_format() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let pd = Prozessdaten {
            transaktionsgrund: Some("E01".to_string()),
            transaktionsgrund_ergaenzung: Some("ZW4".to_string()),
            transaktionsgrund_ergaenzung_befristete_anmeldung: Some("E03".to_string()),
            ..Default::default()
        };
        ProzessdatenWriter::write(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(
            output.contains("STS+7++E01+ZW4+E03'"),
            "STS should use S2.1 format, got: {}",
            output
        );
    }

    #[test]
    fn test_prozessdaten_writer_sts_befristete_without_ergaenzung() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let pd = Prozessdaten {
            transaktionsgrund: Some("E01".to_string()),
            transaktionsgrund_ergaenzung_befristete_anmeldung: Some("E03".to_string()),
            ..Default::default()
        };
        ProzessdatenWriter::write(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(
            output.contains("STS+7++E01++E03'"),
            "STS should have empty placeholder for missing ergaenzung, got: {}",
            output
        );
    }

    #[test]
    fn test_prozessdaten_writer_sts_grund_only() {
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let pd = Prozessdaten {
            transaktionsgrund: Some("E01".to_string()),
            ..Default::default()
        };
        ProzessdatenWriter::write(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(
            output.contains("STS+7++E01'"),
            "STS should have empty element after 7, got: {}",
            output
        );
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
                seq_qualifier: None,
                raw_pia: None,
                raw_cci_cav: Vec::new(),
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
    fn test_prozessdaten_writer_all_dtm_qualifiers() {
        let dt = NaiveDate::from_ymd_opt(2025, 7, 1)
            .unwrap()
            .and_hms_opt(13, 30, 0)
            .unwrap();
        let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default());
        doc.begin_interchange("S", None, "R", None, "REF", "D", "T", false);
        doc.begin_message("M", "TYPE");

        let mut raw_dtm = HashMap::new();
        raw_dtm.insert("Z51".to_string(), "202507011330:303".to_string());
        raw_dtm.insert("Z52".to_string(), "202507011330:303".to_string());
        raw_dtm.insert("Z53".to_string(), "202507011330:303".to_string());
        let pd = Prozessdaten {
            tag_des_empfangs: Some(dt),
            kuendigungsdatum_kunde: Some(dt),
            geplanter_liefertermin: Some(dt),
            raw_dtm,
            ..Default::default()
        };
        ProzessdatenWriter::write(&mut doc, &pd);
        doc.end_message();
        doc.end_interchange();

        let output = doc.output();
        assert!(output.contains("DTM+Z51:202507011330:303'"), "missing Z51");
        assert!(output.contains("DTM+Z52:202507011330:303'"), "missing Z52");
        assert!(output.contains("DTM+Z53:202507011330:303'"), "missing Z53");
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
                seq_qualifier: None,
                seq_sub_id: None,
                raw_qty: Vec::new(),
                raw_segments: Vec::new(),
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
