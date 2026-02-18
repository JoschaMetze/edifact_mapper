---
feature: web-stack-implementation
epic: 2
title: "tonic gRPC Services"
depends_on: [1]
estimated_tasks: 6
crate: automapper-api
---

# Epic 2: tonic gRPC Services

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-api/src/`. All code must compile with `cargo check -p automapper-api`.

**Goal:** Add gRPC services to the `automapper-api` crate using tonic. Define `transform.proto` and `inspection.proto` with fresh Rust-idiomatic message types (not ported from the C# protos). Implement `TransformServiceImpl` and `InspectionServiceImpl` that delegate to the same `CoordinatorRegistry`. Multiplex gRPC alongside HTTP on the same port using Axum's built-in support. Write integration tests that call gRPC endpoints with a tonic test client.

**Architecture:** Protobuf definitions live in `proto/` at the workspace root. `tonic-build` compiles them in `build.rs`. The generated Rust code lives in `OUT_DIR` (not committed). Two gRPC service implementations wrap `CoordinatorRegistry` the same way the REST handlers do. Axum serves both REST and gRPC on the same TCP listener by inspecting the `content-type` header (tonic's `MultiplexService` or Axum's native routing). Streaming RPCs are included for batch conversion.

**Tech Stack:** tonic 0.12, prost 0.13, tonic-build 0.12, protobuf, axum 0.8 (multiplexing)

**Depends On:** Epic 1 (Axum REST API) — specifically `AppState`, `CoordinatorRegistry`, and the REST router.

---

## Task 1: Define Protobuf Files

### Step 1: Create proto directory

```bash
mkdir -p proto
```

### Step 2: Write `proto/transform.proto`

```protobuf
syntax = "proto3";

package automapper.transform;

/// Service for EDIFACT <-> BO4E conversion.
service TransformService {
  /// Convert a single EDIFACT message to BO4E JSON.
  rpc ConvertEdifactToBo4e(EdifactToBo4eRequest) returns (ConvertResponse);

  /// Convert a single BO4E JSON to EDIFACT.
  rpc ConvertBo4eToEdifact(Bo4eToEdifactRequest) returns (ConvertResponse);

  /// Stream: convert multiple EDIFACT messages to BO4E.
  rpc ConvertEdifactToBo4eStream(stream EdifactToBo4eRequest) returns (stream ConvertResponse);
}

/// Request to convert EDIFACT to BO4E.
message EdifactToBo4eRequest {
  /// Raw EDIFACT content.
  string edifact = 1;

  /// Optional format version override (e.g., "FV2504").
  string format_version = 2;

  /// Whether to include a mapping trace.
  bool include_trace = 3;
}

/// Request to convert BO4E to EDIFACT.
message Bo4eToEdifactRequest {
  /// BO4E JSON content.
  string bo4e_json = 1;

  /// Message type (e.g., "UTILMD").
  string message_type = 2;

  /// Optional format version override.
  string format_version = 3;
}

/// Conversion response (used for both directions).
message ConvertResponse {
  /// Whether the conversion succeeded.
  bool success = 1;

  /// Converted content (BO4E JSON or EDIFACT string).
  string result = 2;

  /// Mapping trace (populated if include_trace was true).
  MappingTrace trace = 3;

  /// Errors encountered during conversion.
  repeated ConversionError errors = 4;

  /// Conversion duration in milliseconds.
  double duration_ms = 5;
}

/// A mapping trace recording the conversion steps.
message MappingTrace {
  /// Name of the coordinator used.
  string coordinator_used = 1;

  /// Individual mapping steps.
  repeated MappingStep steps = 2;

  /// Total duration in milliseconds.
  double duration_ms = 3;
}

/// A single step in the mapping trace.
message MappingStep {
  /// Mapper/writer name.
  string mapper = 1;

  /// Source segment reference.
  string source_segment = 2;

  /// Target BO4E path.
  string target_path = 3;

  /// Mapped value.
  string value = 4;

  /// Optional note.
  string note = 5;
}

/// An error from a conversion operation.
message ConversionError {
  /// Machine-readable error code.
  string code = 1;

  /// Human-readable error message.
  string message = 2;

  /// Location in the source content.
  string location = 3;

  /// Error severity.
  ErrorSeverity severity = 4;
}

