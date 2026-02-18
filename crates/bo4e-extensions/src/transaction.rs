use serde::{Deserialize, Serialize};

use crate::bo4e_types::*;
use crate::edifact_types::*;
use crate::prozessdaten::*;
use crate::with_validity::WithValidity;

/// A complete UTILMD message containing one or more transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilmdNachricht {
    pub nachrichtendaten: Nachrichtendaten,
    pub dokumentennummer: String,
    pub kategorie: Option<String>,
    pub transaktionen: Vec<UtilmdTransaktion>,
}

/// A single UTILMD transaction (IDE segment group).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UtilmdTransaktion {
    pub transaktions_id: String,
    pub referenz_transaktions_id: Option<String>,
    pub absender: Marktteilnehmer,
    pub empfaenger: Marktteilnehmer,
    pub prozessdaten: Prozessdaten,
    pub antwortstatus: Option<Antwortstatus>,
    pub zeitscheiben: Vec<Zeitscheibe>,
    pub marktlokationen: Vec<WithValidity<Marktlokation, MarktlokationEdifact>>,
    pub messlokationen: Vec<WithValidity<Messlokation, MesslokationEdifact>>,
    pub netzlokationen: Vec<WithValidity<Netzlokation, NetzlokationEdifact>>,
    pub steuerbare_ressourcen: Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>,
    pub technische_ressourcen: Vec<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>>,
    pub tranchen: Vec<WithValidity<Tranche, TrancheEdifact>>,
    pub mabis_zaehlpunkte: Vec<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>>,
    pub parteien: Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>,
    pub vertrag: Option<WithValidity<Vertrag, VertragEdifact>>,
    pub bilanzierung: Option<WithValidity<Bilanzierung, BilanzierungEdifact>>,
    pub zaehler: Vec<WithValidity<Zaehler, ZaehlerEdifact>>,
    pub produktpakete: Vec<WithValidity<Produktpaket, ProduktpaketEdifact>>,
    pub lokationszuordnungen: Vec<WithValidity<Lokationszuordnung, LokationszuordnungEdifact>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utilmd_transaktion_default() {
        let tx = UtilmdTransaktion::default();
        assert!(tx.transaktions_id.is_empty());
        assert!(tx.marktlokationen.is_empty());
        assert!(tx.vertrag.is_none());
    }

    #[test]
    fn test_utilmd_transaktion_serde_roundtrip() {
        let tx = UtilmdTransaktion {
            transaktions_id: "TX001".to_string(),
            absender: Marktteilnehmer {
                mp_id: Some("9900123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&tx).unwrap();
        let de: UtilmdTransaktion = serde_json::from_str(&json).unwrap();
        assert_eq!(de.transaktions_id, "TX001");
        assert_eq!(de.absender.mp_id, Some("9900123".to_string()));
    }

    #[test]
    fn test_utilmd_nachricht_serde() {
        let msg = UtilmdNachricht {
            nachrichtendaten: Nachrichtendaten::default(),
            dokumentennummer: "DOC001".to_string(),
            kategorie: Some("E03".to_string()),
            transaktionen: vec![UtilmdTransaktion::default()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let de: UtilmdNachricht = serde_json::from_str(&json).unwrap();
        assert_eq!(de.dokumentennummer, "DOC001");
        assert_eq!(de.transaktionen.len(), 1);
    }
}
