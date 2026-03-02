//! V2 validation request/response types.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Request body for `POST /api/v2/validate`.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ValidateV2Request {
    /// Raw EDIFACT content to validate.
    pub input: String,
    /// Format version (e.g., "FV2504").
    pub format_version: String,
    /// Validation level: "structure", "conditions", or "full". Defaults to "full".
    #[serde(default = "default_level")]
    pub level: String,
    /// Optional external condition overrides (condition_name -> bool).
    pub external_conditions: Option<HashMap<String, bool>>,
    /// Optional: generate an APERAK/CONTRL response message.
    pub generate_response: Option<ResponseGenerationOptions>,
}

fn default_level() -> String {
    "full".to_string()
}

/// Options for generating an APERAK/CONTRL response message.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ResponseGenerationOptions {
    /// Response message type: "aperak" or "contrl". If omitted, auto-detected from variant.
    pub response_type: Option<String>,
    /// Output format: "bo4e" or "edifact". Defaults to "bo4e".
    pub format: Option<String>,
}

/// Generated response message payload.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GeneratedResponsePayload {
    /// "APERAK" or "CONTRL".
    pub message_type: String,
    /// BO4E JSON of the response message.
    #[schema(value_type = Object)]
    pub bo4e: Option<serde_json::Value>,
    /// EDIFACT string of the response message.
    pub edifact: Option<String>,
}

/// Response body for `POST /api/v2/validate`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ValidateV2Response {
    /// The validation report.
    #[schema(value_type = Object)]
    pub report: serde_json::Value,
    /// Validation duration in milliseconds.
    pub duration_ms: f64,
    /// Generated response message (if requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_message: Option<GeneratedResponsePayload>,
}
