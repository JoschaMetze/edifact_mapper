# Fixture Enhancer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace placeholder values in generated EDIFACT fixtures with realistic German energy market data by round-tripping through the BO4E mapping engine.

**Architecture:** Generate EDIFACT → forward-map to BO4E JSON → enhance JSON values by field name → reverse-map back to EDIFACT. Three new modules: ID generators (MaLo/MeLo/NeLo/etc.), seed data (names/addresses), and the enhancer that walks MappedMessage JSON replacing values.

**Tech Stack:** Rust, serde_json::Value manipulation, existing MappingEngine from mig-bo4e crate, PID schema JSON for code list sampling.

---

### Task 1: Add mig-bo4e dependency to automapper-generator

The enhancer needs the MappingEngine for forward/reverse mapping. Add mig-bo4e as a regular dependency.

**Files:**
- Modify: `crates/automapper-generator/Cargo.toml`

**Step 1: Add the dependency**

In `crates/automapper-generator/Cargo.toml`, add to `[dependencies]`:

```toml
mig-bo4e = { path = "../mig-bo4e" }
```

**Step 2: Verify no circular dependency**

Run: `cargo check -p automapper-generator`
Expected: compiles successfully (mig-bo4e only has automapper-generator as a dev-dep, not a regular dep)

**Step 3: Commit**

```bash
git add crates/automapper-generator/Cargo.toml
git commit -m "build: add mig-bo4e dependency to automapper-generator"
```

---

### Task 2: Create ID generators module

Implements check-digit-valid generators for all German energy market identifier types: MaLo, MeLo, NeLo, Steuerbare Ressource, Technische Ressource, GLN.

**Files:**
- Create: `crates/automapper-generator/src/fixture_generator/id_generators.rs`
- Modify: `crates/automapper-generator/src/fixture_generator/mod.rs` (add `mod id_generators;`)

**Step 1: Write tests for ID generators**

