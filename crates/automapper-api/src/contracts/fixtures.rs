//! Fixture file browser types.

use serde::{Deserialize, Serialize};

/// A single fixture entry (a pair of `.edi` / `.bo.json` files sharing the same base name).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureListResponse {
    pub fixtures: Vec<FixtureEntry>,
}
