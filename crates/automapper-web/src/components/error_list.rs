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
                            .map(|err| {
                                let severity_class = match err.severity.as_str() {
                                    "warning" => "warning",
                                    "critical" => "critical",
                                    "info" => "info",
                                    _ => "error",
                                };
                                let severity_char = match err.severity.as_str() {
                                    "warning" => "W",
                                    "critical" => "!",
                                    "info" => "i",
                                    _ => "E",
                                };
                                let icon_class = format!("severity-icon {severity_class}");
                                let location = err.location.clone();
                                view! {
                                    <li class="error-item">
                                        <span class=icon_class>
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
