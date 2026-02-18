use automapper_generator::schema::ahb::*;

#[test]
fn test_ahb_field_is_mandatory() {
    let field_muss = AhbFieldDefinition {
        segment_path: "SG2/NAD/3035".to_string(),
        name: "Qualifier".to_string(),
        ahb_status: "Muss".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(field_muss.is_mandatory());

    let field_x = AhbFieldDefinition {
        segment_path: "SG2/NAD/3035".to_string(),
        name: "Qualifier".to_string(),
        ahb_status: "X".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(field_x.is_mandatory());

    let field_conditional = AhbFieldDefinition {
        segment_path: "SG2/NAD/3035".to_string(),
        name: "Qualifier".to_string(),
        ahb_status: "X [931]".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(!field_conditional.is_mandatory());
}

#[test]
fn test_ahb_field_condition_ids() {
    let field = AhbFieldDefinition {
        segment_path: "SG8/SEQ/1245".to_string(),
        name: "Status".to_string(),
        ahb_status: "Muss [1] \u{2227} [2]".to_string(), // "Muss [1] ∧ [2]"
        description: None,
        codes: vec![],
    };
    let ids = field.condition_ids();
    assert_eq!(ids, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn test_ahb_field_no_conditions() {
    let field = AhbFieldDefinition {
        segment_path: "BGM/C002/1001".to_string(),
        name: "Doc Type".to_string(),
        ahb_status: "Muss".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(field.condition_ids().is_empty());
}

#[test]
fn test_bedingung_definition() {
    let bed = BedingungDefinition {
        id: "931".to_string(),
        description: "Wenn Zeitformat korrekt ist".to_string(),
    };
    assert_eq!(bed.id, "931");
}

#[test]
fn test_ahb_schema_serialization_roundtrip() {
    let schema = AhbSchema {
        message_type: "UTILMD".to_string(),
        variant: Some("Strom".to_string()),
        version: "2.1".to_string(),
        format_version: "FV2510".to_string(),
        source_file: "test_ahb.xml".to_string(),
        workflows: vec![Pruefidentifikator {
            id: "55001".to_string(),
            beschreibung: "Lieferantenwechsel".to_string(),
            kommunikation_von: Some("NB an LF".to_string()),
            fields: vec![AhbFieldDefinition {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Qualifier".to_string(),
                ahb_status: "X".to_string(),
                description: None,
                codes: vec![AhbCodeValue {
                    value: "MS".to_string(),
                    name: "Absender".to_string(),
                    description: None,
                    ahb_status: Some("X".to_string()),
                }],
            }],
        }],
        bedingungen: vec![BedingungDefinition {
            id: "1".to_string(),
            description: "Wenn Aufteilung vorhanden".to_string(),
        }],
    };

    let json = serde_json::to_string_pretty(&schema).unwrap();
    let roundtripped: AhbSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtripped.message_type, "UTILMD");
    assert_eq!(roundtripped.workflows.len(), 1);
    assert_eq!(roundtripped.workflows[0].id, "55001");
    assert_eq!(roundtripped.bedingungen.len(), 1);
}

#[test]
fn test_pruefidentifikator_summary() {
    let pid = Pruefidentifikator {
        id: "55001".to_string(),
        beschreibung: "Lieferantenwechsel".to_string(),
        kommunikation_von: Some("NB an LF".to_string()),
        fields: vec![
            AhbFieldDefinition {
                segment_path: "BGM/1001".to_string(),
                name: "Doc Type".to_string(),
                ahb_status: "Muss".to_string(),
                description: None,
                codes: vec![],
            },
            AhbFieldDefinition {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Qualifier".to_string(),
                ahb_status: "X [931]".to_string(),
                description: None,
                codes: vec![],
            },
        ],
    };

    let total = pid.fields.len();
    let mandatory_count = pid.fields.iter().filter(|f| f.is_mandatory()).count();
    assert_eq!(total, 2);
    assert_eq!(mandatory_count, 1); // Only "Muss" is mandatory, "X [931]" is conditional
}
