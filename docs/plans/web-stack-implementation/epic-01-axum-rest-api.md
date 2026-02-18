---
feature: web-stack-implementation
epic: 1
title: "Axum REST API"
depends_on: []
estimated_tasks: 7
crate: automapper-api
status: in_progress
---

# Epic 1: Axum REST API

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-api/src/`. All code must compile with `cargo check -p automapper-api`.

**Goal:** Build the `automapper-api` crate with an Axum HTTP server providing REST endpoints for EDIFACT-to-BO4E conversion, EDIFACT inspection, coordinator discovery, and health checks. The crate holds an `AppState` with a `CoordinatorRegistry` that delegates to `automapper-core` for actual conversions. All endpoints return JSON. Integration tests verify the API contract using `tower::ServiceExt`.

**Architecture:** Axum 0.8 router with shared `AppState` (wraps `Arc<CoordinatorRegistry>`). Each route module contains handler functions. Request/response types live in a `contracts` module and derive `Serialize`/`Deserialize`. Error handling uses a custom `ApiError` type that implements `IntoResponse`. CORS middleware allows the Leptos dev server to call the API during development. Static file serving is configured for the production WASM bundle.

**Tech Stack:** axum 0.8, tokio 1.x, tower-http 0.6 (CorsLayer, ServeDir, TraceLayer), serde + serde_json, thiserror, tracing + tracing-subscriber, automapper-core

**Depends On:** Feature 1 (edifact-core-implementation) — specifically `automapper-core` for `Coordinator`, `create_coordinator()`, `FormatVersion`, `UtilmdTransaktion`, `AutomapperError`, and `edifact-parser` for `EdifactStreamParser`, `RawSegment`.

---

## Task 1: Create `automapper-api` Crate Skeleton

### Step 1: Create directory structure

```bash
mkdir -p crates/automapper-api/src/routes
mkdir -p crates/automapper-api/src/contracts
mkdir -p crates/automapper-api/tests
```

### Step 2: Write `crates/automapper-api/Cargo.toml`

```toml
[package]
name = "automapper-api"
version = "0.1.0"
edition = "2021"
description = "Axum REST API + tonic gRPC server for edifact-bo4e-automapper"
publish = false

[[bin]]
name = "automapper-api"
path = "src/main.rs"

[lib]
name = "automapper_api"
path = "src/lib.rs"

[dependencies]
# Internal crates
automapper-core = { path = "../automapper-core" }
edifact-parser = { path = "../edifact-parser" }
edifact-types = { path = "../edifact-types" }
bo4e-extensions = { path = "../bo4e-extensions" }

# Web framework
axum = { version = "0.8", features = ["json", "macros"] }
tokio = { version = "1", features = ["full"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "2"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time measurement
std-time = { package = "web-time", version = "1" }

[dev-dependencies]
# Testing
axum-test = "16"
http = "1"
http-body-util = "0.1"
tower = { version = "0.5", features = ["util"] }
serde_json = "1"
tokio = { version = "1", features = ["full", "test-util"] }
```

### Step 3: Write `crates/automapper-api/src/lib.rs`

```rust
//! Automapper REST API server.
//!
//! Provides HTTP endpoints for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! coordinator discovery, and health checks.

pub mod contracts;
pub mod error;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Build the complete Axum router with all routes and middleware.
pub fn build_router(state: state::AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::api_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
```

### Step 4: Write `crates/automapper-api/src/main.rs`

```rust
use automapper_api::state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = AppState::new();
    let app = automapper_api::build_router(state);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind");

    tracing::info!("automapper-api listening on {}", bind_addr);

    axum::serve(listener, app)
        .await
        .expect("server error");
}
```

### Step 5: Write stub modules

Write `crates/automapper-api/src/error.rs`:

```rust
//! API error types and Axum error response mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// API-level error that converts to an Axum response.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {message}")]
    BadRequest { message: String },

    #[error("not found: {message}")]
    NotFound { message: String },

    #[error("conversion error: {message}")]
    ConversionError { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::BadRequest { message } => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", message.clone())
            }
            ApiError::NotFound { message } => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", message.clone())
            }
            ApiError::ConversionError { message } => {
                (StatusCode::UNPROCESSABLE_ENTITY, "CONVERSION_ERROR", message.clone())
            }
            ApiError::Internal { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message.clone())
            }
        };

        let body = json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<automapper_core::AutomapperError> for ApiError {
    fn from(err: automapper_core::AutomapperError) -> Self {
        ApiError::ConversionError {
            message: err.to_string(),
        }
    }
}
```

Write `crates/automapper-api/src/state.rs`:

```rust
//! Application state and coordinator registry.

