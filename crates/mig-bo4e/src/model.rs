//! Output model types for the MIG-driven mapping pipeline.
//!
//! Three-level hierarchy: `Interchange` → `Nachricht` → `Transaktion`
//! matching the EDIFACT structure: UNB/UNZ → UNH/UNT → IDE/SG4.

use mig_types::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// A complete EDIFACT interchange (UNB...UNZ) containing one or more messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interchange {
    /// Service segment data extracted from UNA/UNB/UNZ.
    /// Contains absender, empfaenger, interchange reference, etc.
    pub nachrichtendaten: serde_json::Value,

    /// One entry per UNH/UNT message pair in the interchange.
    pub nachrichten: Vec<Nachricht>,
}

/// A single EDIFACT message (UNH...UNT) within an interchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nachricht {
    /// UNH message reference number (first element of UNH segment).
    pub unh_referenz: String,

    /// Message type identifier from UNH (e.g., "UTILMD", "ORDERS").
    pub nachrichten_typ: String,

    /// Message-level BO4E entities (e.g., Marktteilnehmer from SG2).
    /// Mapped from definitions with `level = "message"` or from `message/` TOML directory.
    pub stammdaten: serde_json::Value,

    /// One entry per transaction group within this message
    /// (SG4 in UTILMD, each starting with IDE).
    pub transaktionen: Vec<Transaktion>,
}

/// A single transaction within an EDIFACT message.
///
/// In UTILMD, each SG4 group (starting with IDE) is one transaction.
/// Contains the mapped BO4E entities (stammdaten) and process metadata
/// (transaktionsdaten) extracted from the transaction group's root segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaktion {
    /// BO4E entities mapped from this transaction's segment groups.
    /// Keys are entity names (e.g., "Marktlokation", "Messlokation").
    pub stammdaten: serde_json::Value,

    /// Process metadata from the transaction group's root segments
    /// (IDE, STS, DTM in UTILMD). Not mapped to BO4E types.
    pub transaktionsdaten: serde_json::Value,
}

/// Intermediate result from mapping a single message's assembled tree.
///
/// Contains message-level stammdaten and per-transaction results.
/// Used by `MappingEngine::map_interchange()` before wrapping into `Nachricht`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappedMessage {
    /// Message-level BO4E entities (e.g., Marktteilnehmer from SG2).
    pub stammdaten: serde_json::Value,

    /// Per-transaction results (one per SG4 instance).
    pub transaktionen: Vec<Transaktion>,
}

/// Extract message reference and message type from a UNH segment.
pub fn extract_unh_fields(unh: &OwnedSegment) -> (String, String) {
    let referenz = unh.get_element(0).to_string();
    let typ = unh.get_component(1, 0).to_string();
    (referenz, typ)
}

