//! Mapping engine — loads TOML definitions and provides bidirectional conversion.
//!
//! Supports nested group paths (e.g., "SG4.SG5") for navigating the assembled tree
//! and provides `map_forward` / `map_reverse` for full entity conversion.

use std::path::Path;

use mig_assembly::assembler::{
    AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree,
};

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
    pub fn resolve_group_instance<'a>(
        tree: &'a AssembledTree,
        group_path: &str,
        repetition: usize,
    ) -> Option<&'a AssembledGroupInstance> {
        let parts: Vec<&str> = group_path.split('.').collect();

        let first_group = tree.groups.iter().find(|g| g.group_id == parts[0])?;

        if parts.len() == 1 {
            return first_group.repetitions.get(repetition);
        }

        // Navigate through parent groups using repetition 0
        let mut current_instance = first_group.repetitions.first()?;

        for (i, part) in parts[1..].iter().enumerate() {
            let child_group = current_instance
                .child_groups
                .iter()
                .find(|g| g.group_id == *part)?;

            if i == parts.len() - 2 {
                // Last part — use the specified repetition
                return child_group.repetitions.get(repetition);
            }
            // Intermediate — use repetition 0
            current_instance = child_group.repetitions.first()?;
        }

        None
    }

    /// Extract a field from a group instance by path.
    pub fn extract_from_instance(instance: &AssembledGroupInstance, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // First part is the segment tag
        let segment_tag = parts[0].to_uppercase();
        let segment = instance
            .segments
            .iter()
            .find(|s| s.tag.eq_ignore_ascii_case(&segment_tag))?;

        Self::resolve_field_path(segment, &parts[1..])
    }

    /// Map all fields in a definition from the assembled tree to a BO4E JSON object.
    ///
    /// `group_path` is the definition's `source_group` (may be dotted, e.g., "SG4.SG5").
    /// Returns a flat JSON object with target field names as keys.
    pub fn map_forward(
        &self,
        tree: &AssembledTree,
        def: &MappingDefinition,
        repetition: usize,
    ) -> serde_json::Value {
        let mut result = serde_json::Map::new();

        let instance = Self::resolve_group_instance(tree, &def.meta.source_group, repetition);

        if let Some(instance) = instance {
            for (path, field_mapping) in &def.fields {
                let target = match field_mapping {
                    FieldMapping::Simple(t) => t.clone(),
                    FieldMapping::Structured(s) => s.target.clone(),
                    FieldMapping::Nested(_) => continue,
                };
                if target.is_empty() {
                    continue;
                }
                if let Some(val) = Self::extract_from_instance(instance, path) {
                    set_nested_value(&mut result, &target, val);
                }
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
    /// Element placement follows the forward mapping convention:
    /// - 1-part sub-path (e.g., "d3227") → element\[0\]
    /// - 2-part sub-path (e.g., "c517.d3225") → element\[1\]
    pub fn map_reverse(
        &self,
        bo4e_value: &serde_json::Value,
        def: &MappingDefinition,
    ) -> AssembledGroupInstance {
        // Collect (segment_tag, element_index, value) tuples first,
        // then build segments in correct element order.
        let mut field_values: Vec<(String, usize, String)> = Vec::new();

        for (path, field_mapping) in &def.fields {
            let (target, default) = match field_mapping {
                FieldMapping::Simple(t) => (t.clone(), None),
                FieldMapping::Structured(s) => (s.target.clone(), s.default.clone()),
                FieldMapping::Nested(_) => continue,
            };

            let parts: Vec<&str> = path.split('.').collect();
            if parts.len() < 2 {
                continue;
            }
            let seg_tag = parts[0].to_uppercase();

            // Determine element index from sub-path length
            // (matching resolve_field_path convention)
            let sub_path_len = parts.len() - 1; // parts after segment tag
            let element_idx = match sub_path_len {
                1 => 0, // "d3227" → element[0]
                2 => 1, // "c517.d3225" → element[1]
                _ => continue,
            };

            // Try BO4E value first, fall back to default
            let val = if target.is_empty() {
                default
            } else {
                self.populate_field(bo4e_value, &target, path).or(default)
            };

            if let Some(val) = val {
                field_values.push((seg_tag, element_idx, val));
            }
        }

        // Build segments with elements in correct positions
        let mut segments: Vec<AssembledSegment> = Vec::new();

        for (seg_tag, element_idx, val) in field_values {
            let seg = segments.iter_mut().find(|s| s.tag == seg_tag);
            let seg = match seg {
                Some(existing) => existing,
                None => {
                    segments.push(AssembledSegment {
                        tag: seg_tag.clone(),
                        elements: vec![],
                    });
                    segments.last_mut().unwrap()
                }
            };

            // Extend elements vector if needed
            while seg.elements.len() <= element_idx {
                seg.elements.push(vec![]);
            }
            seg.elements[element_idx] = vec![val];
        }

        AssembledGroupInstance {
            segments,
            child_groups: vec![],
        }
    }

    fn resolve_field_path(segment: &AssembledSegment, path: &[&str]) -> Option<String> {
        // Path navigation through composites and data elements.
        // For now, use positional access: path parts map to element indices.
        // A single remaining part -> element[0][0], two parts -> element[1][0].
        match path.len() {
            1 => {
                // Direct data element: e.g. "d3227" -> element[0][0]
                segment.elements.first()?.first().cloned()
            }
            2 => {
                // Composite + data element: e.g. "c517.d3225" -> element[1][0]
                segment.elements.get(1)?.first().cloned()
            }
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
