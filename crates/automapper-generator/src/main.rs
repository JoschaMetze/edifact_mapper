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

        /// Only regenerate conditions that changed or are low-confidence
        #[arg(long, default_value = "false")]
        incremental: bool,

        /// Maximum concurrent Claude CLI calls
        #[arg(long, default_value = "4")]
        max_concurrent: usize,

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
            max_concurrent,
            mig_path,
            batch_size,
            dry_run,
        } => {
            eprintln!(
                "Generating conditions for {} {} (incremental={})",
                message_type, format_version, incremental
            );

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

            // Determine what needs regeneration
            let existing_ids = std::collections::HashSet::new();
            let decision = automapper_generator::conditions::metadata::decide_regeneration(
                &conditions,
                existing_metadata.as_ref(),
                &existing_ids,
                !incremental,
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
            let condition_inputs: Vec<
                automapper_generator::conditions::condition_types::ConditionInput,
            > = decision
                .to_regenerate
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
                eprintln!(
                    "[Batch {}/{}] Processing {} conditions...",
                    i + 1,
                    condition_inputs.len().div_ceil(batch_size),
                    batch.len()
                );

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
                            "  Generated: {} (High: {}, Medium: {}, Low: {})",
                            generated.len(),
                            high,
                            medium,
                            low
                        );
                        all_generated.extend(generated);
                    }
                    Err(e) => {
                        eprintln!("  ERROR: {}", e);
                    }
                }
            }

            // Generate output file
            let output_path = output_dir.join(format!(
                "{}_conditions_{}.rs",
                message_type.to_lowercase(),
                format_version.to_lowercase()
            ));

            let preserved = std::collections::HashMap::new();
            let source_code =
                automapper_generator::conditions::codegen::generate_condition_evaluator_file(
                    &message_type,
                    &format_version,
                    &all_generated,
                    ahb_path.to_str().unwrap_or("unknown"),
                    &preserved,
                );

            std::fs::write(&output_path, &source_code)?;
            eprintln!("Generated: {:?}", output_path);

            // Save metadata
            let mut meta_conditions = std::collections::HashMap::new();
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
    }
}
