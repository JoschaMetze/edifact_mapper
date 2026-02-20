//! TOML mapping definition types.
//!
//! These types are deserialized from TOML mapping files
//! in the `mappings/{format_version}/{message_type}_{variant}/` directory.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Root mapping definition — one per TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDefinition {
    pub meta: MappingMeta,
    pub fields: BTreeMap<String, FieldMapping>,
    pub companion_fields: Option<BTreeMap<String, FieldMapping>>,
    pub complex_handlers: Option<Vec<ComplexHandlerRef>>,
}

/// Metadata about the entity being mapped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMeta {
    pub entity: String,
    pub bo4e_type: String,
    pub companion_type: Option<String>,
    pub source_group: String,
    pub discriminator: Option<String>,
}

/// A field mapping — either a simple string target or a structured mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldMapping {
    /// Simple: "source_path" = "target_field"
    Simple(String),
    /// Structured: with optional transform, condition, etc.
    Structured(StructuredFieldMapping),
    /// Nested group mappings
    Nested(BTreeMap<String, FieldMapping>),
}

/// A structured field mapping with optional transform and condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFieldMapping {
    pub target: String,
    pub transform: Option<String>,
    pub when: Option<String>,
    pub default: Option<String>,
    /// Bidirectional enum translation map (EDIFACT value → BO4E value).
    /// Forward: looks up extracted EDIFACT value to produce BO4E value.
    /// Reverse: reverse-looks up BO4E value to produce EDIFACT value.
    /// Uses BTreeMap for deterministic reverse lookup (first key alphabetically wins).
    pub enum_map: Option<BTreeMap<String, String>>,
}

/// Reference to a complex handler function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexHandlerRef {
    pub name: String,
    pub description: Option<String>,
}
