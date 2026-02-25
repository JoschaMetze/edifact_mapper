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
            }],
            segment_numbers: vec!["0010".to_string()],
        };

        let workflow = ahb_workflow_from_pruefidentifikator(&pruefid);

        assert_eq!(workflow.pruefidentifikator, "55001");
        assert_eq!(workflow.description, "Anmeldung MaLo");
        assert_eq!(workflow.fields.len(), 1);
        assert_eq!(workflow.fields[0].name, "ID der Marktlokation");
    }
}