/// Severity level.
enum ErrorSeverity {
  ERROR_SEVERITY_UNSPECIFIED = 0;
  ERROR_SEVERITY_WARNING = 1;
  ERROR_SEVERITY_ERROR = 2;
  ERROR_SEVERITY_CRITICAL = 3;
}
```

### Step 3: Write `proto/inspection.proto`

```protobuf
syntax = "proto3";

package automapper.inspection;

/// Service for EDIFACT message inspection and coordinator discovery.
service InspectionService {
  /// Parse EDIFACT content into a segment tree.
  rpc InspectEdifact(InspectEdifactRequest) returns (InspectEdifactResponse);

  /// List all available coordinators.
  rpc ListCoordinators(ListCoordinatorsRequest) returns (ListCoordinatorsResponse);
}

/// Request to inspect EDIFACT content.
message InspectEdifactRequest {
  /// Raw EDIFACT content to parse.
  string edifact = 1;
}

/// Response with the parsed segment tree.
message InspectEdifactResponse {
  /// Parsed segments.
  repeated SegmentNode segments = 1;

  /// Total segment count.
  uint32 segment_count = 2;

  /// Detected message type (e.g., "UTILMD").
  string message_type = 3;

  /// Detected format version.
  string format_version = 4;
}

/// A single EDIFACT segment.
message SegmentNode {
  /// Segment tag (e.g., "UNH", "NAD").
  string tag = 1;

  /// 1-based segment ordinal.
  uint32 line_number = 2;

  /// Raw segment content.
  string raw_content = 3;

  /// Parsed data elements.
  repeated DataElement elements = 4;

  /// Child segments (for hierarchical grouping).
  repeated SegmentNode children = 5;
}

/// A data element within a segment.
message DataElement {
  /// 1-based position.
  uint32 position = 1;

  /// Simple element value.
  string value = 2;

  /// Component elements (for composite elements).
  repeated ComponentElement components = 3;
}

/// A component element within a composite data element.
message ComponentElement {
  /// 1-based position.
  uint32 position = 1;

  /// Component value.
  string value = 2;
}

/// Request to list coordinators (empty — no parameters needed).
message ListCoordinatorsRequest {}

/// Response with available coordinators.
message ListCoordinatorsResponse {
  repeated CoordinatorInfo coordinators = 1;
}

/// Information about an available coordinator.
message CoordinatorInfo {
  /// EDIFACT message type (e.g., "UTILMD").
  string message_type = 1;

  /// Human-readable description.
  string description = 2;

  /// Supported format versions.
  repeated string supported_versions = 3;
}
```

### Step 4: Commit

```bash
git add -A
git commit -m "feat(proto): define transform.proto and inspection.proto with Rust-idiomatic gRPC service definitions"
```

---

## Task 2: Add tonic Dependencies and build.rs

### Step 1: Update `crates/automapper-api/Cargo.toml` — add tonic dependencies

Add to `[dependencies]`:

```toml
# gRPC
tonic = "0.12"
prost = "0.13"
```

Add a `[build-dependencies]` section:

```toml
[build-dependencies]
tonic-build = "0.12"
```

### Step 2: Write `crates/automapper-api/build.rs`

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile transform.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true) // client needed for integration tests
        .compile_protos(
            &["../../proto/transform.proto"],
            &["../../proto"],
        )?;

    // Compile inspection.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../../proto/inspection.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
```

### Step 3: Run `cargo check -p automapper-api`

```bash
cargo check -p automapper-api
```

Expected: compiles. The generated code is in `OUT_DIR` but not yet referenced from Rust source.

### Step 4: Commit

```bash
git add -A
git commit -m "feat(api): add tonic/prost dependencies and build.rs for protobuf compilation"
```

---

## Task 3: Create gRPC Module and Include Generated Code

### Step 1: Create gRPC module directory

```bash
mkdir -p crates/automapper-api/src/grpc
```

### Step 2: Write `crates/automapper-api/src/grpc/mod.rs`

