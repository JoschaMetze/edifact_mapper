//! Lightweight PID schema index for checking which groups exist in a PID.
//!
//! Used by `MappingEngine::load_with_common()` to filter common TOML templates
//! to only those whose `source_path` exists in the target PID's schema.

use std::collections::HashSet;
use std::path::Path;

/// A set of valid `source_path` values for a PID schema.
///
/// Built from the PID schema JSON file, this index allows O(1) lookup
/// of whether a given `source_path` (e.g., `sg4.sg5_z16`, `sg4.sg8_z98.sg10`)
/// exists in the PID's group structure.
#[derive(Debug, Clone)]
pub struct PidSchemaIndex {
    paths: HashSet<String>,
}

impl PidSchemaIndex {
    /// Build an index from a PID schema JSON file.
    ///
    /// Reads the schema's `fields` tree and collects all group paths
    /// (e.g., `sg2`, `sg4`, `sg4.sg5_z16`, `sg4.sg8_z98`, `sg4.sg8_z98.sg10`).
    pub fn from_schema_file(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let schema: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self::from_json(&schema))
    }

    /// Build an index from an already-parsed PID schema JSON value.
    pub fn from_json(schema: &serde_json::Value) -> Self {
        let mut paths = HashSet::new();
        if let Some(fields) = schema.get("fields").and_then(|v| v.as_object()) {
            for (key, value) in fields {
                paths.insert(key.clone());
                Self::collect_children(key, value, &mut paths);
            }
        }
        paths.insert(String::new()); // root level (source_path = "")
        Self { paths }
    }

    /// Check if a `source_path` corresponds to an existing group in this PID schema.
    ///
    /// Supports exact matches and generic group matches where the TOML uses
    /// an unqualified path (e.g., `sg4.sg12`) that should match any variant
    /// in the schema (e.g., `sg4.sg12_vy`, `sg4.sg12_dp`).
    pub fn has_group(&self, source_path: &str) -> bool {
        if self.paths.contains(source_path) {
            return true;
        }
        // Check for variant suffix match: "sg4.sg12" should match "sg4.sg12_vy"
        let prefix = format!("{source_path}_");
        self.paths.iter().any(|p| p.starts_with(&prefix))
    }

    /// Recursively collect all paths from the schema's children tree.
    fn collect_children(prefix: &str, node: &serde_json::Value, paths: &mut HashSet<String>) {
        if let Some(children) = node.get("children").and_then(|v| v.as_object()) {
            for (key, value) in children {
                let path = format!("{prefix}.{key}");
                paths.insert(path.clone());
                Self::collect_children(&path, value, paths);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_basic() {
        let schema = serde_json::json!({
            "fields": {
                "sg2": { "segments": {} },
                "sg4": {
                    "children": {
                        "sg5_z16": { "segments": {} },
                        "sg8_z98": {
                            "children": {
                                "sg10": { "segments": {} }
                            }
                        },
                        "sg12": { "segments": {} }
                    }
                }
            }
        });

        let index = PidSchemaIndex::from_json(&schema);

        // Root level
        assert!(index.has_group(""));
        // Top-level groups
        assert!(index.has_group("sg2"));
        assert!(index.has_group("sg4"));
        // SG4 children
        assert!(index.has_group("sg4.sg5_z16"));
        assert!(index.has_group("sg4.sg8_z98"));
        assert!(index.has_group("sg4.sg12"));
        // Nested children
        assert!(index.has_group("sg4.sg8_z98.sg10"));
        // Non-existent
        assert!(!index.has_group("sg4.sg5_z17"));
        assert!(!index.has_group("sg4.sg8_zd7"));
        assert!(!index.has_group("sg3"));

        // Generic group match — unqualified path matches qualified variant
        assert!(index.has_group("sg4.sg5")); // matches sg4.sg5_z16
        assert!(index.has_group("sg4.sg8")); // matches sg4.sg8_z98
    }

    #[test]
    fn test_variant_suffix_match() {
        let schema = serde_json::json!({
            "fields": {
                "sg4": {
                    "children": {
                        "sg12_vy": { "segments": {} },
                        "sg12_dp": { "segments": {} }
                    }
                }
            }
        });
        let index = PidSchemaIndex::from_json(&schema);

        // Exact match
        assert!(index.has_group("sg4.sg12_vy"));
        assert!(index.has_group("sg4.sg12_dp"));

        // Generic match — unqualified path matches any variant
        assert!(index.has_group("sg4.sg12"));

        // Non-existent group — no variant matches
        assert!(!index.has_group("sg4.sg11"));
    }

    #[test]
    fn test_empty_schema() {
        let schema = serde_json::json!({ "fields": {} });
        let index = PidSchemaIndex::from_json(&schema);
        assert!(index.has_group("")); // root always present
        assert!(!index.has_group("sg4"));
    }

    #[test]
    fn test_load_real_schema() {
        let path = Path::new(
            "../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json",
        );
        if !path.exists() {
            eprintln!("schema not found, skipping");
            return;
        }
        let index = PidSchemaIndex::from_schema_file(path).unwrap();

        // PID 55001 has sg5_z16 but not sg5_z17
        assert!(index.has_group("sg4.sg5_z16"));
        assert!(index.has_group("sg4.sg5_z22"));
        assert!(!index.has_group("sg4.sg5_z17"));
        assert!(!index.has_group("sg4.sg5_z18"));

        // SG2 exists at top level
        assert!(index.has_group("sg2"));
    }
}
