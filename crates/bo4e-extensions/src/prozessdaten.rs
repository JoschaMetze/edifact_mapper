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
    fn test_zeitscheibe_serde() {
        let zs = Zeitscheibe {
            zeitscheiben_id: "1".to_string(),
            gueltigkeitszeitraum: None,
        };
        let json = serde_json::to_string(&zs).unwrap();
        assert!(json.contains("\"zeitscheiben_id\":\"1\""));
    }
}
