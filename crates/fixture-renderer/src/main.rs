use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use fixture_renderer::{generate_canonical_bo4e, render_fixture, RenderInput, RendererError};

#[derive(Parser)]
#[command(name = "fixture-renderer")]
#[command(about = "Render EDIFACT fixtures from BO4E via TOML mappings")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render an EDIFACT fixture from a source .edi through forward+reverse mapping.
    /// Phase 3: validates TOML mappings can produce complete EDIFACT output.
    RenderFixture {
        /// Path to the source .edi fixture.
        #[arg(long)]
        source: PathBuf,

        /// Path to the MIG XML file.
        #[arg(long)]
        mig_xml: PathBuf,

        /// Path to the AHB XML file.
        #[arg(long)]
        ahb_xml: PathBuf,

        /// Path to message-level TOML mappings directory.
        #[arg(long)]
        message_mappings: PathBuf,

        /// Path to transaction-level TOML mappings directory.
        #[arg(long)]
        transaction_mappings: PathBuf,

        /// Message type (e.g., "UTILMD").
        #[arg(long)]
        message_type: String,

        /// Message variant (e.g., "Strom", "Gas").
        #[arg(long)]
        variant: Option<String>,

        /// Format version (e.g., "FV2504").
        #[arg(long)]
        format_version: String,

        /// PID identifier (e.g., "55001").
        #[arg(long)]
        pid: String,

        /// Output path for the rendered .edi file.
        #[arg(long)]
        output: PathBuf,
    },

    /// Generate a canonical .mig.bo.json from an existing .edi fixture.
    /// Bootstraps the version-independent test corpus.
    GenerateCanonicalBo4e {
        /// Path to the source .edi fixture.
        #[arg(long)]
        source: PathBuf,

        /// Path to the MIG XML file.
        #[arg(long)]
        mig_xml: PathBuf,

        /// Path to the AHB XML file.
        #[arg(long)]
        ahb_xml: PathBuf,

        /// Path to message-level TOML mappings directory.
        #[arg(long)]
        message_mappings: PathBuf,

        /// Path to transaction-level TOML mappings directory.
        #[arg(long)]
        transaction_mappings: PathBuf,

        /// Message type (e.g., "UTILMD").
        #[arg(long)]
        message_type: String,

        /// Message variant (e.g., "Strom").
        #[arg(long)]
        variant: Option<String>,

        /// Format version (e.g., "FV2504").
        #[arg(long)]
        format_version: String,

        /// PID identifier (e.g., "55001").
        #[arg(long)]
        pid: String,

        /// Output path for the .mig.bo.json file. If omitted, writes alongside the source .edi.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), RendererError> {
    match cli.command {
        Commands::RenderFixture {
            source,
            mig_xml,
            ahb_xml,
            message_mappings,
            transaction_mappings,
            message_type,
            variant,
            format_version,
            pid,
            output,
        } => {
            let input = RenderInput {
                mig_xml_path: mig_xml,
                ahb_xml_path: ahb_xml,
                message_mappings_dir: message_mappings,
                transaction_mappings_dir: transaction_mappings,
                message_type,
                variant,
                format_version,
                pid: pid.clone(),
            };

            let edifact = render_fixture(&source, &input)?;

            std::fs::create_dir_all(output.parent().unwrap_or(Path::new(".")))?;
            std::fs::write(&output, &edifact)?;

            println!(
                "Rendered fixture: {} ({} bytes)",
                output.display(),
                edifact.len()
            );
            println!("PID: {}", pid);
            Ok(())
        }

        Commands::GenerateCanonicalBo4e {
            source,
            mig_xml,
            ahb_xml,
            message_mappings,
            transaction_mappings,
            message_type,
            variant,
            format_version,
            pid,
            output,
        } => {
            let input = RenderInput {
                mig_xml_path: mig_xml,
                ahb_xml_path: ahb_xml,
                message_mappings_dir: message_mappings,
                transaction_mappings_dir: transaction_mappings,
                message_type,
                variant,
                format_version,
                pid: pid.clone(),
            };

            let canonical = generate_canonical_bo4e(&source, &input)?;
            let json = serde_json::to_string_pretty(&canonical)?;

            let output_path = output.unwrap_or_else(|| source.with_extension("mig.bo.json"));

            std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
            std::fs::write(&output_path, &json)?;

            println!(
                "Wrote canonical BO4E: {} ({} bytes)",
                output_path.display(),
                json.len()
            );
            println!("PID: {}", pid);
            Ok(())
        }
    }
}
