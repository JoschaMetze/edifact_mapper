//! Converter page — two-panel layout with code editors and collapsible detail panels.

use leptos::prelude::*;

use crate::api_client;
use crate::components::code_editor::CodeEditor;
use crate::components::collapsible_panel::CollapsiblePanel;
use crate::components::error_list::ErrorList;
use crate::components::fixture_selector::FixtureSelector;
use crate::components::segment_tree::SegmentTreeView;
use crate::types::{
    extract_validation_issues, ApiErrorEntry, Direction, GeneratedResponsePayload,
    ResponseGenerationOptions, SegmentNode,
};

/// Main converter page.
#[component]
pub fn ConverterPage() -> impl IntoView {
    // Input/output state
    let (input, set_input) = signal(String::new());
    let (output, set_output) = signal(String::new());
    let direction = Direction::EdifactToBo4e;
    let (is_converting, set_is_converting) = signal(false);
    let (format_version, set_format_version) = signal("FV2504".to_string());

    // Detail panel state
    let (segments, set_segments) = signal(Vec::<SegmentNode>::new());
    let (errors, set_errors) = signal(Vec::<ApiErrorEntry>::new());
    let (duration_ms, set_duration_ms) = signal(0.0_f64);
    let (validation_issues, set_validation_issues) = signal(Vec::<ApiErrorEntry>::new());
    let (is_validating, set_is_validating) = signal(false);
    let (_validation_duration_ms, set_validation_duration_ms) = signal(0.0_f64);

    // Response generation state
    let (gen_response_enabled, set_gen_response_enabled) = signal(false);
    let (response_type_sel, set_response_type_sel) = signal("auto".to_string());
    let (response_format_sel, set_response_format_sel) = signal("bo4e".to_string());
    let (response_message, set_response_message) = signal(None::<GeneratedResponsePayload>);

    // Convert action — uses v2 MIG-driven pipeline (EDIFACT -> BO4E only)
    let convert_action = Action::new_local(move |_: &()| {
        let input_val = input.get();
        let fv = format_version.get();

        async move {
            set_is_converting.set(true);
            set_errors.set(vec![]);
            set_validation_issues.set(vec![]);
            set_segments.set(vec![]);
            set_output.set(String::new());
            set_response_message.set(None);

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
            match api_client::convert_v2(&input_val, "bo4e", &fv).await {
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
        let fv = format_version.get();
        let gen_enabled = gen_response_enabled.get();
        let resp_type = response_type_sel.get();
        let resp_format = response_format_sel.get();

        async move {
            set_is_validating.set(true);
            set_validation_issues.set(vec![]);
            set_response_message.set(None);

            let gen_opts = if gen_enabled {
                Some(ResponseGenerationOptions {
                    response_type: if resp_type == "auto" {
                        None
                    } else {
                        Some(resp_type)
                    },
                    format: Some(resp_format),
                })
            } else {
                None
            };

            match api_client::validate_v2(&input_val, &fv, gen_opts).await {
                Ok(resp) => {
                    set_validation_duration_ms.set(resp.duration_ms);
                    set_validation_issues.set(extract_validation_issues(&resp.report));
                    set_response_message.set(resp.response_message);
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

    let response_badge = Signal::derive(move || match response_message.get() {
        Some(ref msg) => msg.message_type.clone(),
        None => String::new(),
    });

    let response_empty = Signal::derive(move || response_message.get().is_none());

    let segments_empty = Signal::derive(move || segments.get().is_empty());

    view! {
        <div class="app-container">
            // Fixture selector bar
            <FixtureSelector on_load=Callback::new(move |(content, _dir, fv): (String, Direction, String)| {
                set_input.set(content);
                set_output.set(String::new());
                set_format_version.set(fv);
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
                    <div class="response-controls">
                        <label class="response-checkbox">
                            <input
                                type="checkbox"
                                on:change=move |ev| {
                                    let checked = event_target::<web_sys::HtmlInputElement>(&ev).checked();
                                    set_gen_response_enabled.set(checked);
                                }
                            />
                            " Response"
                        </label>
                        {move || gen_response_enabled.get().then(|| view! {
                            <select
                                class="fixture-select"
                                on:change=move |ev| {
                                    set_response_type_sel.set(event_target_value(&ev));
                                }
                            >
                                <option value="auto" selected=true>"Auto"</option>
                                <option value="aperak">"APERAK"</option>
                                <option value="contrl">"CONTRL"</option>
                            </select>
                            <select
                                class="fixture-select"
                                on:change=move |ev| {
                                    set_response_format_sel.set(event_target_value(&ev));
                                }
                            >
                                <option value="bo4e" selected=true>"BO4E"</option>
                                <option value="edifact">"EDIFACT"</option>
                            </select>
                        })}
                    </div>
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

            <CollapsiblePanel
                title="Response Message"
                badge=response_badge
                disabled=response_empty
            >
                {move || {
                    match response_message.get() {
                        Some(msg) => {
                            let has_bo4e = msg.bo4e.is_some();
                            let content = if let Some(ref bo4e) = msg.bo4e {
                                serde_json::to_string_pretty(bo4e)
                                    .unwrap_or_else(|_| bo4e.to_string())
                            } else if let Some(ref edi) = msg.edifact {
                                edi.clone()
                            } else {
                                String::new()
                            };
                            let format_label = if has_bo4e { "BO4E JSON" } else { "EDIFACT" };
                            view! {
                                <div class="response-content">
                                    <div class="response-meta">
                                        <span class="badge">{msg.message_type.clone()}</span>
                                        <span class="response-format-label">{format_label}</span>
                                    </div>
                                    <textarea
                                        class="response-textarea"
                                        readonly=true
                                        prop:value=content
                                    />
                                </div>
                            }.into_any()
                        }
                        None => {
                            view! {
                                <p class="no-errors">"No response message generated"</p>
                            }.into_any()
                        }
                    }
                }}
            </CollapsiblePanel>
        </div>
    }
}
