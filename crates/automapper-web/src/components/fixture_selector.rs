//! Fixture selector component — cascading dropdowns to browse all message types,
//! format versions, and fixture files.

use leptos::prelude::*;

use crate::api_client;
use crate::types::{Direction, FixtureCatalogEntry, FixtureEntry};

/// A horizontal bar with three cascading dropdowns:
/// Message Type → Format Version → Fixture, plus a "Load EDI" button.
///
/// Fetches the fixture catalog on mount. Changing message type auto-selects
/// the first (or default) format version. Changing format version fetches
/// the fixture list for that combination.
#[component]
pub fn FixtureSelector(
    /// Callback receiving `(content, direction, format_version)` when a fixture is loaded.
    on_load: Callback<(String, Direction, String)>,
) -> impl IntoView {
    let (catalog, set_catalog) = signal(Vec::<FixtureCatalogEntry>::new());
    let (selected_msg_type, set_selected_msg_type) = signal("UTILMD".to_string());
    let (selected_fv, set_selected_fv) = signal("FV2504".to_string());
    let (fixtures, set_fixtures) = signal(Vec::<FixtureEntry>::new());
    let (selected_idx, set_selected_idx) = signal(Option::<usize>::None);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // Track which (msg_type, fv) we last fetched fixtures for, to avoid redundant fetches.
    let (last_fetched, set_last_fetched) = signal(("".to_string(), "".to_string()));

    // Fetch catalog on mount.
    Effect::new(move || {
        leptos::task::spawn_local(async move {
            match api_client::list_fixture_catalog().await {
                Ok(resp) => set_catalog.set(resp.message_types),
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    // Derived: format versions available for the selected message type.
    let available_fvs = Signal::derive(move || {
        let msg = selected_msg_type.get();
        catalog
            .get()
            .iter()
            .find(|e| e.message_type == msg)
            .map(|e| e.format_versions.clone())
            .unwrap_or_default()
    });

    // Fetch fixtures when (selected_msg_type, selected_fv) changes.
    Effect::new(move || {
        let msg = selected_msg_type.get();
        let fv = selected_fv.get();

        // Skip if empty or same as last fetch.
        if msg.is_empty() || fv.is_empty() {
            return;
        }
        let (last_msg, last_fv) = last_fetched.get();
        if last_msg == msg && last_fv == fv {
            return;
        }
        set_last_fetched.set((msg.clone(), fv.clone()));

        set_fixtures.set(vec![]);
        set_selected_idx.set(None);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match api_client::list_fixtures(&msg, &fv).await {
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
        let msg = selected_msg_type.get();
        let fv = selected_fv.get();
        if let Some(fixture) = fixture {
            set_loading.set(true);
            set_error.set(None);
            leptos::task::spawn_local(async move {
                match api_client::get_fixture_content(&msg, &fv, &fixture.name, file_type).await {
                    Ok(content) => {
                        on_load.run((content, Direction::EdifactToBo4e, fv));
                    }
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="fixture-selector">
            <span class="fixture-label">"Browse:"</span>

            // Message Type dropdown
            <select
                class="fixture-select fixture-select-narrow"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    set_selected_msg_type.set(value.clone());

                    // Auto-select FV2504 if available, otherwise first FV.
                    let fvs = catalog.get();
                    let entry = fvs.iter().find(|e| e.message_type == value);
                    if let Some(entry) = entry {
                        let default_fv = entry
                            .format_versions
                            .iter()
                            .find(|fv| fv.format_version == "FV2504")
                            .or(entry.format_versions.first());
                        if let Some(fv) = default_fv {
                            set_selected_fv.set(fv.format_version.clone());
                        }
                    }
                }
            >
                {move || {
                    let current = selected_msg_type.get();
                    catalog.get().iter().map(|entry| {
                        let total: usize = entry.format_versions.iter().map(|fv| fv.fixture_count).sum();
                        let label = format!("{} ({})", entry.message_type, total);
                        let selected = entry.message_type == current;
                        view! {
                            <option value={entry.message_type.clone()} selected=selected>{label}</option>
                        }
                    }).collect::<Vec<_>>()
                }}
            </select>

            // Format Version dropdown
            <select
                class="fixture-select fixture-select-narrow"
                on:change=move |ev| {
                    let value = event_target_value(&ev);
                    set_selected_fv.set(value);
                }
            >
                {move || {
                    let current = selected_fv.get();
                    available_fvs.get().iter().map(|fv| {
                        let label = format!("{} ({})", fv.format_version, fv.fixture_count);
                        let selected = fv.format_version == current;
                        view! {
                            <option value={fv.format_version.clone()} selected=selected>{label}</option>
                        }
                    }).collect::<Vec<_>>()
                }}
            </select>

            // Fixture dropdown
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
                <option value="">{move || {
                    let count = fixtures.get().len();
                    if count > 0 {
                        format!("-- select fixture ({count}) --")
                    } else {
                        "-- select fixture --".to_string()
                    }
                }}</option>
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

            {move || loading.get().then(|| view! { <span class="fixture-loading">"Loading..."</span> })}
            {move || error.get().map(|e| view! { <span class="fixture-error">{e}</span> })}
        </div>
    }
}
