//! Tests for the fixture migrator engine.

use automapper_generator::fixture_migrator::{
    generate_skeleton_segment, migrate_fixture, WarningSeverity,
};
use automapper_generator::schema_diff::types::*;

/// Build a minimal PidSchemaDiff with only a UNH version change.
fn version_only_diff() -> PidSchemaDiff {
    PidSchemaDiff {
        old_version: "FV2504".into(),
        new_version: "FV2510".into(),
        message_type: "UTILMD".into(),
        pid: "55001".into(),
        unh_version: Some(VersionChange {
            old: "S2.1".into(),
            new: "S2.2".into(),
        }),
        segments: SegmentDiff {
            added: vec![],
            removed: vec![],
            unchanged: vec![],
        },
        codes: CodeDiff { changed: vec![] },
        groups: GroupDiff {
            added: vec![],
            removed: vec![],
            restructured: vec![],
        },
        elements: ElementDiff {
            added: vec![],
            removed: vec![],
        },
    }
}

#[test]
fn test_migrate_updates_unh_version() {
    let old_edi = "\
UNB+UNOC:3+9978842000002:500+9900269000000:500+250331:1329+REF123'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E01+MSG001BGM'\
UNT+3+MSG001'\
UNZ+1+REF123'";

    let diff = version_only_diff();
    let new_schema = serde_json::json!({
        "pid": "55001",
        "beschreibung": "Test",
        "format_version": "FV2510",
        "fields": {}
    });

    let result = migrate_fixture(old_edi, &diff, &new_schema);
    assert!(
        result.edifact.contains("UTILMD:D:11A:UN:S2.2"),
        "UNH version should be updated to S2.2, got: {}",
        result.edifact
    );
    assert!(
        result.edifact.contains("UNB+UNOC:3"),
        "UNB should be preserved unchanged"
    );
    assert!(
        result.edifact.contains("BGM+E01"),
        "BGM should be preserved unchanged"
    );
}

#[test]
fn test_migrate_removes_dropped_segments() {
    let old_edi = "\
UNB+UNOC:3+SENDER:500+RECEIVER:500+250101:0900+REF1'\
UNH+M1+UTILMD:D:11A:UN:S2.1'\
BGM+E01+M1BGM'\
IMD++Z36+Z13'\
UNT+4+M1'\
UNZ+1+REF1'";

    let mut diff = version_only_diff();
    diff.segments.removed.push(SegmentEntry {
        group: "sg4".into(),
        tag: "IMD".into(),
        context: Some("Removed in S2.2".into()),
    });

    let new_schema = serde_json::json!({
        "pid": "55001",
        "fields": {}
    });

    let result = migrate_fixture(old_edi, &diff, &new_schema);
    assert!(
        !result.edifact.contains("IMD"),
        "IMD should be removed, got: {}",
        result.edifact
    );
    assert!(
        result.edifact.contains("BGM+E01"),
        "BGM should be preserved"
    );
    assert_eq!(result.stats.segments_removed, 1);
}

#[test]
fn test_migrate_substitutes_renamed_code() {
    let old_edi = "\
UNH+M1+UTILMD:D:11A:UN:S2.1'\
CCI+Z88'\
UNT+2+M1'";

    let mut diff = version_only_diff();
    diff.codes.changed.push(CodeChange {
        segment: "CCI".into(),
        element: "0".into(),
        group: "sg10".into(),
        added: vec!["Z95".into()],
        removed: vec!["Z88".into()],
        context: None,
    });

    let new_schema = serde_json::json!({"pid": "55001", "fields": {}});
    let result = migrate_fixture(old_edi, &diff, &new_schema);

    assert!(
        result.edifact.contains("CCI+Z95"),
        "Z88 should be renamed to Z95, got: {}",
        result.edifact
    );
    assert!(
        !result.edifact.contains("Z88"),
        "Old code Z88 should not appear"
    );
    assert_eq!(result.stats.codes_substituted, 1);
}

#[test]
fn test_migrate_warns_on_restructured_groups() {
    let old_edi = "UNH+M1+UTILMD:D:11A:UN:S2.1'\nUNT+1+M1'";

    let mut diff = version_only_diff();
    diff.groups.restructured.push(RestructuredGroup {
        group: "sg10".into(),
        description: "Moved from SG8 to SG5".into(),
        manual_review: true,
    });

    let new_schema = serde_json::json!({"pid": "55001", "fields": {}});
    let result = migrate_fixture(old_edi, &diff, &new_schema);

    assert_eq!(result.stats.manual_review_items, 1);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.severity == WarningSeverity::Error));
    assert!(result.warnings.iter().any(|w| w.message.contains("sg10")));
}

#[test]
fn test_migrate_warns_on_new_groups() {
    let old_edi = "UNH+M1+UTILMD:D:11A:UN:S2.1'\nUNT+1+M1'";

    let mut diff = version_only_diff();
    diff.groups.added.push(GroupEntry {
        group: "sg8_zh5".into(),
        parent: "sg4".into(),
        entry_segment: Some("SEQ+ZH5".into()),
    });

    let new_schema = serde_json::json!({"pid": "55001", "fields": {}});
    let result = migrate_fixture(old_edi, &diff, &new_schema);

    assert!(result
        .warnings
        .iter()
        .any(|w| w.severity == WarningSeverity::Warning && w.message.contains("sg8_zh5")));
}

#[test]
fn test_generate_skeleton_segment_from_schema() {
    // A schema segment definition with one code element and one data element
    let segment_schema = serde_json::json!({
        "id": "MEA",
        "elements": [
            {
                "index": 0,
                "id": "6311",
                "type": "code",
                "codes": [{"value": "AAA", "name": "Test"}],
                "components": []
            },
            {
                "index": 1,
                "id": "6314",
                "type": "data",
                "codes": [],
                "components": []
            }
        ]
    });

    let skeleton = generate_skeleton_segment(&segment_schema);
    assert_eq!(
        skeleton, "MEA+AAA",
        "Should use first valid code for code elements, omit trailing empty data elements"
    );
}

