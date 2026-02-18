---
feature: web-stack-implementation
epic: 3
title: "Leptos WASM Frontend"
depends_on: [1]
estimated_tasks: 8
crate: automapper-web
---

# Epic 3: Leptos WASM Frontend

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-web/src/`. All code must compile with `cargo check -p automapper-web`.

**Goal:** Build the `automapper-web` crate as a Leptos CSR (client-side rendered) WASM application. The frontend provides a two-panel converter UI with direction toggle, collapsible detail panels (segment tree, mapping trace, errors), and a coordinator discovery page. The WASM bundle is served as static files by the Axum server from Epic 1, enabling single-binary deployment.

**Architecture:** Leptos 0.7 in CSR mode compiles to WASM via `trunk`. The `App` component sets up a router with two pages: `ConverterPage` (/) and `CoordinatorsPage` (/coordinators). The `ConverterPage` has two textarea-based code editors (upgradable to Monaco later), a direction toggle, and a convert button that calls the REST API via `reqwasm`/`gloo-net`. Collapsible panels show the segment tree (recursive `SegmentTreeView`), mapping trace (`TraceTable`), and errors (`ErrorList`). An `api_client` module wraps all REST calls.

**Tech Stack:** leptos 0.7 (CSR), trunk (WASM bundler), gloo-net (HTTP client), serde + serde_json, web-sys, wasm-bindgen

**Depends On:** Epic 1 (Axum REST API) — the frontend calls the REST endpoints defined there.

---

## Task 1: Create `automapper-web` Crate Skeleton

### Step 1: Create directory structure

```bash
mkdir -p crates/automapper-web/src/pages
mkdir -p crates/automapper-web/src/components
```

### Step 2: Write `crates/automapper-web/Cargo.toml`

```toml
[package]
name = "automapper-web"
version = "0.1.0"
edition = "2021"
description = "Leptos WASM frontend for edifact-bo4e-automapper"
publish = false

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = { version = "0.7", features = ["csr"] }
leptos_router = { version = "0.7", features = ["csr"] }

# HTTP client for WASM
gloo-net = { version = "0.6", features = ["http", "json"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# WASM bindings
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Document", "console"] }

# Logging
log = "0.4"
console_log = "1"
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### Step 3: Write `crates/automapper-web/trunk.toml`

This file configures the `trunk` WASM bundler:

```toml
[build]
target = "index.html"
dist = "dist"

[watch]
watch = ["src", "index.html", "style"]

[serve]
address = "127.0.0.1"
port = 3000

[[proxy]]
rewrite = "/api/"
backend = "http://127.0.0.1:8080/api/"

[[proxy]]
rewrite = "/health"
backend = "http://127.0.0.1:8080/health"
```

### Step 4: Write `crates/automapper-web/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Automapper — EDIFACT/BO4E Converter</title>
    <link data-trunk rel="css" href="style/main.css" />
</head>
<body>
    <div id="app"></div>
</body>
</html>
```

### Step 5: Create style directory and write `crates/automapper-web/style/main.css`

```bash
mkdir -p crates/automapper-web/style
```

```css
/* === Reset === */
*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

/* === Variables === */
:root {
    --color-bg: #1e1e2e;
    --color-surface: #282a36;
    --color-surface-hover: #313345;
    --color-border: #44475a;
    --color-text: #f8f8f2;
    --color-text-muted: #6272a4;
    --color-primary: #8be9fd;
    --color-secondary: #bd93f9;
    --color-success: #50fa7b;
    --color-warning: #f1fa8c;
    --color-error: #ff5555;
    --color-info: #8be9fd;
    --font-mono: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace;
    --font-sans: 'Inter', 'Segoe UI', system-ui, sans-serif;
    --radius: 6px;
    --shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

body {
    background-color: var(--color-bg);
    color: var(--color-text);
    font-family: var(--font-sans);
    line-height: 1.6;
}

/* === Layout === */
.app-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 1rem;
}

/* === Navbar === */
.navbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.5rem;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 1.5rem;
}

.navbar h1 {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-primary);
}

.navbar nav a {
    color: var(--color-text-muted);
    text-decoration: none;
    margin-left: 1.5rem;
    font-size: 0.9rem;
    transition: color 0.2s;
}

.navbar nav a:hover,
.navbar nav a.active {
    color: var(--color-primary);
}

/* === Converter Layout === */
.converter-layout {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 1rem;
    align-items: start;
    margin-bottom: 1.5rem;
}

@media (max-width: 768px) {
    .converter-layout {
        grid-template-columns: 1fr;
    }
}

/* === Editor Panel === */
.editor-panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    overflow: hidden;
}

.editor-panel .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.85rem;
    color: var(--color-text-muted);
}

.editor-panel textarea {
    width: 100%;
    min-height: 350px;
    padding: 1rem;
    background: transparent;
    color: var(--color-text);
    font-family: var(--font-mono);
    font-size: 0.85rem;
    border: none;
    outline: none;
    resize: vertical;
    line-height: 1.5;
}

.editor-panel textarea::placeholder {
    color: var(--color-text-muted);
}

.editor-panel textarea:read-only {
    color: var(--color-text-muted);
    cursor: default;
}

/* === Controls (center column) === */
.controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding-top: 2rem;
}

/* === Buttons === */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.2rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
}