```rust
//! gRPC service implementations.
//!
//! Generated protobuf types are included via `tonic::include_proto!`.

pub mod transform;
pub mod inspection;

/// Generated protobuf types for the transform service.
pub mod transform_proto {
    tonic::include_proto!("automapper.transform");
}

/// Generated protobuf types for the inspection service.
pub mod inspection_proto {
    tonic::include_proto!("automapper.inspection");
}
```

### Step 3: Update `crates/automapper-api/src/lib.rs` — add grpc module

Add `pub mod grpc;` to `lib.rs`:

```rust
//! Automapper REST API server.
//!
//! Provides HTTP endpoints for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! coordinator discovery, and health checks. Also serves gRPC via tonic on the
//! same port and the Leptos WASM frontend as static files.

pub mod contracts;
pub mod error;
pub mod grpc;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Build the complete Axum router with all routes and middleware.
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

### Step 4: Write stub implementations

Write `crates/automapper-api/src/grpc/transform.rs`:

```rust
//! gRPC TransformService implementation.

use std::pin::Pin;
use std::sync::Arc;

use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};

use crate::grpc::transform_proto::transform_service_server::TransformService;
use crate::grpc::transform_proto::{
    Bo4eToEdifactRequest, ConvertResponse as ProtoConvertResponse, ConversionError,
    EdifactToBo4eRequest, ErrorSeverity, MappingStep, MappingTrace,
};
use crate::state::CoordinatorRegistry;

/// gRPC implementation of TransformService.
pub struct TransformServiceImpl {
    registry: Arc<CoordinatorRegistry>,
}

impl TransformServiceImpl {
    pub fn new(registry: Arc<CoordinatorRegistry>) -> Self {
        Self { registry }
    }

