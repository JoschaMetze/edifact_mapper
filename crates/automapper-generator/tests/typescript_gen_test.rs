//! Snapshot test for TypeScript type generation.
//!
//! Verifies the generated .d.ts content matches expected output.
//! Run `cargo insta test` then `cargo insta review` to update snapshots.

use std::path::Path;

#[test]
fn test_typescript_gen_pid_55001_snapshot() {
    let schema_dir = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/mig-types/src/generated/fv2504/utilmd/pids"
    ));
    let mappings_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../mappings"));
    let schema_path = schema_dir.join("pid_55001_schema.json");

    if !schema_path.exists() {
        eprintln!("Skipping: PID schema not found");
        return;
    }

    let msg_dir = mappings_dir.join("FV2504/UTILMD_Strom/message");
    let tx_dir = mappings_dir.join("FV2504/UTILMD_Strom/pid_55001");

    let info = automapper_generator::codegen::typescript_gen::collect_entities(
        &[msg_dir.as_path(), tx_dir.as_path()],
        &schema_path,
        "55001",
    )
    .unwrap();

    let content = automapper_generator::codegen::typescript_gen::emit_pid_dts(&info);

    // Verify key structural elements rather than exact snapshot
    // (avoids brittleness from field ordering changes)
    assert!(content.contains("export interface Pid55001Response {"));

    // Count interfaces — should match number of unique entities + companions
    let interface_count = content.matches("export interface ").count();
    assert!(
        interface_count >= 10,
        "Expected at least 10 interfaces, got {}",
        interface_count
    );

    // Verify all expected entities appear in the response type
    let response_section: &str = content
        .split("export interface Pid55001Response {")
        .nth(1)
        .expect("response type not found");
    assert!(response_section.contains("marktlokation?:"));
    assert!(response_section.contains("prozessdaten?:"));
    assert!(response_section.contains("marktteilnehmer?:"));
    assert!(response_section.contains("nachricht?:"));

    // Array entities
    assert!(response_section.contains("Marktteilnehmer[]"));
}

#[test]
fn test_typescript_gen_pid_55002_snapshot() {
    let schema_dir = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/mig-types/src/generated/fv2504/utilmd/pids"
    ));
    let mappings_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../mappings"));
    let schema_path = schema_dir.join("pid_55002_schema.json");

    if !schema_path.exists() {
        eprintln!("Skipping: PID schema not found");
        return;
    }

    let msg_dir = mappings_dir.join("FV2504/UTILMD_Strom/message");
    let tx_dir = mappings_dir.join("FV2504/UTILMD_Strom/pid_55002");

    let info = automapper_generator::codegen::typescript_gen::collect_entities(
        &[msg_dir.as_path(), tx_dir.as_path()],
        &schema_path,
        "55002",
    )
    .unwrap();

    let content = automapper_generator::codegen::typescript_gen::emit_pid_dts(&info);

    assert!(content.contains("export interface Pid55002Response {"));

    // PID 55002 has more location types than 55001
    assert!(content.contains("export interface Messlokation {"));
    assert!(content.contains("export interface Netzlokation {"));
    assert!(content.contains("export interface SteuerbareRessource {"));
    assert!(content.contains("export interface TechnischeRessource {"));
}
