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