    fn build_response(
        &self,
        result: Result<crate::contracts::convert::ConvertResponse, crate::error::ApiError>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        match result {
            Ok(resp) => {
                let trace = if resp.trace.is_empty() {
                    None
                } else {
                    Some(MappingTrace {
                        coordinator_used: resp
                            .trace
                            .first()
                            .map(|t| t.mapper.clone())
                            .unwrap_or_default(),
                        steps: resp
                            .trace
                            .iter()
                            .map(|t| MappingStep {
                                mapper: t.mapper.clone(),
                                source_segment: t.source_segment.clone(),
                                target_path: t.target_path.clone(),
                                value: t.value.clone().unwrap_or_default(),
                                note: t.note.clone().unwrap_or_default(),
                            })
                            .collect(),
                        duration_ms: resp.duration_ms,
                    })
                };

                let errors: Vec<ConversionError> = resp
                    .errors
                    .iter()
                    .map(|e| ConversionError {
                        code: e.code.clone(),
                        message: e.message.clone(),
                        location: e.location.clone().unwrap_or_default(),
                        severity: match e.severity {
                            crate::contracts::error::ErrorSeverity::Warning => {
                                ErrorSeverity::Warning as i32
                            }
                            crate::contracts::error::ErrorSeverity::Error => {
                                ErrorSeverity::Error as i32
                            }
                            crate::contracts::error::ErrorSeverity::Critical => {
                                ErrorSeverity::Critical as i32
                            }
                        },
                    })
                    .collect();

                Ok(Response::new(ProtoConvertResponse {
                    success: resp.success,
                    result: resp.result.unwrap_or_default(),
                    trace,
                    errors,
                    duration_ms: resp.duration_ms,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

#[tonic::async_trait]
impl TransformService for TransformServiceImpl {
    async fn convert_edifact_to_bo4e(
        &self,
        request: Request<EdifactToBo4eRequest>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "gRPC: Converting EDIFACT to BO4E, format_version={}",
            if req.format_version.is_empty() {
                "auto"
            } else {
                &req.format_version
            }
        );

        let fv = if req.format_version.is_empty() {
            None
        } else {
            Some(req.format_version.as_str())
        };

        let result =
            self.registry
                .convert_edifact_to_bo4e(&req.edifact, fv, req.include_trace);

        self.build_response(result)
    }

    async fn convert_bo4e_to_edifact(
        &self,
        request: Request<Bo4eToEdifactRequest>,
    ) -> Result<Response<ProtoConvertResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            "gRPC: Converting BO4E to EDIFACT, message_type={}",
            req.message_type
        );

        let fv = if req.format_version.is_empty() {
            None
        } else {
            Some(req.format_version.as_str())
        };

        let result = self.registry.convert_bo4e_to_edifact(&req.bo4e_json, fv);

        self.build_response(result)
    }

    type ConvertEdifactToBo4eStreamStream =
        Pin<Box<dyn Stream<Item = Result<ProtoConvertResponse, Status>> + Send>>;

    async fn convert_edifact_to_bo4e_stream(
        &self,
        request: Request<Streaming<EdifactToBo4eRequest>>,
    ) -> Result<Response<Self::ConvertEdifactToBo4eStreamStream>, Status> {
        let registry = self.registry.clone();
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.message().await? {
                let fv = if req.format_version.is_empty() {
                    None
                } else {
                    Some(req.format_version.as_str())
                };

                let result = registry.convert_edifact_to_bo4e(
                    &req.edifact,
                    fv,
                    req.include_trace,
                );

                match result {
                    Ok(resp) => {
                        yield ProtoConvertResponse {
                            success: resp.success,
                            result: resp.result.unwrap_or_default(),
                            trace: None,
                            errors: vec![],
                            duration_ms: resp.duration_ms,
                        };
                    }
                    Err(e) => {
                        yield ProtoConvertResponse {
                            success: false,
                            result: String::new(),
                            trace: None,
                            errors: vec![ConversionError {
                                code: "CONVERSION_ERROR".to_string(),
                                message: e.to_string(),
                                location: String::new(),
                                severity: ErrorSeverity::Error as i32,
                            }],
                            duration_ms: 0.0,
                        };
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output)))
    }
}
```

Write `crates/automapper-api/src/grpc/inspection.rs`:

```rust
//! gRPC InspectionService implementation.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::grpc::inspection_proto::inspection_service_server::InspectionService;
use crate::grpc::inspection_proto::{
    ComponentElement as ProtoComponentElement, CoordinatorInfo as ProtoCoordinatorInfo,
    DataElement as ProtoDataElement, InspectEdifactRequest, InspectEdifactResponse,
    ListCoordinatorsRequest, ListCoordinatorsResponse, SegmentNode as ProtoSegmentNode,
};
use crate::state::CoordinatorRegistry;

/// gRPC implementation of InspectionService.
pub struct InspectionServiceImpl {
    registry: Arc<CoordinatorRegistry>,
}

impl InspectionServiceImpl {
    pub fn new(registry: Arc<CoordinatorRegistry>) -> Self {
        Self { registry }
    }
}

#[tonic::async_trait]
impl InspectionService for InspectionServiceImpl {
    async fn inspect_edifact(
        &self,
        request: Request<InspectEdifactRequest>,
    ) -> Result<Response<InspectEdifactResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("gRPC: Inspecting EDIFACT, length={}", req.edifact.len());

        let result = self
            .registry
            .inspect_edifact(&req.edifact)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let segments: Vec<ProtoSegmentNode> = result
            .segments
            .iter()
            .map(segment_node_to_proto)
            .collect();

        Ok(Response::new(InspectEdifactResponse {
            segments,
            segment_count: result.segment_count as u32,
            message_type: result.message_type.unwrap_or_default(),
            format_version: result.format_version.unwrap_or_default(),
        }))
    }

    async fn list_coordinators(
        &self,
        _request: Request<ListCoordinatorsRequest>,
    ) -> Result<Response<ListCoordinatorsResponse>, Status> {
        tracing::info!("gRPC: Listing coordinators");

        let coordinators: Vec<ProtoCoordinatorInfo> = self
            .registry
            .list()
            .iter()
            .map(|c| ProtoCoordinatorInfo {
                message_type: c.message_type.clone(),
                description: c.description.clone(),
                supported_versions: c.supported_versions.clone(),
            })
            .collect();

        Ok(Response::new(ListCoordinatorsResponse { coordinators }))
    }
}

/// Convert a REST `SegmentNode` to a proto `SegmentNode`.
fn segment_node_to_proto(
    node: &crate::contracts::inspect::SegmentNode,
) -> ProtoSegmentNode {
    let elements: Vec<ProtoDataElement> = node
        .elements
        .iter()
        .map(|e| ProtoDataElement {
            position: e.position,
            value: e.value.clone().unwrap_or_default(),
            components: e
                .components
                .as_ref()
                .map(|comps| {
                    comps
                        .iter()
                        .map(|c| ProtoComponentElement {
                            position: c.position,
                            value: c.value.clone().unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();

    let children: Vec<ProtoSegmentNode> = node
        .children
        .as_ref()
        .map(|ch| ch.iter().map(segment_node_to_proto).collect())
        .unwrap_or_default();

    ProtoSegmentNode {
        tag: node.tag.clone(),
        line_number: node.line_number,
        raw_content: node.raw_content.clone(),
        elements,
        children,
    }
}
```

### Step 5: Add `async-stream` and `tokio-stream` dependencies to `Cargo.toml`

Add to `[dependencies]`:

```toml
async-stream = "0.3"
tokio-stream = "0.1"
```

### Step 6: Run `cargo check -p automapper-api`

```bash
cargo check -p automapper-api
```

Expected: compiles with the gRPC module included.

### Step 7: Commit

```bash
git add -A
git commit -m "feat(api): implement TransformServiceImpl and InspectionServiceImpl for gRPC with streaming support"
```

---

## Task 4: Multiplex gRPC and HTTP on the Same Port

### Step 1: Update `crates/automapper-api/src/lib.rs` — add gRPC routing

```rust
//! Automapper REST API server.
//!
//! Provides HTTP endpoints for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! coordinator discovery, and health checks. Also serves gRPC via tonic on the
//! same port and the Leptos WASM frontend as static files.

pub mod contracts;
pub mod error;
pub mod grpc;
pub mod routes;
pub mod state;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use grpc::inspection::InspectionServiceImpl;
use grpc::inspection_proto::inspection_service_server::InspectionServiceServer;
use grpc::transform::TransformServiceImpl;
use grpc::transform_proto::transform_service_server::TransformServiceServer;

/// Build the complete Axum router with REST routes, gRPC services, and middleware.
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

    // Build gRPC services
    let transform_service =
        TransformServiceServer::new(TransformServiceImpl::new(state.registry.clone()));
    let inspection_service =
        InspectionServiceServer::new(InspectionServiceImpl::new(state.registry.clone()));

    // Build the gRPC router (tonic services exposed via Axum)
    let grpc_router = tonic::transport::Server::builder()
        .add_service(transform_service)
        .add_service(inspection_service)
        .into_router();

    Router::new()
        .nest("/api/v1", routes::api_routes())
        .route("/health", axum::routing::get(routes::health::health_check))
        .merge(grpc_router)
        .fallback_service(ServeDir::new(static_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Build an HTTP-only router (no gRPC) for testing scenarios.
pub fn build_http_router(state: state::AppState) -> Router {
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

### Step 2: Update `crates/automapper-api/src/main.rs` — log gRPC availability

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
        "automapper-api listening on {} (REST + gRPC), static_dir={}",
        bind_addr,
        static_dir
    );

    axum::serve(listener, app)
        .await
        .expect("server error");
}
```

### Step 3: Update the integration test helper to use `build_http_router`

Update the `app()` function in `crates/automapper-api/tests/api_integration.rs`:

```rust
fn app() -> axum::Router {
    let state = AppState::new();
    automapper_api::build_http_router(state)
}
```

This ensures the REST integration tests keep working without gRPC setup complexity.

### Step 4: Run `cargo check -p automapper-api`

```bash
cargo check -p automapper-api
```

Expected: compiles.

### Step 5: Run tests

```bash
cargo test -p automapper-api
```

Expected: all existing REST tests pass.

### Step 6: Commit

```bash
git add -A
git commit -m "feat(api): multiplex gRPC (tonic) alongside REST (axum) on the same port"
```

---

## Task 5: Add gRPC Integration Tests

### Step 1: Write `crates/automapper-api/tests/grpc_integration.rs`

```rust
//! Integration tests for gRPC services.
//!
//! Spins up a real TCP server and connects a tonic client.

use std::net::SocketAddr;

use tokio::net::TcpListener;

use automapper_api::grpc::inspection_proto::inspection_service_client::InspectionServiceClient;
use automapper_api::grpc::inspection_proto::{InspectEdifactRequest, ListCoordinatorsRequest};
use automapper_api::grpc::transform_proto::transform_service_client::TransformServiceClient;
use automapper_api::grpc::transform_proto::EdifactToBo4eRequest;
use automapper_api::state::AppState;

/// Start a test server and return the address.
async fn start_test_server() -> SocketAddr {
    let state = AppState::new();
    let app = automapper_api::build_router(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    addr
}

#[tokio::test]
async fn test_grpc_list_coordinators() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = InspectionServiceClient::connect(url).await.unwrap();

    let response = client
        .list_coordinators(ListCoordinatorsRequest {})
        .await
        .unwrap();

    let coordinators = response.into_inner().coordinators;
    assert!(!coordinators.is_empty());
    assert!(coordinators.iter().any(|c| c.message_type == "UTILMD"));
}

#[tokio::test]
async fn test_grpc_inspect_edifact() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = InspectionServiceClient::connect(url).await.unwrap();

    let response = client
        .inspect_edifact(InspectEdifactRequest {
            edifact: "UNH+1+UTILMD:D:11A:UN:5.2e'BGM+E01+DOC001'UNT+3+1'".to_string(),
        })
        .await
        .unwrap();

    let inner = response.into_inner();
    assert_eq!(inner.segment_count, 3);
    assert_eq!(inner.message_type, "UTILMD");
    assert_eq!(inner.segments[0].tag, "UNH");
    assert_eq!(inner.segments[1].tag, "BGM");
    assert_eq!(inner.segments[2].tag, "UNT");
}

#[tokio::test]
async fn test_grpc_convert_edifact_to_bo4e() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = TransformServiceClient::connect(url).await.unwrap();

    let edifact = concat!(
        "UNB+UNOC:3+sender+receiver+231215:1200+ref001'",
        "UNH+1+UTILMD:D:11A:UN:5.2e'",
        "BGM+E01+DOC001'",
        "UNT+3+1'",
        "UNZ+1+ref001'"
    );

    let response = client
        .convert_edifact_to_bo4e(EdifactToBo4eRequest {
            edifact: edifact.to_string(),
            format_version: "FV2504".to_string(),
            include_trace: false,
        })
        .await
        .unwrap();

    let inner = response.into_inner();
    // The response should be well-formed (success or conversion error, not gRPC error)
    assert!(inner.duration_ms >= 0.0);
}

#[tokio::test]
async fn test_grpc_inspect_empty_edifact_returns_error() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}");

    let mut client = InspectionServiceClient::connect(url).await.unwrap();

    let result = client
        .inspect_edifact(InspectEdifactRequest {
            edifact: String::new(),
        })
        .await;

    // Empty EDIFACT should return a gRPC error (INVALID_ARGUMENT)
    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}
```

### Step 2: Run gRPC integration tests

```bash
cargo test -p automapper-api -- grpc
```

Expected: all 4 gRPC tests pass.

### Step 3: Run all tests

```bash
cargo test -p automapper-api
```

Expected: all tests pass (REST integration + contract + gRPC integration).

### Step 4: Commit

```bash
git add -A
git commit -m "test(api): add gRPC integration tests for TransformService and InspectionService"
```

---

## Task 6: Final Verification and Cleanup

### Step 1: Run full check suite

```bash
cargo check -p automapper-api
cargo test -p automapper-api
cargo clippy -p automapper-api -- -D warnings
cargo fmt -p automapper-api -- --check
```

Expected: all pass cleanly.

### Step 2: Verify the gRPC service summary

| Service | Method | Status |
|---------|--------|--------|
| `TransformService` | `ConvertEdifactToBo4e` | Implemented, tested |
| `TransformService` | `ConvertBo4eToEdifact` | Implemented |
| `TransformService` | `ConvertEdifactToBo4eStream` | Implemented (streaming) |
| `InspectionService` | `InspectEdifact` | Implemented, tested |
| `InspectionService` | `ListCoordinators` | Implemented, tested |

### Step 3: Commit

```bash
git add -A
git commit -m "chore(api): final cleanup and verification for Epic 2 — tonic gRPC services complete"
```
