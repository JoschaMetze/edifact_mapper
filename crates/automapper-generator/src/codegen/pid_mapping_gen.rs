//! Generate filled TOML mappings for a PID using reference mappings from existing PIDs.
//!
//! Scans all `mappings/{fv}/{variant}/pid_*/` directories to build a reference index
//! keyed by `(leaf_group_id, qualifier_value)`. For a new PID, walks its schema JSON
//! and copies matching reference mappings (updating `source_path` and `source_group`
//! as needed). Unmatched groups fall back to scaffold generation.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::GeneratorError;

// ---------------------------------------------------------------------------
// Local TOML mapping types (mirrors mig_bo4e::definition, avoids cyclic dep)
// ---------------------------------------------------------------------------

/// Root mapping definition — one per TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDefinition {
    pub meta: MappingMeta,
    pub fields: BTreeMap<String, FieldMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_fields: Option<BTreeMap<String, FieldMapping>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complex_handlers: Option<Vec<ComplexHandlerRef>>,
}

/// Metadata about the entity being mapped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMeta {
    pub entity: String,
    pub bo4e_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_type: Option<String>,
    pub source_group: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
}

/// A field mapping — either a simple string target or a structured mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldMapping {
    Simple(String),
    Structured(StructuredFieldMapping),
    Nested(BTreeMap<String, FieldMapping>),
}

/// A structured field mapping with optional transform and condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFieldMapping {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_map: Option<BTreeMap<String, String>>,
}

/// Reference to a complex handler function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexHandlerRef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Reference index
// ---------------------------------------------------------------------------

/// A reference mapping from an existing PID, keyed for reuse.
#[derive(Debug, Clone)]
struct ReferenceMapping {
    definition: MappingDefinition,
    source_pid: String,
}

/// Index of existing TOML mappings keyed by `(leaf_group_id, qualifier)`.
///
/// Examples:
/// - `("SG8", "Z79")` — SG8 with SEQ+Z79 discriminator
/// - `("SG12", "Z04")` — SG12 with NAD+Z04 discriminator
/// - `("SG4", "")` — SG4 without qualifier
/// - `("", "root")` — root-level mapping (source_group = "")
/// - `("SG10", "parent:Z79")` — SG10 whose parent SG8 has qualifier Z79
#[derive(Debug)]
pub struct ReferenceIndex {
    entries: HashMap<(String, String), ReferenceMapping>,
}

impl ReferenceIndex {
    /// Load all existing TOML mappings from `mappings_base/{fv}/{variant}/pid_*/`.
    pub fn load(mappings_base: &Path, fv: &str, variant: &str) -> Result<Self, GeneratorError> {
        let mut entries = HashMap::new();
        let variant_dir = mappings_base.join(fv).join(variant);
        if !variant_dir.exists() {
            return Ok(Self { entries });
        }

        // Find all pid_* directories
        let pattern = variant_dir.join("pid_*");
        let pattern_str = pattern.to_string_lossy().to_string();
        let pid_dirs: Vec<PathBuf> = glob::glob(&pattern_str)
            .map_err(|e| GeneratorError::Validation {
                message: format!("glob error: {e}"),
            })?
            .filter_map(|r| r.ok())
            .filter(|p| p.is_dir())
            .collect();

        for pid_dir in &pid_dirs {
            let pid_id = pid_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .strip_prefix("pid_")
                .unwrap_or("")
                .to_string();

            let toml_pattern = pid_dir.join("*.toml");
            let toml_str = toml_pattern.to_string_lossy().to_string();
            let toml_files: Vec<PathBuf> = glob::glob(&toml_str)
                .map_err(|e| GeneratorError::Validation {
                    message: format!("glob error: {e}"),
                })?
                .filter_map(|r| r.ok())
                .collect();

            for toml_path in &toml_files {
                let content = std::fs::read_to_string(toml_path)?;
                let def: MappingDefinition = match toml::from_str(&content) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("WARN: skipping {}: {}", toml_path.display(), e);
                        continue;
                    }
                };

                let key = derive_index_key(&def, toml_path);
                let rm = ReferenceMapping {
                    definition: def,
                    source_pid: pid_id.clone(),
                };

                // Also register a no-qualifier fallback for the leaf group.
                // This handles intra-group discriminators (e.g., multiple RFFs
                // within SG6) where the schema has no qualifier values.
                let leaf = key.0.clone();
                if !key.1.is_empty() && !leaf.is_empty() {
                    let fallback_key = (leaf, String::new());
                    entries.entry(fallback_key).or_insert(rm.clone());
                }

                // First mapping found wins
                entries.entry(key).or_insert(rm);
            }
        }

        Ok(Self { entries })
    }

    /// Look up a reference mapping by key.
    fn get(&self, key: &(String, String)) -> Option<&ReferenceMapping> {
        self.entries.get(key)
    }

    /// All keys in the index (for testing).
    pub fn keys(&self) -> impl Iterator<Item = &(String, String)> {
        self.entries.keys()
    }
}

