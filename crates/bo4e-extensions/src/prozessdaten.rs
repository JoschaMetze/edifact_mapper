use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Process-level metadata for UTILMD transactions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Raw DTM composite values for roundtrip fidelity.
    /// Maps qualifier (e.g. "137") to raw value:format string (e.g. "202503311329?+00:303").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub raw_dtm: HashMap<String, String>,
}

/// Message-level metadata (from UNB/BGM/DTM segments).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nachrichtendaten {
    pub dokumentennummer: Option<String>,
    pub nachrichtenreferenz: Option<String>,
    pub absender_mp_id: Option<String>,
    pub empfaenger_mp_id: Option<String>,
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
    /// Original message type identifier from UNH (e.g. "UTILMD:D:11A:UN:S2.1").
    pub nachrichtentyp: Option<String>,
    /// Raw DTM+137 composite for message-level Nachrichtendatum roundtrip.
    /// Stores the full "value:format" string (e.g. "202503311329?+00:303").
    pub raw_nachrichtendatum: Option<String>,
}

/// A time slice reference within a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zeitscheibe {
    pub zeitscheiben_id: String,
    pub gueltigkeitszeitraum: Option<crate::zeitraum::Zeitraum>,
}

/// Response status for answer messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Antwortstatus {
    pub status: Option<String>,
    pub grund: Option<String>,
    pub details: Option<Vec<String>>,
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
        assert!(json.contains("\"zeitscheiben_id\":\"1\""));
    }
}
