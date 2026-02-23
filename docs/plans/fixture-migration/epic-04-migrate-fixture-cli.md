---
feature: fixture-migration
epic: 4
title: "migrate-fixture CLI Subcommand"
depends_on: [1, 3]
estimated_tasks: 3
crate: automapper-generator
status: in_progress
---

# Epic 4: `migrate-fixture` CLI Subcommand

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add a `migrate-fixture` subcommand to `automapper-generator` that takes an old `.edi` fixture, a diff JSON from `mig-diff`, and a new PID schema, and produces a migrated `.edi` file plus a `.warnings.txt` file.

**Architecture:** New `MigrateFixture` variant in the `Commands` enum. The handler loads the diff JSON (from Epic 2 output), loads the new PID schema, reads the old `.edi`, calls `migrate_fixture()` from Epic 3, writes the output `.edi` and warnings file.

**Existing code:**
- `Commands` enum at `crates/automapper-generator/src/main.rs` — add new variant
- `migrate_fixture()` from Epic 3 — core migration logic
- `PidSchemaDiff` from Epic 1 — deserialized from diff JSON

---

## Task 1: Add `MigrateFixture` CLI Variant

**Files:**
- Modify: `crates/automapper-generator/src/main.rs`

**Step 1: Add the new subcommand variant**

Add to the `Commands` enum in `main.rs`:
```rust
/// Migrate an EDIFACT fixture file between format versions using a diff report.
MigrateFixture {
    /// Path to the old .edi fixture file.
    #[arg(long)]
    old_fixture: PathBuf,

    /// Path to the diff JSON file (output of mig-diff).
    #[arg(long)]
    diff: PathBuf,

    /// Path to the new PID schema JSON.
    #[arg(long)]
    new_pid_schema: PathBuf,

    /// Output path for the migrated .edi file. If omitted, derives from old fixture name.
    #[arg(long)]
    output: Option<PathBuf>,
},
```

**Step 2: Add the handler in the match block**

Add the match arm in `main()`:
```rust
Commands::MigrateFixture {
    old_fixture,
    diff,
    new_pid_schema,
    output,
} => {
    use crate::fixture_migrator::migrate_fixture;
    use crate::schema_diff::PidSchemaDiff;

    // Load inputs
    let old_edi = std::fs::read_to_string(&old_fixture)?;
    let diff_json: PidSchemaDiff =
        serde_json::from_str(&std::fs::read_to_string(&diff)?)?;
    let new_schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&new_pid_schema)?)?;

    // Run migration
    let result = migrate_fixture(&old_edi, &diff_json, &new_schema);

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let stem = old_fixture.file_stem().unwrap().to_string_lossy();
        let parent = old_fixture.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}_migrated.edi", stem))
    });

    // Write migrated .edi
    std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(&output_path, &result.edifact)?;
    println!("Wrote migrated fixture: {}", output_path.display());

    // Write warnings file
    if !result.warnings.is_empty() {
        let warnings_path = output_path.with_extension("edi.warnings.txt");
        let warnings_text: String = result
            .warnings
            .iter()
            .map(|w| w.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&warnings_path, &warnings_text)?;
        println!("Wrote {} warnings: {}", result.warnings.len(), warnings_path.display());
    }

    // Print summary
    println!("\nMigration summary:");
    println!("  Segments copied:    {}", result.stats.segments_copied);
    println!("  Segments removed:   {}", result.stats.segments_removed);
    println!("  Segments added:     {}", result.stats.segments_added);
    println!("  Codes substituted:  {}", result.stats.codes_substituted);
    println!("  Manual review items: {}", result.stats.manual_review_items);
}
```

**Step 3: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/main.rs
git commit -m "feat(generator): add migrate-fixture CLI subcommand"
```

---

## Task 2: Batch Migration Support

**Files:**
- Create: `crates/automapper-generator/src/fixture_migrator/batch.rs`
- Modify: `crates/automapper-generator/src/fixture_migrator/mod.rs`
- Modify: `crates/automapper-generator/tests/fixture_migrator_test.rs`

**Step 1: Write the failing test**

Add to `fixture_migrator_test.rs`:
```rust
use automapper_generator::fixture_migrator::batch::migrate_directory;

#[test]
fn test_migrate_directory_processes_multiple_fixtures() {
    let tmp_old = tempfile::tempdir().unwrap();
    let tmp_out = tempfile::tempdir().unwrap();

    // Write two synthetic fixture files
    std::fs::write(
        tmp_old.path().join("55001_UTILMD_S2.1_test1.edi"),
        "UNH+M1+UTILMD:D:11A:UN:S2.1'\nBGM+E01+M1'\nUNT+2+M1'",
    ).unwrap();
    std::fs::write(
        tmp_old.path().join("55001_UTILMD_S2.1_test2.edi"),
        "UNH+M2+UTILMD:D:11A:UN:S2.1'\nBGM+E01+M2'\nUNT+2+M2'",
    ).unwrap();
    // Write a non-.edi file that should be skipped
    std::fs::write(
        tmp_old.path().join("55001_UTILMD_S2.1_test1.bo.json"),
        "{}",
    ).unwrap();

    let diff = version_only_diff();
    let schema = serde_json::json!({"pid": "55001", "fields": {}});

    let results = migrate_directory(tmp_old.path(), tmp_out.path(), &diff, &schema);
    assert_eq!(results.len(), 2, "Should process exactly 2 .edi files");
    assert!(results.iter().all(|r| r.is_ok()), "All migrations should succeed");

    // Check output files exist
    let output_files: Vec<_> = std::fs::read_dir(tmp_out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "edi").unwrap_or(false))
        .collect();
    assert_eq!(output_files.len(), 2, "Should write 2 output .edi files");
}
```

**Step 2: Write the implementation**

`crates/automapper-generator/src/fixture_migrator/batch.rs`:
```rust
use super::migrator::migrate_fixture;
use super::types::MigrationResult;
use crate::schema_diff::PidSchemaDiff;
use std::path::Path;

