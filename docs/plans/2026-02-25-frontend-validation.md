# Frontend Validation Integration

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Surface AHB validation results in the frontend — automatically on every convert, and via a standalone Validate button.

**Architecture:** Two paths feed the same `validation_issues` signal: (1) the existing convert call gains `?validate=true`, reading `response.validation`; (2) a new Validate button calls `POST /api/v2/validate` directly. Results render in a "Validation" `CollapsiblePanel` using the existing `ErrorList` component. No backend changes needed — both API surfaces already exist.

**Tech Stack:** Leptos (Rust WASM), gloo-net, existing `CollapsiblePanel`/`ErrorList` components.

---

## Task 1: Add frontend types for validation

**Files:**
- Modify: `crates/automapper-web/src/types.rs`
- Modify: `crates/automapper-web/tests/wasm_build.rs`

**Step 1: Write failing tests**

Add to `crates/automapper-web/tests/wasm_build.rs`:

```rust
#[test]
fn test_validate_v2_request_serialization() {
    let req = ValidateV2Request {
        input: "UNH+1+UTILMD'".to_string(),
        format_version: "FV2504".to_string(),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["input"], "UNH+1+UTILMD'");
    assert_eq!(json["format_version"], "FV2504");
    assert_eq!(json["level"], "full");
}

#[test]
fn test_validate_v2_response_deserialization() {
    let json = r#"{
        "report": {
            "message_type": "UTILMD",
            "pruefidentifikator": "55001",
            "format_version": "FV2504",
            "level": "Full",
            "issues": [
                {
                    "severity": "Error",
                    "category": "Ahb",
                    "code": "CONDITION_FAILED",
                    "message": "Condition [7] failed",
                    "segment_position": null,
                    "field_path": null,
                    "rule": null,
                    "actual_value": null,
                    "expected_value": null
                }
            ]
        },
        "duration_ms": 5.2
    }"#;

    let resp: ValidateV2Response = serde_json::from_str(json).unwrap();
    assert_eq!(resp.duration_ms, 5.2);
    assert!(resp.report.is_object());
}

#[test]
fn test_convert_v2_response_with_validation() {
    let json = r#"{
        "mode": "bo4e",
        "result": {"stammdaten": {}},
        "duration_ms": 12.3,
        "validation": {
            "message_type": "UTILMD",
            "pruefidentifikator": "55001",
            "format_version": "FV2504",
            "level": "Full",
            "issues": []
        }
    }"#;

    let resp: ConvertV2Response = serde_json::from_str(json).unwrap();
    assert!(resp.validation.is_some());
}

#[test]
fn test_extract_validation_issues_from_report() {
    let report: serde_json::Value = serde_json::json!({
        "message_type": "UTILMD",
        "issues": [
            {
                "severity": "Error",
                "category": "Ahb",
                "code": "CONDITION_FAILED",
                "message": "Condition [7] failed"
            },
            {
                "severity": "Warning",
                "category": "Structure",
                "code": "UNEXPECTED_SEGMENT",
                "message": "FTX at position 14"
            },
            {
                "severity": "Info",
                "category": "Ahb",
                "code": "CONDITION_UNKNOWN",
                "message": "External: sender_is_lf"
            }
        ]
    });

    let issues = extract_validation_issues(&report);
    assert_eq!(issues.len(), 3);
    assert_eq!(issues[0].severity, "error");
    assert_eq!(issues[0].code, "CONDITION_FAILED");
    assert_eq!(issues[1].severity, "warning");
    assert_eq!(issues[1].code, "UNEXPECTED_SEGMENT");
    assert_eq!(issues[2].severity, "info");
    assert_eq!(issues[2].code, "CONDITION_UNKNOWN");
}

#[test]
fn test_extract_validation_issues_empty_report() {
    let report: serde_json::Value = serde_json::json!({
        "message_type": "UTILMD",
        "issues": []
    });

    let issues = extract_validation_issues(&report);
    assert!(issues.is_empty());
}
```

Add `ValidateV2Request`, `ValidateV2Response`, and `extract_validation_issues` to the import at the top of the test file.

**Step 2: Run tests to verify they fail**

Run: `cargo test -p automapper-web`
Expected: Compile error — `ValidateV2Request` not found.

**Step 3: Add types to `types.rs`**

Add to `crates/automapper-web/src/types.rs`:

```rust
/// V2 validation request.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateV2Request {
    pub input: String,
    pub format_version: String,
    /// Always "full" — hardcoded in serialization.
    #[serde(default = "default_level")]
    pub level: String,
}

fn default_level() -> String {
    "full".to_string()
}

/// V2 validation response.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateV2Response {
    pub report: serde_json::Value,
    pub duration_ms: f64,
}
```

