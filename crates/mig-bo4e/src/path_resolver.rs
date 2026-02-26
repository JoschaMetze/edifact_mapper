//! Resolves EDIFACT ID paths to numeric element indices (and vice versa).
//!
//! Built from a PID schema JSON. Used at TOML load time to normalize
//! named paths (e.g., `loc.c517.d3225`) to numeric paths (`loc.1.0`).
//! This keeps the engine hot path unchanged — all resolution happens once at load time.
//!
//! Also provides [`ReversePathResolver`] for converting numeric paths back to
//! self-documenting EDIFACT ID paths (used by the `migrate-paths` CLI).

use std::collections::HashMap;
use std::path::Path;

// ── Forward resolver: named → numeric ──

/// Resolves EDIFACT ID paths to numeric element indices.
///
/// Built from a PID schema JSON. Used at TOML load time to normalize
/// named paths (e.g., "loc.c517.d3225") to numeric paths ("loc.1.0").
///
/// Supports ordinal suffixes for duplicate IDs:
/// - Duplicate composites per segment: `c556` (first), `c556_2` (second), `c556_3` (third)
/// - Duplicate data elements per composite: `d3036` (first), `d3036_2` (second), etc.
#[derive(Clone)]
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

    /// Build from all PID schema JSON files in a directory.
    ///
    /// Loads every `pid_*_schema.json` file and merges their element mappings.
    /// This ensures comprehensive coverage across all PIDs.
    pub fn from_schema_dir(dir: &Path) -> Self {
        let mut resolver = Self {
            simple_elements: HashMap::new(),
            composite_elements: HashMap::new(),
        };

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let is_schema = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("pid_") && n.ends_with("_schema.json"))
                .unwrap_or(false);
            if is_schema {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(schema) = serde_json::from_str::<serde_json::Value>(&content) {
                        resolver.merge_schema(&schema);
                    }
                }
            }
        }

        resolver
    }

    /// Merge another PID schema into this resolver.
    pub fn merge_schema(&mut self, schema: &serde_json::Value) {
        if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
            for (_group_key, group_val) in fields {
                collect_from_group(
                    group_val,
                    &mut self.simple_elements,
                    &mut self.composite_elements,
                );
            }
        }
    }

    /// Resolve a single field path. Returns the numeric path if the input
    /// is a named path; returns the input unchanged if already numeric.
    ///
    /// Examples:
    /// - `"loc.c517.d3225"` → `"loc.1.0"`
    /// - `"seq.d1229"` → `"seq.0"`
    /// - `"cav[Z91].c889.d7111"` → `"cav[Z91].0.0"`
    /// - `"sts.c556_2.d9013"` → `"sts.3.0"` (ordinal suffix for duplicate composite)
    /// - `"nad.c080.d3036_2"` → `"nad.3.1"` (ordinal suffix for duplicate data element)
    /// - `"loc.1.0"` → `"loc.1.0"` (unchanged)
    pub fn resolve_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return path.to_string();
        }

        // Parse segment tag and optional qualifier: "cav[Z91]" → ("cav", "[Z91]")
        let (seg_raw, qualifier_suffix) = split_qualifier(parts[0]);
        let seg_upper = seg_raw.to_ascii_uppercase();

        let rest = &parts[1..];

        // Check if already numeric: first rest part starts with a digit
        if rest[0]
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return path.to_string();
        }

        // Try resolving as composite path: seg.cNNN.dNNN (with optional ordinal suffixes)
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

    /// Resolve a discriminator string to 3-part numeric format.
    ///
    /// The engine's `resolve_repetition` requires `TAG.N.M=VALUE` (3-part).
    ///
    /// Input formats:
    /// - Named simple: `"SEQ.d1229=ZF0"` → `"SEQ.0.0=ZF0"`
    /// - Named composite: `"STS.c556.d9013=E01"` → `"STS.2.0=E01"`
    /// - Numeric 3-part: `"LOC.0.0=Z16"` → `"LOC.0.0=Z16"` (unchanged)
    /// - Numeric 2-part: `"SEQ.0=ZF0"` → `"SEQ.0.0=ZF0"` (upgraded)
    pub fn resolve_discriminator(&self, disc: &str) -> String {
        let Some((path_part, value_part)) = disc.split_once('=') else {
            return disc.to_string();
        };

        let parts: Vec<&str> = path_part.split('.').collect();

        match parts.len() {
            2 => {
                let seg_upper = parts[0].to_ascii_uppercase();
                let element_ref = parts[1];

                // Check if already numeric — upgrade 2-part to 3-part
                if element_ref
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return format!("{}.{}.0={}", parts[0], element_ref, value_part);
                }

                // Try simple element resolution
                if is_edifact_id(element_ref) {
                    let element_id = element_ref.to_ascii_lowercase();
                    if let Some(&elem_idx) = self.simple_elements.get(&(seg_upper, element_id)) {
                        return format!("{}.{}.0={}", parts[0], elem_idx, value_part);
                    }
                }

                disc.to_string()
            }
            3 => {
                let seg_upper = parts[0].to_ascii_uppercase();

                // Check if already numeric
                if parts[1]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return disc.to_string();
                }

                // Try composite resolution: TAG.cNNN.dNNN=VALUE
                if is_edifact_id(parts[1]) && is_edifact_id(parts[2]) {
                    let composite_id = parts[1].to_ascii_lowercase();
                    let data_element_id = parts[2].to_ascii_lowercase();

                    if let Some(&(elem_idx, sub_idx)) =
                        self.composite_elements
                            .get(&(seg_upper, composite_id, data_element_id))
                    {
                        return format!("{}.{}.{}={}", parts[0], elem_idx, sub_idx, value_part);
                    }
                }

                disc.to_string()
            }
            _ => disc.to_string(),
        }
    }
}