.btn:hover {
    background: var(--color-surface-hover);
    border-color: var(--color-primary);
}

.btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-primary {
    background: var(--color-primary);
    color: var(--color-bg);
    border-color: var(--color-primary);
    font-weight: 600;
}

.btn-primary:hover {
    background: #a8f0ff;
}

.btn-small {
    padding: 0.3rem 0.6rem;
    font-size: 0.8rem;
}

/* === Direction Toggle === */
.direction-toggle {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
}

.direction-toggle .label {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    text-align: center;
}

/* === Collapsible Panel === */
.collapsible-panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    margin-bottom: 0.75rem;
    overflow: hidden;
}

.collapsible-panel .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 1rem;
    cursor: pointer;
    user-select: none;
    transition: background 0.2s;
}

.collapsible-panel .panel-header:hover {
    background: var(--color-surface-hover);
}

.collapsible-panel .panel-header .title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    font-weight: 500;
}

.collapsible-panel .panel-header .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    font-size: 0.75rem;
    background: var(--color-border);
    color: var(--color-text-muted);
}

.collapsible-panel .panel-header .chevron {
    transition: transform 0.2s;
    color: var(--color-text-muted);
}

.collapsible-panel.open .panel-header .chevron {
    transform: rotate(90deg);
}

.collapsible-panel .panel-body {
    display: none;
    padding: 1rem;
    border-top: 1px solid var(--color-border);
}

.collapsible-panel.open .panel-body {
    display: block;
}

/* === Segment Tree === */
.segment-tree {
    font-family: var(--font-mono);
    font-size: 0.8rem;
}

.segment-tree .tree-node {
    padding: 0.25rem 0;
}

.segment-tree .tree-node .node-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.2rem 0.5rem;
    border-radius: 3px;
    cursor: default;
}

.segment-tree .tree-node .node-row:hover {
    background: var(--color-surface-hover);
}

.segment-tree .tag-badge {
    display: inline-flex;
    padding: 0.1rem 0.4rem;
    background: var(--color-primary);
    color: var(--color-bg);
    border-radius: 3px;
    font-weight: 600;
    font-size: 0.75rem;
}

.segment-tree .line-number {
    color: var(--color-text-muted);
    font-size: 0.7rem;
    min-width: 3rem;
}

.segment-tree .raw-content {
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 500px;
}

.segment-tree .children {
    padding-left: 1.5rem;
    border-left: 1px solid var(--color-border);
    margin-left: 0.75rem;
}

/* === Trace Table === */
.trace-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
}

.trace-table th {
    text-align: left;
    padding: 0.5rem;
    border-bottom: 2px solid var(--color-border);
    color: var(--color-text-muted);
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.trace-table td {
    padding: 0.4rem 0.5rem;
    border-bottom: 1px solid var(--color-border);
    font-family: var(--font-mono);
}

.trace-table tr:hover td {
    background: var(--color-surface-hover);
}

.trace-table .value-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.trace-table .note-cell {
    color: var(--color-text-muted);
    font-family: var(--font-sans);
    font-style: italic;
}

/* === Error List === */
.error-list {
    list-style: none;
}

.error-list .error-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.5rem;
    border-bottom: 1px solid var(--color-border);
}

.error-list .error-item:last-child {
    border-bottom: none;
}

.error-list .severity-icon {
    flex-shrink: 0;
    width: 1.25rem;
    height: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 0.7rem;
    font-weight: 700;
}

.error-list .severity-icon.warning {
    background: var(--color-warning);
    color: var(--color-bg);
}

.error-list .severity-icon.error {
    background: var(--color-error);
    color: white;
}

.error-list .severity-icon.critical {
    background: var(--color-error);
    color: white;
}

.error-list .error-code {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 0.85rem;
}

.error-list .error-message {
    font-size: 0.85rem;
}

.error-list .error-location {
    font-size: 0.75rem;
    color: var(--color-text-muted);
}

.no-errors {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: rgba(80, 250, 123, 0.1);
    border: 1px solid rgba(80, 250, 123, 0.3);
    border-radius: var(--radius);
    color: var(--color-success);
    font-size: 0.85rem;
}

/* === Coordinator Cards === */
.coordinator-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
}

.coordinator-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 1.25rem;
    transition: border-color 0.2s;
}

.coordinator-card:hover {
    border-color: var(--color-primary);
}

.coordinator-card h3 {
    font-size: 1.1rem;
    color: var(--color-primary);
    margin-bottom: 0.25rem;
}

.coordinator-card .description {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    margin-bottom: 0.75rem;
}

.coordinator-card .versions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
}

.coordinator-card .version-badge {
    display: inline-flex;
    padding: 0.15rem 0.5rem;
    background: var(--color-secondary);
    color: var(--color-bg);
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 500;
}

/* === Loading / Status === */
.loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    color: var(--color-text-muted);
}

.loading .spinner {
    width: 1.5rem;
    height: 1.5rem;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-right: 0.75rem;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}
```

### Step 6: Commit

```bash
git add -A
git commit -m "feat(web): scaffold automapper-web crate with Cargo.toml, trunk.toml, index.html, and CSS theme"
```

---

## Task 2: Implement Shared Types and API Client

### Step 1: Write `crates/automapper-web/src/types.rs`

```rust
//! Shared types for the frontend.

use serde::{Deserialize, Serialize};

/// Conversion direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    EdifactToBo4e,
    Bo4eToEdifact,
}