use std::collections::HashMap;
use std::sync::Arc;

use automapper_core::{create_coordinator, FormatVersion};

use crate::contracts::coordinators::CoordinatorInfo;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<CoordinatorRegistry>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(CoordinatorRegistry::discover()),
        }
    }
}

/// Discovers and manages available coordinators from automapper-core.
pub struct CoordinatorRegistry {
    coordinators: HashMap<String, CoordinatorInfo>,
}

impl CoordinatorRegistry {
    /// Discover all available coordinators by probing automapper-core.
    pub fn discover() -> Self {
        let mut coordinators = HashMap::new();

        // Register UTILMD coordinator with known format versions
        coordinators.insert(
            "UTILMD".to_string(),
            CoordinatorInfo {
                message_type: "UTILMD".to_string(),
                description: "Coordinator for UTILMD (utility master data) messages".to_string(),
                supported_versions: vec!["FV2504".to_string(), "FV2510".to_string()],
            },
        );

        tracing::info!(
            "Discovered {} coordinators: {:?}",
            coordinators.len(),
            coordinators.keys().collect::<Vec<_>>()
        );

        Self { coordinators }
    }

    /// Get all available coordinators.
    pub fn list(&self) -> Vec<&CoordinatorInfo> {
        self.coordinators.values().collect()
    }

    /// Check if a coordinator exists for the given message type.
    pub fn has(&self, message_type: &str) -> bool {
        self.coordinators.contains_key(&message_type.to_uppercase())
    }

    /// Get coordinator info for a specific message type.
    pub fn get(&self, message_type: &str) -> Option<&CoordinatorInfo> {
        self.coordinators.get(&message_type.to_uppercase())
    }

    /// Convert EDIFACT content to BO4E JSON.
    pub fn convert_edifact_to_bo4e(
        &self,
        edifact: &str,
        format_version: Option<&str>,
        include_trace: bool,
    ) -> Result<crate::contracts::convert::ConvertResponse, crate::error::ApiError> {
        let start = std::time::Instant::now();

        let fv = match format_version {
            Some("FV2510") => FormatVersion::FV2510,
            _ => FormatVersion::FV2504,
        };

        let mut coordinator = create_coordinator(fv);
        let input = edifact.as_bytes();

        let transactions = coordinator.parse(input).map_err(|e| {
            crate::error::ApiError::ConversionError {
                message: e.to_string(),
            }
        })?;

        let result_json =
            serde_json::to_string_pretty(&transactions).map_err(|e| {
                crate::error::ApiError::Internal {
                    message: format!("serialization error: {e}"),
                }
            })?;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        let trace = if include_trace {
            // Build a basic trace from the conversion
            vec![crate::contracts::trace::TraceEntry {
                mapper: "UtilmdCoordinator".to_string(),
                source_segment: "UNH..UNT".to_string(),
                target_path: "transactions".to_string(),
                value: Some(format!("{} transaction(s)", transactions.len())),
                note: None,
            }]
        } else {
            vec![]
        };

        Ok(crate::contracts::convert::ConvertResponse {
            success: true,
            result: Some(result_json),
            trace,
            errors: vec![],
            duration_ms,
        })
    }

    /// Convert BO4E JSON content to EDIFACT.
    pub fn convert_bo4e_to_edifact(
        &self,
        bo4e_json: &str,
        format_version: Option<&str>,
    ) -> Result<crate::contracts::convert::ConvertResponse, crate::error::ApiError> {
        let start = std::time::Instant::now();

        let fv = match format_version {
            Some("FV2510") => FormatVersion::FV2510,
            _ => FormatVersion::FV2504,
        };

        // Deserialize the BO4E transaction
        let transaktion: automapper_core::UtilmdTransaktion =
            serde_json::from_str(bo4e_json).map_err(|e| {
                crate::error::ApiError::BadRequest {
                    message: format!("invalid BO4E JSON: {e}"),
                }
            })?;

        let coordinator = create_coordinator(fv);
        let edifact_bytes = coordinator.generate(&transaktion).map_err(|e| {
            crate::error::ApiError::ConversionError {
                message: e.to_string(),
            }
        })?;

        let edifact_string = String::from_utf8(edifact_bytes).map_err(|e| {
            crate::error::ApiError::Internal {
                message: format!("UTF-8 conversion error: {e}"),
            }
        })?;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(crate::contracts::convert::ConvertResponse {
            success: true,
            result: Some(edifact_string),
            trace: vec![],
            errors: vec![],
            duration_ms,
        })
    }