/// Derive the index key from a parsed mapping definition.
///
/// Rules:
/// - `source_group = ""` → `("", "root")`
/// - Has discriminator `"SEQ.0.0=Z79"` → extract leaf group + value after `=`
/// - SG10 with `source_path` containing parent qualifier → `("SG10", "parent:Z79")`
/// - Otherwise → `(leaf_group, "")`
fn derive_index_key(def: &MappingDefinition, path: &Path) -> (String, String) {
    let sg = &def.meta.source_group;

    // Root mapping
    if sg.is_empty() {
        return ("".to_string(), "root".to_string());
    }

    // Extract leaf group: last dot-separated component, stripping any `:N` rep index
    let leaf = sg
        .split('.')
        .next_back()
        .unwrap_or(sg)
        .split(':')
        .next()
        .unwrap_or(sg)
        .to_string();

    // Check for explicit discriminator (e.g., "SEQ.0.0=Z79" or "NAD.0.0=Z04")
    if let Some(disc) = &def.meta.discriminator {
        if let Some(eq_pos) = disc.find('=') {
            let value = disc[eq_pos + 1..].to_string();
            return (leaf, value);
        }
    }

    // SG10 detection: if leaf is SG10, look for parent qualifier in source_path
    if leaf == "SG10" {
        if let Some(sp) = &def.meta.source_path {
            // source_path like "sg4.sg8_z79.sg10" — extract qualifier from parent
            let parts: Vec<&str> = sp.split('.').collect();
            if parts.len() >= 2 {
                let parent = parts[parts.len() - 2]; // "sg8_z79"
                if let Some(underscore_pos) = parent.find('_') {
                    let qualifier = parent[underscore_pos + 1..].to_uppercase();
                    return ("SG10".to_string(), format!("parent:{qualifier}"));
                }
            }
        }
        let _ = path;
    }

    // No explicit discriminator — try to derive qualifier from source_path
    // e.g., source_path "sg4.sg5_z16" → leaf part "sg5_z16" → qualifier "Z16"
    if let Some(sp) = &def.meta.source_path {
        let sp_leaf = sp.split('.').next_back().unwrap_or(sp);
        if let Some(underscore_pos) = sp_leaf.find('_') {
            let qualifier = sp_leaf[underscore_pos + 1..].to_uppercase();
            return (leaf, qualifier);
        }
    }

    (leaf, String::new())
}

// ---------------------------------------------------------------------------
// Schema loader (parses pre-generated pid_*_schema.json)
// ---------------------------------------------------------------------------

/// A group in the PID schema JSON.
#[derive(Debug, Clone)]
pub struct SchemaGroup {
    pub field_name: String,
    pub source_group: String,
    pub qualifier: Option<String>,
    pub disc_segment: Option<String>,
    pub segments: Vec<SchemaSegmentInfo>,
    pub children: Vec<SchemaGroup>,
}

/// Rich segment info from the PID schema JSON.
#[derive(Debug, Clone)]
pub struct SchemaSegmentInfo {
    pub id: String,
    pub name: Option<String>,
    pub elements: Vec<SchemaElementInfo>,
}

/// Element info within a segment.
#[derive(Debug, Clone)]
pub struct SchemaElementInfo {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub element_type: String,
    pub composite_id: Option<String>,
    pub codes: Vec<SchemaCodeInfo>,
    pub components: Vec<SchemaComponentInfo>,
}

/// Code value info.
#[derive(Debug, Clone)]
pub struct SchemaCodeInfo {
    pub value: String,
    pub name: String,
}

