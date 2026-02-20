//! Generate filled TOML mappings for a PID using reference mappings from existing PIDs.
//!
//! Scans all `mappings/{fv}/{variant}/pid_*/` directories to build a reference index
//! keyed by `(leaf_group_id, qualifier_value)`. For a new PID, walks its schema JSON
//! and copies matching reference mappings (updating `source_path` and `source_group`
//! as needed). Unmatched groups fall back to scaffold generation.

use std::collections::{BTreeMap, HashMap};
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
    pub segments: Vec<String>,
    pub children: Vec<SchemaGroup>,
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
    segments: Vec<String>,
    #[serde(default)]
    discriminator: Option<SchemaDiscriminatorJson>,
    #[serde(default)]
    children: Option<HashMap<String, SchemaFieldJson>>,
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

    SchemaGroup {
        field_name: name,
        source_group: field.source_group,
        qualifier,
        disc_segment,
        segments: field.segments,
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
        let filename = derive_filename(group);

        if let Some(rm) = index.get(&key) {
            // Clone and adapt the reference mapping
            let mut def = rm.definition.clone();
            adapt_mapping(&mut def, group, source_path);
            let content = serialize_mapping(&def);
            files.push((filename, content));
            report.matched.push((entity_name, rm.source_pid.clone()));
        } else {
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

/// Derive output filename from group field_name.
fn derive_filename(group: &SchemaGroup) -> String {
    // Use the entity name from the reference mapping if possible,
    // otherwise use the field_name
    format!("{}.toml", group.field_name)
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
    // List known segments as comments
    for seg in &group.segments {
        out.push_str(&format!(
            "# \"{}.0\" = {{ target = \"\" }}\n",
            seg.to_lowercase()
        ));
    }

    out
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
            "Should have SG10+parent:Z79 (MerkmalZaehlpunkt)"
        );
        assert!(
            index
                .get(&("SG10".to_string(), "parent:ZH0".to_string()))
                .is_some(),
            "Should have SG10+parent:ZH0 (MerkmalMessstellenbetrieb)"
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
            filenames.contains(&"sg4.toml"),
            "Should have sg4.toml (Prozessdaten)"
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
            !all_filenames.contains(&"sg8_z79.toml"),
            "55109 should not have sg8_z79.toml"
        );
        assert!(
            !all_filenames.contains(&"sg8_zh0.toml"),
            "55109 should not have sg8_zh0.toml"
        );
    }
}
