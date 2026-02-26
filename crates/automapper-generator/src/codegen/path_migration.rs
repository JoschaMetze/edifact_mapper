//! Migrates TOML mapping files from numeric paths to EDIFACT ID paths.
//!
//! Walks all `*.toml` files under a mappings directory, uses a reverse path
//! resolver (built from PID schema JSON files) to convert opaque numeric
//! paths (`loc.1.0`) to self-documenting named paths (`loc.c517.d3225`).
//! Processes files as raw text to preserve comments, ordering, and formatting.

use std::collections::HashMap;
use std::path::Path;

use regex::Regex;

use crate::error::GeneratorError;

/// Statistics for a single file migration.
#[derive(Default)]
struct FileStats {
    paths: usize,
    discriminators: usize,
    unresolved: usize,
}

/// Statistics for the entire migration run.
pub struct MigrationStats {
    pub files_processed: usize,
    pub files_changed: usize,
    pub paths_migrated: usize,
    pub discriminators_migrated: usize,
    pub unresolved: usize,
}

/// Migrate all TOML files under `mappings_dir` from numeric to named EDIFACT ID paths.
///
/// Builds a reverse resolver from all `pid_*_schema.json` files in `schema_dir`,
/// then processes each TOML file as raw text with regex replacements.
pub fn migrate_toml_dir(
    schema_dir: &Path,
    mappings_dir: &Path,
    dry_run: bool,
) -> Result<MigrationStats, GeneratorError> {
    let resolver = ReverseResolver::from_schema_dir(schema_dir)?;

    let mut stats = MigrationStats {
        files_processed: 0,
        files_changed: 0,
        paths_migrated: 0,
        discriminators_migrated: 0,
        unresolved: 0,
    };

    let pattern = format!("{}/**/*.toml", mappings_dir.display());
    let mut paths: Vec<_> = glob::glob(&pattern)
        .map_err(|e| GeneratorError::Io(std::io::Error::other(e)))?
        .filter_map(|e| e.ok())
        .collect();
    paths.sort();

    for path in &paths {
        stats.files_processed += 1;

        let content = std::fs::read_to_string(path)?;
        let (new_content, file_stats) = migrate_toml_content(&content, &resolver);

        if new_content != content {
            stats.files_changed += 1;
            stats.paths_migrated += file_stats.paths;
            stats.discriminators_migrated += file_stats.discriminators;
            stats.unresolved += file_stats.unresolved;

            let prefix = if dry_run { "[dry-run] " } else { "" };
            let unresolved_suffix = if file_stats.unresolved > 0 {
                format!(", {} unresolved", file_stats.unresolved)
            } else {
                String::new()
            };
            eprintln!(
                "  {}{} — {} paths, {} disc{}",
                prefix,
                path.display(),
                file_stats.paths,
                file_stats.discriminators,
                unresolved_suffix
            );

            if !dry_run {
                std::fs::write(path, &new_content)?;
            }
        }
    }

    Ok(stats)
}