// ── Reverse resolver: numeric → named ──

/// Converts numeric element paths back to self-documenting EDIFACT ID paths.
///
/// Used by the `migrate-paths` CLI to convert existing TOML files from
/// opaque numeric paths (`loc.1.0`) to readable named paths (`loc.c517.d3225`).
#[derive(Clone)]
pub struct ReversePathResolver {
    /// (seg_upper, elem_idx, sub_idx) → named suffix like "c517.d3225"
    composite_reverse: HashMap<(String, usize, usize), String>,
    /// (seg_upper, elem_idx) → named id like "d3227"
    simple_reverse: HashMap<(String, usize), String>,
    /// (seg_upper, elem_idx) → true if composite element
    is_composite: HashMap<(String, usize), bool>,
}

impl ReversePathResolver {
    /// Build from a PID schema JSON.
    pub fn from_schema(schema: &serde_json::Value) -> Self {
        let mut composite_reverse = HashMap::new();
        let mut simple_reverse = HashMap::new();
        let mut is_composite = HashMap::new();

        if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
            for (_group_key, group_val) in fields {
                collect_reverse_from_group(
                    group_val,
                    &mut composite_reverse,
                    &mut simple_reverse,
                    &mut is_composite,
                );
            }
        }

        Self {
            composite_reverse,
            simple_reverse,
            is_composite,
        }
    }

    /// Build from all PID schema JSON files in a directory.
    pub fn from_schema_dir(dir: &Path) -> Self {
        let mut resolver = Self {
            composite_reverse: HashMap::new(),
            simple_reverse: HashMap::new(),
            is_composite: HashMap::new(),
        };

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let is_schema = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("pid_") && n.ends_with("_schema.json"))
                .unwrap_or(false);
            if is_schema {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(schema) = serde_json::from_str::<serde_json::Value>(&content) {
                        resolver.merge_schema(&schema);
                    }
                }
            }
        }

        resolver
    }

    /// Merge another PID schema into this resolver.
    pub fn merge_schema(&mut self, schema: &serde_json::Value) {
        if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
            for (_group_key, group_val) in fields {
                collect_reverse_from_group(
                    group_val,
                    &mut self.composite_reverse,
                    &mut self.simple_reverse,
                    &mut self.is_composite,
                );
            }
        }
    }

    /// Convert a numeric path to a named EDIFACT ID path.
    ///
    /// Examples:
    /// - `"loc.1.0"` → `"loc.c517.d3225"`
    /// - `"loc.0"` → `"loc.d3227"` (simple element)
    /// - `"sts.2"` → `"sts.c556.d9013"` (2-part → expands to first component)
    /// - `"sts.3.0"` → `"sts.c556_2.d9013"` (ordinal suffix for duplicate composite)
    /// - `"cav[Z91].0.1"` → `"cav[Z91].c889.d7110"` (preserves qualifier)
    /// - `"loc.c517.d3225"` → `"loc.c517.d3225"` (already named, unchanged)
    pub fn reverse_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return path.to_string();
        }

        let (seg_raw, qualifier_suffix) = split_qualifier(parts[0]);
        let seg_upper = seg_raw.to_ascii_uppercase();
        let rest = &parts[1..];

        // If not numeric, already named — return as-is
        if !rest[0]
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return path.to_string();
        }

        match rest.len() {
            1 => {
                // 2-part: seg.N
                let Ok(elem_idx) = rest[0].parse::<usize>() else {
                    return path.to_string();
                };

                match self.is_composite.get(&(seg_upper.clone(), elem_idx)) {
                    Some(true) => {
                        // Composite — expand to first component: seg.cNNN.dNNN
                        if let Some(named) = self.composite_reverse.get(&(seg_upper, elem_idx, 0)) {
                            format!("{}{}.{}", seg_raw, qualifier_suffix, named)
                        } else {
                            path.to_string()
                        }
                    }
                    Some(false) => {
                        // Simple element: seg.dNNN
                        if let Some(named) = self.simple_reverse.get(&(seg_upper, elem_idx)) {
                            format!("{}{}.{}", seg_raw, qualifier_suffix, named)
                        } else {
                            path.to_string()
                        }
                    }
                    None => path.to_string(),
                }
            }
            2 => {
                // 3-part: seg.N.M
                let Ok(elem_idx) = rest[0].parse::<usize>() else {
                    return path.to_string();
                };
                let Ok(sub_idx) = rest[1].parse::<usize>() else {
                    return path.to_string();
                };

                if let Some(named) = self.composite_reverse.get(&(seg_upper, elem_idx, sub_idx)) {
                    format!("{}{}.{}", seg_raw, qualifier_suffix, named)
                } else {
                    path.to_string()
                }
            }
            _ => path.to_string(),
        }
    }

    /// Convert a 3-part numeric discriminator to named EDIFACT ID format.
    ///
    /// Examples:
    /// - `"LOC.0.0=Z16"` → `"LOC.d3227=Z16"` (simple element)
    /// - `"STS.2.0=E01"` → `"STS.c556.d9013=E01"` (composite element)
    /// - `"LOC.d3227=Z16"` → `"LOC.d3227=Z16"` (already named, unchanged)
    pub fn reverse_discriminator(&self, disc: &str) -> String {
        let Some((path_part, value_part)) = disc.split_once('=') else {
            return disc.to_string();
        };

        let parts: Vec<&str> = path_part.split('.').collect();
        if parts.len() != 3 {
            return disc.to_string();
        }

        let seg_raw = parts[0];
        let seg_upper = seg_raw.to_ascii_uppercase();

        // Check if numeric
        let Ok(elem_idx) = parts[1].parse::<usize>() else {
            return disc.to_string(); // Already named
        };
        let Ok(sub_idx) = parts[2].parse::<usize>() else {
            return disc.to_string();
        };

        // Check if it's a simple element (sub_idx 0 and element is not composite)
        if sub_idx == 0 {
            if let Some(false) = self.is_composite.get(&(seg_upper.clone(), elem_idx)) {
                if let Some(named) = self.simple_reverse.get(&(seg_upper.clone(), elem_idx)) {
                    return format!("{}.{}={}", seg_raw, named, value_part);
                }
            }
        }

        // Composite element
        if let Some(named) = self
            .composite_reverse
            .get(&(seg_upper.clone(), elem_idx, sub_idx))
        {
            return format!("{}.{}={}", seg_raw, named, value_part);
        }

        disc.to_string()
    }
}

