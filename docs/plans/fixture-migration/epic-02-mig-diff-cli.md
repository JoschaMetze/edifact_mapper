---
feature: fixture-migration
epic: 2
title: "mig-diff CLI Subcommand"
depends_on: [1]
estimated_tasks: 3
crate: automapper-generator
status: in_progress
---

# Epic 2: `mig-diff` CLI Subcommand

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Add a `mig-diff` subcommand to `automapper-generator` that reads two PID schema JSON files and produces a structured diff JSON file, plus an optional human-readable markdown report.

**Architecture:** New `MigDiff` variant in the `Commands` enum. The handler loads both schema JSONs, calls `diff_pid_schemas()` from Epic 1, writes the diff JSON to the output path, and optionally generates a markdown summary. UNH version metadata is extracted from the schema JSONs' format_version fields.

**Existing code:**
- `Commands` enum at `crates/automapper-generator/src/main.rs:13` — add new variant
- `diff_pid_schemas()` from Epic 1 — core diffing logic
- Pattern: all existing subcommands follow the same `match command { ... }` pattern in `main()`

---

## Task 1: Add `MigDiff` CLI Variant

**Files:**
- Modify: `crates/automapper-generator/src/main.rs`

**Step 1: Add the new subcommand variant**

Add to the `Commands` enum in `main.rs`:
```rust
/// Compare two PID schemas and produce a structured diff report.
MigDiff {
    /// Path to the old PID schema JSON (e.g., pid_55001_schema.json from FV2504).
    #[arg(long)]
    old_schema: PathBuf,

    /// Path to the new PID schema JSON (e.g., pid_55001_schema.json from FV2510).
    #[arg(long)]
    new_schema: PathBuf,

    /// Old format version identifier (e.g., "FV2504").
    #[arg(long)]
    old_version: String,

    /// New format version identifier (e.g., "FV2510").
    #[arg(long)]
    new_version: String,

    /// Message type (e.g., "UTILMD").
    #[arg(long)]
    message_type: String,

    /// PID identifier (e.g., "55001").
    #[arg(long)]
    pid: String,

    /// Output directory for diff files. Creates <pid>_diff.json and <pid>_diff.md.
    #[arg(long)]
    output_dir: PathBuf,
},
```

**Step 2: Add the handler in the match block**

Add the match arm in `main()`:
```rust
Commands::MigDiff {
    old_schema,
    new_schema,
    old_version,
    new_version,
    message_type,
    pid,
    output_dir,
} => {
    use crate::schema_diff::{diff_pid_schemas, DiffInput};

    let old_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&old_schema)?)?;
    let new_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&new_schema)?)?;

    let input = DiffInput {
        old_schema: old_json,
        new_schema: new_json,
        old_version: old_version.clone(),
        new_version: new_version.clone(),
        message_type,
        pid: pid.clone(),
    };

    let diff = diff_pid_schemas(&input);

    std::fs::create_dir_all(&output_dir)?;

    // Write JSON diff
    let json_path = output_dir.join(format!("pid_{}_diff.json", pid));
    let json = serde_json::to_string_pretty(&diff)?;
    std::fs::write(&json_path, &json)?;
    println!("Wrote diff JSON: {}", json_path.display());

    // Write markdown summary
    let md_path = output_dir.join(format!("pid_{}_diff.md", pid));
    let md = crate::schema_diff::render_diff_markdown(&diff);
    std::fs::write(&md_path, &md)?;
    println!("Wrote diff summary: {}", md_path.display());

    // Print summary
    println!(
        "\nDiff summary ({} → {}, PID {}):",
        old_version, new_version, pid
    );
    println!("  Groups:   +{} -{} ~{}",
        diff.groups.added.len(),
        diff.groups.removed.len(),
        diff.groups.restructured.len(),
    );
    println!("  Segments: +{} -{}",
        diff.segments.added.len(),
        diff.segments.removed.len(),
    );
    println!("  Codes:    {} changes", diff.codes.changed.len());
    println!("  Elements: +{} -{}",
        diff.elements.added.len(),
        diff.elements.removed.len(),
    );

    if diff.is_empty() {
        println!("\nNo differences found.");
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: FAIL — `render_diff_markdown` doesn't exist yet. That's OK, we'll add it in the next task.

---

## Task 2: Markdown Report Renderer

**Files:**
- Create: `crates/automapper-generator/src/schema_diff/markdown.rs`
- Modify: `crates/automapper-generator/src/schema_diff/mod.rs`

**Step 1: Write the failing test**

Add to `crates/automapper-generator/tests/schema_diff_test.rs`:
```rust
use automapper_generator::schema_diff::render_diff_markdown;

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
    assert!(md.contains("MEA"));        // added segment
    assert!(md.contains("Z17"));        // added code
}
```

**Step 2: Write the implementation**

`crates/automapper-generator/src/schema_diff/markdown.rs`:
```rust
use super::types::PidSchemaDiff;

