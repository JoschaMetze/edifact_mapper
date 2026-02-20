//! EDIFACT companion types that store functional domain data
//! not present in standard BO4E.

use serde::{Deserialize, Serialize};

use crate::data_quality::DataQuality;

/// EDIFACT companion for Marktlokation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarktlokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_netzlokation: Option<String>,
    pub vorgelagerte_lokations_ids: Option<Vec<LokationsTypZuordnung>>,
    /// Raw NAD+DP/Z63 segment strings for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_nad_address: Vec<String>,
    /// Raw LOC+Z16 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for Messlokation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MesslokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_netzlokation: Option<String>,
    pub vorgelagerte_lokations_ids: Option<Vec<LokationsTypZuordnung>>,
    /// Raw LOC+Z17 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for Zaehler.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZaehlerEdifact {
    pub referenz_messlokation: Option<String>,
    pub referenz_gateway: Option<String>,
    pub produktpaket_id: Option<String>,
    pub is_smartmeter_gateway: Option<bool>,
    pub smartmeter_gateway_zuordnung: Option<String>,
    /// SEQ sub-ID (e.g. the "1" in SEQ+Z03+1) for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq_sub_id: Option<String>,
    /// Raw CCI/CAV segments within SEQ+Z03 for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
    /// Raw QTY segments within SEQ+Z03 for roundtrip fidelity (Z33, Z34, 31, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_qty: Vec<String>,
}

/// EDIFACT companion for Geschaeftspartner.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeschaeftspartnerEdifact {
    pub nad_qualifier: Option<String>,
    /// Raw NAD segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_nad: Option<String>,
    /// Raw RFF segments following this NAD (e.g. RFF+Z18:ref, RFF+Z01:ref).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_rffs: Vec<String>,
}

/// EDIFACT companion for Vertrag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VertragEdifact {
    pub haushaltskunde: Option<bool>,
    pub versorgungsart: Option<String>,
}

/// EDIFACT companion for Netzlokation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetzlokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_marktlokation: Option<String>,
    pub zugeordnete_messlokationen: Option<Vec<LokationsTypZuordnung>>,
    /// Raw LOC+Z18 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for TechnischeRessource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnischeRessourceEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_marktlokation: Option<String>,
    pub referenz_steuerbare_ressource: Option<String>,
    pub referenz_messlokation: Option<String>,
    /// Raw LOC+Z20 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for SteuerbareRessource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SteuerbareRessourceEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub produktpaket_id: Option<String>,
    /// Raw LOC+Z19 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for Tranche (placeholder).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrancheEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    /// Raw LOC+Z21 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for MabisZaehlpunkt (placeholder).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MabisZaehlpunktEdifact {
    pub zaehlpunkt_typ: Option<String>,
    /// Raw LOC+Z15 segment string for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_loc: Option<String>,
}

/// EDIFACT companion for Bilanzierung.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BilanzierungEdifact {
    pub temperatur_arbeit: Option<f64>,
    pub jahresverbrauchsprognose: Option<f64>,
    /// SEQ qualifier used for this Bilanzierung (Z98 or Z81).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq_qualifier: Option<String>,
    /// SEQ sub-ID (e.g. the "1" in SEQ+Z81+1) for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq_sub_id: Option<String>,
    /// Raw QTY segments for roundtrip fidelity (preserves unit codes like :Z16).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_qty: Vec<String>,
    /// All raw segments (CCI, CAV, QTY, RFF) in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_segments: Vec<String>,
}

/// EDIFACT companion for Produktpaket.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProduktpaketEdifact {
    pub produktpaket_name: Option<String>,
    /// SEQ qualifier used for this Produktpaket (Z79 or ZH0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq_qualifier: Option<String>,
    /// Raw PIA segment for roundtrip fidelity (e.g. "PIA+5+9991000002082:Z11").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_pia: Option<String>,
    /// Raw CCI/CAV segments in Z79/ZH0 group for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// EDIFACT companion for Lokationszuordnung.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LokationszuordnungEdifact {
    pub zuordnungstyp: Option<String>,
    /// SEQ sub-ID (e.g. "1" in SEQ+Z78+1) for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq_sub_id: Option<String>,
    /// Raw RFF segments inside the Z78 group for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_rffs: Vec<String>,
}

/// A location type assignment (used in vorgelagerte_lokations_ids).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LokationsTypZuordnung {
    pub lokations_id: String,
    pub lokationstyp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marktlokation_edifact_default() {
        let e = MarktlokationEdifact::default();
        assert!(e.datenqualitaet.is_none());
        assert!(e.referenz_netzlokation.is_none());
    }

    #[test]
    fn test_zaehler_edifact_serde() {
        let e = ZaehlerEdifact {
            referenz_messlokation: Some("MELO001".to_string()),
            is_smartmeter_gateway: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_string(&e).unwrap();
        let deserialized: ZaehlerEdifact = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.referenz_messlokation,
            Some("MELO001".to_string())
        );
        assert_eq!(deserialized.is_smartmeter_gateway, Some(true));
    }

    #[test]
    fn test_all_edifact_types_default() {
        // Verify all types implement Default
        let _ = MarktlokationEdifact::default();
        let _ = MesslokationEdifact::default();
        let _ = ZaehlerEdifact::default();
        let _ = GeschaeftspartnerEdifact::default();
        let _ = VertragEdifact::default();
        let _ = NetzlokationEdifact::default();
        let _ = TechnischeRessourceEdifact::default();
        let _ = SteuerbareRessourceEdifact::default();
        let _ = TrancheEdifact::default();
        let _ = MabisZaehlpunktEdifact::default();
        let _ = BilanzierungEdifact::default();
        let _ = ProduktpaketEdifact::default();
        let _ = LokationszuordnungEdifact::default();
    }
}
