use std::process::Command;

/// Test that the CLI binary exists and shows help.
#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .arg("--help")
        .output()
        .expect("failed to run automapper-generator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("automapper-generator"));
    assert!(stdout.contains("generate-mappers"));
    assert!(output.status.success());
}

#[test]
fn test_cli_generate_mappers_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args(["generate-mappers", "--help"])
        .output()
        .expect("failed to run automapper-generator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--mig-path"));
    assert!(stdout.contains("--ahb-path"));
    assert!(stdout.contains("--output-dir"));
    assert!(stdout.contains("--format-version"));
    assert!(stdout.contains("--message-type"));
    assert!(output.status.success());
}

#[test]
fn test_cli_missing_required_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .arg("generate-mappers")
        .output()
        .expect("failed to run automapper-generator");

    // Should fail with missing required arguments
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("error"),
        "stderr should mention missing required args: {}",
        stderr
    );
}