Also update `ConvertV2Response` to include the optional `validation` field:

```rust
/// V2 conversion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertV2Response {
    pub mode: String,
    pub result: serde_json::Value,
    pub duration_ms: f64,
    /// Validation report (present when ?validate=true).
    pub validation: Option<serde_json::Value>,
}
```

Add the `extract_validation_issues` helper function:

```rust
/// Extract validation issues from a ValidationReport JSON value into ApiErrorEntry list.
///
/// Maps backend Severity (Error/Warning/Info) to lowercase strings for ErrorList rendering.
pub fn extract_validation_issues(report: &serde_json::Value) -> Vec<ApiErrorEntry> {
    let Some(issues) = report.get("issues").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    issues
        .iter()
        .filter_map(|issue| {
            let severity = issue.get("severity")?.as_str()?;
            let code = issue.get("code")?.as_str()?;
            let message = issue.get("message")?.as_str()?;

            // Map backend Severity enum (PascalCase) to lowercase for ErrorList
            let severity_lower = match severity {
                "Error" => "error",
                "Warning" => "warning",
                "Info" => "info",
                _ => "error",
            };

            // Build location string from optional fields
            let location = issue
                .get("field_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(ApiErrorEntry {
                code: code.to_string(),
                message: message.to_string(),
                location,
                severity: severity_lower.to_string(),
            })
        })
        .collect()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p automapper-web`
Expected: All tests pass (existing + 5 new).

**Step 5: Commit**

```bash
git add crates/automapper-web/src/types.rs crates/automapper-web/tests/wasm_build.rs
git commit -m "feat(web): add validation types and report-to-error extraction"
```

---

## Task 2: Add validate_v2 to API client

**Files:**
- Modify: `crates/automapper-web/src/api_client.rs`

**Step 1: Add `validate_v2` function**

Add to `crates/automapper-web/src/api_client.rs`, after the `convert_v2` function. Also update the import to include `ValidateV2Request` and `ValidateV2Response`:

```rust
use crate::types::{
    ConvertV2Request, ConvertV2Response, CoordinatorInfo, FixtureListResponse, HealthResponse,
    InspectRequest, InspectResponse, ValidateV2Request, ValidateV2Response,
};
```

Add the function:

```rust
/// Validate EDIFACT against AHB rules using the v2 pipeline.
pub async fn validate_v2(
    input: &str,
    format_version: &str,
) -> Result<ValidateV2Response, String> {
    let request_body = ValidateV2Request {
        input: input.to_string(),
        format_version: format_version.to_string(),
        level: "full".to_string(),
    };

    let url = format!("{API_BASE}/api/v2/validate");

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("failed to serialize request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if response.ok() {
        response
            .json::<ValidateV2Response>()
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
```

**Step 2: Update `convert_v2` to pass `?validate=true`**

Change the URL line in the existing `convert_v2` function from:

```rust
let url = format!("{API_BASE}/api/v2/convert");
```

to:

```rust
let url = format!("{API_BASE}/api/v2/convert?validate=true");
```

**Step 3: Verify compilation**

Run: `cargo check -p automapper-web`
Expected: Compiles. (Cannot run API client tests without a server, but type-checking confirms the contract.)

**Step 4: Commit**

```bash
git add crates/automapper-web/src/api_client.rs
git commit -m "feat(web): add validate_v2 API client and enable validation on convert"
```

---

## Task 3: Add validation panel and Validate button to converter page

**Files:**
- Modify: `crates/automapper-web/src/pages/converter.rs`

**Step 1: Add validation signal and import**

At the top of `converter.rs`, update the types import:

```rust
use crate::types::{ApiErrorEntry, Direction, SegmentNode, extract_validation_issues};
```

Inside `ConverterPage`, add a new signal after the existing `duration_ms` signal (line 25):

```rust
let (validation_issues, set_validation_issues) = signal(Vec::<ApiErrorEntry>::new());
let (is_validating, set_is_validating) = signal(false);
let (validation_duration_ms, set_validation_duration_ms) = signal(0.0_f64);
```

**Step 2: Update the convert action to extract validation**

In the convert action's success handler (the `Ok(resp)` branch around line 49), add after `set_output.set(pretty)`:

```rust
// Extract validation results if present
if let Some(ref validation) = resp.validation {
    set_validation_issues.set(extract_validation_issues(validation));
} else {
    set_validation_issues.set(vec![]);
}
```

Also clear validation issues at the start of the convert action (after the existing `set_errors.set(vec![])` on line 33):

```rust
set_validation_issues.set(vec![]);
```

**Step 3: Add the validate action**

After the `convert_action` definition (after line 67), add:

