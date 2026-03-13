//! Fixture file listing and serving endpoints.

use std::collections::BTreeMap;
use std::path::PathBuf;

use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::contracts::fixtures::{
    FixtureCatalogEntry, FixtureCatalogResponse, FixtureEntry, FixtureListResponse,
    FormatVersionInfo,
};
use crate::error::ApiError;
use crate::state::AppState;

/// Base directory for fixture files (git submodule).
const FIXTURES_DIR: &str = "example_market_communication_bo4e_transactions";

/// Build fixture routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/fixtures/catalog", get(list_fixture_catalog))
        .route("/fixtures", get(list_fixtures))
        .route(
            "/fixtures/{message_type}/{format_version}/{name}",
            get(get_fixture),
        )
}

/// `GET /api/v1/fixtures/catalog`
///
/// Scans the fixture base directory and returns all available message types
/// with their format versions and fixture counts.
#[utoipa::path(
    get,
    path = "/api/v1/fixtures/catalog",
    responses(
        (status = 200, description = "Catalog of available message types and format versions", body = FixtureCatalogResponse),
    ),
    tag = "v1"
)]
pub(crate) async fn list_fixture_catalog() -> Result<Json<FixtureCatalogResponse>, ApiError> {
    let base = PathBuf::from(FIXTURES_DIR);

    if !base.is_dir() {
        return Ok(Json(FixtureCatalogResponse {
            message_types: vec![],
        }));
    }

    let mut entries: Vec<FixtureCatalogEntry> = Vec::new();

    let mut top_dirs: Vec<_> = std::fs::read_dir(&base)
        .map_err(|e| ApiError::Internal {
            message: format!("failed to read fixture base dir: {e}"),
        })?
        .flatten()
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .collect();
    top_dirs.sort_by_key(|e| e.file_name());

    for msg_dir in top_dirs {
        let message_type = msg_dir.file_name().to_string_lossy().to_string();

        let mut format_versions: Vec<FormatVersionInfo> = Vec::new();

        let Ok(sub_dirs) = std::fs::read_dir(msg_dir.path()) else {
            continue;
        };

        let mut fv_dirs: Vec<_> = sub_dirs
            .flatten()
            .filter(|e| {
                e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) && {
                    let name = e.file_name().to_string_lossy().to_string();
                    // Match FV + 4 digits (e.g., FV2504, FV2510)
                    name.len() >= 6
                        && name.starts_with("FV")
                        && name[2..6].chars().all(|c| c.is_ascii_digit())
                }
            })
            .collect();
        fv_dirs.sort_by_key(|e| e.file_name());

        for fv_dir in fv_dirs {
            let format_version = fv_dir.file_name().to_string_lossy().to_string();

            // Only show FV2504 and newer (older versions are unsupported)
            if format_version.as_str() < "FV2504" {
                continue;
            }

            // Count unique fixture base names (files ending in .edi or .bo.json).
            let fixture_count = count_fixture_bases(&fv_dir.path());
            if fixture_count == 0 {
                continue;
            }

            format_versions.push(FormatVersionInfo {
                format_version,
                fixture_count,
            });
        }

        if !format_versions.is_empty() {
            entries.push(FixtureCatalogEntry {
                message_type,
                format_versions,
            });
        }
    }

    Ok(Json(FixtureCatalogResponse {
        message_types: entries,
    }))
}

/// Count unique fixture base names in a directory (files with `.edi` or `.bo.json` extension).
fn count_fixture_bases(dir: &std::path::Path) -> usize {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return 0;
    };

    let mut bases = std::collections::BTreeSet::new();
    for entry in read_dir.flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();
        if let Some(base) = fname.strip_suffix(".edi") {
            bases.insert(base.to_string());
        } else if let Some(base) = fname.strip_suffix(".bo.json") {
            bases.insert(base.to_string());
        }
    }
    bases.len()
}

/// Query parameters for listing fixtures.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct ListFixturesQuery {
    pub message_type: String,
    pub format_version: String,
}