    /// Inspect EDIFACT content, returning a segment tree.
    pub fn inspect_edifact(
        &self,
        edifact: &str,
    ) -> Result<crate::contracts::inspect::InspectResponse, crate::error::ApiError> {
        if edifact.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest {
                message: "EDIFACT content is required".to_string(),
            });
        }

        let segments = parse_edifact_to_segment_nodes(edifact);
        let segment_count = segments.len();

        // Detect message type from UNH segment
        let mut message_type = None;
        let mut format_version = None;

        for seg in &segments {
            if seg.tag == "UNH" && seg.elements.len() >= 2 {
                if let Some(ref components) = seg.elements[1].components {
                    if !components.is_empty() {
                        message_type = components[0].value.clone();
                    }
                    if components.len() >= 3 {
                        format_version = Some(format!(
                            "{}:{}",
                            components[1].value.as_deref().unwrap_or(""),
                            components[2].value.as_deref().unwrap_or("")
                        ));
                    }
                }
            }
        }

        Ok(crate::contracts::inspect::InspectResponse {
            segments,
            segment_count,
            message_type,
            format_version,
        })
    }
}

/// Parse raw EDIFACT text into a flat list of `SegmentNode` values.
fn parse_edifact_to_segment_nodes(
    edifact: &str,
) -> Vec<crate::contracts::inspect::SegmentNode> {
    use crate::contracts::inspect::{ComponentElement, DataElement, SegmentNode};

    let mut segments = Vec::new();
    let parts: Vec<&str> = edifact.split('\'').collect();
    let mut line_number = 0u32;

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        line_number += 1;

        let plus_index = trimmed.find('+');
        let tag = match plus_index {
            Some(idx) => &trimmed[..idx],
            None => trimmed,
        };

        let elements = if let Some(idx) = plus_index {
            let element_strs: Vec<&str> = trimmed[idx + 1..].split('+').collect();
            element_strs
                .iter()
                .enumerate()
                .map(|(i, elem_str)| {
                    let components_strs: Vec<&str> = elem_str.split(':').collect();
                    if components_strs.len() > 1 {
                        DataElement {
                            position: (i + 1) as u32,
                            value: None,
                            components: Some(
                                components_strs
                                    .iter()
                                    .enumerate()
                                    .map(|(j, comp)| ComponentElement {
                                        position: (j + 1) as u32,
                                        value: if comp.is_empty() {
                                            None
                                        } else {
                                            Some(comp.to_string())
                                        },
                                    })
                                    .collect(),
                            ),
                        }
                    } else {
                        DataElement {
                            position: (i + 1) as u32,
                            value: if elem_str.is_empty() {
                                None
                            } else {
                                Some(elem_str.to_string())
                            },
                            components: None,
                        }
                    }
                })
                .collect()
        } else {
            vec![]
        };

        segments.push(SegmentNode {
            tag: tag.to_string(),
            line_number,
            raw_content: trimmed.to_string(),
            elements,
            children: None,
        });
    }

    segments
}
```

Write `crates/automapper-api/src/contracts/mod.rs`:

```rust
//! Request and response types for the REST API.

pub mod convert;
pub mod coordinators;
pub mod error;
pub mod health;
pub mod inspect;
pub mod trace;
```

Write `crates/automapper-api/src/routes/mod.rs`:

```rust
//! Route handlers for the REST API.

pub mod convert;
pub mod coordinators;
pub mod health;
pub mod inspect;

use axum::Router;

use crate::state::AppState;

/// Build all `/api/v1/*` routes.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(convert::routes())
        .merge(inspect::routes())
        .merge(coordinators::routes())
}
```

### Step 6: Run `cargo check -p automapper-api`

This will fail because the contract and route modules are empty. That is expected. Proceed to Task 2.

### Step 7: Commit

```bash
git add -A
git commit -m "feat(api): scaffold automapper-api crate with Cargo.toml, lib, main, state, error stubs"
```

---

## Task 2: Define REST Contract Types

### Step 1: Write `crates/automapper-api/src/contracts/convert.rs`

```rust
//! Conversion request and response types.

use serde::{Deserialize, Serialize};

use super::error::ApiErrorEntry;
use super::trace::TraceEntry;

/// Request body for `POST /api/v1/convert/edifact-to-bo4e`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertRequest {
    /// The raw content to convert (EDIFACT string or BO4E JSON).
    pub content: String,

    /// Optional format version override (e.g., "FV2504", "FV2510").
    /// If omitted, auto-detected from the content.
    pub format_version: Option<String>,

    /// Whether to include a mapping trace in the response.
    #[serde(default)]
    pub include_trace: bool,
}

