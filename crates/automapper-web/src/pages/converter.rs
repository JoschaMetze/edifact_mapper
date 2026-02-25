//! Converter page — two-panel layout with code editors and collapsible detail panels.

use leptos::prelude::*;

use crate::api_client;
use crate::components::code_editor::CodeEditor;
use crate::components::collapsible_panel::CollapsiblePanel;
use crate::components::error_list::ErrorList;
use crate::components::fixture_selector::FixtureSelector;
use crate::components::segment_tree::SegmentTreeView;
use crate::types::{extract_validation_issues, ApiErrorEntry, Direction, SegmentNode};

/// Main converter page.
#[component]
pub fn ConverterPage() -> impl IntoView {
    // Input/output state
    let (input, set_input) = signal(String::new());
    let (output, set_output) = signal(String::new());
    let direction = Direction::EdifactToBo4e;
    let (is_converting, set_is_converting) = signal(false);

    // Detail panel state
    let (segments, set_segments) = signal(Vec::<SegmentNode>::new());
    let (errors, set_errors) = signal(Vec::<ApiErrorEntry>::new());
    let (duration_ms, set_duration_ms) = signal(0.0_f64);
    let (validation_issues, set_validation_issues) = signal(Vec::<ApiErrorEntry>::new());
    let (is_validating, set_is_validating) = signal(false);
    let (_validation_duration_ms, set_validation_duration_ms) = signal(0.0_f64);

    // Convert action — uses v2 MIG-driven pipeline (EDIFACT -> BO4E only)
    let convert_action = Action::new_local(move |_: &()| {
        let input_val = input.get();

        async move {
            set_is_converting.set(true);
            set_errors.set(vec![]);
            set_validation_issues.set(vec![]);
            set_segments.set(vec![]);
            set_output.set(String::new());

            // Inspect EDIFACT input for segment tree visualization
            match api_client::inspect_edifact(&input_val).await {
                Ok(inspect) => {
                    set_segments.set(inspect.segments);
                }
                Err(e) => {
                    log::warn!("Inspect failed: {e}");
                }
            }

            // Convert via v2 MIG-driven pipeline
            match api_client::convert_v2(&input_val, "bo4e", "FV2504").await {
                Ok(resp) => {
                    set_duration_ms.set(resp.duration_ms);
                    let pretty = serde_json::to_string_pretty(&resp.result)
                        .unwrap_or_else(|_| resp.result.to_string());
                    set_output.set(pretty);

                    // Extract validation results if present
                    if let Some(ref validation) = resp.validation {
                        set_validation_issues.set(extract_validation_issues(validation));
                    } else {
                        set_validation_issues.set(vec![]);
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

    // Badge signals for collapsible panels
    let segment_badge = Signal::derive(move || {
        let count = segments.get().len();
        if count > 0 {
            format!("{count} segments")
        } else {
            String::new()
        }
    });

    let duration_badge = Signal::derive(move || {
        let ms = duration_ms.get();
        if ms > 0.0 {
            format!("{ms:.1}ms")
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

    let segments_empty = Signal::derive(move || segments.get().is_empty());

    view! {
        <div class="app-container">
            // Fixture selector bar
            <FixtureSelector on_load=Callback::new(move |(content, _dir): (String, Direction)| {
                set_input.set(content);
                set_output.set(String::new());
            }) />

            // Two-panel converter layout (EDIFACT -> BO4E)
            <div class="converter-layout">
                <CodeEditor
                    value=input
                    on_change=Callback::new(move |val: String| set_input.set(val))
                    placeholder=direction.input_placeholder()
                    label=direction.input_label()
                />

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

                <CodeEditor
                    value=output
                    readonly=true
                    placeholder="Output will appear here"
                    label=direction.output_label()
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
                title="Conversion"
                badge=duration_badge
            >
                <p>{move || format!("Duration: {:.1}ms", duration_ms.get())}</p>
            </CollapsiblePanel>

            <CollapsiblePanel
                title="Errors"
                badge=error_badge
                initially_open=true
            >
                <ErrorList errors=errors.into() />
            </CollapsiblePanel>

            <CollapsiblePanel
                title="Validation"
                badge=validation_badge
                initially_open=true
            >
                <ErrorList errors=validation_issues.into() />
            </CollapsiblePanel>
        </div>
    }
}
