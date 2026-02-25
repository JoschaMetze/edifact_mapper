//! V2 reverse conversion request/response types.
//!
//! Accepts BO4E JSON and converts back to EDIFACT.

use serde::{Deserialize, Serialize};

/// Input level for the reverse endpoint.
#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InputLevel {
    /// Full interchange JSON (nachrichtendaten + nachrichten array).
    Interchange,
    /// Single message JSON (unhReferenz, nachrichtenTyp, stammdaten, transaktionen).
    Nachricht,
    /// Single transaction JSON (stammdaten, transaktionsdaten).
    #[default]
    Transaktion,
}

/// Output mode for the reverse endpoint.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReverseMode {
    /// Return EDIFACT string.
    Edifact,
    /// Return the assembled MIG tree as JSON (debugging).
    MigTree,
}

/// Optional envelope overrides for missing levels.
///
/// When input is `nachricht` or `transaktion`, these values fill in
/// the envelope segments that aren't present in the input.
#[derive(Debug, Clone, Deserialize, Default, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeOverrides {
    pub absender_code: Option<String>,
    pub empfaenger_code: Option<String>,
    pub nachrichten_typ: Option<String>,
}

/// Request body for `POST /api/v2/reverse`.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReverseV2Request {
    /// The BO4E JSON to convert back to EDIFACT.
    /// Shape depends on `level`.
    #[schema(value_type = Object)]
    pub input: serde_json::Value,

    /// Which level the input represents.
    pub level: InputLevel,

    /// Format version (e.g., "FV2504").
    pub format_version: String,

    /// Output mode: "edifact" or "mig-tree".
    #[serde(default = "default_mode")]
    pub mode: ReverseMode,

    /// Optional envelope overrides for missing levels.
    #[serde(default)]
    pub envelope: Option<EnvelopeOverrides>,
}

fn default_mode() -> ReverseMode {
    ReverseMode::Edifact
}

/// Response body for `POST /api/v2/reverse`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReverseV2Response {
    /// The mode used for conversion.
    pub mode: String,

    /// The result: EDIFACT string or MIG tree JSON.
    #[schema(value_type = Object)]
    pub result: serde_json::Value,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,
}

/// Normalize input JSON to an `Interchange`, wrapping lower-level inputs as needed.
pub fn normalize_to_interchange(
    input: &serde_json::Value,
    level: &InputLevel,
    overrides: Option<&EnvelopeOverrides>,
) -> Result<mig_bo4e::Interchange, String> {
    match level {
        InputLevel::Interchange => serde_json::from_value(input.clone())
            .map_err(|e| format!("Invalid interchange JSON: {e}")),
        InputLevel::Nachricht => {
            let nachricht: mig_bo4e::Nachricht = serde_json::from_value(input.clone())
                .map_err(|e| format!("Invalid nachricht JSON: {e}"))?;

            let nachrichten_typ = overrides
                .and_then(|o| o.nachrichten_typ.clone())
                .unwrap_or_else(|| nachricht.nachrichten_typ.clone());

            let nd = build_default_nachrichtendaten(overrides);

            Ok(mig_bo4e::Interchange {
                nachrichtendaten: serde_json::Value::Object(nd),
                nachrichten: vec![mig_bo4e::Nachricht {
                    nachrichten_typ,
                    ..nachricht
                }],
            })
        }
        InputLevel::Transaktion => {
            let tx: mig_bo4e::Transaktion = serde_json::from_value(input.clone())
                .map_err(|e| format!("Invalid transaktion JSON: {e}"))?;

            let nachrichten_typ = overrides
                .and_then(|o| o.nachrichten_typ.clone())
                .unwrap_or_else(|| "UTILMD".to_string());

            let nd = build_default_nachrichtendaten(overrides);

            Ok(mig_bo4e::Interchange {
                nachrichtendaten: serde_json::Value::Object(nd),
                nachrichten: vec![mig_bo4e::Nachricht {
                    unh_referenz: "00001".to_string(),
                    nachrichten_typ,
                    stammdaten: serde_json::json!({}),
                    transaktionen: vec![tx],
                }],
            })
        }
    }
}

