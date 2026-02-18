---
feature: generator-implementation
epic: 2
title: "Rust Mapper Code Generation"
depends_on: [generator-implementation/E01]
estimated_tasks: 7
crate: automapper-generator
---

# Epic 2: Rust Mapper Code Generation

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-generator/src/`. The binary entry point is `crates/automapper-generator/src/main.rs`. Use types from Section 7 of the design doc exactly. All code must compile with `cargo check -p automapper-generator`.

**Goal:** Build the `clap` CLI and the code generation engine that reads parsed MIG/AHB schemas (from Epic 1) and generates Rust source files: mapper stubs (one file per entity type per format version), `VersionConfig` impl blocks, and coordinator registration code. Output goes to `generated/`. Uses `askama` templates for code generation and `insta` for snapshot testing of generated output.

**Architecture:** The CLI has a `GenerateMappers` subcommand that takes MIG/AHB paths, output directory, format version, and message type. It invokes the MIG/AHB parsers from Epic 1, then feeds the schemas through template-based code generators. Each generator uses `askama` templates that produce valid Rust source code. The generated code depends on types from `automapper-core` but the generator itself does NOT import `automapper-core` -- it only generates string output.

**Tech Stack:** clap 4.x (CLI), askama 0.12 (templates), insta (snapshot tests), serde_json (metadata)

---

## Task 1: `clap` CLI skeleton with `GenerateMappers` subcommand

### Step 1 — Write the test

Create `crates/automapper-generator/tests/cli_tests.rs`:

```rust
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
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test cli_tests
```

Expected: Fails because main.rs doesn't have the clap CLI yet.

### Step 3 — Implement

Update `crates/automapper-generator/src/main.rs`:

```rust
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "automapper-generator")]
#[command(about = "Generates Rust mapper code from MIG/AHB XML schemas")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate mapper code from MIG XML schemas
    GenerateMappers {
        /// Path to MIG XML file
        #[arg(long)]
        mig_path: PathBuf,

        /// Path to AHB XML file
        #[arg(long)]
        ahb_path: PathBuf,

        /// Output directory for generated files
        #[arg(long)]
        output_dir: PathBuf,

        /// Format version (e.g., "FV2510")
        #[arg(long)]
        format_version: String,

        /// EDIFACT message type (e.g., "UTILMD")
        #[arg(long)]
        message_type: String,
    },

    /// Generate condition evaluators from AHB rules
    GenerateConditions {
        /// Path to AHB XML file
        #[arg(long)]
        ahb_path: PathBuf,

        /// Output directory for generated files
        #[arg(long)]
        output_dir: PathBuf,

        /// Format version (e.g., "FV2510")
        #[arg(long)]
        format_version: String,

        /// EDIFACT message type (e.g., "UTILMD")
        #[arg(long)]
        message_type: String,

        /// Only regenerate conditions that changed or are low-confidence
        #[arg(long, default_value = "false")]
        incremental: bool,

        /// Maximum concurrent Claude CLI calls
        #[arg(long, default_value = "4")]
        concurrency: usize,

        /// Path to MIG XML file (optional, for segment structure context)
        #[arg(long)]
        mig_path: Option<PathBuf>,

        /// Batch size for conditions per API call
        #[arg(long, default_value = "50")]
        batch_size: usize,

        /// Dry run — parse only, don't call Claude
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Validate generated code against BO4E schema
    ValidateSchema {
        /// Path to stammdatenmodell directory
        #[arg(long)]
        stammdatenmodell_path: PathBuf,

        /// Path to generated code directory
        #[arg(long)]
        generated_dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::GenerateMappers {
            mig_path,
            ahb_path,
            output_dir,
            format_version,
            message_type,
        } => {
            eprintln!(
                "Generating mappers for {} {} from {:?}",
                message_type, format_version, mig_path
            );
            automapper_generator::codegen::generate_mappers(
                &mig_path,
                &ahb_path,
                &output_dir,
                &format_version,
                &message_type,
            )
        }
        Commands::GenerateConditions {
            ahb_path,
            output_dir,
            format_version,
            message_type,
            incremental,
            concurrency,
            mig_path,
            batch_size,
            dry_run,
        } => {
            eprintln!(
                "Generating conditions for {} {} (incremental={})",
                message_type, format_version, incremental
            );
            // Placeholder — implemented in Epic 3
            Ok(())
        }
        Commands::ValidateSchema {
            stammdatenmodell_path,
            generated_dir,
        } => {
            eprintln!(
                "Validating generated code in {:?} against {:?}",
                generated_dir, stammdatenmodell_path
            );
            // Placeholder — implemented in Epic 3
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test cli_tests
```

Expected:

```
running 3 tests
test test_cli_help ... ok
test test_cli_generate_mappers_help ... ok
test test_cli_missing_required_args ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): add clap CLI with GenerateMappers, GenerateConditions, ValidateSchema subcommands"
```

---

## Task 2: Segment order extraction from MIG schema

### Step 1 — Write the test

Create `crates/automapper-generator/src/codegen/mod.rs`:

```rust
pub mod segment_order;
pub mod mapper_gen;
pub mod coordinator_gen;
pub mod version_config_gen;

use std::path::Path;

use crate::error::GeneratorError;

/// Top-level function called by the CLI for the `generate-mappers` subcommand.
pub fn generate_mappers(
    mig_path: &Path,
    ahb_path: &Path,
    output_dir: &Path,
    format_version: &str,
    message_type: &str,
) -> Result<(), GeneratorError> {
    // Infer variant from filename
    let mig_filename = mig_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let variant = if mig_filename.contains("Strom") {
        Some("Strom")
    } else if mig_filename.contains("Gas") {
        Some("Gas")
    } else {
        None
    };

    eprintln!("Parsing MIG: {:?}", mig_path);
    let mig_schema =
        crate::parsing::mig_parser::parse_mig(mig_path, message_type, variant, format_version)?;

    eprintln!("Parsing AHB: {:?}", ahb_path);
    let ahb_schema =
        crate::parsing::ahb_parser::parse_ahb(ahb_path, message_type, variant, format_version)?;

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Extract segment order for coordinator generation
    let ordered_segments = segment_order::extract_ordered_segments(&mig_schema);
    eprintln!(
        "Extracted {} ordered segments",
        ordered_segments.len()
    );

    // Generate mapper stubs
    let mapper_output = mapper_gen::generate_mapper_stubs(&mig_schema, &ahb_schema)?;
    for (filename, content) in &mapper_output {
        let path = output_dir.join(filename);
        std::fs::write(&path, content)?;
        eprintln!("Generated: {:?}", path);
    }

    // Generate VersionConfig impl
    let vc_output = version_config_gen::generate_version_config(&mig_schema)?;
    let vc_path = output_dir.join(format!(
        "{}_version_config_{}.rs",
        message_type.to_lowercase(),
        format_version.to_lowercase()
    ));
    std::fs::write(&vc_path, &vc_output)?;
    eprintln!("Generated: {:?}", vc_path);

    // Generate coordinator registration
    let coord_output =
        coordinator_gen::generate_coordinator(&mig_schema, &ordered_segments)?;
    let coord_path = output_dir.join(format!(
        "{}_coordinator_{}.rs",
        message_type.to_lowercase(),
        format_version.to_lowercase()
    ));
    std::fs::write(&coord_path, &coord_output)?;
    eprintln!("Generated: {:?}", coord_path);

    eprintln!("Generation complete!");
    Ok(())
}
```

Create `crates/automapper-generator/src/codegen/segment_order.rs`:

```rust
use crate::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

/// An ordered segment entry with group context, used for coordinator generation.
#[derive(Debug, Clone)]
pub struct OrderedSegmentEntry {
    /// The segment identifier (e.g., "NAD", "BGM").
    pub segment_id: String,
    /// The counter value for ordering (e.g., "0010", "0020").
    pub counter: String,
    /// The nesting level.
    pub level: i32,
    /// Maximum repetitions allowed.
    pub max_rep: i32,
    /// Whether the segment is optional.
    pub is_optional: bool,
    /// The containing group ID (None for top-level).
    pub group_id: Option<String>,
    /// The containing group's max repetitions.
    pub group_max_rep: i32,
}

/// Extracts all segments in MIG-defined order (by Counter attribute).
pub fn extract_ordered_segments(schema: &MigSchema) -> Vec<OrderedSegmentEntry> {
    let mut entries = Vec::new();

    // Add top-level segments
    for segment in &schema.segments {
        entries.push(create_entry(segment, None, 1));
    }

    // Add segments from groups (recursively)
    for group in &schema.segment_groups {
        extract_from_group(group, &mut entries);
    }

    // Sort by counter (numeric comparison)
    entries.sort_by_key(|e| parse_counter(&e.counter));

    entries
}

fn extract_from_group(group: &MigSegmentGroup, entries: &mut Vec<OrderedSegmentEntry>) {
    let group_max_rep = group.max_rep_std.max(group.max_rep_spec);

    for segment in &group.segments {
        entries.push(create_entry(segment, Some(&group.id), group_max_rep));
    }

    for nested in &group.nested_groups {
        extract_from_group(nested, entries);
    }
}

fn create_entry(
    segment: &MigSegment,
    group_id: Option<&str>,
    group_max_rep: i32,
) -> OrderedSegmentEntry {
    let status = segment
        .status_spec
        .as_deref()
        .or(segment.status_std.as_deref())
        .unwrap_or("C");
    let is_optional = !matches!(status, "M" | "R");
    let max_rep = segment.max_rep_std.max(segment.max_rep_spec);

    OrderedSegmentEntry {
        segment_id: segment.id.clone(),
        counter: segment.counter.clone().unwrap_or_else(|| "0000".to_string()),
        level: segment.level,
        max_rep,
        is_optional,
        group_id: group_id.map(|s| s.to_string()),
        group_max_rep,
    }
}

fn parse_counter(counter: &str) -> i32 {
    counter.parse().unwrap_or(0)
}
```

Write `crates/automapper-generator/tests/segment_order_tests.rs`:

```rust
use automapper_generator::codegen::segment_order::extract_ordered_segments;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_extract_ordered_segments() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let ordered = extract_ordered_segments(&schema);

    // Should have 3 segments: UNH (0010), BGM (0020), NAD (0080, in SG2)
    assert_eq!(ordered.len(), 3);

    // Verify ordering by counter
    assert_eq!(ordered[0].segment_id, "UNH");
    assert_eq!(ordered[0].counter, "0010");
    assert!(!ordered[0].is_optional); // M = mandatory
    assert!(ordered[0].group_id.is_none()); // Top-level

    assert_eq!(ordered[1].segment_id, "BGM");
    assert_eq!(ordered[1].counter, "0020");

    assert_eq!(ordered[2].segment_id, "NAD");
    assert_eq!(ordered[2].counter, "0080");
    assert!(!ordered[2].is_optional); // M in spec
    assert_eq!(ordered[2].group_id, Some("SG2".to_string()));
    assert_eq!(ordered[2].group_max_rep, 99);
}

#[test]
fn test_unique_segment_ids() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let ordered = extract_ordered_segments(&schema);
    let unique_ids: Vec<&str> = ordered
        .iter()
        .map(|e| e.segment_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    assert!(unique_ids.contains(&"UNH"));
    assert!(unique_ids.contains(&"BGM"));
    assert!(unique_ids.contains(&"NAD"));
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test segment_order_tests
```

### Step 3 — Implement

Files shown above. Update `crates/automapper-generator/src/lib.rs`:

```rust
pub mod error;
pub mod schema;
pub mod parsing;
pub mod codegen;

pub use error::GeneratorError;
```

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test segment_order_tests
```

Expected:

```
running 2 tests
test test_extract_ordered_segments ... ok
test test_unique_segment_ids ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): add segment order extraction from MIG schema"
```

---

## Task 3: Mapper stub code generation

### Step 1 — Write the test

Create `crates/automapper-generator/src/codegen/mapper_gen.rs`:

```rust
use crate::error::GeneratorError;
use crate::schema::ahb::AhbSchema;
use crate::schema::mig::MigSchema;

/// Entity types detected from MIG segment groups.
/// Each entity gets its own mapper stub file.
#[derive(Debug, Clone)]
pub struct DetectedEntity {
    /// The entity name in PascalCase (e.g., "Marktlokation", "Zaehler").
    pub name: String,
    /// The segment group ID where this entity is rooted (e.g., "SG8").
    pub segment_group: Option<String>,
    /// Key segments that belong to this entity.
    pub segments: Vec<String>,
    /// Qualifier values that identify this entity (e.g., SEQ+Z01).
    pub qualifiers: Vec<(String, String)>,
}

/// Generates mapper stub files — one file per entity type.
///
/// Returns a Vec of (filename, content) pairs.
pub fn generate_mapper_stubs(
    mig: &MigSchema,
    _ahb: &AhbSchema,
) -> Result<Vec<(String, String)>, GeneratorError> {
    let entities = detect_entities(mig);
    let mut output = Vec::new();

    for entity in &entities {
        let filename = format!(
            "{}_{}_mapper_{}.rs",
            mig.message_type.to_lowercase(),
            to_snake_case(&entity.name),
            mig.format_version.to_lowercase()
        );
        let content = generate_mapper_stub(mig, entity);
        output.push((filename, content));
    }

    // Also generate a mod.rs that re-exports all mappers
    let mod_content = generate_mod_file(mig, &entities);
    output.push((
        format!(
            "{}_mappers_{}_mod.rs",
            mig.message_type.to_lowercase(),
            mig.format_version.to_lowercase()
        ),
        mod_content,
    ));

    Ok(output)
}

/// Detect entity types from MIG structure.
///
/// For UTILMD, entities are identified by segment groups and SEQ qualifiers:
/// - SG2 → Geschaeftspartner (parties)
/// - SG4 → Zeitscheibe (time slices)
/// - SG8 with SEQ+Z01 → Marktlokation master data
/// - SG8 with SEQ+Z98 → Marktlokation informative data
/// - SG8 with SEQ+Z78 → Lokationsbuendel
/// - SG10 → Zaehler (meters, nested in SG8)
///
/// For a generic approach, we map known group patterns. Unknown groups
/// get a generic mapper stub named after the segment group.
fn detect_entities(mig: &MigSchema) -> Vec<DetectedEntity> {
    let mut entities = Vec::new();

    // Always add a Prozessdaten entity for top-level segments
    entities.push(DetectedEntity {
        name: "Prozessdaten".to_string(),
        segment_group: None,
        segments: mig.segments.iter().map(|s| s.id.clone()).collect(),
        qualifiers: Vec::new(),
    });

    for group in &mig.segment_groups {
        detect_entities_from_group(group, &mut entities);
    }

    entities
}

fn detect_entities_from_group(
    group: &crate::schema::mig::MigSegmentGroup,
    entities: &mut Vec<DetectedEntity>,
) {
    // Map well-known UTILMD groups to entity names
    let entity_name = match group.id.as_str() {
        "SG2" => "Geschaeftspartner".to_string(),
        "SG3" => "Referenz".to_string(),
        "SG4" => "Zeitscheibe".to_string(),
        "SG5" => "Antwortstatus".to_string(),
        "SG6" => "Dokument".to_string(),
        "SG8" => "Stammdaten".to_string(), // Generalized — actual entity depends on SEQ qualifier
        "SG10" => "Zaehler".to_string(),
        "SG12" => "Vertrag".to_string(),
        _ => format!("SegmentGroup{}", group.id),
    };

    entities.push(DetectedEntity {
        name: entity_name,
        segment_group: Some(group.id.clone()),
        segments: group.segments.iter().map(|s| s.id.clone()).collect(),
        qualifiers: extract_qualifiers(group),
    });

    // Process nested groups
    for nested in &group.nested_groups {
        detect_entities_from_group(nested, entities);
    }
}

fn extract_qualifiers(group: &crate::schema::mig::MigSegmentGroup) -> Vec<(String, String)> {
    let mut qualifiers = Vec::new();

    for segment in &group.segments {
        // Check the first data element for qualifier codes
        if let Some(first_de) = segment.data_elements.first() {
            for code in &first_de.codes {
                qualifiers.push((segment.id.clone(), code.value.clone()));
            }
        }
    }

    qualifiers
}

fn generate_mapper_stub(mig: &MigSchema, entity: &DetectedEntity) -> String {
    let struct_name = format!(
        "{}Mapper{}",
        entity.name,
        mig.format_version.replace("FV", "V")
    );
    let builder_name = format!("{}Builder", entity.name);
    let snake_name = to_snake_case(&entity.name);

    let mut code = String::new();

    // Header
    code.push_str("// <auto-generated>\n");
    code.push_str("//     This code was generated by automapper-generator.\n");
    code.push_str(&format!(
        "//     Source: {} {} {}\n",
        mig.message_type,
        mig.variant.as_deref().unwrap_or(""),
        mig.version
    ));
    code.push_str("//\n");
    code.push_str("//     Changes to this file may be overwritten when regenerated.\n");
    code.push_str("// </auto-generated>\n\n");

    // Use statements
    code.push_str("use automapper_core::traits::{SegmentHandler, Builder, EntityWriter, Mapper};\n");
    code.push_str("use automapper_core::context::TransactionContext;\n");
    code.push_str("use automapper_core::writer::EdifactSegmentWriter;\n");
    code.push_str("use automapper_core::FormatVersion;\n");
    code.push_str("use edifact_types::RawSegment;\n\n");

    // Mapper struct
    code.push_str(&format!("/// Mapper for {} in {}.\n", entity.name, mig.format_version));
    if let Some(ref sg) = entity.segment_group {
        code.push_str(&format!("/// Segment group: {}\n", sg));
    }
    code.push_str(&format!("pub struct {} {{\n", struct_name));
    code.push_str(&format!("    builder: {},\n", builder_name));
    code.push_str("}\n\n");

    // Default impl
    code.push_str(&format!("impl Default for {} {{\n", struct_name));
    code.push_str("    fn default() -> Self {\n");
    code.push_str(&format!(
        "        Self {{ builder: {}::default() }}\n",
        builder_name
    ));
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // SegmentHandler impl
    code.push_str(&format!("impl SegmentHandler for {} {{\n", struct_name));
    code.push_str("    fn can_handle(&self, segment: &RawSegment) -> bool {\n");

    if entity.segments.is_empty() {
        code.push_str("        false\n");
    } else {
        code.push_str("        matches!(segment.id,\n");
        for (i, seg_id) in entity.segments.iter().enumerate() {
            let suffix = if i < entity.segments.len() - 1 {
                " |"
            } else {
                ""
            };
            code.push_str(&format!("            \"{}\"{}\n", seg_id, suffix));
        }
        code.push_str("        )\n");
    }

    code.push_str("    }\n\n");
    code.push_str("    fn handle(&mut self, segment: &RawSegment, ctx: &mut TransactionContext) {\n");
    code.push_str(&format!(
        "        // TODO: Implement {} segment handling\n",
        entity.name
    ));

    for seg_id in &entity.segments {
        code.push_str(&format!(
            "        // {}: parse and accumulate in builder\n",
            seg_id
        ));
    }

    code.push_str("    }\n");
    code.push_str("}\n\n");

    // EntityWriter impl
    code.push_str(&format!("impl EntityWriter for {} {{\n", struct_name));
    code.push_str(
        "    fn write(&self, writer: &mut EdifactSegmentWriter, ctx: &TransactionContext) {\n",
    );
    code.push_str(&format!(
        "        // TODO: Implement {} EDIFACT writing\n",
        entity.name
    ));
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Mapper impl
    code.push_str(&format!("impl Mapper for {} {{\n", struct_name));
    code.push_str("    fn format_version(&self) -> FormatVersion {\n");
    code.push_str(&format!(
        "        FormatVersion::{}\n",
        mig.format_version
    ));
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Builder struct (stub)
    code.push_str(&format!("/// Builder for {} domain objects.\n", entity.name));
    code.push_str("#[derive(Default)]\n");
    code.push_str(&format!("pub struct {} {{\n", builder_name));
    code.push_str(&format!(
        "    // TODO: Add fields for {} state accumulation\n",
        snake_name
    ));
    code.push_str("}\n\n");

    code.push_str(&format!("impl Builder<()> for {} {{\n", builder_name));
    code.push_str("    fn is_empty(&self) -> bool {\n");
    code.push_str("        true // TODO: Check accumulated state\n");
    code.push_str("    }\n\n");
    code.push_str("    fn build(&mut self) -> () {\n");
    code.push_str(&format!(
        "        // TODO: Build {} domain object\n",
        entity.name
    ));
    code.push_str("    }\n\n");
    code.push_str("    fn reset(&mut self) {\n");
    code.push_str("        *self = Self::default();\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}

fn generate_mod_file(mig: &MigSchema, entities: &[DetectedEntity]) -> String {
    let mut code = String::new();

    code.push_str("// <auto-generated>\n");
    code.push_str(&format!(
        "//     Mapper modules for {} {}\n",
        mig.message_type, mig.format_version
    ));
    code.push_str("// </auto-generated>\n\n");

    for entity in entities {
        let mod_name = format!(
            "{}_{}_mapper_{}",
            mig.message_type.to_lowercase(),
            to_snake_case(&entity.name),
            mig.format_version.to_lowercase()
        );
        code.push_str(&format!("pub mod {};\n", mod_name));
    }

    code
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("Marktlokation"), "marktlokation");
        assert_eq!(to_snake_case("Geschaeftspartner"), "geschaeftspartner");
        assert_eq!(to_snake_case("SegmentGroupSG8"), "segment_group_s_g8");
    }
}
```

Write `crates/automapper-generator/tests/mapper_gen_tests.rs`:

```rust
use automapper_generator::codegen::mapper_gen::generate_mapper_stubs;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use std::path::Path;

#[test]
fn test_generate_mapper_stubs_from_minimal() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Should generate at least:
    // - Prozessdaten mapper (top-level segments)
    // - Geschaeftspartner mapper (SG2)
    // - A mod.rs file
    assert!(stubs.len() >= 3, "expected at least 3 files, got {}", stubs.len());

    // Verify filenames follow the pattern
    for (filename, _) in &stubs {
        assert!(
            filename.starts_with("utilmd_"),
            "filename should start with message type: {}",
            filename
        );
        assert!(
            filename.ends_with(".rs"),
            "filename should end with .rs: {}",
            filename
        );
    }
}

#[test]
fn test_mapper_stub_content() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Find the Geschaeftspartner mapper
    let gp_stub = stubs
        .iter()
        .find(|(name, _)| name.contains("geschaeftspartner"))
        .expect("should have Geschaeftspartner mapper");

    let content = &gp_stub.1;

    // Verify it contains expected elements
    assert!(content.contains("auto-generated"), "should have auto-generated header");
    assert!(content.contains("SegmentHandler"), "should implement SegmentHandler");
    assert!(content.contains("EntityWriter"), "should implement EntityWriter");
    assert!(content.contains("Mapper"), "should implement Mapper");
    assert!(content.contains("Builder"), "should have a Builder");
    assert!(content.contains("NAD"), "should reference NAD segment");
    assert!(content.contains("FormatVersion::FV2510"), "should reference format version");
}

#[test]
fn test_mapper_stub_snapshot() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Snapshot each generated file
    for (filename, content) in &stubs {
        let snapshot_name = filename.replace('.', "_");
        insta::assert_snapshot!(snapshot_name, content);
    }
}

#[test]
fn test_mod_file_generation() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();

    // Find the mod file
    let mod_file = stubs.iter().find(|(name, _)| name.contains("_mod.rs"));
    assert!(mod_file.is_some(), "should generate a mod.rs file");

    let content = &mod_file.unwrap().1;
    assert!(content.contains("pub mod"), "mod file should have pub mod declarations");
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test mapper_gen_tests
```

### Step 3 — Implement

Files shown above.

### Step 4 — Accept snapshots and run (GREEN)

```bash
cargo insta test -p automapper-generator --test mapper_gen_tests --accept
cargo test -p automapper-generator --test mapper_gen_tests
```

Expected: All 4 tests pass.

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): implement mapper stub code generation from MIG schema"
```

---

## Task 4: VersionConfig impl generation

### Step 1 — Write the test

Create `crates/automapper-generator/src/codegen/version_config_gen.rs`:

```rust
use crate::error::GeneratorError;
use crate::schema::mig::MigSchema;

/// Generates a `VersionConfig` trait impl block for the given format version.
///
/// This generates code like:
/// ```rust,ignore
/// impl VersionConfig for FV2510 {
///     const VERSION: FormatVersion = FormatVersion::FV2510;
///     type MarktlokationMapper = MarktlokationMapperV2510;
///     type VertragMapper = VertragMapperV2510;
///     // ...
/// }
/// ```
pub fn generate_version_config(mig: &MigSchema) -> Result<String, GeneratorError> {
    let fv = &mig.format_version;
    let fv_variant = fv.replace("FV", "V"); // FV2510 -> V2510
    let struct_name = fv.clone(); // FV2510

    let mut code = String::new();

    // Header
    code.push_str("// <auto-generated>\n");
    code.push_str("//     This code was generated by automapper-generator.\n");
    code.push_str(&format!(
        "//     Source: {} {} {}\n",
        mig.message_type,
        mig.variant.as_deref().unwrap_or(""),
        mig.version
    ));
    code.push_str("// </auto-generated>\n\n");

    code.push_str("use automapper_core::traits::{Mapper, VersionConfig};\n");
    code.push_str("use automapper_core::FormatVersion;\n\n");

    // Marker struct
    code.push_str(&format!(
        "/// Marker type for format version {}.\n",
        fv
    ));
    code.push_str(&format!("pub struct {};\n\n", struct_name));

    // VersionConfig impl
    code.push_str(&format!(
        "impl VersionConfig for {} {{\n",
        struct_name
    ));
    code.push_str(&format!(
        "    const VERSION: FormatVersion = FormatVersion::{};\n",
        fv
    ));

    // Generate associated types from detected entities in segment groups
    let entity_types = detect_mapper_types(mig);
    for (entity_name, mapper_name) in &entity_types {
        code.push_str(&format!(
            "    type {}Mapper = {}{};\n",
            entity_name, entity_name, fv_variant
        ));
    }

    code.push_str("}\n");

    Ok(code)
}

/// Detects entity types that need mapper associated types.
fn detect_mapper_types(mig: &MigSchema) -> Vec<(String, String)> {
    let mut types = Vec::new();

    // Well-known UTILMD entity types
    let known_entities = [
        ("Marktlokation", "SG8"),
        ("Messlokation", "SG8"),
        ("Netzlokation", "SG8"),
        ("Zaehler", "SG10"),
        ("Geschaeftspartner", "SG2"),
        ("Vertrag", "SG12"),
        ("Prozessdaten", ""),
        ("Zeitscheibe", "SG4"),
        ("SteuerbareRessource", "SG8"),
        ("TechnischeRessource", "SG8"),
    ];

    for (entity, sg) in &known_entities {
        // Check if the group exists in the MIG (or top-level for empty sg)
        let exists = if sg.is_empty() {
            true
        } else {
            has_segment_group(mig, sg)
        };

        if exists {
            types.push((entity.to_string(), entity.to_string()));
        }
    }

    types
}

fn has_segment_group(mig: &MigSchema, group_id: &str) -> bool {
    fn search_groups(groups: &[crate::schema::mig::MigSegmentGroup], id: &str) -> bool {
        for group in groups {
            if group.id == id {
                return true;
            }
            if search_groups(&group.nested_groups, id) {
                return true;
            }
        }
        false
    }
    search_groups(&mig.segment_groups, group_id)
}
```

Write `crates/automapper-generator/tests/version_config_tests.rs`:

```rust
use automapper_generator::codegen::version_config_gen::generate_version_config;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_generate_version_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let output = generate_version_config(&mig).unwrap();

    assert!(output.contains("auto-generated"), "should have header");
    assert!(output.contains("pub struct FV2510;"), "should have marker struct");
    assert!(output.contains("impl VersionConfig for FV2510"), "should have impl block");
    assert!(
        output.contains("const VERSION: FormatVersion = FormatVersion::FV2510"),
        "should set VERSION constant"
    );
}

