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
            ahb_path: _,
            output_dir: _,
            format_version,
            message_type,
            incremental,
            concurrency: _,
            mig_path: _,
            batch_size: _,
            dry_run: _,
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