/// Extract interchange-level metadata from envelope segments (UNB).
pub fn extract_nachrichtendaten(envelope: &[OwnedSegment]) -> serde_json::Value {
    let mut result = serde_json::Map::new();

    for seg in envelope {
        if seg.is("UNB") {
            // UNB+syntax+sender+recipient+date+ref
            if !seg.get_component(0, 0).is_empty() {
                result.insert(
                    "syntaxKennung".to_string(),
                    serde_json::Value::String(seg.get_component(0, 0).to_string()),
                );
            }
            if !seg.get_component(1, 0).is_empty() {
                result.insert(
                    "absenderCode".to_string(),
                    serde_json::Value::String(seg.get_component(1, 0).to_string()),
                );
            }
            if !seg.get_component(2, 0).is_empty() {
                result.insert(
                    "empfaengerCode".to_string(),
                    serde_json::Value::String(seg.get_component(2, 0).to_string()),
                );
            }
            if !seg.get_component(3, 0).is_empty() {
                result.insert(
                    "datum".to_string(),
                    serde_json::Value::String(seg.get_component(3, 0).to_string()),
                );
            }
            if !seg.get_component(3, 1).is_empty() {
                result.insert(
                    "zeit".to_string(),
                    serde_json::Value::String(seg.get_component(3, 1).to_string()),
                );
            }
            if !seg.get_element(4).is_empty() {
                result.insert(
                    "interchangeRef".to_string(),
                    serde_json::Value::String(seg.get_element(4).to_string()),
                );
            }
        }
    }

    serde_json::Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaktion_serde_roundtrip() {
        let tx = Transaktion {
            stammdaten: serde_json::json!({
                "Marktlokation": { "marktlokationsId": "DE000111222333" }
            }),
            transaktionsdaten: serde_json::json!({
                "vorgangId": "TX001",
                "transaktionsgrund": "E01"
            }),
        };

        let json = serde_json::to_string(&tx).unwrap();
        let de: Transaktion = serde_json::from_str(&json).unwrap();
        assert_eq!(de.transaktionsdaten["vorgangId"].as_str().unwrap(), "TX001");
        assert!(de.stammdaten["Marktlokation"].is_object());
    }

    #[test]
    fn test_nachricht_serde_roundtrip() {
        let msg = Nachricht {
            unh_referenz: "00001".to_string(),
            nachrichten_typ: "UTILMD".to_string(),
            stammdaten: serde_json::json!({
                "Marktteilnehmer": [
                    { "marktrolle": "MS", "rollencodenummer": "9900123" }
                ]
            }),
            transaktionen: vec![Transaktion {
                stammdaten: serde_json::json!({}),
                transaktionsdaten: serde_json::json!({}),
            }],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let de: Nachricht = serde_json::from_str(&json).unwrap();
        assert_eq!(de.unh_referenz, "00001");
        assert_eq!(de.nachrichten_typ, "UTILMD");
        assert_eq!(de.transaktionen.len(), 1);
    }

    #[test]
    fn test_interchange_serde_roundtrip() {
        let interchange = Interchange {
            nachrichtendaten: serde_json::json!({
                "absender": "9900123456789",
                "empfaenger": "9900987654321"
            }),
            nachrichten: vec![Nachricht {
                unh_referenz: "00001".to_string(),
                nachrichten_typ: "UTILMD".to_string(),
                stammdaten: serde_json::json!({}),
                transaktionen: vec![],
            }],
        };

        let json = serde_json::to_string_pretty(&interchange).unwrap();
        let de: Interchange = serde_json::from_str(&json).unwrap();
        assert_eq!(de.nachrichten.len(), 1);
        assert_eq!(de.nachrichten[0].unh_referenz, "00001");
    }

    #[test]
    fn test_extract_envelope_from_segments() {
        use mig_types::segment::OwnedSegment;

        let envelope = vec![OwnedSegment {
            id: "UNB".to_string(),
            elements: vec![
                vec!["UNOC".to_string(), "3".to_string()],
                vec!["9900123456789".to_string(), "500".to_string()],
                vec!["9900987654321".to_string(), "500".to_string()],
                vec!["210101".to_string(), "1200".to_string()],
                vec!["REF001".to_string()],
            ],
            segment_number: 0,
        }];

        let nd = extract_nachrichtendaten(&envelope);
        assert_eq!(nd["absenderCode"].as_str().unwrap(), "9900123456789");
        assert_eq!(nd["empfaengerCode"].as_str().unwrap(), "9900987654321");
        assert_eq!(nd["interchangeRef"].as_str().unwrap(), "REF001");
        assert_eq!(nd["syntaxKennung"].as_str().unwrap(), "UNOC");
        assert_eq!(nd["datum"].as_str().unwrap(), "210101");
        assert_eq!(nd["zeit"].as_str().unwrap(), "1200");
    }

    #[test]
    fn test_extract_unh_fields() {
        use mig_types::segment::OwnedSegment;

        let unh = OwnedSegment {
            id: "UNH".to_string(),
            elements: vec![
                vec!["MSG001".to_string()],
                vec![
                    "UTILMD".to_string(),
                    "D".to_string(),
                    "11A".to_string(),
                    "UN".to_string(),
                    "S2.1".to_string(),
                ],
            ],
            segment_number: 0,
        };

        let (referenz, typ) = extract_unh_fields(&unh);
        assert_eq!(referenz, "MSG001");
        assert_eq!(typ, "UTILMD");
    }

    #[test]
    fn test_rebuild_unb_from_nachrichtendaten() {
        let nd = serde_json::json!({
            "syntaxKennung": "UNOC",
            "absenderCode": "9900123456789",
            "empfaengerCode": "9900987654321",
            "datum": "210101",
            "zeit": "1200",
            "interchangeRef": "REF001"
        });

        let unb = rebuild_unb(&nd);
        assert_eq!(unb.id, "UNB");
        // UNB+UNOC:3+sender:500+receiver:500+date:time+ref
        assert_eq!(unb.elements[0], vec!["UNOC", "3"]);
        assert_eq!(unb.elements[1][0], "9900123456789");
        assert_eq!(unb.elements[2][0], "9900987654321");
        assert_eq!(unb.elements[3], vec!["210101", "1200"]);
        assert_eq!(unb.elements[4], vec!["REF001"]);
    }

    #[test]
    fn test_rebuild_unb_defaults() {
        // Empty nachrichtendaten — should produce valid UNB with placeholders
        let nd = serde_json::json!({});
        let unb = rebuild_unb(&nd);
        assert_eq!(unb.id, "UNB");
        assert_eq!(unb.elements[0], vec!["UNOC", "3"]);
    }

    #[test]
    fn test_rebuild_unh() {
        let unh = rebuild_unh("00001", "UTILMD");
        assert_eq!(unh.id, "UNH");
        assert_eq!(unh.elements[0], vec!["00001"]);
        assert_eq!(unh.elements[1][0], "UTILMD");
        assert_eq!(unh.elements[1][1], "D");
        assert_eq!(unh.elements[1][2], "11A");
        assert_eq!(unh.elements[1][3], "UN");
        assert_eq!(unh.elements[1][4], "S2.1");
    }

    #[test]
    fn test_rebuild_unt() {
        let unt = rebuild_unt(25, "00001");
        assert_eq!(unt.id, "UNT");
        assert_eq!(unt.elements[0], vec!["25"]);
        assert_eq!(unt.elements[1], vec!["00001"]);
    }

    #[test]
    fn test_rebuild_unz() {
        let unz = rebuild_unz(1, "REF001");
        assert_eq!(unz.id, "UNZ");
        assert_eq!(unz.elements[0], vec!["1"]);
        assert_eq!(unz.elements[1], vec!["REF001"]);
    }

    #[test]
    fn test_roundtrip_nachrichtendaten_rebuild() {
        // extract_nachrichtendaten() → rebuild_unb() should preserve fields
        let original = OwnedSegment {
            id: "UNB".to_string(),
            elements: vec![
                vec!["UNOC".to_string(), "3".to_string()],
                vec!["9900123456789".to_string(), "500".to_string()],
                vec!["9900987654321".to_string(), "500".to_string()],
                vec!["210101".to_string(), "1200".to_string()],
                vec!["REF001".to_string()],
            ],
            segment_number: 0,
        };

        let nd = extract_nachrichtendaten(&[original]);
        let rebuilt = rebuild_unb(&nd);
        assert_eq!(rebuilt.elements[0], vec!["UNOC", "3"]);
        assert_eq!(rebuilt.elements[1][0], "9900123456789");
        assert_eq!(rebuilt.elements[2][0], "9900987654321");
        assert_eq!(rebuilt.elements[3], vec!["210101", "1200"]);
        assert_eq!(rebuilt.elements[4], vec!["REF001"]);
    }
}
