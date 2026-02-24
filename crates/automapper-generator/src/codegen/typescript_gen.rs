//! TypeScript type generation from TOML mapping definitions and PID schema JSONs.
//!
//! Generates `.d.ts` files with per-entity interfaces and per-PID response types.
//! Each TOML mapping contributes fields to an entity; multiple files targeting the
//! same entity are merged (matching the engine's `deep_merge_insert` behavior).

use std::collections::BTreeMap;
use std::path::Path;

use crate::codegen::pid_mapping_gen;
use crate::error::GeneratorError;

// Re-use the local TOML mapping types from pid_mapping_gen to avoid a cyclic
// dependency (mig-bo4e already depends on automapper-generator).
use pid_mapping_gen::{FieldMapping, MappingDefinition};

/// A field on a TypeScript interface.
#[derive(Debug, Clone)]
pub struct TsField {
    /// camelCase field name (e.g., "marktlokationsId").
    pub name: String,
    /// TypeScript type string (e.g., "string", "CodeField").
    pub ts_type: String,
}

/// A collected TypeScript interface (entity or companion).
#[derive(Debug, Clone)]
pub struct TsInterface {
    /// PascalCase interface name (e.g., "Marktlokation").
    pub name: String,
    /// Fields on this interface (sorted by name for deterministic output).
    pub fields: BTreeMap<String, TsField>,
}

/// An entity with optional companion type, collected from TOML mappings.
#[derive(Debug, Clone)]
pub struct TsEntity {
    /// camelCase key in the response (e.g., "marktlokation").
    pub key: String,
    /// The core interface.
    pub interface: TsInterface,
    /// Optional companion interface (e.g., MarktlokationEdifact).
    pub companion: Option<TsInterface>,
    /// Whether this entity appears as an array in the response.
    pub is_array: bool,
}

/// Collected type information for a PID.
#[derive(Debug)]
pub struct PidTypeInfo {
    pub pid: String,
    pub entities: Vec<TsEntity>,
}

/// Collect entity type information from TOML mapping directories.
///
/// `mapping_dirs` are the directories to load TOMLs from (e.g., message/ + pid_55001/).
/// `schema_path` is the PID schema JSON for determining code vs data field types.
pub fn collect_entities(
    mapping_dirs: &[&Path],
    schema_path: &Path,
    pid: &str,
) -> Result<PidTypeInfo, GeneratorError> {
    // Load PID schema for code/data type info
    let schema_groups = pid_mapping_gen::load_pid_schema(schema_path)?;
    let code_fields = build_code_field_index(&schema_groups);

    // Load all TOML definitions from all directories
    let mut all_defs = Vec::new();
    for dir in mapping_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                let def: MappingDefinition =
                    toml::from_str(&content).map_err(|e| GeneratorError::Validation {
                        message: format!("TOML parse error in {}: {}", path.display(), e),
                    })?;
                all_defs.push(def);
            }
        }
    }

    // Group by entity name, merge fields
    let mut entity_map: BTreeMap<String, TsEntity> = BTreeMap::new();

    for def in &all_defs {
        let entity_name = &def.meta.entity;
        let key = to_camel_case(entity_name);

        let entity = entity_map.entry(key.clone()).or_insert_with(|| TsEntity {
            key: key.clone(),
            interface: TsInterface {
                name: entity_name.clone(),
                fields: BTreeMap::new(),
            },
            companion: None,
            is_array: false,
        });

        // Collect [fields] with non-empty targets
        for (path, field_mapping) in &def.fields {
            let target = match field_mapping {
                FieldMapping::Simple(t) => t.as_str(),
                FieldMapping::Structured(s) => s.target.as_str(),
                FieldMapping::Nested(_) => continue,
            };
            if target.is_empty() {
                continue;
            }
            let ts_type = resolve_field_type(path, &def.meta.source_path, &code_fields);
            insert_field(&mut entity.interface.fields, target, &ts_type);
        }

        // Collect [companion_fields]
        if let Some(ref companion_fields) = def.companion_fields {
            let default_companion_name = format!("{}Edifact", entity_name);
            let companion_name = def
                .meta
                .companion_type
                .as_deref()
                .unwrap_or(&default_companion_name);

            let companion = entity.companion.get_or_insert_with(|| TsInterface {
                name: companion_name.to_string(),
                fields: BTreeMap::new(),
            });

            for (path, field_mapping) in companion_fields {
                let target = match field_mapping {
                    FieldMapping::Simple(t) => t.as_str(),
                    FieldMapping::Structured(s) => s.target.as_str(),
                    FieldMapping::Nested(_) => continue,
                };
                if target.is_empty() {
                    continue;
                }
                let ts_type = resolve_field_type(path, &def.meta.source_path, &code_fields);
                insert_field(&mut companion.fields, target, &ts_type);
            }
        }
    }

    // Determine array vs object from schema group structure
    detect_arrays(&mut entity_map, &all_defs);

    let entities: Vec<TsEntity> = entity_map.into_values().collect();
    Ok(PidTypeInfo {
        pid: pid.to_string(),
        entities,
    })
}

