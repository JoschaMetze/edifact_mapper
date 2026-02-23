//! V2 reverse conversion request/response types.
//!
//! Accepts BO4E JSON and converts back to EDIFACT.

use serde::{Deserialize, Serialize};

/// Input level for the reverse endpoint.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InputLevel {
    /// Full interchange JSON (nachrichtendaten + nachrichten array).
    Interchange,
    /// Single message JSON (unhReferenz, nachrichtenTyp, stammdaten, transaktionen).
    Nachricht,
    /// Single transaction JSON (stammdaten, transaktionsdaten).
    Transaktion,
}

/// Output mode for the reverse endpoint.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeOverrides {
    pub absender_code: Option<String>,
    pub empfaenger_code: Option<String>,
    pub nachrichten_typ: Option<String>,
}

/// Request body for `POST /api/v2/reverse`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseV2Request {
    /// The BO4E JSON to convert back to EDIFACT.
    /// Shape depends on `level`.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseV2Response {
    /// The mode used for conversion.
    pub mode: String,

    /// The result: EDIFACT string or MIG tree JSON.
    pub result: serde_json::Value,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,
}
