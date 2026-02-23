//! Placeholder BO4E types.
//!
//! These will be replaced by imports from the `bo4e` crate once available.
//! For now, we define minimal structs that satisfy the API contract.

use serde::{Deserialize, Serialize};

/// Marktlokation — a market location in the German energy market.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Marktlokation {
    pub marktlokations_id: Option<String>,
    pub sparte: Option<String>,
    pub lokationsadresse: Option<Adresse>,
    pub bilanzierungsmethode: Option<String>,
    pub netzebene: Option<String>,
}

/// Messlokation — a metering location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Messlokation {
    pub messlokations_id: Option<String>,
    pub sparte: Option<String>,
    pub messlokationszaehler: Option<Vec<String>>,
}

/// Netzlokation — a network location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Netzlokation {
    pub netzlokations_id: Option<String>,
    pub sparte: Option<String>,
}

/// SteuerbareRessource — a controllable resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteuerbareRessource {
    pub steuerbare_ressource_id: Option<String>,
}

/// TechnischeRessource — a technical resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnischeRessource {
    pub technische_ressource_id: Option<String>,
}

/// Tranche — a tranche.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tranche {
    pub tranche_id: Option<String>,
}

/// MabisZaehlpunkt — a MaBiS metering point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MabisZaehlpunkt {
    pub zaehlpunkt_id: Option<String>,
}

/// Geschaeftspartner — a business partner.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geschaeftspartner {
    pub name1: Option<String>,
    pub name2: Option<String>,
    pub gewerbekennzeichnung: Option<String>,
    pub geschaeftspartner_rolle: Option<Vec<String>>,
    pub partneradresse: Option<Adresse>,
}

/// Vertrag — a contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vertrag {
    pub vertragsnummer: Option<String>,
    pub vertragsart: Option<String>,
    pub vertragsbeginn: Option<String>,
    pub vertragsende: Option<String>,
}

/// Bilanzierung — balancing data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bilanzierung {
    pub bilanzkreis: Option<String>,
    pub regelzone: Option<String>,
    pub bilanzierungsgebiet: Option<String>,
}

/// Zaehler — a meter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zaehler {
    pub zaehlernummer: Option<String>,
    pub zaehlertyp: Option<String>,
    pub sparte: Option<String>,
}

/// Produktpaket — a product package.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Produktpaket {
    pub produktpaket_id: Option<String>,
}

/// Lokationszuordnung — a location assignment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lokationszuordnung {
    pub marktlokations_id: Option<String>,
    pub messlokations_id: Option<String>,
}

/// Marktteilnehmer — a market participant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Marktteilnehmer {
    pub mp_id: Option<String>,
    pub marktrolle: Option<String>,
}

/// Adresse — a postal address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Adresse {
    pub strasse: Option<String>,
    pub hausnummer: Option<String>,
    pub postleitzahl: Option<String>,
    pub ort: Option<String>,
    pub landescode: Option<String>,
}

/// Zaehlwerk — a meter register.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zaehlwerk {
    pub obis_kennzahl: Option<String>,
    pub bezeichnung: Option<String>,
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
            sparte: Some("STROM".to_string()),
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
