use super::migrator::migrate_fixture;
use super::types::MigrationResult;
use crate::schema_diff::types::PidSchemaDiff;
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