#[test]
fn test_generate_skeleton_with_composite() {
    let segment_schema = serde_json::json!({
        "id": "LOC",
        "elements": [
            {
                "index": 0,
                "id": "3227",
                "type": "code",
                "codes": [{"value": "Z16", "name": "Marktlokation"}],
                "components": []
            },
            {
                "index": 1,
                "id": "C517",
                "type": "data",
                "composite": "C517",
                "codes": [],
                "components": [
                    {"sub_index": 0, "id": "3225", "type": "data", "codes": []},
                    {"sub_index": 1, "id": "1131", "type": "data", "codes": []}
                ]
            }
        ]
    });

    let skeleton = generate_skeleton_segment(&segment_schema);
    // Code element filled, composite left empty (data elements)
    assert_eq!(
        skeleton, "LOC+Z16",
        "Should fill code, omit trailing empty composites"
    );
}

use automapper_generator::fixture_migrator::batch::migrate_directory;
use std::path::Path;

#[test]
fn test_migrate_directory_processes_multiple_fixtures() {
    let tmp_old = tempfile::tempdir().unwrap();
    let tmp_out = tempfile::tempdir().unwrap();

    // Write two synthetic fixture files
    std::fs::write(
        tmp_old.path().join("55001_UTILMD_S2.1_test1.edi"),
        "UNH+M1+UTILMD:D:11A:UN:S2.1'\nBGM+E01+M1'\nUNT+2+M1'",
    )
    .unwrap();
    std::fs::write(
        tmp_old.path().join("55001_UTILMD_S2.1_test2.edi"),
        "UNH+M2+UTILMD:D:11A:UN:S2.1'\nBGM+E01+M2'\nUNT+2+M2'",
    )
    .unwrap();
    // Write a non-.edi file that should be skipped
    std::fs::write(tmp_old.path().join("55001_UTILMD_S2.1_test1.bo.json"), "{}").unwrap();

    let diff = version_only_diff();
    let schema = serde_json::json!({"pid": "55001", "fields": {}});

    let results = migrate_directory(tmp_old.path(), tmp_out.path(), &diff, &schema);
    assert_eq!(results.len(), 2, "Should process exactly 2 .edi files");
    assert!(
        results.iter().all(|r| r.is_ok()),
        "All migrations should succeed"
    );

    // Check output files exist
    let output_files: Vec<_> = std::fs::read_dir(tmp_out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "edi")
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(output_files.len(), 2, "Should write 2 output .edi files");
}

#[test]
fn test_migrate_directory_writes_warnings_file() {
    let tmp_old = tempfile::tempdir().unwrap();
    let tmp_out = tempfile::tempdir().unwrap();

    std::fs::write(
        tmp_old.path().join("55001_test.edi"),
        "UNH+M1+UTILMD:D:11A:UN:S2.1'\nUNT+1+M1'",
    )
    .unwrap();

    let mut diff = version_only_diff();
    diff.groups.restructured.push(RestructuredGroup {
        group: "sg10".into(),
        description: "Moved".into(),
        manual_review: true,
    });

    let schema = serde_json::json!({"pid": "55001", "fields": {}});

    let results = migrate_directory(tmp_old.path(), tmp_out.path(), &diff, &schema);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());

    // Check warnings file was written
    let warnings_path = tmp_out.path().join("55001_test.edi.warnings.txt");
    assert!(
        warnings_path.exists(),
        "Warnings file should be written when there are warnings"
    );
    let warnings_content = std::fs::read_to_string(&warnings_path).unwrap();
    assert!(warnings_content.contains("sg10"));
}

#[test]
fn test_migrate_real_55001_fixture_with_synthetic_diff() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let fixture_path = base.join(
        "example_market_communication_bo4e_transactions/UTILMD/FV2504/55001_UTILMD_S2.1_ALEXANDE121980.edi",
    );
    let schema_path =
        base.join("crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json");

    if !fixture_path.exists() || !schema_path.exists() {
        eprintln!("Skipping: fixture or schema not found");
        return;
    }

    let old_edi = std::fs::read_to_string(&fixture_path).unwrap();
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();

    // Create a synthetic diff: just a version bump, no structural changes
    let mut diff = version_only_diff();
    diff.unh_version = Some(VersionChange {
        old: "S2.1".into(),
        new: "S2.2".into(),
    });

    let result = migrate_fixture(&old_edi, &diff, &schema);

    // Verify version was updated
    assert!(
        result.edifact.contains("S2.2"),
        "Should contain updated version"
    );
    assert!(
        !result.edifact.contains("S2.1"),
        "Should not contain old version"
    );

    // Verify all non-UNH segments are preserved
    assert!(result.edifact.contains("UNB+UNOC:3"));
    assert!(result.edifact.contains("BGM+E01"));
    assert!(result.edifact.contains("LOC+Z16"));
    assert!(result.edifact.contains("RFF+Z13:55001"));
    assert!(result.edifact.contains("UNT+"));
    assert!(result.edifact.contains("UNZ+"));

    // No warnings for a simple version bump
    let error_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.severity == WarningSeverity::Error)
        .collect();
    assert!(
        error_warnings.is_empty(),
        "Version-only diff should produce no error warnings, got: {:?}",
        error_warnings
    );

    eprintln!(
        "Migration stats: copied={}, removed={}, added={}, subs={}",
        result.stats.segments_copied,
        result.stats.segments_removed,
        result.stats.segments_added,
        result.stats.codes_substituted,
    );
}
