use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Process-level metadata for UTILMD transactions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prozessdaten {
    pub transaktionsgrund: Option<String>,
    pub transaktionsgrund_ergaenzung: Option<String>,
    pub transaktionsgrund_ergaenzung_befristete_anmeldung: Option<String>,
    pub prozessdatum: Option<NaiveDateTime>,
    pub wirksamkeitsdatum: Option<NaiveDateTime>,
    pub vertragsbeginn: Option<NaiveDateTime>,
    pub vertragsende: Option<NaiveDateTime>,
    pub lieferbeginndatum_in_bearbeitung: Option<NaiveDateTime>,
    pub datum_naechste_bearbeitung: Option<NaiveDateTime>,
    pub tag_des_empfangs: Option<NaiveDateTime>,
    pub kuendigungsdatum_kunde: Option<NaiveDateTime>,
    pub geplanter_liefertermin: Option<NaiveDateTime>,
    pub verwendung_der_daten_ab: Option<NaiveDateTime>,
    pub verwendung_der_daten_bis: Option<NaiveDateTime>,
    pub referenz_vorgangsnummer: Option<String>,
    pub anfrage_referenz: Option<String>,
    pub geplantes_paket: Option<String>,
    pub bemerkung: Option<String>,
    pub andere_partei_mp_id: Option<String>,
    /// Code qualifier for NAD+VY party (e.g. "293" for STROM, "332" for GAS, "9" for EAN)
    pub andere_partei_code_qualifier: Option<String>,
    /// Raw RFF segment following NAD+VY (e.g. "RFF+Z18:ref").
    pub andere_partei_rff: Option<String>,
    /// LOC+Z22 schlafende (sleeping/dormant) Marktlokation ID
    pub schlafende_marktlokation_id: Option<String>,

    // --- Additional RFF qualifiers ---
    /// RFF+AGI: Vorgangsnummer (C# AGI)
    pub vorgangsnummer: Option<String>,
    /// RFF+ACW: ReferenzVorgangsnummer
    pub referenz_vorgangsnummer_acw: Option<String>,
    /// RFF+AAV: AnfrageReferenz
    pub anfrage_referenz_aav: Option<String>,
    /// RFF+TN: ReferenzTransaktionsId
    pub referenz_transaktions_id: Option<String>,
    /// RFF+Z60: GeplantesPaket (if Z60 qualifier used)
    pub geplantes_paket_z60: Option<String>,
    /// RFF+Z42: Reference to prior transaction UNH
    pub rff_z42: Option<String>,

    // --- STS: Additional status segments ---
    /// Raw STS segments (E01/Z17/Z18/Z35) for roundtrip fidelity, in original order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sts_raw: Vec<String>,

    // --- Additional DTM qualifiers ---
    /// DTM+157: Wirksamkeitsdatum (alternative to 471)
    pub dtm_157: Option<NaiveDateTime>,
    /// DTM+Z01: WiederholungsDatum
    pub dtm_z01: Option<NaiveDateTime>,
    /// DTM+76: Lieferbeginndatum
    pub dtm_76: Option<NaiveDateTime>,
    /// DTM+154
    pub dtm_154: Option<NaiveDateTime>,
    /// DTM+Z05
    pub dtm_z05: Option<NaiveDateTime>,
    /// DTM+752: Recurring date (format 106 = MMDD)
    pub dtm_752: Option<NaiveDateTime>,
    /// DTM+158
    pub dtm_158: Option<NaiveDateTime>,
    /// DTM+159
    pub dtm_159: Option<NaiveDateTime>,
    /// DTM+672
    pub dtm_672: Option<NaiveDateTime>,
    /// DTM+155
    pub dtm_155: Option<NaiveDateTime>,

    // --- FTX raw storage for roundtrip ---
    /// All FTX segments as raw composites for roundtrip fidelity.
    /// Each entry is the full segment string (e.g. "FTX+ACB+++text").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_ftx: Vec<String>,

    // --- IMD raw storage for roundtrip ---
    /// All IMD segments as raw strings for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_imd: Vec<String>,

    /// Raw DTM composite values for roundtrip fidelity.
    /// Maps qualifier (e.g. "137") to raw value:format string (e.g. "202503311329?+00:303").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub raw_dtm: HashMap<String, String>,

    /// Ordered raw process-level DTM segment strings for roundtrip fidelity.
    /// Preferred over raw_dtm when non-empty; preserves ordering and duplicates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_process_dtms: Vec<String>,

    /// RFF+Z18: reference to prior document. Some("") = valueless, Some(val) = with value.
    pub rff_z18: Option<String>,
    /// RFF+Z01: Wiederholungsdatum reference
    pub rff_z01: Option<String>,
    /// RFF+Z31: Lokationsbuendel reference (e.g. Netzgebietsnummer).
    /// Stored as raw composite value for roundtrip fidelity.
    pub rff_z31: Option<String>,
    /// RFF+Z60: Folgenummer (sequence number).
    pub rff_z60: Option<String>,
    /// RFF+Z39: Vorgangs-ID (process ID).
    pub rff_z39: Option<String>,
    /// RFF+Z43: Referenz-Vorgangs-ID (reference process ID).
    pub rff_z43: Option<String>,
    /// Zeitscheibe reference blocks: each block is an RFF (Z49/Z50/Z53/Z47) followed by DTM+Z25/Z26.
    /// Stored in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zeitscheibe_refs: Vec<ZeitscheibeRef>,
}