At the bottom of `id_generators.rs`, add a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_malo_id_length_and_format() {
        let id = generate_malo_id(42);
        assert_eq!(id.len(), 11, "MaLo ID must be 11 digits");
        assert!(id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_malo_id_deterministic() {
        assert_eq!(generate_malo_id(42), generate_malo_id(42));
        assert_ne!(generate_malo_id(42), generate_malo_id(43));
    }

    #[test]
    fn test_melo_id_format() {
        let id = generate_melo_id(42);
        assert!(id.starts_with("DE"), "MeLo ID must start with DE");
        assert_eq!(id.len(), 33, "MeLo ID must be 33 chars");
    }

    #[test]
    fn test_nelo_id_format() {
        let id = generate_nelo_id(42);
        assert_eq!(id.len(), 11, "NeLo ID must be 11 chars");
        assert!(id.starts_with('E'), "NeLo ID must start with E");
    }

    #[test]
    fn test_steuerbare_ressource_id_format() {
        let id = generate_steuress_id(42);
        assert!(id.starts_with('C'), "SteuRess ID must start with C");
        assert_eq!(id.len(), 11);
    }

    #[test]
    fn test_technische_ressource_id_format() {
        let id = generate_techress_id(42);
        assert!(id.starts_with('D'), "TechRess ID must start with D");
        assert_eq!(id.len(), 11);
    }

    #[test]
    fn test_gln_format_and_check_digit() {
        let gln = generate_gln(42);
        assert_eq!(gln.len(), 13, "GLN must be 13 digits");
        assert!(gln.chars().all(|c| c.is_ascii_digit()));
        // Verify GS1 check digit: weighted sum mod 10
        let digits: Vec<u32> = gln.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let sum: u32 = digits[..12]
            .iter()
            .enumerate()
            .map(|(i, &d)| if i % 2 == 0 { d } else { d * 3 })
            .sum();
        let expected_check = (10 - (sum % 10)) % 10;
        assert_eq!(digits[12], expected_check, "GLN check digit must be valid");
    }

    #[test]
    fn test_reference_id_format() {
        let id = generate_reference_id(42);
        assert!(!id.is_empty());
        // Should not contain EDIFACT special chars
        assert!(!id.contains('+'));
        assert!(!id.contains(':'));
        assert!(!id.contains('\''));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p automapper-generator -- id_generators --nocapture`
Expected: FAIL — module doesn't exist yet

**Step 3: Implement ID generators**

Create `crates/automapper-generator/src/fixture_generator/id_generators.rs`:

```rust
//! Deterministic ID generators for German energy market identifiers.
//!
//! All generators take a `seed: u64` parameter for deterministic, reproducible output.
//! Same seed always produces the same ID.

/// Generate an 11-digit Marktlokations-ID (MaLo) with valid check digit.
///
/// Format: 10 digits + 1 check digit (mod 10 weighted sum).
pub fn generate_malo_id(seed: u64) -> String {
    let base = format!("{:010}", seed % 10_000_000_000);
    let check = luhn_check_digit(&base);
    format!("{base}{check}")
}

/// Generate a 33-character Messlokations-ID (MeLo).
///
/// Format: "DE" + 31 digits derived from seed.
pub fn generate_melo_id(seed: u64) -> String {
    let numeric = format!("{:031}", seed % 10_000_000_000_u64.pow(3));
    // Ensure exactly 31 chars by truncating if the modulo formatting went long
    let truncated = &numeric[numeric.len().saturating_sub(31)..];
    format!("DE{truncated}")
}

/// Generate an 11-character Netzlokations-ID (NeLo).
///
/// Format: "E" + 10 digits.
pub fn generate_nelo_id(seed: u64) -> String {
    format!("E{:010}", seed % 10_000_000_000)
}

/// Generate an 11-character Steuerbare-Ressource-ID.
///
/// Format: "C" + 10 digits.
pub fn generate_steuress_id(seed: u64) -> String {
    format!("C{:010}", seed % 10_000_000_000)
}

/// Generate an 11-character Technische-Ressource-ID.
///
/// Format: "D" + 10 digits.
pub fn generate_techress_id(seed: u64) -> String {
    format!("D{:010}", seed % 10_000_000_000)
}

/// Generate a 13-digit GLN (Global Location Number) with valid GS1 check digit.
pub fn generate_gln(seed: u64) -> String {
    // Start with 99 prefix (reserved range for test GLNs)
    let base = format!("99{:010}", seed % 10_000_000_000);
    let digits: Vec<u32> = base.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 0 { d } else { d * 3 })
        .sum();
    let check = (10 - (sum % 10)) % 10;
    format!("{base}{check}")
}

/// Generate a business reference ID (for Vorgangs-IDs, RFF references, etc.).
///
/// Produces a readable alphanumeric string safe for EDIFACT (no special chars).
pub fn generate_reference_id(seed: u64) -> String {
    // Encode seed as base-36 for compact alphanumeric representation
    let mut n = seed;
    let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .collect();
    let mut result = Vec::new();
    if n == 0 {
        result.push('0');
    }
    while n > 0 {
        result.push(chars[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    format!("REF{}", result.iter().collect::<String>())
}

/// Luhn mod-10 check digit for numeric strings.
fn luhn_check_digit(digits: &str) -> u32 {
    let sum: u32 = digits
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            let d = c.to_digit(10).unwrap();
            if i % 2 == 0 {
                let doubled = d * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                d
            }
        })
        .sum();
    (10 - (sum % 10)) % 10
}
```

**Step 4: Add module declaration**

In `crates/automapper-generator/src/fixture_generator/mod.rs`, add after line 2:

```rust
pub mod id_generators;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p automapper-generator -- id_generators --nocapture`
Expected: all 8 tests PASS

**Step 6: Commit**

```bash
git add crates/automapper-generator/src/fixture_generator/id_generators.rs
git add crates/automapper-generator/src/fixture_generator/mod.rs
git commit -m "feat: add deterministic ID generators for German energy market identifiers"
```

---

### Task 3: Create seed data module

Static const arrays of realistic German names, addresses, and business data for human-readable fields.

**Files:**
- Create: `crates/automapper-generator/src/fixture_generator/seed_data.rs`
- Modify: `crates/automapper-generator/src/fixture_generator/mod.rs` (add `mod seed_data;`)

**Step 1: Create seed data module**

Create `crates/automapper-generator/src/fixture_generator/seed_data.rs`:

```rust
//! Embedded seed data for realistic German energy market fixture values.
//!
//! All data is synthetic but plausible. Addresses are coherent tuples
//! (street/PLZ/city/region always belong together geographically).

/// Coherent German address tuple.
pub struct SeedAddress {
    pub strasse: &'static str,
    pub hausnummer: &'static str,
    pub plz: &'static str,
    pub ort: &'static str,
    pub bundesland: &'static str,
}

pub const NACHNAMEN: &[&str] = &[
    "Müller", "Schmidt", "Schneider", "Fischer", "Weber",
    "Meyer", "Wagner", "Becker", "Schulz", "Hoffmann",
    "Koch", "Richter", "Wolf", "Klein", "Schröder",
    "Neumann", "Braun", "Zimmermann", "Krüger", "Hartmann",
];

pub const VORNAMEN: &[&str] = &[
    "Anna", "Thomas", "Maria", "Stefan", "Julia",
    "Michael", "Sabine", "Andreas", "Petra", "Klaus",
    "Monika", "Jürgen", "Claudia", "Hans", "Heike",
    "Wolfgang", "Martina", "Dieter", "Susanne", "Frank",
];

pub const ANREDEN: &[&str] = &["Herr", "Frau"];

pub const TITEL: &[&str] = &["", "", "", "", "", "Dr.", "Prof.", "Dr.-Ing."];

pub const ADDRESSES: &[SeedAddress] = &[
    SeedAddress { strasse: "Berliner Str.", hausnummer: "42", plz: "10115", ort: "Berlin", bundesland: "BE" },
    SeedAddress { strasse: "Hauptstr.", hausnummer: "1", plz: "80331", ort: "München", bundesland: "BY" },
    SeedAddress { strasse: "Königsallee", hausnummer: "27", plz: "40212", ort: "Düsseldorf", bundesland: "NW" },
    SeedAddress { strasse: "Mönckebergstr.", hausnummer: "8", plz: "20095", ort: "Hamburg", bundesland: "HH" },
    SeedAddress { strasse: "Zeil", hausnummer: "15", plz: "60313", ort: "Frankfurt am Main", bundesland: "HE" },
    SeedAddress { strasse: "Kröpcke", hausnummer: "3", plz: "30159", ort: "Hannover", bundesland: "NI" },
    SeedAddress { strasse: "Marienplatz", hausnummer: "11", plz: "70178", ort: "Stuttgart", bundesland: "BW" },
    SeedAddress { strasse: "Schildergasse", hausnummer: "99", plz: "50667", ort: "Köln", bundesland: "NW" },
    SeedAddress { strasse: "Prager Str.", hausnummer: "5", plz: "01069", ort: "Dresden", bundesland: "SN" },
    SeedAddress { strasse: "Kurfürstendamm", hausnummer: "62", plz: "10707", ort: "Berlin", bundesland: "BE" },
    SeedAddress { strasse: "Lange Reihe", hausnummer: "22", plz: "20099", ort: "Hamburg", bundesland: "HH" },
    SeedAddress { strasse: "Leopoldstr.", hausnummer: "77", plz: "80802", ort: "München", bundesland: "BY" },
    SeedAddress { strasse: "Bahnhofstr.", hausnummer: "14", plz: "04109", ort: "Leipzig", bundesland: "SN" },
    SeedAddress { strasse: "Friedrichstr.", hausnummer: "33", plz: "10117", ort: "Berlin", bundesland: "BE" },
    SeedAddress { strasse: "Rheinstr.", hausnummer: "46", plz: "76185", ort: "Karlsruhe", bundesland: "BW" },
    SeedAddress { strasse: "Schloßstr.", hausnummer: "7", plz: "45468", ort: "Mülheim an der Ruhr", bundesland: "NW" },
    SeedAddress { strasse: "Am Markt", hausnummer: "19", plz: "28195", ort: "Bremen", bundesland: "HB" },
    SeedAddress { strasse: "Holstenstr.", hausnummer: "88", plz: "24103", ort: "Kiel", bundesland: "SH" },
    SeedAddress { strasse: "Domplatz", hausnummer: "2", plz: "99084", ort: "Erfurt", bundesland: "TH" },
    SeedAddress { strasse: "Breite Str.", hausnummer: "51", plz: "14467", ort: "Potsdam", bundesland: "BB" },
    SeedAddress { strasse: "Ludwigstr.", hausnummer: "16", plz: "55116", ort: "Mainz", bundesland: "RP" },
    SeedAddress { strasse: "Obernstr.", hausnummer: "34", plz: "33602", ort: "Bielefeld", bundesland: "NW" },
    SeedAddress { strasse: "Steinweg", hausnummer: "9", plz: "06108", ort: "Halle", bundesland: "ST" },
    SeedAddress { strasse: "Am Wall", hausnummer: "71", plz: "28195", ort: "Bremen", bundesland: "HB" },
    SeedAddress { strasse: "Marktplatz", hausnummer: "4", plz: "69117", ort: "Heidelberg", bundesland: "BW" },
    SeedAddress { strasse: "Schlossstr.", hausnummer: "55", plz: "52066", ort: "Aachen", bundesland: "NW" },
    SeedAddress { strasse: "Kaiserstr.", hausnummer: "23", plz: "90402", ort: "Nürnberg", bundesland: "BY" },
    SeedAddress { strasse: "Georgstr.", hausnummer: "38", plz: "30159", ort: "Hannover", bundesland: "NI" },
    SeedAddress { strasse: "Schillerstr.", hausnummer: "12", plz: "99096", ort: "Erfurt", bundesland: "TH" },
    SeedAddress { strasse: "Poststr.", hausnummer: "6", plz: "18055", ort: "Rostock", bundesland: "MV" },
];

/// Deterministic selection from a slice using a seed.
pub fn pick<T>(items: &[T], seed: u64) -> &T {
    &items[(seed as usize) % items.len()]
}

/// Deterministic selection returning index for correlated picks.
pub fn pick_index(len: usize, seed: u64) -> usize {
    (seed as usize) % len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_data_non_empty() {
        assert!(!NACHNAMEN.is_empty());
        assert!(!VORNAMEN.is_empty());
        assert!(!ANREDEN.is_empty());
        assert!(!ADDRESSES.is_empty());
    }

    #[test]
    fn test_pick_deterministic() {
        assert_eq!(pick(NACHNAMEN, 42), pick(NACHNAMEN, 42));
        assert_eq!(pick(VORNAMEN, 99), pick(VORNAMEN, 99));
    }

    #[test]
    fn test_addresses_coherent() {
        for addr in ADDRESSES {
            assert!(!addr.strasse.is_empty());
            assert!(!addr.hausnummer.is_empty());
            assert_eq!(addr.plz.len(), 5, "PLZ must be 5 digits: {}", addr.ort);
            assert!(!addr.ort.is_empty());
            assert_eq!(addr.bundesland.len(), 2, "Bundesland must be 2-char code");
        }
    }
}
```

**Step 2: Add module declaration**

In `crates/automapper-generator/src/fixture_generator/mod.rs`, add:

```rust
pub mod seed_data;
```

**Step 3: Run tests**

Run: `cargo test -p automapper-generator -- seed_data --nocapture`
Expected: all 3 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-generator/src/fixture_generator/seed_data.rs
git add crates/automapper-generator/src/fixture_generator/mod.rs
git commit -m "feat: add embedded seed data for realistic German fixture values"
```

---

### Task 4: Create enhancer module

The core logic: walks `MappedMessage` JSON, recognizes fields by BO4E name, replaces placeholder values with realistic data from ID generators and seed data, and samples code variants from the PID schema.

**Files:**
- Create: `crates/automapper-generator/src/fixture_generator/enhancer.rs`
- Modify: `crates/automapper-generator/src/fixture_generator/mod.rs` (add `mod enhancer;` + public re-export)

**Step 1: Write the enhancer tests**

At the bottom of `enhancer.rs`, add a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_enhance_replaces_marktlokations_id() {
        let mut value = json!({"marktlokationsId": "DE0012345678901234567890123456789012"});
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Marktlokation", &config, &None);
        let id = value["marktlokationsId"].as_str().unwrap();
        assert_ne!(id, "DE0012345678901234567890123456789012");
        assert_eq!(id.len(), 11, "Enhanced MaLo ID should be 11 digits");
    }

    #[test]
    fn test_enhance_replaces_nachname() {
        let mut value = json!({"nachname": "Mustermann", "vorname": "Mustermann"});
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Geschaeftspartner", &config, &None);
        let nachname = value["nachname"].as_str().unwrap();
        assert_ne!(nachname, "Mustermann");
        assert_ne!(value["vorname"].as_str().unwrap(), "Mustermann");
    }

    #[test]
    fn test_enhance_replaces_address_coherently() {
        let mut value = json!({
            "strasse": "Musterstrasse",
            "ort": "Musterstadt",
            "postleitzahl": "12345",
            "land": "DE"
        });
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Geschaeftspartner", &config, &None);
        let ort = value["ort"].as_str().unwrap();
        let plz = value["postleitzahl"].as_str().unwrap();
        assert_ne!(ort, "Musterstadt");
        assert_ne!(plz, "12345");
        // Verify coherence: PLZ and city should come from same address tuple
        let addr_idx = seed_data::ADDRESSES.iter().position(|a| a.ort == ort);
        assert!(addr_idx.is_some(), "City should be from seed data");
        assert_eq!(seed_data::ADDRESSES[addr_idx.unwrap()].plz, plz);
    }

    #[test]
    fn test_enhance_preserves_companion_qualifiers() {
        let mut value = json!({
            "nachname": "Mustermann",
            "geschaeftspartnerEdifact": {
                "nad_qualifier": "Z04",
                "codelist_code": "293"
            }
        });
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Geschaeftspartner", &config, &None);
        // Companion qualifier must be preserved
        assert_eq!(value["geschaeftspartnerEdifact"]["nad_qualifier"], "Z04");
    }

    #[test]
    fn test_enhance_deterministic() {
        let mut v1 = json!({"marktlokationsId": "placeholder", "nachname": "Mustermann"});
        let mut v2 = v1.clone();
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut v1, "Marktlokation", &config, &None);
        enhance_entity_value(&mut v2, "Marktlokation", &config, &None);
        assert_eq!(v1, v2, "Same seed must produce same output");
    }

    #[test]
    fn test_enhance_variant_changes_codes() {
        let schema = json!({
            "fields": {
                "sg4": {
                    "segments": [{
                        "id": "STS",
                        "elements": [{
                            "index": 1,
                            "components": [{
                                "sub_index": 0,
                                "id": "9013",
                                "type": "code",
                                "codes": [
                                    {"value": "E01"},
                                    {"value": "E02"},
                                    {"value": "E03"}
                                ]
                            }]
                        }]
                    }]
                }
            }
        });
        let code_map = build_code_map(&schema);

        let mut v0 = json!({"transaktionsgrund": "E01"});
        let mut v1 = json!({"transaktionsgrund": "E01"});

        let config0 = EnhancerConfig::new(42, 0);
        let config1 = EnhancerConfig::new(42, 1);

        enhance_entity_value(&mut v0, "Prozessdaten", &config0, &Some(code_map.clone()));
        enhance_entity_value(&mut v1, "Prozessdaten", &config1, &Some(code_map));

        // Different variants should produce different code values
        // (as long as the code list has >1 entry)
        assert_ne!(
            v0["transaktionsgrund"], v1["transaktionsgrund"],
            "Different variants should sample different codes"
        );
    }

    #[test]
    fn test_enhance_gln_identifikation() {
        let mut value = json!({"identifikation": "1234567890128"});
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Marktteilnehmer", &config, &None);
        let gln = value["identifikation"].as_str().unwrap();
        assert_eq!(gln.len(), 13, "GLN should be 13 digits");
        assert_ne!(gln, "1234567890128");
    }

    #[test]
    fn test_enhance_reference_ids() {
        let mut value = json!({"vorgangId": "GENERATED00001"});
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Prozessdaten", &config, &None);
        let id = value["vorgangId"].as_str().unwrap();
        assert_ne!(id, "GENERATED00001");
        assert!(!id.contains('+'), "Reference ID must not contain EDIFACT specials");
    }

    #[test]
    fn test_enhance_dates() {
        let mut value = json!({"gueltigAb": "20250401120000?+00", "gueltigBis": "20250401120000?+00"});
        let config = EnhancerConfig::new(42, 0);
        enhance_entity_value(&mut value, "Prozessdaten", &config, &None);
        let ab = value["gueltigAb"].as_str().unwrap();
        let bis = value["gueltigBis"].as_str().unwrap();
        // Dates should be different from each other and from placeholder
        assert_ne!(ab, "20250401120000?+00");
        assert_ne!(ab, bis, "gueltigAb and gueltigBis should differ");
    }

    #[test]
    fn test_enhance_mapped_message() {
        let mut msg = mig_bo4e::model::MappedMessage {
            stammdaten: json!({
                "marktteilnehmer": [
                    {"identifikation": "1234567890128"},
                    {"identifikation": "1234567890128"}
                ]
            }),
            transaktionen: vec![mig_bo4e::model::Transaktion {
                stammdaten: json!({
                    "marktlokation": {"marktlokationsId": "DE0012345678901234567890123456789012"},
                    "geschaeftspartner": [{"nachname": "Mustermann", "ort": "Musterstadt"}]
                }),
                transaktionsdaten: json!({
                    "vorgangId": "GENERATED00001"
                }),
            }],
        };
        let config = EnhancerConfig::new(42, 0);
        enhance_mapped_message(&mut msg, &None, &config);
        // Verify stammdaten enhanced
        assert_ne!(msg.stammdaten["marktteilnehmer"][0]["identifikation"], "1234567890128");
        // Verify transaction stammdaten enhanced
        assert_ne!(msg.transaktionen[0].stammdaten["marktlokation"]["marktlokationsId"], "DE0012345678901234567890123456789012");
        // Verify transaktionsdaten enhanced
        assert_ne!(msg.transaktionen[0].transaktionsdaten["vorgangId"], "GENERATED00001");
    }
}
```

**Step 2: Implement the enhancer**

Create `crates/automapper-generator/src/fixture_generator/enhancer.rs`:

```rust
//! BO4E-level fixture enhancer.
//!
//! Walks a `MappedMessage` JSON structure and replaces placeholder values
//! with realistic German energy market data. Field recognition is based on
//! BO4E target field names from the TOML mappings.

use std::collections::HashMap;
use super::id_generators;
use super::seed_data;

/// Configuration for the enhancer.
pub struct EnhancerConfig {
    /// Base seed for deterministic generation.
    pub seed: u64,
    /// Variant index for code sampling (0 = first code, 1 = second, etc.).
    pub variant: usize,
}

impl EnhancerConfig {
    pub fn new(seed: u64, variant: usize) -> Self {
        Self { seed, variant }
    }

    /// Derive a sub-seed for a specific field position to avoid correlated values.
    fn field_seed(&self, entity: &str, field: &str) -> u64 {
        let mut hash: u64 = self.seed;
        for b in entity.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        for b in field.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        hash
    }
}

/// Map from BO4E field name → list of valid code values (from PID schema).
/// Built by `build_code_map()` from the PID schema JSON.
pub type CodeMap = HashMap<String, Vec<String>>;

/// Build a code map from the PID schema JSON.
///
/// Walks all segments in all groups, finds code-type fields that have a
/// non-empty `codes` array, and maps them by their data element ID.
/// The enhancer uses the BO4E field name (not element ID) for lookup,
/// so this requires a reverse lookup through TOML mappings — for now,
/// we key by well-known field name patterns derived from element descriptions.
pub fn build_code_map(schema: &serde_json::Value) -> CodeMap {
    let mut map = CodeMap::new();
    collect_codes_recursive(&schema["fields"], &mut map);
    collect_codes_recursive(&schema["root_segments"], &mut map);
    map
}

fn collect_codes_recursive(value: &serde_json::Value, map: &mut CodeMap) {
    match value {
        serde_json::Value::Object(obj) => {
            // Check if this is a segment with code elements
            if let Some(segments) = obj.get("segments").and_then(|s| s.as_array()) {
                for seg in segments {
                    collect_segment_codes(seg, map);
                }
            }
            // Recurse into children
            if let Some(children) = obj.get("children").and_then(|c| c.as_object()) {
                for child in children.values() {
                    collect_codes_recursive(child, map);
                }
            }
            // Recurse into other object values
            for (key, val) in obj {
                if key != "segments" && key != "children" {
                    collect_codes_recursive(val, map);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_codes_recursive(item, map);
            }
        }
        _ => {}
    }
}

fn collect_segment_codes(segment: &serde_json::Value, map: &mut CodeMap) {
    let Some(elements) = segment["elements"].as_array() else { return };
    for el in elements {
        // Check components
        if let Some(components) = el["components"].as_array() {
            for comp in components {
                if comp["type"].as_str() == Some("code") {
                    if let Some(codes) = comp["codes"].as_array() {
                        let values: Vec<String> = codes
                            .iter()
                            .filter_map(|c| c["value"].as_str().map(String::from))
                            .collect();
                        if values.len() > 1 {
                            if let Some(name) = comp["name"].as_str() {
                                map.insert(name.to_string(), values);
                            }
                            if let Some(id) = comp["id"].as_str() {
                                map.entry(id.to_string()).or_insert_with(|| {
                                    codes.iter()
                                        .filter_map(|c| c["value"].as_str().map(String::from))
                                        .collect()
                                });
                            }
                        }
                    }
                }
            }
        }
        // Check simple code elements
        if el["type"].as_str() == Some("code") {
            if let Some(codes) = el["codes"].as_array() {
                let values: Vec<String> = codes
                    .iter()
                    .filter_map(|c| c["value"].as_str().map(String::from))
                    .collect();
                if values.len() > 1 {
                    if let Some(name) = el["name"].as_str() {
                        map.insert(name.to_string(), values);
                    }
                    if let Some(id) = el["id"].as_str() {
                        map.entry(id.to_string()).or_insert_with(|| {
                            codes.iter()
                                .filter_map(|c| c["value"].as_str().map(String::from))
                                .collect()
                        });
                    }
                }
            }
        }
    }
}

/// Enhance a `MappedMessage` in place with realistic values.
pub fn enhance_mapped_message(
    msg: &mut mig_bo4e::model::MappedMessage,
    code_map: &Option<CodeMap>,
    config: &EnhancerConfig,
) {
    // Enhance message-level stammdaten
    if let Some(obj) = msg.stammdaten.as_object_mut() {
        let keys: Vec<String> = obj.keys().cloned().collect();
        for key in keys {
            let entity_name = to_entity_name(&key);
            if let Some(val) = obj.get_mut(&key) {
                enhance_value_recursive(val, &entity_name, config, code_map, 0);
            }
        }
    }

    // Enhance each transaction
    for (tx_idx, tx) in msg.transaktionen.iter_mut().enumerate() {
        let tx_config = EnhancerConfig {
            seed: config.seed.wrapping_add(tx_idx as u64 * 1000),
            variant: config.variant,
        };

        // Enhance transaction stammdaten
        if let Some(obj) = tx.stammdaten.as_object_mut() {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for key in keys {
                let entity_name = to_entity_name(&key);
                if let Some(val) = obj.get_mut(&key) {
                    enhance_value_recursive(val, &entity_name, &tx_config, code_map, 0);
                }
            }
        }

        // Enhance transaktionsdaten (Prozessdaten)
        enhance_entity_value(&mut tx.transaktionsdaten, "Prozessdaten", &tx_config, code_map);
    }
}

/// Recursively enhance JSON values — handles arrays of entities and nested objects.
fn enhance_value_recursive(
    value: &mut serde_json::Value,
    entity_name: &str,
    config: &EnhancerConfig,
    code_map: &Option<CodeMap>,
    array_idx: usize,
) {
    match value {
        serde_json::Value::Array(arr) => {
            for (i, item) in arr.iter_mut().enumerate() {
                let item_config = EnhancerConfig {
                    seed: config.seed.wrapping_add(i as u64 * 100),
                    variant: config.variant,
                };
                enhance_value_recursive(item, entity_name, &item_config, code_map, i);
            }
        }
        serde_json::Value::Object(_) => {
            let adjusted_config = EnhancerConfig {
                seed: config.seed.wrapping_add(array_idx as u64 * 100),
                variant: config.variant,
            };
            enhance_entity_value(value, entity_name, &adjusted_config, code_map);
        }
        _ => {}
    }
}

/// Convert camelCase JSON key to PascalCase entity name for matching.
fn to_entity_name(key: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in key.chars() {
        if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Enhance a single entity's JSON object in place.
///
/// Recognizes fields by their BO4E target name and replaces placeholder values
/// with realistic data. Companion fields (nested under `*Edifact` keys) are preserved.
pub fn enhance_entity_value(
    value: &mut serde_json::Value,
    entity_name: &str,
    config: &EnhancerConfig,
    code_map: &Option<CodeMap>,
) {
    let Some(obj) = value.as_object_mut() else { return };

    let keys: Vec<String> = obj.keys().cloned().collect();
    for key in &keys {
        // Skip companion type objects (contain structural qualifiers)
        if key.ends_with("Edifact") {
            continue;
        }

        // Skip nested objects/arrays (already handled by recursive walk)
        if let Some(val) = obj.get(key) {
            if val.is_object() || val.is_array() {
                continue;
            }
        }

        let Some(current) = obj.get(key).and_then(|v| v.as_str()).map(String::from) else {
            continue;
        };

        let fseed = config.field_seed(entity_name, key);

        if let Some(replacement) = generate_replacement(key, &current, entity_name, fseed, config, code_map) {
            obj.insert(key.clone(), serde_json::Value::String(replacement));
        }
    }
}

/// Generate a replacement value for a field based on its name and entity context.
///
/// Returns `None` if the field should not be enhanced (e.g., unknown pattern).
fn generate_replacement(
    field_name: &str,
    current_value: &str,
    entity_name: &str,
    seed: u64,
    config: &EnhancerConfig,
    code_map: &Option<CodeMap>,
) -> Option<String> {
    // 1. Energy market IDs — by field name
    match field_name {
        "marktlokationsId" => return Some(id_generators::generate_malo_id(seed)),
        "messlokationsId" => return Some(id_generators::generate_melo_id(seed)),
        "netzlokationsId" => return Some(id_generators::generate_nelo_id(seed)),
        "steuerbareRessourceId" => return Some(id_generators::generate_steuress_id(seed)),
        "technischeRessourceId" => return Some(id_generators::generate_techress_id(seed)),
        "tranchenId" => return Some(id_generators::generate_malo_id(seed.wrapping_add(7))),
        _ => {}
    }

    // 2. GLN identifiers
    if field_name == "identifikation" || field_name == "absenderCode" || field_name == "empfaengerCode" {
        if current_value.len() == 13 && current_value.chars().all(|c| c.is_ascii_digit()) {
            return Some(id_generators::generate_gln(seed));
        }
    }

    // 3. Person names
    match field_name {
        "nachname" => return Some(seed_data::pick(seed_data::NACHNAMEN, seed).to_string()),
        "vorname" => return Some(seed_data::pick(seed_data::VORNAMEN, seed).to_string()),
        "anrede" => return Some(seed_data::pick(seed_data::ANREDEN, seed).to_string()),
        "titel" => {
            let t = seed_data::pick(seed_data::TITEL, seed);
            if t.is_empty() { return None; }
            return Some(t.to_string());
        }
        _ => {}
    }

    // 4. Address fields — pick coherent tuple
    if matches!(field_name, "strasse" | "hausnummer" | "ort" | "postleitzahl" | "region") {
        // Use entity+seed to pick same address for all fields of one entity instance
        let addr_seed = config.field_seed(entity_name, "address_tuple");
        let addr = seed_data::pick(seed_data::ADDRESSES, addr_seed);
        return Some(match field_name {
            "strasse" => addr.strasse.to_string(),
            "hausnummer" => addr.hausnummer.to_string(),
            "ort" => addr.ort.to_string(),
            "postleitzahl" => addr.plz.to_string(),
            "region" => addr.bundesland.to_string(),
            _ => unreachable!(),
        });
    }

    // 5. Reference IDs
    if field_name == "vorgangId" || field_name.ends_with("Referenz") || field_name.ends_with("referenz") {
        if current_value == "GENERATED00001" || current_value == "TESTID" {
            return Some(id_generators::generate_reference_id(seed));
        }
    }

    // 6. Date/time fields
    if field_name.ends_with("Ab") || field_name.starts_with("gueltig") || field_name.ends_with("Datum") || field_name.ends_with("datum") {
        return Some(generate_date(seed, field_name));
    }

    // 7. Code-type fields — sample from schema code lists
    if let Some(code_map) = code_map {
        // Try field name as key, then check all code lists for current value
        if let Some(codes) = code_map.get(field_name) {
            if codes.len() > 1 {
                let idx = config.variant % codes.len();
                return Some(codes[idx].clone());
            }
        }
    }

    // 8. Generic placeholders
    if current_value == "X" || current_value == "TESTID" || current_value == "TESTPRODUCT" || current_value == "GENERATED00001" {
        return Some(id_generators::generate_reference_id(seed));
    }

    None
}

/// Generate a realistic EDIFACT date string for a field.
fn generate_date(seed: u64, field_name: &str) -> String {
    // Base date: 2025-01-01 + seed-derived offset
    let day_offset = (seed % 730) as i32; // 0-729 days range (2 years)

    let base_days = if field_name.contains("Ab") || field_name.contains("Von") || field_name.contains("von") {
        // Start dates: 2024-01-01 to 2025-12-31
        day_offset
    } else {
        // End dates: offset further into the future
        day_offset + 365
    };

    // Convert to CCYYMMDD
    let year = 2024 + (base_days / 365);
    let day_in_year = base_days % 365;
    let month = (day_in_year / 30).min(11) + 1;
    let day = (day_in_year % 30).min(27) + 1;

    format!("{year:04}{month:02}{day:02}120000?+00")
}
```

**Step 3: Add module declaration and public re-export**

In `crates/automapper-generator/src/fixture_generator/mod.rs`, add:

```rust
pub mod enhancer;
```

Also add the public re-export for the entry point (after the existing `pub use validate::validate_fixture;`):

```rust
pub use enhancer::{enhance_mapped_message, build_code_map, EnhancerConfig};
```

**Step 4: Run tests**

Run: `cargo test -p automapper-generator -- enhancer --nocapture`
Expected: all 10 tests PASS

**Step 5: Commit**

```bash
git add crates/automapper-generator/src/fixture_generator/enhancer.rs
git add crates/automapper-generator/src/fixture_generator/mod.rs
git commit -m "feat: add BO4E-level fixture enhancer with field recognition and code sampling"
```

---

### Task 5: Add `generate_enhanced_fixture` entry point

Wires the full pipeline: generate → assemble → forward map → enhance → reverse map → disassemble → render.

**Files:**
- Modify: `crates/automapper-generator/src/fixture_generator/mod.rs`

**Step 1: Add the entry point function**

In `crates/automapper-generator/src/fixture_generator/mod.rs`, add:

```rust
use mig_assembly::assembler::{AssembledSegment, Assembler};
use mig_assembly::disassembler::Disassembler;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_types::schema::mig::MigSchema;

pub use enhancer::{build_code_map, enhance_mapped_message, EnhancerConfig};

/// Generate an enhanced EDIFACT fixture with realistic values.
///
/// Pipeline: generate basic fixture → tokenize → assemble → forward map to BO4E
/// → enhance BO4E values → reverse map back to EDIFACT → render.
///
/// Returns the enhanced EDIFACT string, or falls back to unenhanced if mapping
/// directories don't exist for this PID.
pub fn generate_enhanced_fixture(
    schema: &Value,
    filtered_mig: &MigSchema,
    msg_engine: &MappingEngine,
    tx_engine: &MappingEngine,
    seed: u64,
    variant: usize,
) -> Result<String, crate::error::GeneratorError> {
    // Step 1: Generate basic fixture
    let edi = generate_fixture(schema);

    // Step 2: Tokenize and split messages
    let segments = parse_to_segments(edi.as_bytes())
        .map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("tokenization failed: {e}"),
        })?;
    let chunks = split_messages(segments)
        .map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("split failed: {e}"),
        })?;

    if chunks.messages.is_empty() {
        return Err(crate::error::GeneratorError::Validation {
            message: "no messages found in generated fixture".to_string(),
        });
    }

    let msg_chunk = &chunks.messages[0];

    // Step 3: Assemble with UNH + body + UNT
    let mut msg_segs = vec![msg_chunk.unh.clone()];
    msg_segs.extend(msg_chunk.body.iter().cloned());
    msg_segs.push(msg_chunk.unt.clone());

    let assembler = Assembler::new(filtered_mig);
    let tree = assembler.assemble_generic(&msg_segs)
        .map_err(|e| crate::error::GeneratorError::Validation {
            message: format!("assembly failed: {e}"),
        })?;

    // Step 4: Forward map to BO4E
    let mut mapped = MappingEngine::map_interchange(msg_engine, tx_engine, &tree, "SG4", true);

    // Step 5: Enhance BO4E values
    let code_map = build_code_map(schema);
    let config = EnhancerConfig::new(seed, variant);
    enhance_mapped_message(&mut mapped, &Some(code_map), &config);

    // Step 6: Reverse map back to assembled tree
    let mut reverse_tree = MappingEngine::map_interchange_reverse(msg_engine, tx_engine, &mapped, "SG4");

    // Add UNH to front
    let unh_assembled = owned_to_assembled(&msg_chunk.unh);
    reverse_tree.segments.insert(0, unh_assembled);
    reverse_tree.post_group_start += 1;

    // Add UNT if original had it
    let original_has_unt = tree.segments.last().map(|s| s.tag.as_str()) == Some("UNT");
    if original_has_unt {
        let unt_assembled = owned_to_assembled(&msg_chunk.unt);
        reverse_tree.segments.push(unt_assembled);
    }

    // Step 7: Disassemble and render
    let disassembler = Disassembler::new(filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();
    let dis_segs = disassembler.disassemble(&reverse_tree);
    let rendered = render_edifact(&dis_segs, &delimiters);

    // Wrap with UNB/UNZ envelope
    let unb = format!(
        "UNB+UNOC:3+{}:500+{}:500+{}+{}",
        chunks.unb.elements.get(1).and_then(|e| e.first()).map(|s| s.as_str()).unwrap_or("9900000000003"),
        chunks.unb.elements.get(2).and_then(|e| e.first()).map(|s| s.as_str()).unwrap_or("9900000000004"),
        chunks.unb.elements.get(3).and_then(|e| e.first()).map(|s| s.as_str()).unwrap_or("250401:1200"),
        chunks.unb.elements.get(4).and_then(|e| e.first()).map(|s| s.as_str()).unwrap_or("REF00001"),
    );

    let unz = format!("UNZ+1+{}", chunks.unb.elements.get(4).and_then(|e| e.first()).map(|s| s.as_str()).unwrap_or("REF00001"));

    Ok(format!("{unb}'\n{rendered}{unz}'\n"))
}

fn owned_to_assembled(seg: &mig_assembly::tokenize::OwnedSegment) -> AssembledSegment {
    AssembledSegment {
        tag: seg.id.clone(),
        elements: seg
            .elements
            .iter()
            .map(|el| el.iter().map(|c| c.to_string()).collect())
            .collect(),
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p automapper-generator`
Expected: compiles (may need import adjustments)

**Step 3: Commit**

```bash
git add crates/automapper-generator/src/fixture_generator/mod.rs
git commit -m "feat: add generate_enhanced_fixture entry point with full roundtrip pipeline"
```

---

### Task 6: Add CLI flags for enhancement

Add `--enhance`, `--variants`, and `--seed` flags to the `GenerateFixture` CLI command in `main.rs`, and wire them to the enhancer pipeline.

**Files:**
- Modify: `crates/automapper-generator/src/main.rs`

**Step 1: Add CLI flags to GenerateFixture**

In `main.rs`, find the `GenerateFixture` variant (around line 327) and add these fields after `format_version`:

```rust
        /// Enhance fixture with realistic values via BO4E roundtrip
        #[arg(long)]
        enhance: bool,

        /// Number of enhanced variants to generate (default: 1)
        #[arg(long, default_value = "1")]
        variants: Option<usize>,

        /// Seed for deterministic generation (default: 42)
        #[arg(long, default_value = "42")]
        seed: Option<u64>,
```

**Step 2: Wire CLI flags to enhancer in the command handler**

In the `Commands::GenerateFixture` match arm (around line 1333), add the new fields to the destructuring:

```rust
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
        } => {
```

Replace the body of the handler with logic that branches on `enhance`:

```rust
            eprintln!("Generating fixture from schema: {:?}", pid_schema);

            let schema_str = std::fs::read_to_string(&pid_schema)?;
            let schema: serde_json::Value = serde_json::from_str(&schema_str)?;

            let pid = schema["pid"].as_str().unwrap_or("unknown");
            let beschreibung = schema["beschreibung"].as_str().unwrap_or("");
            eprintln!("  PID: {} ({})", pid, beschreibung);

            let msg_type = message_type.as_deref().unwrap_or("UTILMD");
            let var = variant.as_deref();
            let fv = format_version.as_deref().unwrap_or("FV2504");
            let seed_val = seed.unwrap_or(42);
            let variant_count = variants.unwrap_or(1);

            if enhance {
                // Enhancement requires MIG/AHB XMLs
                let mig_path = mig_xml.ok_or_else(|| automapper_generator::GeneratorError::Validation {
                    message: "--mig-xml is required when --enhance is set".to_string(),
                })?;
                let ahb_path = ahb_xml.ok_or_else(|| automapper_generator::GeneratorError::Validation {
                    message: "--ahb-xml is required when --enhance is set".to_string(),
                })?;

                // Load MIG and filter for PID
                let mig = mig_assembly::parsing::parse_mig(&mig_path, msg_type, var, fv)?;
                let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(&ahb_path, msg_type, var, fv)?;
                let pid_def = ahb.workflows.iter().find(|w| w.id == pid)
                    .ok_or_else(|| automapper_generator::GeneratorError::Validation {
                        message: format!("PID {pid} not found in AHB"),
                    })?;
                let ahb_numbers: std::collections::HashSet<String> = pid_def.segment_numbers.iter().cloned().collect();
                let filtered_mig = mig_assembly::pid_filter::filter_mig_for_pid(&mig, &ahb_numbers);

                // Load mapping engines
                let fv_lower = fv.to_lowercase();
                let mappings_base = format!("mappings/{fv_lower}/UTILMD_{}", var.unwrap_or("Strom"));
                let msg_dir = std::path::Path::new(&mappings_base).join("message");
                let tx_dir = std::path::Path::new(&mappings_base).join(format!("pid_{pid}"));

                if !msg_dir.exists() || !tx_dir.exists() {
                    eprintln!("  WARNING: TOML mappings not found at {:?}, falling back to unenhanced", tx_dir);
                    let edi = automapper_generator::fixture_generator::generate_fixture(&schema);
                    std::fs::write(&output, &edi)?;
                    eprintln!("  Output: {:?} ({} segments, unenhanced)", output, edi.matches('\'').count());
                    return Ok(());
                }

                let schema_dir = std::path::Path::new("crates/mig-types/src/generated")
                    .join(&fv_lower)
                    .join("utilmd")
                    .join("pids");
                let resolver = mig_bo4e::path_resolver::PathResolver::from_schema_dir(&schema_dir);
                let msg_engine = mig_bo4e::engine::MappingEngine::load(&msg_dir)
                    .map_err(|e| automapper_generator::GeneratorError::Validation {
                        message: format!("failed to load message engine: {e}"),
                    })?
                    .with_path_resolver(resolver.clone());
                let tx_engine = mig_bo4e::engine::MappingEngine::load(&tx_dir)
                    .map_err(|e| automapper_generator::GeneratorError::Validation {
                        message: format!("failed to load PID engine: {e}"),
                    })?
                    .with_path_resolver(resolver);

                for v in 0..variant_count {
                    let variant_seed = seed_val.wrapping_add(v as u64 * 10000);
                    let edi = automapper_generator::fixture_generator::generate_enhanced_fixture(
                        &schema, &filtered_mig, &msg_engine, &tx_engine, variant_seed, v,
                    )?;

                    let seg_count = edi.matches('\'').count();

                    let variant_output = if variant_count > 1 {
                        let stem = output.file_stem().and_then(|s| s.to_str()).unwrap_or("fixture");
                        let ext = output.extension().and_then(|s| s.to_str()).unwrap_or("edi");
                        output.with_file_name(format!("{stem}_v{v}.{ext}"))
                    } else {
                        output.clone()
                    };

                    if let Some(parent) = variant_output.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::write(&variant_output, &edi)?;
                    eprintln!("  Output: {:?} ({} segments, enhanced, variant {})", variant_output, seg_count, v);
                }
            } else {
                let edi = automapper_generator::fixture_generator::generate_fixture(&schema);
                let seg_count = edi.matches('\'').count();
                eprintln!("  Generated {} segments", seg_count);

                if let Some(parent) = output.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&output, &edi)?;
                eprintln!("  Output: {:?}", output);

                if validate {
                    let mig_path = mig_xml.ok_or_else(|| automapper_generator::GeneratorError::Validation {
                        message: "--mig-xml is required when --validate is set".to_string(),
                    })?;
                    let ahb_path = ahb_xml.ok_or_else(|| automapper_generator::GeneratorError::Validation {
                        message: "--ahb-xml is required when --validate is set".to_string(),
                    })?;

                    eprintln!("\nValidating fixture against MIG/AHB...");
                    let result = automapper_generator::fixture_generator::validate_fixture(
                        &edi, pid, &mig_path, &ahb_path, msg_type, var, fv,
                    )?;

                    eprintln!("  Tokenized: {} segments", result.segment_count);
                    eprintln!("  Assembled: {} segments in {} groups", result.assembled_segment_count, result.assembled_group_count);

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
```

**Step 3: Verify it compiles**

Run: `cargo build -p automapper-generator`
Expected: compiles

**Step 4: Smoke test with a PID that has TOML mappings**

Run:
```bash
cargo run -p automapper-generator -- generate-fixture \
  --pid-schema crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json \
  --output /tmp/55001_enhanced.edi \
  --enhance \
  --mig-xml xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-xml xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml
```
Expected: generates enhanced fixture, output should NOT contain "Mustermann" or "GENERATED00001"

**Step 5: Smoke test with variants**

Run:
```bash
cargo run -p automapper-generator -- generate-fixture \
  --pid-schema crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json \
  --output /tmp/55001.edi \
  --enhance \
  --variants 3 \
  --seed 99 \
  --mig-xml xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --ahb-xml xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml
```
Expected: generates 3 files: `55001_v0.edi`, `55001_v1.edi`, `55001_v2.edi`

**Step 6: Commit**

```bash
git add crates/automapper-generator/src/main.rs
git commit -m "feat: add --enhance, --variants, --seed CLI flags for fixture enhancement"
```

---

### Task 7: Integration test and cleanup

Write an integration test that verifies the full enhancement pipeline, then run clippy and format.

**Files:**
- Create: `crates/automapper-generator/tests/fixture_enhancer_test.rs`

**Step 1: Write integration test**

Create `crates/automapper-generator/tests/fixture_enhancer_test.rs`:

```rust
//! Integration test for the fixture enhancer pipeline.
//!
//! Verifies that generate_enhanced_fixture() produces structurally valid EDIFACT
//! with enhanced (non-placeholder) values.

use automapper_generator::fixture_generator::{
    generate_enhanced_fixture, generate_fixture, EnhancerConfig,
};
use mig_assembly::assembler::Assembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use std::collections::HashSet;
use std::path::Path;

const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml";
const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml";
const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";
const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Strom";

fn path_resolver() -> PathResolver {
    PathResolver::from_schema_dir(Path::new(SCHEMA_DIR))
}

/// Test enhancement for a PID that has TOML mappings.
#[test]
fn test_enhanced_fixture_55001() {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        eprintln!("Skipping: MIG/AHB XML not available");
        return;
    }

    let schema_path = Path::new(SCHEMA_DIR).join("pid_55001_schema.json");
    if !schema_path.exists() {
        eprintln!("Skipping: PID schema not available");
        return;
    }

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();

    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
    let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(
        ahb_path, "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();
    let pid = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(&mig, &numbers);

    let msg_dir = Path::new(MAPPINGS_BASE).join("message");
    let tx_dir = Path::new(MAPPINGS_BASE).join("pid_55001");
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping: TOML mappings not available");
        return;
    }

    let resolver = path_resolver();
    let msg_engine = MappingEngine::load(&msg_dir).unwrap().with_path_resolver(resolver.clone());
    let tx_engine = MappingEngine::load(&tx_dir).unwrap().with_path_resolver(resolver);

    let enhanced = generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 42, 0).unwrap();

    // Verify it's valid EDIFACT (can be tokenized)
    let segments = parse_to_segments(enhanced.as_bytes()).unwrap();
    assert!(segments.len() > 10, "Enhanced fixture should have multiple segments");

    // Verify placeholders are replaced
    assert!(!enhanced.contains("Mustermann"), "Should not contain placeholder name");
    assert!(!enhanced.contains("Musterstadt"), "Should not contain placeholder city");
    assert!(!enhanced.contains("Musterstrasse"), "Should not contain placeholder street");

    // Verify it still has proper EDIFACT structure
    assert!(enhanced.contains("UNB+"), "Must have UNB");
    assert!(enhanced.contains("UNH+"), "Must have UNH");
    assert!(enhanced.contains("BGM+"), "Must have BGM");

    eprintln!("Enhanced fixture has {} segments", segments.len());
    eprintln!("First 5 segments:");
    for seg in segments.iter().take(5) {
        eprintln!("  {}", seg.id);
    }
}

/// Test that different seeds produce different output.
#[test]
fn test_enhanced_fixture_deterministic_and_varied() {
    let mig_path = Path::new(MIG_XML_PATH);
    let ahb_path = Path::new(AHB_XML_PATH);
    if !mig_path.exists() || !ahb_path.exists() {
        eprintln!("Skipping: MIG/AHB XML not available");
        return;
    }

    let schema_path = Path::new(SCHEMA_DIR).join("pid_55001_schema.json");
    if !schema_path.exists() {
        eprintln!("Skipping: PID schema not available");
        return;
    }

    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&schema_path).unwrap()).unwrap();

    let mig = parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
    let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(
        ahb_path, "UTILMD", Some("Strom"), "FV2504",
    ).unwrap();
    let pid = ahb.workflows.iter().find(|w| w.id == "55001").unwrap();
    let numbers: HashSet<String> = pid.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(&mig, &numbers);

    let msg_dir = Path::new(MAPPINGS_BASE).join("message");
    let tx_dir = Path::new(MAPPINGS_BASE).join("pid_55001");
    if !msg_dir.exists() || !tx_dir.exists() {
        eprintln!("Skipping: TOML mappings not available");
        return;
    }

    let resolver = path_resolver();
    let msg_engine = MappingEngine::load(&msg_dir).unwrap().with_path_resolver(resolver.clone());
    let tx_engine = MappingEngine::load(&tx_dir).unwrap().with_path_resolver(resolver);

    // Same seed = same output
    let e1 = generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 42, 0).unwrap();
    let e2 = generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 42, 0).unwrap();
    assert_eq!(e1, e2, "Same seed must produce identical output");

    // Different seed = different output
    let e3 = generate_enhanced_fixture(&schema, &filtered_mig, &msg_engine, &tx_engine, 99, 0).unwrap();
    assert_ne!(e1, e3, "Different seeds should produce different output");
}
```

**Step 2: Run the integration test**

Run: `cargo test -p automapper-generator -- fixture_enhancer --nocapture`
Expected: PASS (or skip if MIG/AHB XMLs not present)

**Step 3: Run clippy and format**

Run: `cargo clippy -p automapper-generator -- -D warnings && cargo fmt --all -- --check`
Expected: no warnings, no format issues

Fix any issues that come up.

**Step 4: Run full workspace tests**

Run: `cargo test --workspace --exclude automapper-web`
Expected: all tests pass

**Step 5: Commit**

```bash
git add crates/automapper-generator/tests/fixture_enhancer_test.rs
git commit -m "test: add integration tests for fixture enhancer pipeline"
```
