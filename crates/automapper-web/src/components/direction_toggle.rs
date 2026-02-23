//! Direction label component showing the current conversion direction.
//!
//! Currently only EDIFACT -> BO4E is supported. This component displays
//! the direction as a static label. When BO4E -> EDIFACT is added via the
//! MIG reverse pipeline, this can be restored to a toggle.

use leptos::prelude::*;

use crate::types::Direction;

#[component]
pub fn DirectionLabel(
    /// Current direction.
    direction: Direction,
) -> impl IntoView {
    view! {
        <div class="direction-label">
            <span class="label">{direction.label()}</span>
        </div>
    }
}
