use automapper_generator::schema::mig::*;
use mig_assembly::service::ConversionService;

fn make_minimal_mig() -> MigSchema {
    MigSchema {
        message_type: "UTILMD".to_string(),
        variant: Some("Strom".to_string()),
        version: "S2.1".to_string(),
        publication_date: "2025-03-20".to_string(),
        author: "BDEW".to_string(),
        format_version: "FV2504".to_string(),
        source_file: "test".to_string(),
        segments: vec![
            MigSegment {
                id: "UNB".to_string(),
                name: "UNB".to_string(),
                description: None,
                counter: None,
                level: 0,
                number: None,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                example: None,
                data_elements: vec![],
                composites: vec![],
            },
            MigSegment {
                id: "UNH".to_string(),
                name: "UNH".to_string(),
                description: None,
                counter: None,
                level: 0,
                number: None,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                example: None,
                data_elements: vec![],
                composites: vec![],
            },
            MigSegment {
                id: "BGM".to_string(),
                name: "BGM".to_string(),
                description: None,
                counter: None,
                level: 0,
                number: None,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                example: None,
                data_elements: vec![],
                composites: vec![],
            },
            MigSegment {
                id: "UNT".to_string(),
                name: "UNT".to_string(),
                description: None,
                counter: None,
                level: 0,
                number: None,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                example: None,
                data_elements: vec![],
                composites: vec![],
            },
            MigSegment {
                id: "UNZ".to_string(),
                name: "UNZ".to_string(),
                description: None,
                counter: None,
                level: 0,
                number: None,
                max_rep_std: 1,
                max_rep_spec: 1,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                example: None,
                data_elements: vec![],
                composites: vec![],
            },
        ],
        segment_groups: vec![],
    }
}

#[test]
fn test_convert_interchange_single_message() {
    let mig = make_minimal_mig();
    let service = ConversionService::from_mig(mig);

    let input = "UNA:+.? 'UNB+UNOC:3+SEND+RECV+210101:1200+REF'UNH+001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001'UNT+2+001'UNZ+1+REF'";

    let (chunks, trees) = service.convert_interchange_to_trees(input).unwrap();
    assert_eq!(chunks.messages.len(), 1);
    assert_eq!(trees.len(), 1);
    assert!(trees[0].segments.iter().any(|s| s.tag == "BGM"));
}

#[test]
fn test_convert_interchange_two_messages() {
    let mig = make_minimal_mig();
    let service = ConversionService::from_mig(mig);

    let input = "UNA:+.? 'UNB+UNOC:3+SEND+RECV+210101:1200+REF'UNH+001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001'UNT+2+001'UNH+002+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC002'UNT+2+002'UNZ+2+REF'";

    let (chunks, trees) = service.convert_interchange_to_trees(input).unwrap();
    assert_eq!(chunks.messages.len(), 2);
    assert_eq!(trees.len(), 2);
}