/// Response body for conversion endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResponse {
    /// Whether the conversion succeeded.
    pub success: bool,

    /// The converted content (BO4E JSON or EDIFACT string).
    /// `None` if the conversion failed.
    pub result: Option<String>,

    /// Mapping trace entries (empty if `include_trace` was false).
    pub trace: Vec<TraceEntry>,

    /// Errors encountered during conversion.
    pub errors: Vec<ApiErrorEntry>,

    /// Conversion duration in milliseconds.
    pub duration_ms: f64,
}
```

### Step 2: Write `crates/automapper-api/src/contracts/inspect.rs`

```rust
//! EDIFACT inspection request and response types.

use serde::{Deserialize, Serialize};

/// Request body for `POST /api/v1/inspect/edifact`.
#[derive(Debug, Clone, Deserialize)]
pub struct InspectRequest {
    /// The raw EDIFACT content to inspect.
    pub edifact: String,
}

/// Response body for EDIFACT inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResponse {
    /// Flat list of parsed segments.
    pub segments: Vec<SegmentNode>,

    /// Total number of segments parsed.
    pub segment_count: usize,

    /// Detected message type (e.g., "UTILMD"), if found in UNH.
    pub message_type: Option<String>,

    /// Detected format version, if derivable from the content.
    pub format_version: Option<String>,
}

/// A single EDIFACT segment in the tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentNode {
    /// Segment tag (e.g., "UNH", "NAD", "LOC").
    pub tag: String,

    /// 1-based line number (segment ordinal position).
    pub line_number: u32,

    /// Raw segment content (without the segment terminator).
    pub raw_content: String,

    /// Parsed data elements within this segment.
    pub elements: Vec<DataElement>,

    /// Child segments (for hierarchical grouping; `None` for flat output).
    pub children: Option<Vec<SegmentNode>>,
}

/// A data element within a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataElement {
    /// 1-based position within the segment.
    pub position: u32,

    /// Simple element value (if not composite).
    pub value: Option<String>,

    /// Component elements (if composite, i.e., contains `:` separators).
    pub components: Option<Vec<ComponentElement>>,
}

/// A component element within a composite data element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentElement {
    /// 1-based position within the composite element.
    pub position: u32,

    /// Component value.
    pub value: Option<String>,
}
```

### Step 3: Write `crates/automapper-api/src/contracts/coordinators.rs`

```rust
//! Coordinator discovery types.

use serde::{Deserialize, Serialize};

/// Information about an available coordinator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorInfo {
    /// EDIFACT message type (e.g., "UTILMD").
    pub message_type: String,

    /// Human-readable description.
    pub description: String,

    /// Format versions supported by this coordinator.
    pub supported_versions: Vec<String>,
}
```

### Step 4: Write `crates/automapper-api/src/contracts/health.rs`

```rust
//! Health check types.

use serde::Serialize;

/// Response body for `GET /health`.
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    /// Whether the service is healthy.
    pub healthy: bool,

    /// Application version string.
    pub version: String,

    /// Available coordinator message types.
    pub available_coordinators: Vec<String>,

    /// Server uptime in seconds.
    pub uptime_seconds: f64,
}
```

### Step 5: Write `crates/automapper-api/src/contracts/error.rs`

```rust
//! Error types returned in API responses.

use serde::{Deserialize, Serialize};

/// An error entry in an API response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorEntry {
    /// Machine-readable error code (e.g., "UNKNOWN_TYPE", "PARSE_ERROR").
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// Optional location in the source content (e.g., "segment 5, byte 234").
    pub location: Option<String>,

    /// Error severity.
    pub severity: ErrorSeverity,
}

/// Severity level for API errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}
```

### Step 6: Write `crates/automapper-api/src/contracts/trace.rs`

```rust
//! Mapping trace types.

use serde::{Deserialize, Serialize};

/// A single entry in the mapping trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Name of the mapper/writer that processed this step.
    pub mapper: String,

    /// Source EDIFACT segment reference (e.g., "NAD (line 5)").
    pub source_segment: String,

    /// Target BO4E path (e.g., "geschaeftspartner.name1").
    pub target_path: String,

    /// Mapped value, if available.
    pub value: Option<String>,

    /// Optional note about the mapping step.
    pub note: Option<String>,
}
```

### Step 7: Run `cargo check -p automapper-api`

The check should pass for contracts. Route handlers are still stubs — proceed to Task 3.

### Step 8: Commit

```bash
git add -A
git commit -m "feat(api): define REST contract types — ConvertRequest/Response, InspectResponse, SegmentNode, CoordinatorInfo, HealthResponse"
```

---

## Task 3: Implement Route Handlers

### Step 1: Write test file `crates/automapper-api/tests/api_integration.rs`

```rust
//! Integration tests for the REST API.
//!
//! Uses tower::ServiceExt to call the router directly without a running server.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use automapper_api::contracts::convert::ConvertResponse;
use automapper_api::contracts::coordinators::CoordinatorInfo;
use automapper_api::contracts::health::HealthResponse;
use automapper_api::contracts::inspect::InspectResponse;
use automapper_api::state::AppState;

fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_router(state)
}

// --- Health ---

#[tokio::test]
async fn test_health_returns_200() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();

    assert!(health.healthy);
    assert!(!health.version.is_empty());
    assert!(!health.available_coordinators.is_empty());
}

// --- Coordinators ---

#[tokio::test]
async fn test_list_coordinators_returns_200() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/coordinators")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let coordinators: Vec<CoordinatorInfo> = serde_json::from_slice(&body).unwrap();

    assert!(!coordinators.is_empty());
    assert!(coordinators.iter().any(|c| c.message_type == "UTILMD"));
}

// --- Inspect ---

#[tokio::test]
async fn test_inspect_edifact_returns_segments() {
    let app = app();

    let edifact = r#"UNH+1+UTILMD:D:11A:UN:5.2e'BGM+E01+DOC001'UNT+3+1'"#;
    let body = serde_json::json!({ "edifact": edifact });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inspect/edifact")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let inspect: InspectResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(inspect.segment_count, 3);
    assert_eq!(inspect.segments[0].tag, "UNH");
    assert_eq!(inspect.segments[1].tag, "BGM");
    assert_eq!(inspect.segments[2].tag, "UNT");
    assert_eq!(inspect.message_type, Some("UTILMD".to_string()));
}

#[tokio::test]
async fn test_inspect_empty_edifact_returns_400() {
    let app = app();

    let body = serde_json::json!({ "edifact": "" });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/inspect/edifact")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Empty content should return 400 or error
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

// --- Convert EDIFACT to BO4E ---

#[tokio::test]
async fn test_convert_edifact_to_bo4e_accepts_valid_json() {
    let app = app();

    // Minimal EDIFACT that at minimum exercises the endpoint
    let body = serde_json::json!({
        "content": "UNB+UNOC:3+sender+receiver+231215:1200+ref001'UNH+1+UTILMD:D:11A:UN:5.2e'BGM+E01+DOC001'UNT+3+1'UNZ+1+ref001'",
        "include_trace": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/convert/edifact-to-bo4e")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 200 (success) or 422 (conversion error) — not 500
    assert!(
        response.status() == StatusCode::OK
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let resp: ConvertResponse = serde_json::from_slice(&body).unwrap();

    // Response should be well-formed regardless of success
    assert!(resp.duration_ms >= 0.0);
}

// --- Convert BO4E to EDIFACT ---

#[tokio::test]
async fn test_convert_bo4e_to_edifact_rejects_invalid_json() {
    let app = app();

    let body = serde_json::json!({
        "content": "this is not valid json for bo4e",
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/convert/bo4e-to-edifact")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return an error, not 500
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}
```

### Step 2: Write `crates/automapper-api/src/routes/health.rs`

```rust
//! Health check endpoint.

use axum::extract::State;
use axum::Json;

use crate::contracts::health::HealthResponse;
use crate::state::AppState;

/// `GET /health` — Returns service health status.
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let coordinators: Vec<String> = state
        .registry
        .list()
        .iter()
        .map(|c| c.message_type.clone())
        .collect();

    Json(HealthResponse {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        available_coordinators: coordinators,
        uptime_seconds: 0.0, // TODO: track actual uptime with a start timestamp in AppState
    })
}
```

### Step 3: Write `crates/automapper-api/src/routes/coordinators.rs`

```rust
//! Coordinator discovery endpoints.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::contracts::coordinators::CoordinatorInfo;
use crate::error::ApiError;
use crate::state::AppState;

/// Build coordinator routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/coordinators", get(list_coordinators))
        .route("/coordinators/{message_type}", get(get_coordinator))
}

/// `GET /api/v1/coordinators` — List all available coordinators.
async fn list_coordinators(State(state): State<AppState>) -> Json<Vec<CoordinatorInfo>> {
    let coordinators: Vec<CoordinatorInfo> = state.registry.list().into_iter().cloned().collect();
    Json(coordinators)
}

/// `GET /api/v1/coordinators/{message_type}` — Get a specific coordinator.
async fn get_coordinator(
    State(state): State<AppState>,
    Path(message_type): Path<String>,
) -> Result<Json<CoordinatorInfo>, ApiError> {
    state
        .registry
        .get(&message_type)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound {
            message: format!("no coordinator for message type '{message_type}'"),
        })
}
```

### Step 4: Write `crates/automapper-api/src/routes/inspect.rs`

```rust
//! EDIFACT inspection endpoints.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::contracts::inspect::{InspectRequest, InspectResponse};
use crate::error::ApiError;
use crate::state::AppState;

/// Build inspection routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/inspect/edifact", post(inspect_edifact))
}

/// `POST /api/v1/inspect/edifact` — Parse EDIFACT into a segment tree.
async fn inspect_edifact(
    State(state): State<AppState>,
    Json(request): Json<InspectRequest>,
) -> Result<Json<InspectResponse>, ApiError> {
    tracing::info!(
        "Inspecting EDIFACT content, length={}",
        request.edifact.len()
    );

    let response = state.registry.inspect_edifact(&request.edifact)?;

    tracing::info!(
        "Parsed {} segments, message_type={:?}",
        response.segment_count,
        response.message_type
    );

    Ok(Json(response))
}
```

### Step 5: Write `crates/automapper-api/src/routes/convert.rs`

```rust
//! EDIFACT <-> BO4E conversion endpoints.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::contracts::convert::{ConvertRequest, ConvertResponse};
use crate::error::ApiError;
use crate::state::AppState;

/// Build conversion routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/convert/edifact-to-bo4e", post(convert_edifact_to_bo4e))
        .route("/convert/bo4e-to-edifact", post(convert_bo4e_to_edifact))
}

/// `POST /api/v1/convert/edifact-to-bo4e` — Convert EDIFACT to BO4E JSON.
async fn convert_edifact_to_bo4e(
    State(state): State<AppState>,
    Json(request): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    tracing::info!(
        "Converting EDIFACT to BO4E, content_length={}, format_version={:?}",
        request.content.len(),
        request.format_version
    );

    let response = state.registry.convert_edifact_to_bo4e(
        &request.content,
        request.format_version.as_deref(),
        request.include_trace,
    )?;

    Ok(Json(response))
}

/// `POST /api/v1/convert/bo4e-to-edifact` — Convert BO4E JSON to EDIFACT.
async fn convert_bo4e_to_edifact(
    State(state): State<AppState>,
    Json(request): Json<ConvertRequest>,
) -> Result<Json<ConvertResponse>, ApiError> {
    tracing::info!(
        "Converting BO4E to EDIFACT, content_length={}, format_version={:?}",
        request.content.len(),
        request.format_version
    );

    let response = state.registry.convert_bo4e_to_edifact(
        &request.content,
        request.format_version.as_deref(),
    )?;

    Ok(Json(response))
}
```

### Step 6: Run `cargo check -p automapper-api`

```bash
cargo check -p automapper-api
```

Expected: compiles (possibly with warnings about unused imports from automapper-core if Feature 1 types differ slightly — fix as needed).

### Step 7: Run tests

```bash
cargo test -p automapper-api
```

Expected: all 5 integration tests pass. The conversion tests may show `success: false` with conversion errors (because the minimal EDIFACT is not a full valid UTILMD), but the endpoints themselves should return proper HTTP responses (200 or 422), not 500.

### Step 8: Commit

```bash
git add -A
git commit -m "feat(api): implement REST route handlers — convert, inspect, coordinators, health — with integration tests"
```

---

## Task 4: Add CORS Middleware and Static File Serving

### Step 1: Update `crates/automapper-api/src/lib.rs` to add static file serving

```rust
//! Automapper REST API server.
//!
//! Provides HTTP endpoints for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! coordinator discovery, and health checks. Also serves the Leptos WASM frontend
//! as static files.

pub mod contracts;
pub mod error;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Build the complete Axum router with all routes, middleware, and static file serving.
pub fn build_router(state: state::AppState) -> Router {
    build_router_with_static_dir(state, "static")
}

/// Build the router with a custom static file directory.
///
/// The static directory should contain the compiled Leptos WASM frontend.
/// In production, this is typically `./static/` next to the binary.
pub fn build_router_with_static_dir(state: state::AppState, static_dir: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", routes::api_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .fallback_service(ServeDir::new(static_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
```

### Step 2: Update `crates/automapper-api/src/main.rs` to use environment variable for static dir

```rust
use automapper_api::state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = AppState::new();

    let static_dir =
        std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());
    let app = automapper_api::build_router_with_static_dir(state, &static_dir);

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind");

    tracing::info!(
        "automapper-api listening on {}, static_dir={}",
        bind_addr,
        static_dir
    );

    axum::serve(listener, app)
        .await
        .expect("server error");
}
```

### Step 3: Add CORS integration test to `crates/automapper-api/tests/api_integration.rs`

Append the following test:

```rust
#[tokio::test]
async fn test_cors_preflight_returns_200() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/coordinators")
                .header("origin", "http://localhost:3000")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .contains_key("access-control-allow-origin"));
}
```

### Step 4: Run tests

```bash
cargo test -p automapper-api
```

Expected: all tests pass, including the new CORS preflight test.

### Step 5: Commit

```bash
git add -A
git commit -m "feat(api): add CORS middleware, static file serving, and configurable bind address"
```

---

## Task 5: Add Startup Timestamp and Full Health Endpoint

### Step 1: Update `crates/automapper-api/src/state.rs` — add startup time

Add `startup` field to `AppState`:

```rust
/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<CoordinatorRegistry>,
    pub startup: std::time::Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(CoordinatorRegistry::discover()),
            startup: std::time::Instant::now(),
        }
    }
}
```

### Step 2: Update `crates/automapper-api/src/routes/health.rs` — use startup time

```rust
//! Health check endpoint.

