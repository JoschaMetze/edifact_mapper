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
            <For
                each=move || segments.get().into_iter().enumerate().collect::<Vec<_>>()
                key=|(i, _)| *i
                children=move |(_, node)| {
                    view! { <SegmentNodeView node=node /> }
                }
            />
        </div>
    }
}

/// Display a single segment node (recursive for children).
#[component]
fn SegmentNodeView(
    /// The segment node to display.
    node: SegmentNode,
) -> impl IntoView {
    let has_children = node
        .children
        .as_ref()
        .map(|c| !c.is_empty())
        .unwrap_or(false);

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
                            .map(|child| view! { <SegmentNodeView node=child /> })
                            .collect::<Vec<_>>()}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}
