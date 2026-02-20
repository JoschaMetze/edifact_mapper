//! Mapping engine — loads TOML definitions and provides bidirectional conversion.
//!
//! Supports nested group paths (e.g., "SG4.SG5") for navigating the assembled tree
//! and provides `map_forward` / `map_reverse` for full entity conversion.

use std::path::Path;

use mig_assembly::assembler::{
    AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree,
};
use mig_types::segment::OwnedSegment;

use crate::definition::{FieldMapping, MappingDefinition};
use crate::error::MappingError;

/// The mapping engine holds all loaded mapping definitions
/// and provides methods for bidirectional conversion.
pub struct MappingEngine {
    definitions: Vec<MappingDefinition>,
}

impl MappingEngine {
    /// Load all TOML mapping files from a directory.
    pub fn load(dir: &Path) -> Result<Self, MappingError> {
        let mut definitions = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                let def: MappingDefinition =
                    toml::from_str(&content).map_err(|e| MappingError::TomlParse {
                        file: path.display().to_string(),
                        message: e.to_string(),
                    })?;
                definitions.push(def);
            }
        }

        Ok(Self { definitions })
    }

    /// Create an engine from an already-parsed list of definitions.
    pub fn from_definitions(definitions: Vec<MappingDefinition>) -> Self {
        Self { definitions }
    }

    /// Get all loaded definitions.
    pub fn definitions(&self) -> &[MappingDefinition] {
        &self.definitions
    }

    /// Find a definition by entity name.
    pub fn definition_for_entity(&self, entity: &str) -> Option<&MappingDefinition> {
        self.definitions.iter().find(|d| d.meta.entity == entity)
    }

    // ── Forward mapping: tree → BO4E ──

    /// Extract a field value from an assembled tree using a mapping path.
    ///
    /// `group_path` supports dotted notation for nested groups (e.g., "SG4.SG5").
    /// Parent groups default to repetition 0; `repetition` applies to the leaf group.
    ///
    /// Path format: "segment.composite.data_element" e.g., "loc.c517.d3225"
    pub fn extract_field(
        &self,
        tree: &AssembledTree,
        group_path: &str,
        path: &str,
        repetition: usize,
    ) -> Option<String> {
        let instance = Self::resolve_group_instance(tree, group_path, repetition)?;
        Self::extract_from_instance(instance, path)
    }

    /// Navigate a potentially nested group path to find a group instance.
    ///
    /// For "SG4.SG5", finds SG4\[0\] then SG5 at the given repetition within it.
    /// For "SG8", finds SG8 at the given repetition in the top-level groups.
    ///
    /// Supports intermediate repetition with colon syntax: "SG4.SG8:1.SG10"
    /// means SG4\[0\] → SG8\[1\] → SG10\[repetition\]. Without a colon suffix,
    /// intermediate groups default to repetition 0.
    pub fn resolve_group_instance<'a>(
        tree: &'a AssembledTree,
        group_path: &str,
        repetition: usize,
    ) -> Option<&'a AssembledGroupInstance> {
        let parts: Vec<&str> = group_path.split('.').collect();

        let (first_id, first_rep) = parse_group_spec(parts[0]);
        let first_group = tree.groups.iter().find(|g| g.group_id == first_id)?;

        if parts.len() == 1 {
            // Single part — use the explicit rep from spec or the `repetition` param
            let rep = first_rep.unwrap_or(repetition);
            return first_group.repetitions.get(rep);
        }

        // Navigate through groups; intermediate parts default to rep 0
        // unless explicitly specified via `:N` suffix
        let mut current_instance = first_group.repetitions.get(first_rep.unwrap_or(0))?;

        for (i, part) in parts[1..].iter().enumerate() {
            let (group_id, explicit_rep) = parse_group_spec(part);
            let child_group = current_instance
                .child_groups
                .iter()
                .find(|g| g.group_id == group_id)?;

            if i == parts.len() - 2 {
                // Last part — use explicit rep, or fall back to `repetition`
                let rep = explicit_rep.unwrap_or(repetition);
                return child_group.repetitions.get(rep);
            }
            // Intermediate — use explicit rep or 0
            current_instance = child_group.repetitions.get(explicit_rep.unwrap_or(0))?;
        }

        None
    }

    /// Extract a field from a group instance by path.
    ///
    /// Supports qualifier-based segment selection with `tag[qualifier]` syntax:
    /// - `"dtm.0.1"` → first DTM segment, elements\[0\]\[1\]
    /// - `"dtm[92].0.1"` → DTM where elements\[0\]\[0\] == "92", then elements\[0\]\[1\]
    pub fn extract_from_instance(instance: &AssembledGroupInstance, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // Parse segment tag and optional qualifier: "dtm[92]" → ("DTM", Some("92"))
        let (segment_tag, qualifier) = parse_tag_qualifier(parts[0]);

        let segment = if let Some(q) = qualifier {
            instance.segments.iter().find(|s| {
                s.tag.eq_ignore_ascii_case(&segment_tag)
                    && s.elements
                        .first()
                        .and_then(|e| e.first())
                        .map(|v| v.as_str())
                        == Some(q)
            })?
        } else {
            instance
                .segments
                .iter()
                .find(|s| s.tag.eq_ignore_ascii_case(&segment_tag))?
        };

        Self::resolve_field_path(segment, &parts[1..])
    }

    /// Map all fields in a definition from the assembled tree to a BO4E JSON object.
    ///
    /// `group_path` is the definition's `source_group` (may be dotted, e.g., "SG4.SG5").
    /// An empty `source_group` maps root-level segments (BGM, DTM, etc.).
    /// Returns a flat JSON object with target field names as keys.
    pub fn map_forward(
        &self,
        tree: &AssembledTree,
        def: &MappingDefinition,
        repetition: usize,
    ) -> serde_json::Value {
        let mut result = serde_json::Map::new();

        // Root-level mapping: source_group is empty → use tree's own segments
        if def.meta.source_group.is_empty() {
            let root_instance = AssembledGroupInstance {
                segments: tree.segments[..tree.post_group_start].to_vec(),
                child_groups: vec![],
            };
            Self::extract_fields_from_instance(&root_instance, &def.fields, &mut result);
            return serde_json::Value::Object(result);
        }

        let instance = Self::resolve_group_instance(tree, &def.meta.source_group, repetition);

        if let Some(instance) = instance {
            Self::extract_fields_from_instance(instance, &def.fields, &mut result);
        }

        serde_json::Value::Object(result)
    }

    /// Extract all fields from an instance into a result map (shared logic).
    fn extract_fields_from_instance(
        instance: &AssembledGroupInstance,
        fields: &std::collections::BTreeMap<String, FieldMapping>,
        result: &mut serde_json::Map<String, serde_json::Value>,
    ) {
        for (path, field_mapping) in fields {
            let (target, enum_map) = match field_mapping {
                FieldMapping::Simple(t) => (t.clone(), None),
                FieldMapping::Structured(s) => (s.target.clone(), s.enum_map.as_ref()),
                FieldMapping::Nested(_) => continue,
            };
            if target.is_empty() {
                continue;
            }
            if let Some(val) = Self::extract_from_instance(instance, path) {
                let mapped_val = if let Some(map) = enum_map {
                    map.get(&val).cloned().unwrap_or(val)
                } else {
                    val
                };
                set_nested_value(result, &target, mapped_val);
            }
        }
    }

    /// Map a PID struct field's segments to BO4E JSON.
    ///
    /// `segments` are the `OwnedSegment`s from a PID wrapper field.
    /// Converts to `AssembledSegment` format for compatibility with existing
    /// field extraction logic, then applies the definition's field mappings.
    pub fn map_forward_from_segments(
        &self,
        segments: &[OwnedSegment],
        def: &MappingDefinition,
    ) -> serde_json::Value {
        let assembled_segments: Vec<AssembledSegment> = segments
            .iter()
            .map(|s| AssembledSegment {
                tag: s.id.clone(),
                elements: s.elements.clone(),
            })
            .collect();

        let instance = AssembledGroupInstance {
            segments: assembled_segments,
            child_groups: vec![],
        };

        let mut result = serde_json::Map::new();
        for (path, field_mapping) in &def.fields {
            let (target, enum_map) = match field_mapping {
                FieldMapping::Simple(t) => (t.clone(), None),
                FieldMapping::Structured(s) => (s.target.clone(), s.enum_map.as_ref()),
                FieldMapping::Nested(_) => continue,
            };
            if target.is_empty() {
                continue;
            }
            if let Some(val) = Self::extract_from_instance(&instance, path) {
                let mapped_val = if let Some(map) = enum_map {
                    map.get(&val).cloned().unwrap_or(val)
                } else {
                    val
                };
                set_nested_value(&mut result, &target, mapped_val);
            }
        }

        serde_json::Value::Object(result)
    }

    // ── Reverse mapping: BO4E → tree ──

    /// Map a BO4E JSON object back to an assembled group instance.
    ///
    /// Uses the definition's field mappings to populate segment elements.
    /// Fields with `default` values are used when no BO4E value is present
    /// (useful for fixed qualifiers like LOC qualifier "Z16").
    ///
    /// Supports:
    /// - Named paths: `"d3227"` → element\[0\]\[0\], `"c517.d3225"` → element\[1\]\[0\]
    /// - Numeric index: `"0"` → element\[0\]\[0\], `"1.2"` → element\[1\]\[2\]
    /// - Qualifier selection: `"dtm[92].0.1"` → DTM segment with qualifier "92"
    pub fn map_reverse(
        &self,
        bo4e_value: &serde_json::Value,
        def: &MappingDefinition,
    ) -> AssembledGroupInstance {
        // Collect (segment_key, element_index, component_index, value) tuples.
        // segment_key includes qualifier for disambiguation: "DTM" or "DTM[92]".
        let mut field_values: Vec<(String, String, usize, usize, String)> = Vec::new();

        for (path, field_mapping) in &def.fields {
            let (target, default, enum_map) = match field_mapping {
                FieldMapping::Simple(t) => (t.clone(), None, None),
                FieldMapping::Structured(s) => {
                    (s.target.clone(), s.default.clone(), s.enum_map.as_ref())
                }
                FieldMapping::Nested(_) => continue,
            };

            let parts: Vec<&str> = path.split('.').collect();
            if parts.len() < 2 {
                continue;
            }

            let (seg_tag, qualifier) = parse_tag_qualifier(parts[0]);
            // Use the raw first part as segment key to group fields by segment instance
            let seg_key = parts[0].to_uppercase();
            let sub_path = &parts[1..];

            // Determine (element_idx, component_idx) from path
            let (element_idx, component_idx) = if let Ok(ei) = sub_path[0].parse::<usize>() {
                let ci = if sub_path.len() > 1 {
                    sub_path[1].parse::<usize>().unwrap_or(0)
                } else {
                    0
                };
                (ei, ci)
            } else {
                match sub_path.len() {
                    1 => (0, 0),
                    2 => (1, 0),
                    _ => continue,
                }
            };

            // Try BO4E value first, fall back to default
            let val = if target.is_empty() {
                default
            } else {
                let bo4e_val = self.populate_field(bo4e_value, &target, path);
                // Apply reverse enum_map: BO4E value → EDIFACT value
                let mapped_val = match (bo4e_val, enum_map) {
                    (Some(v), Some(map)) => {
                        // Reverse lookup: find EDIFACT key for BO4E value
                        map.iter()
                            .find(|(_, bo4e_v)| *bo4e_v == &v)
                            .map(|(edifact_k, _)| edifact_k.clone())
                            .or(Some(v))
                    }
                    (v, _) => v,
                };
                mapped_val.or(default)
            };

            if let Some(val) = val {
                field_values.push((
                    seg_key.clone(),
                    seg_tag.clone(),
                    element_idx,
                    component_idx,
                    val,
                ));
            }

            // If there's a qualifier, also inject it at elements[0][0]
            if let Some(q) = qualifier {
                let key_upper = seg_key.clone();
                let already_has = field_values
                    .iter()
                    .any(|(k, _, ei, ci, _)| *k == key_upper && *ei == 0 && *ci == 0);
                if !already_has {
                    field_values.push((seg_key, seg_tag, 0, 0, q.to_string()));
                }
            }
        }

        // Build segments with elements/components in correct positions.
        // Group by segment_key to create separate segments for "DTM[92]" vs "DTM[93]".
        let mut segments: Vec<AssembledSegment> = Vec::new();
        let mut seen_keys: Vec<String> = Vec::new();

        for (seg_key, seg_tag, element_idx, component_idx, val) in &field_values {
            let seg = if let Some(pos) = seen_keys.iter().position(|k| k == seg_key) {
                &mut segments[pos]
            } else {
                seen_keys.push(seg_key.clone());
                segments.push(AssembledSegment {
                    tag: seg_tag.clone(),
                    elements: vec![],
                });
                segments.last_mut().unwrap()
            };

            while seg.elements.len() <= *element_idx {
                seg.elements.push(vec![]);
            }
            while seg.elements[*element_idx].len() <= *component_idx {
                seg.elements[*element_idx].push(String::new());
            }
            seg.elements[*element_idx][*component_idx] = val.clone();
        }

        AssembledGroupInstance {
            segments,
            child_groups: vec![],
        }
    }

    /// Resolve a field path within a segment to extract a value.
    ///
    /// Two path conventions are supported:
    ///
    /// **Named paths** (backward compatible):
    /// - 1-part `"d3227"` → elements\[0\]\[0\]
    /// - 2-part `"c517.d3225"` → elements\[1\]\[0\]
    ///
    /// **Numeric index paths** (for multi-component access):
    /// - `"0"` → elements\[0\]\[0\]
    /// - `"1.0"` → elements\[1\]\[0\]
    /// - `"1.2"` → elements\[1\]\[2\]
    fn resolve_field_path(segment: &AssembledSegment, path: &[&str]) -> Option<String> {
        if path.is_empty() {
            return None;
        }

        // Check if the first sub-path part is numeric → use index-based resolution
        if let Ok(element_idx) = path[0].parse::<usize>() {
            let component_idx = if path.len() > 1 {
                path[1].parse::<usize>().unwrap_or(0)
            } else {
                0
            };
            return segment
                .elements
                .get(element_idx)?
                .get(component_idx)
                .cloned();
        }

        // Named path convention
        match path.len() {
            1 => segment.elements.first()?.first().cloned(),
            2 => segment.elements.get(1)?.first().cloned(),
            _ => None,
        }
    }

    /// Extract a value from a BO4E JSON object by target field name.
    /// Supports dotted paths like "nested.field_name".
    pub fn populate_field(
        &self,
        bo4e_value: &serde_json::Value,
        target_field: &str,
        _source_path: &str,
    ) -> Option<String> {
        let parts: Vec<&str> = target_field.split('.').collect();
        let mut current = bo4e_value;
        for part in &parts {
            current = current.get(part)?;
        }
        current.as_str().map(|s| s.to_string())
    }

    /// Build a segment from BO4E values using the reverse mapping.
    pub fn build_segment_from_bo4e(
        &self,
        bo4e_value: &serde_json::Value,
        segment_tag: &str,
        target_field: &str,
    ) -> AssembledSegment {
        let value = self.populate_field(bo4e_value, target_field, "");
        let elements = if let Some(val) = value {
            vec![vec![val]]
        } else {
            vec![]
        };
        AssembledSegment {
            tag: segment_tag.to_uppercase(),
            elements,
        }
    }

    /// Build an assembled group from BO4E values and a definition.
    pub fn build_group_from_bo4e(
        &self,
        bo4e_value: &serde_json::Value,
        def: &MappingDefinition,
    ) -> AssembledGroup {
        let instance = self.map_reverse(bo4e_value, def);
        let leaf_group = def
            .meta
            .source_group
            .rsplit('.')
            .next()
            .unwrap_or(&def.meta.source_group);

        AssembledGroup {
            group_id: leaf_group.to_string(),
            repetitions: vec![instance],
        }
    }
}

