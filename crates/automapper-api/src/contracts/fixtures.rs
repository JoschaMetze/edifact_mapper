//! Fixture file browser types.

use serde::{Deserialize, Serialize};

/// A single fixture entry (a pair of `.edi` / `.bo.json` files sharing the same base name).
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FixtureEntry {
    /// Base file name without extension (e.g., `55001_UTILMD_S2.1_ALEXANDE121980`).
    pub name: String,

    /// PID extracted from the file name prefix (e.g., `55001`).
    pub pid: String,

    /// Whether a `.edi` file exists for this fixture.
    pub has_edi: bool,

    /// Whether a `.bo.json` file exists for this fixture.
    pub has_bo4e: bool,
}

/// Response for `GET /api/v1/fixtures`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FixtureListResponse {
    pub fixtures: Vec<FixtureEntry>,
}

/// Info about a single format version within a message type.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FormatVersionInfo {
    /// Format version identifier (e.g., `FV2504`).
    pub format_version: String,

    /// Number of unique fixture base names in this format version directory.
    pub fixture_count: usize,
}

/// A message type with its available format versions.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FixtureCatalogEntry {
    /// EDIFACT message type (e.g., `UTILMD`, `APERAK`, `MSCONS`).
    pub message_type: String,

    /// Available format versions, sorted ascending.
    pub format_versions: Vec<FormatVersionInfo>,
}

/// Response for `GET /api/v1/fixtures/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FixtureCatalogResponse {
    pub message_types: Vec<FixtureCatalogEntry>,
}