impl Direction {
    pub fn label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "EDIFACT -> BO4E",
            Direction::Bo4eToEdifact => "BO4E -> EDIFACT",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Direction::EdifactToBo4e => Direction::Bo4eToEdifact,
            Direction::Bo4eToEdifact => Direction::EdifactToBo4e,
        }
    }

    pub fn input_label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "EDIFACT",
            Direction::Bo4eToEdifact => "BO4E JSON",
        }
    }

    pub fn output_label(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "BO4E JSON",
            Direction::Bo4eToEdifact => "EDIFACT",
        }
    }

    pub fn api_path(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => "/api/v1/convert/edifact-to-bo4e",
            Direction::Bo4eToEdifact => "/api/v1/convert/bo4e-to-edifact",
        }
    }

    pub fn input_placeholder(&self) -> &'static str {
        match self {
            Direction::EdifactToBo4e => concat!(
                "UNA:+.? '\n",
                "UNB+UNOC:3+sender+recipient+231215:1200+123456789'\n",
                "UNH+1+UTILMD:D:11A:UN:5.2e'\n",
                "BGM+E01+DOC001'\n",
                "...\n",
                "UNT+42+1'\n",
                "UNZ+1+123456789'"
            ),
            Direction::Bo4eToEdifact => concat!(
                "{\n",
                "  \"transaktions_id\": \"TXN001\",\n",
                "  \"absender\": { ... },\n",
                "  \"empfaenger\": { ... },\n",
                "  \"marktlokationen\": [ ... ]\n",
                "}"
            ),
        }
    }
}

/// Conversion request (matches the REST API contract).
#[derive(Debug, Clone, Serialize)]
pub struct ConvertRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_version: Option<String>,
    pub include_trace: bool,
}

/// Conversion response (matches the REST API contract).
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertResponse {
    pub success: bool,
    pub result: Option<String>,
    pub trace: Vec<TraceEntry>,
    pub errors: Vec<ApiErrorEntry>,
    pub duration_ms: f64,
}

/// Inspect request.
#[derive(Debug, Clone, Serialize)]
pub struct InspectRequest {
    pub edifact: String,
}

/// Inspect response.
#[derive(Debug, Clone, Deserialize)]
pub struct InspectResponse {
    pub segments: Vec<SegmentNode>,
    pub segment_count: usize,
    pub message_type: Option<String>,
    pub format_version: Option<String>,
}

/// A single EDIFACT segment.
#[derive(Debug, Clone, Deserialize)]
pub struct SegmentNode {
    pub tag: String,
    pub line_number: u32,
    pub raw_content: String,
    pub elements: Vec<DataElement>,
    pub children: Option<Vec<SegmentNode>>,
}

/// A data element within a segment.
#[derive(Debug, Clone, Deserialize)]
pub struct DataElement {
    pub position: u32,
    pub value: Option<String>,
    pub components: Option<Vec<ComponentElement>>,
}

/// A component element within a composite data element.
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentElement {
    pub position: u32,
    pub value: Option<String>,
}

/// A mapping trace entry.
#[derive(Debug, Clone, Deserialize)]
pub struct TraceEntry {
    pub mapper: String,
    pub source_segment: String,
    pub target_path: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

/// An error entry from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorEntry {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
    pub severity: String,
}

/// Coordinator info.
#[derive(Debug, Clone, Deserialize)]
pub struct CoordinatorInfo {
    pub message_type: String,
    pub description: String,
    pub supported_versions: Vec<String>,
}

/// Health response.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub available_coordinators: Vec<String>,
    pub uptime_seconds: f64,
}
```

### Step 2: Write `crates/automapper-web/src/api_client.rs`

```rust
//! REST API client for calling the automapper-api server.
//!
//! In production, the API is served on the same origin (single binary).
//! During development, trunk proxies `/api/*` to the API server.

use gloo_net::http::Request;

use crate::types::{
    ConvertRequest, ConvertResponse, CoordinatorInfo, HealthResponse, InspectRequest,
    InspectResponse,
};

/// Base URL for API calls. Empty string means same origin.
const API_BASE: &str = "";

/// Convert content using the specified direction endpoint.
pub async fn convert(
    api_path: &str,
    content: &str,
    format_version: Option<String>,
    include_trace: bool,
) -> Result<ConvertResponse, String> {
    let request_body = ConvertRequest {
        content: content.to_string(),
        format_version,
        include_trace,
    };

    let url = format!("{API_BASE}{api_path}");

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<ConvertResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(format!("HTTP {status}: {body}"))
    }
}

/// Inspect EDIFACT content, returning a segment tree.
pub async fn inspect_edifact(edifact: &str) -> Result<InspectResponse, String> {
    let request_body = InspectRequest {
        edifact: edifact.to_string(),
    };

    let url = format!("{API_BASE}/api/v1/inspect/edifact");

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<InspectResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}

/// List available coordinators.
pub async fn list_coordinators() -> Result<Vec<CoordinatorInfo>, String> {
    let url = format!("{API_BASE}/api/v1/coordinators");

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<Vec<CoordinatorInfo>>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}

