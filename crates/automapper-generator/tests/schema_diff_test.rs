//! Tests for PID schema diffing.

use automapper_generator::schema_diff::{diff_pid_schemas, DiffInput, PidSchemaDiff};

fn minimal_schema_json(groups: &[(&str, &str, &str)]) -> serde_json::Value {
    // Build a minimal PID schema JSON with specified groups.
    // Each tuple is (field_name, source_group, discriminator_segment:qualifier).
    let mut fields = serde_json::Map::new();
    for (field_name, source_group, disc) in groups {
        let parts: Vec<&str> = disc.split(':').collect();
        let disc_obj = if parts.len() == 2 {
            serde_json::json!({
                "segment": parts[0],
                "element": "3227",
                "values": [parts[1]]
            })
        } else {
            serde_json::Value::Null
        };

        fields.insert(
            field_name.to_string(),
            serde_json::json!({
                "source_group": source_group,
                "discriminator": disc_obj,
                "segments": [],
                "children": null
            }),
        );
    }

    serde_json::json!({
        "pid": "55001",
        "beschreibung": "Test",
        "format_version": "FV2504",
        "fields": fields
    })
}

#[test]
fn test_diff_identical_schemas_has_no_group_changes() {
    let schema = minimal_schema_json(&[
        ("sg5_z16", "SG5", "LOC:Z16"),
        ("sg8_z98", "SG8", "SEQ:Z98"),
    ]);

    let input = DiffInput {
        old_schema: schema.clone(),
        new_schema: schema,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert!(diff.groups.added.is_empty());
    assert!(diff.groups.removed.is_empty());
    assert!(diff.groups.restructured.is_empty());
}

#[test]
fn test_diff_detects_added_group() {
    let old = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16")]);
    let new = minimal_schema_json(&[
        ("sg5_z16", "SG5", "LOC:Z16"),
        ("sg8_zh5", "SG8", "SEQ:ZH5"),
    ]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.groups.added.len(), 1);
    assert_eq!(diff.groups.added[0].group, "sg8_zh5");
}

#[test]
fn test_diff_detects_removed_group() {
    let old = minimal_schema_json(&[
        ("sg5_z16", "SG5", "LOC:Z16"),
        ("sg8_z98", "SG8", "SEQ:Z98"),
    ]);
    let new = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16")]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.groups.removed.len(), 1);
    assert_eq!(diff.groups.removed[0].group, "sg8_z98");
}

fn schema_with_segments(
    groups: &[(&str, &str, &str, &[(&str, &[(usize, &str, &str, &[&str])])])],
) -> serde_json::Value {
    // groups: [(field_name, source_group, disc, segments)]
    // segments: [(tag, elements)]
    // elements: [(index, id, type, codes)]
    let mut fields = serde_json::Map::new();
    for (field_name, source_group, disc, segments) in groups {
        let parts: Vec<&str> = disc.split(':').collect();
        let disc_obj = if parts.len() == 2 {
            serde_json::json!({
                "segment": parts[0],
                "element": "3227",
                "values": [parts[1]]
            })
        } else {
            serde_json::Value::Null
        };

        let segs: Vec<serde_json::Value> = segments
            .iter()
            .map(|(tag, elems)| {
                let elements: Vec<serde_json::Value> = elems
                    .iter()
                    .map(|(idx, id, etype, codes)| {
                        let code_arr: Vec<serde_json::Value> = codes
                            .iter()
                            .map(|c| serde_json::json!({"value": c, "name": c}))
                            .collect();
                        serde_json::json!({
                            "index": idx,
                            "id": id,
                            "type": etype,
                            "codes": code_arr,
                            "components": []
                        })
                    })
                    .collect();
                serde_json::json!({"id": tag, "elements": elements})
            })
            .collect();

        fields.insert(
            field_name.to_string(),
            serde_json::json!({
                "source_group": source_group,
                "discriminator": disc_obj,
                "segments": segs,
                "children": null
            }),
        );
    }

    serde_json::json!({
        "pid": "55001",
        "beschreibung": "Test",
        "format_version": "FV2504",
        "fields": fields
    })
}

#[test]
fn test_diff_detects_added_segment_within_group() {
    let old = schema_with_segments(&[(
        "sg5_z16",
        "SG5",
        "LOC:Z16",
        &[("LOC", &[(0, "3227", "code", &["Z16"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg5_z16",
        "SG5",
        "LOC:Z16",
        &[
            ("LOC", &[(0, "3227", "code", &["Z16"])]),
            ("MEA", &[(0, "6311", "code", &["AAA"])]),
        ],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.segments.added.len(), 1);
    assert_eq!(diff.segments.added[0].tag, "MEA");
    assert_eq!(diff.segments.unchanged.len(), 1);
    assert_eq!(diff.segments.unchanged[0].tag, "LOC");
}

#[test]
fn test_diff_detects_code_change() {
    let old = schema_with_segments(&[(
        "sg10",
        "SG10",
        "CCI:Z66",
        &[("CCI", &[(0, "7059", "code", &["Z66", "Z88"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg10",
        "SG10",
        "CCI:Z66",
        &[("CCI", &[(0, "7059", "code", &["Z66", "Z95"])])],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.codes.changed.len(), 1);
    assert_eq!(diff.codes.changed[0].added, vec!["Z95"]);
    assert_eq!(diff.codes.changed[0].removed, vec!["Z88"]);
}

#[test]
fn test_diff_detects_added_element() {
    let old = schema_with_segments(&[(
        "sg4",
        "SG4",
        ":",
        &[("STS", &[(0, "9015", "code", &["7"]), (2, "9013", "code", &["E01"])])],
    )]);
    let new = schema_with_segments(&[(
        "sg4",
        "SG4",
        ":",
        &[(
            "STS",
            &[
                (0, "9015", "code", &["7"]),
                (2, "9013", "code", &["E01"]),
                (4, "9013b", "code", &["E03"]),
            ],
        )],
    )]);

    let input = DiffInput {
        old_schema: old,
        new_schema: new,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert_eq!(diff.elements.added.len(), 1);
    assert_eq!(diff.elements.added[0].index, 4);
    assert_eq!(diff.elements.added[0].segment, "STS");
}
