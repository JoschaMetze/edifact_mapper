use automapper_generator::schema::common::{
    Cardinality, CodeDefinition, EdifactDataType, EdifactFormat,
};
use automapper_generator::schema::mig::*;

#[test]
fn test_cardinality_from_status() {
    assert_eq!(Cardinality::from_status("M"), Cardinality::Mandatory);
    assert_eq!(Cardinality::from_status("R"), Cardinality::Required);
    assert_eq!(Cardinality::from_status("C"), Cardinality::Conditional);
    assert_eq!(Cardinality::from_status("D"), Cardinality::Dependent);
    assert_eq!(Cardinality::from_status("O"), Cardinality::Optional);
    assert_eq!(Cardinality::from_status("N"), Cardinality::NotUsed);
}

#[test]
fn test_cardinality_is_required() {
    assert!(Cardinality::Mandatory.is_required());
    assert!(Cardinality::Required.is_required());
    assert!(!Cardinality::Conditional.is_required());
    assert!(!Cardinality::Optional.is_required());
}

#[test]
fn test_edifact_format_parse() {
    let f = EdifactFormat::parse("an..35").unwrap();
    assert_eq!(f.data_type, EdifactDataType::Alphanumeric);
    assert_eq!(f.min_length, None);
    assert_eq!(f.max_length, 35);

    let f = EdifactFormat::parse("n13").unwrap();
    assert_eq!(f.data_type, EdifactDataType::Numeric);
    assert_eq!(f.min_length, Some(13));
    assert_eq!(f.max_length, 13);

    let f = EdifactFormat::parse("a3").unwrap();
    assert_eq!(f.data_type, EdifactDataType::Alphabetic);
    assert_eq!(f.min_length, Some(3));
    assert_eq!(f.max_length, 3);

    assert!(EdifactFormat::parse("").is_none());
    assert!(EdifactFormat::parse("xyz").is_none());
}

#[test]
fn test_mig_segment_cardinality() {
    let seg = MigSegment {
        id: "UNH".to_string(),
        name: "Message Header".to_string(),
        description: None,
        counter: Some("0010".to_string()),
        level: 0,
        number: None,
        max_rep_std: 1,
        max_rep_spec: 1,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        example: None,
        data_elements: vec![],
        composites: vec![],
    };
    assert!(seg.cardinality().is_required());
    assert_eq!(seg.max_rep(), 1);
}

#[test]
fn test_mig_schema_serialization_roundtrip() {
    let schema = MigSchema {
        message_type: "UTILMD".to_string(),
        variant: Some("Strom".to_string()),
        version: "S2.1".to_string(),
        publication_date: "20250320".to_string(),
        author: "BDEW".to_string(),
        format_version: "FV2510".to_string(),
        source_file: "test.xml".to_string(),
        segments: vec![MigSegment {
            id: "UNH".to_string(),
            name: "Message Header".to_string(),
            description: Some("Nachrichtenkopfsegment".to_string()),
            counter: Some("0010".to_string()),
            level: 0,
            number: Some("1".to_string()),
            max_rep_std: 1,
            max_rep_spec: 1,
            status_std: Some("M".to_string()),
            status_spec: Some("M".to_string()),
            example: Some("UNH+1+UTILMD:D:11A:UN:S2.1".to_string()),
            data_elements: vec![MigDataElement {
                id: "0062".to_string(),
                name: "Nachrichten-Referenznummer".to_string(),
                description: None,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                format_std: Some("an..14".to_string()),
                format_spec: Some("an..14".to_string()),
                codes: vec![],
                position: 0,
            }],
            composites: vec![MigComposite {
                id: "S009".to_string(),
                name: "Nachrichtenkennung".to_string(),
                description: None,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                data_elements: vec![MigDataElement {
                    id: "0065".to_string(),
                    name: "Nachrichtentyp-Kennung".to_string(),
                    description: None,
                    status_std: Some("M".to_string()),
                    status_spec: Some("M".to_string()),
                    format_std: Some("an..6".to_string()),
                    format_spec: Some("an..6".to_string()),
                    codes: vec![CodeDefinition {
                        value: "UTILMD".to_string(),
                        name: "Stammdaten".to_string(),
                        description: None,
                    }],
                    position: 0,
                }],
                position: 1,
            }],
        }],
        segment_groups: vec![],
    };

    let json = serde_json::to_string_pretty(&schema).unwrap();
    let roundtripped: MigSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtripped.message_type, "UTILMD");
    assert_eq!(roundtripped.variant, Some("Strom".to_string()));
    assert_eq!(roundtripped.segments.len(), 1);
    assert_eq!(
        roundtripped.segments[0].composites[0].data_elements[0].codes[0].value,
        "UTILMD"
    );
}

#[test]
fn test_code_definition() {
    let code = CodeDefinition {
        value: "E40".to_string(),
        name: "Energieart Strom".to_string(),
        description: Some("Electricity energy type".to_string()),
    };
    assert_eq!(code.value, "E40");
    assert_eq!(code.description.as_deref(), Some("Electricity energy type"));
}