/// Message-level metadata (from UNB/BGM/DTM segments).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nachrichtendaten {
    pub dokumentennummer: Option<String>,
    pub nachrichtenreferenz: Option<String>,
    pub absender_mp_id: Option<String>,
    pub empfaenger_mp_id: Option<String>,
    /// NAD+MS MP-ID (may differ from UNB sender in rare cases).
    /// When set, used for NAD+MS writing instead of absender_mp_id.
    pub nad_ms_mp_id: Option<String>,
    /// NAD+MR MP-ID (may differ from UNB recipient in rare cases).
    /// When set, used for NAD+MR writing instead of empfaenger_mp_id.
    pub nad_mr_mp_id: Option<String>,
    /// Code list qualifier from NAD+MS (e.g. "293" for STROM, "332" for GAS, "9" for EAN).
    pub absender_code_qualifier: Option<String>,
    /// Code list qualifier from NAD+MR (e.g. "293" for STROM, "332" for GAS, "9" for EAN).
    pub empfaenger_code_qualifier: Option<String>,
    pub erstellungsdatum: Option<NaiveDateTime>,
    pub datenaustauschreferenz: Option<String>,
    pub pruefidentifikator: Option<String>,
    pub kategorie: Option<String>,
    /// UNB sender identification code qualifier (e.g. "500"), element 1 component 1.
    pub absender_unb_qualifier: Option<String>,
    /// UNB recipient identification code qualifier (e.g. "500"), element 2 component 1.
    pub empfaenger_unb_qualifier: Option<String>,
    /// UNB preparation date (YYMMDD), element 3 component 0.
    pub unb_datum: Option<String>,
    /// UNB preparation time (HHMM), element 3 component 1.
    pub unb_zeit: Option<String>,
    /// Whether the original message had an explicit UNA service string.
    #[serde(default)]
    pub explicit_una: bool,
    /// UNA decimal mark character (b'.' or b',').
    /// Stored as byte value for serde compatibility. Default is b'.' (46).
    #[serde(default)]
    pub una_decimal_mark: u8,
    /// Original message type identifier from UNH (e.g. "UTILMD:D:11A:UN:S2.1").
    pub nachrichtentyp: Option<String>,
    /// Raw DTM+137 composite for message-level Nachrichtendatum roundtrip.
    /// Stores the full "value:format" string (e.g. "202503311329?+00:303").
    pub raw_nachrichtendatum: Option<String>,
    /// Contact details for sender (after NAD+MS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absender_ansprechpartner: Option<Ansprechpartner>,
    /// Contact details for recipient (after NAD+MR).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empfaenger_ansprechpartner: Option<Ansprechpartner>,
    /// Message-level passthrough segments (before first IDE).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passthrough_segments: Vec<crate::PassthroughSegment>,
    /// Original UNT segment count for byte-identical roundtrip.
    /// Many fixture files have incorrect UNT counts; we preserve them verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_unt_count: Option<String>,
    /// Original UNT message reference for byte-identical roundtrip.
    /// Some files have UNT references that differ from UNH, or omit them entirely.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_unt_reference: Option<String>,
}