/// Get health status.
pub async fn get_health() -> Result<HealthResponse, String> {
    let url = format!("{API_BASE}/health");

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("failed to parse response: {e}"))
    } else {
        let status = response.status();
        Err(format!("HTTP {status}"))
    }
}
```

### Step 3: Run `cargo check -p automapper-web`

```bash
cargo check -p automapper-web
```

Expected: compiles (native target for the library portion).

### Step 4: Commit

```bash
git add -A
git commit -m "feat(web): implement shared types and REST API client module"
```

---

## Task 3: Implement Components

### Step 1: Write `crates/automapper-web/src/components/mod.rs`

```rust
//! Reusable UI components.

pub mod code_editor;
pub mod collapsible_panel;
pub mod direction_toggle;
pub mod error_list;
pub mod segment_tree;
pub mod trace_table;
```

### Step 2: Write `crates/automapper-web/src/components/code_editor.rs`

```rust
//! Textarea-based code editor component.
//!
//! A simple textarea with monospace font and syntax-appropriate placeholder text.
//! Designed to be upgraded to Monaco Editor later.

use leptos::prelude::*;

/// Props for the CodeEditor component.
#[component]
pub fn CodeEditor(
    /// The current value of the editor.
    value: ReadSignal<String>,
    /// Callback when the value changes.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// Whether the editor is read-only.
    #[prop(default = false)]
    readonly: bool,
    /// Placeholder text.
    #[prop(default = "")]
    placeholder: &'static str,
    /// Label shown in the panel header.
    #[prop(default = "Editor")]
    label: &'static str,
) -> impl IntoView {
    view! {
        <div class="editor-panel">
            <div class="panel-header">
                <span>{label}</span>
            </div>
            <textarea
                prop:value=move || value.get()
                on:input=move |ev| {
                    if let Some(on_change) = &on_change {
                        let val = event_target_value(&ev);
                        on_change.run(val);
                    }
                }
                readonly=readonly
                placeholder=placeholder
                spellcheck="false"
            />
        </div>
    }
}
```

### Step 3: Write `crates/automapper-web/src/components/direction_toggle.rs`

```rust
//! Direction toggle component for switching between EDIFACT->BO4E and BO4E->EDIFACT.

use leptos::prelude::*;

use crate::types::Direction;

#[component]
pub fn DirectionToggle(
    /// Current direction.
    direction: ReadSignal<Direction>,
    /// Callback when direction is toggled.
    on_toggle: WriteSignal<Direction>,
) -> impl IntoView {
    let toggle = move |_| {
        on_toggle.update(|d| *d = d.toggle());
    };

    view! {
        <div class="direction-toggle">
            <button class="btn btn-small" on:click=toggle>
                {move || match direction.get() {
                    Direction::EdifactToBo4e => "EDIFACT -> BO4E",
                    Direction::Bo4eToEdifact => "BO4E -> EDIFACT",
                }}
            </button>
            <span class="label">"click to swap"</span>
        </div>
    }
}
```

### Step 4: Write `crates/automapper-web/src/components/collapsible_panel.rs`

```rust
//! Collapsible panel component with a title bar and expandable body.

use leptos::prelude::*;

#[component]
pub fn CollapsiblePanel(
    /// Panel title.
    title: &'static str,
    /// Optional badge text (e.g., count).
    #[prop(optional, into)]
    badge: Signal<String>,
    /// Whether the panel starts expanded.
    #[prop(default = false)]
    initially_open: bool,
    /// Whether the panel is disabled (cannot be opened).
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Panel body content.
    children: Children,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(initially_open);

    let toggle = move |_| {
        if !disabled.get() {
            set_is_open.update(|open| *open = !*open);
        }
    };

    let panel_class = move || {
        let mut cls = "collapsible-panel".to_string();
        if is_open.get() {
            cls.push_str(" open");
        }
        if disabled.get() {
            cls.push_str(" disabled");
        }
        cls
    };

    view! {
        <div class=panel_class>
            <div class="panel-header" on:click=toggle>
                <div class="title">
                    <span>{title}</span>
                    {move || {
                        let b = badge.get();
                        if b.is_empty() {
                            None
                        } else {
                            Some(view! { <span class="badge">{b}</span> })
                        }
                    }}
                </div>
                <span class="chevron">{move || if is_open.get() { "v" } else { ">" }}</span>
            </div>
            <div class="panel-body">
                {children()}
            </div>
        </div>
    }
}
```

### Step 5: Write `crates/automapper-web/src/components/segment_tree.rs`

```rust
//! Recursive segment tree view component.

use leptos::prelude::*;

use crate::types::SegmentNode;

/// Display a list of segment nodes as a tree.
#[component]
pub fn SegmentTreeView(
    /// The segments to display.
    segments: Signal<Vec<SegmentNode>>,
) -> impl IntoView {
    view! {
        <div class="segment-tree">
            <For
                each=move || segments.get().into_iter().enumerate().collect::<Vec<_>>()
                key=|(i, _)| *i
                children=move |(_, node)| {
                    view! { <SegmentNodeView node=node /> }
                }
            />
        </div>
    }
}