/// Parse a group path part with optional repetition: "SG8:1" → ("SG8", Some(1)).
fn parse_group_spec(part: &str) -> (&str, Option<usize>) {
    if let Some(colon_pos) = part.find(':') {
        let id = &part[..colon_pos];
        let rep = part[colon_pos + 1..].parse::<usize>().ok();
        (id, rep)
    } else {
        (part, None)
    }
}

/// Parse a segment tag with optional qualifier: "dtm[92]" → ("DTM", Some("92")).
fn parse_tag_qualifier(tag_part: &str) -> (String, Option<&str>) {
    if let Some(bracket_start) = tag_part.find('[') {
        let tag = tag_part[..bracket_start].to_uppercase();
        let qualifier = tag_part[bracket_start + 1..].trim_end_matches(']');
        (tag, Some(qualifier))
    } else {
        (tag_part.to_uppercase(), None)
    }
}

/// Set a value in a nested JSON map using a dotted path.
/// E.g., "address.city" sets `{"address": {"city": "value"}}`.
fn set_nested_value(map: &mut serde_json::Map<String, serde_json::Value>, path: &str, val: String) {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() == 1 {
        map.insert(parts[0].to_string(), serde_json::Value::String(val));
        return;
    }

    // Navigate/create intermediate objects
    let mut current = map;
    for part in &parts[..parts.len() - 1] {
        let entry = current
            .entry(part.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
        current = entry.as_object_mut().expect("expected object in path");
    }
    current.insert(
        parts.last().unwrap().to_string(),
        serde_json::Value::String(val),
    );
}
