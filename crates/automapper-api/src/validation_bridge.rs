//! Bridge between automapper-generator's AhbSchema and automapper-validation's AhbWorkflow.
//!
//! The generator produces [`AhbSchema`] from AHB XML parsing, containing all PIDs
//! and their field definitions. The validator expects an [`AhbWorkflow`] for a
//! specific PID. This module converts between them.

use automapper_generator::schema::ahb::{AhbSchema, Pruefidentifikator};
use automapper_validation::{AhbCodeRule, AhbFieldRule, AhbWorkflow};

/// Convert an [`AhbSchema`] + PID string into an [`AhbWorkflow`] for the validator.
///
/// Returns `None` if the PID is not found in the schema's workflows.
///
/// # Field mapping
///
/// | Generator field | Validator field |
/// |---|---|
/// | `Pruefidentifikator.id` | `AhbWorkflow.pruefidentifikator` |
/// | `Pruefidentifikator.beschreibung` | `AhbWorkflow.description` |
/// | `Pruefidentifikator.kommunikation_von` | `AhbWorkflow.communication_direction` |
/// | `AhbFieldDefinition.name` | `AhbFieldRule.name` |
/// | `AhbCodeValue.name` | `AhbCodeRule.description` |
/// | `AhbCodeValue.ahb_status` (Option) | `AhbCodeRule.ahb_status` (defaults to `"X"`) |
pub fn ahb_workflow_from_schema(schema: &AhbSchema, pid: &str) -> Option<AhbWorkflow> {
    let pruefid = schema.workflows.iter().find(|w| w.id == pid)?;
    Some(ahb_workflow_from_pruefidentifikator(pruefid))
}

/// Convert a single [`Pruefidentifikator`] into an [`AhbWorkflow`].
///
/// This is useful when you already have the specific PID object and don't need
/// to search through the full schema.
/// Build an [`AhbWorkflow`] from an enriched PID schema JSON value.
///
/// The schema must contain `ahb_status` fields on elements/components (produced by
/// the generator with AHB enrichment enabled). Returns `None` if the schema is
/// missing required top-level fields (`pid`, `beschreibung`).
pub fn ahb_workflow_from_pid_schema(schema: &serde_json::Value) -> Option<AhbWorkflow> {
    let pid = schema.get("pid")?.as_str()?;
    let beschreibung = schema.get("beschreibung")?.as_str().unwrap_or("");
    let kommunikation_von = schema
        .get("kommunikation_von")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut fields = Vec::new();

    // Walk groups in "fields" object
    if let Some(groups) = schema.get("fields").and_then(|v| v.as_object()) {
        for (_group_name, group) in groups {
            let source_group = group
                .get("source_group")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let group_ahb_status = group
                .get("ahb_status")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            collect_fields_from_group(&mut fields, group, source_group, &group_ahb_status);
        }
    }

    // Walk root_segments (outside any group)
    if let Some(root_segs) = schema.get("root_segments").and_then(|v| v.as_array()) {
        for seg in root_segs {
            collect_fields_from_segment(&mut fields, seg, "", &None);
        }
    }

    Some(AhbWorkflow {
        pruefidentifikator: pid.to_string(),
        description: beschreibung.to_string(),
        communication_direction: kommunikation_von,
        fields,
    })
}

/// Recursively collect AHB field rules from a group and its children.
fn collect_fields_from_group(
    fields: &mut Vec<AhbFieldRule>,
    group: &serde_json::Value,
    group_path: &str,
    parent_group_ahb_status: &Option<String>,
) {
    // Collect from segments in this group
    if let Some(segments) = group.get("segments").and_then(|v| v.as_array()) {
        for seg in segments {
            collect_fields_from_segment(fields, seg, group_path, parent_group_ahb_status);
        }
    }

    // Recurse into children
    if let Some(children) = group.get("children").and_then(|v| v.as_object()) {
        for (_child_name, child) in children {
            let child_source = child
                .get("source_group")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let child_path = if group_path.is_empty() {
                child_source.to_string()
            } else {
                format!("{}/{}", group_path, child_source)
            };
            let child_group_status = child
                .get("ahb_status")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            collect_fields_from_group(fields, child, &child_path, &child_group_status);
        }
    }
}

