//! SEQ group data types for roundtrip-fidelity storage.
//!
//! Each SEQ group in a UTILMD transaction (Z45, Z71, Z21, Z08, Z01, Z20)
//! contains CCI/CAV segments that must be stored verbatim for byte-identical
//! roundtrip. Some groups also contain PIA and QTY segments.

use serde::{Deserialize, Serialize};

/// A single CCI segment's data for roundtrip-fidelity storage.
///
/// CCI segments can have various formats:
/// - `CCI+qualifier++code` → qualifier is element 0, code is component(2,0)
/// - `CCI+++code` → empty qualifier, code is component(2,0)
/// - `CCI+qualifier+additional+code` → all three present
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CciEntry {
    /// CCI element 0 qualifier (e.g. "Z30", "Z15", "Z99")
    pub qualifier: Option<String>,
    /// CCI element 1 additional qualifier
    pub additional_qualifier: Option<String>,
    /// CCI element 2 component 0 characteristic code (e.g. "Z07", "Z12")
    pub characteristic_code: Option<String>,
}

/// SEQ+Z45 — Stammdaten des Zählwerks (meter reading master data)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeqZ45Group {
    pub zeitscheibe_ref: Option<String>,
    /// PIA+Z02 component 0: ArtikelId
    pub artikel_id: Option<String>,
    /// PIA+Z02 component 1: ArtikelIdTyp (Z09/Z10)
    pub artikel_id_typ: Option<String>,
    /// QTY+Z38:value:unit — Wandlerfaktor (raw composite for roundtrip)
    pub wandlerfaktor: Option<String>,
    /// QTY+Z16:value:unit — Vorkommastelle (raw composite)
    pub vorkommastelle: Option<String>,
    /// QTY+Z37:value:unit — Nachkommastelle (raw composite)
    pub nachkommastelle: Option<String>,
    /// All CCI segments in this group
    pub cci_segments: Vec<CciEntry>,
    /// All CAV raw values (component 0,0 of each CAV)
    pub cav_segments: Vec<String>,
    /// Raw CCI/CAV segment strings in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// SEQ+Z71 — Abrechnungsdaten der Netzlokation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeqZ71Group {
    pub zeitscheibe_ref: Option<String>,
    /// PIA+Z02 component 0: ArtikelId
    pub artikel_id: Option<String>,
    /// PIA+Z02 component 1: ArtikelIdTyp (Z09/Z10)
    pub artikel_id_typ: Option<String>,
    /// All CCI segments in this group
    pub cci_segments: Vec<CciEntry>,
    /// All CAV raw values
    pub cav_segments: Vec<String>,
    /// Raw CCI/CAV segment strings in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// SEQ+Z21 — Profiltyp data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeqZ21Group {
    pub zeitscheibe_ref: Option<String>,
    /// Raw RFF segment strings inside this group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rff_segments: Vec<String>,
    /// All CCI segments in this group
    pub cci_segments: Vec<CciEntry>,
    /// All CAV raw values
    pub cav_segments: Vec<String>,
    /// Raw CCI/CAV segment strings in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// SEQ+Z08 — Messstellenart data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeqZ08Group {
    pub zeitscheibe_ref: Option<String>,
    /// Raw RFF segment strings inside this group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rff_segments: Vec<String>,
    /// All CCI segments in this group
    pub cci_segments: Vec<CciEntry>,
    /// All CAV raw values
    pub cav_segments: Vec<String>,
    /// Raw CCI/CAV segment strings in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// SEQ+Z01 — Marktrollen (market roles) and related data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeqZ01Group {
    pub zeitscheibe_ref: Option<String>,
    /// All CCI segments in this group
    pub cci_segments: Vec<CciEntry>,
    /// All CAV raw values
    pub cav_segments: Vec<String>,
    /// Raw QTY composites for roundtrip (e.g. "Z09:12345:KWH")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qty_segments: Vec<String>,
    /// Raw RFF segment strings inside this Z01 group (e.g. "RFF+Z18:value")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rff_segments: Vec<String>,
    /// Raw CCI/CAV segment strings in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// SEQ+Z20 — Technische Ressource details
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeqZ20Group {
    pub zeitscheibe_ref: Option<String>,
    /// Raw RFF segments (e.g. RFF+MG) within this Z20 group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rff_segments: Vec<String>,
    /// Raw PIA segments (e.g. PIA+5) within this Z20 group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pia_segments: Vec<String>,
    pub cci_segments: Vec<CciEntry>,
    pub cav_segments: Vec<String>,
    /// Raw CCI/CAV segment strings in original order for roundtrip fidelity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_cci_cav: Vec<String>,
}

/// Generic SEQ group for less common qualifiers (Z02, Z44, Z59, ZE1, Z98, etc.)
/// Stored as raw segments for roundtrip fidelity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericSeqGroup {
    /// The SEQ qualifier (e.g. "Z02", "Z44", "Z59", "ZE1")
    pub qualifier: String,
    pub zeitscheibe_ref: Option<String>,
    /// Raw segment strings for roundtrip (all segments in this group)
    pub raw_segments: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cci_entry_default() {
        let cci = CciEntry::default();
        assert!(cci.qualifier.is_none());
        assert!(cci.characteristic_code.is_none());
    }

    #[test]
    fn test_seq_z45_serde() {
        let group = SeqZ45Group {
            zeitscheibe_ref: Some("1".to_string()),
            artikel_id: Some("ART001".to_string()),
            artikel_id_typ: Some("Z09".to_string()),
            wandlerfaktor: Some("Z38:1:PCE".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&group).unwrap();
        let de: SeqZ45Group = serde_json::from_str(&json).unwrap();
        assert_eq!(de.artikel_id, Some("ART001".to_string()));
    }
}