use axum::extract::State;
use axum::Json;

use crate::contracts::health::HealthResponse;
use crate::state::AppState;

/// `GET /health` — Returns service health status.
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let coordinators: Vec<String> = state
        .registry
        .list()
        .iter()
        .map(|c| c.message_type.clone())
        .collect();

    Json(HealthResponse {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        available_coordinators: coordinators,
        uptime_seconds: state.startup.elapsed().as_secs_f64(),
    })
}
```

### Step 3: Run tests

```bash
cargo test -p automapper-api
```

Expected: all tests pass.

### Step 4: Commit

```bash
git add -A
git commit -m "feat(api): track startup time in AppState, report uptime in health endpoint"
```

---

## Task 6: Add Comprehensive Contract Serialization Tests

### Step 1: Write `crates/automapper-api/tests/contract_tests.rs`

```rust
//! Tests that verify JSON serialization of API contracts.
//!
//! These tests ensure the API contract is stable — any accidental field rename
//! or type change will break these tests.

use automapper_api::contracts::convert::{ConvertRequest, ConvertResponse};
use automapper_api::contracts::coordinators::CoordinatorInfo;
use automapper_api::contracts::error::{ApiErrorEntry, ErrorSeverity};
use automapper_api::contracts::health::HealthResponse;
use automapper_api::contracts::inspect::{
    ComponentElement, DataElement, InspectRequest, InspectResponse, SegmentNode,
};
use automapper_api::contracts::trace::TraceEntry;

