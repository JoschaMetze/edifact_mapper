use std::path::{Path, PathBuf};

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
    /// Generate shared MIG types (enums, composites, segments, groups)
    GenerateMigTypes {
        /// Path to MIG XML file
        #[arg(long)]
        mig_path: PathBuf,

        /// Output directory for generated files (e.g., crates/mig-types/src/generated)
        #[arg(long)]
        output_dir: PathBuf,

        /// Format version (e.g., "FV2504")
        #[arg(long)]
        format_version: String,

        /// EDIFACT message type (e.g., "UTILMD")
        #[arg(long)]
        message_type: String,
    },

    /// Generate per-PID composition types from AHB + MIG XML
    GeneratePidTypes {
        /// Path to MIG XML file
        #[arg(long)]
        mig_path: PathBuf,

        /// Path to AHB XML file
        #[arg(long)]
        ahb_path: PathBuf,

        /// Output directory for generated files (e.g., crates/mig-types/src/generated)
        #[arg(long)]
        output_dir: PathBuf,

        /// Format version (e.g., "FV2504")
        #[arg(long)]
        format_version: String,

        /// EDIFACT message type (e.g., "UTILMD")
        #[arg(long)]
        message_type: String,
    },

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

        /// Only regenerate conditions that changed or are low-confidence.
        /// Default: true. Use --no-incremental to force full regeneration.
        #[arg(long, default_value = "true")]
        incremental: bool,

        /// Force regeneration of all conditions, ignoring metadata.
        #[arg(long, default_value = "false")]
        force: bool,

        /// Maximum concurrent Claude CLI calls
        #[arg(long, default_value = "4")]
        max_concurrent: usize,

        /// Path to MIG XML file (optional, for segment structure context)
        #[arg(long)]
        mig_path: Option<PathBuf>,

        /// Batch size for conditions per API call
        #[arg(long, default_value = "10")]
        batch_size: usize,

        /// Dry run — parse only, don't call Claude
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Maximum number of conditions to generate (useful for testing)
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Generate TOML mapping scaffolds from PID schema
    GenerateTomlScaffolds {
        /// Path to MIG XML file
        #[arg(long)]
        mig_path: PathBuf,

        /// Path to AHB XML file
        #[arg(long)]
        ahb_path: PathBuf,

        /// Output directory for generated TOML files
        #[arg(long)]
        output_dir: PathBuf,

        /// Format version (e.g., "FV2504")
        #[arg(long)]
        format_version: String,

        /// EDIFACT message type (e.g., "UTILMD")
        #[arg(long)]
        message_type: String,

        /// Generate scaffolds only for this PID (e.g., "55001")
        #[arg(long)]
        pid: Option<String>,
    },

    /// Generate TOML mappings for a PID using reference mappings from existing PIDs
    GeneratePidMappings {
        /// PID to generate mappings for (e.g., "55109")
        #[arg(long)]
        pid: String,

        /// Directory containing pid_*_schema.json files
        #[arg(long)]
        schema_dir: PathBuf,

        /// Base directory for TOML mappings (e.g., "mappings")
        #[arg(long)]
        mappings_dir: PathBuf,

        /// Format version (e.g., "FV2504")
        #[arg(long)]
        format_version: String,

        /// Message type variant (e.g., "UTILMD_Strom")
        #[arg(long)]
        message_type: String,

        /// Overwrite existing mapping files
        #[arg(long, default_value = "false")]
        overwrite: bool,
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

    /// Look up PID schema context for TOML mapping authoring
    SchemaLookup {
        /// PID number (e.g., "55035")
        #[arg(long)]
        pid: String,

        /// Directory containing pid_*_schema.json files
        #[arg(
            long,
            default_value = "crates/mig-types/src/generated/fv2504/utilmd/pids"
        )]
        schema_dir: PathBuf,

        /// Group path to show detail for (e.g., "sg4.sg8_zf0"). Omit to list all groups.
        #[arg(long)]
        group: Option<String>,

        /// Include a pre-filled TOML template in the output
        #[arg(long, default_value = "false")]
        toml_template: bool,
    },

    /// Migrate TOML mapping files from numeric paths to EDIFACT ID paths
    MigratePaths {
        /// Directory containing pid_*_schema.json files
        #[arg(long)]
        schema_dir: PathBuf,

        /// Root directory of TOML mappings (e.g., mappings/FV2504/UTILMD_Strom)
        #[arg(long)]
        mappings_dir: PathBuf,

        /// Preview changes without writing files
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Generate TypeScript type definitions from TOML mappings and PID schemas
    GenerateTypescript {
        /// PIDs to generate types for (comma-separated, e.g., "55001,55002")
        #[arg(long)]
        pids: String,

        /// Directory containing pid_*_schema.json files
        #[arg(long)]
        schema_dir: PathBuf,

        /// Base directory for TOML mappings (e.g., "mappings")
        #[arg(long)]
        mappings_dir: PathBuf,

        /// Format version (e.g., "FV2504")
        #[arg(long)]
        format_version: String,

        /// Message type variant (e.g., "UTILMD_Strom")
        #[arg(long)]
        message_type: String,

        /// Output directory for generated .d.ts files
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Generate a synthetic EDIFACT fixture from a PID schema JSON.
    GenerateFixture {
        /// Path to the PID schema JSON file (e.g., pid_55043_schema.json)
        #[arg(long)]
        pid_schema: PathBuf,

        /// Output path for the generated .edi file
        #[arg(long)]
        output: PathBuf,

        /// Validate the generated fixture against MIG/AHB schemas
        #[arg(long)]
        validate: bool,

        /// Path to MIG XML file (required when --validate is set)
        #[arg(long)]
        mig_xml: Option<PathBuf>,

        /// Path to AHB XML file (required when --validate is set)
        #[arg(long)]
        ahb_xml: Option<PathBuf>,

        /// EDIFACT message type (default: UTILMD)
        #[arg(long, default_value = "UTILMD")]
        message_type: Option<String>,

        /// Message type variant (default: Strom)
        #[arg(long, default_value = "Strom")]
        variant: Option<String>,

        /// Format version (default: FV2504)
        #[arg(long, default_value = "FV2504")]
        format_version: Option<String>,

        /// Enhance fixture with realistic values via BO4E roundtrip
        #[arg(long)]
        enhance: bool,

        /// Number of enhanced variants to generate (default: 1)
        #[arg(long, default_value = "1")]
        variants: Option<usize>,

        /// Seed for deterministic generation (default: 42)
        #[arg(long, default_value = "42")]
        seed: Option<u64>,
    },
}

/// Parses an existing generated condition evaluator `.rs` file and extracts
/// complete method blocks (doc comments + signature + body) keyed by condition number.
///
/// Returns a `HashMap<u32, String>` where the value is the full method text
/// including leading doc comments and trailing `}\n`.
fn parse_existing_method_bodies(source: &str) -> std::collections::HashMap<u32, String> {
    use std::collections::HashMap;

    let mut methods: HashMap<u32, String> = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for method signatures: `    fn evaluate_N(&self, ctx: &EvaluationContext) -> ConditionResult {`
        let trimmed = lines[i].trim();
        if trimmed.starts_with("fn evaluate_")
            && trimmed.contains("(&self")
            && trimmed.ends_with('{')
        {
            // Extract condition number from `evaluate_N`
            let after = trimmed.strip_prefix("fn evaluate_").unwrap_or("");
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = num_str.parse::<u32>() {
                // Collect doc comments above this method
                let mut comment_start = i;
                while comment_start > 0 {
                    let prev = lines[comment_start - 1].trim();
                    if prev.starts_with("///") || prev.starts_with("// ") {
                        comment_start -= 1;
                    } else {
                        break;
                    }
                }

                // Find closing brace (track brace depth)
                let mut depth = 1;
                let mut j = i + 1;
                while j < lines.len() && depth > 0 {
                    for ch in lines[j].chars() {
                        match ch {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                    }
                    j += 1;
                }

                // Collect the full block: comments + signature + body + closing brace
                let block: String = lines[comment_start..j]
                    .iter()
                    .map(|l| format!("{}\n", l))
                    .collect();

                methods.insert(num, block);
                i = j;
                continue;
            }
        }
        i += 1;
    }

    methods
}

/// Infer energy variant (Strom/Gas) from a filename.
fn infer_variant(filename: &str) -> Option<&'static str> {
    if filename.contains("Strom") {
        Some("Strom")
    } else if filename.contains("Gas") {
        Some("Gas")
    } else {
        None
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), automapper_generator::GeneratorError> {
    match cli.command {
        Commands::GenerateMigTypes {
            mig_path,
            output_dir,
            format_version,
            message_type,
        } => {
            let variant =
                infer_variant(mig_path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
            eprintln!(
                "Generating MIG types for {} {} from {:?}",
                message_type, format_version, mig_path
            );
            automapper_generator::codegen::mig_type_gen::generate_mig_types(
                &mig_path,
                &message_type,
                variant,
                &format_version,
                &output_dir,
            )?;
            eprintln!("MIG types generated to {:?}", output_dir);
            Ok(())
        }
        Commands::GeneratePidTypes {
            mig_path,
            ahb_path,
            output_dir,
            format_version,
            message_type,
        } => {
            let mig_variant =
                infer_variant(mig_path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
            let ahb_variant =
                infer_variant(ahb_path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
            let variant = mig_variant.or(ahb_variant);
            eprintln!(
                "Generating PID types for {} {} from {:?} + {:?}",
                message_type, format_version, mig_path, ahb_path
            );
            let mig = automapper_generator::parsing::mig_parser::parse_mig(
                &mig_path,
                &message_type,
                variant,
                &format_version,
            )?;
            let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(
                &ahb_path,
                &message_type,
                variant,
                &format_version,
            )?;
            automapper_generator::codegen::pid_type_gen::generate_pid_types(
                &mig,
                &ahb,
                &format_version,
                &output_dir,
            )?;
            eprintln!(
                "Generated {} PID types to {:?}",
                ahb.workflows.len(),
                output_dir
            );
            Ok(())
        }
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
            force,
            max_concurrent,
            mig_path,
            batch_size,
            dry_run,
            limit,
        } => {
            // Parse AHB
            let variant =
                infer_variant(ahb_path.file_name().and_then(|n| n.to_str()).unwrap_or(""));

            let ahb_schema = automapper_generator::parsing::ahb_parser::parse_ahb(
                &ahb_path,
                &message_type,
                variant,
                &format_version,
            )?;

            // Optionally parse MIG for segment structure context
            let mig_schema = if let Some(ref mig) = mig_path {
                Some(automapper_generator::parsing::mig_parser::parse_mig(
                    mig,
                    &message_type,
                    variant,
                    &format_version,
                )?)
            } else {
                None
            };

            // Extract condition descriptions
            let conditions: Vec<(String, String)> = ahb_schema
                .bedingungen
                .iter()
                .map(|b| (b.id.clone(), b.description.clone()))
                .collect();

            eprintln!("Found {} conditions", conditions.len());

            // Create output directory
            std::fs::create_dir_all(&output_dir)?;

            // Load existing metadata for incremental mode
            let metadata_path = output_dir.join(format!(
                "{}_condition_evaluator_{}.conditions.json",
                message_type.to_lowercase(),
                format_version.to_lowercase()
            ));
            let existing_metadata =
                automapper_generator::conditions::metadata::load_metadata(&metadata_path)?;

            // Load existing .rs file to extract preserved method bodies
            let output_path = output_dir.join(format!(
                "{}_conditions_{}.rs",
                message_type.to_lowercase(),
                format_version.to_lowercase()
            ));
            let existing_method_bodies = if output_path.exists() {
                let existing_source = std::fs::read_to_string(&output_path)?;
                parse_existing_method_bodies(&existing_source)
            } else {
                std::collections::HashMap::new()
            };

            // Populate existing IDs from metadata
            let existing_ids: std::collections::HashSet<String> = existing_metadata
                .as_ref()
                .map(|m| m.conditions.keys().cloned().collect())
                .unwrap_or_default();

            let force_all = force || (!incremental && existing_metadata.is_none());
            eprintln!(
                "Generating conditions for {} {} (incremental={}, force={})",
                message_type, format_version, incremental, force_all
            );

            // Determine what needs regeneration
            let decision = automapper_generator::conditions::metadata::decide_regeneration(
                &conditions,
                existing_metadata.as_ref(),
                &existing_ids,
                force_all,
            );

            eprintln!(
                "Regeneration: {} to regenerate, {} to preserve",
                decision.to_regenerate.len(),
                decision.to_preserve.len()
            );

            if dry_run {
                eprintln!("\n=== DRY RUN MODE ===");
                for item in &decision.to_regenerate {
                    eprintln!(
                        "  [{}] {} - {}",
                        item.condition_id, item.reason, item.description
                    );
                }
                return Ok(());
            }

            if decision.to_regenerate.is_empty() {
                eprintln!("All conditions are up-to-date. Nothing to regenerate.");
                return Ok(());
            }

            // Build condition inputs
            let to_regenerate = if let Some(max) = limit {
                eprintln!("Limiting to first {} conditions (--limit)", max);
                &decision.to_regenerate[..max.min(decision.to_regenerate.len())]
            } else {
                &decision.to_regenerate
            };
            let condition_inputs: Vec<
                automapper_generator::conditions::condition_types::ConditionInput,
            > = to_regenerate
                .iter()
                .map(
                    |c| automapper_generator::conditions::condition_types::ConditionInput {
                        id: c.condition_id.clone(),
                        description: c.description.clone(),
                        referencing_fields: None,
                    },
                )
                .collect();

            // Generate conditions in batches
            let generator =
                automapper_generator::conditions::claude_generator::ClaudeConditionGenerator::new(
                    max_concurrent,
                );
            let context = automapper_generator::conditions::prompt::ConditionContext {
                message_type: &message_type,
                format_version: &format_version,
                mig_schema: mig_schema.as_ref(),
                example_implementations:
                    automapper_generator::conditions::prompt::default_example_implementations(),
            };

            let mut all_generated = Vec::new();

            for (i, batch) in condition_inputs.chunks(batch_size).enumerate() {
                let batch_ids: Vec<&str> = batch.iter().map(|c| c.id.as_str()).collect();
                let id_range = format!(
                    "{}..{}",
                    batch_ids.first().unwrap_or(&"?"),
                    batch_ids.last().unwrap_or(&"?")
                );
                eprint!(
                    "[Batch {}/{}] Sending {} conditions ({}) to Claude... ",
                    i + 1,
                    condition_inputs.len().div_ceil(batch_size),
                    batch.len(),
                    id_range,
                );
                // Flush so the user sees the message before the blocking call
                use std::io::Write;
                let _ = std::io::stderr().flush();

                let batch_start = std::time::Instant::now();
                match generator.generate_batch(batch, &context) {
                    Ok(mut generated) => {
                        // Enrich with original descriptions
                        for gc in &mut generated {
                            if let Some(input) = batch
                                .iter()
                                .find(|c| c.id == gc.condition_number.to_string())
                            {
                                gc.original_description = Some(input.description.clone());
                                gc.referencing_fields = input.referencing_fields.clone();
                            }
                        }
                        let high = generated
                            .iter()
                            .filter(|c| {
                                c.confidence
                                    == automapper_generator::conditions::condition_types::ConfidenceLevel::High
                            })
                            .count();
                        let medium = generated
                            .iter()
                            .filter(|c| {
                                c.confidence
                                    == automapper_generator::conditions::condition_types::ConfidenceLevel::Medium
                            })
                            .count();
                        let low = generated
                            .iter()
                            .filter(|c| {
                                c.confidence
                                    == automapper_generator::conditions::condition_types::ConfidenceLevel::Low
                            })
                            .count();
                        eprintln!(
                            "done in {:.1}s — {} results (High: {}, Medium: {}, Low: {})",
                            batch_start.elapsed().as_secs_f64(),
                            generated.len(),
                            high,
                            medium,
                            low
                        );
                        all_generated.extend(generated);
                    }
                    Err(e) => {
                        eprintln!(
                            "FAILED after {:.1}s — {}",
                            batch_start.elapsed().as_secs_f64(),
                            e
                        );
                    }
                }
            }

            // Build preserved method bodies for conditions we're keeping
            let preserved: std::collections::HashMap<u32, String> = decision
                .to_preserve
                .iter()
                .filter_map(|id| {
                    let num: u32 = id.parse().ok()?;
                    let body = existing_method_bodies.get(&num)?;
                    Some((num, body.clone()))
                })
                .collect();

            if !preserved.is_empty() {
                eprintln!("Preserving {} existing conditions", preserved.len());
            }

            // Collect external IDs from preserved conditions (from metadata)
            let preserved_external_ids: std::collections::HashSet<u32> =
                if let Some(ref existing_meta) = existing_metadata {
                    decision
                        .to_preserve
                        .iter()
                        .filter_map(|id| {
                            let meta = existing_meta.conditions.get(id)?;
                            if meta.is_external {
                                id.parse().ok()
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    std::collections::HashSet::new()
                };

            // Generate output file
            let source_code =
                automapper_generator::conditions::codegen::generate_condition_evaluator_file(
                    &message_type,
                    &format_version,
                    &all_generated,
                    ahb_path.to_str().unwrap_or("unknown"),
                    &preserved,
                    &preserved_external_ids,
                );

            std::fs::write(&output_path, &source_code)?;
            eprintln!("Generated: {:?}", output_path);

            // Run rustfmt on the generated file to ensure consistent formatting
            match std::process::Command::new("rustfmt")
                .arg(&output_path)
                .status()
            {
                Ok(status) if status.success() => {
                    eprintln!("Formatted: {:?}", output_path);
                }
                Ok(status) => {
                    eprintln!(
                        "Warning: rustfmt exited with {} for {:?}",
                        status, output_path
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Warning: rustfmt not available ({}), skipping formatting",
                        e
                    );
                }
            }

            // Save metadata — merge newly generated with preserved
            let mut meta_conditions: std::collections::HashMap<
                String,
                automapper_generator::conditions::metadata::ConditionMetadata,
            > = std::collections::HashMap::new();

            // First, carry over preserved conditions from existing metadata
            if let Some(ref existing_meta) = existing_metadata {
                for id in &decision.to_preserve {
                    if let Some(meta) = existing_meta.conditions.get(id) {
                        meta_conditions.insert(id.clone(), meta.clone());
                    }
                }
            }

            // Then add newly generated conditions
            for gc in &all_generated {
                let desc = gc.original_description.as_deref().unwrap_or("");
                meta_conditions.insert(
                    gc.condition_number.to_string(),
                    automapper_generator::conditions::metadata::ConditionMetadata {
                        confidence: gc.confidence.to_string(),
                        reasoning: gc.reasoning.clone(),
                        description_hash:
                            automapper_generator::conditions::metadata::compute_description_hash(
                                desc,
                            ),
                        is_external: gc.is_external,
                    },
                );
            }

            let metadata_file = automapper_generator::conditions::metadata::ConditionMetadataFile {
                generated_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                ahb_file: ahb_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                format_version: format_version.clone(),
                conditions: meta_conditions,
            };

            automapper_generator::conditions::metadata::save_metadata(
                &metadata_path,
                &metadata_file,
            )?;
            eprintln!("Metadata: {:?}", metadata_path);
            eprintln!("Generation complete!");

            Ok(())
        }
        Commands::GenerateTomlScaffolds {
            mig_path,
            ahb_path,
            output_dir,
            format_version,
            message_type,
            pid,
        } => {
            let mig_variant =
                infer_variant(mig_path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
            let ahb_variant =
                infer_variant(ahb_path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
            let variant = mig_variant.or(ahb_variant);
            let mig = automapper_generator::parsing::mig_parser::parse_mig(
                &mig_path,
                &message_type,
                variant,
                &format_version,
            )?;
            let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(
                &ahb_path,
                &message_type,
                variant,
                &format_version,
            )?;

            let pids: Vec<&str> = if let Some(ref p) = pid {
                vec![p.as_str()]
            } else {
                ahb.workflows.iter().map(|w| w.id.as_str()).collect()
            };

            std::fs::create_dir_all(&output_dir)?;
            let mut total = 0;

            for pid_id in pids {
                let pid_dir = output_dir.join(format!("pid_{}", pid_id.to_lowercase()));
                std::fs::create_dir_all(&pid_dir)?;

                let scaffolds =
                    automapper_generator::codegen::toml_scaffold_gen::generate_pid_scaffolds(
                        pid_id, &mig, &ahb,
                    );

                for (filename, content) in &scaffolds {
                    std::fs::write(pid_dir.join(filename), content)?;
                }
                total += scaffolds.len();
                eprintln!("PID {}: {} scaffolds", pid_id, scaffolds.len());
            }

            eprintln!(
                "Generated {} total TOML scaffolds to {:?}",
                total, output_dir
            );
            Ok(())
        }
        Commands::GeneratePidMappings {
            pid,
            schema_dir,
            mappings_dir,
            format_version,
            message_type,
            overwrite,
        } => {
            eprintln!(
                "Generating TOML mappings for PID {} ({} {})",
                pid, message_type, format_version
            );

            // Find schema JSON
            let schema_path = schema_dir.join(format!("pid_{}_schema.json", pid.to_lowercase()));
            if !schema_path.exists() {
                return Err(automapper_generator::GeneratorError::FileNotFound(
                    schema_path,
                ));
            }

            // Generate mappings
            let (files, report) =
                automapper_generator::codegen::pid_mapping_gen::generate_pid_mappings(
                    &pid,
                    &schema_path,
                    &mappings_dir,
                    &format_version,
                    &message_type,
                )?;

            // Write output files
            let output_dir = mappings_dir
                .join(&format_version)
                .join(&message_type)
                .join(format!("pid_{}", pid.to_lowercase()));
            std::fs::create_dir_all(&output_dir)?;

            let mut written = 0;
            let mut skipped = 0;
            for (filename, content) in &files {
                let path = output_dir.join(filename);
                if path.exists() && !overwrite {
                    eprintln!("  SKIP (exists): {}", filename);
                    skipped += 1;
                } else {
                    std::fs::write(&path, content)?;
                    written += 1;
                }
            }

            // Print report
            eprintln!("\n=== Generation Report ===");
            eprintln!(
                "Written: {}, Skipped: {}, Total: {}",
                written,
                skipped,
                files.len()
            );
            eprintln!(
                "Matched: {}, Scaffolded: {}",
                report.matched.len(),
                report.scaffolded.len()
            );
            for (entity, source_pid) in &report.matched {
                eprintln!("  MATCH: {} (from pid_{})", entity, source_pid);
            }
            for (entity, reason) in &report.scaffolded {
                eprintln!("  SCAFFOLD: {} ({})", entity, reason);
            }
            eprintln!("\nOutput: {:?}", output_dir);
            Ok(())
        }
        Commands::MigrateFixture {
            old_fixture,
            diff,
            new_pid_schema,
            output,
        } => {
            use automapper_generator::fixture_migrator::migrate_fixture;
            use automapper_generator::schema_diff::types::PidSchemaDiff;

            // Load inputs
            let old_edi = std::fs::read_to_string(&old_fixture)?;
            let diff_json: PidSchemaDiff = serde_json::from_str(&std::fs::read_to_string(&diff)?)?;
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
                println!(
                    "Wrote {} warnings: {}",
                    result.warnings.len(),
                    warnings_path.display()
                );
            }

            // Print summary
            println!("\nMigration summary:");
            println!("  Segments copied:    {}", result.stats.segments_copied);
            println!("  Segments removed:   {}", result.stats.segments_removed);
            println!("  Segments added:     {}", result.stats.segments_added);
            println!("  Codes substituted:  {}", result.stats.codes_substituted);
            println!(
                "  Manual review items: {}",
                result.stats.manual_review_items
            );

            Ok(())
        }
        Commands::MigrateFixtureDir {
            input_dir,
            diff,
            new_pid_schema,
            output_dir,
        } => {
            use automapper_generator::fixture_migrator::batch::migrate_directory;
            use automapper_generator::schema_diff::types::PidSchemaDiff;

            let diff_json: PidSchemaDiff = serde_json::from_str(&std::fs::read_to_string(&diff)?)?;
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
            println!(
                "  {} succeeded, {} failed, {} total warnings",
                success, failed, total_warnings
            );

            Ok(())
        }
        Commands::MigDiff {
            old_schema,
            new_schema,
            old_version,
            new_version,
            message_type,
            pid,
            output_dir,
        } => {
            use automapper_generator::schema_diff::{
                diff_pid_schemas, render_diff_markdown, DiffInput,
            };

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
            let md = render_diff_markdown(&diff);
            std::fs::write(&md_path, &md)?;
            println!("Wrote diff summary: {}", md_path.display());

            // Print summary
            println!(
                "\nDiff summary ({} → {}, PID {}):",
                old_version, new_version, pid
            );
            println!(
                "  Groups:   +{} -{} ~{}",
                diff.groups.added.len(),
                diff.groups.removed.len(),
                diff.groups.restructured.len(),
            );
            println!(
                "  Segments: +{} -{}",
                diff.segments.added.len(),
                diff.segments.removed.len(),
            );
            println!("  Codes:    {} changes", diff.codes.changed.len());
            println!(
                "  Elements: +{} -{}",
                diff.elements.added.len(),
                diff.elements.removed.len(),
            );

            if diff.is_empty() {
                println!("\nNo differences found.");
            }

            Ok(())
        }
        Commands::SchemaLookup {
            pid,
            schema_dir,
            group,
            toml_template,
        } => {
            let schema_path = schema_dir.join(format!("pid_{}_schema.json", pid.to_lowercase()));
            if !schema_path.exists() {
                return Err(automapper_generator::GeneratorError::FileNotFound(
                    schema_path,
                ));
            }
            let schema: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&schema_path)?)?;

            if let Some(ref group_path) = group {
                match automapper_generator::codegen::schema_lookup::print_group_detail(
                    &schema, group_path,
                ) {
                    Some(detail) => {
                        println!("{}", detail);
                        if toml_template {
                            if let Some(template) =
                                automapper_generator::codegen::schema_lookup::print_toml_template(
                                    &schema, group_path,
                                )
                            {
                                println!("---\n");
                                println!("{}", template);
                            }
                        }
                    }
                    None => {
                        eprintln!("Group '{}' not found in PID {} schema", group_path, pid);
                        std::process::exit(1);
                    }
                }
            } else {
                println!(
                    "{}",
                    automapper_generator::codegen::schema_lookup::print_group_list(&schema)
                );
            }

            Ok(())
        }
        Commands::MigratePaths {
            schema_dir,
            mappings_dir,
            dry_run,
        } => {
            eprintln!(
                "Migrating TOML paths in {:?} using schemas from {:?}{}",
                mappings_dir,
                schema_dir,
                if dry_run { " [dry-run]" } else { "" }
            );

            let stats = automapper_generator::codegen::path_migration::migrate_toml_dir(
                &schema_dir,
                &mappings_dir,
                dry_run,
            )?;

            eprintln!(
                "\n=== Migration {} ===",
                if dry_run { "Preview" } else { "Complete" }
            );
            eprintln!("  Files processed: {}", stats.files_processed);
            eprintln!("  Files changed:   {}", stats.files_changed);
            eprintln!("  Paths migrated:  {}", stats.paths_migrated);
            eprintln!("  Discriminators:  {}", stats.discriminators_migrated);
            if stats.unresolved > 0 {
                eprintln!("  Unresolved:      {}", stats.unresolved);
            }

            Ok(())
        }
        Commands::GenerateTypescript {
            pids,
            schema_dir,
            mappings_dir,
            format_version,
            message_type,
            output_dir,
        } => {
            let pid_list: Vec<&str> = pids.split(',').map(|s| s.trim()).collect();
            eprintln!(
                "Generating TypeScript types for PIDs: {:?} ({} {})",
                pid_list, message_type, format_version
            );

            let files = automapper_generator::codegen::typescript_gen::generate_typescript(
                &pid_list,
                &schema_dir,
                &mappings_dir,
                &format_version,
                &message_type,
                &output_dir,
            )?;

            eprintln!("\n=== TypeScript Generation Complete ===");
            for f in &files {
                eprintln!("  Generated: {}", f);
            }
            eprintln!("Output: {:?}", output_dir);
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

            let known_types =
                automapper_generator::validation::schema_validator::extract_bo4e_types(
                    &stammdatenmodell_path,
                )?;
            eprintln!("Found {} BO4E types in stammdatenmodell", known_types.len());

            let report =
                automapper_generator::validation::schema_validator::validate_generated_code(
                    &generated_dir,
                    &known_types,
                )?;

            eprintln!("Checked {} type references", report.total_references);

            if !report.errors.is_empty() {
                eprintln!("\n=== Errors ===");
                for error in &report.errors {
                    eprintln!("  ERROR: {}", error);
                }
            }

            if !report.warnings.is_empty() {
                eprintln!("\n=== Warnings ===");
                for warning in &report.warnings {
                    eprintln!("  WARN: {}", warning);
                }
            }

            if report.is_valid() {
                eprintln!("\nValidation PASSED");
            } else {
                eprintln!("\nValidation FAILED");
                return Err(automapper_generator::GeneratorError::Validation {
                    message: format!("{} errors found", report.errors.len()),
                });
            }

            Ok(())
        }
        Commands::GenerateFixture {
            pid_schema,
            output,
            validate,
            mig_xml,
            ahb_xml,
            message_type,
            variant,
            format_version,
            enhance,
            variants,
            seed,
        } => {
            eprintln!("Generating fixture from schema: {:?}", pid_schema);

            let schema_str = std::fs::read_to_string(&pid_schema)?;
            let schema: serde_json::Value = serde_json::from_str(&schema_str)?;

            let pid = schema["pid"].as_str().unwrap_or("unknown");
            let beschreibung = schema["beschreibung"].as_str().unwrap_or("");
            eprintln!("  PID: {} ({})", pid, beschreibung);

            if enhance {
                // Enhanced mode: roundtrip through BO4E mapping engine
                let mig_path =
                    mig_xml.ok_or_else(|| automapper_generator::GeneratorError::Validation {
                        message: "--mig-xml is required when --enhance is set".to_string(),
                    })?;
                let ahb_path =
                    ahb_xml.ok_or_else(|| automapper_generator::GeneratorError::Validation {
                        message: "--ahb-xml is required when --enhance is set".to_string(),
                    })?;
                let msg_type = message_type.as_deref().unwrap_or("UTILMD");
                let var = variant.as_deref();
                let fv = format_version.as_deref().unwrap_or("FV2504");

                // Parse MIG and AHB, filter for PID
                let mig = mig_assembly::parsing::parse_mig(&mig_path, msg_type, var, fv)?;
                let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(
                    &ahb_path, msg_type, var, fv,
                )?;
                let pid_def = ahb.workflows.iter().find(|w| w.id == pid).ok_or_else(|| {
                    automapper_generator::GeneratorError::Validation {
                        message: format!("PID {pid} not found in AHB"),
                    }
                })?;
                let ahb_numbers: std::collections::HashSet<String> =
                    pid_def.segment_numbers.iter().cloned().collect();
                let filtered_mig = mig_assembly::pid_filter::filter_mig_for_pid(&mig, &ahb_numbers);

                // Resolve mapping directories
                let fv_lower = fv.to_lowercase();
                let variant_suffix = var.unwrap_or("Strom");
                let mappings_base = format!("mappings/{fv}/{msg_type}_{variant_suffix}");
                let msg_dir = PathBuf::from(format!("{mappings_base}/message"));
                let tx_dir = PathBuf::from(format!("{mappings_base}/pid_{pid}"));

                if !msg_dir.exists() || !tx_dir.exists() {
                    eprintln!(
                        "  WARNING: mapping directories not found ({:?} or {:?})",
                        msg_dir, tx_dir
                    );
                    eprintln!("  Falling back to unenhanced fixture generation");

                    let edi = automapper_generator::fixture_generator::generate_fixture(&schema);
                    let seg_count = edi.matches('\'').count();
                    eprintln!("  Generated {} segments (unenhanced)", seg_count);

                    if let Some(parent) = output.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::write(&output, &edi)?;
                    eprintln!("  Output: {:?}", output);
                    return Ok(());
                }

                // Load PathResolver from schema directory
                let schema_dir = PathBuf::from(format!(
                    "crates/mig-types/src/generated/{fv_lower}/utilmd/pids"
                ));
                let path_resolver =
                    mig_bo4e::path_resolver::PathResolver::from_schema_dir(&schema_dir);

                // Load mapping engines
                let msg_engine = mig_bo4e::engine::MappingEngine::load(&msg_dir)?
                    .with_path_resolver(path_resolver.clone());
                let tx_engine = mig_bo4e::engine::MappingEngine::load(&tx_dir)?
                    .with_path_resolver(path_resolver);

                let variant_count = variants.unwrap_or(1);
                let base_seed = seed.unwrap_or(42);

                if variant_count <= 1 {
                    // Single variant: output directly to the specified path
                    let edi = automapper_generator::fixture_generator::generate_enhanced_fixture(
                        &schema,
                        &filtered_mig,
                        &msg_engine,
                        &tx_engine,
                        base_seed,
                        0,
                    )?;

                    let seg_count = edi.matches('\'').count();
                    eprintln!("  Generated {} segments (enhanced)", seg_count);

                    if let Some(parent) = output.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::write(&output, &edi)?;
                    eprintln!("  Output: {:?}", output);
                } else {
                    // Multiple variants: output to {stem}_v{i}.{ext}
                    let stem = output
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("fixture");
                    let ext = output.extension().and_then(|e| e.to_str()).unwrap_or("edi");
                    let parent = output.parent().unwrap_or_else(|| Path::new("."));

                    std::fs::create_dir_all(parent).ok();

                    for i in 0..variant_count {
                        let variant_seed = base_seed.wrapping_add(i as u64 * 1000);
                        let edi =
                            automapper_generator::fixture_generator::generate_enhanced_fixture(
                                &schema,
                                &filtered_mig,
                                &msg_engine,
                                &tx_engine,
                                variant_seed,
                                i,
                            )?;

                        let seg_count = edi.matches('\'').count();
                        let variant_path = parent.join(format!("{stem}_v{i}.{ext}"));
                        std::fs::write(&variant_path, &edi)?;
                        eprintln!(
                            "  Variant {i}: {} segments -> {:?}",
                            seg_count, variant_path
                        );
                    }
                }
            } else {
                // Unenhanced mode: generate basic fixture with placeholders
                let edi = automapper_generator::fixture_generator::generate_fixture(&schema);

                let seg_count = edi.matches('\'').count();
                eprintln!("  Generated {} segments", seg_count);

                if let Some(parent) = output.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&output, &edi)?;

                eprintln!("  Output: {:?}", output);

                if validate {
                    let mig_path = mig_xml.ok_or_else(|| {
                        automapper_generator::GeneratorError::Validation {
                            message: "--mig-xml is required when --validate is set".to_string(),
                        }
                    })?;
                    let ahb_path = ahb_xml.ok_or_else(|| {
                        automapper_generator::GeneratorError::Validation {
                            message: "--ahb-xml is required when --validate is set".to_string(),
                        }
                    })?;
                    let msg_type = message_type.as_deref().unwrap_or("UTILMD");
                    let var = variant.as_deref();
                    let fv = format_version.as_deref().unwrap_or("FV2504");

                    eprintln!("\nValidating fixture against MIG/AHB...");
                    let result = automapper_generator::fixture_generator::validate_fixture(
                        &edi, pid, &mig_path, &ahb_path, msg_type, var, fv,
                    )?;

                    eprintln!("  Tokenized: {} segments", result.segment_count);
                    eprintln!(
                        "  Assembled: {} segments in {} groups",
                        result.assembled_segment_count, result.assembled_group_count
                    );

                    for warning in &result.warnings {
                        eprintln!("  WARNING: {}", warning);
                    }
                    for error in &result.errors {
                        eprintln!("  ERROR: {}", error);
                    }

                    if result.is_ok() {
                        eprintln!("  Validation PASSED");
                    } else {
                        return Err(automapper_generator::GeneratorError::Validation {
                            message: format!("{} validation errors", result.errors.len()),
                        });
                    }
                }
            }

            Ok(())
        }
    }
}
