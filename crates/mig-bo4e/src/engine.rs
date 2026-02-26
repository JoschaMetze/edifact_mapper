//! Mapping engine — loads TOML definitions and provides bidirectional conversion.
//!
//! Supports nested group paths (e.g., "SG4.SG5") for navigating the assembled tree
//! and provides `map_forward` / `map_reverse` for full entity conversion.

use std::collections::{HashMap, HashSet};
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
    code_lookup: Option<crate::code_lookup::CodeLookup>,
}

impl MappingEngine {
    /// Load all TOML mapping files from a directory.
    pub fn load(dir: &Path) -> Result<Self, MappingError> {
        let mut definitions = Vec::new();

        let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
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
            code_lookup: None,
        })
    }

    /// Load message-level and transaction-level TOML mappings from separate directories.
    ///
    /// Returns `(message_engine, transaction_engine)` where:
    /// - `message_engine` maps SG2/SG3/root-level definitions (shared across PIDs)
    /// - `transaction_engine` maps SG4+ definitions (PID-specific)
    pub fn load_split(
        message_dir: &Path,
        transaction_dir: &Path,
    ) -> Result<(Self, Self), MappingError> {
        let msg_engine = Self::load(message_dir)?;
        let tx_engine = Self::load(transaction_dir)?;
        Ok((msg_engine, tx_engine))
    }

    /// Load TOML mapping files from multiple directories into a single engine.
    ///
    /// Useful for combining message-level and transaction-level mappings
    /// when a single engine with all definitions is needed.
    pub fn load_merged(dirs: &[&Path]) -> Result<Self, MappingError> {
        let mut definitions = Vec::new();
        for dir in dirs {
            let engine = Self::load(dir)?;
            definitions.extend(engine.definitions);
        }
        Ok(Self {
            definitions,
            segment_structure: None,
            code_lookup: None,
        })
    }

    /// Create an engine from an already-parsed list of definitions.
    pub fn from_definitions(definitions: Vec<MappingDefinition>) -> Self {
        Self {
            definitions,
            segment_structure: None,
            code_lookup: None,
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

    /// Attach a code lookup for enriching companion field values.
    ///
    /// When set, companion fields that map to code-type elements in the PID schema
    /// are emitted as `{"code": "Z15", "meaning": "Ja"}` objects instead of plain strings.
    pub fn with_code_lookup(mut self, cl: crate::code_lookup::CodeLookup) -> Self {
        self.code_lookup = Some(cl);
        self
    }

    /// Attach a path resolver to normalize EDIFACT ID paths to numeric indices.
    ///
    /// This allows TOML mapping files to use named paths like `loc.c517.d3225`
    /// instead of numeric indices like `loc.1.0`. Resolution happens once at
    /// load time — the engine hot path is completely unchanged.
    pub fn with_path_resolver(mut self, resolver: crate::path_resolver::PathResolver) -> Self {
        for def in &mut self.definitions {
            def.normalize_paths(&resolver);
        }
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

    /// Navigate the assembled tree using a source_path with qualifier suffixes.
    ///
    /// Source paths like `"sg4.sg8_z98.sg10"` encode qualifiers inline:
    /// `sg8_z98` means "find the SG8 repetition whose entry segment has qualifier Z98".
    /// Parts without underscores (e.g., `sg4`, `sg10`) use the first repetition.
    ///
    /// Returns `None` if any part of the path can't be resolved.
    pub fn resolve_by_source_path<'a>(
        tree: &'a AssembledTree,
        source_path: &str,
    ) -> Option<&'a AssembledGroupInstance> {
        let parts: Vec<&str> = source_path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let (first_id, first_qualifier) = parse_source_path_part(parts[0]);
        let first_group = tree
            .groups
            .iter()
            .find(|g| g.group_id.eq_ignore_ascii_case(first_id))?;

        let mut current_instance = if let Some(q) = first_qualifier {
            find_rep_by_entry_qualifier(&first_group.repetitions, q)?
        } else {
            first_group.repetitions.first()?
        };

        if parts.len() == 1 {
            return Some(current_instance);
        }

        for part in &parts[1..] {
            let (group_id, qualifier) = parse_source_path_part(part);
            let child_group = current_instance
                .child_groups
                .iter()
                .find(|g| g.group_id.eq_ignore_ascii_case(group_id))?;

            current_instance = if let Some(q) = qualifier {
                find_rep_by_entry_qualifier(&child_group.repetitions, q)?
            } else {
                child_group.repetitions.first()?
            };
        }

        Some(current_instance)
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
        self.map_forward_inner(tree, def, repetition, true)
    }

    /// Inner implementation with enrichment control.
    fn map_forward_inner(
        &self,
        tree: &AssembledTree,
        def: &MappingDefinition,
        repetition: usize,
        enrich_codes: bool,
    ) -> serde_json::Value {
        let mut result = serde_json::Map::new();

        // Root-level mapping: source_group is empty → use tree's own segments
        if def.meta.source_group.is_empty() {
            let root_instance = AssembledGroupInstance {
                segments: tree.segments[..tree.post_group_start].to_vec(),
                child_groups: vec![],
            };
            self.extract_fields_from_instance(&root_instance, def, &mut result, enrich_codes);
            self.extract_companion_fields(&root_instance, def, &mut result, enrich_codes);
            return serde_json::Value::Object(result);
        }

        // Try source_path-based resolution when:
        //   1. source_path has qualifier suffixes (e.g., "sg4.sg8_z98.sg10")
        //   2. source_group has no explicit :N indices (those take priority)
        // This allows definitions without positional indices to navigate via
        // entry-segment qualifiers (e.g., SEQ qualifier Z98).
        let instance = if let Some(ref sp) = def.meta.source_path {
            if has_source_path_qualifiers(sp) && !def.meta.source_group.contains(':') {
                Self::resolve_by_source_path(tree, sp).or_else(|| {
                    Self::resolve_group_instance(tree, &def.meta.source_group, repetition)
                })
            } else {
                Self::resolve_group_instance(tree, &def.meta.source_group, repetition)
            }
        } else {
            Self::resolve_group_instance(tree, &def.meta.source_group, repetition)
        };

        if let Some(instance) = instance {
            self.extract_fields_from_instance(instance, def, &mut result, enrich_codes);
            self.extract_companion_fields(instance, def, &mut result, enrich_codes);
        }

        serde_json::Value::Object(result)
    }

    /// Extract companion_fields into a nested object within the result.
    ///
    /// When a `code_lookup` is configured, code-type fields are emitted as
    /// `{"code": "Z15", "meaning": "Ja"}` objects. Data-type fields remain plain strings.
    fn extract_companion_fields(
        &self,
        instance: &AssembledGroupInstance,
        def: &MappingDefinition,
        result: &mut serde_json::Map<String, serde_json::Value>,
        enrich_codes: bool,
    ) {
        if let Some(ref companion_fields) = def.companion_fields {
            let raw_key = def.meta.companion_type.as_deref().unwrap_or("_companion");
            let companion_key = to_camel_case(raw_key);
            let mut companion_result = serde_json::Map::new();

            for (path, field_mapping) in companion_fields {
                let (target, enum_map) = match field_mapping {
                    FieldMapping::Simple(t) => (t.as_str(), None),
                    FieldMapping::Structured(s) => (s.target.as_str(), s.enum_map.as_ref()),
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

                    // Enrich code fields with meaning from PID schema
                    if enrich_codes {
                        if let (Some(ref code_lookup), Some(ref source_path)) =
                            (&self.code_lookup, &def.meta.source_path)
                        {
                            let parts: Vec<&str> = path.split('.').collect();
                            let (seg_tag, _qualifier) = parse_tag_qualifier(parts[0]);
                            let (element_idx, component_idx) =
                                Self::parse_element_component(&parts[1..]);

                            if code_lookup.is_code_field(
                                source_path,
                                &seg_tag,
                                element_idx,
                                component_idx,
                            ) {
                                let meaning = code_lookup
                                    .meaning_for(
                                        source_path,
                                        &seg_tag,
                                        element_idx,
                                        component_idx,
                                        &mapped_val,
                                    )
                                    .map(|m| serde_json::Value::String(m.to_string()))
                                    .unwrap_or(serde_json::Value::Null);

                                let enriched = serde_json::json!({
                                    "code": mapped_val,
                                    "meaning": meaning,
                                });
                                set_nested_value_json(&mut companion_result, target, enriched);
                                continue;
                            }
                        }
                    }

                    set_nested_value(&mut companion_result, target, mapped_val);
                }
            }

            if !companion_result.is_empty() {
                result.insert(
                    companion_key.to_string(),
                    serde_json::Value::Object(companion_result),
                );
            }
        }
    }

    /// Extract all fields from an instance into a result map.
    ///
    /// When a `code_lookup` is configured, code-type fields are emitted as
    /// `{"code": "E01", "meaning": "..."}` objects. Data-type fields remain plain strings.
    fn extract_fields_from_instance(
        &self,
        instance: &AssembledGroupInstance,
        def: &MappingDefinition,
        result: &mut serde_json::Map<String, serde_json::Value>,
        enrich_codes: bool,
    ) {
        for (path, field_mapping) in &def.fields {
            let (target, enum_map) = match field_mapping {
                FieldMapping::Simple(t) => (t.as_str(), None),
                FieldMapping::Structured(s) => (s.target.as_str(), s.enum_map.as_ref()),
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

                // Enrich code fields with meaning from PID schema
                if enrich_codes {
                    if let (Some(ref code_lookup), Some(ref source_path)) =
                        (&self.code_lookup, &def.meta.source_path)
                    {
                        let parts: Vec<&str> = path.split('.').collect();
                        let (seg_tag, _qualifier) = parse_tag_qualifier(parts[0]);
                        let (element_idx, component_idx) =
                            Self::parse_element_component(&parts[1..]);

                        if code_lookup.is_code_field(
                            source_path,
                            &seg_tag,
                            element_idx,
                            component_idx,
                        ) {
                            let meaning = code_lookup
                                .meaning_for(
                                    source_path,
                                    &seg_tag,
                                    element_idx,
                                    component_idx,
                                    &mapped_val,
                                )
                                .map(|m| serde_json::Value::String(m.to_string()))
                                .unwrap_or(serde_json::Value::Null);

                            let enriched = serde_json::json!({
                                "code": mapped_val,
                                "meaning": meaning,
                            });
                            set_nested_value_json(result, target, enriched);
                            continue;
                        }
                    }
                }

                set_nested_value(result, target, mapped_val);
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
        self.extract_fields_from_instance(&instance, def, &mut result, true);
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
        let mut field_values: Vec<(String, String, usize, usize, String)> =
            Vec::with_capacity(def.fields.len());

        // Track whether any field with a non-empty target resolved to an actual
        // BO4E value.  When a definition has data fields but none resolved to
        // values, only defaults (qualifiers) would be emitted — producing phantom
        // segments for groups not present in the original EDIFACT message.
        // Definitions with ONLY qualifier/default fields (no data targets) are
        // "container" definitions (e.g., SEQ entry segments) and are always kept.
        let mut has_real_data = false;
        let mut has_data_fields = false;
        // Per-segment phantom tracking: segments with data fields but no resolved
        // data are phantoms — their entries should be removed from field_values.
        let mut seg_has_data_field: HashSet<String> = HashSet::new();
        let mut seg_has_real_data: HashSet<String> = HashSet::new();

        for (path, field_mapping) in &def.fields {
            let (target, default, enum_map) = match field_mapping {
                FieldMapping::Simple(t) => (t.as_str(), None, None),
                FieldMapping::Structured(s) => {
                    (s.target.as_str(), s.default.as_ref(), s.enum_map.as_ref())
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
                default.cloned()
            } else {
                has_data_fields = true;
                seg_has_data_field.insert(seg_key.clone());
                let bo4e_val = self.populate_field(bo4e_value, target, path);
                if bo4e_val.is_some() {
                    has_real_data = true;
                    seg_has_real_data.insert(seg_key.clone());
                }
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
                mapped_val.or_else(|| default.cloned())
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
                let already_has = field_values
                    .iter()
                    .any(|(k, _, ei, ci, _)| *k == seg_key && *ei == 0 && *ci == 0);
                if !already_has {
                    field_values.push((seg_key, seg_tag, 0, 0, q.to_string()));
                }
            }
        }

        // Process companion_fields — values are nested under the companion type key
        if let Some(ref companion_fields) = def.companion_fields {
            let raw_key = def.meta.companion_type.as_deref().unwrap_or("_companion");
            let companion_key = to_camel_case(raw_key);
            let companion_value = bo4e_value
                .get(&companion_key)
                .unwrap_or(&serde_json::Value::Null);

            for (path, field_mapping) in companion_fields {
                let (target, default, enum_map) = match field_mapping {
                    FieldMapping::Simple(t) => (t.as_str(), None, None),
                    FieldMapping::Structured(s) => {
                        (s.target.as_str(), s.default.as_ref(), s.enum_map.as_ref())
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
                    default.cloned()
                } else {
                    has_data_fields = true;
                    seg_has_data_field.insert(seg_key.clone());
                    let bo4e_val = self.populate_field(companion_value, target, path);
                    if bo4e_val.is_some() {
                        has_real_data = true;
                        seg_has_real_data.insert(seg_key.clone());
                    }
                    let mapped_val = match (bo4e_val, enum_map) {
                        (Some(v), Some(map)) => map
                            .iter()
                            .find(|(_, bo4e_v)| *bo4e_v == &v)
                            .map(|(edifact_k, _)| edifact_k.clone())
                            .or(Some(v)),
                        (v, _) => v,
                    };
                    mapped_val.or_else(|| default.cloned())
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
                    let already_has = field_values
                        .iter()
                        .any(|(k, _, ei, ci, _)| *k == seg_key && *ei == 0 && *ci == 0);
                    if !already_has {
                        field_values.push((seg_key, seg_tag, 0, 0, q.to_string()));
                    }
                }
            }
        }

        // Per-segment phantom prevention for qualified segments: remove entries
        // for segments using tag[qualifier] syntax (e.g., FTX[ACB], DTM[Z07])
        // that have data fields but none resolved to actual BO4E values.  This
        // prevents phantom segments when a definition maps multiple segment types
        // and optional qualified segments are not in the original message.
        // Unqualified segments (plain tags like SEQ, IDE) are always kept — they
        // are typically entry/mandatory segments of their group.
        field_values.retain(|(seg_key, _, _, _, _)| {
            if !seg_key.contains('[') {
                return true; // unqualified segments always kept
            }
            !seg_has_data_field.contains(seg_key) || seg_has_real_data.contains(seg_key)
        });

        // If the definition has data fields but none resolved to actual BO4E values,
        // return an empty instance to prevent phantom segments for groups not
        // present in the original EDIFACT message.  Definitions with only
        // qualifier/default fields (has_data_fields=false) are always kept.
        if has_data_fields && !has_real_data {
            return AssembledGroupInstance {
                segments: vec![],
                child_groups: vec![],
            };
        }

        // Build segments with elements/components in correct positions.
        // Group by segment_key to create separate segments for "DTM[92]" vs "DTM[93]".
        let mut segments: Vec<AssembledSegment> = Vec::with_capacity(field_values.len());
        let mut seen_keys: HashMap<String, usize> = HashMap::new();

        for (seg_key, seg_tag, element_idx, component_idx, val) in &field_values {
            let seg = if let Some(&pos) = seen_keys.get(seg_key) {
                &mut segments[pos]
            } else {
                let pos = segments.len();
                seen_keys.insert(seg_key.clone(), pos);
                segments.push(AssembledSegment {
                    tag: seg_tag.clone(),
                    elements: vec![],
                });
                &mut segments[pos]
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

    /// Parse element and component indices from path parts after the segment tag.
    /// E.g., ["2"] -> (2, 0), ["0", "3"] -> (0, 3), ["1", "0"] -> (1, 0)
    fn parse_element_component(parts: &[&str]) -> (usize, usize) {
        if parts.is_empty() {
            return (0, 0);
        }
        let element_idx = parts[0].parse::<usize>().unwrap_or(0);
        let component_idx = if parts.len() > 1 {
            parts[1].parse::<usize>().unwrap_or(0)
        } else {
            0
        };
        (element_idx, component_idx)
    }

    /// Extract a value from a BO4E JSON object by target field name.
    /// Supports dotted paths like "nested.field_name".
    pub fn populate_field(
        &self,
        bo4e_value: &serde_json::Value,
        target_field: &str,
        _source_path: &str,
    ) -> Option<String> {
        let mut current = bo4e_value;
        for part in target_field.split('.') {
            current = current.get(part)?;
        }
        // Handle enriched code objects: {"code": "Z15", "meaning": "..."}
        if let Some(code) = current.get("code").and_then(|v| v.as_str()) {
            return Some(code.to_string());
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

    /// Resolve a discriminated instance using source_path for parent navigation.
    ///
    /// Like `resolve_repetition` + `resolve_group_instance`, but navigates to the
    /// parent group via source_path qualifier suffixes. Returns the matching instance
    /// directly (not just a rep index) to avoid re-navigation in `map_forward_inner`.
    ///
    /// For example, `source_path = "sg4.sg8_z98.sg10"` with `discriminator = "CCI.2.0=ZB3"`
    /// navigates to the SG8 instance with SEQ qualifier Z98, then finds the SG10 rep
    /// where CCI element 2 component 0 equals "ZB3".
    fn resolve_discriminated_by_source_path<'a>(
        tree: &'a AssembledTree,
        source_path: &str,
        discriminator: &str,
    ) -> Option<&'a AssembledGroupInstance> {
        let (spec, expected) = discriminator.split_once('=')?;
        let parts: Vec<&str> = spec.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let tag = parts[0];
        let element_idx: usize = parts[1].parse().ok()?;
        let component_idx: usize = parts[2].parse().ok()?;

        // Split source_path into parent + leaf
        let sp_parts: Vec<&str> = source_path.split('.').collect();
        if sp_parts.len() < 2 {
            return None;
        }
        let parent_path = sp_parts[..sp_parts.len() - 1].join(".");
        let (leaf_id, _) = parse_source_path_part(sp_parts.last()?);

        // Navigate to parent instance via source_path
        let parent = Self::resolve_by_source_path(tree, &parent_path)?;

        // Find leaf group in parent's children
        let leaf_group = parent
            .child_groups
            .iter()
            .find(|g| g.group_id.eq_ignore_ascii_case(leaf_id))?;

        // Scan reps for discriminator match
        leaf_group.repetitions.iter().find(|instance| {
            instance.segments.iter().any(|s| {
                s.tag.eq_ignore_ascii_case(tag)
                    && s.elements
                        .get(element_idx)
                        .and_then(|e| e.get(component_idx))
                        .map(|v| v == expected)
                        .unwrap_or(false)
            })
        })
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
        self.map_all_forward_inner(tree, true)
    }

    /// Inner implementation with enrichment control.
    fn map_all_forward_inner(&self, tree: &AssembledTree, enrich_codes: bool) -> serde_json::Value {
        let mut result = serde_json::Map::new();

        for def in &self.definitions {
            let entity = &def.meta.entity;

            let bo4e = if let Some(ref disc) = def.meta.discriminator {
                // Has discriminator — resolve to specific rep.
                // Use source_path navigation when qualifiers are present
                // (e.g., "sg4.sg8_z98.sg10" navigates to Z98's SG10 reps).
                let use_source_path = def.meta.source_path.as_ref().is_some_and(|sp| {
                    has_source_path_qualifiers(sp) && !def.meta.source_group.contains(':')
                });
                if use_source_path {
                    // Navigate to parent via source_path, find discriminated instance
                    Self::resolve_discriminated_by_source_path(
                        tree,
                        def.meta.source_path.as_deref().unwrap(),
                        disc,
                    )
                    .map(|instance| {
                        let mut r = serde_json::Map::new();
                        self.extract_fields_from_instance(instance, def, &mut r, enrich_codes);
                        self.extract_companion_fields(instance, def, &mut r, enrich_codes);
                        serde_json::Value::Object(r)
                    })
                } else {
                    Self::resolve_repetition(tree, &def.meta.source_group, disc)
                        .map(|rep| self.map_forward_inner(tree, def, rep, enrich_codes))
                }
            } else if def.meta.source_group.is_empty() {
                // Root-level mapping — always single object
                Some(self.map_forward_inner(tree, def, 0, enrich_codes))
            } else if def.meta.source_path.as_ref().is_some_and(|sp| {
                has_source_path_qualifiers(sp)
            }) {
                // Source path has qualifier suffixes (e.g., "sg4.sg8_zd7.sg10")
                // — navigate via entry-segment qualifiers instead of hardcoded :N
                // indices from source_group.  Returns None when the qualified parent
                // doesn't exist in the tree (e.g., no SEQ+ZD7 in this message).
                let sp = def.meta.source_path.as_deref().unwrap();
                Self::resolve_by_source_path(tree, sp).map(|instance| {
                    let mut r = serde_json::Map::new();
                    self.extract_fields_from_instance(instance, def, &mut r, enrich_codes);
                    self.extract_companion_fields(instance, def, &mut r, enrich_codes);
                    serde_json::Value::Object(r)
                })
            } else {
                let num_reps = Self::count_repetitions(tree, &def.meta.source_group);
                if num_reps <= 1 {
                    Some(self.map_forward_inner(tree, def, 0, enrich_codes))
                } else {
                    // Multiple reps, no discriminator — map all into array
                    let mut items = Vec::with_capacity(num_reps);
                    for rep in 0..num_reps {
                        items.push(self.map_forward_inner(tree, def, rep, enrich_codes));
                    }
                    Some(serde_json::Value::Array(items))
                }
            };

            if let Some(bo4e) = bo4e {
                let bo4e = inject_bo4e_metadata(bo4e, &def.meta.bo4e_type);
                let key = to_camel_case(entity);
                deep_merge_insert(&mut result, &key, bo4e);
            }
        }

        serde_json::Value::Object(result)
    }

    /// Reverse-map a BO4E entity map back to an AssembledTree.
    ///
    /// For each definition:
    /// 1. Look up entity in input by `meta.entity` name
    /// 2. If entity value is an array, map each element as a separate group repetition
    /// 3. Place results by `source_group`: `""` → root segments, `"SGn"` → groups
    ///
    /// This is the inverse of `map_all_forward()`.
    pub fn map_all_reverse(&self, entities: &serde_json::Value) -> AssembledTree {
        let mut root_segments: Vec<AssembledSegment> = Vec::new();
        let mut groups: Vec<AssembledGroup> = Vec::new();

        for def in &self.definitions {
            let entity_key = to_camel_case(&def.meta.entity);

            // Look up entity value
            let entity_value = entities.get(&entity_key);

            if entity_value.is_none() {
                continue;
            }
            let entity_value = entity_value.unwrap();

            // Determine target group from source_group (use leaf part after last dot)
            let leaf_group = def
                .meta
                .source_group
                .rsplit('.')
                .next()
                .unwrap_or(&def.meta.source_group);

            if def.meta.source_group.is_empty() {
                // Root-level: reverse into root segments
                let instance = self.map_reverse(entity_value, def);
                root_segments.extend(instance.segments);
            } else if entity_value.is_array() {
                // Array entity: each element becomes a group repetition
                let arr = entity_value.as_array().unwrap();
                let mut reps = Vec::new();
                for item in arr {
                    reps.push(self.map_reverse(item, def));
                }

                // Merge into existing group or create new one
                if let Some(existing) = groups.iter_mut().find(|g| g.group_id == leaf_group) {
                    existing.repetitions.extend(reps);
                } else {
                    groups.push(AssembledGroup {
                        group_id: leaf_group.to_string(),
                        repetitions: reps,
                    });
                }
            } else {
                // Single object: one repetition
                let instance = self.map_reverse(entity_value, def);

                if let Some(existing) = groups.iter_mut().find(|g| g.group_id == leaf_group) {
                    existing.repetitions.push(instance);
                } else {
                    groups.push(AssembledGroup {
                        group_id: leaf_group.to_string(),
                        repetitions: vec![instance],
                    });
                }
            }
        }

        AssembledTree {
            segments: root_segments,
            groups,
            post_group_start: 0,
        }
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
        enrich_codes: bool,
    ) -> crate::model::MappedMessage {
        // Map message-level entities
        let stammdaten = msg_engine.map_all_forward_inner(tree, enrich_codes);

        // Find the transaction group and map each repetition
        let transaktionen = tree
            .groups
            .iter()
            .find(|g| g.group_id == transaction_group)
            .map(|sg| {
                sg.repetitions
                    .iter()
                    .map(|instance| {
                        // Wrap the instance in its group so that definitions with
                        // source_group paths like "SG4.SG5" can resolve correctly.
                        let wrapped_tree = AssembledTree {
                            segments: vec![],
                            groups: vec![AssembledGroup {
                                group_id: transaction_group.to_string(),
                                repetitions: vec![instance.clone()],
                            }],
                            post_group_start: 0,
                        };
                        let tx_result =
                            tx_engine.map_all_forward_inner(&wrapped_tree, enrich_codes);

                        // Split: "Prozessdaten" entity goes into transaktionsdaten,
                        // everything else into stammdaten
                        let mut tx_stammdaten = serde_json::Map::new();
                        let mut transaktionsdaten = serde_json::Value::Null;

                        if let Some(obj) = tx_result.as_object() {
                            for (key, value) in obj {
                                if key == "prozessdaten" || key == "nachricht" {
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

    /// Reverse-map a `MappedMessage` back to an `AssembledTree`.
    ///
    /// Two-engine approach mirroring `map_interchange()`:
    /// - `msg_engine` handles message-level stammdaten → SG2/SG3 groups
    /// - `tx_engine` handles per-transaction stammdaten + transaktionsdaten → SG4 instances
    ///
    /// For each transaction, performs two passes:
    /// - **Pass 1**: Reverse-map transaktionsdaten (entities named "Prozessdaten" or "Nachricht")
    ///   → root segments (IDE, STS, DTM) + SG6 groups (RFF)
    /// - **Pass 2**: Reverse-map stammdaten (all other entities)
    ///   → SG5, SG8, SG10, SG12 groups
    ///
    /// Results are merged into one `AssembledGroupInstance` per transaction,
    /// collected into an SG4 `AssembledGroup`, then combined with message-level groups.
    pub fn map_interchange_reverse(
        msg_engine: &MappingEngine,
        tx_engine: &MappingEngine,
        mapped: &crate::model::MappedMessage,
        transaction_group: &str,
    ) -> AssembledTree {
        // Step 1: Reverse message-level stammdaten
        let msg_tree = msg_engine.map_all_reverse(&mapped.stammdaten);

        // Step 2: Build transaction instances from each Transaktion
        let mut sg4_reps: Vec<AssembledGroupInstance> = Vec::new();

        // Collect all definitions with their relative paths and sort by depth.
        // Shallower paths (SG8) must be processed before deeper ones (SG8:0.SG10)
        // so that parent group repetitions exist before children are added.
        struct DefWithMeta<'a> {
            def: &'a MappingDefinition,
            relative: String,
            depth: usize,
            is_transaktionsdaten: bool,
        }

        let mut sorted_defs: Vec<DefWithMeta> = tx_engine
            .definitions
            .iter()
            .map(|def| {
                let relative = strip_tx_group_prefix(&def.meta.source_group, transaction_group);
                let depth = if relative.is_empty() {
                    0
                } else {
                    relative.chars().filter(|c| *c == '.').count() + 1
                };
                let entity_key = to_camel_case(&def.meta.entity);
                let is_transaktionsdaten =
                    entity_key == "prozessdaten" || entity_key == "nachricht";
                DefWithMeta {
                    def,
                    relative,
                    depth,
                    is_transaktionsdaten,
                }
            })
            .collect();

        // Build parent source_path → rep_index map from deeper definitions.
        // SG10 defs like "SG4.SG8:0.SG10" with source_path "sg4.sg8_z79.sg10"
        // tell us that the SG8 def with source_path "sg4.sg8_z79" should be rep 0.
        let mut parent_rep_map: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for dm in &sorted_defs {
            if dm.depth >= 2 {
                let parts: Vec<&str> = dm.relative.split('.').collect();
                let (_, parent_rep) = parse_group_spec(parts[0]);
                if let Some(rep_idx) = parent_rep {
                    if let Some(sp) = &dm.def.meta.source_path {
                        if let Some((parent_path, _)) = sp.rsplit_once('.') {
                            parent_rep_map
                                .entry(parent_path.to_string())
                                .or_insert(rep_idx);
                        }
                    }
                }
            }
        }

        // Augment shallow definitions with explicit rep indices from the map.
        // E.g., SG8 def with source_path "sg4.sg8_z79" gets relative "SG8:0".
        for dm in &mut sorted_defs {
            if dm.depth == 1 && !dm.relative.contains(':') {
                if let Some(sp) = &dm.def.meta.source_path {
                    if let Some(rep_idx) = parent_rep_map.get(sp.as_str()) {
                        dm.relative = format!("{}:{}", dm.relative, rep_idx);
                    }
                }
            }
        }

        // Sort: shallower depth first, so SG8 defs create reps before SG8:N.SG10 defs.
        // Within same depth, sort by relative path for deterministic ordering.
        sorted_defs.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.relative.cmp(&b.relative)));

        for tx in &mapped.transaktionen {
            let mut root_segs: Vec<AssembledSegment> = Vec::new();
            let mut child_groups: Vec<AssembledGroup> = Vec::new();

            // Track source_path → repetition index for parent groups (top-down).
            // Built during depth-1 processing, used by depth-2+ defs without
            // explicit rep indices to find their correct parent via source_path.
            let mut source_path_to_rep: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for dm in &sorted_defs {
                // Determine the BO4E value to reverse-map from
                let bo4e_value = if dm.is_transaktionsdaten {
                    &tx.transaktionsdaten
                } else {
                    let entity_key = to_camel_case(&dm.def.meta.entity);
                    match tx.stammdaten.get(&entity_key) {
                        Some(v) => v,
                        None => continue,
                    }
                };

                // Handle array entities: each element becomes a separate group rep.
                // This supports the NAD/SG12 pattern where multiple NAD qualifiers
                // (Z63-Z70) map to a single "Geschaeftspartner" entity as an array.
                let items: Vec<&serde_json::Value> = if bo4e_value.is_array() {
                    bo4e_value.as_array().unwrap().iter().collect()
                } else {
                    vec![bo4e_value]
                };

                for item in &items {
                    let instance = tx_engine.map_reverse(item, dm.def);

                    // Skip empty instances (definition had no real BO4E data)
                    if instance.segments.is_empty() && instance.child_groups.is_empty() {
                        continue;
                    }

                    if dm.relative.is_empty() {
                        root_segs.extend(instance.segments);
                    } else {
                        // For depth-2+ defs without explicit rep index, resolve
                        // parent rep from source_path matching (qualifier-based).
                        let effective_relative = if dm.depth >= 2 {
                            resolve_child_relative(
                                &dm.relative,
                                dm.def.meta.source_path.as_deref(),
                                &source_path_to_rep,
                            )
                        } else {
                            dm.relative.clone()
                        };

                        let rep_used =
                            place_in_groups(&mut child_groups, &effective_relative, instance);

                        // Track source_path → rep_index for depth-1 (parent) defs
                        if dm.depth == 1 {
                            if let Some(sp) = &dm.def.meta.source_path {
                                source_path_to_rep.insert(sp.clone(), rep_used);
                            }
                        }
                    }
                }
            }

            sg4_reps.push(AssembledGroupInstance {
                segments: root_segs,
                child_groups,
            });
        }

        // Step 3: Combine message tree with transaction group
        let pre_group_count = msg_tree.segments.len();
        let mut all_groups = msg_tree.groups;
        if !sg4_reps.is_empty() {
            all_groups.push(AssembledGroup {
                group_id: transaction_group.to_string(),
                repetitions: sg4_reps,
            });
        }

        AssembledTree {
            segments: msg_tree.segments,
            groups: all_groups,
            post_group_start: pre_group_count,
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
/// Parse a source_path part into (group_id, optional_qualifier).
///
/// `"sg8_z98"` → `("sg8", Some("z98"))`
/// `"sg4"` → `("sg4", None)`
/// `"sg10"` → `("sg10", None)`
fn parse_source_path_part(part: &str) -> (&str, Option<&str>) {
    // Find the first underscore that separates group from qualifier.
    // Source path parts look like "sg8_z98", "sg4", "sg10", "sg12_z04".
    // The group ID is always "sgN", so the underscore after the digits is the separator.
    if let Some(pos) = part.find('_') {
        let group = &part[..pos];
        let qualifier = &part[pos + 1..];
        if !qualifier.is_empty() {
            return (group, Some(qualifier));
        }
    }
    (part, None)
}

/// Find a group repetition whose entry segment has a matching qualifier.
///
/// The entry segment is the first segment in the instance (e.g., SEQ for SG8).
/// The qualifier is matched against `elements[0][0]` (case-insensitive).
fn find_rep_by_entry_qualifier<'a>(
    reps: &'a [AssembledGroupInstance],
    qualifier: &str,
) -> Option<&'a AssembledGroupInstance> {
    reps.iter().find(|inst| {
        inst.segments.first().is_some_and(|seg| {
            seg.elements
                .first()
                .and_then(|e| e.first())
                .is_some_and(|v| v.eq_ignore_ascii_case(qualifier))
        })
    })
}

/// Check if a source_path contains qualifier suffixes (e.g., "sg8_z98").
fn has_source_path_qualifiers(source_path: &str) -> bool {
    source_path.split('.').any(|part| {
        if let Some(pos) = part.find('_') {
            pos < part.len() - 1
        } else {
            false
        }
    })
}

fn parse_group_spec(part: &str) -> (&str, Option<usize>) {
    if let Some(colon_pos) = part.find(':') {
        let id = &part[..colon_pos];
        let rep = part[colon_pos + 1..].parse::<usize>().ok();
        (id, rep)
    } else {
        (part, None)
    }
}

/// Strip the transaction group prefix from a source_group path.
///
/// Given `source_group = "SG4.SG8:0.SG10"` and `tx_group = "SG4"`,
/// returns `"SG8:0.SG10"`.
/// Given `source_group = "SG4"` and `tx_group = "SG4"`, returns `""`.
fn strip_tx_group_prefix(source_group: &str, tx_group: &str) -> String {
    if source_group == tx_group || source_group.is_empty() {
        String::new()
    } else if let Some(rest) = source_group.strip_prefix(tx_group) {
        rest.strip_prefix('.').unwrap_or(rest).to_string()
    } else {
        source_group.to_string()
    }
}

/// Place a reverse-mapped group instance into the correct nesting position.
///
/// `relative_path` is the group path relative to the transaction group:
/// - `"SG5"` → top-level child group
/// - `"SG8:0.SG10"` → SG10 inside SG8 repetition 0
///
/// Returns the repetition index used at the first nesting level.
fn place_in_groups(
    groups: &mut Vec<AssembledGroup>,
    relative_path: &str,
    instance: AssembledGroupInstance,
) -> usize {
    let parts: Vec<&str> = relative_path.split('.').collect();

    if parts.len() == 1 {
        // Leaf group: "SG5", "SG8", "SG12", or with explicit index "SG8:0"
        let (id, rep) = parse_group_spec(parts[0]);

        // Find or create the group
        let group = if let Some(g) = groups.iter_mut().find(|g| g.group_id == id) {
            g
        } else {
            groups.push(AssembledGroup {
                group_id: id.to_string(),
                repetitions: vec![],
            });
            groups.last_mut().unwrap()
        };

        if let Some(rep_idx) = rep {
            // Explicit index: place at specific position, merging into existing
            while group.repetitions.len() <= rep_idx {
                group.repetitions.push(AssembledGroupInstance {
                    segments: vec![],
                    child_groups: vec![],
                });
            }
            group.repetitions[rep_idx]
                .segments
                .extend(instance.segments);
            group.repetitions[rep_idx]
                .child_groups
                .extend(instance.child_groups);
            rep_idx
        } else {
            // No index: append new repetition
            let pos = group.repetitions.len();
            group.repetitions.push(instance);
            pos
        }
    } else {
        // Nested path: e.g., "SG8:0.SG10" → place SG10 inside SG8 rep 0
        let (parent_id, parent_rep) = parse_group_spec(parts[0]);
        let rep_idx = parent_rep.unwrap_or(0);

        // Find or create the parent group
        let parent_group = if let Some(g) = groups.iter_mut().find(|g| g.group_id == parent_id) {
            g
        } else {
            groups.push(AssembledGroup {
                group_id: parent_id.to_string(),
                repetitions: vec![],
            });
            groups.last_mut().unwrap()
        };

        // Ensure the target repetition exists (extend with empty instances if needed)
        while parent_group.repetitions.len() <= rep_idx {
            parent_group.repetitions.push(AssembledGroupInstance {
                segments: vec![],
                child_groups: vec![],
            });
        }

        let remaining = parts[1..].join(".");
        place_in_groups(
            &mut parent_group.repetitions[rep_idx].child_groups,
            &remaining,
            instance,
        );
        rep_idx
    }
}

/// Resolve the effective relative path for a child definition (depth >= 2).
///
/// If the child's relative already has an explicit parent rep index (e.g., "SG8:5.SG10"),
/// use it as-is. Otherwise, use the `source_path` to look up the parent's actual
/// repetition index from `source_path_to_rep`.
///
/// Example: relative = "SG8.SG10", source_path = "sg4.sg8_ze1.sg10"
/// → looks up "sg4.sg8_ze1" in map → finds rep 6 → returns "SG8:6.SG10"
fn resolve_child_relative(
    relative: &str,
    source_path: Option<&str>,
    source_path_to_rep: &std::collections::HashMap<String, usize>,
) -> String {
    let parts: Vec<&str> = relative.split('.').collect();
    if parts.is_empty() {
        return relative.to_string();
    }

    // If first part already has explicit index, keep as-is
    let (parent_id, parent_rep) = parse_group_spec(parts[0]);
    if parent_rep.is_some() {
        return relative.to_string();
    }

    // Try to resolve from source_path: extract parent path and look up its rep
    if let Some(sp) = source_path {
        if let Some((parent_path, _child)) = sp.rsplit_once('.') {
            if let Some(rep_idx) = source_path_to_rep.get(parent_path) {
                let rest = parts[1..].join(".");
                return format!("{}:{}.{}", parent_id, rep_idx, rest);
            }
        }
    }

    // No resolution possible, keep original
    relative.to_string()
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

/// Convert a PascalCase name to camelCase by lowering the first character.
///
/// E.g., `"Ansprechpartner"` → `"ansprechpartner"`,
/// `"AnsprechpartnerEdifact"` → `"ansprechpartnerEdifact"`,
/// `"ProduktpaketPriorisierung"` → `"produktpaketPriorisierung"`.
fn to_camel_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Set a value in a nested JSON map using a dotted path.
/// E.g., "address.city" sets `{"address": {"city": "value"}}`.
fn set_nested_value(map: &mut serde_json::Map<String, serde_json::Value>, path: &str, val: String) {
    set_nested_value_json(map, path, serde_json::Value::String(val));
}

/// Like `set_nested_value` but accepts a `serde_json::Value` instead of a `String`.
fn set_nested_value_json(
    map: &mut serde_json::Map<String, serde_json::Value>,
    path: &str,
    val: serde_json::Value,
) {
    if let Some((prefix, leaf)) = path.rsplit_once('.') {
        let mut current = map;
        for part in prefix.split('.') {
            let entry = current
                .entry(part.to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            current = entry.as_object_mut().expect("expected object in path");
        }
        current.insert(leaf.to_string(), val);
    } else {
        map.insert(path.to_string(), val);
    }
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
                    elements: vec![vec!["E01".to_string()], vec!["DOC001".to_string()]],
                },
            ],
            groups: vec![
                AssembledGroup {
                    group_id: "SG2".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "NAD".to_string(),
                            elements: vec![vec!["MS".to_string()], vec!["9900123".to_string()]],
                        }],
                        child_groups: vec![],
                    }],
                },
                AssembledGroup {
                    group_id: "SG4".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![vec!["24".to_string()], vec!["TX001".to_string()]],
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
                    source_group: "SG4".to_string(),
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
                    source_group: "SG4.SG5".to_string(),
                    source_path: None,
                    discriminator: None,
                },
                fields: malo_fields,
                companion_fields: None,
                complex_handlers: None,
            },
        ]);

        let result = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", true);

        assert_eq!(result.transaktionen.len(), 1);
        assert_eq!(
            result.transaktionen[0].transaktionsdaten["vorgangId"]
                .as_str()
                .unwrap(),
            "TX001"
        );
        assert_eq!(
            result.transaktionen[0].stammdaten["marktlokation"]["marktlokationsId"]
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
                            elements: vec![vec!["MS".to_string()], vec!["9900123".to_string()]],
                        }],
                        child_groups: vec![],
                    }],
                },
                AssembledGroup {
                    group_id: "SG4".to_string(),
                    repetitions: vec![AssembledGroupInstance {
                        segments: vec![AssembledSegment {
                            tag: "IDE".to_string(),
                            elements: vec![vec!["24".to_string()], vec!["TX001".to_string()]],
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
        assert!(result.get("marktteilnehmer").is_some());
        let mt = &result["marktteilnehmer"];
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
                        elements: vec![vec!["24".to_string()], vec!["TX001".to_string()]],
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
            result["prozessdaten"]["vorgangId"].as_str().unwrap(),
            "TX001"
        );

        // Should contain Marktlokation from SG5 within SG4
        assert_eq!(
            result["marktlokation"]["marktlokationsId"]
                .as_str()
                .unwrap(),
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
                            elements: vec![vec!["MS".to_string()], vec!["9900123".to_string()]],
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
                                elements: vec![vec!["24".to_string()], vec!["TX001".to_string()]],
                            }],
                            child_groups: vec![],
                        },
                        AssembledGroupInstance {
                            segments: vec![AssembledSegment {
                                tag: "IDE".to_string(),
                                elements: vec![vec!["24".to_string()], vec!["TX002".to_string()]],
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

        // Transaction-level definitions (source_group includes SG4 prefix)
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
                source_group: "SG4".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: tx_fields,
            companion_fields: None,
            complex_handlers: None,
        }];

        let msg_engine = MappingEngine::from_definitions(msg_defs);
        let tx_engine = MappingEngine::from_definitions(tx_defs);

        let result = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", true);

        // Message-level stammdaten
        assert!(result.stammdaten["marktteilnehmer"].is_object());
        assert_eq!(
            result.stammdaten["marktteilnehmer"]["marktrolle"]
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

    #[test]
    fn test_extract_companion_fields_with_code_enrichment() {
        use crate::code_lookup::CodeLookup;
        use mig_assembly::assembler::*;

        let schema = serde_json::json!({
            "fields": {
                "sg4": {
                    "children": {
                        "sg8_z01": {
                            "children": {
                                "sg10": {
                                    "segments": [{
                                        "id": "CCI",
                                        "elements": [{
                                            "index": 2,
                                            "components": [{
                                                "sub_index": 0,
                                                "type": "code",
                                                "codes": [
                                                    {"value": "Z15", "name": "Haushaltskunde"},
                                                    {"value": "Z18", "name": "Kein Haushaltskunde"}
                                                ]
                                            }]
                                        }]
                                    }],
                                    "source_group": "SG10"
                                }
                            },
                            "segments": [],
                            "source_group": "SG8"
                        }
                    },
                    "segments": [],
                    "source_group": "SG4"
                }
            }
        });

        let code_lookup = CodeLookup::from_schema_value(&schema);

        let tree = AssembledTree {
            segments: vec![],
            groups: vec![AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![],
                    child_groups: vec![AssembledGroup {
                        group_id: "SG8".to_string(),
                        repetitions: vec![AssembledGroupInstance {
                            segments: vec![],
                            child_groups: vec![AssembledGroup {
                                group_id: "SG10".to_string(),
                                repetitions: vec![AssembledGroupInstance {
                                    segments: vec![AssembledSegment {
                                        tag: "CCI".to_string(),
                                        elements: vec![vec![], vec![], vec!["Z15".to_string()]],
                                    }],
                                    child_groups: vec![],
                                }],
                            }],
                        }],
                    }],
                }],
            }],
            post_group_start: 0,
        };

        let mut companion_fields = BTreeMap::new();
        companion_fields.insert(
            "cci.2".to_string(),
            FieldMapping::Simple("haushaltskunde".to_string()),
        );

        let def = MappingDefinition {
            meta: MappingMeta {
                entity: "Marktlokation".to_string(),
                bo4e_type: "Marktlokation".to_string(),
                companion_type: Some("MarktlokationEdifact".to_string()),
                source_group: "SG4.SG8.SG10".to_string(),
                source_path: Some("sg4.sg8_z01.sg10".to_string()),
                discriminator: None,
            },
            fields: BTreeMap::new(),
            companion_fields: Some(companion_fields),
            complex_handlers: None,
        };

        // Without code lookup — plain string
        let engine_plain = MappingEngine::from_definitions(vec![]);
        let bo4e_plain = engine_plain.map_forward(&tree, &def, 0);
        assert_eq!(
            bo4e_plain["marktlokationEdifact"]["haushaltskunde"].as_str(),
            Some("Z15"),
            "Without code lookup, should be plain string"
        );

        // With code lookup — enriched object
        let engine_enriched = MappingEngine::from_definitions(vec![]).with_code_lookup(code_lookup);
        let bo4e_enriched = engine_enriched.map_forward(&tree, &def, 0);
        let hk = &bo4e_enriched["marktlokationEdifact"]["haushaltskunde"];
        assert_eq!(hk["code"].as_str(), Some("Z15"));
        assert_eq!(hk["meaning"].as_str(), Some("Haushaltskunde"));
    }

    #[test]
    fn test_reverse_mapping_accepts_enriched_companion() {
        // Reverse mapping should accept both plain string and enriched object format
        let mut companion_fields = BTreeMap::new();
        companion_fields.insert(
            "cci.2".to_string(),
            FieldMapping::Simple("haushaltskunde".to_string()),
        );

        let def = MappingDefinition {
            meta: MappingMeta {
                entity: "Test".to_string(),
                bo4e_type: "Test".to_string(),
                companion_type: Some("TestEdifact".to_string()),
                source_group: "SG4".to_string(),
                source_path: None,
                discriminator: None,
            },
            fields: BTreeMap::new(),
            companion_fields: Some(companion_fields),
            complex_handlers: None,
        };

        let engine = MappingEngine::from_definitions(vec![]);

        // Test 1: Plain string format (backward compat)
        let bo4e_plain = serde_json::json!({
            "testEdifact": {
                "haushaltskunde": "Z15"
            }
        });
        let instance_plain = engine.map_reverse(&bo4e_plain, &def);
        assert_eq!(instance_plain.segments[0].elements[2], vec!["Z15"]);

        // Test 2: Enriched object format
        let bo4e_enriched = serde_json::json!({
            "testEdifact": {
                "haushaltskunde": {
                    "code": "Z15",
                    "meaning": "Haushaltskunde gem. EnWG"
                }
            }
        });
        let instance_enriched = engine.map_reverse(&bo4e_enriched, &def);
        assert_eq!(instance_enriched.segments[0].elements[2], vec!["Z15"]);
    }

    #[test]
    fn test_resolve_child_relative_with_source_path() {
        let mut map = std::collections::HashMap::new();
        map.insert("sg4.sg8_ze1".to_string(), 6usize);
        map.insert("sg4.sg8_z98".to_string(), 0usize);

        // Child without explicit index → resolved from source_path
        assert_eq!(
            resolve_child_relative("SG8.SG10", Some("sg4.sg8_ze1.sg10"), &map),
            "SG8:6.SG10"
        );

        // Child with explicit index → kept as-is
        assert_eq!(
            resolve_child_relative("SG8:3.SG10", Some("sg4.sg8_ze1.sg10"), &map),
            "SG8:3.SG10"
        );

        // Source path not in map → kept as-is
        assert_eq!(
            resolve_child_relative("SG8.SG10", Some("sg4.sg8_unknown.sg10"), &map),
            "SG8.SG10"
        );

        // No source_path → kept as-is
        assert_eq!(resolve_child_relative("SG8.SG10", None, &map), "SG8.SG10");

        // SG9 also works
        assert_eq!(
            resolve_child_relative("SG8.SG9", Some("sg4.sg8_z98.sg9"), &map),
            "SG8:0.SG9"
        );
    }

    #[test]
    fn test_place_in_groups_returns_rep_index() {
        let mut groups: Vec<AssembledGroup> = Vec::new();

        // Append (no index) → returns position 0
        let instance = AssembledGroupInstance {
            segments: vec![],
            child_groups: vec![],
        };
        assert_eq!(place_in_groups(&mut groups, "SG8", instance), 0);

        // Append again → returns position 1
        let instance = AssembledGroupInstance {
            segments: vec![],
            child_groups: vec![],
        };
        assert_eq!(place_in_groups(&mut groups, "SG8", instance), 1);

        // Explicit index → returns that index
        let instance = AssembledGroupInstance {
            segments: vec![],
            child_groups: vec![],
        };
        assert_eq!(place_in_groups(&mut groups, "SG8:5", instance), 5);
    }

    #[test]
    fn test_resolve_by_source_path() {
        use mig_assembly::assembler::*;

        // Build a tree: SG4[0] → SG8 with two reps (Z98 and ZD7) → each has SG10
        let tree = AssembledTree {
            segments: vec![],
            groups: vec![AssembledGroup {
                group_id: "SG4".to_string(),
                repetitions: vec![AssembledGroupInstance {
                    segments: vec![],
                    child_groups: vec![AssembledGroup {
                        group_id: "SG8".to_string(),
                        repetitions: vec![
                            AssembledGroupInstance {
                                segments: vec![AssembledSegment {
                                    tag: "SEQ".to_string(),
                                    elements: vec![vec!["Z98".to_string()]],
                                }],
                                child_groups: vec![AssembledGroup {
                                    group_id: "SG10".to_string(),
                                    repetitions: vec![AssembledGroupInstance {
                                        segments: vec![AssembledSegment {
                                            tag: "CCI".to_string(),
                                            elements: vec![vec![], vec![], vec!["ZB3".to_string()]],
                                        }],
                                        child_groups: vec![],
                                    }],
                                }],
                            },
                            AssembledGroupInstance {
                                segments: vec![AssembledSegment {
                                    tag: "SEQ".to_string(),
                                    elements: vec![vec!["ZD7".to_string()]],
                                }],
                                child_groups: vec![AssembledGroup {
                                    group_id: "SG10".to_string(),
                                    repetitions: vec![AssembledGroupInstance {
                                        segments: vec![AssembledSegment {
                                            tag: "CCI".to_string(),
                                            elements: vec![vec![], vec![], vec!["ZE6".to_string()]],
                                        }],
                                        child_groups: vec![],
                                    }],
                                }],
                            },
                        ],
                    }],
                }],
            }],
            post_group_start: 0,
        };

        // Resolve SG10 under Z98
        let inst = MappingEngine::resolve_by_source_path(&tree, "sg4.sg8_z98.sg10");
        assert!(inst.is_some());
        assert_eq!(inst.unwrap().segments[0].elements[2][0], "ZB3");

        // Resolve SG10 under ZD7
        let inst = MappingEngine::resolve_by_source_path(&tree, "sg4.sg8_zd7.sg10");
        assert!(inst.is_some());
        assert_eq!(inst.unwrap().segments[0].elements[2][0], "ZE6");

        // Unknown qualifier → None
        let inst = MappingEngine::resolve_by_source_path(&tree, "sg4.sg8_zzz.sg10");
        assert!(inst.is_none());

        // Without qualifier → first rep (Z98)
        let inst = MappingEngine::resolve_by_source_path(&tree, "sg4.sg8.sg10");
        assert!(inst.is_some());
        assert_eq!(inst.unwrap().segments[0].elements[2][0], "ZB3");
    }

    #[test]
    fn test_parse_source_path_part() {
        assert_eq!(parse_source_path_part("sg4"), ("sg4", None));
        assert_eq!(parse_source_path_part("sg8_z98"), ("sg8", Some("z98")));
        assert_eq!(parse_source_path_part("sg10"), ("sg10", None));
        assert_eq!(parse_source_path_part("sg12_z04"), ("sg12", Some("z04")));
    }

    #[test]
    fn test_has_source_path_qualifiers() {
        assert!(has_source_path_qualifiers("sg4.sg8_z98.sg10"));
        assert!(has_source_path_qualifiers("sg4.sg8_ze1.sg9"));
        assert!(!has_source_path_qualifiers("sg4.sg6"));
        assert!(!has_source_path_qualifiers("sg4.sg8.sg10"));
    }
}