/// Migrate a single TOML file's content. Returns (new_content, stats).
fn migrate_toml_content(content: &str, resolver: &ReverseResolver) -> (String, FileStats) {
    let disc_re = Regex::new(r#"(discriminator\s*=\s*")([A-Z]+\.\d+(?:\.\d+)?=[^"]+)(")"#).unwrap();
    let key_re =
        Regex::new(r#"^(\s*")([a-z]+(?:\[[A-Z0-9]+\])?\.\d+(?:\.\d+)?)("\s*[=\]])"#).unwrap();

    let mut stats = FileStats::default();
    let has_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let mut new_line = line.to_string();

        // Try discriminator replacement
        if let Some(caps) = disc_re.captures(&new_line) {
            let original = caps.get(2).unwrap().as_str().to_string();
            let reversed = resolver.reverse_discriminator(&original);
            if reversed != original {
                stats.discriminators += 1;
                let prefix = caps.get(1).unwrap().as_str();
                let suffix = caps.get(3).unwrap().as_str();
                new_line = new_line.replacen(
                    &format!("{}{}{}", prefix, original, suffix),
                    &format!("{}{}{}", prefix, reversed, suffix),
                    1,
                );
            }
        }

        // Try field key replacement
        if let Some(caps) = key_re.captures(&new_line) {
            let original = caps.get(2).unwrap().as_str().to_string();
            let reversed = resolver.reverse_path(&original);
            if reversed != original {
                stats.paths += 1;
                let prefix = caps.get(1).unwrap().as_str();
                let suffix = caps.get(3).unwrap().as_str();
                new_line = new_line.replacen(
                    &format!("{}{}{}", prefix, original, suffix),
                    &format!("{}{}{}", prefix, reversed, suffix),
                    1,
                );
            }
        }

        lines.push(new_line);
    }

    let mut result = lines.join("\n");
    if has_trailing_newline {
        result.push('\n');
    }

    (result, stats)
}

// ── Embedded reverse resolver ──
// Self-contained to avoid circular dependency (mig-bo4e depends on automapper-generator).

struct ReverseResolver {
    /// (seg_upper, elem_idx, sub_idx) → named suffix like "c517.d3225"
    composite_reverse: HashMap<(String, usize, usize), String>,
    /// (seg_upper, elem_idx) → named id like "d3227"
    simple_reverse: HashMap<(String, usize), String>,
    /// (seg_upper, elem_idx) → true if composite element
    is_composite: HashMap<(String, usize), bool>,
}

impl ReverseResolver {
    fn from_schema_dir(dir: &Path) -> Result<Self, GeneratorError> {
        let mut resolver = Self {
            composite_reverse: HashMap::new(),
            simple_reverse: HashMap::new(),
            is_composite: HashMap::new(),
        };

        let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        let mut schema_count = 0;
        for entry in entries {
            let path = entry.path();
            let is_schema = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("pid_") && n.ends_with("_schema.json"))
                .unwrap_or(false);
            if is_schema {
                let content = std::fs::read_to_string(&path)?;
                let schema: serde_json::Value = serde_json::from_str(&content)?;
                resolver.merge_schema(&schema);
                schema_count += 1;
            }
        }

        eprintln!("Loaded {} PID schemas for reverse resolution", schema_count);
        Ok(resolver)
    }

    fn merge_schema(&mut self, schema: &serde_json::Value) {
        if let Some(fields) = schema.get("fields").and_then(|f| f.as_object()) {
            for (_group_key, group_val) in fields {
                collect_reverse_from_group(
                    group_val,
                    &mut self.composite_reverse,
                    &mut self.simple_reverse,
                    &mut self.is_composite,
                );
            }
        }
    }

    fn reverse_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return path.to_string();
        }

        let (seg_raw, qualifier_suffix) = split_qualifier(parts[0]);
        let seg_upper = seg_raw.to_ascii_uppercase();
        let rest = &parts[1..];

        // If not numeric, already named
        if !rest[0]
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return path.to_string();
        }

        match rest.len() {
            1 => {
                let Ok(elem_idx) = rest[0].parse::<usize>() else {
                    return path.to_string();
                };
                match self.is_composite.get(&(seg_upper.clone(), elem_idx)) {
                    Some(true) => {
                        if let Some(named) = self.composite_reverse.get(&(seg_upper, elem_idx, 0)) {
                            format!("{}{}.{}", seg_raw, qualifier_suffix, named)
                        } else {
                            path.to_string()
                        }
                    }
                    Some(false) => {
                        if let Some(named) = self.simple_reverse.get(&(seg_upper, elem_idx)) {
                            format!("{}{}.{}", seg_raw, qualifier_suffix, named)
                        } else {
                            path.to_string()
                        }
                    }
                    None => path.to_string(),
                }
            }
            2 => {
                let Ok(elem_idx) = rest[0].parse::<usize>() else {
                    return path.to_string();
                };
                let Ok(sub_idx) = rest[1].parse::<usize>() else {
                    return path.to_string();
                };
                if let Some(named) = self.composite_reverse.get(&(seg_upper, elem_idx, sub_idx)) {
                    format!("{}{}.{}", seg_raw, qualifier_suffix, named)
                } else {
                    path.to_string()
                }
            }
            _ => path.to_string(),
        }
    }

    fn reverse_discriminator(&self, disc: &str) -> String {
        let Some((path_part, value_part)) = disc.split_once('=') else {
            return disc.to_string();
        };

        let parts: Vec<&str> = path_part.split('.').collect();
        if parts.len() != 3 {
            return disc.to_string();
        }

        let seg_raw = parts[0];
        let seg_upper = seg_raw.to_ascii_uppercase();

        let Ok(elem_idx) = parts[1].parse::<usize>() else {
            return disc.to_string();
        };
        let Ok(sub_idx) = parts[2].parse::<usize>() else {
            return disc.to_string();
        };

        if sub_idx == 0 {
            if let Some(false) = self.is_composite.get(&(seg_upper.clone(), elem_idx)) {
                if let Some(named) = self.simple_reverse.get(&(seg_upper.clone(), elem_idx)) {
                    return format!("{}.{}={}", seg_raw, named, value_part);
                }
            }
        }

        if let Some(named) = self
            .composite_reverse
            .get(&(seg_upper.clone(), elem_idx, sub_idx))
        {
            return format!("{}.{}={}", seg_raw, named, value_part);
        }

        disc.to_string()
    }
}

