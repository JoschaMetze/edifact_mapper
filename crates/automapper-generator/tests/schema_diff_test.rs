//! Tests for PID schema diffing.

use std::path::Path;

use automapper_generator::schema_diff::{
    diff_pid_schemas, render_diff_markdown, DiffInput, PidSchemaDiff,
};

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
    let schema =
        minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16"), ("sg8_z98", "SG8", "SEQ:Z98")]);

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
    let new = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16"), ("sg8_zh5", "SG8", "SEQ:ZH5")]);

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
    let old = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16"), ("sg8_z98", "SG8", "SEQ:Z98")]);
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

#[allow(clippy::type_complexity)]
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
        &[(
            "STS",
            &[(0, "9015", "code", &["7"]), (2, "9013", "code", &["E01"])],
        )],
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

#[test]
fn test_diff_real_55001_against_itself() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");

    if !schema_path.exists() {
        eprintln!("Skipping: schema not found at {:?}", schema_path);
        return;
    }

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();

    let input = DiffInput {
        old_schema: schema.clone(),
        new_schema: schema,
        old_version: "FV2504".into(),
        new_version: "FV2504".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert!(
        diff.is_empty(),
        "Diffing a schema against itself should produce no changes, got: {} added groups, {} removed groups, {} code changes, {} added segments, {} removed segments",
        diff.groups.added.len(),
        diff.groups.removed.len(),
        diff.codes.changed.len(),
        diff.segments.added.len(),
        diff.segments.removed.len(),
    );
}

#[test]
fn test_diff_55001_vs_55002_shows_differences() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("mig-types/src/generated/fv2504/utilmd/pids");

    let schema_55001_path = base.join("pid_55001_schema.json");
    let schema_55002_path = base.join("pid_55002_schema.json");

    if !schema_55001_path.exists() || !schema_55002_path.exists() {
        eprintln!("Skipping: schemas not found");
        return;
    }

    let schema_55001: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_55001_path).unwrap()).unwrap();
    let schema_55002: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_55002_path).unwrap()).unwrap();

    let input = DiffInput {
        old_schema: schema_55001,
        new_schema: schema_55002,
        old_version: "FV2504".into(),
        new_version: "FV2504".into(),
        message_type: "UTILMD".into(),
        pid: "55001-vs-55002".into(),
    };

    let diff = diff_pid_schemas(&input);
    assert!(
        !diff.is_empty(),
        "55001 and 55002 should have structural differences"
    );

    // 55002 has more LOC groups (Z17, Z18, Z19, Z20) that 55001 doesn't
    assert!(
        !diff.groups.added.is_empty(),
        "55002 should have groups not in 55001"
    );

    // Print diff summary for manual inspection
    eprintln!(
        "Groups added: {:?}",
        diff.groups
            .added
            .iter()
            .map(|g| &g.group)
            .collect::<Vec<_>>()
    );
    eprintln!(
        "Groups removed: {:?}",
        diff.groups
            .removed
            .iter()
            .map(|g| &g.group)
            .collect::<Vec<_>>()
    );
    eprintln!("Code changes: {}", diff.codes.changed.len());
}

#[test]
fn test_diff_serializes_to_json() {
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
    let json = serde_json::to_string_pretty(&diff).unwrap();
    assert!(json.contains("\"old_version\": \"FV2504\""));
    assert!(json.contains("\"new_version\": \"FV2510\""));
    assert!(json.contains("\"tag\": \"MEA\""));

    // Verify round-trip: deserialize back
    let parsed: PidSchemaDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.segments.added.len(), 1);
}

#[test]
fn test_render_diff_markdown_includes_sections() {
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
            ("LOC", &[(0, "3227", "code", &["Z16", "Z17"])]),
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
    let md = render_diff_markdown(&diff);

    assert!(md.contains("# PID Schema Diff"));
    assert!(md.contains("FV2504"));
    assert!(md.contains("FV2510"));
    assert!(md.contains("MEA")); // added segment
    assert!(md.contains("Z17")); // added code
}

#[test]
fn test_render_diff_markdown_empty_diff() {
    let schema = minimal_schema_json(&[("sg5_z16", "SG5", "LOC:Z16")]);

    let input = DiffInput {
        old_schema: schema.clone(),
        new_schema: schema,
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
    };

    let diff = diff_pid_schemas(&input);
    let md = render_diff_markdown(&diff);

    assert!(md.contains("# PID Schema Diff"));
    assert!(md.contains("No differences found."));
}

#[test]
fn test_diff_real_schemas_produces_valid_json_and_markdown() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("mig-types/src/generated/fv2504/utilmd/pids");

    let schema_55001 = base.join("pid_55001_schema.json");
    let schema_55002 = base.join("pid_55002_schema.json");

    if !schema_55001.exists() || !schema_55002.exists() {
        eprintln!("Skipping: schemas not found");
        return;
    }

    let old_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_55001).unwrap()).unwrap();
    let new_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_55002).unwrap()).unwrap();

    let input = DiffInput {
        old_schema: old_json,
        new_schema: new_json,
        old_version: "FV2504".into(),
        new_version: "FV2504".into(),
        message_type: "UTILMD".into(),
        pid: "55001-vs-55002".into(),
    };

    let diff = diff_pid_schemas(&input);

    // Verify JSON serialization
    let json = serde_json::to_string_pretty(&diff).unwrap();
    assert!(json.len() > 50, "JSON should have meaningful content");

    // Verify markdown rendering
    let md = render_diff_markdown(&diff);
    assert!(md.contains("# PID Schema Diff"));
    assert!(md.contains("55001-vs-55002"));

    // Write to temp dir for manual inspection
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("diff.json"), &json).unwrap();
    std::fs::write(tmp.path().join("diff.md"), &md).unwrap();
    eprintln!("Diff output written to: {:?}", tmp.path());
    eprintln!("Markdown preview:\n{}", &md[..md.len().min(500)]);
}