/// Render a PidSchemaDiff as a human-readable markdown report.
pub fn render_diff_markdown(diff: &PidSchemaDiff) -> String {
    let mut md = String::new();

    md.push_str(&format!(
        "# PID Schema Diff: {} ({} → {})\n\n",
        diff.pid, diff.old_version, diff.new_version
    ));
    md.push_str(&format!(
        "**Message type:** {}  \n",
        diff.message_type
    ));
    if let Some(ref v) = diff.unh_version {
        md.push_str(&format!(
            "**UNH version:** {} → {}  \n",
            v.old, v.new
        ));
    }
    md.push_str("\n---\n\n");

    // Groups
    if !diff.groups.added.is_empty()
        || !diff.groups.removed.is_empty()
        || !diff.groups.restructured.is_empty()
    {
        md.push_str("## Group Changes\n\n");

        if !diff.groups.added.is_empty() {
            md.push_str("### Added Groups\n\n");
            md.push_str("| Group | Parent | Entry Segment |\n");
            md.push_str("|-------|--------|---------------|\n");
            for g in &diff.groups.added {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    g.group,
                    g.parent,
                    g.entry_segment.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }

        if !diff.groups.removed.is_empty() {
            md.push_str("### Removed Groups\n\n");
            md.push_str("| Group | Parent |\n");
            md.push_str("|-------|--------|\n");
            for g in &diff.groups.removed {
                md.push_str(&format!("| {} | {} |\n", g.group, g.parent));
            }
            md.push('\n');
        }

        if !diff.groups.restructured.is_empty() {
            md.push_str("### Restructured Groups (Manual Review Required)\n\n");
            for g in &diff.groups.restructured {
                md.push_str(&format!("- **{}**: {}\n", g.group, g.description));
            }
            md.push('\n');
        }
    }

    // Segments
    if !diff.segments.added.is_empty() || !diff.segments.removed.is_empty() {
        md.push_str("## Segment Changes\n\n");

        if !diff.segments.added.is_empty() {
            md.push_str("### Added Segments\n\n");
            md.push_str("| Segment | Group | Context |\n");
            md.push_str("|---------|-------|---------|\n");
            for s in &diff.segments.added {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    s.tag,
                    s.group,
                    s.context.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }

        if !diff.segments.removed.is_empty() {
            md.push_str("### Removed Segments\n\n");
            md.push_str("| Segment | Group | Context |\n");
            md.push_str("|---------|-------|---------|\n");
            for s in &diff.segments.removed {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    s.tag,
                    s.group,
                    s.context.as_deref().unwrap_or("-")
                ));
            }
            md.push('\n');
        }
    }

    // Codes
    if !diff.codes.changed.is_empty() {
        md.push_str("## Code Changes\n\n");
        md.push_str("| Segment | Element | Group | Added | Removed |\n");
        md.push_str("|---------|---------|-------|-------|--------|\n");
        for c in &diff.codes.changed {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                c.segment,
                c.element,
                c.group,
                c.added.join(", "),
                c.removed.join(", "),
            ));
        }
        md.push('\n');
    }

    // Elements
    if !diff.elements.added.is_empty() || !diff.elements.removed.is_empty() {
        md.push_str("## Element Changes\n\n");

        if !diff.elements.added.is_empty() {
            md.push_str("### Added Elements\n\n");
            md.push_str("| Segment | Group | Index | Sub-Index | Description |\n");
            md.push_str("|---------|-------|-------|-----------|-------------|\n");
            for e in &diff.elements.added {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    e.segment,
                    e.group,
                    e.index,
                    e.sub_index.map(|i| i.to_string()).unwrap_or("-".into()),
                    e.description.as_deref().unwrap_or("-"),
                ));
            }
            md.push('\n');
        }

        if !diff.elements.removed.is_empty() {
            md.push_str("### Removed Elements\n\n");
            md.push_str("| Segment | Group | Index | Sub-Index | Description |\n");
            md.push_str("|---------|-------|-------|-----------|-------------|\n");
            for e in &diff.elements.removed {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    e.segment,
                    e.group,
                    e.index,
                    e.sub_index.map(|i| i.to_string()).unwrap_or("-".into()),
                    e.description.as_deref().unwrap_or("-"),
                ));
            }
            md.push('\n');
        }
    }

    if diff.is_empty() {
        md.push_str("**No differences found.**\n");
    }

    md
}
```

Update `crates/automapper-generator/src/schema_diff/mod.rs`:
```rust
pub mod types;
pub mod differ;
pub mod markdown;

pub use types::*;
pub use differ::*;
pub use markdown::*;
```

**Step 3: Run tests**

Run: `cargo test -p automapper-generator test_render_diff_markdown`
Expected: PASS

**Step 4: Verify CLI compiles**

Run: `cargo check -p automapper-generator`
Expected: PASS (now that `render_diff_markdown` exists)

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/schema_diff/markdown.rs crates/automapper-generator/src/schema_diff/mod.rs crates/automapper-generator/src/main.rs crates/automapper-generator/tests/schema_diff_test.rs
git commit -m "feat(generator): add mig-diff CLI subcommand with markdown report"
```

---

## Task 3: Integration Test with Real Schemas via CLI

**Files:**
- Modify: `crates/automapper-generator/tests/schema_diff_test.rs`

**Step 1: Write integration test**

Add to `schema_diff_test.rs`:
```rust
#[test]
fn test_diff_real_schemas_produces_valid_json_and_markdown() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("crates/mig-types/src/generated/fv2504/utilmd/pids");

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
```

**Step 2: Run test**

Run: `cargo test -p automapper-generator test_diff_real_schemas_produces -- --nocapture`
Expected: PASS with markdown preview in output.

**Step 3: Run full test suite and lint**

Run: `cargo test -p automapper-generator && cargo clippy -p automapper-generator -- -D warnings`
Expected: All PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/tests/schema_diff_test.rs
git commit -m "test(generator): add integration test for mig-diff JSON and markdown output"
```
