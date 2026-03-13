use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

mod compile_cache;

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

        /// Path to persistent enum index JSON (default: generated/code_enum_index.json)
        #[arg(long, default_value = "generated/code_enum_index.json")]
        enum_index: PathBuf,
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

    /// Generate stub condition evaluators from AHB XML (no AI, just parses conditions)
    GenerateConditionStubs {
        /// Output base directory (e.g., "crates/automapper-validation/src/generated")
        #[arg(long)]
        output_dir: PathBuf,
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

        /// Transaction group ID for enhanced fixture generation (e.g., SG4, SG29, SG5).
        /// Required for --enhance mode to correctly route through the mapping pipeline.
        #[arg(long, default_value = "SG4")]
        tx_group: Option<String>,
    },

    /// Extract all code lists from a MIG XML into a JSON file.
    /// Run once per format version; output is committed as generated artifact.
    ExtractCodeLists {
        /// Path to MIG XML file
        #[arg(long)]
        mig_path: PathBuf,

        /// Output JSON file path
        #[arg(long)]
        output: PathBuf,

        /// EDIFACT message type (e.g., "UTILMD")
        #[arg(long)]
        message_type: String,

        /// Format version (e.g., "FV2504")
        #[arg(long)]
        format_version: String,
    },

    /// Compile TOML mappings into cache files for fast loading
    CompileMappings {
        /// Base directory containing TOML mapping files (e.g., mappings/)
        #[arg(long, default_value = "mappings")]
        mappings_dir: PathBuf,

        /// Base directory containing generated PID schema files
        #[arg(long, default_value = "crates/mig-types/src/generated")]
        schema_dir: PathBuf,

        /// Output directory for cache files
        #[arg(long, default_value = "cache/mappings")]
        output_dir: PathBuf,
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

/// Configuration for one message type's condition generation across format versions.
struct ConditionStubConfig {
    /// Message type key (e.g., "UTILMD_Strom", "INVOIC")
    key: &'static str,
    /// Message type for parsing (e.g., "UTILMD")
    msg_type: &'static str,
    /// Variant (e.g., "Strom", "Gas", or "")
    variant: &'static str,
    /// Which format versions to generate (others are aliased)
    generate_fvs: &'static [&'static str],
    /// Alias map: (alias_fv, source_fv) — type alias in alias_fv points to source_fv
    aliases: &'static [(&'static str, &'static str)],
}

