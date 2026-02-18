//! Mapping trace table component.

use leptos::prelude::*;

use crate::types::TraceEntry;

/// Display mapping trace entries as a table.
#[component]
pub fn TraceTable(
    /// The trace entries to display.
    entries: Signal<Vec<TraceEntry>>,
) -> impl IntoView {
    view! {
        <table class="trace-table">
            <thead>
                <tr>
                    <th>"Mapper"</th>
                    <th>"Source Segment"</th>
                    <th>"Target Path"</th>
                    <th>"Value"</th>
                    <th>"Note"</th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || entries.get().into_iter().enumerate().collect::<Vec<_>>()
                    key=|(i, _)| *i
                    children=move |(_, entry)| {
                        let value = entry.value.clone().unwrap_or_default();
                        let note = entry.note.clone().unwrap_or_default();
                        view! {
                            <tr>
                                <td>{entry.mapper.clone()}</td>
                                <td><code>{entry.source_segment.clone()}</code></td>
                                <td><code>{entry.target_path.clone()}</code></td>
                                <td class="value-cell">{value}</td>
                                <td class="note-cell">{note}</td>
                            </tr>
                        }
                    }
                />
            </tbody>
        </table>
    }
}