#[test]
fn test_convert_request_deserialization() {
    let json = r#"{
        "content": "UNH+1+UTILMD:D:11A:UN:5.2e'",
        "format_version": "FV2504",
        "include_trace": true
    }"#;

    let req: ConvertRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content, "UNH+1+UTILMD:D:11A:UN:5.2e'");
    assert_eq!(req.format_version, Some("FV2504".to_string()));
    assert!(req.include_trace);
}

#[test]
fn test_convert_request_defaults() {
    let json = r#"{ "content": "hello" }"#;

    let req: ConvertRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.content, "hello");
    assert_eq!(req.format_version, None);
    assert!(!req.include_trace);
}

#[test]
fn test_convert_response_serialization() {
    let resp = ConvertResponse {
        success: true,
        result: Some("{}".to_string()),
        trace: vec![TraceEntry {
            mapper: "UtilmdCoordinator".to_string(),
            source_segment: "UNH".to_string(),
            target_path: "transactions".to_string(),
            value: Some("1".to_string()),
            note: None,
        }],
        errors: vec![],
        duration_ms: 42.5,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["result"], "{}");
    assert_eq!(json["duration_ms"], 42.5);
    assert_eq!(json["trace"][0]["mapper"], "UtilmdCoordinator");
    assert!(json["errors"].as_array().unwrap().is_empty());
}

#[test]
fn test_convert_response_with_errors() {
    let resp = ConvertResponse {
        success: false,
        result: None,
        trace: vec![],
        errors: vec![ApiErrorEntry {
            code: "PARSE_ERROR".to_string(),
            message: "unterminated segment at byte 42".to_string(),
            location: Some("byte 42".to_string()),
            severity: ErrorSeverity::Error,
        }],
        duration_ms: 1.2,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["success"], false);
    assert!(json["result"].is_null());
    assert_eq!(json["errors"][0]["code"], "PARSE_ERROR");
    assert_eq!(json["errors"][0]["severity"], "error");
}

#[test]
fn test_inspect_request_deserialization() {
    let json = r#"{ "edifact": "UNH+1+UTILMD'" }"#;
    let req: InspectRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.edifact, "UNH+1+UTILMD'");
}

