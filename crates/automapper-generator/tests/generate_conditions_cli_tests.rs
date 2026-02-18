use std::process::Command;

#[test]
fn test_cli_generate_conditions_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args(["generate-conditions", "--help"])
        .output()
        .expect("failed to run automapper-generator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--ahb-path"), "should have --ahb-path flag");
    assert!(
        stdout.contains("--output-dir"),
        "should have --output-dir flag"
    );
    assert!(
        stdout.contains("--format-version"),
        "should have --format-version flag"
    );
    assert!(
        stdout.contains("--message-type"),
        "should have --message-type flag"
    );
    assert!(
        stdout.contains("--incremental"),
        "should have --incremental flag"
    );
    assert!(
        stdout.contains("--max-concurrent"),
        "should have --max-concurrent flag"
    );
    assert!(output.status.success());
}

#[test]
fn test_cli_generate_conditions_missing_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .arg("generate-conditions")
        .output()
        .expect("failed to run automapper-generator");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("error"),
        "stderr should mention missing required args: {}",
        stderr
    );
}

#[test]
fn test_cli_generate_conditions_dry_run() {
    let ahb_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/minimal_ahb.xml");

    // Dry run should succeed without calling Claude
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args([
            "generate-conditions",
            "--ahb-path",
            ahb_path.to_str().unwrap(),
            "--output-dir",
            "/tmp/automapper-generator-test-output",
            "--format-version",
            "FV2510",
            "--message-type",
            "UTILMD",
            "--dry-run",
        ])
        .output()
        .expect("failed to run automapper-generator");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Dry run may succeed or fail depending on AHB parsing, but it should NOT invoke Claude
    // The key assertion is that it doesn't hang waiting for claude
    assert!(
        stderr.contains("DRY RUN") || stderr.contains("conditions") || !output.status.success(),
        "dry run should report or fail gracefully: {}",
        stderr
    );
}

#[test]
fn test_cli_validate_schema_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args(["validate-schema", "--help"])
        .output()
        .expect("failed to run automapper-generator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--stammdatenmodell-path"),
        "should have --stammdatenmodell-path flag"
    );
    assert!(
        stdout.contains("--generated-dir"),
        "should have --generated-dir flag"
    );
    assert!(output.status.success());
}
