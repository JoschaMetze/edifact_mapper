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
                {move || {
                    entries.get()
                        .into_iter()
                        .map(|entry| {
                            let value = entry.value.clone().unwrap_or_default();
                            let note = entry.note.clone().unwrap_or_default();
                            view! {
                                <tr>
                                    <td>{entry.mapper.clone()}</td>
                                    <td>{entry.source_segment.clone()}</td>
                                    <td>{entry.target_path.clone()}</td>
                                    <td class="value-cell">{value}</td>
                                    <td class="note-cell">{note}</td>
                                </tr>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </tbody>
        </table>
    }
}