/// `GET /api/v1/fixtures?message_type=UTILMD&format_version=FV2504`
///
/// Scans the fixture directory and returns grouped entries.
#[utoipa::path(
    get,
    path = "/api/v1/fixtures",
    params(ListFixturesQuery),
    responses(
        (status = 200, description = "List of fixture entries", body = FixtureListResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "No fixtures found"),
    ),
    tag = "v1"
)]
pub(crate) async fn list_fixtures(
    Query(params): Query<ListFixturesQuery>,
) -> Result<Json<FixtureListResponse>, ApiError> {
    // Reject path traversal
    if params.message_type.contains("..") || params.format_version.contains("..") {
        return Err(ApiError::BadRequest {
            message: "invalid path component".to_string(),
        });
    }

    let dir = PathBuf::from(FIXTURES_DIR)
        .join(&params.message_type)
        .join(&params.format_version);

    if !dir.is_dir() {
        return Err(ApiError::NotFound {
            message: format!(
                "no fixtures for {}/{}",
                params.message_type, params.format_version
            ),
        });
    }

    // Scan the directory and group by base name.
    // Base name = filename with `.edi` or `.bo.json` stripped.
    let mut entries: BTreeMap<String, (bool, bool)> = BTreeMap::new();

    let read_dir = std::fs::read_dir(&dir).map_err(|e| ApiError::Internal {
        message: format!("failed to read fixture dir: {e}"),
    })?;

    for entry in read_dir.flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();

        if let Some(base) = fname.strip_suffix(".edi") {
            entries.entry(base.to_string()).or_default().0 = true;
        } else if let Some(base) = fname.strip_suffix(".bo.json") {
            entries.entry(base.to_string()).or_default().1 = true;
        }
    }

    let fixtures: Vec<FixtureEntry> = entries
        .into_iter()
        .map(|(name, (has_edi, has_bo4e))| {
            // Extract PID from the filename prefix (digits before the first underscore).
            let pid = name.split('_').next().unwrap_or("").to_string();
            FixtureEntry {
                name,
                pid,
                has_edi,
                has_bo4e,
            }
        })
        .collect();

    Ok(Json(FixtureListResponse { fixtures }))
}

/// Query parameters for getting a specific fixture file.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct GetFixtureQuery {
    /// File type: `edi` or `bo4e`.
    #[serde(rename = "type")]
    pub file_type: String,
}

/// `GET /api/v1/fixtures/{message_type}/{format_version}/{name}?type=edi|bo4e`
///
/// Serves the raw content of a fixture file.
#[utoipa::path(
    get,
    path = "/api/v1/fixtures/{message_type}/{format_version}/{name}",
    params(
        ("message_type" = String, Path, description = "EDIFACT message type"),
        ("format_version" = String, Path, description = "Format version (e.g. FV2504)"),
        ("name" = String, Path, description = "Fixture base name"),
        GetFixtureQuery,
    ),
    responses(
        (status = 200, description = "Fixture file content", body = String),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Fixture not found"),
    ),
    tag = "v1"
)]
pub(crate) async fn get_fixture(
    Path((message_type, format_version, name)): Path<(String, String, String)>,
    Query(params): Query<GetFixtureQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Reject path traversal in all components.
    if message_type.contains("..")
        || format_version.contains("..")
        || name.contains("..")
        || name.contains('/')
        || name.contains('\\')
    {
        return Err(ApiError::BadRequest {
            message: "invalid path component".to_string(),
        });
    }

    let extension = match params.file_type.as_str() {
        "edi" => "edi",
        "bo4e" => "bo.json",
        other => {
            return Err(ApiError::BadRequest {
                message: format!("invalid type '{other}', expected 'edi' or 'bo4e'"),
            });
        }
    };

    let path = PathBuf::from(FIXTURES_DIR)
        .join(&message_type)
        .join(&format_version)
        .join(format!("{name}.{extension}"));

    if !path.is_file() {
        return Err(ApiError::NotFound {
            message: format!("fixture not found: {name}.{extension}"),
        });
    }

    let content = std::fs::read_to_string(&path).map_err(|e| ApiError::Internal {
        message: format!("failed to read fixture: {e}"),
    })?;

    Ok(content)
}