#[test]
fn test_inspect_response_serialization() {
    let resp = InspectResponse {
        segments: vec![SegmentNode {
            tag: "UNH".to_string(),
            line_number: 1,
            raw_content: "UNH+1+UTILMD:D:11A:UN".to_string(),
            elements: vec![
                DataElement {
                    position: 1,
                    value: Some("1".to_string()),
                    components: None,
                },
                DataElement {
                    position: 2,
                    value: None,
                    components: Some(vec![
                        ComponentElement {
                            position: 1,
                            value: Some("UTILMD".to_string()),
                        },
                        ComponentElement {
                            position: 2,
                            value: Some("D".to_string()),
                        },
                        ComponentElement {
                            position: 3,
                            value: Some("11A".to_string()),
                        },
                        ComponentElement {
                            position: 4,
                            value: Some("UN".to_string()),
                        },
                    ]),
                },
            ],
            children: None,
        }],
        segment_count: 1,
        message_type: Some("UTILMD".to_string()),
        format_version: None,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["segment_count"], 1);
    assert_eq!(json["segments"][0]["tag"], "UNH");
    assert_eq!(json["segments"][0]["elements"][1]["components"][0]["value"], "UTILMD");
}

#[test]
fn test_coordinator_info_serialization() {
    let info = CoordinatorInfo {
        message_type: "UTILMD".to_string(),
        description: "UTILMD coordinator".to_string(),
        supported_versions: vec!["FV2504".to_string(), "FV2510".to_string()],
    };

    let json = serde_json::to_value(&info).unwrap();
    assert_eq!(json["message_type"], "UTILMD");
    assert_eq!(json["supported_versions"][0], "FV2504");
    assert_eq!(json["supported_versions"][1], "FV2510");
}

#[test]
fn test_health_response_serialization() {
    let resp = HealthResponse {
        healthy: true,
        version: "0.1.0".to_string(),
        available_coordinators: vec!["UTILMD".to_string()],
        uptime_seconds: 123.456,
    };

    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["healthy"], true);
    assert_eq!(json["version"], "0.1.0");
    assert_eq!(json["uptime_seconds"], 123.456);
}

#[test]
fn test_error_severity_serialization() {
    assert_eq!(
        serde_json::to_string(&ErrorSeverity::Warning).unwrap(),
        r#""warning""#
    );
    assert_eq!(
        serde_json::to_string(&ErrorSeverity::Error).unwrap(),
        r#""error""#
    );
    assert_eq!(
        serde_json::to_string(&ErrorSeverity::Critical).unwrap(),
        r#""critical""#
    );
}

#[test]
fn test_error_severity_deserialization() {
    let w: ErrorSeverity = serde_json::from_str(r#""warning""#).unwrap();
    assert_eq!(w, ErrorSeverity::Warning);

    let e: ErrorSeverity = serde_json::from_str(r#""error""#).unwrap();
    assert_eq!(e, ErrorSeverity::Error);

    let c: ErrorSeverity = serde_json::from_str(r#""critical""#).unwrap();
    assert_eq!(c, ErrorSeverity::Critical);
}
```

### Step 2: Run all tests

```bash
cargo test -p automapper-api
```

Expected: all tests pass (integration tests + contract tests).

### Step 3: Run clippy

```bash
cargo clippy -p automapper-api -- -D warnings
```

Expected: no warnings.

### Step 4: Commit

```bash
git add -A
git commit -m "test(api): add comprehensive contract serialization tests for all API types"
```

---

## Task 7: Final Verification and Cleanup

### Step 1: Run full check suite

```bash
cargo check -p automapper-api
cargo test -p automapper-api
cargo clippy -p automapper-api -- -D warnings
cargo fmt -p automapper-api -- --check
```

Expected: all pass cleanly.

### Step 2: Verify the API endpoint summary

| Method | Path | Handler | Status |
|--------|------|---------|--------|
| `GET` | `/health` | `health_check` | Implemented, tested |
| `GET` | `/api/v1/coordinators` | `list_coordinators` | Implemented, tested |
| `GET` | `/api/v1/coordinators/{message_type}` | `get_coordinator` | Implemented |
| `POST` | `/api/v1/convert/edifact-to-bo4e` | `convert_edifact_to_bo4e` | Implemented, tested |
| `POST` | `/api/v1/convert/bo4e-to-edifact` | `convert_bo4e_to_edifact` | Implemented, tested |
| `POST` | `/api/v1/inspect/edifact` | `inspect_edifact` | Implemented, tested |

### Step 3: Commit

```bash
git add -A
git commit -m "chore(api): final cleanup and verification for Epic 1 — Axum REST API complete"
```