fn split_qualifier(tag: &str) -> (&str, &str) {
    if let Some(bracket_pos) = tag.find('[') {
        (&tag[..bracket_pos], &tag[bracket_pos..])
    } else {
        (tag, "")
    }
}

fn collect_reverse_from_group(
    group: &serde_json::Value,
    composite_reverse: &mut HashMap<(String, usize, usize), String>,
    simple_reverse: &mut HashMap<(String, usize), String>,
    is_composite: &mut HashMap<(String, usize), bool>,
) {
    if let Some(segments) = group.get("segments").and_then(|s| s.as_array()) {
        for seg in segments {
            let seg_tag = seg
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_ascii_uppercase();

            if let Some(elements) = seg.get("elements").and_then(|e| e.as_array()) {
                let mut composite_id_count: HashMap<String, usize> = HashMap::new();

                for elem in elements {
                    let elem_index =
                        elem.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                    if let Some(composite_id) = elem.get("composite").and_then(|v| v.as_str()) {
                        let base = format!("c{}", &composite_id[1..]).to_ascii_lowercase();

                        let count = composite_id_count.entry(base.clone()).or_insert(0);
                        *count += 1;

                        let comp_key = if *count == 1 {
                            base
                        } else {
                            format!("{}_{}", base, count)
                        };

                        is_composite
                            .entry((seg_tag.clone(), elem_index))
                            .or_insert(true);

                        if let Some(components) = elem.get("components").and_then(|c| c.as_array())
                        {
                            let mut data_elem_count: HashMap<String, usize> = HashMap::new();

                            for comp in components {
                                let comp_id = comp.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let sub_index =
                                    comp.get("sub_index").and_then(|v| v.as_u64()).unwrap_or(0)
                                        as usize;
                                let base_data = format!("d{}", comp_id).to_ascii_lowercase();

                                let dcount = data_elem_count.entry(base_data.clone()).or_insert(0);
                                *dcount += 1;

                                let data_key = if *dcount == 1 {
                                    base_data
                                } else {
                                    format!("{}_{}", base_data, dcount)
                                };

                                composite_reverse
                                    .entry((seg_tag.clone(), elem_index, sub_index))
                                    .or_insert(format!("{}.{}", comp_key, data_key));
                            }
                        }
                    } else {
                        let elem_id = elem.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let elem_id_lower = format!("d{}", elem_id).to_ascii_lowercase();

                        is_composite
                            .entry((seg_tag.clone(), elem_index))
                            .or_insert(false);
                        simple_reverse
                            .entry((seg_tag.clone(), elem_index))
                            .or_insert(elem_id_lower);
                    }
                }
            }
        }
    }

    if let Some(children) = group.get("children").and_then(|c| c.as_object()) {
        for (_child_key, child_val) in children {
            collect_reverse_from_group(child_val, composite_reverse, simple_reverse, is_composite);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_resolver() -> ReverseResolver {
        let schema = serde_json::json!({
            "beschreibung": "Test",
            "fields": {
                "sg4": {
                    "segments": [
                        {
                            "id": "LOC",
                            "name": "Lokation",
                            "elements": [
                                { "id": "3227", "index": 0, "type": "code" },
                                {
                                    "composite": "C517",
                                    "index": 1,
                                    "components": [
                                        { "id": "3225", "sub_index": 0, "type": "data" },
                                        { "id": "1131", "sub_index": 1, "type": "data" }
                                    ]
                                }
                            ]
                        },
                        {
                            "id": "STS",
                            "name": "Status",
                            "elements": [
                                {
                                    "composite": "C601",
                                    "index": 0,
                                    "components": [{ "id": "9015", "sub_index": 0, "type": "code" }]
                                },
                                {
                                    "composite": "C556",
                                    "index": 2,
                                    "components": [{ "id": "9013", "sub_index": 0, "type": "code" }]
                                },
                                {
                                    "composite": "C556",
                                    "index": 3,
                                    "components": [{ "id": "9013", "sub_index": 0, "type": "code" }]
                                }
                            ]
                        }
                    ],
                    "source_group": "SG4"
                }
            }
        });

        let mut resolver = ReverseResolver {
            composite_reverse: HashMap::new(),
            simple_reverse: HashMap::new(),
            is_composite: HashMap::new(),
        };
        resolver.merge_schema(&schema);
        resolver
    }

    #[test]
    fn migrate_field_keys() {
        let resolver = make_resolver();
        let input = r#"[fields]
"loc.0" = { target = "", default = "Z16" }
"loc.1.0" = "marktlokationsId"
"loc.1.1" = "codelisteCode"
"#;
        let (output, stats) = migrate_toml_content(input, &resolver);
        assert!(output.contains(r#""loc.d3227""#));
        assert!(output.contains(r#""loc.c517.d3225""#));
        assert!(output.contains(r#""loc.c517.d1131""#));
        assert_eq!(stats.paths, 3);
    }

    #[test]
    fn migrate_discriminator() {
        let resolver = make_resolver();
        let input = r#"[meta]
discriminator = "LOC.0.0=Z16"
"#;
        let (output, stats) = migrate_toml_content(input, &resolver);
        assert!(output.contains(r#"discriminator = "LOC.d3227=Z16""#));
        assert_eq!(stats.discriminators, 1);
    }

    #[test]
    fn migrate_ordinal_suffix() {
        let resolver = make_resolver();
        let input = r#"[fields]
"sts.0" = { target = "", default = "7" }
"sts.2" = "transaktionsgrund"
"sts.3" = "ergaenzung"
"#;
        let (output, stats) = migrate_toml_content(input, &resolver);
        assert!(output.contains(r#""sts.c601.d9015""#));
        assert!(output.contains(r#""sts.c556.d9013""#));
        assert!(output.contains(r#""sts.c556_2.d9013""#));
        assert_eq!(stats.paths, 3);
    }

    #[test]
    fn migrate_qualifier_path() {
        let resolver = make_resolver();
        let input = r#"[fields]
"loc[Z16].1.0" = "id"
"#;
        let (output, stats) = migrate_toml_content(input, &resolver);
        assert!(output.contains(r#""loc[Z16].c517.d3225""#));
        assert_eq!(stats.paths, 1);
    }

    #[test]
    fn preserve_comments_and_named_paths() {
        let resolver = make_resolver();
        let input = r#"# This is a comment
[meta]
entity = "Test"

[fields]
# LOC qualifier
"loc.d3227" = { target = "", default = "Z16" }
"loc.c517.d3225" = "id"
"#;
        let (output, stats) = migrate_toml_content(input, &resolver);
        assert_eq!(output, input, "Already-named paths should not change");
        assert_eq!(stats.paths, 0);
        assert_eq!(stats.discriminators, 0);
    }
}