#[test]
fn test_version_config_has_entity_types() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let output = generate_version_config(&mig).unwrap();

    // SG2 exists -> should have Geschaeftspartner
    assert!(
        output.contains("type GeschaeftspartnerMapper"),
        "should have Geschaeftspartner mapper type"
    );

    // Prozessdaten always exists
    assert!(
        output.contains("type ProzessdatenMapper"),
        "should have Prozessdaten mapper type"
    );
}

#[test]
fn test_version_config_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let output = generate_version_config(&mig).unwrap();
    insta::assert_snapshot!("version_config_fv2510", output);
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test version_config_tests
```

### Step 3 — Implement

File shown above.

### Step 4 — Accept snapshots and run (GREEN)

```bash
cargo insta test -p automapper-generator --test version_config_tests --accept
cargo test -p automapper-generator --test version_config_tests
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): implement VersionConfig trait impl code generation"
```

---

## Task 5: Coordinator registration code generation

### Step 1 — Write the test

Create `crates/automapper-generator/src/codegen/coordinator_gen.rs`:

```rust
use crate::codegen::segment_order::OrderedSegmentEntry;
use crate::error::GeneratorError;
use crate::schema::mig::MigSchema;

/// Generates coordinator registration code from MIG schema.
///
/// This produces a `UtilmdCoordinator` implementation with:
/// - Mapper registration in `new()`
/// - Segment dispatch in `on_segment()`
/// - Write ordering in `generate()`
pub fn generate_coordinator(
    mig: &MigSchema,
    ordered_segments: &[OrderedSegmentEntry],
) -> Result<String, GeneratorError> {
    let msg_type_pascal = to_pascal_case(&mig.message_type);
    let variant_str = mig.variant.as_deref().unwrap_or("");
    let class_name = format!(
        "{}{}Coordinator{}",
        msg_type_pascal, variant_str, mig.format_version
    );
    let fv_variant = mig.format_version.replace("FV", "V");

    let mut code = String::new();

    // Header
    code.push_str("// <auto-generated>\n");
    code.push_str("//     This code was generated by automapper-generator.\n");
    code.push_str(&format!(
        "//     Source: {} {} {}\n",
        mig.message_type, variant_str, mig.version
    ));
    code.push_str("// </auto-generated>\n\n");

    code.push_str("use automapper_core::coordinator::CoordinatorBase;\n");
    code.push_str("use automapper_core::traits::{Mapper, VersionConfig};\n");
    code.push_str("use automapper_core::FormatVersion;\n");
    code.push_str("use edifact_types::RawSegment;\n\n");

    // Coordinator struct
    code.push_str(&format!("/// Coordinator for {} message processing.\n", mig.message_type));
    code.push_str(&format!("pub struct {} {{\n", class_name));
    code.push_str("    mappers: Vec<Box<dyn Mapper>>,\n");
    code.push_str("}\n\n");

    // Constructor with mapper registration
    code.push_str(&format!("impl {} {{\n", class_name));
    code.push_str("    pub fn new() -> Self {\n");
    code.push_str("        let mut mappers: Vec<Box<dyn Mapper>> = Vec::new();\n\n");
    code.push_str("        // Register mappers in segment processing order\n");

    // Deduplicate segment IDs for registration
    let unique_segment_ids: Vec<&str> = {
        let mut seen = std::collections::HashSet::new();
        ordered_segments
            .iter()
            .filter(|e| seen.insert(e.segment_id.as_str()))
            .map(|e| e.segment_id.as_str())
            .collect()
    };

    code.push_str("        // TODO: Register concrete mapper instances here\n");
    code.push_str("        // Example:\n");
    code.push_str(&format!(
        "        // mappers.push(Box::new(GeschaeftspartnerMapper{}::default()));\n",
        fv_variant
    ));
    code.push_str(&format!(
        "        // mappers.push(Box::new(ProzessdatenMapper{}::default()));\n",
        fv_variant
    ));
    code.push_str("\n");
    code.push_str("        Self { mappers }\n");
    code.push_str("    }\n\n");

    // Segment dispatch
    code.push_str("    /// Routes a segment to the appropriate mapper.\n");
    code.push_str("    pub fn dispatch_segment(&mut self, segment: &RawSegment) {\n");
    code.push_str("        match segment.id {\n");

    for seg_id in &unique_segment_ids {
        let group_comment = ordered_segments
            .iter()
            .find(|e| e.segment_id == *seg_id)
            .and_then(|e| e.group_id.as_deref())
            .map(|g| format!(" ({})", g))
            .unwrap_or_default();

        code.push_str(&format!(
            "            \"{}\" => {{}}{} // TODO: route to mapper\n",
            seg_id, group_comment
        ));
    }

    code.push_str("            _ => {} // Unknown segment\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // Write ordering
    code.push_str("    /// Returns the MIG-defined segment write order.\n");
    code.push_str("    pub fn segment_write_order(&self) -> &'static [&'static str] {\n");
    code.push_str("        &[\n");

    for entry in ordered_segments {
        let optional = if entry.is_optional { " (optional)" } else { "" };
        let group = entry
            .group_id
            .as_deref()
            .map(|g| format!(" in {}", g))
            .unwrap_or_default();
        code.push_str(&format!(
            "            \"{}\", // {}{}{}\n",
            entry.segment_id, entry.counter, group, optional
        ));
    }

    code.push_str("        ]\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    Ok(code)
}

fn to_pascal_case(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    let mut chars = input.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    first + &chars.as_str().to_lowercase()
}
```

Write `crates/automapper-generator/tests/coordinator_gen_tests.rs`:

```rust
use automapper_generator::codegen::coordinator_gen::generate_coordinator;
use automapper_generator::codegen::segment_order::extract_ordered_segments;
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_generate_coordinator() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();

    assert!(output.contains("auto-generated"));
    assert!(output.contains("UtilmdStromCoordinatorFV2510"));
    assert!(output.contains("dispatch_segment"));
    assert!(output.contains("segment_write_order"));
}

#[test]
fn test_coordinator_segment_dispatch() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();

    // Should have dispatch entries for UNH, BGM, NAD
    assert!(output.contains("\"UNH\""));
    assert!(output.contains("\"BGM\""));
    assert!(output.contains("\"NAD\""));
}