/// A time slice reference within a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zeitscheibe {
    pub zeitscheiben_id: String,
    pub gueltigkeitszeitraum: Option<crate::zeitraum::Zeitraum>,
}

/// A zeitscheibe reference block: an RFF segment (Z49/Z50/Z53) followed by DTM+Z25/Z26.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZeitscheibeRef {
    /// Raw RFF segment (e.g. "RFF+Z49::2")
    pub raw_rff: String,
    /// Raw DTM segments following this RFF (DTM+Z25, DTM+Z26)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_dtms: Vec<String>,
}

/// Response status for answer messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Antwortstatus {
    pub status: Option<String>,
    pub grund: Option<String>,
    pub details: Option<Vec<String>>,
}

/// Contact details (CTA+IC followed by COM segments).
///
/// Appears after NAD+MS/MR in EDIFACT messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ansprechpartner {
    /// CTA+IC+:name — contact person name
    pub name: Option<String>,
    /// COM segments: list of (value, qualifier) pairs
    /// qualifier: "TE" (telephone), "FX" (fax), "EM" (email)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kommunikation: Vec<Kommunikationsdetail>,
}

/// A single COM entry (e.g. phone, fax, email).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kommunikationsdetail {
    pub value: String,
    pub qualifier: String, // "TE", "FX", "EM"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prozessdaten_default() {
        let pd = Prozessdaten::default();
        assert!(pd.transaktionsgrund.is_none());
        assert!(pd.prozessdatum.is_none());
    }

    #[test]
    fn test_nachrichtendaten_serde() {
        let nd = Nachrichtendaten {
            dokumentennummer: Some("DOC001".to_string()),
            absender_mp_id: Some("9900123000002".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&nd).unwrap();
        let de: Nachrichtendaten = serde_json::from_str(&json).unwrap();
        assert_eq!(de.dokumentennummer, Some("DOC001".to_string()));
    }

    #[test]
    fn test_nachrichtendaten_has_envelope_fields() {
        let nd = Nachrichtendaten {
            absender_unb_qualifier: Some("500".to_string()),
            empfaenger_unb_qualifier: Some("500".to_string()),
            unb_datum: Some("250331".to_string()),
            unb_zeit: Some("1329".to_string()),
            explicit_una: true,
            nachrichtentyp: Some("UTILMD:D:11A:UN:S2.1".to_string()),
            ..Default::default()
        };
        assert_eq!(nd.absender_unb_qualifier, Some("500".to_string()));
        assert_eq!(nd.unb_datum, Some("250331".to_string()));
        assert!(nd.explicit_una);
        assert_eq!(nd.nachrichtentyp.as_deref(), Some("UTILMD:D:11A:UN:S2.1"));
    }

    #[test]
    fn test_zeitscheibe_serde() {
        let zs = Zeitscheibe {
            zeitscheiben_id: "1".to_string(),
            gueltigkeitszeitraum: None,
        };
        let json = serde_json::to_string(&zs).unwrap();
        assert!(json.contains("\"zeitscheibenId\":\"1\""));
    }
}
