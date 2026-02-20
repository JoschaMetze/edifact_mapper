//! V2 conversion request/response types supporting dual API modes.

use serde::{Deserialize, Serialize};

/// Conversion mode for the v2 API.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConvertMode {
    /// Return the MIG-assembled tree as JSON.
    MigTree,
    /// Run MIG assembly + TOML mapping to produce BO4E JSON.
    Bo4e,
    /// Use the legacy automapper-core pipeline.
    Legacy,
}

/// Request body for `POST /api/v2/convert`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertV2Request {
    /// The raw EDIFACT content to convert.
    pub input: String,

    /// Conversion mode: "mig-tree", "bo4e", or "legacy".
    pub mode: ConvertMode,

    /// Format version (e.g., "FV2504", "FV2510").
    pub format_version: String,
}

/// Response body for `POST /api/v2/convert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertV2Response {
    /// The mode used for conversion.
    pub mode: String,

    /// The converted result (tree JSON, BO4E JSON, or legacy result).
    pub result: serde_json::Value,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,
}
