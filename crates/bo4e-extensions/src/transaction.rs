use serde::{Deserialize, Serialize};

use crate::bo4e_types::*;
use crate::edifact_types::*;
use crate::passthrough::PassthroughSegment;
use crate::prozessdaten::*;
use crate::seq_groups::*;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilmdTransaktion {
    pub transaktions_id: String,
    /// Original IDE qualifier (e.g. "24" or deprecated "Z01").
    /// Default is "24" for standard UTILMD messages.
    #[serde(
        default = "default_ide_qualifier",
        skip_serializing_if = "is_default_ide"
    )]
    pub ide_qualifier: String,
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

    // --- SEQ groups for roundtrip fidelity ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_z45_groups: Vec<SeqZ45Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_z71_groups: Vec<SeqZ71Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_z21_groups: Vec<SeqZ21Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_z08_groups: Vec<SeqZ08Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_z01_groups: Vec<SeqZ01Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_z20_groups: Vec<SeqZ20Group>,
    /// Generic SEQ groups for less common qualifiers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_seq_groups: Vec<GenericSeqGroup>,

    /// Order in which SEQ groups appeared in the original message.
    /// Each entry is a (qualifier, index_within_type) pair, e.g. ("Z01", 0), ("Z02", 0), ("Z03", 0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seq_group_order: Vec<(String, usize)>,

    /// Order in which entity LOC segments appeared in the original message.
    /// Each entry is a LOC qualifier (Z18, Z16, Z17, Z20, Z19, Z21, Z15).
    /// Used to replay entities in the same order for byte-identical roundtrip.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entity_loc_order: Vec<String>,

    /// Order in which NAD qualifiers appeared in the original message.
    /// Used to replay NAD segments in the same order for byte-identical roundtrip.
    /// Includes both party NADs (Z04, Z09, etc.) and address NADs (DP, Z63).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nad_qualifier_order: Vec<String>,

    /// Segments not handled by any mapper, preserved for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub passthrough_segments: Vec<PassthroughSegment>,
}

fn default_ide_qualifier() -> String {
    "24".to_string()
}

fn is_default_ide(q: &str) -> bool {
    q == "24"
}

impl Default for UtilmdTransaktion {
    fn default() -> Self {
        Self {
            transaktions_id: String::new(),
            ide_qualifier: "24".to_string(),
            referenz_transaktions_id: None,
            absender: Marktteilnehmer::default(),
            empfaenger: Marktteilnehmer::default(),
            prozessdaten: Prozessdaten::default(),
            antwortstatus: None,
            zeitscheiben: Vec::new(),
            marktlokationen: Vec::new(),
            messlokationen: Vec::new(),
            netzlokationen: Vec::new(),
            steuerbare_ressourcen: Vec::new(),
            technische_ressourcen: Vec::new(),
            tranchen: Vec::new(),
            mabis_zaehlpunkte: Vec::new(),
            parteien: Vec::new(),
            vertrag: None,
            bilanzierung: None,
            zaehler: Vec::new(),
            produktpakete: Vec::new(),
            lokationszuordnungen: Vec::new(),
            seq_z45_groups: Vec::new(),
            seq_z71_groups: Vec::new(),
            seq_z21_groups: Vec::new(),
            seq_z08_groups: Vec::new(),
            seq_z01_groups: Vec::new(),
            seq_z20_groups: Vec::new(),
            generic_seq_groups: Vec::new(),
            seq_group_order: Vec::new(),
            entity_loc_order: Vec::new(),
            nad_qualifier_order: Vec::new(),
            passthrough_segments: Vec::new(),
        }
    }
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