/// Collect AHB field rules from a single segment's elements.
fn collect_fields_from_segment(
    fields: &mut Vec<AhbFieldRule>,
    seg: &serde_json::Value,
    group_path: &str,
    parent_group_ahb_status: &Option<String>,
) {
    let seg_id = seg.get("id").and_then(|v| v.as_str()).unwrap_or("");

    if let Some(elements) = seg.get("elements").and_then(|v| v.as_array()) {
        for el in elements {
            // Direct data element (has "id" but no "composite")
            if el.get("composite").is_none() {
                if let Some(de_id) = el.get("id").and_then(|v| v.as_str()) {
                    if let Some(ahb_status) = el.get("ahb_status").and_then(|v| v.as_str()) {
                        let segment_path = if group_path.is_empty() {
                            format!("{}/{}", seg_id, de_id)
                        } else {
                            format!("{}/{}/{}", group_path, seg_id, de_id)
                        };
                        let el_parent_status = el
                            .get("parent_group_ahb_status")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| parent_group_ahb_status.clone());
                        let name = el
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let codes = collect_code_rules(el);
                        fields.push(AhbFieldRule {
                            segment_path,
                            name,
                            ahb_status: ahb_status.to_string(),
                            codes,
                            parent_group_ahb_status: el_parent_status,
                        });
                    }
                }
            }

            // Composite element — walk components
            if let Some(composite_id) = el.get("composite").and_then(|v| v.as_str()) {
                if let Some(components) = el.get("components").and_then(|v| v.as_array()) {
                    for comp in components {
                        let comp_id = comp.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(ahb_status) = comp.get("ahb_status").and_then(|v| v.as_str()) {
                            let segment_path = if group_path.is_empty() {
                                format!("{}/{}/{}", seg_id, composite_id, comp_id)
                            } else {
                                format!("{}/{}/{}/{}", group_path, seg_id, composite_id, comp_id)
                            };
                            let comp_parent_status = comp
                                .get("parent_group_ahb_status")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or_else(|| parent_group_ahb_status.clone());
                            let name = comp
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let codes = collect_code_rules(comp);
                            fields.push(AhbFieldRule {
                                segment_path,
                                name,
                                ahb_status: ahb_status.to_string(),
                                codes,
                                parent_group_ahb_status: comp_parent_status,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Collect code rules from an element or component's "codes" array.
fn collect_code_rules(el: &serde_json::Value) -> Vec<AhbCodeRule> {
    let Some(codes) = el.get("codes").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    codes
        .iter()
        .map(|c| AhbCodeRule {
            value: c
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: c
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ahb_status: c
                .get("ahb_status")
                .and_then(|v| v.as_str())
                .unwrap_or("X")
                .to_string(),
        })
        .collect()
}

pub fn ahb_workflow_from_pruefidentifikator(pruefid: &Pruefidentifikator) -> AhbWorkflow {
    AhbWorkflow {
        pruefidentifikator: pruefid.id.clone(),
        description: pruefid.beschreibung.clone(),
        communication_direction: pruefid.kommunikation_von.clone(),
        fields: pruefid
            .fields
            .iter()
            .map(|f| AhbFieldRule {
                segment_path: f.segment_path.clone(),
                name: f.name.clone(),
                ahb_status: f.ahb_status.clone(),
                codes: f
                    .codes
                    .iter()
                    .map(|c| AhbCodeRule {
                        value: c.value.clone(),
                        description: c.name.clone(),
                        ahb_status: c.ahb_status.clone().unwrap_or_else(|| "X".to_string()),
                    })
                    .collect(),
                parent_group_ahb_status: f.parent_group_ahb_status.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automapper_generator::schema::ahb::{AhbCodeValue, AhbFieldDefinition, AhbSchema};

    /// Helper to build a minimal AhbSchema with one PID.
    fn make_schema(pid: &str, fields: Vec<AhbFieldDefinition>) -> AhbSchema {
        AhbSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "2.1".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test.xml".to_string(),
            workflows: vec![Pruefidentifikator {
                id: pid.to_string(),
                beschreibung: "Anmeldung MaLo".to_string(),
                kommunikation_von: Some("NB an LF".to_string()),
                fields,
                segment_numbers: vec!["0001".to_string(), "0002".to_string()],
            }],
            bedingungen: vec![],
        }
    }

    #[test]
    fn test_bridge_returns_none_for_unknown_pid() {
        let schema = make_schema("55001", vec![]);
        assert!(ahb_workflow_from_schema(&schema, "99999").is_none());
    }

    #[test]
    fn test_bridge_maps_pid_metadata() {
        let schema = make_schema("55001", vec![]);
        let workflow = ahb_workflow_from_schema(&schema, "55001").unwrap();

        assert_eq!(workflow.pruefidentifikator, "55001");
        assert_eq!(workflow.description, "Anmeldung MaLo");
        assert_eq!(
            workflow.communication_direction,
            Some("NB an LF".to_string())
        );
        assert!(workflow.fields.is_empty());
    }

    #[test]
    fn test_bridge_maps_fields() {
        let fields = vec![AhbFieldDefinition {
            segment_path: "SG2/NAD/3035".to_string(),
            name: "Partnerrolle".to_string(),
            ahb_status: "Muss [182] \u{2227} [152]".to_string(),
            description: Some("Rolle des Absenders".to_string()),
            codes: vec![],
            mig_number: Some("0042".to_string()),
            parent_group_ahb_status: None,
        }];

        let schema = make_schema("55001", fields);
        let workflow = ahb_workflow_from_schema(&schema, "55001").unwrap();

        assert_eq!(workflow.fields.len(), 1);
        let field = &workflow.fields[0];
        assert_eq!(field.segment_path, "SG2/NAD/3035");
        assert_eq!(field.name, "Partnerrolle");
        assert_eq!(field.ahb_status, "Muss [182] \u{2227} [152]");
        assert!(field.codes.is_empty());
    }

    #[test]
    fn test_bridge_maps_codes_with_explicit_status() {
        let fields = vec![AhbFieldDefinition {
            segment_path: "SG4/IDE/C206/7140".to_string(),
            name: "Transaktionsgrund".to_string(),
            ahb_status: "X".to_string(),
            description: None,
            codes: vec![
                AhbCodeValue {
                    value: "E01".to_string(),
                    name: "Einzug/Neuanlage".to_string(),
                    description: Some("Neuanlage einer Marktlokation".to_string()),
                    ahb_status: Some("X [494]".to_string()),
                },
                AhbCodeValue {
                    value: "E03".to_string(),
                    name: "Wechsel".to_string(),
                    description: None,
                    ahb_status: Some("X [931]".to_string()),
                },
            ],
            mig_number: None,
            parent_group_ahb_status: None,
        }];

        let schema = make_schema("55001", fields);
        let workflow = ahb_workflow_from_schema(&schema, "55001").unwrap();

        let field = &workflow.fields[0];
        assert_eq!(field.codes.len(), 2);

        assert_eq!(field.codes[0].value, "E01");
        assert_eq!(field.codes[0].description, "Einzug/Neuanlage");
        assert_eq!(field.codes[0].ahb_status, "X [494]");

        assert_eq!(field.codes[1].value, "E03");
        assert_eq!(field.codes[1].description, "Wechsel");
        assert_eq!(field.codes[1].ahb_status, "X [931]");
    }

    #[test]
    fn test_bridge_code_status_defaults_to_x() {
        let fields = vec![AhbFieldDefinition {
            segment_path: "SG2/NAD/3035".to_string(),
            name: "Partnerrolle".to_string(),
            ahb_status: "Muss".to_string(),
            description: None,
            codes: vec![AhbCodeValue {
                value: "MS".to_string(),
                name: "Messstellenbetreiber".to_string(),
                description: None,
                ahb_status: None, // No explicit status
            }],
            mig_number: None,
            parent_group_ahb_status: None,
        }];

        let schema = make_schema("55001", fields);
        let workflow = ahb_workflow_from_schema(&schema, "55001").unwrap();

        // When AhbCodeValue.ahb_status is None, default to "X"
        assert_eq!(workflow.fields[0].codes[0].ahb_status, "X");
    }

    #[test]
    fn test_bridge_communication_direction_none() {
        let schema = AhbSchema {
            message_type: "UTILMD".to_string(),
            variant: None,
            version: "2.1".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test.xml".to_string(),
            workflows: vec![Pruefidentifikator {
                id: "11001".to_string(),
                beschreibung: "Test PID".to_string(),
                kommunikation_von: None,
                fields: vec![],
                segment_numbers: vec![],
            }],
            bedingungen: vec![],
        };

        let workflow = ahb_workflow_from_schema(&schema, "11001").unwrap();

        assert_eq!(workflow.pruefidentifikator, "11001");
        assert_eq!(workflow.description, "Test PID");
        assert!(workflow.communication_direction.is_none());
    }

    #[test]
    fn test_bridge_multiple_pids_selects_correct_one() {
        let schema = AhbSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "2.1".to_string(),
            format_version: "FV2504".to_string(),
            source_file: "test.xml".to_string(),
            workflows: vec![
                Pruefidentifikator {
                    id: "55001".to_string(),
                    beschreibung: "Anmeldung".to_string(),
                    kommunikation_von: Some("NB an LF".to_string()),
                    fields: vec![AhbFieldDefinition {
                        segment_path: "SG2/NAD/3035".to_string(),
                        name: "Partnerrolle 55001".to_string(),
                        ahb_status: "Muss".to_string(),
                        description: None,
                        codes: vec![],
                        mig_number: None,
                        parent_group_ahb_status: None,
                    }],
                    segment_numbers: vec![],
                },
                Pruefidentifikator {
                    id: "55002".to_string(),
                    beschreibung: "Bestätigung".to_string(),
                    kommunikation_von: Some("LF an NB".to_string()),
                    fields: vec![AhbFieldDefinition {
                        segment_path: "SG2/NAD/3035".to_string(),
                        name: "Partnerrolle 55002".to_string(),
                        ahb_status: "X".to_string(),
                        description: None,
                        codes: vec![],
                        mig_number: None,
                        parent_group_ahb_status: None,
                    }],
                    segment_numbers: vec![],
                },
            ],
            bedingungen: vec![],
        };

        let w1 = ahb_workflow_from_schema(&schema, "55001").unwrap();
        assert_eq!(w1.description, "Anmeldung");
        assert_eq!(w1.fields[0].name, "Partnerrolle 55001");

        let w2 = ahb_workflow_from_schema(&schema, "55002").unwrap();
        assert_eq!(w2.description, "Bestätigung");
        assert_eq!(w2.fields[0].name, "Partnerrolle 55002");
    }

    #[test]
    fn test_bridge_from_pruefidentifikator_directly() {
        let pruefid = Pruefidentifikator {
            id: "55001".to_string(),
            beschreibung: "Anmeldung MaLo".to_string(),
            kommunikation_von: Some("NB an LF".to_string()),
            fields: vec![AhbFieldDefinition {
                segment_path: "SG4/IDE/C206/7140".to_string(),
                name: "ID der Marktlokation".to_string(),
                ahb_status: "X".to_string(),
                description: None,
                codes: vec![],
                mig_number: Some("0010".to_string()),
                parent_group_ahb_status: None,
            }],
            segment_numbers: vec!["0010".to_string()],
        };

        let workflow = ahb_workflow_from_pruefidentifikator(&pruefid);

        assert_eq!(workflow.pruefidentifikator, "55001");
        assert_eq!(workflow.description, "Anmeldung MaLo");
        assert_eq!(workflow.fields.len(), 1);
        assert_eq!(workflow.fields[0].name, "ID der Marktlokation");
    }

    #[test]
    fn test_schema_workflow_matches_ahb_workflow() {
        use automapper_generator::parsing::ahb_parser;

        let mig_path = std::path::Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        );
        let ahb_path = std::path::Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml",
        );
        let schema_path = std::path::Path::new(
            "../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json",
        );
        if !mig_path.exists() || !ahb_path.exists() || !schema_path.exists() {
            eprintln!("Skipping: test fixtures not found");
            return;
        }

        // Build workflow from AHB XML (old way)
        let ahb = ahb_parser::parse_ahb(ahb_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        let ahb_workflow = ahb_workflow_from_schema(&ahb, "55001").unwrap();

        // Build workflow from PID schema JSON (new way)
        let schema_str = std::fs::read_to_string(schema_path).unwrap();
        let schema: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
        let schema_workflow = ahb_workflow_from_pid_schema(&schema).unwrap();

        // Compare metadata
        assert_eq!(
            ahb_workflow.pruefidentifikator,
            schema_workflow.pruefidentifikator
        );
        assert_eq!(ahb_workflow.description, schema_workflow.description);
        assert_eq!(
            ahb_workflow.communication_direction,
            schema_workflow.communication_direction
        );

        // Log field counts for diagnostics
        eprintln!(
            "AHB workflow: {} fields, Schema workflow: {} fields",
            ahb_workflow.fields.len(),
            schema_workflow.fields.len()
        );

        // Build lookup sets
        let schema_paths: std::collections::HashSet<&str> = schema_workflow
            .fields
            .iter()
            .map(|f| f.segment_path.as_str())
            .collect();
        let ahb_paths: std::collections::HashSet<&str> = ahb_workflow
            .fields
            .iter()
            .map(|f| f.segment_path.as_str())
            .collect();

        // Schema fields that are in groups (not root/transport) should all be in AHB
        for schema_field in &schema_workflow.fields {
            assert!(
                ahb_paths.contains(schema_field.segment_path.as_str()),
                "Schema field {} not found in AHB workflow",
                schema_field.segment_path
            );
        }

        // AHB fields not in schema are expected for root/transport segments (UNH, UNT, BGM, DTM)
        let mut missing_from_schema = Vec::new();
        for ahb_field in &ahb_workflow.fields {
            if !schema_paths.contains(ahb_field.segment_path.as_str()) {
                missing_from_schema.push(&ahb_field.segment_path);
            }
        }
        eprintln!(
            "AHB fields not in schema (root/transport): {:?}",
            missing_from_schema
        );

        // For matching fields, compare ahb_status.
        // Note: Some paths appear multiple times (e.g., CCI/C240/7037 in different SG8 variants).
        // We check that for each schema field, there's at least one AHB field with matching
        // segment_path and ahb_status.
        for schema_field in &schema_workflow.fields {
            let ahb_matches: Vec<_> = ahb_workflow
                .fields
                .iter()
                .filter(|f| f.segment_path == schema_field.segment_path)
                .collect();
            assert!(
                !ahb_matches.is_empty(),
                "Schema field {} should have AHB counterpart",
                schema_field.segment_path
            );
            let status_match = ahb_matches
                .iter()
                .any(|f| f.ahb_status == schema_field.ahb_status);
            assert!(
                status_match,
                "No AHB field with matching ahb_status for {} (schema: {}, ahb: {:?})",
                schema_field.segment_path,
                schema_field.ahb_status,
                ahb_matches
                    .iter()
                    .map(|f| &f.ahb_status)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_ahb_workflow_from_pid_schema() {
        let schema_path = std::path::Path::new(
            "../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json",
        );
        if !schema_path.exists() {
            eprintln!("Skipping: schema not found");
            return;
        }

        let schema_str = std::fs::read_to_string(schema_path).unwrap();
        let schema: serde_json::Value = serde_json::from_str(&schema_str).unwrap();

        let workflow = ahb_workflow_from_pid_schema(&schema).unwrap();

        assert_eq!(workflow.pruefidentifikator, "55001");
        assert!(!workflow.description.is_empty());
        assert!(workflow.communication_direction.is_some());
        assert!(!workflow.fields.is_empty(), "Should have field rules");

        // Check that at least some fields have non-empty ahb_status
        let has_status = workflow
            .fields
            .iter()
            .any(|f| !f.ahb_status.is_empty());
        assert!(has_status, "Should have at least one field with ahb_status");

        // Check that code rules are populated
        let has_codes = workflow.fields.iter().any(|f| !f.codes.is_empty());
        assert!(has_codes, "Should have at least one field with code rules");
    }
}
