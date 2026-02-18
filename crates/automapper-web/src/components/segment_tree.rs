//! Recursive segment tree view component.

use leptos::prelude::*;

use crate::types::SegmentNode;

/// Display a list of segment nodes as a tree.
#[component]
pub fn SegmentTreeView(
    /// The segments to display.
    segments: Signal<Vec<SegmentNode>>,
) -> impl IntoView {
    view! {
        <div class="segment-tree">
            {move || {
                segments.get()
                    .into_iter()
                    .map(render_segment_node)
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

/// Render a single segment node (recursive for children).
/// Uses `AnyView` to break the recursive opaque type cycle.
fn render_segment_node(node: SegmentNode) -> AnyView {
    let has_children = node.children.as_ref().is_some_and(|c| !c.is_empty());

    let children_nodes = node.children.clone().unwrap_or_default();
    let tag = node.tag.clone();
    let line_number = node.line_number;
    let raw_content = node.raw_content.clone();

    view! {
        <div class="tree-node">
            <div class="node-row">
                <span class="tag-badge">{tag}</span>
                <span class="line-number">"L"{line_number}</span>
                <span class="raw-content">{raw_content}</span>
            </div>
            {if has_children {
                Some(view! {
                    <div class="children">
                        {children_nodes
                            .into_iter()
                            .map(render_segment_node)
                            .collect::<Vec<_>>()}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
    .into_any()
}