/// Build default nachrichtendaten JSON map from optional overrides.
fn build_default_nachrichtendaten(
    overrides: Option<&EnvelopeOverrides>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut nd = serde_json::Map::new();
    nd.insert("syntaxKennung".to_string(), serde_json::json!("UNOC"));
    if let Some(o) = overrides {
        if let Some(ref s) = o.absender_code {
            nd.insert("absenderCode".to_string(), serde_json::json!(s));
        }
        if let Some(ref r) = o.empfaenger_code {
            nd.insert("empfaengerCode".to_string(), serde_json::json!(r));
        }
    }
    nd.insert("interchangeRef".to_string(), serde_json::json!("00000"));
    nd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_interchange_level() {
        let input = serde_json::json!({
            "nachrichtendaten": { "absenderCode": "9900123" },
            "nachrichten": [{
                "unhReferenz": "00001",
                "nachrichtenTyp": "UTILMD",
                "stammdaten": {},
                "transaktionen": []
            }]
        });

        let interchange = normalize_to_interchange(&input, &InputLevel::Interchange, None).unwrap();
        assert_eq!(interchange.nachrichten.len(), 1);
        assert_eq!(interchange.nachrichten[0].unh_referenz, "00001");
    }

    #[test]
    fn test_normalize_nachricht_level() {
        let input = serde_json::json!({
            "unhReferenz": "00001",
            "nachrichtenTyp": "UTILMD",
            "stammdaten": { "Marktteilnehmer": [] },
            "transaktionen": [{ "stammdaten": {}, "transaktionsdaten": {} }]
        });

        let interchange = normalize_to_interchange(&input, &InputLevel::Nachricht, None).unwrap();
        assert_eq!(interchange.nachrichten.len(), 1);
        assert_eq!(interchange.nachrichten[0].unh_referenz, "00001");
    }

    #[test]
    fn test_normalize_transaktion_level() {
        let input = serde_json::json!({
            "stammdaten": { "Marktlokation": {} },
            "transaktionsdaten": { "pruefidentifikator": "55001" }
        });

        let overrides = EnvelopeOverrides {
            absender_code: Some("9900123".to_string()),
            empfaenger_code: Some("9900456".to_string()),
            nachrichten_typ: Some("UTILMD".to_string()),
        };

        let interchange =
            normalize_to_interchange(&input, &InputLevel::Transaktion, Some(&overrides)).unwrap();
        assert_eq!(interchange.nachrichten.len(), 1);
        assert_eq!(interchange.nachrichten[0].transaktionen.len(), 1);
        assert_eq!(interchange.nachrichten[0].nachrichten_typ, "UTILMD");
    }

    #[test]
    fn test_normalize_transaktion_default_nachrichten_typ() {
        let input = serde_json::json!({
            "stammdaten": {},
            "transaktionsdaten": { "pruefidentifikator": "55001" }
        });

        let interchange = normalize_to_interchange(&input, &InputLevel::Transaktion, None).unwrap();
        // Without overrides, defaults to "UTILMD"
        assert_eq!(interchange.nachrichten[0].nachrichten_typ, "UTILMD");
    }

    #[test]
    fn test_normalize_nachricht_overrides_nachrichten_typ() {
        let input = serde_json::json!({
            "unhReferenz": "00001",
            "nachrichtenTyp": "ORDERS",
            "stammdaten": {},
            "transaktionen": []
        });

        let overrides = EnvelopeOverrides {
            absender_code: None,
            empfaenger_code: None,
            nachrichten_typ: Some("UTILMD".to_string()),
        };

        let interchange =
            normalize_to_interchange(&input, &InputLevel::Nachricht, Some(&overrides)).unwrap();
        // Overrides should take precedence
        assert_eq!(interchange.nachrichten[0].nachrichten_typ, "UTILMD");
    }

    #[test]
    fn test_normalize_interchange_invalid_json() {
        let input = serde_json::json!("not an object");
        let result = normalize_to_interchange(&input, &InputLevel::Interchange, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid interchange JSON"));
    }

    #[test]
    fn test_deserialize_input_level() {
        let json = r#""interchange""#;
        let level: InputLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, InputLevel::Interchange);

        let json = r#""nachricht""#;
        let level: InputLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, InputLevel::Nachricht);

        let json = r#""transaktion""#;
        let level: InputLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level, InputLevel::Transaktion);
    }

    #[test]
    fn test_deserialize_reverse_mode() {
        let json = r#""edifact""#;
        let mode: ReverseMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, ReverseMode::Edifact);

        let json = r#""mig-tree""#;
        let mode: ReverseMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, ReverseMode::MigTree);
    }
}
