//! BO4E types for EDIFACT mapping.
//!
//! Most types are re-exported from the `bo4e-german` crate.
//! EDIFACT-specific types not present in standard BO4E are defined locally.

use serde::{Deserialize, Serialize};

// --- Re-exports from bo4e-german ---
pub use bo4e_german::{
    Adresse, Bilanzierung, Geschaeftspartner, Lokationszuordnung, Marktlokation, Marktteilnehmer,
    Messlokation, Netzlokation, SteuerbareRessource, TechnischeRessource, Vertrag, Zaehler,
    Zaehlwerk,
};

// --- EDIFACT-specific types not in standard BO4E ---

/// Tranche — a tranche.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tranche {
    pub tranche_id: Option<String>,
}

/// MabisZaehlpunkt — a MaBiS metering point.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MabisZaehlpunkt {
    pub zaehlpunkt_id: Option<String>,
}

/// Produktpaket — a product package.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Produktpaket {
    pub produktpaket_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marktlokation_default() {
        let ml = Marktlokation::default();
        assert!(ml.marktlokations_id.is_none());
    }

    #[test]
    fn test_marktlokation_serde_roundtrip() {
        let ml = Marktlokation {
            marktlokations_id: Some("DE00014545768S0000000000000003054".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&ml).unwrap();
        let deserialized: Marktlokation = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.marktlokations_id,
            Some("DE00014545768S0000000000000003054".to_string())
        );
    }
}