/// Insert a field into a fields map, handling dotted paths as nested types.
/// For now, dotted paths create flat fields (e.g., "address.city" -> "address.city": string).
/// A future enhancement could generate nested interfaces.
fn insert_field(fields: &mut BTreeMap<String, TsField>, target: &str, ts_type: &str) {
    // Use the full dotted target as the field name for now.
    // Nested interface generation is a future enhancement.
    fields.entry(target.to_string()).or_insert_with(|| TsField {
        name: target.to_string(),
        ts_type: ts_type.to_string(),
    });
}

/// Index of (source_path, segment_tag, element_index, component_index) -> is_code_field.
/// Built from the PID schema JSON.
type CodeFieldIndex = std::collections::HashSet<(String, String, usize, usize)>;

/// Build an index of which (source_path, segment, element, component) positions are code fields.
fn build_code_field_index(groups: &[pid_mapping_gen::SchemaGroup]) -> CodeFieldIndex {
    let mut index = CodeFieldIndex::new();
    build_code_index_recursive(groups, "", &mut index);
    index
}

fn build_code_index_recursive(
    groups: &[pid_mapping_gen::SchemaGroup],
    parent_path: &str,
    index: &mut CodeFieldIndex,
) {
    for group in groups {
        let path = if parent_path.is_empty() {
            group.field_name.clone()
        } else {
            format!("{}.{}", parent_path, group.field_name)
        };

        for seg in &group.segments {
            for elem in &seg.elements {
                if elem.element_type == "code" {
                    index.insert((path.clone(), seg.id.clone(), elem.index, 0));
                }
                for comp in &elem.components {
                    if comp.element_type == "code" {
                        index.insert((path.clone(), seg.id.clone(), elem.index, comp.sub_index));
                    }
                }
            }
        }

        build_code_index_recursive(&group.children, &path, index);
    }
}

/// Determine the TypeScript type for a TOML field path.
///
/// Cross-references the path against the PID schema to determine if it's a code field.
/// Code fields -> "CodeField", data fields -> "string".
fn resolve_field_type(
    toml_path: &str,
    source_path: &Option<String>,
    code_fields: &CodeFieldIndex,
) -> String {
    let source = match source_path {
        Some(s) => s.as_str(),
        None => return "string".to_string(),
    };

    // Parse toml_path: "cci.2.0" -> (CCI, 2, 0), "loc.1.0" -> (LOC, 1, 0)
    let parts: Vec<&str> = toml_path.split('.').collect();
    if parts.len() < 2 {
        return "string".to_string();
    }

    // Extract segment tag (strip qualifier bracket if present)
    let seg_tag = parts[0]
        .split('[')
        .next()
        .unwrap_or(parts[0])
        .to_uppercase();

    // Parse element index and component index
    let element_idx: usize = match parts[1].parse() {
        Ok(v) => v,
        Err(_) => return "string".to_string(),
    };
    let component_idx: usize = if parts.len() > 2 {
        parts[2].parse().unwrap_or(0)
    } else {
        0
    };

    if code_fields.contains(&(source.to_string(), seg_tag, element_idx, component_idx)) {
        "CodeField".to_string()
    } else {
        "string".to_string()
    }
}

/// Detect which entities should be arrays based on TOML source_group patterns.
///
/// - SG2 entities (Marktteilnehmer) -> array (repeating NAD groups)
/// - Entities with discriminators -> single object
/// - Root-level entities -> single object
fn detect_arrays(entities: &mut BTreeMap<String, TsEntity>, defs: &[MappingDefinition]) {
    for def in defs {
        let key = to_camel_case(&def.meta.entity);
        if let Some(entity) = entities.get_mut(&key) {
            // SG2 is always an array (repeating Marktteilnehmer)
            if def.meta.source_group == "SG2" {
                entity.is_array = true;
            }
            // SG3 within SG2 is also array-context (Kontakt)
            if def.meta.source_group.starts_with("SG2.SG3") {
                entity.is_array = true;
            }
        }
    }
}