/// Display a single segment node (recursive for children).
#[component]
fn SegmentNodeView(
    /// The segment node to display.
    node: SegmentNode,
) -> impl IntoView {
    let has_children = node
        .children
        .as_ref()
        .map(|c| !c.is_empty())
        .unwrap_or(false);

    let children_nodes = node.children.clone().unwrap_or_default();
    let tag = node.tag.clone();
    let line_number = node.line_number;
    let raw_content = node.raw_content.clone();

    view! {
        <div class="tree-node">
            <div class="node-row">
                <span class="tag-badge">{tag}</span>
                <span class="line-number">"L"{line_number}</span>
                <span class="raw-content">{raw_content}</span>
            </div>
            {if has_children {
                Some(view! {
                    <div class="children">
                        {children_nodes
                            .into_iter()
                            .map(|child| view! { <SegmentNodeView node=child /> })
                            .collect::<Vec<_>>()}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}
```

### Step 6: Write `crates/automapper-web/src/components/trace_table.rs`

```rust
//! Mapping trace table component.

use leptos::prelude::*;

use crate::types::TraceEntry;

/// Display mapping trace entries as a table.
#[component]
pub fn TraceTable(
    /// The trace entries to display.
    entries: Signal<Vec<TraceEntry>>,
) -> impl IntoView {
    view! {
        <table class="trace-table">
            <thead>
                <tr>
                    <th>"Mapper"</th>
                    <th>"Source Segment"</th>
                    <th>"Target Path"</th>
                    <th>"Value"</th>
                    <th>"Note"</th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || entries.get().into_iter().enumerate().collect::<Vec<_>>()
                    key=|(i, _)| *i
                    children=move |(_, entry)| {
                        let value = entry.value.clone().unwrap_or_default();
                        let note = entry.note.clone().unwrap_or_default();
                        view! {
                            <tr>
                                <td>{entry.mapper.clone()}</td>
                                <td><code>{entry.source_segment.clone()}</code></td>
                                <td><code>{entry.target_path.clone()}</code></td>
                                <td class="value-cell">{value}</td>
                                <td class="note-cell">{note}</td>
                            </tr>
                        }
                    }
                />
            </tbody>
        </table>
    }
}
```

### Step 7: Write `crates/automapper-web/src/components/error_list.rs`

```rust
//! Error list component with severity-based styling.

use leptos::prelude::*;

use crate::types::ApiErrorEntry;

/// Display a list of API errors grouped by severity.
#[component]
pub fn ErrorList(
    /// The errors to display.
    errors: Signal<Vec<ApiErrorEntry>>,
) -> impl IntoView {
    view! {
        {move || {
            let errs = errors.get();
            if errs.is_empty() {
                view! {
                    <div class="no-errors">
                        "No errors"
                    </div>
                }
                .into_any()
            } else {
                view! {
                    <ul class="error-list">
                        {errs
                            .into_iter()
                            .enumerate()
                            .map(|(i, err)| {
                                let severity_class = match err.severity.as_str() {
                                    "warning" => "warning",
                                    "critical" => "critical",
                                    _ => "error",
                                };
                                let severity_char = match err.severity.as_str() {
                                    "warning" => "W",
                                    "critical" => "!",
                                    _ => "E",
                                };
                                let location = err.location.clone();
                                view! {
                                    <li class="error-item" key=i>
                                        <span class={format!("severity-icon {severity_class}")}>
                                            {severity_char}
                                        </span>
                                        <div>
                                            <div>
                                                <span class="error-code">"["{err.code.clone()}"] "</span>
                                                <span class="error-message">{err.message.clone()}</span>
                                            </div>
                                            {location.map(|loc| {
                                                view! {
                                                    <div class="error-location">"Location: "{loc}</div>
                                                }
                                            })}
                                        </div>
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </ul>
                }
                .into_any()
            }
        }}
    }
}
```

### Step 8: Run `cargo check -p automapper-web`

```bash
cargo check -p automapper-web
```

Expected: compiles.

### Step 9: Commit

```bash
git add -A
git commit -m "feat(web): implement all UI components — CodeEditor, DirectionToggle, CollapsiblePanel, SegmentTreeView, TraceTable, ErrorList"
```

---

## Task 4: Implement Pages

### Step 1: Write `crates/automapper-web/src/pages/mod.rs`

```rust
//! Page components.

pub mod converter;
pub mod coordinators;
```

### Step 2: Write `crates/automapper-web/src/pages/converter.rs`

```rust
//! Converter page — two-panel layout with code editors and collapsible detail panels.

use leptos::prelude::*;

use crate::api_client;
use crate::components::code_editor::CodeEditor;
use crate::components::collapsible_panel::CollapsiblePanel;
use crate::components::direction_toggle::DirectionToggle;
use crate::components::error_list::ErrorList;
use crate::components::segment_tree::SegmentTreeView;
use crate::components::trace_table::TraceTable;
use crate::types::{ApiErrorEntry, Direction, SegmentNode, TraceEntry};

/// Main converter page.
#[component]
pub fn ConverterPage() -> impl IntoView {
    // Input/output state
    let (input, set_input) = signal(String::new());
    let (output, set_output) = signal(String::new());
    let (direction, set_direction) = signal(Direction::EdifactToBo4e);
    let (is_converting, set_is_converting) = signal(false);

    // Detail panel state
    let (segments, set_segments) = signal(Vec::<SegmentNode>::new());
    let (trace, set_trace) = signal(Vec::<TraceEntry>::new());
    let (errors, set_errors) = signal(Vec::<ApiErrorEntry>::new());
    let (duration_ms, set_duration_ms) = signal(0.0_f64);

    // Convert action
    let convert_action = Action::new(move |_: &()| {
        let input_val = input.get();
        let dir = direction.get();

        async move {
            set_is_converting.set(true);
            set_errors.set(vec![]);
            set_segments.set(vec![]);
            set_trace.set(vec![]);
            set_output.set(String::new());

            // If EDIFACT input, also inspect for segment tree
            if dir == Direction::EdifactToBo4e {
                match api_client::inspect_edifact(&input_val).await {
                    Ok(inspect) => {
                        set_segments.set(inspect.segments);
                    }
                    Err(e) => {
                        log::warn!("Inspect failed: {e}");
                    }
                }
            }

            // Call convert endpoint
            match api_client::convert(dir.api_path(), &input_val, None, true).await {
                Ok(resp) => {
                    set_duration_ms.set(resp.duration_ms);
                    if resp.success {
                        set_output.set(resp.result.unwrap_or_default());
                        set_trace.set(resp.trace);
                    }
                    if !resp.errors.is_empty() {
                        set_errors.set(resp.errors);
                    }
                }
                Err(e) => {
                    set_errors.set(vec![ApiErrorEntry {
                        code: "CLIENT_ERROR".to_string(),
                        message: e,
                        location: None,
                        severity: "error".to_string(),
                    }]);
                }
            }

            set_is_converting.set(false);
        }
    });

    // Badge signals for collapsible panels
    let segment_badge = Signal::derive(move || {
        let count = segments.get().len();
        if count > 0 {
            format!("{count} segments")
        } else {
            String::new()
        }
    });

    let trace_badge = Signal::derive(move || {
        let count = trace.get().len();
        let ms = duration_ms.get();
        if count > 0 {
            format!("{count} steps ({ms:.1}ms)")
        } else {
            String::new()
        }
    });

    let error_badge = Signal::derive(move || {
        let count = errors.get().len();
        if count > 0 {
            format!("{count}")
        } else {
            String::new()
        }
    });

    let segments_empty = Signal::derive(move || segments.get().is_empty());
    let trace_empty = Signal::derive(move || trace.get().is_empty());

    view! {
        <div class="app-container">
            // Two-panel converter layout
            <div class="converter-layout">
                <CodeEditor
                    value=input
                    on_change=Callback::new(move |val: String| set_input.set(val))
                    placeholder=move || direction.get().input_placeholder()
                    label=move || direction.get().input_label()
                />

                <div class="controls">
                    <DirectionToggle direction=direction on_toggle=set_direction />
                    <button
                        class="btn btn-primary"
                        on:click=move |_| convert_action.dispatch(())
                        disabled=move || is_converting.get()
                    >
                        {move || if is_converting.get() { "Converting..." } else { "Convert" }}
                    </button>
                </div>

                <CodeEditor
                    value=output
                    readonly=true
                    placeholder="Output will appear here"
                    label=move || direction.get().output_label()
                />
            </div>

            // Collapsible detail panels
            <CollapsiblePanel
                title="Segment Tree"
                badge=segment_badge
                disabled=segments_empty
            >
                <SegmentTreeView segments=segments.into() />
            </CollapsiblePanel>

            <CollapsiblePanel
                title="Mapping Trace"
                badge=trace_badge
                disabled=trace_empty
            >
                <TraceTable entries=trace.into() />
            </CollapsiblePanel>

            <CollapsiblePanel
                title="Errors"
                badge=error_badge
                initially_open=true
            >
                <ErrorList errors=errors.into() />
            </CollapsiblePanel>
        </div>
    }
}
```

### Step 3: Write `crates/automapper-web/src/pages/coordinators.rs`

```rust
//! Coordinators page — lists available coordinators and their supported format versions.

use leptos::prelude::*;

use crate::api_client;
use crate::types::CoordinatorInfo;

/// Coordinators listing page.
#[component]
pub fn CoordinatorsPage() -> impl IntoView {
    let coordinators = LocalResource::new(|| async { api_client::list_coordinators().await });

    view! {
        <div class="app-container">
            <h2 style="margin-bottom: 1rem; color: var(--color-primary);">"Available Coordinators"</h2>

            <Suspense fallback=move || view! {
                <div class="loading">
                    <div class="spinner"></div>
                    "Loading coordinators..."
                </div>
            }>
                {move || {
                    coordinators.get().map(|result| match result {
                        Ok(coords) => {
                            if coords.is_empty() {
                                view! {
                                    <p style="color: var(--color-text-muted);">
                                        "No coordinators available."
                                    </p>
                                }
                                .into_any()
                            } else {
                                view! {
                                    <div class="coordinator-grid">
                                        {coords
                                            .into_iter()
                                            .map(|coord| {
                                                view! { <CoordinatorCard coordinator=coord /> }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                }
                                .into_any()
                            }
                        }
                        Err(e) => {
                            view! {
                                <p style="color: var(--color-error);">
                                    "Failed to load coordinators: " {e}
                                </p>
                            }
                            .into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Card for a single coordinator.
#[component]
fn CoordinatorCard(coordinator: CoordinatorInfo) -> impl IntoView {
    view! {
        <div class="coordinator-card">
            <h3>{coordinator.message_type}</h3>
            <p class="description">{coordinator.description}</p>
            <div class="versions">
                {coordinator
                    .supported_versions
                    .iter()
                    .map(|v| {
                        view! { <span class="version-badge">{v.clone()}</span> }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}
```

### Step 4: Run `cargo check -p automapper-web`

```bash
cargo check -p automapper-web
```

Expected: compiles.

### Step 5: Commit

```bash
git add -A
git commit -m "feat(web): implement ConverterPage and CoordinatorsPage with full API integration"
```

---

## Task 5: Implement App Component with Router and Entry Point

### Step 1: Write `crates/automapper-web/src/app.rs`

```rust
//! Root application component with router.

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::pages::converter::ConverterPage;
use crate::pages::coordinators::CoordinatorsPage;

/// Root application component.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="navbar">
                <h1>"Automapper"</h1>
                <nav>
                    <A href="/">"Converter"</A>
                    <A href="/coordinators">"Coordinators"</A>
                </nav>
            </div>

            <main>
                <Routes fallback=|| view! { <p>"Page not found."</p> }>
                    <Route path=path!("/") view=ConverterPage />
                    <Route path=path!("/coordinators") view=CoordinatorsPage />
                </Routes>
            </main>
        </Router>
    }
}
```

### Step 2: Write `crates/automapper-web/src/lib.rs`

```rust
//! Automapper WASM frontend built with Leptos.
//!
//! Provides a web UI for EDIFACT <-> BO4E conversion, EDIFACT inspection,
//! and coordinator discovery.

pub mod api_client;
pub mod app;
pub mod components;
pub mod pages;
pub mod types;
```

### Step 3: Write `crates/automapper-web/src/main.rs`

```rust
//! Entry point for the Leptos WASM frontend.
//!
//! Mounts the App component into the DOM.

use leptos::prelude::*;

use automapper_web::app::App;

fn main() {
    // Set up panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();

    // Initialize console logging
    _ = console_log::init_with_level(log::Level::Debug);

    log::info!("automapper-web starting");

    mount_to_body(App);
}
```

### Step 4: Run `cargo check -p automapper-web`

```bash
cargo check -p automapper-web
```

Expected: compiles on native target.

### Step 5: Install wasm32 target and check WASM compilation

```bash
rustup target add wasm32-unknown-unknown
cargo check -p automapper-web --target wasm32-unknown-unknown
```

Expected: compiles for WASM target.

### Step 6: Commit

```bash
git add -A
git commit -m "feat(web): implement App component with router and WASM entry point"
```

---

## Task 6: Configure Static File Serving in automapper-api

### Step 1: Create the placeholder static directory

```bash
mkdir -p static
```

Write `static/index.html` as a fallback page:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Automapper</title>
    <style>
        body {
            background: #1e1e2e;
            color: #f8f8f2;
            font-family: system-ui, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            text-align: center;
        }
        h1 { color: #8be9fd; }
        p { color: #6272a4; margin-top: 1rem; }
        code { color: #bd93f9; }
    </style>
</head>
<body>
    <div>
        <h1>Automapper API</h1>
        <p>The API is running. Build the frontend to see the web UI.</p>
        <p>
            Build the frontend: <code>trunk build crates/automapper-web/index.html --release</code><br/>
            Then copy <code>crates/automapper-web/dist/*</code> to <code>static/</code>
        </p>
        <p>API docs: <a href="/health" style="color: #8be9fd;">/health</a></p>
    </div>
</body>
</html>
```

### Step 2: Add a build script note to the README

Create `crates/automapper-web/BUILD.md`:

No — do not create documentation files unless explicitly requested. Instead, add a comment to `Cargo.toml`.

Add a comment to `crates/automapper-web/Cargo.toml`:

```toml
# Build instructions:
# 1. Install trunk: cargo install trunk
# 2. Build WASM: trunk build --release (from this directory)
# 3. Copy dist/* to ../../static/ for the API server to serve
# 4. Or run `trunk serve` for dev mode with hot-reload (proxies API calls to localhost:8080)
```

### Step 3: Run full check suite for both crates

```bash
cargo check -p automapper-api
cargo check -p automapper-web
cargo test -p automapper-api
```

Expected: all pass.

### Step 4: Commit

```bash
git add -A
git commit -m "feat(web): add placeholder static/index.html and configure single-binary deployment path"
```

---

## Task 7: Add WASM Build Verification Test

### Step 1: Write `crates/automapper-web/tests/wasm_build.rs`

This is a native-target test that verifies types compile and serialize correctly (not a WASM test, but validates the contract types used by the frontend):

```rust
//! Tests for frontend types and API client contracts.
//!
//! These tests run on the native target (not WASM) to verify
//! serialization and deserialization of API types.

use automapper_web::types::*;

#[test]
fn test_direction_toggle() {
    let dir = Direction::EdifactToBo4e;
    assert_eq!(dir.toggle(), Direction::Bo4eToEdifact);
    assert_eq!(dir.toggle().toggle(), Direction::EdifactToBo4e);
}

#[test]
fn test_direction_labels() {
    let dir = Direction::EdifactToBo4e;
    assert_eq!(dir.input_label(), "EDIFACT");
    assert_eq!(dir.output_label(), "BO4E JSON");
    assert_eq!(dir.api_path(), "/api/v1/convert/edifact-to-bo4e");

    let dir = Direction::Bo4eToEdifact;
    assert_eq!(dir.input_label(), "BO4E JSON");
    assert_eq!(dir.output_label(), "EDIFACT");
    assert_eq!(dir.api_path(), "/api/v1/convert/bo4e-to-edifact");
}

#[test]
fn test_convert_request_serialization() {
    let req = ConvertRequest {
        content: "UNH+1+UTILMD'".to_string(),
        format_version: Some("FV2504".to_string()),
        include_trace: true,
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["content"], "UNH+1+UTILMD'");
    assert_eq!(json["format_version"], "FV2504");
    assert_eq!(json["include_trace"], true);
}

#[test]
fn test_convert_request_omits_none_format_version() {
    let req = ConvertRequest {
        content: "test".to_string(),
        format_version: None,
        include_trace: false,
    };

    let json = serde_json::to_value(&req).unwrap();
    assert!(json.get("format_version").is_none());
}

#[test]
fn test_convert_response_deserialization() {
    let json = r#"{
        "success": true,
        "result": "{}",
        "trace": [
            {
                "mapper": "UtilmdCoordinator",
                "source_segment": "UNH",
                "target_path": "transactions",
                "value": "1",
                "note": null
            }
        ],
        "errors": [],
        "duration_ms": 42.5
    }"#;

    let resp: ConvertResponse = serde_json::from_str(json).unwrap();
    assert!(resp.success);
    assert_eq!(resp.result, Some("{}".to_string()));
    assert_eq!(resp.trace.len(), 1);
    assert_eq!(resp.trace[0].mapper, "UtilmdCoordinator");
    assert_eq!(resp.duration_ms, 42.5);
}

#[test]
fn test_inspect_response_deserialization() {
    let json = r#"{
        "segments": [
            {
                "tag": "UNH",
                "line_number": 1,
                "raw_content": "UNH+1+UTILMD:D:11A:UN",
                "elements": [
                    {
                        "position": 1,
                        "value": "1",
                        "components": null
                    }
                ],
                "children": null
            }
        ],
        "segment_count": 1,
        "message_type": "UTILMD",
        "format_version": null
    }"#;

    let resp: InspectResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.segment_count, 1);
    assert_eq!(resp.segments[0].tag, "UNH");
    assert_eq!(resp.message_type, Some("UTILMD".to_string()));
}

#[test]
fn test_coordinator_info_deserialization() {
    let json = r#"{
        "message_type": "UTILMD",
        "description": "UTILMD coordinator",
        "supported_versions": ["FV2504", "FV2510"]
    }"#;

    let info: CoordinatorInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.message_type, "UTILMD");
    assert_eq!(info.supported_versions.len(), 2);
}

#[test]
fn test_health_response_deserialization() {
    let json = r#"{
        "healthy": true,
        "version": "0.1.0",
        "available_coordinators": ["UTILMD"],
        "uptime_seconds": 123.456
    }"#;

    let health: HealthResponse = serde_json::from_str(json).unwrap();
    assert!(health.healthy);
    assert_eq!(health.version, "0.1.0");
}

#[test]
fn test_error_entry_deserialization() {
    let json = r#"{
        "code": "PARSE_ERROR",
        "message": "unterminated segment",
        "location": "byte 42",
        "severity": "error"
    }"#;

    let err: ApiErrorEntry = serde_json::from_str(json).unwrap();
    assert_eq!(err.code, "PARSE_ERROR");
    assert_eq!(err.severity, "error");
    assert_eq!(err.location, Some("byte 42".to_string()));
}
```

### Step 2: Run web crate tests

```bash
cargo test -p automapper-web
```

Expected: all 8 tests pass.

### Step 3: Run clippy on both crates

```bash
cargo clippy -p automapper-api -p automapper-web -- -D warnings
```

Expected: no warnings.

### Step 4: Commit

```bash
git add -A
git commit -m "test(web): add contract type tests for frontend API client types"
```

---

## Task 8: Final Verification and Cleanup

### Step 1: Run full check suite for both crates

```bash
cargo check -p automapper-api -p automapper-web
cargo test -p automapper-api -p automapper-web
cargo clippy -p automapper-api -p automapper-web -- -D warnings
cargo fmt -p automapper-api -p automapper-web -- --check
```

Expected: all pass cleanly.

### Step 2: Verify WASM compilation

```bash
cargo check -p automapper-web --target wasm32-unknown-unknown
```

Expected: compiles.

### Step 3: Verify the component summary

| Component | File | Status |
|-----------|------|--------|
| `App` | `app.rs` | Implemented |
| `ConverterPage` | `pages/converter.rs` | Implemented |
| `CoordinatorsPage` | `pages/coordinators.rs` | Implemented |
| `CodeEditor` | `components/code_editor.rs` | Implemented |
| `DirectionToggle` | `components/direction_toggle.rs` | Implemented |
| `CollapsiblePanel` | `components/collapsible_panel.rs` | Implemented |
| `SegmentTreeView` | `components/segment_tree.rs` | Implemented |
| `TraceTable` | `components/trace_table.rs` | Implemented |
| `ErrorList` | `components/error_list.rs` | Implemented |
| API client | `api_client.rs` | Implemented |
| Types | `types.rs` | Implemented, tested |

### Step 4: Verify the page routes

| Route | Component | Description |
|-------|-----------|-------------|
| `/` | `ConverterPage` | Two-panel converter with segment tree, trace, errors |
| `/coordinators` | `CoordinatorsPage` | Card grid of available coordinators |

### Step 5: Commit

```bash
git add -A
git commit -m "chore(web): final cleanup and verification for Epic 3 — Leptos WASM frontend complete"
```
