use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Input for the PID schema diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffInput {
    pub old_schema: serde_json::Value,
    pub new_schema: serde_json::Value,
    pub old_version: String,
    pub new_version: String,
    pub message_type: String,
    pub pid: String,
}

/// Compare two PID schema JSONs and produce a structured diff.
pub fn diff_pid_schemas(input: &DiffInput) -> PidSchemaDiff {
    let old_fields = extract_fields(&input.old_schema);
    let new_fields = extract_fields(&input.new_schema);

    let groups = diff_groups(&old_fields, &new_fields);
    let segments = diff_segments(&old_fields, &new_fields);
    let codes = diff_codes(&old_fields, &new_fields);
    let elements = diff_elements(&old_fields, &new_fields);

    PidSchemaDiff {
        old_version: input.old_version.clone(),
        new_version: input.new_version.clone(),
        message_type: input.message_type.clone(),
        pid: input.pid.clone(),
        unh_version: None,
        segments,
        codes,
        groups,
        elements,
    }
}

/// A flattened representation of a schema group for diffing.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FlatGroup {
    field_name: String,
    source_group: String,
    qualifier: Option<String>,
    disc_segment: Option<String>,
    parent: Option<String>,
    segments: Vec<FlatSegment>,
}

#[derive(Debug, Clone)]
struct FlatSegment {
    tag: String,
    elements: Vec<FlatElement>,
}

#[derive(Debug, Clone)]
struct FlatElement {
    index: usize,
    id: String,
    element_type: String,
    codes: Vec<String>,
    components: Vec<FlatComponent>,
}

#[derive(Debug, Clone)]
struct FlatComponent {
    sub_index: usize,
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    element_type: String,
    codes: Vec<String>,
}

/// Extract all groups from PID schema JSON into a flat map keyed by field_name.
fn extract_fields(schema: &serde_json::Value) -> BTreeMap<String, FlatGroup> {
    let mut result = BTreeMap::new();
    if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
        flatten_groups(fields, None, &mut result);
    }
    result
}

fn flatten_groups(
    fields: &serde_json::Map<String, serde_json::Value>,
    parent: Option<&str>,
    result: &mut BTreeMap<String, FlatGroup>,
) {
    for (field_name, field_value) in fields {
        let source_group = field_value
            .get("source_group")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let (disc_segment, qualifier) = extract_discriminator(field_value);
        let segments = extract_segments(field_value);

        result.insert(
            field_name.clone(),
            FlatGroup {
                field_name: field_name.clone(),
                source_group,
                qualifier,
                disc_segment,
                parent: parent.map(String::from),
                segments,
            },
        );

        // Recurse into children
        if let Some(children) = field_value.get("children").and_then(|c| c.as_object()) {
            flatten_groups(children, Some(field_name), result);
        }
    }
}

fn extract_discriminator(field: &serde_json::Value) -> (Option<String>, Option<String>) {
    if let Some(disc) = field.get("discriminator") {
        if disc.is_null() {
            return (None, None);
        }
        let segment = disc
            .get("segment")
            .and_then(|v| v.as_str())
            .map(String::from);
        let values = disc
            .get("values")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(String::from);
        (segment, values)
    } else {
        (None, None)
    }
}

