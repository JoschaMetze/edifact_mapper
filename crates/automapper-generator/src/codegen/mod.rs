pub mod coordinator_gen;
pub mod mapper_gen;
pub mod mig_type_gen;
pub mod segment_order;
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
    let mig_filename = mig_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
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
    eprintln!("Extracted {} ordered segments", ordered_segments.len());

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
    let coord_output = coordinator_gen::generate_coordinator(&mig_schema, &ordered_segments)?;
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
