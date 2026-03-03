//! Semantic enum key derivation and persistent index for EDIFACT code enrichment.
//!
//! Generates deterministic UPPER_SNAKE_CASE identifiers from German MIG descriptions.
//! The index file (`generated/code_enum_index.json`) is committed to git and consulted
//! first on regeneration to ensure stability.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// Outer key: data element ID (e.g., "7037").
/// Inner key: code value (e.g., "Z15").
/// Value: enum key (e.g., "HAUSHALTSKUNDE_ENWG").
pub type CodeEnumIndex = BTreeMap<String, BTreeMap<String, String>>;

/// Load a persistent enum index from a JSON file.
/// Returns an empty index if the file doesn't exist.
pub fn load_index(path: &Path) -> Result<CodeEnumIndex, io::Error> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = std::fs::read_to_string(path)?;
    let index: CodeEnumIndex = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(index)
}

/// Save the enum index to a JSON file (pretty-printed, sorted).
pub fn save_index(index: &CodeEnumIndex, path: &Path) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(index).map_err(io::Error::other)?;
    std::fs::write(path, json + "\n")
}

/// Filler words stripped during derivation (lowercase, post-umlaut-replacement).
const FILLER_WORDS: &[&str] = &[
    "der", "die", "das", "des", "dem", "den", "ein", "eine", "einer", "eines", "einem", "einen",
    "und", "oder", "bzw", "fuer", "nach", "bei", "mit", "von", "vom", "zur", "zum", "auf", "in",
    "an", "am", "im", "ist", "wird", "hat", "als", "nur", "gem", "gemaess", "ggf", "ueber",
    "unter", "zwischen", "zu", "aus", "ab", "da", "wenn", "nicht", "auch", "noch", "schon",
];

/// Derive an UPPER_SNAKE_CASE enum key from a German code description.
///
/// Algorithm:
/// 1. Replace `§` → `PARAGRAPH`, umlauts → ae/oe/ue/ss
/// 2. Strip non-alphanumeric except whitespace/underscore
/// 3. Split on whitespace, drop filler words
/// 4. UPPER_SNAKE_CASE, collapse multiple underscores
/// 5. Truncate at 60 chars at word boundary
pub fn derive_enum_key(name: &str) -> String {
    let mut s = name.to_string();

    // 1. Replace special chars and umlauts
    s = s.replace('§', "PARAGRAPH");
    s = s.replace('ä', "ae");
    s = s.replace('ö', "oe");
    s = s.replace('ü', "ue");
    s = s.replace('Ä', "Ae");
    s = s.replace('Ö', "Oe");
    s = s.replace('Ü', "Ue");
    s = s.replace('ß', "ss");

    // 2. Strip non-alphanumeric except whitespace/underscore
    s = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();

    // 3. Split on whitespace, drop filler words
    let words: Vec<String> = s
        .split_whitespace()
        .filter(|w| !FILLER_WORDS.contains(&w.to_lowercase().as_str()))
        .map(|w| w.to_uppercase())
        .collect();

    // 4. UPPER_SNAKE_CASE
    let mut result = words.join("_");

    // Collapse multiple underscores
    while result.contains("__") {
        result = result.replace("__", "_");
    }
    result = result.trim_matches('_').to_string();

    // 5. Truncate at 60 chars at word boundary
    if result.len() > 60 {
        let parts: Vec<&str> = result.split('_').collect();
        let mut truncated = String::new();
        for (i, part) in parts.iter().enumerate() {
            let next = if truncated.is_empty() {
                part.to_string()
            } else {
                format!("{}_{}", truncated, part)
            };
            if next.len() > 60 {
                break;
            }
            truncated = next;
            // Ensure at least the first word is included
            if i == 0 && truncated.len() > 60 {
                truncated.truncate(60);
            }
        }
        result = truncated;
    }

    if result.is_empty() {
        result = "UNKNOWN".to_string();
    }

    result
}