/// Convert PascalCase to camelCase.
fn to_camel_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Generate the content of `common.d.ts`.
pub fn emit_common_dts() -> String {
    r#"// Auto-generated by automapper-generator. Do not edit.

/** Enriched code value returned when enrich_codes=true. */
export interface CodeValue {
  code: string;
  meaning: string | null;
}

/**
 * Fields that are code-type in the PID schema.
 * Plain string when enrich_codes=false, CodeValue object when enrich_codes=true.
 */
export type CodeField = string | CodeValue;
"#
    .to_string()
}

/// Generate the content of a per-PID `.d.ts` file.
pub fn emit_pid_dts(info: &PidTypeInfo) -> String {
    let mut out = String::new();

    // Header
    out.push_str("// Auto-generated by automapper-generator. Do not edit.\n\n");

    // Import CodeField if any entity uses it
    let uses_code_field = info.entities.iter().any(|e| {
        e.interface
            .fields
            .values()
            .any(|f| f.ts_type == "CodeField")
            || e.companion
                .as_ref()
                .map(|c| c.fields.values().any(|f| f.ts_type == "CodeField"))
                .unwrap_or(false)
    });
    if uses_code_field {
        out.push_str("import type { CodeField } from \"../../common\";\n\n");
    }

    // Emit entity interfaces
    for entity in &info.entities {
        emit_interface(&mut out, &entity.interface);
        out.push('\n');

        if let Some(ref companion) = entity.companion {
            emit_interface(&mut out, companion);
            out.push('\n');
        }
    }

    // Emit PID response type
    out.push_str(&format!("export interface Pid{}Response {{\n", info.pid));
    for entity in &info.entities {
        if let Some(ref companion) = entity.companion {
            let companion_key = to_camel_case(&companion.name);
            if entity.is_array {
                out.push_str(&format!(
                    "  {}?: ({} & {{\n    {}?: {};\n  }})[];\n",
                    entity.key, entity.interface.name, companion_key, companion.name
                ));
            } else {
                out.push_str(&format!(
                    "  {}?: {} & {{\n    {}?: {};\n  }};\n",
                    entity.key, entity.interface.name, companion_key, companion.name
                ));
            }
        } else if entity.is_array {
            out.push_str(&format!(
                "  {}?: {}[];\n",
                entity.key, entity.interface.name
            ));
        } else {
            out.push_str(&format!("  {}?: {};\n", entity.key, entity.interface.name));
        }
    }
    out.push_str("}\n");

    out
}

/// Emit a single TypeScript interface.
fn emit_interface(out: &mut String, iface: &TsInterface) {
    out.push_str(&format!("export interface {} {{\n", iface.name));
    for field in iface.fields.values() {
        out.push_str(&format!("  {}?: {};\n", field.name, field.ts_type));
    }
    out.push_str("}\n");
}

/// Generate the content of `index.d.ts` (barrel re-exports).
pub fn emit_index_dts(pids: &[&str]) -> String {
    let mut out = String::from("// Auto-generated by automapper-generator. Do not edit.\n\n");
    for pid in pids {
        out.push_str(&format!("export * from \"./pid_{}\";\n", pid));
    }
    out.push_str("export * from \"../../common\";\n");
    out
}

