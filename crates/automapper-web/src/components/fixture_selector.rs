//! Fixture selector component — dropdown to load test fixtures into the editor.

use leptos::prelude::*;

use crate::api_client;
use crate::types::{Direction, FixtureEntry};

/// A horizontal bar with a fixture dropdown and load buttons.
///
/// Fetches the fixture list on mount. "Load EDI" sets direction to EdifactToBo4e,
/// "Load BO4E" sets direction to Bo4eToEdifact, and both populate the input editor.
#[component]
pub fn FixtureSelector(
    /// Callback receiving `(content, direction)` when a fixture is loaded.
    on_load: Callback<(String, Direction)>,
) -> impl IntoView {
    let (fixtures, set_fixtures) = signal(Vec::<FixtureEntry>::new());
    let (selected_idx, set_selected_idx) = signal(Option::<usize>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // Fetch fixture list on mount.
    Effect::new(move || {
        leptos::task::spawn_local(async move {
            match api_client::list_fixtures("UTILMD", "FV2504").await {
                Ok(resp) => set_fixtures.set(resp.fixtures),
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let selected_fixture = Signal::derive(move || {
        selected_idx
            .get()
            .and_then(|i| fixtures.get().get(i).cloned())
    });

    let load_fixture = move |file_type: &'static str| {
        let fixture = selected_fixture.get();
        if let Some(fixture) = fixture {
            set_loading.set(true);
            set_error.set(None);
            leptos::task::spawn_local(async move {
                match api_client::get_fixture_content("UTILMD", "FV2504", &fixture.name, file_type)
                    .await
                {
                    Ok(content) => {
                        let direction = if file_type == "edi" {
                            Direction::EdifactToBo4e
                        } else {
                            Direction::Bo4eToEdifact
                        };
                        on_load.run((content, direction));
                    }
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="fixture-selector">
            <span class="fixture-label">"Fixture:"</span>
            <select
                class="fixture-select"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    if value.is_empty() {
                        set_selected_idx.set(None);
                    } else if let Ok(idx) = value.parse::<usize>() {
                        set_selected_idx.set(Some(idx));
                    }
                }
            >
                <option value="">"-- select a fixture --"</option>
                {move || {
                    fixtures.get().iter().enumerate().map(|(i, f)| {
                        let label = format!("[{}] {}", f.pid, f.name.split('_').skip(1).collect::<Vec<_>>().join("_"));
                        view! {
                            <option value={i.to_string()}>{label}</option>
                        }
                    }).collect::<Vec<_>>()
                }}
            </select>

            <button
                class="btn btn-small btn-load"
                disabled=move || selected_fixture.get().is_none_or(|f| !f.has_edi) || loading.get()
                on:click=move |_| load_fixture("edi")
            >
                "Load EDI"
            </button>

            <button
                class="btn btn-small btn-load"
                disabled=move || selected_fixture.get().is_none_or(|f| !f.has_bo4e) || loading.get()
                on:click=move |_| load_fixture("bo4e")
            >
                "Load BO4E"
            </button>

            {move || loading.get().then(|| view! { <span class="fixture-loading">"Loading..."</span> })}
            {move || error.get().map(|e| view! { <span class="fixture-error">{e}</span> })}
        </div>
    }
}