/// Look up an existing enum key or derive a new one, handling collisions.
///
/// If the `(de_id, code_value)` pair already exists in the index, returns it.
/// Otherwise derives from `code_name`, disambiguates collisions within the same DE,
/// inserts into the index, and returns the new key.
pub fn lookup_or_derive(
    index: &mut CodeEnumIndex,
    de_id: &str,
    code_value: &str,
    code_name: &str,
) -> String {
    let de_map = index.entry(de_id.to_string()).or_default();

    // Already indexed — return stable key
    if let Some(existing) = de_map.get(code_value) {
        return existing.clone();
    }

    // Derive new key
    let base = derive_enum_key(code_name);

    // Check for collisions within this DE
    let existing_values: Vec<String> = de_map.values().cloned().collect();
    let final_key = if existing_values.contains(&base) {
        // Disambiguate with _2, _3, ...
        let mut suffix = 2;
        loop {
            let candidate = format!("{}_{}", base, suffix);
            if !existing_values.contains(&candidate) {
                break candidate;
            }
            suffix += 1;
        }
    } else {
        base
    };

    de_map.insert(code_value.to_string(), final_key.clone());
    final_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_basic() {
        assert_eq!(
            derive_enum_key("Haushaltskunde gem. EnWG"),
            "HAUSHALTSKUNDE_ENWG"
        );
    }

    #[test]
    fn test_derive_umlauts() {
        assert_eq!(
            derive_enum_key("Nächster Zählerstand"),
            "NAECHSTER_ZAEHLERSTAND"
        );
    }

    #[test]
    fn test_derive_paragraph() {
        assert_eq!(derive_enum_key("§14a EnWG"), "PARAGRAPH14A_ENWG");
    }

    #[test]
    fn test_derive_filler_words() {
        assert_eq!(
            derive_enum_key("Kunde des Lieferanten"),
            "KUNDE_LIEFERANTEN"
        );
    }

    #[test]
    fn test_derive_special_chars() {
        assert_eq!(derive_enum_key("Ja / Nein (Auswahl)"), "JA_NEIN_AUSWAHL");
    }

    #[test]
    fn test_derive_truncation() {
        let long = "Sehr langer Text der über sechzig Zeichen hinausgeht und dann irgendwann abgeschnitten werden muss";
        let result = derive_enum_key(long);
        assert!(result.len() <= 60, "Got {} chars: {}", result.len(), result);
        // Should end at a word boundary (no partial words)
        assert!(!result.ends_with('_'));
    }

    #[test]
    fn test_derive_empty() {
        assert_eq!(derive_enum_key(""), "UNKNOWN");
        assert_eq!(derive_enum_key("der die das"), "UNKNOWN");
    }

    #[test]
    fn test_derive_eszett() {
        assert_eq!(derive_enum_key("Straße"), "STRASSE");
    }

    #[test]
    fn test_lookup_or_derive_new() {
        let mut index = CodeEnumIndex::new();
        let key = lookup_or_derive(&mut index, "7037", "Z15", "Haushaltskunde gem. EnWG");
        assert_eq!(key, "HAUSHALTSKUNDE_ENWG");
        assert_eq!(index["7037"]["Z15"], "HAUSHALTSKUNDE_ENWG");
    }

    #[test]
    fn test_lookup_or_derive_existing() {
        let mut index = CodeEnumIndex::new();
        index
            .entry("7037".to_string())
            .or_default()
            .insert("Z15".to_string(), "CUSTOM_KEY".to_string());
        let key = lookup_or_derive(&mut index, "7037", "Z15", "Haushaltskunde gem. EnWG");
        assert_eq!(key, "CUSTOM_KEY"); // Existing key preserved
    }

    #[test]
    fn test_lookup_or_derive_collision() {
        let mut index = CodeEnumIndex::new();
        let k1 = lookup_or_derive(&mut index, "9013", "E01", "Status");
        let k2 = lookup_or_derive(&mut index, "9013", "E02", "Status");
        assert_eq!(k1, "STATUS");
        assert_eq!(k2, "STATUS_2");
        // Third collision
        let k3 = lookup_or_derive(&mut index, "9013", "E03", "Status");
        assert_eq!(k3, "STATUS_3");
    }

    #[test]
    fn test_index_persistence_roundtrip() {
        let mut index = CodeEnumIndex::new();
        lookup_or_derive(&mut index, "7037", "Z15", "Haushaltskunde");
        lookup_or_derive(&mut index, "7037", "Z18", "Kein Haushaltskunde");
        lookup_or_derive(&mut index, "3035", "Z63", "Standortadresse");

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_index.json");
        save_index(&index, &path).unwrap();
        let loaded = load_index(&path).unwrap();
        assert_eq!(index, loaded);
    }

    #[test]
    fn test_load_nonexistent_returns_empty() {
        let index = load_index(Path::new("/nonexistent/path.json")).unwrap();
        assert!(index.is_empty());
    }
}
