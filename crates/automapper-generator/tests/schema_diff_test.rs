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
