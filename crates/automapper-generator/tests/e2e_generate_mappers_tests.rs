use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_e2e_generate_mappers() {
    let output_dir = TempDir::new().unwrap();

    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args([
            "generate-mappers",
            "--mig-path",
            mig_path.to_str().unwrap(),
            "--ahb-path",
            ahb_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
            "--format-version",
            "FV2510",
            "--message-type",
            "UTILMD",
        ])
        .output()
        .expect("failed to run automapper-generator");

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stderr: {}", stderr);

    assert!(
        output.status.success(),
        "generate-mappers should succeed, stderr: {}",
        stderr
    );

    // Verify output files were created
    let files: Vec<_> = std::fs::read_dir(output_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    eprintln!("Generated files: {:?}", files);

    // Should have mapper stubs
    assert!(
        files.iter().any(|f| f.contains("mapper")),
        "should generate mapper files"
    );

    // Should have version config
    assert!(
        files.iter().any(|f| f.contains("version_config")),
        "should generate version config"
    );

    // Should have coordinator
    assert!(
        files.iter().any(|f| f.contains("coordinator")),
        "should generate coordinator"
    );

    // Verify all generated files are non-empty and contain valid Rust-like content
    for entry in std::fs::read_dir(output_dir.path()).unwrap() {
        let entry = entry.unwrap();
        let content = std::fs::read_to_string(entry.path()).unwrap();
        assert!(
            !content.is_empty(),
            "generated file should not be empty: {:?}",
            entry.path()
        );
        assert!(
            content.contains("auto-generated") || content.contains("pub mod"),
            "generated file should have expected content: {:?}",
            entry.path()
        );
    }
}

#[test]
fn test_e2e_generate_mappers_missing_mig() {
    let output_dir = TempDir::new().unwrap();
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args([
            "generate-mappers",
            "--mig-path",
            "/nonexistent/mig.xml",
            "--ahb-path",
            ahb_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
            "--format-version",
            "FV2510",
            "--message-type",
            "UTILMD",
        ])
        .output()
        .expect("failed to run automapper-generator");

    assert!(
        !output.status.success(),
        "should fail for nonexistent MIG file"
    );
}

#[test]
fn test_e2e_output_dir_created() {
    let parent = TempDir::new().unwrap();
    let output_dir = parent.path().join("nested").join("output");

    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args([
            "generate-mappers",
            "--mig-path",
            mig_path.to_str().unwrap(),
            "--ahb-path",
            ahb_path.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--format-version",
            "FV2510",
            "--message-type",
            "UTILMD",
        ])
        .output()
        .expect("failed to run automapper-generator");

    assert!(
        output.status.success(),
        "should succeed and create nested output dir"
    );
    assert!(output_dir.exists(), "output dir should be created");
}
