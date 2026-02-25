//! Request/response types for BO4E validation endpoint.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::reverse_v2::{EnvelopeOverrides, InputLevel};

/// Request body for `POST /api/v2/validate-bo4e`.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateBo4eRequest {
    /// The BO4E JSON to validate. Shape depends on `level`.
    #[schema(value_type = Object)]
    pub input: serde_json::Value,

    /// Which level the input represents: "interchange", "nachricht", or "transaktion".
    #[serde(default)]
    pub level: InputLevel,

    /// Format version (e.g., "FV2504").
    pub format_version: String,

    /// Validation level: "Structure", "Conditions", or "Full". Defaults to "Full".
    #[serde(default)]
    #[schema(value_type = String)]
    pub validation_level: automapper_validation::ValidationLevel,

    /// Optional external condition overrides (condition_name -> bool).
    pub external_conditions: Option<HashMap<String, bool>>,

    /// Optional envelope overrides for missing levels.
    #[serde(default)]
    pub envelope: Option<EnvelopeOverrides>,
}

/// Response body for `POST /api/v2/validate-bo4e`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateBo4eResponse {
    /// The validation report (with optional bo4e_path on issues).
    #[schema(value_type = Object)]
    pub report: serde_json::Value,
    /// Validation duration in milliseconds.
    pub duration_ms: f64,
}
