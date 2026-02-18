use std::collections::HashSet;
use std::path::Path;

use regex::Regex;

use crate::error::GeneratorError;

/// Result of schema validation.
#[derive(Debug, Clone)]
pub struct SchemaValidationReport {
    /// Errors that indicate invalid references.
    pub errors: Vec<SchemaValidationIssue>,
    /// Warnings that might indicate problems.
    pub warnings: Vec<SchemaValidationIssue>,
    /// Total number of type references checked.
    pub total_references: usize,
}

impl SchemaValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// A single validation issue.
#[derive(Debug, Clone)]
pub struct SchemaValidationIssue {
    /// The file where the issue was found.
    pub file: String,
    /// The line number (1-based).
    pub line: usize,
    /// Description of the issue.
    pub message: String,
}

impl std::fmt::Display for SchemaValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

/// Extracts BO4E type names from the stammdatenmodell directory.
///
/// Scans for Rust struct/enum definitions in .rs files, or JSON schema files.
pub fn extract_bo4e_types(stammdatenmodell_path: &Path) -> Result<HashSet<String>, GeneratorError> {
    let mut types = HashSet::new();

    if !stammdatenmodell_path.exists() {
        return Err(GeneratorError::FileNotFound(
            stammdatenmodell_path.to_path_buf(),
        ));
    }

    // Look for JSON schema files (*.json)
    let json_pattern = stammdatenmodell_path.join("**/*.json");
    for path in glob::glob(json_pattern.to_str().unwrap_or(""))
        .map_err(|e| GeneratorError::Io(std::io::Error::other(e.to_string())))?
        .flatten()
    {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            types.insert(stem.to_string());
        }
    }

    // Look for Rust type definitions (*.rs) with `pub struct` or `pub enum`
    let rs_pattern = stammdatenmodell_path.join("**/*.rs");
    let type_regex = Regex::new(r"pub\s+(?:struct|enum)\s+(\w+)").unwrap();

    for path in glob::glob(rs_pattern.to_str().unwrap_or(""))
        .map_err(|e| GeneratorError::Io(std::io::Error::other(e.to_string())))?
        .flatten()
    {
        let content = std::fs::read_to_string(&path)?;
        for cap in type_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                types.insert(name.as_str().to_string());
            }
        }
    }

    Ok(types)
}

/// Validates that generated Rust mapper code references valid BO4E types.
///
/// Scans generated files for type references (use statements, struct fields)
/// and checks them against the known BO4E types from stammdatenmodell.
pub fn validate_generated_code(
    generated_dir: &Path,
    known_types: &HashSet<String>,
) -> Result<SchemaValidationReport, GeneratorError> {
    let errors = Vec::new();
    let mut warnings = Vec::new();
    let mut total_references = 0;

    if !generated_dir.exists() {
        return Err(GeneratorError::FileNotFound(generated_dir.to_path_buf()));
    }

    // Known types from the crate ecosystem (not from stammdatenmodell)
    let internal_types: HashSet<&str> = [
        "ConditionResult",
        "ConditionEvaluator",
        "EvaluationContext",
        "ExternalConditionProvider",
        "RawSegment",
        "SegmentHandler",
        "Builder",
        "EntityWriter",
        "Mapper",
        "FormatVersion",
        "TransactionContext",
        "EdifactSegmentWriter",
        "VersionConfig",
        "WithValidity",
        "UtilmdTransaktion",
        "UtilmdNachricht",
        "Prozessdaten",
        "Zeitscheibe",
        "Nachrichtendaten",
        "Marktteilnehmer",
        "Antwortstatus",
        "String",
        "Vec",
        "Option",
        "HashMap",
        "HashSet",
        "bool",
        "u32",
        "i32",
        "f32",
        "f64",
        "usize",
        "Self",
    ]
    .into_iter()
    .collect();

    // Regex for BO4E type references in generated code (CamelCase or PascalCase single words)
    let bo4e_ref_regex = Regex::new(r"(?:bo4e::)?(\b[A-Z][a-z][a-zA-Z]*\b)").unwrap();

    // Scan all .rs files in generated_dir
    let pattern = generated_dir.join("*.rs");
    for path in glob::glob(pattern.to_str().unwrap_or(""))
        .map_err(|e| GeneratorError::Io(std::io::Error::other(e.to_string())))?
        .flatten()
    {
        let content = std::fs::read_to_string(&path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        for (line_num, line) in content.lines().enumerate() {
            // Skip comments and auto-generated headers
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            for cap in bo4e_ref_regex.captures_iter(line) {
                if let Some(type_name) = cap.get(1) {
                    let name = type_name.as_str();

                    // Skip internal/framework types
                    if internal_types.contains(name) {
                        continue;
                    }

                    // Skip types ending with "Edifact" (companion types from our crate)
                    if name.ends_with("Edifact") {
                        continue;
                    }

                    total_references += 1;

                    // Check if the type is in the BO4E schema
                    if !known_types.contains(name) {
                        warnings.push(SchemaValidationIssue {
                            file: filename.to_string(),
                            line: line_num + 1,
                            message: format!("type '{}' not found in stammdatenmodell", name),
                        });
                    }
                }
            }
        }
    }

    Ok(SchemaValidationReport {
        errors,
        warnings,
        total_references,
    })
}
