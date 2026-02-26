//! Resolves EDIFACT ID paths to numeric element indices.
//!
//! Built from a PID schema JSON. Used at TOML load time to normalize
//! named paths (e.g., `loc.c517.d3225`) to numeric paths (`loc.1.0`).
//! This keeps the engine hot path unchanged — all resolution happens once at load time.

use std::collections::HashMap;

/// Resolves EDIFACT ID paths to numeric element indices.
///
/// Built from a PID schema JSON. Used at TOML load time to normalize
/// named paths (e.g., "loc.c517.d3225") to numeric paths ("loc.1.0").
pub struct PathResolver {
    /// (segment_tag_upper, composite_id_lower, data_element_id_lower) → (element_index, sub_index)
    composite_elements: HashMap<(String, String, String), (usize, usize)>,
    /// (segment_tag_upper, element_id_lower) → element_index for simple data elements
    simple_elements: HashMap<(String, String), usize>,
}

impl PathResolver {
    /// Build from a PID schema JSON (`serde_json::Value`).
    ///
    /// Walks all groups recursively, collecting segment element indices.
    pub fn from_schema(schema: &serde_json::Value) -> Self {
        let mut simple_elements = HashMap::new();
        let mut composite_elements = HashMap::new();

        if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
            for (_group_key, group_val) in fields {
                collect_from_group(group_val, &mut simple_elements, &mut composite_elements);
            }
        }

        Self {
            simple_elements,
            composite_elements,
        }
    }

    /// Resolve a single field path. Returns the numeric path if the input
    /// is a named path; returns the input unchanged if already numeric.
    ///
    /// Examples:
    /// - `"loc.c517.d3225"` → `"loc.1.0"`
    /// - `"seq.d1229"` → `"seq.0"`
    /// - `"cav[Z91].c889.d7111"` → `"cav[Z91].0.1"`
    /// - `"loc.1.0"` → `"loc.1.0"` (unchanged)
    pub fn resolve_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return path.to_string();
        }

        // Parse segment tag and optional qualifier: "cav[Z91]" → ("cav", Some("[Z91]"))
        let (seg_raw, qualifier_suffix) = split_qualifier(parts[0]);
        let seg_upper = seg_raw.to_ascii_uppercase();

        let rest = &parts[1..];

        // Check if already numeric: first rest part is a digit
        if rest[0]
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return path.to_string();
        }

        // Try resolving as composite path: seg.cNNN.dNNN
        if rest.len() == 2 && is_edifact_id(rest[0]) && is_edifact_id(rest[1]) {
            let composite_id = rest[0].to_ascii_lowercase();
            let data_element_id = rest[1].to_ascii_lowercase();

            if let Some(&(elem_idx, sub_idx)) =
                self.composite_elements
                    .get(&(seg_upper.clone(), composite_id, data_element_id))
            {
                return format!("{}{}.{}.{}", seg_raw, qualifier_suffix, elem_idx, sub_idx);
            }
        }

        // Try resolving as simple element: seg.dNNN
        if rest.len() == 1 && is_edifact_id(rest[0]) {
            let element_id = rest[0].to_ascii_lowercase();

            if let Some(&elem_idx) = self.simple_elements.get(&(seg_upper, element_id)) {
                return format!("{}{}.{}", seg_raw, qualifier_suffix, elem_idx);
            }
        }

        // Unresolved — return as-is
        path.to_string()
    }

    /// Resolve a discriminator string.
    ///
    /// `"SEQ.d1229=ZF0"` → `"SEQ.0=ZF0"`
    pub fn resolve_discriminator(&self, disc: &str) -> String {
        // Split on '=' to get path part and value part
        let Some((path_part, value_part)) = disc.split_once('=') else {
            return disc.to_string();
        };

        let parts: Vec<&str> = path_part.split('.').collect();
        if parts.len() != 2 {
            return disc.to_string();
        }

        let seg_upper = parts[0].to_ascii_uppercase();
        let element_ref = parts[1];

        // Check if already numeric
        if element_ref
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return disc.to_string();
        }

        // Try simple element resolution
        if is_edifact_id(element_ref) {
            let element_id = element_ref.to_ascii_lowercase();
            if let Some(&elem_idx) = self.simple_elements.get(&(seg_upper.clone(), element_id)) {
                return format!("{}.{}={}", parts[0], elem_idx, value_part);
            }
        }

        disc.to_string()
    }
}

/// Check if a string looks like an EDIFACT ID: starts with 'c' or 'd' followed by digits.
fn is_edifact_id(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some('c' | 'd' | 'C' | 'D') => chars.all(|c| c.is_ascii_digit()),
        _ => false,
    }
}

/// Split qualifier from tag: `"cav[Z91]"` → `("cav", "[Z91]")`, `"loc"` → `("loc", "")`
fn split_qualifier(tag: &str) -> (&str, &str) {
    if let Some(bracket_pos) = tag.find('[') {
        (&tag[..bracket_pos], &tag[bracket_pos..])
    } else {
        (tag, "")
    }
}

