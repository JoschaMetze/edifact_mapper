//! Compile TOML mapping definitions into pre-resolved cache files.
//!
//! Mirrors the loading logic in `automapper-api/src/state.rs` but writes
//! `.bin` (JSON) cache files instead of keeping engines in memory.

use std::path::Path;

use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_bo4e::MappingEngine;

/// Statistics from a cache compilation run.
#[derive(Debug, Default)]
pub struct CompileStats {
    pub message_engines: usize,
    pub transaction_engines: usize,
    pub combined_engines: usize,
    pub errors: Vec<String>,
}

impl CompileStats {
    fn merge(&mut self, other: CompileStats) {
        self.message_engines += other.message_engines;
        self.transaction_engines += other.transaction_engines;
        self.combined_engines += other.combined_engines;
        self.errors.extend(other.errors);
    }
}

/// Walk all `{mappings_base}/{FV}/{variant}/` directories and compile
/// each into cache files under `{output_base}/{FV}/{variant}/`.
pub fn compile_all(
    mappings_base: &Path,
    schema_base: &Path,
    output_base: &Path,
) -> Result<CompileStats, Box<dyn std::error::Error>> {
    let mut stats = CompileStats::default();

    let fv_entries = std::fs::read_dir(mappings_base)?;
    for fv_entry in fv_entries.flatten() {
        let fv_path = fv_entry.path();
        if !fv_path.is_dir() {
            continue;
        }
        let fv = fv_entry.file_name().to_string_lossy().to_string();

        let variant_entries = std::fs::read_dir(&fv_path)?;
        for variant_entry in variant_entries.flatten() {
            let variant_path = variant_entry.path();
            if !variant_path.is_dir() {
                continue;
            }
            let variant = variant_entry.file_name().to_string_lossy().to_string();

            match compile_variant(mappings_base, schema_base, &fv, &variant, output_base) {
                Ok(variant_stats) => stats.merge(variant_stats),
                Err(e) => {
                    stats
                        .errors
                        .push(format!("{fv}/{variant}: {e}"));
                }
            }
        }
    }

    Ok(stats)
}

/// Compile a single format-version/variant pair.
///
/// Produces:
/// - `{output}/{fv}/{variant}/msg.bin` — message-level engine
/// - `{output}/{fv}/{variant}/tx_pid_NNNNN.bin` — transaction engine per PID
/// - `{output}/{fv}/{variant}/combined_pid_NNNNN.bin` — combined engine per PID
pub fn compile_variant(
    mappings_base: &Path,
    schema_base: &Path,
    fv: &str,
    variant: &str,
    output_base: &Path,
) -> Result<CompileStats, Box<dyn std::error::Error>> {
    let mut stats = CompileStats::default();
    let variant_path = mappings_base.join(fv).join(variant);
    let output_dir = output_base.join(fv).join(variant);

    // Build PathResolver from schema dir
    let msg_type = variant.split('_').next().unwrap_or(variant).to_lowercase();
    let fv_lower = fv.to_lowercase();
    let schema_dir = schema_base.join(&fv_lower).join(&msg_type).join("pids");
    let resolver = if schema_dir.is_dir() {
        Some(PathResolver::from_schema_dir(&schema_dir))
    } else {
        None
    };

    // Load and cache message engine
    let message_dir = variant_path.join("message");
    if message_dir.is_dir() {
        match load_and_resolve(&message_dir, resolver.as_ref()) {
            Ok(engine) => {
                let cache_path = output_dir.join("msg.bin");
                engine.save_cached(&cache_path)?;
                stats.message_engines += 1;
            }
            Err(e) => {
                stats
                    .errors
                    .push(format!("{fv}/{variant}/message: {e}"));
            }
        }
    }

    // Iterate PID dirs
    let common_dir = variant_path.join("common");
    let pid_entries = std::fs::read_dir(&variant_path)?;
    for pid_entry in pid_entries.flatten() {
        let pid_path = pid_entry.path();
        if !pid_path.is_dir() {
            continue;
        }
        let dirname = pid_entry.file_name().to_string_lossy().to_string();
        if dirname == "message" || dirname == "common" {
            continue;
        }

        let pid_num = dirname.strip_prefix("pid_").unwrap_or(&dirname);

        // Load transaction engine (with common inheritance)
        let tx_result = load_tx_engine(&common_dir, &pid_path, &schema_dir, pid_num);
        match tx_result {
            Ok(tx_engine) => {
                let tx_engine = apply_resolver(tx_engine, resolver.as_ref());
                let tx_path = output_dir.join(format!("tx_{dirname}.bin"));
                if let Err(e) = tx_engine.save_cached(&tx_path) {
                    stats
                        .errors
                        .push(format!("{fv}/{variant}/{dirname} tx: {e}"));
                } else {
                    stats.transaction_engines += 1;
                }

                // Build combined engine (message + tx)
                if message_dir.is_dir() {
                    match MappingEngine::load(&message_dir) {
                        Ok(msg_engine) => {
                            let msg_engine = apply_resolver(msg_engine, resolver.as_ref());
                            let mut combined_defs = msg_engine.definitions().to_vec();
                            combined_defs.extend(tx_engine.definitions().to_vec());
                            let combined = MappingEngine::from_definitions(combined_defs);
                            let combined_path =
                                output_dir.join(format!("combined_{dirname}.bin"));
                            if let Err(e) = combined.save_cached(&combined_path) {
                                stats
                                    .errors
                                    .push(format!("{fv}/{variant}/{dirname} combined: {e}"));
                            } else {
                                stats.combined_engines += 1;
                            }
                        }
                        Err(e) => {
                            stats
                                .errors
                                .push(format!("{fv}/{variant}/{dirname} combined (msg load): {e}"));
                        }
                    }
                }
            }
            Err(e) => {
                stats
                    .errors
                    .push(format!("{fv}/{variant}/{dirname} tx: {e}"));
            }
        }
    }

    Ok(stats)
}

