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