// ── Helpers ──

/// Check if a string looks like an EDIFACT ID: starts with 'c' or 'd' followed by digits,
/// with an optional ordinal suffix (`_N`).
///
/// Matches: `c517`, `d3225`, `c556_2`, `d3036_3`
fn is_edifact_id(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some('c' | 'd' | 'C' | 'D') => {
            let rest: String = chars.collect();
            if rest.is_empty() {
                return false;
            }
            if let Some((base, suffix)) = rest.split_once('_') {
                !base.is_empty()
                    && base.chars().all(|c| c.is_ascii_digit())
                    && !suffix.is_empty()
                    && suffix.chars().all(|c| c.is_ascii_digit())
            } else {
                rest.chars().all(|c| c.is_ascii_digit())
            }
        }
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

/// Recursively collect forward element mappings from a group in the schema.
///
/// Tracks ordinal suffixes for duplicate composite IDs per segment and
/// duplicate data element IDs per composite.
fn collect_from_group(
    group: &serde_json::Value,
    simple: &mut HashMap<(String, String), usize>,
    composite: &mut HashMap<(String, String, String), (usize, usize)>,
) {
    if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
        for seg in segments {
            let seg_tag = seg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_ascii_uppercase();

            if let Some(elements) = seg.get("elements").and_then(|e| e.as_array()) {
                // Track composite ID occurrences for ordinal suffixes
                let mut composite_id_count: HashMap<String, usize> = HashMap::new();

                for elem in elements {
                    let elem_index =
                        elem.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    if let Some(composite_id) = elem.get("composite").and_then(|v| v.as_str()) {
                        let base_composite =
                            format!("c{}", &composite_id[1..]).to_ascii_lowercase();

                        // Track occurrence for ordinal suffix
                        let count = composite_id_count
                            .entry(base_composite.clone())
                            .or_insert(0);
                        *count += 1;

                        let composite_key = if *count == 1 {
                            base_composite
                        } else {
                            format!("{}_{}", base_composite, count)
                        };

                        if let Some(components) = elem.get("components").and_then(|c| c.as_array())
                        {
                            // Track data element ID occurrences within this composite
                            let mut data_elem_count: HashMap<String, usize> = HashMap::new();

                            for comp in components {
                                let comp_id = comp.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let sub_index =
                                    comp.get("sub_index").and_then(|v| v.as_u64()).unwrap_or(0)
                                        as usize;
                                let base_data = format!("d{}", comp_id).to_ascii_lowercase();

                                let dcount = data_elem_count.entry(base_data.clone()).or_insert(0);
                                *dcount += 1;

                                let data_key = if *dcount == 1 {
                                    base_data
                                } else {
                                    format!("{}_{}", base_data, dcount)
                                };

                                composite
                                    .entry((seg_tag.clone(), composite_key.clone(), data_key))
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

/// Recursively collect reverse element mappings from a group in the schema.
///
/// Builds (seg, elem_idx, sub_idx) → named path mappings, with ordinal suffixes
/// for duplicates.
fn collect_reverse_from_group(
    group: &serde_json::Value,
    composite_reverse: &mut HashMap<(String, usize, usize), String>,
    simple_reverse: &mut HashMap<(String, usize), String>,
    is_composite: &mut HashMap<(String, usize), bool>,
) {
    if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
        for seg in segments {
            let seg_tag = seg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_ascii_uppercase();

            if let Some(elements) = seg.get("elements").and_then(|e| e.as_array()) {
                let mut composite_id_count: HashMap<String, usize> = HashMap::new();

                for elem in elements {
                    let elem_index =
                        elem.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    if let Some(composite_id) = elem.get("composite").and_then(|v| v.as_str()) {
                        let base = format!("c{}", &composite_id[1..]).to_ascii_lowercase();

                        let count = composite_id_count.entry(base.clone()).or_insert(0);
                        *count += 1;

                        let comp_key = if *count == 1 {
                            base
                        } else {
                            format!("{}_{}", base, count)
                        };

                        is_composite
                            .entry((seg_tag.clone(), elem_index))
                            .or_insert(true);

                        if let Some(components) = elem.get("components").and_then(|c| c.as_array())
                        {
                            let mut data_elem_count: HashMap<String, usize> = HashMap::new();

                            for comp in components {
                                let comp_id = comp.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let sub_index =
                                    comp.get("sub_index").and_then(|v| v.as_u64()).unwrap_or(0)
                                        as usize;
                                let base_data = format!("d{}", comp_id).to_ascii_lowercase();

                                let dcount = data_elem_count.entry(base_data.clone()).or_insert(0);
                                *dcount += 1;

                                let data_key = if *dcount == 1 {
                                    base_data
                                } else {
                                    format!("{}_{}", base_data, dcount)
                                };

                                composite_reverse
                                    .entry((seg_tag.clone(), elem_index, sub_index))
                                    .or_insert(format!("{}.{}", comp_key, data_key));
                            }
                        }
                    } else {
                        let elem_id = elem.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let elem_id_lower = format!("d{}", elem_id).to_ascii_lowercase();

                        is_composite
                            .entry((seg_tag.clone(), elem_index))
                            .or_insert(false);
                        simple_reverse
                            .entry((seg_tag.clone(), elem_index))
                            .or_insert(elem_id_lower);
                    }
                }
            }
        }
    }

    if let Some(children) = group.get("children").and_then(|c| c.as_object()) {
        for (_child_key, child_val) in children {
            collect_reverse_from_group(child_val, composite_reverse, simple_reverse, is_composite);
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

    fn sts_schema() -> serde_json::Value {
        serde_json::json!({
            "beschreibung": "STS ordinal test",
            "fields": {
                "sg4": {
                    "segments": [{
                        "id": "STS",
                        "name": "Status",
                        "elements": [
                            {
                                "composite": "C601",
                                "index": 0,
                                "name": "Statuskategorie",
                                "components": [{
                                    "id": "9015",
                                    "sub_index": 0,
                                    "type": "code"
                                }]
                            },
                            {
                                "composite": "C555",
                                "index": 1,
                                "name": "Status",
                                "components": [{
                                    "id": "4405",
                                    "sub_index": 0,
                                    "type": "data"
                                }]
                            },
                            {
                                "composite": "C556",
                                "index": 2,
                                "name": "Statusanlaß",
                                "components": [{
                                    "id": "9013",
                                    "sub_index": 0,
                                    "type": "code"
                                }]
                            },
                            {
                                "composite": "C556",
                                "index": 3,
                                "name": "Statusanlaß",
                                "components": [{
                                    "id": "9013",
                                    "sub_index": 0,
                                    "type": "code"
                                }]
                            },
                            {
                                "composite": "C556",
                                "index": 4,
                                "name": "Statusanlaß",
                                "components": [{
                                    "id": "9013",
                                    "sub_index": 0,
                                    "type": "code"
                                }]
                            }
                        ]
                    }],
                    "source_group": "SG4"
                }
            }
        })
    }

    fn nad_schema() -> serde_json::Value {
        serde_json::json!({
            "beschreibung": "NAD ordinal test",
            "fields": {
                "sg12_z04": {
                    "segments": [{
                        "id": "NAD",
                        "name": "Geschäftspartner",
                        "elements": [
                            {
                                "id": "3229",
                                "index": 0,
                                "type": "code"
                            },
                            {
                                "composite": "C082",
                                "index": 1,
                                "name": "Identifikation",
                                "components": [
                                    { "id": "3039", "sub_index": 0, "type": "data" },
                                    { "id": "1131", "sub_index": 1, "type": "data" },
                                    { "id": "3055", "sub_index": 2, "type": "code" }
                                ]
                            },
                            {
                                "composite": "C058",
                                "index": 2,
                                "name": "Zusatzinfo",
                                "components": [
                                    { "id": "3124", "sub_index": 0, "type": "data" },
                                    { "id": "3124", "sub_index": 1, "type": "data" },
                                    { "id": "3124", "sub_index": 2, "type": "data" }
                                ]
                            },
                            {
                                "composite": "C080",
                                "index": 3,
                                "name": "Name",
                                "components": [
                                    { "id": "3036", "sub_index": 0, "type": "data" },
                                    { "id": "3036", "sub_index": 1, "type": "data" },
                                    { "id": "3036", "sub_index": 2, "type": "data" },
                                    { "id": "3036", "sub_index": 3, "type": "data" },
                                    { "id": "3036", "sub_index": 4, "type": "data" },
                                    { "id": "3045", "sub_index": 5, "type": "code" }
                                ]
                            }
                        ]
                    }],
                    "source_group": "SG12"
                }
            }
        })
    }

    // ── Forward resolver tests ──

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
    fn resolve_discriminator_named() {
        let resolver = PathResolver::from_schema(&test_schema());
        // Named → 3-part numeric
        assert_eq!(
            resolver.resolve_discriminator("SEQ.d1229=ZF0"),
            "SEQ.0.0=ZF0"
        );
        assert_eq!(
            resolver.resolve_discriminator("LOC.d3227=Z16"),
            "LOC.0.0=Z16"
        );
    }

    #[test]
    fn resolve_discriminator_numeric() {
        let resolver = PathResolver::from_schema(&test_schema());
        // Already 3-part numeric — unchanged
        assert_eq!(resolver.resolve_discriminator("SEQ.0.0=ZF0"), "SEQ.0.0=ZF0");
        // 2-part numeric — upgraded to 3-part
        assert_eq!(resolver.resolve_discriminator("SEQ.0=ZF0"), "SEQ.0.0=ZF0");
    }

    #[test]
    fn resolve_discriminator_composite() {
        let resolver = PathResolver::from_schema(&sts_schema());
        // Composite discriminator: TAG.cNNN.dNNN=VALUE → TAG.N.M=VALUE
        assert_eq!(
            resolver.resolve_discriminator("STS.c556.d9013=E01"),
            "STS.2.0=E01"
        );
        assert_eq!(
            resolver.resolve_discriminator("STS.c556_2.d9013=ZW4"),
            "STS.3.0=ZW4"
        );
    }

    #[test]
    fn unresolved_paths_unchanged() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("xyz.d9999"), "xyz.d9999");
        assert_eq!(resolver.resolve_path("loc"), "loc");
    }

    #[test]
    fn composite_id_case_insensitive() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("loc.c517.d3225"), "loc.1.0");
        assert_eq!(resolver.resolve_path("LOC.c517.d3225"), "LOC.1.0");
    }

    #[test]
    fn seq_composite_path() {
        let resolver = PathResolver::from_schema(&test_schema());
        assert_eq!(resolver.resolve_path("seq.c286.d1050"), "seq.1.0");
    }

    // ── Ordinal suffix tests ──

    #[test]
    fn ordinal_suffix_duplicate_composites() {
        let resolver = PathResolver::from_schema(&sts_schema());
        // First C556 at index 2
        assert_eq!(resolver.resolve_path("sts.c556.d9013"), "sts.2.0");
        // Second C556 at index 3
        assert_eq!(resolver.resolve_path("sts.c556_2.d9013"), "sts.3.0");
        // Third C556 at index 4
        assert_eq!(resolver.resolve_path("sts.c556_3.d9013"), "sts.4.0");
        // Non-duplicate composites still work
        assert_eq!(resolver.resolve_path("sts.c601.d9015"), "sts.0.0");
        assert_eq!(resolver.resolve_path("sts.c555.d4405"), "sts.1.0");
    }

    #[test]
    fn ordinal_suffix_duplicate_data_elements() {
        let resolver = PathResolver::from_schema(&nad_schema());
        // NAD C080: d3036×5 + d3045×1
        assert_eq!(resolver.resolve_path("nad.c080.d3036"), "nad.3.0");
        assert_eq!(resolver.resolve_path("nad.c080.d3036_2"), "nad.3.1");
        assert_eq!(resolver.resolve_path("nad.c080.d3036_3"), "nad.3.2");
        assert_eq!(resolver.resolve_path("nad.c080.d3036_4"), "nad.3.3");
        assert_eq!(resolver.resolve_path("nad.c080.d3036_5"), "nad.3.4");
        assert_eq!(resolver.resolve_path("nad.c080.d3045"), "nad.3.5");
        // C058: d3124×3
        assert_eq!(resolver.resolve_path("nad.c058.d3124"), "nad.2.0");
        assert_eq!(resolver.resolve_path("nad.c058.d3124_2"), "nad.2.1");
        assert_eq!(resolver.resolve_path("nad.c058.d3124_3"), "nad.2.2");
    }

    #[test]
    fn is_edifact_id_with_suffix() {
        assert!(is_edifact_id("c556"));
        assert!(is_edifact_id("c556_2"));
        assert!(is_edifact_id("c556_3"));
        assert!(is_edifact_id("d3036"));
        assert!(is_edifact_id("d3036_2"));
        assert!(is_edifact_id("D3036_5"));
        assert!(!is_edifact_id("c"));
        assert!(!is_edifact_id("c_2"));
        assert!(!is_edifact_id("c556_"));
        assert!(!is_edifact_id("c556_a"));
        assert!(!is_edifact_id("abc"));
        assert!(!is_edifact_id("123"));
    }

    // ── Reverse resolver tests ──

    #[test]
    fn reverse_path_composite() {
        let resolver = ReversePathResolver::from_schema(&test_schema());
        assert_eq!(resolver.reverse_path("loc.1.0"), "loc.c517.d3225");
        assert_eq!(resolver.reverse_path("loc.1.1"), "loc.c517.d1131");
        assert_eq!(resolver.reverse_path("seq.1.0"), "seq.c286.d1050");
        assert_eq!(resolver.reverse_path("cav.0.0"), "cav.c889.d7111");
        assert_eq!(resolver.reverse_path("cav.0.1"), "cav.c889.d7110");
    }

    #[test]
    fn reverse_path_simple() {
        let resolver = ReversePathResolver::from_schema(&test_schema());
        assert_eq!(resolver.reverse_path("loc.0"), "loc.d3227");
        assert_eq!(resolver.reverse_path("seq.0"), "seq.d1229");
    }

    #[test]
    fn reverse_path_two_part_composite() {
        let resolver = ReversePathResolver::from_schema(&sts_schema());
        // 2-part numeric for composite → expands to first component
        assert_eq!(resolver.reverse_path("sts.0"), "sts.c601.d9015");
        assert_eq!(resolver.reverse_path("sts.1"), "sts.c555.d4405");
        assert_eq!(resolver.reverse_path("sts.2"), "sts.c556.d9013");
    }

    #[test]
    fn reverse_path_ordinal_composites() {
        let resolver = ReversePathResolver::from_schema(&sts_schema());
        assert_eq!(resolver.reverse_path("sts.2.0"), "sts.c556.d9013");
        assert_eq!(resolver.reverse_path("sts.3.0"), "sts.c556_2.d9013");
        assert_eq!(resolver.reverse_path("sts.4.0"), "sts.c556_3.d9013");
    }

    #[test]
    fn reverse_path_ordinal_data_elements() {
        let resolver = ReversePathResolver::from_schema(&nad_schema());
        assert_eq!(resolver.reverse_path("nad.3.0"), "nad.c080.d3036");
        assert_eq!(resolver.reverse_path("nad.3.1"), "nad.c080.d3036_2");
        assert_eq!(resolver.reverse_path("nad.3.2"), "nad.c080.d3036_3");
        assert_eq!(resolver.reverse_path("nad.3.3"), "nad.c080.d3036_4");
        assert_eq!(resolver.reverse_path("nad.3.4"), "nad.c080.d3036_5");
        assert_eq!(resolver.reverse_path("nad.3.5"), "nad.c080.d3045");
    }

    #[test]
    fn reverse_path_qualifier() {
        let resolver = ReversePathResolver::from_schema(&test_schema());
        assert_eq!(resolver.reverse_path("cav[Z91].0.0"), "cav[Z91].c889.d7111");
        assert_eq!(resolver.reverse_path("cav[Z91].0.1"), "cav[Z91].c889.d7110");
    }

    #[test]
    fn reverse_path_already_named() {
        let resolver = ReversePathResolver::from_schema(&test_schema());
        assert_eq!(resolver.reverse_path("loc.c517.d3225"), "loc.c517.d3225");
        assert_eq!(resolver.reverse_path("loc.d3227"), "loc.d3227");
    }

    #[test]
    fn reverse_discriminator_simple() {
        let resolver = ReversePathResolver::from_schema(&test_schema());
        assert_eq!(
            resolver.reverse_discriminator("LOC.0.0=Z16"),
            "LOC.d3227=Z16"
        );
        assert_eq!(
            resolver.reverse_discriminator("SEQ.0.0=ZF0"),
            "SEQ.d1229=ZF0"
        );
    }

    #[test]
    fn reverse_discriminator_composite() {
        let resolver = ReversePathResolver::from_schema(&sts_schema());
        assert_eq!(
            resolver.reverse_discriminator("STS.2.0=E01"),
            "STS.c556.d9013=E01"
        );
        assert_eq!(
            resolver.reverse_discriminator("STS.3.0=ZW4"),
            "STS.c556_2.d9013=ZW4"
        );
    }

    #[test]
    fn reverse_discriminator_already_named() {
        let resolver = ReversePathResolver::from_schema(&test_schema());
        // Not numeric → unchanged
        assert_eq!(
            resolver.reverse_discriminator("LOC.d3227=Z16"),
            "LOC.d3227=Z16"
        );
    }

    #[test]
    fn forward_reverse_roundtrip() {
        let fwd = PathResolver::from_schema(&sts_schema());
        let rev = ReversePathResolver::from_schema(&sts_schema());

        // Named → numeric → named
        let named = "sts.c556_2.d9013";
        let numeric = fwd.resolve_path(named);
        assert_eq!(numeric, "sts.3.0");
        let back = rev.reverse_path(&numeric);
        assert_eq!(back, named);

        // Simple element roundtrip
        let named_simple = "nad.d3229";
        let fwd_nad = PathResolver::from_schema(&nad_schema());
        let rev_nad = ReversePathResolver::from_schema(&nad_schema());
        let numeric_simple = fwd_nad.resolve_path(named_simple);
        assert_eq!(numeric_simple, "nad.0");
        let back_simple = rev_nad.reverse_path(&numeric_simple);
        assert_eq!(back_simple, named_simple);
    }

    #[test]
    fn discriminator_forward_reverse_roundtrip() {
        let fwd = PathResolver::from_schema(&test_schema());
        let rev = ReversePathResolver::from_schema(&test_schema());

        let named = "LOC.d3227=Z16";
        let numeric = fwd.resolve_discriminator(named);
        assert_eq!(numeric, "LOC.0.0=Z16");
        let back = rev.reverse_discriminator(&numeric);
        assert_eq!(back, named);
    }
}