```rust
// Standalone validate action — calls /api/v2/validate directly
let validate_action = Action::new_local(move |_: &()| {
    let input_val = input.get();

    async move {
        set_is_validating.set(true);
        set_validation_issues.set(vec![]);

        match api_client::validate_v2(&input_val, "FV2504").await {
            Ok(resp) => {
                set_validation_duration_ms.set(resp.duration_ms);
                set_validation_issues.set(extract_validation_issues(&resp.report));
            }
            Err(e) => {
                set_validation_issues.set(vec![ApiErrorEntry {
                    code: "VALIDATION_ERROR".to_string(),
                    message: e,
                    location: None,
                    severity: "error".to_string(),
                }]);
            }
        }

        set_is_validating.set(false);
    }
});
```

**Step 4: Add validation badge signal**

After the existing `error_badge` signal (around line 95), add:

```rust
let validation_badge = Signal::derive(move || {
    let issues = validation_issues.get();
    if issues.is_empty() {
        String::new()
    } else {
        let error_count = issues.iter().filter(|i| i.severity == "error").count();
        if error_count > 0 {
            format!("{} errors", error_count)
        } else {
            format!("{} issues", issues.len())
        }
    }
});
```

**Step 5: Add the Validate button to the template**

In the `view!` macro, find the controls div (around line 116-125):

```rust
<div class="controls">
    <span class="direction-label">{direction.label()}</span>
    <button
        class="btn btn-primary"
        on:click=move |_| { convert_action.dispatch(()); }
        disabled=move || is_converting.get()
    >
        {move || if is_converting.get() { "Converting..." } else { "Convert" }}
    </button>
</div>
```

Replace with:

```rust
<div class="controls">
    <span class="direction-label">{direction.label()}</span>
    <button
        class="btn btn-primary"
        on:click=move |_| { convert_action.dispatch(()); }
        disabled=move || is_converting.get()
    >
        {move || if is_converting.get() { "Converting..." } else { "Convert" }}
    </button>
    <button
        class="btn btn-secondary"
        on:click=move |_| { validate_action.dispatch(()); }
        disabled=move || is_validating.get()
    >
        {move || if is_validating.get() { "Validating..." } else { "Validate" }}
    </button>
</div>
```

**Step 6: Add the Validation collapsible panel**

After the "Errors" `CollapsiblePanel` (around line 157), add:

```rust
<CollapsiblePanel
    title="Validation"
    badge=validation_badge
    initially_open=true
>
    <ErrorList errors=validation_issues.into() />
</CollapsiblePanel>
```

**Step 7: Verify compilation**

Run: `cargo check -p automapper-web`
Expected: Compiles.

**Step 8: Commit**

```bash
git add crates/automapper-web/src/pages/converter.rs
git commit -m "feat(web): add Validate button and validation results panel"
```

---

## Task 4: Style the Validate button

**Files:**
- Modify: `crates/automapper-web/style/main.css`

**Step 1: Add btn-secondary style**

The existing CSS has `.btn-primary` for the Convert button. Find the `.btn` and `.btn-primary` styles and add a `.btn-secondary` style after them. The secondary button should use a muted color to visually distinguish it from the primary Convert action.

Add:

```css
.btn-secondary {
    background-color: #44475a;
    color: #f8f8f2;
}

.btn-secondary:hover:not(:disabled) {
    background-color: #6272a4;
}
```

This uses the Dracula theme's "current line" (#44475a) and "comment" (#6272a4) colors, keeping the secondary button visually subordinate to the primary cyan button.

**Step 2: Verify**

Run: `cargo check -p automapper-web`
Expected: Compiles (CSS changes don't affect compilation, but ensures nothing else broke).

**Step 3: Commit**

```bash
git add crates/automapper-web/style/main.css
git commit -m "style(web): add btn-secondary style for Validate button"
```

---

## Task 5: Final verification

**Step 1: Run all web tests**

Run: `cargo test -p automapper-web`
Expected: All tests pass (7 existing + 5 new = 12).

**Step 2: Type-check the full workspace**

Run: `cargo check --workspace`
Expected: Clean.

**Step 3: Lint**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings.

**Step 4: Format check**

Run: `cargo fmt --all -- --check`
Expected: No diffs.

---

## Summary

| # | Task | Files | Tests |
|---|------|-------|-------|
| 1 | Validation types + extraction helper | `types.rs`, `wasm_build.rs` | 5 new |
| 2 | API client `validate_v2` + `?validate=true` | `api_client.rs` | 0 (type-check only) |
| 3 | Validate button + validation panel | `converter.rs` | 0 (UI, type-check only) |
| 4 | Button styling | `main.css` | 0 |
| 5 | Final verification | — | Full suite |

**Not in scope:**
- Validation level selector (always "full")
- External condition overrides UI
- New components (reuses ErrorList + CollapsiblePanel)
- Backend changes (API already complete)