/// Migrate all `.edi` files in a directory.
///
/// Returns a Vec of results (one per file).
/// Each result contains the filename and either a MigrationResult or an error.
pub fn migrate_directory(
    input_dir: &Path,
    output_dir: &Path,
    diff: &PidSchemaDiff,
    new_schema: &serde_json::Value,
) -> Vec<Result<(String, MigrationResult), String>> {
    let mut results = Vec::new();

    let entries: Vec<_> = match std::fs::read_dir(input_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            results.push(Err(format!("Failed to read directory: {}", e)));
            return results;
        }
    };

    std::fs::create_dir_all(output_dir).ok();

    for entry in entries {
        let path = entry.path();
        if path.extension().map(|e| e == "edi").unwrap_or(false) {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();

            match std::fs::read_to_string(&path) {
                Ok(old_edi) => {
                    let result = migrate_fixture(&old_edi, diff, new_schema);

                    // Write output
                    let output_path = output_dir.join(&filename);
                    if let Err(e) = std::fs::write(&output_path, &result.edifact) {
                        results.push(Err(format!("Failed to write {}: {}", filename, e)));
                        continue;
                    }

                    // Write warnings if any
                    if !result.warnings.is_empty() {
                        let warnings_path = output_path.with_extension("edi.warnings.txt");
                        let warnings_text: String = result
                            .warnings
                            .iter()
                            .map(|w| w.to_string())
                            .collect::<Vec<_>>()
                            .join("\n");
                        std::fs::write(&warnings_path, &warnings_text).ok();
                    }

                    results.push(Ok((filename, result)));
                }
                Err(e) => {
                    results.push(Err(format!("Failed to read {}: {}", filename, e)));
                }
            }
        }
    }

    results
}
```

Update `crates/automapper-generator/src/fixture_migrator/mod.rs`:
```rust
pub mod types;
pub mod migrator;
pub mod skeleton;
pub mod batch;

pub use types::*;
pub use migrator::*;
pub use skeleton::*;
```

**Step 3: Run tests**

Run: `cargo test -p automapper-generator test_migrate_directory`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/fixture_migrator/batch.rs crates/automapper-generator/src/fixture_migrator/mod.rs crates/automapper-generator/tests/fixture_migrator_test.rs
git commit -m "feat(generator): add batch fixture migration for directories"
```

---

## Task 3: Add Batch Mode to CLI and Final Integration Test

**Files:**
- Modify: `crates/automapper-generator/src/main.rs`
- Modify: `crates/automapper-generator/tests/fixture_migrator_test.rs`

**Step 1: Add batch variant to CLI**

Add to the `Commands` enum in `main.rs`:
```rust
/// Migrate all .edi fixtures in a directory between format versions.
MigrateFixtureDir {
    /// Path to the directory containing old .edi fixture files.
    #[arg(long)]
    input_dir: PathBuf,

    /// Path to the diff JSON file (output of mig-diff).
    #[arg(long)]
    diff: PathBuf,

    /// Path to the new PID schema JSON.
    #[arg(long)]
    new_pid_schema: PathBuf,

    /// Output directory for migrated .edi files.
    #[arg(long)]
    output_dir: PathBuf,
},
```

Add the handler:
```rust
Commands::MigrateFixtureDir {
    input_dir,
    diff,
    new_pid_schema,
    output_dir,
} => {
    use crate::fixture_migrator::batch::migrate_directory;
    use crate::schema_diff::PidSchemaDiff;

    let diff_json: PidSchemaDiff =
        serde_json::from_str(&std::fs::read_to_string(&diff)?)?;
    let new_schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&new_pid_schema)?)?;

    let results = migrate_directory(&input_dir, &output_dir, &diff_json, &new_schema);

    let mut success = 0;
    let mut failed = 0;
    let mut total_warnings = 0;

    for result in &results {
        match result {
            Ok((filename, mr)) => {
                success += 1;
                total_warnings += mr.warnings.len();
                if !mr.warnings.is_empty() {
                    println!("  {} — {} warnings", filename, mr.warnings.len());
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("  FAILED: {}", e);
            }
        }
    }

    println!("\nBatch migration complete:");
    println!("  {} succeeded, {} failed, {} total warnings",
        success, failed, total_warnings);
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: PASS

**Step 3: Run full test suite and lint**

Run: `cargo test -p automapper-generator && cargo clippy -p automapper-generator -- -D warnings`
Expected: All PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/main.rs
git commit -m "feat(generator): add migrate-fixture-dir CLI for batch migration"
```
