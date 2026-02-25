//! V2 conversion request/response types for MIG-driven conversion modes.

use serde::{Deserialize, Serialize};

/// Conversion mode for the v2 API.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ConvertMode {
    /// Return the MIG-assembled tree as JSON.
    MigTree,
    /// Run MIG assembly + TOML mapping to produce BO4E JSON.
    Bo4e,
}

/// Request body for `POST /api/v2/convert`.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ConvertV2Request {
    /// The raw EDIFACT content to convert.
    pub input: String,

    /// Conversion mode: "mig-tree" or "bo4e".
    pub mode: ConvertMode,

    /// Format version (e.g., "FV2504", "FV2510").
    pub format_version: String,
}

/// Query parameters for `POST /api/v2/convert`.
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct ConvertV2Query {
    /// When `false`, code fields are emitted as plain strings instead of
    /// `{"code": "...", "meaning": "..."}` objects. Defaults to `true`.
    pub enrich_codes: Option<bool>,

    /// Run validation and include report in response. Defaults to `false`.
    pub validate: Option<bool>,
}

/// Response body for `POST /api/v2/convert`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConvertV2Response {
    /// The mode used for conversion.
    pub mode: String,

    /// The converted result (tree JSON or BO4E JSON).
    #[schema(value_type = Object)]
    pub result: serde_json::Value,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,

    /// Validation report (present when `?validate=true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub validation: Option<serde_json::Value>,
}
