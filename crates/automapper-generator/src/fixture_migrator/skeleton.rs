/// Generate a skeleton EDIFACT segment string from a PID schema segment definition.
///
/// Code elements are filled with the first valid code value.
/// Data elements are left empty.
/// Trailing empty elements are trimmed.
pub fn generate_skeleton_segment(segment_schema: &serde_json::Value) -> String {
    let tag = segment_schema
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("???");

    let elements = segment_schema
        .get("elements")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Build element values in order
    let mut max_index = 0;
    for el in &elements {
        if let Some(idx) = el.get("index").and_then(|v| v.as_u64()) {
            max_index = max_index.max(idx as usize);
        }
    }

    let mut element_values: Vec<String> = vec![String::new(); max_index + 1];

    for el in &elements {
        let Some(idx) = el.get("index").and_then(|v| v.as_u64()) else {
            continue;
        };
        let idx = idx as usize;

        let el_type = el.get("type").and_then(|v| v.as_str()).unwrap_or("data");
        let components = el.get("components").and_then(|v| v.as_array());

        if let Some(components) = components {
            if !components.is_empty() {
                // Composite element — build component string
                let comp_str = build_composite(components);
                element_values[idx] = comp_str;
                continue;
            }
        }

        // Simple element
        if el_type == "code" {
            if let Some(first_code) = el
                .get("codes")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|c| c.get("value"))
                .and_then(|v| v.as_str())
            {
                element_values[idx] = first_code.to_string();
            }
        }
        // Data elements left as empty string
    }

    // Trim trailing empty elements
    while element_values.last().is_some_and(|v| v.is_empty()) {
        element_values.pop();
    }

    if element_values.is_empty() {
        tag.to_string()
    } else {
        format!("{}+{}", tag, element_values.join("+"))
    }
}

fn build_composite(components: &[serde_json::Value]) -> String {
    let mut max_sub = 0;
    for comp in components {
        if let Some(si) = comp.get("sub_index").and_then(|v| v.as_u64()) {
            max_sub = max_sub.max(si as usize);
        }
    }

    let mut comp_values: Vec<String> = vec![String::new(); max_sub + 1];

    for comp in components {
        let Some(si) = comp.get("sub_index").and_then(|v| v.as_u64()) else {
            continue;
        };
        let si = si as usize;
        let comp_type = comp.get("type").and_then(|v| v.as_str()).unwrap_or("data");

        if comp_type == "code" {
            if let Some(first_code) = comp
                .get("codes")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|c| c.get("value"))
                .and_then(|v| v.as_str())
            {
                comp_values[si] = first_code.to_string();
            }
        }
    }

    // Trim trailing empty components
    while comp_values.last().is_some_and(|v| v.is_empty()) {
        comp_values.pop();
    }

    comp_values.join(":")
}