/// Generate TypeScript type definitions for one or more PIDs.
///
/// Creates:
/// - `{output_dir}/common.d.ts` — shared CodeField type
/// - `{output_dir}/{fv}/{variant}/pid_{pid}.d.ts` — per-PID types
/// - `{output_dir}/{fv}/{variant}/index.d.ts` — barrel re-exports
pub fn generate_typescript(
    pids: &[&str],
    schema_dir: &Path,
    mappings_dir: &Path,
    format_version: &str,
    variant: &str,
    output_dir: &Path,
) -> Result<Vec<String>, GeneratorError> {
    let mut generated_files = Vec::new();

    // Write common.d.ts
    let common_path = output_dir.join("common.d.ts");
    std::fs::create_dir_all(output_dir)?;
    std::fs::write(&common_path, emit_common_dts())?;
    generated_files.push("common.d.ts".to_string());

    // Per-PID generation
    let variant_dir = output_dir.join(format_version).join(variant);
    std::fs::create_dir_all(&variant_dir)?;

    let msg_dir = mappings_dir
        .join(format_version)
        .join(variant)
        .join("message");

    for pid in pids {
        let schema_path = schema_dir.join(format!("pid_{}_schema.json", pid.to_lowercase()));
        if !schema_path.exists() {
            return Err(GeneratorError::FileNotFound(schema_path));
        }

        let tx_dir = mappings_dir
            .join(format_version)
            .join(variant)
            .join(format!("pid_{}", pid));

        let dirs: Vec<&Path> = if msg_dir.exists() {
            vec![msg_dir.as_path(), tx_dir.as_path()]
        } else {
            vec![tx_dir.as_path()]
        };

        let info = collect_entities(&dirs, &schema_path, pid)?;
        let content = emit_pid_dts(&info);

        let filename = format!("pid_{}.d.ts", pid);
        std::fs::write(variant_dir.join(&filename), &content)?;
        generated_files.push(format!("{}/{}/{}", format_version, variant, filename));

        eprintln!(
            "PID {}: {} entities, {} with companions",
            pid,
            info.entities.len(),
            info.entities
                .iter()
                .filter(|e| e.companion.is_some())
                .count()
        );
    }

    // Write index.d.ts
    let pid_strs: Vec<&str> = pids.to_vec();
    std::fs::write(variant_dir.join("index.d.ts"), emit_index_dts(&pid_strs))?;
    generated_files.push(format!("{}/{}/index.d.ts", format_version, variant));

    Ok(generated_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_entities_pid_55001() {
        let msg_dir = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../mappings/FV2504/UTILMD_Strom/message"
        ));
        let tx_dir = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../mappings/FV2504/UTILMD_Strom/pid_55001"
        ));
        let schema_path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json"
        ));

        if !schema_path.exists() || !msg_dir.exists() || !tx_dir.exists() {
            eprintln!("Skipping: required files not found");
            return;
        }

        let info = collect_entities(&[msg_dir, tx_dir], schema_path, "55001").unwrap();

        assert_eq!(info.pid, "55001");
        assert!(!info.entities.is_empty());

        // Check known entities exist
        let entity_keys: Vec<&str> = info.entities.iter().map(|e| e.key.as_str()).collect();
        assert!(
            entity_keys.contains(&"marktlokation"),
            "missing marktlokation"
        );
        assert!(
            entity_keys.contains(&"prozessdaten"),
            "missing prozessdaten"
        );
        assert!(
            entity_keys.contains(&"marktteilnehmer"),
            "missing marktteilnehmer"
        );
        assert!(entity_keys.contains(&"nachricht"), "missing nachricht");

        // Marktlokation should have core fields and a companion
        let mkt = info
            .entities
            .iter()
            .find(|e| e.key == "marktlokation")
            .unwrap();
        assert!(mkt.interface.fields.contains_key("marktlokationsId"));
        assert!(
            mkt.companion.is_some(),
            "marktlokation should have companion"
        );
        let companion = mkt.companion.as_ref().unwrap();
        assert_eq!(companion.name, "MarktlokationEdifact");
        assert!(companion.fields.contains_key("haushaltskunde"));
        assert!(!mkt.is_array, "marktlokation should not be array");

        // Marktteilnehmer should be an array
        let mt = info
            .entities
            .iter()
            .find(|e| e.key == "marktteilnehmer")
            .unwrap();
        assert!(mt.is_array, "marktteilnehmer should be array");
        assert!(mt.interface.fields.contains_key("marktrolle"));

        // Prozessdaten should have merged fields from prozessdaten.toml + prozess_referenz.toml
        let pd = info
            .entities
            .iter()
            .find(|e| e.key == "prozessdaten")
            .unwrap();
        assert!(pd.interface.fields.contains_key("vorgangId"));
        assert!(pd.interface.fields.contains_key("pruefidentifikator"));
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("Marktlokation"), "marktlokation");
        assert_eq!(
            to_camel_case("ProduktpaketPriorisierung"),
            "produktpaketPriorisierung"
        );
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn test_resolve_field_type_code() {
        let mut code_fields = CodeFieldIndex::new();
        code_fields.insert(("sg4.sg8_z01.sg10".to_string(), "CCI".to_string(), 2, 0));

        assert_eq!(
            resolve_field_type(
                "cci.2.0",
                &Some("sg4.sg8_z01.sg10".to_string()),
                &code_fields
            ),
            "CodeField"
        );
        assert_eq!(
            resolve_field_type(
                "cci.0.0",
                &Some("sg4.sg8_z01.sg10".to_string()),
                &code_fields
            ),
            "string"
        );
        assert_eq!(resolve_field_type("loc.1.0", &None, &code_fields), "string");
    }

    #[test]
    fn test_emit_common_dts() {
        let content = emit_common_dts();
        assert!(content.contains("export interface CodeValue"));
        assert!(content.contains("code: string"));
        assert!(content.contains("meaning: string | null"));
        assert!(content.contains("export type CodeField = string | CodeValue"));
    }

    #[test]
    fn test_emit_pid_dts_basic() {
        let info = PidTypeInfo {
            pid: "55001".to_string(),
            entities: vec![
                TsEntity {
                    key: "marktlokation".to_string(),
                    interface: TsInterface {
                        name: "Marktlokation".to_string(),
                        fields: BTreeMap::from([(
                            "marktlokationsId".to_string(),
                            TsField {
                                name: "marktlokationsId".to_string(),
                                ts_type: "string".to_string(),
                            },
                        )]),
                    },
                    companion: Some(TsInterface {
                        name: "MarktlokationEdifact".to_string(),
                        fields: BTreeMap::from([(
                            "haushaltskunde".to_string(),
                            TsField {
                                name: "haushaltskunde".to_string(),
                                ts_type: "CodeField".to_string(),
                            },
                        )]),
                    }),
                    is_array: false,
                },
                TsEntity {
                    key: "marktteilnehmer".to_string(),
                    interface: TsInterface {
                        name: "Marktteilnehmer".to_string(),
                        fields: BTreeMap::from([(
                            "marktrolle".to_string(),
                            TsField {
                                name: "marktrolle".to_string(),
                                ts_type: "string".to_string(),
                            },
                        )]),
                    },
                    companion: None,
                    is_array: true,
                },
            ],
        };

        let content = emit_pid_dts(&info);

        // Should have import
        assert!(content.contains("import type { CodeField }"));

        // Entity interfaces
        assert!(content.contains("export interface Marktlokation {"));
        assert!(content.contains("marktlokationsId?: string;"));
        assert!(content.contains("export interface MarktlokationEdifact {"));
        assert!(content.contains("haushaltskunde?: CodeField;"));
        assert!(content.contains("export interface Marktteilnehmer {"));

        // PID response type
        assert!(content.contains("export interface Pid55001Response {"));
        // Companion nested via intersection
        assert!(content.contains("marktlokation?: Marktlokation & {"));
        assert!(content.contains("marktlokationEdifact?: MarktlokationEdifact;"));
        // Array entity
        assert!(content.contains("marktteilnehmer?: Marktteilnehmer[];"));
    }

    #[test]
    fn test_emit_index_dts() {
        let content = emit_index_dts(&["55001", "55002"]);
        assert!(content.contains("export * from \"./pid_55001\";"));
        assert!(content.contains("export * from \"./pid_55002\";"));
    }

    #[test]
    fn test_generate_typescript_pid_55001() {
        let schema_dir = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/mig-types/src/generated/fv2504/utilmd/pids"
        ));
        let mappings_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../mappings"));

        if !schema_dir.join("pid_55001_schema.json").exists() {
            eprintln!("Skipping: PID schema not found");
            return;
        }

        let output_dir = std::env::temp_dir().join("ts_gen_test_55001");
        let _ = std::fs::remove_dir_all(&output_dir);

        generate_typescript(
            &["55001"],
            schema_dir,
            mappings_dir,
            "FV2504",
            "UTILMD_Strom",
            &output_dir,
        )
        .unwrap();

        // Check files were created
        assert!(output_dir.join("common.d.ts").exists());
        assert!(output_dir
            .join("FV2504/UTILMD_Strom/pid_55001.d.ts")
            .exists());
        assert!(output_dir.join("FV2504/UTILMD_Strom/index.d.ts").exists());

        // Check content
        let pid_content =
            std::fs::read_to_string(output_dir.join("FV2504/UTILMD_Strom/pid_55001.d.ts")).unwrap();
        assert!(pid_content.contains("export interface Pid55001Response"));
        assert!(pid_content.contains("export interface Marktlokation"));
        assert!(pid_content.contains("marktlokationsId?: string;"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&output_dir);
    }
}