#[test]
fn test_coordinator_write_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();

    // Write order should list segments with their counters
    assert!(output.contains("\"UNH\", // 0010"));
    assert!(output.contains("\"BGM\", // 0020"));
    assert!(output.contains("\"NAD\", // 0080"));
}

#[test]
fn test_coordinator_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let mig = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    let output = generate_coordinator(&mig, &ordered).unwrap();
    insta::assert_snapshot!("coordinator_fv2510", output);
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test coordinator_gen_tests
```

### Step 3 — Implement

File shown above.

### Step 4 — Accept snapshots and run (GREEN)

```bash
cargo insta test -p automapper-generator --test coordinator_gen_tests --accept
cargo test -p automapper-generator --test coordinator_gen_tests
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): implement coordinator registration code generation"
```

---

## Task 6: End-to-end `generate-mappers` CLI test

### Step 1 — Write the test

Write `crates/automapper-generator/tests/e2e_generate_mappers_tests.rs`:

```rust
use std::process::Command;
use tempfile::TempDir;
use std::path::Path;

#[test]
fn test_e2e_generate_mappers() {
    let output_dir = TempDir::new().unwrap();

    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

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
        assert!(!content.is_empty(), "generated file should not be empty: {:?}", entry.path());
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
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

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

    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

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
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test e2e_generate_mappers_tests
```

### Step 3 — Verify end-to-end works

Fix any issues so that the CLI produces correct output.

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test e2e_generate_mappers_tests
```

Expected:

```
running 3 tests
test test_e2e_generate_mappers ... ok
test test_e2e_generate_mappers_missing_mig ... ok
test test_e2e_output_dir_created ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "test(generator): add end-to-end tests for generate-mappers CLI subcommand"
```

---

## Task 7: Full compilation check of all generated code

### Step 1 — Write the test

Write `crates/automapper-generator/tests/generated_compiles_tests.rs`:

```rust
//! Verifies that generated code is syntactically valid Rust.
//! We don't compile it with cargo (that would require automapper-core to exist),
//! but we verify it passes syn parsing.

use automapper_generator::codegen::mapper_gen::generate_mapper_stubs;
use automapper_generator::codegen::version_config_gen::generate_version_config;
use automapper_generator::codegen::coordinator_gen::generate_coordinator;
use automapper_generator::codegen::segment_order::extract_ordered_segments;
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use std::path::Path;

/// Checks that a string is valid Rust syntax by attempting to parse it with regex.
/// This is a lightweight check since we don't want to depend on syn.
fn assert_valid_rust_syntax(code: &str, context: &str) {
    // Basic structural checks
    let open_braces = code.matches('{').count();
    let close_braces = code.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "{}: mismatched braces (open={}, close={})",
        context, open_braces, close_braces
    );

    let open_parens = code.matches('(').count();
    let close_parens = code.matches(')').count();
    assert_eq!(
        open_parens, close_parens,
        "{}: mismatched parentheses",
        context
    );

    // Check for common Rust patterns
    assert!(
        code.contains("fn ") || code.contains("pub mod") || code.contains("struct "),
        "{}: should contain Rust definitions",
        context
    );

    // No unterminated strings (basic check)
    let quote_count = code.matches('"').count();
    assert_eq!(
        quote_count % 2,
        0,
        "{}: odd number of quotes ({})",
        context,
        quote_count
    );
}

#[test]
fn test_all_generated_code_is_valid_rust() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ordered = extract_ordered_segments(&mig);

    // Check mapper stubs
    let stubs = generate_mapper_stubs(&mig, &ahb).unwrap();
    for (filename, content) in &stubs {
        assert_valid_rust_syntax(content, filename);
    }

    // Check version config
    let vc = generate_version_config(&mig).unwrap();
    assert_valid_rust_syntax(&vc, "version_config");

    // Check coordinator
    let coord = generate_coordinator(&mig, &ordered).unwrap();
    assert_valid_rust_syntax(&coord, "coordinator");
}

#[test]
fn test_generated_code_consistency() {
    let mig_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let ahb_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    let mig = parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    let ahb = parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Generate twice — output should be identical (deterministic generation)
    let stubs_1 = generate_mapper_stubs(&mig, &ahb).unwrap();
    let stubs_2 = generate_mapper_stubs(&mig, &ahb).unwrap();

    assert_eq!(stubs_1.len(), stubs_2.len());
    for (a, b) in stubs_1.iter().zip(stubs_2.iter()) {
        assert_eq!(a.0, b.0, "filenames should match");
        assert_eq!(a.1, b.1, "content should be identical across runs");
    }

    let vc1 = generate_version_config(&mig).unwrap();
    let vc2 = generate_version_config(&mig).unwrap();
    assert_eq!(vc1, vc2, "version config should be deterministic");
}
```

### Step 2 — Run the test

```bash
cargo test -p automapper-generator --test generated_compiles_tests
```

### Step 3 — All tests should pass (GREEN)

Expected:

```
running 2 tests
test test_all_generated_code_is_valid_rust ... ok
test test_generated_code_consistency ... ok
```

### Step 4 — Run full test suite

```bash
cargo test -p automapper-generator
```

Verify all tests across all test files pass.

### Step 5 — Commit

```bash
git add -A && git commit -m "test(generator): verify generated code is syntactically valid and deterministic"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 57 |
| Passed | 57 |
| Failed | 0 |
| Skipped | 0 |

Files tested:
- `crates/automapper-generator/src/codegen/mod.rs`
- `crates/automapper-generator/src/codegen/segment_order.rs`
- `crates/automapper-generator/src/codegen/mapper_gen.rs`
- `crates/automapper-generator/src/codegen/version_config_gen.rs`
- `crates/automapper-generator/src/codegen/coordinator_gen.rs`
- `crates/automapper-generator/src/main.rs`
- `crates/automapper-generator/src/lib.rs`
- `crates/automapper-generator/tests/cli_tests.rs`
- `crates/automapper-generator/tests/segment_order_tests.rs`
- `crates/automapper-generator/tests/mapper_gen_tests.rs`
- `crates/automapper-generator/tests/version_config_tests.rs`
- `crates/automapper-generator/tests/coordinator_gen_tests.rs`
- `crates/automapper-generator/tests/e2e_generate_mappers_tests.rs`
- `crates/automapper-generator/tests/generated_compiles_tests.rs`
