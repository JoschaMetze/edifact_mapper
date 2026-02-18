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
                    coordinators.get().map(|result| match &*result {
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
                                            .iter()
                                            .map(|coord| {
                                                view! { <CoordinatorCard coordinator=coord.clone() /> }
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
                                    "Failed to load coordinators: " {e.clone()}
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
