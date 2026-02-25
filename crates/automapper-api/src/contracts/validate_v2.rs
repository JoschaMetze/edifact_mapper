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
}

fn default_level() -> String {
    "full".to_string()
}

/// Response body for `POST /api/v2/validate`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ValidateV2Response {
    /// The validation report.
    #[schema(value_type = Object)]
    pub report: serde_json::Value,
    /// Validation duration in milliseconds.
    pub duration_ms: f64,
}
