//! Mapping engine — loads TOML definitions and provides bidirectional conversion.

use std::path::Path;

use mig_assembly::assembler::{AssembledGroup, AssembledGroupInstance, AssembledSegment, AssembledTree};

use crate::definition::MappingDefinition;
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
                let def: MappingDefinition = toml::from_str(&content).map_err(|e| {
                    MappingError::TomlParse {
                        file: path.display().to_string(),
                        message: e.to_string(),
                    }
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

    /// Extract a field value from an assembled tree using a mapping path.
    ///
    /// Path format: "segment.composite.data_element" e.g., "loc.c517.d3225"
    /// The segment is found within the specified group at the given repetition index.
    pub fn extract_field(
        &self,
        tree: &AssembledTree,
        group_id: &str,
        path: &str,
        repetition: usize,
    ) -> Option<String> {
        let group = tree.groups.iter().find(|g| g.group_id == group_id)?;
        let instance = group.repetitions.get(repetition)?;
        Self::extract_from_instance(instance, path)
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

    /// Extract a value from a BO4E JSON object by target field name,
    /// for populating back into a tree at the given path.
    pub fn populate_field(
        &self,
        bo4e_value: &serde_json::Value,
        target_field: &str,
        _source_path: &str,
    ) -> Option<String> {
        // Navigate the BO4E JSON to find the target field.
        // Supports dotted paths like "nested.field_name".
        let parts: Vec<&str> = target_field.split('.').collect();
        let mut current = bo4e_value;
        for part in &parts {
            current = current.get(part)?;
        }
        current.as_str().map(|s| s.to_string())
    }

    /// Build a segment from BO4E values using the reverse mapping.
    ///
    /// Given a mapping definition and a BO4E JSON value, creates an
    /// `AssembledSegment` with elements populated from the BO4E fields.
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
        let mut segments = Vec::new();

        for (path, field_mapping) in &def.fields {
            let target = match field_mapping {
                crate::definition::FieldMapping::Simple(t) => t.clone(),
                crate::definition::FieldMapping::Structured(s) => s.target.clone(),
                crate::definition::FieldMapping::Nested(_) => continue,
            };

            let parts: Vec<&str> = path.split('.').collect();
            if parts.is_empty() {
                continue;
            }
            let seg_tag = parts[0].to_uppercase();

            if let Some(val) = self.populate_field(bo4e_value, &target, path) {
                // Find existing segment or create new
                let seg = segments
                    .iter_mut()
                    .find(|s: &&mut AssembledSegment| s.tag == seg_tag);
                match seg {
                    Some(existing) => {
                        existing.elements.push(vec![val]);
                    }
                    None => {
                        segments.push(AssembledSegment {
                            tag: seg_tag,
                            elements: vec![vec![val]],
                        });
                    }
                }
            }
        }

        AssembledGroup {
            group_id: def.meta.source_group.clone(),
            repetitions: vec![AssembledGroupInstance {
                segments,
                child_groups: vec![],
            }],
        }
    }
}