/// Component within a composite element.
#[derive(Debug, Clone)]
pub struct SchemaComponentInfo {
    pub sub_index: usize,
    pub id: String,
    pub name: String,
    pub element_type: String,
    pub codes: Vec<SchemaCodeInfo>,
}

impl SchemaSegmentInfo {
    /// Get just the segment ID string.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Intermediate JSON structures matching the schema format.
#[derive(Debug, Deserialize)]
struct SchemaJson {
    #[allow(dead_code)]
    pid: String,
    fields: HashMap<String, SchemaFieldJson>,
}

#[derive(Debug, Deserialize)]
struct SchemaFieldJson {
    source_group: String,
    #[serde(default)]
    segments: Vec<SchemaSegmentJson>,
    #[serde(default)]
    discriminator: Option<SchemaDiscriminatorJson>,
    #[serde(default)]
    children: Option<HashMap<String, SchemaFieldJson>>,
}

/// Handles both old format (plain string) and new format (rich object).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SchemaSegmentJson {
    Rich(SchemaSegmentRichJson),
    Simple(String),
}

#[derive(Debug, Deserialize)]
struct SchemaSegmentRichJson {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    elements: Vec<SchemaElementJson>,
}

#[derive(Debug, Deserialize)]
struct SchemaElementJson {
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default, rename = "type")]
    element_type: Option<String>,
    #[serde(default)]
    composite: Option<String>,
    #[serde(default)]
    codes: Vec<SchemaCodeJson>,
    #[serde(default)]
    components: Vec<SchemaComponentJson>,
}

#[derive(Debug, Deserialize)]
struct SchemaCodeJson {
    value: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SchemaComponentJson {
    sub_index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default, rename = "type")]
    element_type: Option<String>,
    #[serde(default)]
    codes: Vec<SchemaCodeJson>,
}

#[derive(Debug, Deserialize)]
struct SchemaDiscriminatorJson {
    segment: String,
    #[serde(default)]
    values: Option<Vec<String>>,
    #[allow(dead_code)]
    #[serde(default)]
    element: Option<String>,
}

/// Load the PID schema from a pre-generated JSON file.
pub fn load_pid_schema(path: &Path) -> Result<Vec<SchemaGroup>, GeneratorError> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| GeneratorError::FileNotFound(path.to_path_buf()))?;
    let schema: SchemaJson = serde_json::from_str(&content)?;

    let mut groups: Vec<SchemaGroup> = schema
        .fields
        .into_iter()
        .map(|(name, field)| parse_schema_field(name, field))
        .collect();

    // Sort for deterministic output
    groups.sort_by(|a, b| a.field_name.cmp(&b.field_name));
    groups.iter_mut().for_each(sort_children_recursive);

    Ok(groups)
}

fn parse_schema_field(name: String, field: SchemaFieldJson) -> SchemaGroup {
    let qualifier = field
        .discriminator
        .as_ref()
        .and_then(|d| d.values.as_ref())
        .and_then(|v| v.first())
        .cloned();

    let disc_segment = field.discriminator.as_ref().map(|d| d.segment.clone());

    let children = field
        .children
        .unwrap_or_default()
        .into_iter()
        .map(|(cname, cfield)| parse_schema_field(cname, cfield))
        .collect();

    let segments = field
        .segments
        .into_iter()
        .map(|s| match s {
            SchemaSegmentJson::Simple(id) => SchemaSegmentInfo {
                id,
                name: None,
                elements: Vec::new(),
            },
            SchemaSegmentJson::Rich(rich) => SchemaSegmentInfo {
                id: rich.id,
                name: rich.name,
                elements: rich
                    .elements
                    .into_iter()
                    .map(|el| {
                        let codes: Vec<SchemaCodeInfo> = el
                            .codes
                            .into_iter()
                            .map(|c| SchemaCodeInfo {
                                value: c.value,
                                name: c.name.unwrap_or_default(),
                            })
                            .collect();
                        let components: Vec<SchemaComponentInfo> = el
                            .components
                            .into_iter()
                            .map(|c| SchemaComponentInfo {
                                sub_index: c.sub_index,
                                id: c.id.unwrap_or_default(),
                                name: c.name.unwrap_or_default(),
                                element_type: c.element_type.unwrap_or_default(),
                                codes: c
                                    .codes
                                    .into_iter()
                                    .map(|cc| SchemaCodeInfo {
                                        value: cc.value,
                                        name: cc.name.unwrap_or_default(),
                                    })
                                    .collect(),
                            })
                            .collect();
                        SchemaElementInfo {
                            index: el.index,
                            id: el.id.unwrap_or_default(),
                            name: el.name.unwrap_or_default(),
                            element_type: el.element_type.unwrap_or_default(),
                            composite_id: el.composite,
                            codes,
                            components,
                        }
                    })
                    .collect(),
            },
        })
        .collect();

    SchemaGroup {
        field_name: name,
        source_group: field.source_group,
        qualifier,
        disc_segment,
        segments,
        children,
    }
}

