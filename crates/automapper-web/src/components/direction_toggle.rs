//! Direction toggle component for switching between EDIFACT->BO4E and BO4E->EDIFACT.

use leptos::prelude::*;

use crate::types::Direction;

#[component]
pub fn DirectionToggle(
    /// Current direction.
    direction: ReadSignal<Direction>,
    /// Callback when direction is toggled.
    on_toggle: WriteSignal<Direction>,
) -> impl IntoView {
    let toggle = move |_| {
        on_toggle.update(|d| *d = d.toggle());
    };

    view! {
        <div class="direction-toggle">
            <button class="btn btn-small" on:click=toggle>
                {move || match direction.get() {
                    Direction::EdifactToBo4e => "EDIFACT -> BO4E",
                    Direction::Bo4eToEdifact => "BO4E -> EDIFACT",
                }}
            </button>
            <span class="label">"click to swap"</span>
        </div>
    }
}
