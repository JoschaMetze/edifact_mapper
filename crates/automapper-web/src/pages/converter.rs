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
    let convert_action = Action::new_local(move |_: &()| {
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
                    placeholder=direction.get_untracked().input_placeholder()
                    label=direction.get_untracked().input_label()
                />

                <div class="controls">
                    <DirectionToggle direction=direction on_toggle=set_direction />
                    <button
                        class="btn btn-primary"
                        on:click=move |_| { convert_action.dispatch(()); }
                        disabled=move || is_converting.get()
                    >
                        {move || if is_converting.get() { "Converting..." } else { "Convert" }}
                    </button>
                </div>

                <CodeEditor
                    value=output
                    readonly=true
                    placeholder="Output will appear here"
                    label=direction.get_untracked().output_label()
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
