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
use crate::segment_structure::SegmentStructure;

/// The mapping engine holds all loaded mapping definitions
/// and provides methods for bidirectional conversion.
pub struct MappingEngine {
    definitions: Vec<MappingDefinition>,
    segment_structure: Option<SegmentStructure>,
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

        Ok(Self {
            definitions,
            segment_structure: None,
        })
    }

    /// Create an engine from an already-parsed list of definitions.
    pub fn from_definitions(definitions: Vec<MappingDefinition>) -> Self {
        Self {
            definitions,
            segment_structure: None,
        }
    }

    /// Attach a MIG-derived segment structure for trailing element padding.
    ///
    /// When set, `map_reverse` pads each segment's elements up to the
    /// MIG-defined count, ensuring trailing empty elements are preserved.
    pub fn with_segment_structure(mut self, ss: SegmentStructure) -> Self {
        self.segment_structure = Some(ss);
        self
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
    ///
    /// If the definition has `companion_fields`, those are extracted into a nested
    /// object keyed by `companion_type` (or `"_companion"` if not specified).
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
            Self::extract_companion_fields(&root_instance, def, &mut result);
            return serde_json::Value::Object(result);
        }

        let instance = Self::resolve_group_instance(tree, &def.meta.source_group, repetition);

        if let Some(instance) = instance {
            Self::extract_fields_from_instance(instance, &def.fields, &mut result);
            Self::extract_companion_fields(instance, def, &mut result);
        }

        serde_json::Value::Object(result)
    }

    /// Extract companion_fields into a nested object within the result.
    fn extract_companion_fields(
        instance: &AssembledGroupInstance,
        def: &MappingDefinition,
        result: &mut serde_json::Map<String, serde_json::Value>,
    ) {
        if let Some(ref companion_fields) = def.companion_fields {
            let companion_key = def.meta.companion_type.as_deref().unwrap_or("_companion");
            let mut companion_result = serde_json::Map::new();
            Self::extract_fields_from_instance(instance, companion_fields, &mut companion_result);
            if !companion_result.is_empty() {
                result.insert(
                    companion_key.to_string(),
                    serde_json::Value::Object(companion_result),
                );
            }
        }
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

        // Process companion_fields — values are nested under the companion type key
        if let Some(ref companion_fields) = def.companion_fields {
            let companion_key = def.meta.companion_type.as_deref().unwrap_or("_companion");
            let companion_value = bo4e_value
                .get(companion_key)
                .unwrap_or(&serde_json::Value::Null);

            for (path, field_mapping) in companion_fields {
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
                let seg_key = parts[0].to_uppercase();
                let sub_path = &parts[1..];

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

                let val = if target.is_empty() {
                    default
                } else {
                    let bo4e_val = self.populate_field(companion_value, &target, path);
                    let mapped_val = match (bo4e_val, enum_map) {
                        (Some(v), Some(map)) => map
                            .iter()
                            .find(|(_, bo4e_v)| *bo4e_v == &v)
                            .map(|(edifact_k, _)| edifact_k.clone())
                            .or(Some(v)),
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

        // Pad intermediate empty elements: any [] between position 0 and the last
        // populated position becomes [""] so the EDIFACT renderer emits the `+` separator.
        for seg in &mut segments {
            let last_populated = seg.elements.iter().rposition(|e| !e.is_empty());
            if let Some(last_idx) = last_populated {
                for i in 0..last_idx {
                    if seg.elements[i].is_empty() {
                        seg.elements[i] = vec![String::new()];
                    }
                }
            }
        }

        // MIG-aware trailing padding: extend each segment to the MIG-defined element count.
        if let Some(ref ss) = self.segment_structure {
            for seg in &mut segments {
                if let Some(expected) = ss.element_count(&seg.tag) {
                    while seg.elements.len() < expected {
                        seg.elements.push(vec![String::new()]);
                    }
                }
            }
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
                .filter(|v| !v.is_empty())
                .cloned();
        }

        // Named path convention
        match path.len() {
            1 => segment
                .elements
                .first()?
                .first()
                .filter(|v| !v.is_empty())
                .cloned(),
            2 => segment
                .elements
                .get(1)?
                .first()
                .filter(|v| !v.is_empty())
                .cloned(),
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

    // ── Multi-entity forward mapping ──

    /// Parse a discriminator string (e.g., "SEQ.0.0=Z79") and find the matching
    /// repetition index within the given group path.
    ///
    /// Discriminator format: `"TAG.element_idx.component_idx=expected_value"`
    /// Scans all repetitions of the leaf group and returns the first rep index
    /// where the entry segment matches.
    pub fn resolve_repetition(
        tree: &AssembledTree,
        group_path: &str,
        discriminator: &str,
    ) -> Option<usize> {
        let (spec, expected) = discriminator.split_once('=')?;
        let parts: Vec<&str> = spec.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let tag = parts[0];
        let element_idx: usize = parts[1].parse().ok()?;
        let component_idx: usize = parts[2].parse().ok()?;

        // Navigate to the parent and get the leaf group with all its repetitions
        let path_parts: Vec<&str> = group_path.split('.').collect();

        let leaf_group = if path_parts.len() == 1 {
            let (group_id, _) = parse_group_spec(path_parts[0]);
            tree.groups.iter().find(|g| g.group_id == group_id)?
        } else {
            // Navigate to the parent instance, then find the leaf group
            let parent_parts = &path_parts[..path_parts.len() - 1];
            let mut current_instance = {
                let (first_id, first_rep) = parse_group_spec(parent_parts[0]);
                let first_group = tree.groups.iter().find(|g| g.group_id == first_id)?;
                first_group.repetitions.get(first_rep.unwrap_or(0))?
            };
            for part in &parent_parts[1..] {
                let (group_id, explicit_rep) = parse_group_spec(part);
                let child_group = current_instance
                    .child_groups
                    .iter()
                    .find(|g| g.group_id == group_id)?;
                current_instance = child_group.repetitions.get(explicit_rep.unwrap_or(0))?;
            }
            let (leaf_id, _) = parse_group_spec(path_parts.last()?);
            current_instance
                .child_groups
                .iter()
                .find(|g| g.group_id == leaf_id)?
        };

        // Scan all repetitions for the matching discriminator
        for (rep_idx, instance) in leaf_group.repetitions.iter().enumerate() {
            let matches = instance.segments.iter().any(|s| {
                s.tag.eq_ignore_ascii_case(tag)
                    && s.elements
                        .get(element_idx)
                        .and_then(|e| e.get(component_idx))
                        .map(|v| v == expected)
                        .unwrap_or(false)
            });
            if matches {
                return Some(rep_idx);
            }
        }

        None
    }

    /// Map all definitions against a tree, returning a JSON object with entity names as keys.
    ///
    /// For each definition:
    /// - Has discriminator → find matching rep via `resolve_repetition`, map single instance
    /// - Root-level (empty source_group) → map rep 0 as single object
    /// - No discriminator, 1 rep in tree → map as single object
    /// - No discriminator, multiple reps in tree → map ALL reps into a JSON array
    ///
    /// When multiple definitions share the same `entity` name, their fields are
    /// deep-merged into a single JSON object. This allows related TOML files
    /// (e.g., LOC location + SEQ info + SG10 characteristics) to contribute
    /// fields to the same BO4E entity.
    pub fn map_all_forward(&self, tree: &AssembledTree) -> serde_json::Value {
        let mut result = serde_json::Map::new();

        for def in &self.definitions {
            let entity = &def.meta.entity;

            let bo4e = if let Some(ref disc) = def.meta.discriminator {
                // Has discriminator — resolve to specific rep
                Self::resolve_repetition(tree, &def.meta.source_group, disc)
                    .map(|rep| self.map_forward(tree, def, rep))
            } else if def.meta.source_group.is_empty() {
                // Root-level mapping — always single object
                Some(self.map_forward(tree, def, 0))
            } else {
                let num_reps = Self::count_repetitions(tree, &def.meta.source_group);
                if num_reps <= 1 {
                    Some(self.map_forward(tree, def, 0))
                } else {
                    // Multiple reps, no discriminator — map all into array
                    let mut items = Vec::new();
                    for rep in 0..num_reps {
                        items.push(self.map_forward(tree, def, rep));
                    }
                    Some(serde_json::Value::Array(items))
                }
            };

            if let Some(bo4e) = bo4e {
                let bo4e = inject_bo4e_metadata(bo4e, &def.meta.bo4e_type);
                deep_merge_insert(&mut result, entity, bo4e);
            }
        }

        serde_json::Value::Object(result)
    }

    /// Count the number of repetitions available for a group path in the tree.
    fn count_repetitions(tree: &AssembledTree, group_path: &str) -> usize {
        let parts: Vec<&str> = group_path.split('.').collect();

        let (first_id, first_rep) = parse_group_spec(parts[0]);
        let first_group = match tree.groups.iter().find(|g| g.group_id == first_id) {
            Some(g) => g,
            None => return 0,
        };

        if parts.len() == 1 {
            return first_group.repetitions.len();
        }

        // Navigate to parent, then count leaf group reps
        let mut current_instance = match first_group.repetitions.get(first_rep.unwrap_or(0)) {
            Some(i) => i,
            None => return 0,
        };

        for (i, part) in parts[1..].iter().enumerate() {
            let (group_id, explicit_rep) = parse_group_spec(part);
            let child_group = match current_instance
                .child_groups
                .iter()
                .find(|g| g.group_id == group_id)
            {
                Some(g) => g,
                None => return 0,
            };

            if i == parts.len() - 2 {
                // Last part — return rep count
                return child_group.repetitions.len();
            }
            current_instance = match child_group.repetitions.get(explicit_rep.unwrap_or(0)) {
                Some(i) => i,
                None => return 0,
            };
        }

        0
    }

    /// Map an assembled tree into message-level and transaction-level results.
    ///
    /// - `msg_engine`: MappingEngine loaded with message-level definitions (SG2, SG3, root segments)
    /// - `tx_engine`: MappingEngine loaded with transaction-level definitions (relative to SG4)
    /// - `tree`: The assembled tree for one message
    /// - `transaction_group`: The group ID that represents transactions (e.g., "SG4")
    ///
    /// Returns a `MappedMessage` with message stammdaten and per-transaction results.
    pub fn map_interchange(
        msg_engine: &MappingEngine,
        tx_engine: &MappingEngine,
        tree: &AssembledTree,
        transaction_group: &str,
    ) -> crate::model::MappedMessage {
        // Map message-level entities
        let stammdaten = msg_engine.map_all_forward(tree);

        // Find the transaction group and map each repetition
        let transaktionen = tree
            .groups
            .iter()
            .find(|g| g.group_id == transaction_group)
            .map(|sg| {
                sg.repetitions
                    .iter()
                    .map(|instance| {
                        let sub_tree = instance.as_assembled_tree();
                        let tx_result = tx_engine.map_all_forward(&sub_tree);

                        // Split: "Prozessdaten" entity goes into transaktionsdaten,
                        // everything else into stammdaten
                        let mut tx_stammdaten = serde_json::Map::new();
                        let mut transaktionsdaten = serde_json::Value::Null;

                        if let Some(obj) = tx_result.as_object() {
                            for (key, value) in obj {
                                if key == "Prozessdaten" || key == "Nachricht" {
                                    // Merge into transaktionsdaten
                                    if transaktionsdaten.is_null() {
                                        transaktionsdaten = value.clone();
                                    } else if let (Some(existing), Some(new_map)) =
                                        (transaktionsdaten.as_object_mut(), value.as_object())
                                    {
                                        for (k, v) in new_map {
                                            existing.entry(k.clone()).or_insert(v.clone());
                                        }
                                    }
                                } else {
                                    tx_stammdaten.insert(key.clone(), value.clone());
                                }
                            }
                        }

                        crate::model::Transaktion {
                            stammdaten: serde_json::Value::Object(tx_stammdaten),
                            transaktionsdaten,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        crate::model::MappedMessage {
            stammdaten,
            transaktionen,
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

/// Inject `boTyp` and `versionStruktur` metadata into a BO4E JSON value.
///
/// For objects, inserts both fields (without overwriting existing ones).
/// For arrays, injects into each element object.
fn inject_bo4e_metadata(mut value: serde_json::Value, bo4e_type: &str) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(map) => {
            map.entry("boTyp")
                .or_insert_with(|| serde_json::Value::String(bo4e_type.to_uppercase()));
            map.entry("versionStruktur")
                .or_insert_with(|| serde_json::Value::String("1".to_string()));
        }
        serde_json::Value::Array(items) => {
            for item in items {
                if let serde_json::Value::Object(map) = item {
                    map.entry("boTyp")
                        .or_insert_with(|| serde_json::Value::String(bo4e_type.to_uppercase()));
                    map.entry("versionStruktur")
                        .or_insert_with(|| serde_json::Value::String("1".to_string()));
                }
            }
        }
        _ => {}
    }
    value
}

/// Deep-merge a BO4E value into the result map.
///
/// If the entity already exists as an object, new fields are merged in
/// (existing fields are NOT overwritten). This allows multiple TOML
/// definitions with the same `entity` name to contribute fields to one object.
fn deep_merge_insert(
    result: &mut serde_json::Map<String, serde_json::Value>,
    entity: &str,
    bo4e: serde_json::Value,
) {
    if let Some(existing) = result.get_mut(entity) {
        if let (Some(existing_map), serde_json::Value::Object(new_map)) =
            (existing.as_object_mut(), &bo4e)
        {
            for (k, v) in new_map {
                if let Some(existing_v) = existing_map.get_mut(k) {
                    // Recursively merge nested objects (e.g., companion types)
                    if let (Some(existing_inner), Some(new_inner)) =
                        (existing_v.as_object_mut(), v.as_object())
                    {
                        for (ik, iv) in new_inner {
                            existing_inner
                                .entry(ik.clone())
                                .or_insert_with(|| iv.clone());
                        }
                    }
                    // Don't overwrite existing scalar/array values
                } else {
                    existing_map.insert(k.clone(), v.clone());
                }
            }
            return;
        }
    }
    result.insert(entity.to_string(), bo4e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{MappingDefinition, MappingMeta, StructuredFieldMapping};
    use std::collections::BTreeMap;

    fn make_def(fields: BTreeMap<String, FieldMapping>) -> MappingDefinition {
        MappingDefinition {
            meta: MappingMeta {
                entity: "Test".to_string(),
                bo4e_type: "Test".to_string(),
                companion_type: None,
                source_group: "SG4".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields,
            companion_fields: None,
            complex_handlers: None,
        }
    }

    #[test]
    fn test_map_interchange_single_transaction_backward_compat() {
        use mig_assembly::assembler::*;

        // Single SG4 with SG5 — the common case for current PID 55001 fixtures
        let tree = AssembledTree {
            segments: vec![
                AssembledSegment {
                    tag: "UNH".to_string(),
                    elements: vec![vec!["001".to_string()]],
                },
                AssembledSegment {
                    tag: "BGM".to_string(),
                    elements: vec![
                        vec!["E01".to_string()],
                        vec!["DOC001".to_string()],
                    ],
                },
            ],
            groups: vec![
                AssembledGroup {
                    group_id: "SG2".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "NAD".to_string(),
                            elements: vec![
                                vec!["MS".to_string()],
                                vec!["9900123".to_string()],
                            ],
                        }],
                        child_groups: vec![],
                    }],
                },
                AssembledGroup {
                    group_id: "SG4".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![
                                vec!["24".to_string()],
                                vec!["TX001".to_string()],
                            ],
                        }],
                        child_groups: vec![AssembledGroup {
                            group_id: "SG5".to_string(),
                            repetitions: vec![AssembledGroupInstance {
                                segments: vec![AssembledSegment {
                                    tag: "LOC".to_string(),
                                    elements: vec![
                                        vec!["Z16".to_string()],
                                        vec!["DE000111222333".to_string()],
                                    ],
                                }],
                                child_groups: vec![],
                            }],
                        }],
                    }],
                },
            ],
            post_group_start: 2,
        };

        // Empty message engine (no message-level defs for this test)
        let msg_engine = MappingEngine::from_definitions(vec![]);

        // Transaction defs
        let mut tx_fields = BTreeMap::new();
        tx_fields.insert(
            "ide.1".to_string(),
            FieldMapping::Simple("vorgangId".to_string()),
        );
        let mut malo_fields = BTreeMap::new();
        malo_fields.insert(
            "loc.1".to_string(),
            FieldMapping::Simple("marktlokationsId".to_string()),
        );

        let tx_engine = MappingEngine::from_definitions(vec![
            MappingDefinition {
                meta: MappingMeta {
                    entity: "Prozessdaten".to_string(),
                    bo4e_type: "Prozessdaten".to_string(),
                    companion_type: None,
                    source_group: "".to_string(),
                    source_path: None,
                    discriminator: None,
                },
                fields: tx_fields,
                companion_fields: None,
                complex_handlers: None,
            },
            MappingDefinition {
                meta: MappingMeta {
                    entity: "Marktlokation".to_string(),
                    bo4e_type: "Marktlokation".to_string(),
                    companion_type: None,
                    source_group: "SG5".to_string(),
                    source_path: None,
                    discriminator: None,
                },
                fields: malo_fields,
                companion_fields: None,
                complex_handlers: None,
            },
        ]);

        let result = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4");

        assert_eq!(result.transaktionen.len(), 1);
        assert_eq!(
            result.transaktionen[0].transaktionsdaten["vorgangId"]
                .as_str()
                .unwrap(),
            "TX001"
        );
        assert_eq!(
            result.transaktionen[0].stammdaten["Marktlokation"]["marktlokationsId"]
                .as_str()
                .unwrap(),
            "DE000111222333"
        );
    }

    #[test]
    fn test_map_reverse_pads_intermediate_empty_elements() {
        // NAD+Z09+++Muster:Max — positions 0 and 3 populated, 1 and 2 should become [""]
        let mut fields = BTreeMap::new();
        fields.insert(
            "nad.0".to_string(),
            FieldMapping::Structured(StructuredFieldMapping {
                target: String::new(),
                transform: None,
                when: None,
                default: Some("Z09".to_string()),
                enum_map: None,
            }),
        );
        fields.insert(
            "nad.3.0".to_string(),
            FieldMapping::Simple("name".to_string()),
        );
        fields.insert(
            "nad.3.1".to_string(),
            FieldMapping::Simple("vorname".to_string()),
        );

        let def = make_def(fields);
        let engine = MappingEngine::from_definitions(vec![]);

        let bo4e = serde_json::json!({
            "name": "Muster",
            "vorname": "Max"
        });

        let instance = engine.map_reverse(&bo4e, &def);
        assert_eq!(instance.segments.len(), 1);

        let nad = &instance.segments[0];
        assert_eq!(nad.tag, "NAD");
        assert_eq!(nad.elements.len(), 4);
        assert_eq!(nad.elements[0], vec!["Z09"]);
        // Intermediate positions 1 and 2 should be padded to [""]
        assert_eq!(nad.elements[1], vec![""]);
        assert_eq!(nad.elements[2], vec![""]);
        assert_eq!(nad.elements[3][0], "Muster");
        assert_eq!(nad.elements[3][1], "Max");
    }

    #[test]
    fn test_map_reverse_no_padding_when_contiguous() {
        // DTM+92:20250531:303 — all three components in element 0, no gaps
        let mut fields = BTreeMap::new();
        fields.insert(
            "dtm.0.0".to_string(),
            FieldMapping::Structured(StructuredFieldMapping {
                target: String::new(),
                transform: None,
                when: None,
                default: Some("92".to_string()),
                enum_map: None,
            }),
        );
        fields.insert(
            "dtm.0.1".to_string(),
            FieldMapping::Simple("value".to_string()),
        );
        fields.insert(
            "dtm.0.2".to_string(),
            FieldMapping::Structured(StructuredFieldMapping {
                target: String::new(),
                transform: None,
                when: None,
                default: Some("303".to_string()),
                enum_map: None,
            }),
        );

        let def = make_def(fields);
        let engine = MappingEngine::from_definitions(vec![]);

        let bo4e = serde_json::json!({ "value": "20250531" });

        let instance = engine.map_reverse(&bo4e, &def);
        let dtm = &instance.segments[0];
        // Single element with 3 components — no intermediate padding needed
        assert_eq!(dtm.elements.len(), 1);
        assert_eq!(dtm.elements[0], vec!["92", "20250531", "303"]);
    }

    #[test]
    fn test_map_message_level_extracts_sg2_only() {
        use mig_assembly::assembler::*;

        // Build a tree with SG2 (message-level) and SG4 (transaction-level)
        let tree = AssembledTree {
            segments: vec![
                AssembledSegment {
                    tag: "UNH".to_string(),
                    elements: vec![vec!["001".to_string()]],
                },
                AssembledSegment {
                    tag: "BGM".to_string(),
                    elements: vec![vec!["E01".to_string()]],
                },
            ],
            groups: vec![
                AssembledGroup {
                    group_id: "SG2".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "NAD".to_string(),
                            elements: vec![
                                vec!["MS".to_string()],
                                vec!["9900123".to_string()],
                            ],
                        }],
                        child_groups: vec![],
                    }],
                },
                AssembledGroup {
                    group_id: "SG4".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![
                                vec!["24".to_string()],
                                vec!["TX001".to_string()],
                            ],
                        }],
                        child_groups: vec![],
                    }],
                },
            ],
            post_group_start: 2,
        };

        // Message-level definition maps SG2
        let mut msg_fields = BTreeMap::new();
        msg_fields.insert(
            "nad.0".to_string(),
            FieldMapping::Simple("marktrolle".to_string()),
        );
        msg_fields.insert(
            "nad.1".to_string(),
            FieldMapping::Simple("rollencodenummer".to_string()),
        );
        let msg_def = MappingDefinition {
            meta: MappingMeta {
                entity: "Marktteilnehmer".to_string(),
                bo4e_type: "Marktteilnehmer".to_string(),
                companion_type: None,
                source_group: "SG2".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: msg_fields,
            companion_fields: None,
            complex_handlers: None,
        };

        let engine = MappingEngine::from_definitions(vec![msg_def.clone()]);
        let result = engine.map_all_forward(&tree);

        // Should contain Marktteilnehmer from SG2
        assert!(result.get("Marktteilnehmer").is_some());
        let mt = &result["Marktteilnehmer"];
        assert_eq!(mt["marktrolle"].as_str().unwrap(), "MS");
        assert_eq!(mt["rollencodenummer"].as_str().unwrap(), "9900123");
    }

    #[test]
    fn test_map_transaction_scoped_to_sg4_instance() {
        use mig_assembly::assembler::*;

        // Build a tree with SG4 containing SG5 (LOC+Z16)
        let tree = AssembledTree {
            segments: vec![
                AssembledSegment {
                    tag: "UNH".to_string(),
                    elements: vec![vec!["001".to_string()]],
                },
                AssembledSegment {
                    tag: "BGM".to_string(),
                    elements: vec![vec!["E01".to_string()]],
                },
            ],
            groups: vec![AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![AssembledSegment {
                        tag: "IDE".to_string(),
                        elements: vec![
                            vec!["24".to_string()],
                            vec!["TX001".to_string()],
                        ],
                    }],
                    child_groups: vec![AssembledGroup {
                        group_id: "SG5".to_string(),
                        repetitions: vec![AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "LOC".to_string(),
                                elements: vec![
                                    vec!["Z16".to_string()],
                                    vec!["DE000111222333".to_string()],
                                ],
                            }],
                            child_groups: vec![],
                        }],
                    }],
                }],
            }],
            post_group_start: 2,
        };

        // Transaction-level definitions: prozessdaten (root of SG4) + marktlokation (SG5)
        let mut proz_fields = BTreeMap::new();
        proz_fields.insert(
            "ide.1".to_string(),
            FieldMapping::Simple("vorgangId".to_string()),
        );
        let proz_def = MappingDefinition {
            meta: MappingMeta {
                entity: "Prozessdaten".to_string(),
                bo4e_type: "Prozessdaten".to_string(),
                companion_type: None,
                source_group: "".to_string(), // Root-level within transaction sub-tree
                source_path: None,
                discriminator: None,
            },
            fields: proz_fields,
            companion_fields: None,
            complex_handlers: None,
        };

        let mut malo_fields = BTreeMap::new();
        malo_fields.insert(
            "loc.1".to_string(),
            FieldMapping::Simple("marktlokationsId".to_string()),
        );
        let malo_def = MappingDefinition {
            meta: MappingMeta {
                entity: "Marktlokation".to_string(),
                bo4e_type: "Marktlokation".to_string(),
                companion_type: None,
                source_group: "SG5".to_string(), // Relative to SG4, not "SG4.SG5"
                source_path: None,
                discriminator: None,
            },
            fields: malo_fields,
            companion_fields: None,
            complex_handlers: None,
        };

        let tx_engine = MappingEngine::from_definitions(vec![proz_def, malo_def]);

        // Scope to the SG4 instance and map
        let sg4 = &tree.groups[0]; // SG4 group
        let sg4_instance = &sg4.repetitions[0];
        let sub_tree = sg4_instance.as_assembled_tree();

        let result = tx_engine.map_all_forward(&sub_tree);

        // Should contain Prozessdaten from SG4 root segments
        assert_eq!(
            result["Prozessdaten"]["vorgangId"].as_str().unwrap(),
            "TX001"
        );

        // Should contain Marktlokation from SG5 within SG4
        assert_eq!(
            result["Marktlokation"]["marktlokationsId"].as_str().unwrap(),
            "DE000111222333"
        );
    }

    #[test]
    fn test_map_interchange_produces_full_hierarchy() {
        use mig_assembly::assembler::*;

        // Build a tree with SG2 (message-level) and SG4 with two repetitions (two transactions)
        let tree = AssembledTree {
            segments: vec![
                AssembledSegment {
                    tag: "UNH".to_string(),
                    elements: vec![vec!["001".to_string()]],
                },
                AssembledSegment {
                    tag: "BGM".to_string(),
                    elements: vec![vec!["E01".to_string()]],
                },
            ],
            groups: vec![
                AssembledGroup {
                    group_id: "SG2".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "NAD".to_string(),
                            elements: vec![
                                vec!["MS".to_string()],
                                vec!["9900123".to_string()],
                            ],
                        }],
                        child_groups: vec![],
                    }],
                },
                AssembledGroup {
                    group_id: "SG4".to_string(),
                    repetitions: vec![
                        AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "IDE".to_string(),
                                elements: vec![
                                    vec!["24".to_string()],
                                    vec!["TX001".to_string()],
                                ],
                            }],
                            child_groups: vec![],
                        },
                        AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "IDE".to_string(),
                                elements: vec![
                                    vec!["24".to_string()],
                                    vec!["TX002".to_string()],
                                ],
                            }],
                            child_groups: vec![],
                        },
                    ],
                },
            ],
            post_group_start: 2,
        };

        // Message-level definitions
        let mut msg_fields = BTreeMap::new();
        msg_fields.insert(
            "nad.0".to_string(),
            FieldMapping::Simple("marktrolle".to_string()),
        );
        let msg_defs = vec![MappingDefinition {
            meta: MappingMeta {
                entity: "Marktteilnehmer".to_string(),
                bo4e_type: "Marktteilnehmer".to_string(),
                companion_type: None,
                source_group: "SG2".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: msg_fields,
            companion_fields: None,
            complex_handlers: None,
        }];

        // Transaction-level definitions
        let mut tx_fields = BTreeMap::new();
        tx_fields.insert(
            "ide.1".to_string(),
            FieldMapping::Simple("vorgangId".to_string()),
        );
        let tx_defs = vec![MappingDefinition {
            meta: MappingMeta {
                entity: "Prozessdaten".to_string(),
                bo4e_type: "Prozessdaten".to_string(),
                companion_type: None,
                source_group: "".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: tx_fields,
            companion_fields: None,
            complex_handlers: None,
        }];

        let msg_engine = MappingEngine::from_definitions(msg_defs);
        let tx_engine = MappingEngine::from_definitions(tx_defs);

        let result = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4");

        // Message-level stammdaten
        assert!(result.stammdaten["Marktteilnehmer"].is_object());
        assert_eq!(
            result.stammdaten["Marktteilnehmer"]["marktrolle"]
                .as_str()
                .unwrap(),
            "MS"
        );

        // Two transactions
        assert_eq!(result.transaktionen.len(), 2);
        assert_eq!(
            result.transaktionen[0].transaktionsdaten["vorgangId"]
                .as_str()
                .unwrap(),
            "TX001"
        );
        assert_eq!(
            result.transaktionen[1].transaktionsdaten["vorgangId"]
                .as_str()
                .unwrap(),
            "TX002"
        );
    }

    #[test]
    fn test_map_reverse_with_segment_structure_pads_trailing() {
        // STS+7++E01 — position 0 and 2 populated, MIG says 5 elements
        let mut fields = BTreeMap::new();
        fields.insert(
            "sts.0".to_string(),
            FieldMapping::Structured(StructuredFieldMapping {
                target: String::new(),
                transform: None,
                when: None,
                default: Some("7".to_string()),
                enum_map: None,
            }),
        );
        fields.insert(
            "sts.2".to_string(),
            FieldMapping::Simple("grund".to_string()),
        );

        let def = make_def(fields);

        // Build a SegmentStructure manually via HashMap
        let mut counts = std::collections::HashMap::new();
        counts.insert("STS".to_string(), 5usize);
        let ss = SegmentStructure {
            element_counts: counts,
        };

        let engine = MappingEngine::from_definitions(vec![]).with_segment_structure(ss);

        let bo4e = serde_json::json!({ "grund": "E01" });

        let instance = engine.map_reverse(&bo4e, &def);
        let sts = &instance.segments[0];
        // Should have 5 elements: pos 0 = ["7"], pos 1 = [""] (intermediate pad),
        // pos 2 = ["E01"], pos 3 = [""] (trailing pad), pos 4 = [""] (trailing pad)
        assert_eq!(sts.elements.len(), 5);
        assert_eq!(sts.elements[0], vec!["7"]);
        assert_eq!(sts.elements[1], vec![""]);
        assert_eq!(sts.elements[2], vec!["E01"]);
        assert_eq!(sts.elements[3], vec![""]);
        assert_eq!(sts.elements[4], vec![""]);
    }
}
