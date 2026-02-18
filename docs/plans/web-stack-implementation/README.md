# Web Stack Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the web layer for the Rust EDIFACT-to-BO4E automapper: an Axum REST API, tonic gRPC services, and a Leptos WASM frontend, all served as a single binary. Ports the C# `Automapper.Api` + `Automapper.Web.Client` + `Automapper.Shared` layer to idiomatic Rust with the same feature surface (convert, inspect, coordinator discovery) but fresh Rust-native contracts.

**Architecture:** `automapper-api` crate hosts an Axum HTTP server with REST endpoints for conversion and inspection, plus tonic gRPC services multiplexed on the same port. An `AppState` holds a `CoordinatorRegistry` that discovers available coordinators from `automapper-core`. `automapper-web` crate is a Leptos CSR (client-side rendered) WASM application compiled to static assets and served by the Axum server. The frontend provides a two-panel converter UI with collapsible detail panels (segment tree, mapping trace, errors). Everything ships as a single binary.

**Tech Stack:** Rust 2021 edition, axum 0.8, tonic 0.12, prost 0.13, tokio 1.x, tower-http (CORS, static files, trace), leptos 0.7 (CSR mode), serde + serde_json, thiserror, tracing + tracing-subscriber, automapper-core (from Feature 1)

**Prerequisite:** Feature 1 (edifact-core-implementation) must be complete. This feature depends on `automapper-core` for the `Coordinator` trait, `create_coordinator()`, `FormatVersion`, and all domain types.

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | Axum REST API | `automapper-api` crate with Axum server, `CoordinatorRegistry`, REST endpoints (convert, inspect, coordinators, health), request/response types, error handling, CORS, static file serving, integration tests | Feature 1 |
| 2 | tonic gRPC Services | `transform.proto` and `inspection.proto`, `tonic-build` code generation, `TransformServiceImpl`, `InspectionServiceImpl`, HTTP/gRPC multiplexing on same port, gRPC integration tests | Epic 1 |
| 3 | Leptos WASM Frontend | `automapper-web` crate with Leptos CSR app, `ConverterPage`, `CoordinatorsPage`, `SegmentTreeView`, `TraceTable`, `ErrorList`, API client module, WASM build, static file serving from Axum | Epic 1 |

---

## Files in This Plan

1. [Epic 1: Axum REST API](./epic-01-axum-rest-api.md)
2. [Epic 2: tonic gRPC Services](./epic-02-tonic-grpc.md)
3. [Epic 3: Leptos WASM Frontend](./epic-03-leptos-frontend.md)

---

## Test Strategy

- **Unit tests**: `#[cfg(test)]` modules for `CoordinatorRegistry`, request/response serialization, error mapping
- **Integration tests**: `axum::test` / `tower::ServiceExt` for REST endpoints, `tonic` test client for gRPC
- **API contract tests**: Verify JSON serialization matches expected shapes with `insta` snapshots
- **Frontend tests**: Build verification (`cargo check -p automapper-web --target wasm32-unknown-unknown`), API client unit tests
- **End-to-end**: Single binary serves both API and WASM frontend, verify with `reqwest` HTTP client

## Crate Layout

```
crates/
  automapper-api/
    Cargo.toml
    build.rs                    # tonic-build for proto compilation
    src/
      main.rs                   # Axum + tonic server entry point
      lib.rs                    # Library root (for integration tests)
      state.rs                  # AppState, CoordinatorRegistry
      routes/
        mod.rs
        convert.rs              # POST /api/v1/convert/*
        inspect.rs              # POST /api/v1/inspect/edifact
        coordinators.rs         # GET /api/v1/coordinators
        health.rs               # GET /health
      contracts/
        mod.rs
        convert.rs              # ConvertRequest, ConvertResponse
        inspect.rs              # InspectRequest, InspectResponse, SegmentNode
        coordinators.rs         # CoordinatorInfo
        health.rs               # HealthResponse
        error.rs                # ApiError, ErrorSeverity
        trace.rs                # TraceEntry
      grpc/
        mod.rs
        transform.rs            # TransformServiceImpl
        inspection.rs           # InspectionServiceImpl
      error.rs                  # Axum error response mapping
  automapper-web/
    Cargo.toml
    src/
      main.rs                   # Leptos hydrate entry point
      lib.rs                    # Library root with App component
      app.rs                    # App component with Router
      pages/
        mod.rs
        converter.rs            # ConverterPage component
        coordinators.rs         # CoordinatorsPage component
      components/
        mod.rs
        code_editor.rs          # CodeEditor textarea component
        direction_toggle.rs     # Direction toggle component
        segment_tree.rs         # SegmentTreeView recursive component
        trace_table.rs          # TraceTable component
        error_list.rs           # ErrorList component
        collapsible_panel.rs    # CollapsiblePanel component
      api_client.rs             # REST API client module
      types.rs                  # Shared frontend types (Direction, etc.)
proto/
  transform.proto               # gRPC transform service definition
  inspection.proto              # gRPC inspection service definition
```

## Commands Reference

```bash
# Check API crate compiles
cargo check -p automapper-api

# Check web crate compiles (native target for lib)
cargo check -p automapper-web

# Check web crate compiles (WASM target)
cargo check -p automapper-web --target wasm32-unknown-unknown

# Run API tests
cargo test -p automapper-api

# Run web tests
cargo test -p automapper-web

# Run all web-stack tests
cargo test -p automapper-api -p automapper-web

# Run clippy
cargo clippy -p automapper-api -p automapper-web -- -D warnings

# Build release binary (API + embedded WASM)
cargo build --release -p automapper-api

# Run the server
cargo run -p automapper-api

# Build WASM frontend
trunk build crates/automapper-web/index.html --release
```