fn extract_segments(field: &serde_json::Value) -> Vec<FlatSegment> {
    let Some(segments) = field.get("segments").and_then(|s| s.as_array()) else {
        return vec![];
    };

    segments
        .iter()
        .filter_map(|seg| {
            let tag = seg.get("id").and_then(|v| v.as_str())?.to_string();
            let elements = seg
                .get("elements")
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|el| {
                            let index = el.get("index").and_then(|v| v.as_u64())? as usize;
                            let id = el.get("id").and_then(|v| v.as_str())?.to_string();
                            let element_type = el
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("data")
                                .to_string();
                            let codes = extract_code_values(el);
                            let components = extract_components(el);
                            Some(FlatElement {
                                index,
                                id,
                                element_type,
                                codes,
                                components,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(FlatSegment { tag, elements })
        })
        .collect()
}

fn extract_code_values(element: &serde_json::Value) -> Vec<String> {
    element
        .get("codes")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("value").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_components(element: &serde_json::Value) -> Vec<FlatComponent> {
    element
        .get("components")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|comp| {
                    let sub_index = comp.get("sub_index").and_then(|v| v.as_u64())? as usize;
                    let id = comp.get("id").and_then(|v| v.as_str())?.to_string();
                    let element_type = comp
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("data")
                        .to_string();
                    let codes = extract_code_values(comp);
                    Some(FlatComponent {
                        sub_index,
                        id,
                        element_type,
                        codes,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn diff_groups(old: &BTreeMap<String, FlatGroup>, new: &BTreeMap<String, FlatGroup>) -> GroupDiff {
    let old_keys: BTreeSet<&String> = old.keys().collect();
    let new_keys: BTreeSet<&String> = new.keys().collect();

    let added: Vec<GroupEntry> = new_keys
        .difference(&old_keys)
        .map(|k| {
            let g = &new[*k];
            let entry_seg = g.disc_segment.as_ref().map(|seg| {
                if let Some(ref q) = g.qualifier {
                    format!("{}+{}", seg, q)
                } else {
                    seg.clone()
                }
            });
            GroupEntry {
                group: k.to_string(),
                parent: g.parent.clone().unwrap_or_else(|| "root".to_string()),
                entry_segment: entry_seg,
            }
        })
        .collect();

    let removed: Vec<GroupEntry> = old_keys
        .difference(&new_keys)
        .map(|k| {
            let g = &old[*k];
            GroupEntry {
                group: k.to_string(),
                parent: g.parent.clone().unwrap_or_else(|| "root".to_string()),
                entry_segment: None,
            }
        })
        .collect();

    // Detect restructured: same field_name but different parent
    let mut restructured = Vec::new();
    for key in old_keys.intersection(&new_keys) {
        let old_g = &old[*key];
        let new_g = &new[*key];
        if old_g.parent != new_g.parent {
            restructured.push(RestructuredGroup {
                group: key.to_string(),
                description: format!(
                    "Parent changed from {:?} to {:?}",
                    old_g.parent, new_g.parent
                ),
                manual_review: true,
            });
        }
    }

    GroupDiff {
        added,
        removed,
        restructured,
    }
}

fn diff_segments(
    old: &BTreeMap<String, FlatGroup>,
    new: &BTreeMap<String, FlatGroup>,
) -> SegmentDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut unchanged = Vec::new();

    let all_keys: BTreeSet<&String> = old.keys().chain(new.keys()).collect();

    for key in &all_keys {
        let old_group = old.get(*key);
        let new_group = new.get(*key);

        match (old_group, new_group) {
            (Some(og), Some(ng)) => {
                let old_tags: BTreeSet<&str> = og.segments.iter().map(|s| s.tag.as_str()).collect();
                let new_tags: BTreeSet<&str> = ng.segments.iter().map(|s| s.tag.as_str()).collect();

                for tag in old_tags.intersection(&new_tags) {
                    unchanged.push(SegmentEntry {
                        group: key.to_string(),
                        tag: tag.to_string(),
                        context: None,
                    });
                }
                for tag in new_tags.difference(&old_tags) {
                    added.push(SegmentEntry {
                        group: key.to_string(),
                        tag: tag.to_string(),
                        context: Some(format!("New segment in {}", key)),
                    });
                }
                for tag in old_tags.difference(&new_tags) {
                    removed.push(SegmentEntry {
                        group: key.to_string(),
                        tag: tag.to_string(),
                        context: Some(format!("Removed from {}", key)),
                    });
                }
            }
            (None, Some(ng)) => {
                for seg in &ng.segments {
                    added.push(SegmentEntry {
                        group: key.to_string(),
                        tag: seg.tag.clone(),
                        context: Some(format!("New group {}", key)),
                    });
                }
            }
            (Some(og), None) => {
                for seg in &og.segments {
                    removed.push(SegmentEntry {
                        group: key.to_string(),
                        tag: seg.tag.clone(),
                        context: Some(format!("Removed group {}", key)),
                    });
                }
            }
            (None, None) => unreachable!(),
        }
    }

    SegmentDiff {
        added,
        removed,
        unchanged,
    }
}

fn diff_codes(old: &BTreeMap<String, FlatGroup>, new: &BTreeMap<String, FlatGroup>) -> CodeDiff {
    let mut changed = Vec::new();

    for (key, new_group) in new {
        let Some(old_group) = old.get(key) else {
            continue;
        };

        // Match segments by position (not tag alone) to handle multiple
        // segments with the same tag (e.g., DTM+92 and DTM+93).
        for (seg_idx, new_seg) in new_group.segments.iter().enumerate() {
            let Some(old_seg) = old_group
                .segments
                .get(seg_idx)
                .filter(|s| s.tag == new_seg.tag)
            else {
                continue;
            };

            for new_el in &new_seg.elements {
                let old_el = old_seg.elements.iter().find(|e| e.index == new_el.index);

                if let Some(old_el) = old_el {
                    // Compare top-level codes
                    let old_codes: BTreeSet<&str> =
                        old_el.codes.iter().map(|s| s.as_str()).collect();
                    let new_codes: BTreeSet<&str> =
                        new_el.codes.iter().map(|s| s.as_str()).collect();

                    let added_codes: Vec<String> = new_codes
                        .difference(&old_codes)
                        .map(|s| s.to_string())
                        .collect();
                    let removed_codes: Vec<String> = old_codes
                        .difference(&new_codes)
                        .map(|s| s.to_string())
                        .collect();

                    if !added_codes.is_empty() || !removed_codes.is_empty() {
                        changed.push(CodeChange {
                            segment: new_seg.tag.clone(),
                            element: new_el.index.to_string(),
                            group: key.clone(),
                            added: added_codes,
                            removed: removed_codes,
                            context: None,
                        });
                    }

                    // Compare component codes
                    for new_comp in &new_el.components {
                        let old_comp = old_el
                            .components
                            .iter()
                            .find(|c| c.sub_index == new_comp.sub_index);
                        if let Some(old_comp) = old_comp {
                            let oc: BTreeSet<&str> =
                                old_comp.codes.iter().map(|s| s.as_str()).collect();
                            let nc: BTreeSet<&str> =
                                new_comp.codes.iter().map(|s| s.as_str()).collect();
                            let ac: Vec<String> =
                                nc.difference(&oc).map(|s| s.to_string()).collect();
                            let rc: Vec<String> =
                                oc.difference(&nc).map(|s| s.to_string()).collect();
                            if !ac.is_empty() || !rc.is_empty() {
                                changed.push(CodeChange {
                                    segment: new_seg.tag.clone(),
                                    element: format!("{}.{}", new_el.index, new_comp.sub_index),
                                    group: key.clone(),
                                    added: ac,
                                    removed: rc,
                                    context: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    CodeDiff { changed }
}

fn diff_elements(
    old: &BTreeMap<String, FlatGroup>,
    new: &BTreeMap<String, FlatGroup>,
) -> ElementDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for (key, new_group) in new {
        let Some(old_group) = old.get(key) else {
            continue;
        };

        for (seg_idx, new_seg) in new_group.segments.iter().enumerate() {
            let Some(old_seg) = old_group
                .segments
                .get(seg_idx)
                .filter(|s| s.tag == new_seg.tag)
            else {
                continue;
            };

            let old_indices: BTreeSet<usize> = old_seg.elements.iter().map(|e| e.index).collect();
            let new_indices: BTreeSet<usize> = new_seg.elements.iter().map(|e| e.index).collect();

            for idx in new_indices.difference(&old_indices) {
                let el = new_seg.elements.iter().find(|e| e.index == *idx).unwrap();
                added.push(ElementChange {
                    segment: new_seg.tag.clone(),
                    group: key.clone(),
                    index: *idx,
                    sub_index: None,
                    description: Some(format!("New element {} ({})", el.id, el.element_type)),
                });
            }
            for idx in old_indices.difference(&new_indices) {
                let el = old_seg.elements.iter().find(|e| e.index == *idx).unwrap();
                removed.push(ElementChange {
                    segment: new_seg.tag.clone(),
                    group: key.clone(),
                    index: *idx,
                    sub_index: None,
                    description: Some(format!("Removed element {}", el.id)),
                });
            }

            // Compare components within matching elements
            for new_el in &new_seg.elements {
                let Some(old_el) = old_seg.elements.iter().find(|e| e.index == new_el.index) else {
                    continue;
                };
                let old_subs: BTreeSet<usize> =
                    old_el.components.iter().map(|c| c.sub_index).collect();
                let new_subs: BTreeSet<usize> =
                    new_el.components.iter().map(|c| c.sub_index).collect();

                for si in new_subs.difference(&old_subs) {
                    added.push(ElementChange {
                        segment: new_seg.tag.clone(),
                        group: key.clone(),
                        index: new_el.index,
                        sub_index: Some(*si),
                        description: Some("New component".to_string()),
                    });
                }
                for si in old_subs.difference(&new_subs) {
                    removed.push(ElementChange {
                        segment: new_seg.tag.clone(),
                        group: key.clone(),
                        index: new_el.index,
                        sub_index: Some(*si),
                        description: Some("Removed component".to_string()),
                    });
                }
            }
        }
    }

    ElementDiff { added, removed }
}