/// Recursively collect element mappings from a group in the schema.
fn collect_from_group(
    group: &serde_json::Value,
    simple: &mut HashMap<(String, String), usize>,
    composite: &mut HashMap<(String, String, String), (usize, usize)>,
) {
    // Process segments in this group
    if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
        for seg in segments {
            let seg_tag = seg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_ascii_uppercase();

            if let Some(elements) = seg.get("elements").and_then(|e| e.as_array()) {
                for elem in elements {
                    let elem_index =
                        elem.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    if let Some(composite_id) = elem.get("composite").and_then(|v| v.as_str()) {
                        // Composite element: collect each component
                        let composite_lower =
                            format!("c{}", &composite_id[1..]).to_ascii_lowercase();
                        if let Some(components) = elem.get("components").and_then(|c| c.as_array())
                        {
                            for comp in components {
                                let comp_id = comp.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let sub_index =
                                    comp.get("sub_index").and_then(|v| v.as_u64()).unwrap_or(0)
                                        as usize;
                                let data_elem_lower = format!("d{}", comp_id).to_ascii_lowercase();
                                composite
                                    .entry((
                                        seg_tag.clone(),
                                        composite_lower.clone(),
                                        data_elem_lower,
                                    ))
                                    .or_insert((elem_index, sub_index));
                            }
                        }
                    } else {
                        // Simple data element
                        let elem_id = elem.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let elem_id_lower = format!("d{}", elem_id).to_ascii_lowercase();
                        simple
                            .entry((seg_tag.clone(), elem_id_lower))
                            .or_insert(elem_index);
                    }
                }
            }
        }
    }

    // Recurse into children
    if let Some(children) = group.get("children").and_then(|c| c.as_object()) {
        for (_child_key, child_val) in children {
            collect_from_group(child_val, simple, composite);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> serde_json::Value {
        serde_json::json!({
            "beschreibung": "Test PID",
            "fields": {
                "sg4": {
                    "segments": [
                        {
                            "id": "LOC",
                            "name": "Lokation",
                            "elements": [
                                {
                                    "id": "3227",
                                    "index": 0,
                                    "name": "Lokation, Qualifier",
                                    "type": "code"
                                },
                                {
                                    "composite": "C517",
                                    "index": 1,
                                    "name": "Lokationsidentifikation",
                                    "components": [
                                        {
                                            "id": "3225",
                                            "sub_index": 0,
                                            "name": "MaLo-ID",
                                            "type": "data"
                                        },
                                        {
                                            "id": "1131",
                                            "sub_index": 1,
                                            "name": "Codeliste, Code",
                                            "type": "data"
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "id": "SEQ",
                            "name": "Reihenfolge",
                            "elements": [
                                {
                                    "id": "1229",
                                    "index": 0,
                                    "name": "Handlung, Code",
                                    "type": "code"
                                },
                                {
                                    "composite": "C286",
                                    "index": 1,
                                    "name": "Information über eine Folge",
                                    "components": [
                                        {
                                            "id": "1050",
                                            "sub_index": 0,
                                            "name": "Referenz auf Zeitraum-ID",
                                            "type": "data"
                                        }
                                    ]
                                }
                            ]
                        }
                    ],
                    "source_group": "SG4",
                    "children": {
                        "sg8_zf0": {
                            "segments": [
                                {
                                    "id": "CAV",
                                    "name": "Merkmal",
                                    "elements": [
                                        {
                                            "composite": "C889",
                                            "index": 0,
                                            "name": "Merkmalswert",
                                            "components": [
                                                {
                                                    "id": "7111",
                                                    "sub_index": 0,
                                                    "name": "Merkmalswert, Code",
                                                    "type": "code"
                                                },
                                                {
                                                    "id": "7110",
                                                    "sub_index": 1,
                                                    "name": "Merkmalswert",
                                                    "type": "data"
                                                }
                                            ]
                                        }
                                    ]
                                }
                            ],
                            "source_group": "SG8"
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn resolve_composite_path() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("loc.c517.d3225"), "loc.1.0");
        assert_eq!(resolver.resolve_path("loc.c517.d1131"), "loc.1.1");
    }

    #[test]
    fn resolve_simple_element_path() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("loc.d3227"), "loc.0");
        assert_eq!(resolver.resolve_path("seq.d1229"), "seq.0");
    }

    #[test]
    fn resolve_nested_group_paths() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("cav.c889.d7111"), "cav.0.0");
        assert_eq!(resolver.resolve_path("cav.c889.d7110"), "cav.0.1");
    }

    #[test]
    fn numeric_paths_unchanged() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("loc.1.0"), "loc.1.0");
        assert_eq!(resolver.resolve_path("loc.0"), "loc.0");
        assert_eq!(resolver.resolve_path("seq.0"), "seq.0");
    }

    #[test]
    fn qualifier_paths() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("cav[Z91].c889.d7111"), "cav[Z91].0.0");
        assert_eq!(resolver.resolve_path("cav[Z91].0.1"), "cav[Z91].0.1");
    }

    #[test]
    fn resolve_discriminator() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_discriminator("SEQ.d1229=ZF0"), "SEQ.0=ZF0");
        assert_eq!(resolver.resolve_discriminator("LOC.d3227=Z16"), "LOC.0=Z16");
        // Already numeric — unchanged
        assert_eq!(resolver.resolve_discriminator("SEQ.0=ZF0"), "SEQ.0=ZF0");
    }

    #[test]
    fn unresolved_paths_unchanged() {
        let resolver = PathResolver::from_schema(&test_schema());
        // Unknown segment
        assert_eq!(resolver.resolve_path("xyz.d9999"), "xyz.d9999");
        // Single-part path
        assert_eq!(resolver.resolve_path("loc"), "loc");
    }

    #[test]
    fn composite_id_case_insensitive() {
        let resolver = PathResolver::from_schema(&test_schema());
        // Schema has "C517" — resolve should work with "c517"
        assert_eq!(resolver.resolve_path("loc.c517.d3225"), "loc.1.0");
        assert_eq!(resolver.resolve_path("LOC.c517.d3225"), "LOC.1.0");
    }

    #[test]
    fn seq_composite_path() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("seq.c286.d1050"), "seq.1.0");
    }
}