/// Load a MappingEngine from a TOML dir and apply the PathResolver.
fn load_and_resolve(
    dir: &Path,
    resolver: Option<&PathResolver>,
) -> Result<MappingEngine, mig_bo4e::error::MappingError> {
    let engine = MappingEngine::load(dir)?;
    Ok(apply_resolver(engine, resolver))
}

/// Apply PathResolver if available.
fn apply_resolver(engine: MappingEngine, resolver: Option<&PathResolver>) -> MappingEngine {
    if let Some(r) = resolver {
        engine.with_path_resolver(r.clone())
    } else {
        engine
    }
}

/// Load a transaction engine with optional common/ inheritance.
fn load_tx_engine(
    common_dir: &Path,
    pid_dir: &Path,
    schema_dir: &Path,
    pid_num: &str,
) -> Result<MappingEngine, mig_bo4e::error::MappingError> {
    if common_dir.is_dir() {
        let schema_file = schema_dir.join(format!("pid_{pid_num}_schema.json"));
        if let Ok(idx) = PidSchemaIndex::from_schema_file(&schema_file) {
            return MappingEngine::load_with_common(common_dir, pid_dir, &idx);
        }
    }
    MappingEngine::load(pid_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_utilmd_strom_fv2504() {
        let mappings_base = Path::new("../../mappings");
        let schema_base = Path::new("../../crates/mig-types/src/generated");

        // Only run if mappings exist (CI may not have them)
        let variant_dir = mappings_base.join("FV2504/UTILMD_Strom");
        if !variant_dir.exists() {
            eprintln!("Skipping: mappings not found at {:?}", variant_dir);
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path();

        let stats = compile_all(mappings_base, schema_base, output).unwrap();

        eprintln!("Stats: {:?}", stats);
        eprintln!("Errors: {:?}", stats.errors);

        // Verify UTILMD_Strom files exist
        let utilmd_dir = output.join("FV2504/UTILMD_Strom");
        assert!(
            utilmd_dir.join("msg.bin").exists(),
            "msg.bin should exist"
        );
        assert!(
            utilmd_dir.join("tx_pid_55001.bin").exists(),
            "tx_pid_55001.bin should exist"
        );
        assert!(
            utilmd_dir.join("combined_pid_55001.bin").exists(),
            "combined_pid_55001.bin should exist"
        );

        // Verify stats are reasonable
        assert!(stats.message_engines > 0, "should have message engines");
        assert!(
            stats.transaction_engines > 0,
            "should have transaction engines"
        );
        assert!(stats.combined_engines > 0, "should have combined engines");

        // Verify cache files are loadable
        let loaded = MappingEngine::load_cached(&utilmd_dir.join("msg.bin")).unwrap();
        assert!(
            !loaded.definitions().is_empty(),
            "cached msg engine should have definitions"
        );

        let loaded = MappingEngine::load_cached(&utilmd_dir.join("tx_pid_55001.bin")).unwrap();
        assert!(
            !loaded.definitions().is_empty(),
            "cached tx engine should have definitions"
        );
    }
}
