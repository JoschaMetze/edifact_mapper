//! Entity writers that serialize domain objects back to EDIFACT segments.
//!
//! Each writer knows how to produce the EDIFACT segments for one entity type.
//! They use `EdifactDocumentWriter` to append segments within an open message.

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
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        z: &WithValidity<Zaehler, ZaehlerEdifact>,
    ) {
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
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        v: &WithValidity<Vertrag, VertragEdifact>,
    ) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ::edifact_types::EdifactDelimiters;

    #[test]
    fn test_marktlokation_writer_loc() {
        let mut doc = EdifactDocumentWriter::with_delimiters(
            EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
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
        let mut doc = EdifactDocumentWriter::with_delimiters(
            EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
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
        let mut doc = EdifactDocumentWriter::with_delimiters(
            EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
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
        let mut doc = EdifactDocumentWriter::with_delimiters(
            EdifactDelimiters::default(),
            false,
        );
        doc.begin_interchange("S", "R", "REF", "D", "T");
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
}