fn sort_children_recursive(group: &mut SchemaGroup) {
    group
        .children
        .sort_by(|a, b| a.field_name.cmp(&b.field_name));
    for child in &mut group.children {
        sort_children_recursive(child);
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Report of what was matched vs scaffolded.
#[derive(Debug)]
pub struct GenerationReport {
    pub matched: Vec<(String, String)>,
    pub scaffolded: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

/// Generate TOML mapping files for a PID by reusing reference mappings.
///
/// Returns `(Vec<(filename, content)>, report)`.
pub fn generate_pid_mappings(
    _pid_id: &str,
    schema_path: &Path,
    mappings_base: &Path,
    fv: &str,
    variant: &str,
) -> Result<(Vec<(String, String)>, GenerationReport), GeneratorError> {
    let index = ReferenceIndex::load(mappings_base, fv, variant)?;
    let schema_groups = load_pid_schema(schema_path)?;

    let mut files = Vec::new();
    let mut report = GenerationReport {
        matched: Vec::new(),
        scaffolded: Vec::new(),
        warnings: Vec::new(),
    };

    // 1. Root mapping (nachricht)
    let root_key = ("".to_string(), "root".to_string());
    if let Some(rm) = index.get(&root_key) {
        let mut def = rm.definition.clone();
        // Root mapping: source_group and source_path stay empty
        def.meta.source_path = Some(String::new());
        let content = serialize_mapping(&def);
        files.push(("nachricht.toml".to_string(), content));
        report
            .matched
            .push(("Nachricht".to_string(), rm.source_pid.clone()));
    }

    // 2. Walk schema groups recursively
    for group in &schema_groups {
        generate_group_mappings(
            group,
            &group.field_name,
            None, // no parent qualifier
            &index,
            &mut files,
            &mut report,
        );
    }

    Ok((files, report))
}

fn generate_group_mappings(
    group: &SchemaGroup,
    source_path: &str,
    parent_qualifier: Option<&str>,
    index: &ReferenceIndex,
    files: &mut Vec<(String, String)>,
    report: &mut GenerationReport,
) {
    // Build the reference index key for this group
    let key = build_lookup_key(group, parent_qualifier);

    if !group.segments.is_empty() {
        let entity_name = derive_entity_name(group);

        if let Some(rm) = index.get(&key) {
            // Use the reference entity name for filename (e.g., "zaehlpunkt" not "sg8_z79")
            let filename = derive_filename_from_entity(&rm.definition.meta.entity);
            // Clone and adapt the reference mapping
            let mut def = rm.definition.clone();
            adapt_mapping(&mut def, group, source_path);

            // Validate element paths against target schema
            let validation_warnings =
                validate_mapping_against_schema(&def, group, &entity_name);
            for w in &validation_warnings {
                eprintln!("WARN: {w}");
            }
            report.warnings.extend(validation_warnings);

            let content = serialize_mapping(&def);
            files.push((filename, content));
            report.matched.push((entity_name, rm.source_pid.clone()));
        } else {
            // Use source_path-derived filename for unique scaffolds
            let filename = derive_filename_from_path(source_path);
            // Generate scaffold fallback
            let content = generate_scaffold_fallback(group, source_path, &entity_name);
            files.push((filename, content));
            report
                .scaffolded
                .push((entity_name, "no reference mapping found".to_string()));
        }
    }

    // Recurse into children
    for child in &group.children {
        let child_path = format!("{}.{}", source_path, child.field_name);
        generate_group_mappings(
            child,
            &child_path,
            group.qualifier.as_deref(),
            index,
            files,
            report,
        );
    }
}

/// Build the reference index lookup key for a schema group.
fn build_lookup_key(group: &SchemaGroup, parent_qualifier: Option<&str>) -> (String, String) {
    let sg = &group.source_group;

    // SG10 with parent qualifier
    if sg == "SG10" {
        if let Some(pq) = parent_qualifier {
            return ("SG10".to_string(), format!("parent:{}", pq.to_uppercase()));
        }
    }

    // Has an explicit qualifier value
    if let Some(ref q) = group.qualifier {
        return (sg.clone(), q.clone());
    }

    // No qualifier
    (sg.clone(), String::new())
}

/// Derive entity name from group for reporting and scaffold.
fn derive_entity_name(group: &SchemaGroup) -> String {
    // From field_name: "sg8_z79" → "SG8_Z79", "sg2" → "SG2"
    group.field_name.to_uppercase().replace("_", "")
}

/// Derive filename from the reference entity name (e.g., "Zaehlpunkt" → "zaehlpunkt.toml").
fn derive_filename_from_entity(entity: &str) -> String {
    // Convert PascalCase entity to snake_case filename
    let mut result = String::new();
    for (i, ch) in entity.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    format!("{result}.toml")
}

/// Derive filename from source_path for scaffolded groups (ensures uniqueness).
/// "sg4.sg8_z98.sg10" → "sg8_z98_sg10.toml", "sg4" → "sg4.toml"
fn derive_filename_from_path(source_path: &str) -> String {
    let parts: Vec<&str> = source_path.split('.').collect();
    // Skip the top-level "sg4" prefix for children to keep names shorter
    let name = if parts.len() > 2 {
        parts[1..].join("_")
    } else {
        parts.join("_")
    };
    format!("{name}.toml")
}

/// Validate a reference mapping's element paths against the target schema.
///
/// Checks that:
/// 1. Segment IDs referenced in field paths exist in the schema group
/// 2. CAV discriminator codes (e.g., `cav[ZH9]`) exist in the schema's CAV segment
/// 3. Element indices don't exceed the schema's element count
fn validate_mapping_against_schema(
    def: &MappingDefinition,
    group: &SchemaGroup,
    entity_name: &str,
) -> Vec<String> {
    let mut warnings = Vec::new();

    // Build a lookup of schema segments by lowercase ID
    let schema_segs: HashMap<String, &SchemaSegmentInfo> = group
        .segments
        .iter()
        .map(|s| (s.id.to_lowercase(), s))
        .collect();

    // Collect CAV codes from schema (first component codes of composite C889)
    let cav_codes: HashSet<String> = schema_segs
        .get("cav")
        .map(|seg| {
            seg.elements
                .iter()
                .flat_map(|el| {
                    // Check composite-level codes (e.g., C889 discriminator)
                    el.components
                        .iter()
                        .filter(|c| c.sub_index == 0)
                        .flat_map(|c| c.codes.iter().map(|code| code.value.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    // Validate both fields and companion_fields sections
    let empty = BTreeMap::new();
    let sections: Vec<(&str, &BTreeMap<String, FieldMapping>)> = vec![
        ("fields", &def.fields),
        (
            "companion_fields",
            def.companion_fields.as_ref().unwrap_or(&empty),
        ),
    ];

    for (section, fields) in sections {
        for path in fields.keys() {
            // Parse "seg_id" and optional "[CODE]" from paths like "cav[ZH9].0.3" or "cci.0"
            let (seg_id, disc_code) = parse_element_path(path);

            // Check segment exists in schema
            if !seg_id.is_empty() && !schema_segs.contains_key(&seg_id) {
                warnings.push(format!(
                    "{entity_name}: [{section}] \"{path}\" references segment '{}' \
                     not found in schema (available: {})",
                    seg_id.to_uppercase(),
                    schema_segs
                        .keys()
                        .map(|k| k.to_uppercase())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            // Check CAV discriminator code exists
            if let Some(code) = disc_code {
                if !cav_codes.is_empty() && !cav_codes.contains(&code) {
                    warnings.push(format!(
                        "{entity_name}: [{section}] \"{path}\" uses CAV discriminator code '{}' \
                         not found in schema (available: {})",
                        code,
                        cav_codes.iter().cloned().collect::<Vec<_>>().join(", ")
                    ));
                }
            }

            // Check element index against schema
            if let Some(seg_info) = schema_segs.get(&seg_id) {
                if let Some(idx) = parse_element_index(path) {
                    let max_idx = seg_info.elements.iter().map(|e| e.index).max().unwrap_or(0);
                    if idx > max_idx {
                        warnings.push(format!(
                            "{entity_name}: [{section}] \"{path}\" references element index {idx} \
                             but {} only has indices 0..{max_idx}",
                            seg_id.to_uppercase()
                        ));
                    }
                }
            }
        }
    }

    warnings
}

/// Parse segment ID and optional discriminator code from an element path.
///
/// Examples:
/// - `"cci.0"` → `("cci", None)`
/// - `"cav[ZH9].0.3"` → `("cav", Some("ZH9"))`
/// - `"loc.1.0"` → `("loc", None)`
fn parse_element_path(path: &str) -> (String, Option<String>) {
    let dot_pos = path.find('.').unwrap_or(path.len());
    let seg_part = &path[..dot_pos];

    if let Some(bracket_start) = seg_part.find('[') {
        let seg_id = seg_part[..bracket_start].to_lowercase();
        let bracket_end = seg_part.find(']').unwrap_or(seg_part.len());
        let code = seg_part[bracket_start + 1..bracket_end].to_string();
        (seg_id, Some(code))
    } else {
        (seg_part.to_lowercase(), None)
    }
}

/// Parse the first element index from a path like "cci.3" → Some(3), "cav[ZV4].0.3" → Some(0).
fn parse_element_index(path: &str) -> Option<usize> {
    // Skip segment part (before first dot)
    let after_seg = path.find('.')?.checked_add(1)?;
    let rest = &path[after_seg..];
    // Take the first numeric component
    let idx_str = rest.split('.').next()?;
    idx_str.parse().ok()
}

/// Adapt a cloned reference mapping for the target PID's schema.
fn adapt_mapping(def: &mut MappingDefinition, group: &SchemaGroup, source_path: &str) {
    // Update source_path to match the new PID's field hierarchy
    def.meta.source_path = Some(source_path.to_string());

    // For SG10 mappings, the source_group contains a rep index (e.g., "SG4.SG8:0.SG10")
    // that may differ between PIDs. Strip the rep index — the engine resolves via discriminator.
    if group.source_group == "SG10" {
        // Rebuild source_group from parent path: "sg4.sg8_z79.sg10" → "SG4.SG8.SG10"
        let parts: Vec<&str> = source_path.split('.').collect();
        let mut sg_parts = Vec::new();
        for part in &parts {
            // Extract the SG prefix from field names like "sg8_z79" → "SG8"
            let sg = part.split('_').next().unwrap_or(part).to_uppercase();
            sg_parts.push(sg);
        }
        def.meta.source_group = sg_parts.join(".");
    }
}

/// Generate a scaffold TOML for groups with no reference mapping.
fn generate_scaffold_fallback(group: &SchemaGroup, source_path: &str, entity_name: &str) -> String {
    let mut out = String::new();
    out.push_str("# TODO: Fill in target fields — no reference mapping found\n");
    out.push_str(&format!(
        "# AUTO-GENERATED scaffold for {} ({})\n\n",
        entity_name, group.source_group
    ));
    out.push_str("[meta]\n");
    out.push_str(&format!("entity = \"{}\"\n", entity_name));
    out.push_str(&format!("bo4e_type = \"{}\"\n", entity_name));

    // Build source_group from parent hierarchy
    out.push_str(&format!(
        "source_group = \"{}\"\n",
        build_source_group(source_path)
    ));
    out.push_str(&format!("source_path = \"{}\"\n", source_path));

    // Add discriminator if group has a qualifier
    if let Some(ref q) = group.qualifier {
        if let Some(ref seg) = group.disc_segment {
            out.push_str(&format!(
                "discriminator = \"{}.0.0={}\"\n",
                seg.to_uppercase(),
                q
            ));
        }
    }

    out.push_str("\n[fields]\n");
    // List known segments with element metadata as comments
    for seg_info in &group.segments {
        if seg_info.elements.is_empty() {
            // No metadata — plain comment
            out.push_str(&format!(
                "# \"{}.0\" = {{ target = \"\" }}\n",
                seg_info.id.to_lowercase()
            ));
        } else {
            let seg_lower = seg_info.id.to_lowercase();
            let seg_name = seg_info.name.as_deref().unwrap_or(&seg_info.id);
            out.push_str(&format!("# --- {} ({}) ---\n", seg_info.id, seg_name));
            for el in &seg_info.elements {
                if let Some(ref comp_id) = el.composite_id {
                    // Composite element
                    out.push_str(&format!("# {} {}\n", comp_id, el.name));
                    for comp in &el.components {
                        let codes_hint = format_codes_hint(&comp.codes);
                        out.push_str(&format!("#   D{} {}{}\n", comp.id, comp.name, codes_hint));
                        out.push_str(&format!(
                            "# \"{}.{}.{}\" = {{ target = \"\" }}\n",
                            seg_lower, el.index, comp.sub_index
                        ));
                    }
                } else {
                    // Direct data element
                    let codes_hint = format_codes_hint(&el.codes);
                    out.push_str(&format!("# D{} {}{}\n", el.id, el.name, codes_hint));
                    out.push_str(&format!(
                        "# \"{}.{}\" = {{ target = \"\" }}\n",
                        seg_lower, el.index
                    ));
                }
            }
        }
    }

    out
}

/// Format a short codes hint like ` [Z18=Regelzone, Z66=Zaehlpunkttyp]`.
fn format_codes_hint(codes: &[SchemaCodeInfo]) -> String {
    if codes.is_empty() {
        return String::new();
    }
    let entries: Vec<String> = codes
        .iter()
        .take(5) // Limit for readability
        .map(|c| {
            if c.name.is_empty() {
                c.value.clone()
            } else {
                format!("{}={}", c.value, c.name)
            }
        })
        .collect();
    let suffix = if codes.len() > 5 { ", ..." } else { "" };
    format!(" [{}{}]", entries.join(", "), suffix)
}

/// Build dotted source_group from source_path.
/// "sg4.sg8_z79.sg10" → "SG4.SG8.SG10"
fn build_source_group(source_path: &str) -> String {
    source_path
        .split('.')
        .map(|part| part.split('_').next().unwrap_or(part).to_uppercase())
        .collect::<Vec<_>>()
        .join(".")
}

/// Serialize a MappingDefinition to TOML string.
fn serialize_mapping(def: &MappingDefinition) -> String {
    // Use toml serialization
    match toml::to_string_pretty(def) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("WARN: failed to serialize mapping: {e}");
            format!("# ERROR: serialization failed: {e}\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mappings_base() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("mappings")
    }

    fn schema_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("crates/mig-types/src/generated/fv2504/utilmd/pids")
    }

    #[test]
    fn test_reference_index_loads() {
        let base = mappings_base();
        if !base.join("FV2504/UTILMD_Strom/pid_55001").exists() {
            eprintln!("Skipping: mappings not found");
            return;
        }

        let index = ReferenceIndex::load(&base, "FV2504", "UTILMD_Strom").unwrap();

        // Should have entries from pid_55001
        let keys: Vec<_> = index.keys().collect();
        assert!(!keys.is_empty(), "Index should have entries");

        // Check expected keys
        assert!(
            index.get(&("SG8".to_string(), "Z79".to_string())).is_some(),
            "Should have SG8+Z79 (Zaehlpunkt)"
        );
        assert!(
            index.get(&("SG8".to_string(), "ZH0".to_string())).is_some(),
            "Should have SG8+ZH0 (Messstellenbetrieb)"
        );
        assert!(
            index.get(&("SG8".to_string(), "Z01".to_string())).is_some(),
            "Should have SG8+Z01 (Geraet)"
        );
        assert!(
            index.get(&("SG8".to_string(), "Z75".to_string())).is_some(),
            "Should have SG8+Z75 (Netznutzungsabrechnung)"
        );
        assert!(
            index
                .get(&("SG12".to_string(), "Z04".to_string()))
                .is_some(),
            "Should have SG12+Z04 (Geschaeftspartner)"
        );
        assert!(
            index
                .get(&("SG12".to_string(), "Z09".to_string()))
                .is_some(),
            "Should have SG12+Z09 (Ansprechpartner)"
        );
        assert!(
            index.get(&("".to_string(), "root".to_string())).is_some(),
            "Should have root mapping (Nachricht)"
        );
        assert!(
            index
                .get(&("SG10".to_string(), "parent:Z79".to_string()))
                .is_some(),
            "Should have SG10+parent:Z79 (Zaehlpunkt zuordnung)"
        );
        assert!(
            index
                .get(&("SG10".to_string(), "parent:ZH0".to_string()))
                .is_some(),
            "Should have SG10+parent:ZH0 (Messstellenbetrieb zuordnung)"
        );
        assert!(
            index.get(&("SG4".to_string(), "".to_string())).is_some(),
            "Should have SG4 (Prozessdaten)"
        );
        assert!(
            index.get(&("SG2".to_string(), "".to_string())).is_some(),
            "Should have SG2 (Marktteilnehmer)"
        );
    }

    #[test]
    fn test_generate_self_roundtrip_55001() {
        let base = mappings_base();
        let schema = schema_dir().join("pid_55001_schema.json");
        if !base.join("FV2504/UTILMD_Strom/pid_55001").exists() || !schema.exists() {
            eprintln!("Skipping: mappings or schema not found");
            return;
        }

        let (files, report) =
            generate_pid_mappings("55001", &schema, &base, "FV2504", "UTILMD_Strom").unwrap();

        // Some groups have no TOML mappings (SG3 contact, SG5_Z22 Messlokation)
        // and will be scaffolded. All entities that DO have mappings should match.
        let scaffolded_names: Vec<&str> =
            report.scaffolded.iter().map(|(e, _)| e.as_str()).collect();
        for name in &scaffolded_names {
            assert!(
                *name == "SG3IC" || *name == "SG5Z22",
                "Unexpected scaffolded entity: {} (expected only SG3IC, SG5Z22)",
                name
            );
        }

        // Should generate files
        assert!(!files.is_empty(), "Should generate mapping files");

        // Verify key files are present
        let filenames: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            filenames.contains(&"nachricht.toml"),
            "Should have nachricht.toml"
        );
        assert!(
            filenames.contains(&"prozessdaten.toml"),
            "Should have prozessdaten.toml (from Prozessdaten entity), got: {:?}",
            filenames
        );

        eprintln!(
            "Generated {} files, {} matched, {} scaffolded",
            files.len(),
            report.matched.len(),
            report.scaffolded.len()
        );
        for (entity, pid) in &report.matched {
            eprintln!("  MATCH: {} (from pid_{})", entity, pid);
        }
    }

    #[test]
    fn test_generate_55109_partial_match() {
        let base = mappings_base();
        let schema = schema_dir().join("pid_55109_schema.json");
        if !base.join("FV2504/UTILMD_Strom/pid_55001").exists() || !schema.exists() {
            eprintln!("Skipping: mappings or schema not found");
            return;
        }

        let (files, report) =
            generate_pid_mappings("55109", &schema, &base, "FV2504", "UTILMD_Strom").unwrap();

        assert!(!files.is_empty(), "Should generate mapping files");

        // 55109 has SG8+Z01 and SG8+Z75, SG12+Z04, SG12+Z09 — these should match
        let matched_entities: Vec<&str> = report.matched.iter().map(|(e, _)| e.as_str()).collect();
        eprintln!("Matched: {:?}", matched_entities);
        eprintln!("Scaffolded: {:?}", report.scaffolded);

        // Entities that exist in both 55001 and 55109
        assert!(
            matched_entities
                .iter()
                .any(|e| e.contains("SG8Z01") || e.contains("Geraet")),
            "SG8+Z01 (Geraet) should match: {:?}",
            matched_entities
        );
        assert!(
            matched_entities
                .iter()
                .any(|e| e.contains("SG8Z75") || e.contains("Netznutzung")),
            "SG8+Z75 should match: {:?}",
            matched_entities
        );

        // 55109 does NOT have SG8+Z79 or SG8+ZH0
        let all_filenames: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            !all_filenames.contains(&"zaehlpunkt.toml"),
            "55109 should not have zaehlpunkt.toml (Z79)"
        );
        assert!(
            !all_filenames.contains(&"messstellenbetrieb.toml"),
            "55109 should not have messstellenbetrieb.toml (ZH0)"
        );
    }
}