/// All message type configurations for stub generation.
const CONDITION_STUB_CONFIGS: &[ConditionStubConfig] = &[
    // Identical across all 3 FVs — generate FV2504 only
    ConditionStubConfig {
        key: "UTILMD_Strom",
        msg_type: "UTILMD",
        variant: "Strom",
        generate_fvs: &["FV2504"],
        aliases: &[("FV2510", "FV2504"), ("FV2604", "FV2504")],
    },
    ConditionStubConfig {
        key: "UTILMD_Gas",
        msg_type: "UTILMD",
        variant: "Gas",
        generate_fvs: &["FV2504"],
        aliases: &[("FV2510", "FV2504"), ("FV2604", "FV2504")],
    },
    ConditionStubConfig {
        key: "APERAK",
        msg_type: "APERAK",
        variant: "",
        generate_fvs: &["FV2504"],
        aliases: &[("FV2510", "FV2504"), ("FV2604", "FV2504")],
    },
    ConditionStubConfig {
        key: "CONTRL",
        msg_type: "CONTRL",
        variant: "",
        generate_fvs: &["FV2504"],
        aliases: &[("FV2510", "FV2504"), ("FV2604", "FV2504")],
    },
    ConditionStubConfig {
        key: "ORDCHG",
        msg_type: "ORDCHG",
        variant: "",
        generate_fvs: &["FV2504"],
        aliases: &[("FV2510", "FV2504"), ("FV2604", "FV2504")],
    },
    ConditionStubConfig {
        key: "UTILTS",
        msg_type: "UTILTS",
        variant: "",
        generate_fvs: &["FV2504"],
        aliases: &[("FV2510", "FV2504"), ("FV2604", "FV2504")],
    },
    // FV2504 differs from FV2510, but FV2510=FV2604
    ConditionStubConfig {
        key: "IFTSTA",
        msg_type: "IFTSTA",
        variant: "",
        generate_fvs: &["FV2504", "FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
    ConditionStubConfig {
        key: "INVOIC",
        msg_type: "INVOIC",
        variant: "",
        generate_fvs: &["FV2504", "FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
    ConditionStubConfig {
        key: "ORDRSP",
        msg_type: "ORDRSP",
        variant: "",
        generate_fvs: &["FV2504", "FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
    ConditionStubConfig {
        key: "PRICAT",
        msg_type: "PRICAT",
        variant: "",
        generate_fvs: &["FV2504", "FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
    ConditionStubConfig {
        key: "QUOTES",
        msg_type: "QUOTES",
        variant: "",
        generate_fvs: &["FV2504", "FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
    ConditionStubConfig {
        key: "REQOTE",
        msg_type: "REQOTE",
        variant: "",
        generate_fvs: &["FV2504", "FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
    // All differ or FV2510≠FV2604
    ConditionStubConfig {
        key: "COMDIS",
        msg_type: "COMDIS",
        variant: "",
        generate_fvs: &["FV2504", "FV2510", "FV2604"],
        aliases: &[],
    },
    ConditionStubConfig {
        key: "MSCONS",
        msg_type: "MSCONS",
        variant: "",
        generate_fvs: &["FV2504", "FV2510", "FV2604"],
        aliases: &[],
    },
    ConditionStubConfig {
        key: "ORDERS",
        msg_type: "ORDERS",
        variant: "",
        generate_fvs: &["FV2504", "FV2510", "FV2604"],
        aliases: &[],
    },
    ConditionStubConfig {
        key: "PARTIN",
        msg_type: "PARTIN",
        variant: "",
        generate_fvs: &["FV2504", "FV2510", "FV2604"],
        aliases: &[],
    },
    ConditionStubConfig {
        key: "REMADV",
        msg_type: "REMADV",
        variant: "",
        generate_fvs: &["FV2504", "FV2510", "FV2604"],
        aliases: &[],
    },
    // INSRPT: only FV2510+ (no FV2504 AHB)
    ConditionStubConfig {
        key: "INSRPT",
        msg_type: "INSRPT",
        variant: "",
        generate_fvs: &["FV2510"],
        aliases: &[("FV2604", "FV2510")],
    },
];

/// Generate stub condition evaluator files for all message types.
fn generate_condition_stubs(
    output_dir: &std::path::Path,
) -> Result<(), automapper_generator::GeneratorError> {
    use automapper_generator::parsing::ahb_parser;
    use std::collections::{HashMap, HashSet};

    let xml_base = std::path::Path::new("xml-migs-and-ahbs");

    // Track which files belong in each fv directory for mod.rs generation
    // fv -> Vec<(module_name, struct_name)>
    let mut fv_modules: HashMap<String, Vec<(String, String)>> = HashMap::new();
    // fv -> Vec<(alias_module_name, alias_struct_name, source_fv, source_module_name, source_struct_name)>
    let mut fv_aliases: HashMap<String, Vec<(String, String, String, String, String)>> =
        HashMap::new();

    for config in CONDITION_STUB_CONFIGS {
        let key_lower = config.key.to_lowercase();

        for &fv in config.generate_fvs {
            let fv_lower = fv.to_lowercase();
            let fv_dir = output_dir.join(&fv_lower);
            std::fs::create_dir_all(&fv_dir)?;

            // Find AHB XML
            let ahb_pattern = if config.variant.is_empty() {
                format!("{}_AHB", config.msg_type)
            } else {
                format!("{}_AHB_{}", config.msg_type, config.variant)
            };

            let ahb_dir = xml_base.join(fv);
            let ahb_path = std::fs::read_dir(&ahb_dir).ok().and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.file_name().to_string_lossy().starts_with(&ahb_pattern))
                    .map(|e| e.path())
            });

            let ahb_path = match ahb_path {
                Some(p) => p,
                None => {
                    eprintln!("SKIP: No AHB for {} {}", config.key, fv);
                    continue;
                }
            };

            let variant = if config.variant.is_empty() {
                None
            } else {
                Some(config.variant)
            };
            let ahb = ahb_parser::parse_ahb(&ahb_path, config.msg_type, variant, fv)?;

            // Extract and sort conditions
            let mut conditions: Vec<(u32, String)> = ahb
                .bedingungen
                .iter()
                .filter_map(|b| {
                    b.id.parse::<u32>()
                        .ok()
                        .map(|id| (id, b.description.clone()))
                })
                .collect();
            conditions.sort_by_key(|(id, _)| *id);

            // Determine external conditions (heuristics)
            let external_ids: HashSet<u32> = conditions
                .iter()
                .filter(|(_, desc)| {
                    let d = desc.to_lowercase();
                    d.contains("in der rolle")
                        || d.contains("aufteilung vorhanden")
                        || d.contains("datenclearing")
                        || d.contains("datum bekannt")
                        || d.contains("korrektur erfolgt")
                        || d.contains("befristet")
                        || d.contains("kapitel 6")
                        || d.contains("codeliste der konfigurationen")
                })
                .map(|(id, _)| *id)
                .collect();

            let struct_name = format!("{}ConditionEvaluator{}", to_pascal_case(config.key), fv);
            let module_name = format!("{}_conditions_{}", key_lower, fv_lower);
            let output_file = fv_dir.join(format!("{}.rs", module_name));

            // Generate the stub file
            let mut code = String::new();
            code.push_str(&format!(
                "// <auto-generated>\n// Stub condition evaluator for {} {}.\n// Source AHB: {}\n// {} conditions extracted. Implementations pending — all return Unknown.\n// </auto-generated>\n\n",
                config.key, fv,
                ahb_path.file_name().unwrap_or_default().to_string_lossy(),
                conditions.len()
            ));
            code.push_str(
                "use crate::eval::{ConditionEvaluator, ConditionResult, EvaluationContext};\n\n",
            );

            let mut sorted_ext: Vec<u32> = external_ids.iter().copied().collect();
            sorted_ext.sort();

            // Struct definition (with derive(Default) when no external conditions)
            if sorted_ext.is_empty() {
                code.push_str(&format!("/// Condition evaluator for {} {}.\n#[derive(Default)]\npub struct {} {{\n    external_conditions: std::collections::HashSet<u32>,\n}}\n\n", config.key, fv, struct_name));
            } else {
                code.push_str(&format!("/// Condition evaluator for {} {}.\npub struct {} {{\n    external_conditions: std::collections::HashSet<u32>,\n}}\n\n", config.key, fv, struct_name));
                // Manual Default impl with pre-populated externals
                code.push_str(&format!("impl Default for {} {{\n    fn default() -> Self {{\n        let mut external_conditions = std::collections::HashSet::new();\n", struct_name));
                for id in &sorted_ext {
                    code.push_str(&format!("        external_conditions.insert({});\n", id));
                }
                code.push_str("        Self { external_conditions }\n    }\n}\n\n");
            }

            // ConditionEvaluator impl
            code.push_str(&format!(
                "impl ConditionEvaluator for {} {{\n    fn message_type(&self) -> &str {{ \"{}\" }}\n    fn format_version(&self) -> &str {{ \"{}\" }}\n\n",
                struct_name, config.key, fv
            ));
            code.push_str("    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult {\n        match condition {\n");
            for (id, _) in &conditions {
                code.push_str(&format!(
                    "            {} => self.evaluate_{}(ctx),\n",
                    id, id
                ));
            }
            code.push_str("            _ => ConditionResult::Unknown,\n        }\n    }\n\n");
            code.push_str("    fn is_external(&self, condition: u32) -> bool {\n        self.external_conditions.contains(&condition)\n    }\n}\n\n");

            // Individual methods
            code.push_str(&format!("impl {} {{\n", struct_name));
            for (id, desc) in &conditions {
                // Sanitize description for doc comment: single line, no special chars
                let safe_desc = desc
                    .replace('\n', " ")
                    .replace('\r', "")
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"");
                // Truncate very long descriptions
                let truncated = if safe_desc.len() > 200 {
                    format!("{}...", &safe_desc[..197])
                } else {
                    safe_desc
                };
                code.push_str(&format!("    /// [{}] {}\n", id, truncated.trim()));
                if external_ids.contains(id) {
                    let ext_name = derive_external_name(desc);
                    code.push_str(&format!(
                        "    fn evaluate_{}(&self, ctx: &EvaluationContext) -> ConditionResult {{\n        ctx.external.evaluate(\"{}\")\n    }}\n\n",
                        id, ext_name
                    ));
                } else {
                    code.push_str(&format!(
                        "    fn evaluate_{}(&self, _ctx: &EvaluationContext) -> ConditionResult {{\n        // TODO: implement\n        ConditionResult::Unknown\n    }}\n\n",
                        id
                    ));
                }
            }
            code.push_str("}\n");

            std::fs::write(&output_file, &code)?;

            // Run rustfmt
            let _ = std::process::Command::new("rustfmt")
                .arg(&output_file)
                .status();

            eprintln!(
                "Generated {} ({} conditions, {} external) → {}",
                struct_name,
                conditions.len(),
                external_ids.len(),
                output_file.display()
            );

            fv_modules
                .entry(fv_lower.clone())
                .or_default()
                .push((module_name, struct_name));
        }

        // Record aliases
        for &(alias_fv, source_fv) in config.aliases {
            let alias_fv_lower = alias_fv.to_lowercase();
            let source_fv_lower = source_fv.to_lowercase();
            let struct_name_alias = format!(
                "{}ConditionEvaluator{}",
                to_pascal_case(config.key),
                alias_fv
            );
            let struct_name_source = format!(
                "{}ConditionEvaluator{}",
                to_pascal_case(config.key),
                source_fv
            );
            let source_module = format!("{}_conditions_{}", key_lower, source_fv_lower);

            fv_aliases.entry(alias_fv_lower).or_default().push((
                format!("{}_conditions_{}", key_lower, alias_fv.to_lowercase()),
                struct_name_alias,
                source_fv_lower,
                source_module,
                struct_name_source,
            ));
        }
    }

    // Generate mod.rs for each FV directory
    let mut all_fvs: Vec<String> = fv_modules
        .keys()
        .chain(fv_aliases.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_fvs.sort();

    for fv_lower in &all_fvs {
        let fv_dir = output_dir.join(fv_lower);
        std::fs::create_dir_all(&fv_dir)?;

        let mut mod_code = String::new();

        // Real modules
        if let Some(modules) = fv_modules.get(fv_lower) {
            for (module_name, struct_name) in modules {
                mod_code.push_str(&format!(
                    "mod {};\npub use {}::{};\n\n",
                    module_name, module_name, struct_name
                ));
            }
        }

        // Aliases from other FV directories
        if let Some(aliases) = fv_aliases.get(fv_lower) {
            for (_, alias_struct, source_fv, _source_mod, source_struct) in aliases {
                mod_code.push_str(&format!(
                    "/// Alias: {} conditions are identical to {}.\npub type {} = super::{}::{};\n\n",
                    fv_lower.to_uppercase(),
                    source_fv.to_uppercase(),
                    alias_struct, source_fv, source_struct
                ));
            }
        }

        let mod_path = fv_dir.join("mod.rs");
        std::fs::write(&mod_path, &mod_code)?;
        eprintln!("Wrote {}", mod_path.display());
    }

    // Generate top-level mod.rs
    let mut top_mod = String::new();
    for fv_lower in &all_fvs {
        top_mod.push_str(&format!("pub mod {};\n", fv_lower));
    }
    top_mod.push('\n');

    // Re-export all structs
    for fv_lower in &all_fvs {
        if let Some(modules) = fv_modules.get(fv_lower) {
            for (_, struct_name) in modules {
                top_mod.push_str(&format!("pub use {}::{};\n", fv_lower, struct_name));
            }
        }
        if let Some(aliases) = fv_aliases.get(fv_lower) {
            for (_, alias_struct, _, _, _) in aliases {
                top_mod.push_str(&format!("pub use {}::{};\n", fv_lower, alias_struct));
            }
        }
    }

    let top_mod_path = output_dir.join("mod.rs");
    std::fs::write(&top_mod_path, &top_mod)?;
    eprintln!("Wrote {}", top_mod_path.display());

    eprintln!("\nDone! Run `cargo clippy -p automapper-validation -- -D warnings` to verify.");
    Ok(())
}

/// Derive a snake_case external condition name from a German description.
fn derive_external_name(description: &str) -> String {
    let d = description.to_lowercase();
    if d.contains("aufteilung vorhanden") {
        "message_splitting".to_string()
    } else if d.contains("datenclearing") {
        "data_clearing_required".to_string()
    } else if d.contains("datum bekannt") {
        "date_known".to_string()
    } else if d.contains("korrektur erfolgt") || d.contains("korrektur/storno") {
        "correction_in_progress".to_string()
    } else if d.contains("befristet") {
        "registration_is_time_limited".to_string()
    } else if d.contains("nad+mr") && d.contains("rolle lf") {
        "recipient_is_lf".to_string()
    } else if d.contains("nad+ms") && d.contains("rolle lf") {
        "sender_is_lf".to_string()
    } else if d.contains("nad+mr") && d.contains("rolle") {
        "recipient_role_check".to_string()
    } else if d.contains("nad+ms") && d.contains("rolle") {
        "sender_role_check".to_string()
    } else if d.contains("in der rolle") {
        "market_participant_role_check".to_string()
    } else if d.contains("kapitel 6") || d.contains("codeliste der konfigurationen") {
        "code_list_membership_check".to_string()
    } else {
        // Generate from first few words
        let words: Vec<&str> = description.split_whitespace().take(5).collect();
        words
            .join("_")
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "")
    }
}

/// Convert a key like "UTILMD_Strom" to PascalCase "UtilmdStrom".
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper = c.to_uppercase().to_string();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
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
            enum_index,
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
                Some(&enum_index),
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
        Commands::GenerateConditionStubs { output_dir } => {
            generate_condition_stubs(&output_dir)?;
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
            tx_group,
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
                let msg_type_lower = msg_type.to_lowercase();
                let mappings_base = if let Some(v) = var.filter(|v| !v.is_empty()) {
                    format!("mappings/{fv}/{msg_type}_{v}")
                } else {
                    format!("mappings/{fv}/{msg_type}")
                };
                let msg_dir = PathBuf::from(format!("{mappings_base}/message"));
                let tx_dir = PathBuf::from(format!("{mappings_base}/pid_{pid}"));

                if !msg_dir.exists() {
                    eprintln!(
                        "  WARNING: message mapping directory not found ({:?})",
                        msg_dir
                    );
                    eprintln!("  Falling back to unenhanced fixture generation");

                    let tg = tx_group.as_deref().unwrap_or("SG4");
                    let mig_order = automapper_generator::fixture_generator::build_mig_group_order(
                        &filtered_mig,
                        tg,
                    );
                    let edi = automapper_generator::fixture_generator::generate_fixture(
                        &schema,
                        Some(&mig_order),
                    );
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
                    "crates/mig-types/src/generated/{fv_lower}/{msg_type_lower}/pids"
                ));
                let path_resolver =
                    mig_bo4e::path_resolver::PathResolver::from_schema_dir(&schema_dir);

                // Load mapping engines (with common/ inheritance when available)
                let common_dir = PathBuf::from(format!("{mappings_base}/common"));
                let (msg_engine, tx_engine) = if common_dir.is_dir() && tx_dir.exists() {
                    let schema_file = schema_dir.join(format!("pid_{pid}_schema.json"));
                    if let Ok(idx) =
                        mig_bo4e::pid_schema_index::PidSchemaIndex::from_schema_file(&schema_file)
                    {
                        let (m, t) = mig_bo4e::engine::MappingEngine::load_split_with_common(
                            &msg_dir,
                            &common_dir,
                            &tx_dir,
                            &idx,
                        )?;
                        (
                            m.with_path_resolver(path_resolver.clone()),
                            t.with_path_resolver(path_resolver),
                        )
                    } else {
                        let m = mig_bo4e::engine::MappingEngine::load(&msg_dir)?
                            .with_path_resolver(path_resolver.clone());
                        let t = mig_bo4e::engine::MappingEngine::load(&tx_dir)?
                            .with_path_resolver(path_resolver);
                        (m, t)
                    }
                } else if common_dir.is_dir() {
                    // common/ exists but no per-PID dir: load common filtered by schema
                    let m = mig_bo4e::engine::MappingEngine::load(&msg_dir)?
                        .with_path_resolver(path_resolver.clone());
                    let schema_file = schema_dir.join(format!("pid_{pid}_schema.json"));
                    let t = if let Ok(idx) =
                        mig_bo4e::pid_schema_index::PidSchemaIndex::from_schema_file(&schema_file)
                    {
                        let common_engine = mig_bo4e::engine::MappingEngine::load(&common_dir)?;
                        let mut common_defs = common_engine.definitions().to_vec();
                        common_defs.retain(|d| {
                            d.meta
                                .source_path
                                .as_deref()
                                .map(|sp| idx.has_group(sp))
                                .unwrap_or(true)
                        });
                        mig_bo4e::engine::MappingEngine::from_definitions(common_defs)
                            .with_path_resolver(path_resolver)
                    } else {
                        mig_bo4e::engine::MappingEngine::load(&common_dir)?
                            .with_path_resolver(path_resolver)
                    };
                    (m, t)
                } else if tx_dir.exists() {
                    let m = mig_bo4e::engine::MappingEngine::load(&msg_dir)?
                        .with_path_resolver(path_resolver.clone());
                    let t = mig_bo4e::engine::MappingEngine::load(&tx_dir)?
                        .with_path_resolver(path_resolver);
                    (m, t)
                } else {
                    // Message-only: no common/ and no per-PID dir
                    let m = mig_bo4e::engine::MappingEngine::load(&msg_dir)?
                        .with_path_resolver(path_resolver.clone());
                    let t = mig_bo4e::engine::MappingEngine::from_definitions(vec![])
                        .with_path_resolver(path_resolver);
                    (m, t)
                };

                let variant_count = variants.unwrap_or(1);
                let base_seed = seed.unwrap_or(42);
                let tg = tx_group.as_deref().unwrap_or("SG4");

                if variant_count <= 1 {
                    // Single variant: output directly to the specified path
                    let edi = automapper_generator::fixture_generator::generate_enhanced_fixture(
                        &schema,
                        &filtered_mig,
                        &msg_engine,
                        &tx_engine,
                        base_seed,
                        0,
                        tg,
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
                                tg,
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
                let edi = automapper_generator::fixture_generator::generate_fixture(&schema, None);

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
        Commands::ExtractCodeLists {
            mig_path,
            output,
            message_type,
            format_version,
        } => {
            let mig_filename = mig_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let variant = infer_variant(mig_filename);
            let mig = automapper_generator::parsing::mig_parser::parse_mig(
                &mig_path,
                &message_type,
                variant,
                &format_version,
            )?;
            let code_lists =
                automapper_generator::codegen::code_list_extractor::extract_code_lists(&mig);
            automapper_generator::codegen::code_list_extractor::write_code_lists(
                &code_lists,
                &output,
            )?;
            eprintln!(
                "Extracted {} code lists ({} total codes) to {}",
                code_lists.len(),
                code_lists.values().map(|e| e.codes.len()).sum::<usize>(),
                output.display()
            );
            Ok(())
        }
        Commands::CompileMappings {
            mappings_dir,
            schema_dir,
            output_dir,
        } => {
            eprintln!(
                "Compiling TOML mappings from {:?} to {:?}",
                mappings_dir, output_dir
            );
            let stats = compile_cache::compile_all(&mappings_dir, &schema_dir, &output_dir)
                .map_err(|e| automapper_generator::GeneratorError::Validation {
                    message: e.to_string(),
                })?;

            eprintln!("\n=== Compilation Complete ===");
            eprintln!("  Message engines:     {}", stats.message_engines);
            eprintln!("  Transaction engines:  {}", stats.transaction_engines);
            eprintln!("  Combined engines:     {}", stats.combined_engines);

            if !stats.errors.is_empty() {
                eprintln!("\n  Errors ({}):", stats.errors.len());
                for err in &stats.errors {
                    eprintln!("    - {}", err);
                }
            }

            eprintln!("Output: {:?}", output_dir);
            Ok(())
        }
    }
}
