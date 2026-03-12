# External Condition Providers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement static and configurable `ExternalConditionProvider` implementations that resolve ~164 of 316 external conditions (code lists, sector, market roles), with format-version-aware code list extraction from MIG XML.

**Architecture:** A new `extract-code-lists` CLI command in `automapper-generator` extracts all code definitions from MIG XML into a per-format-version JSON file. Three new provider structs (`CodeListProvider`, `SectorProvider`, `MarketRoleProvider`) implement `ExternalConditionProvider` and compose via the existing `CompositeExternalProvider`. The `SectorProvider` and `MarketRoleProvider` derive answers from `EvaluationContext` (variant config and NAD segments), while `CodeListProvider` uses the extracted JSON data.

**Tech Stack:** Rust, serde_json, clap (CLI), existing `ExternalConditionProvider` trait

---

## Background

The codebase has 316 external conditions across 17 message types. These call `ctx.external.evaluate("some_name")` and currently all return `Unknown` via `NoOpExternalProvider`. Three categories can be resolved with static/configurable data:

| Category | Conditions | Provider |
|----------|-----------|----------|
| Code list membership | ~46 | `CodeListProvider` — checks if a value belongs to a BDEW/DVGW code list |
| Sector (Strom/Gas) | ~39 | `SectorProvider` — answers based on message type variant |
| Market participant roles | ~79 | `MarketRoleProvider` — derives sender/recipient roles from NAD+MS/MR segments |

### Key Files

- **Trait definition:** `crates/automapper-validation/src/eval/evaluator.rs` — `ExternalConditionProvider`
- **Existing providers:** `crates/automapper-validation/src/eval/providers.rs` — `MapExternalProvider`, `CompositeExternalProvider`, `NoOpExternalProvider`
- **Evaluation context:** `crates/automapper-validation/src/eval/context.rs` — `EvaluationContext`
- **Code extraction:** `crates/automapper-generator/src/codegen/mig_type_gen.rs` — `collect_code_elements()`
- **CLI:** `crates/automapper-generator/src/main.rs` — clap `Commands` enum
- **MIG parsing:** `crates/mig-assembly/src/parsing.rs` — `parse_mig()`, `parse_code_from_xml()`

---

## Task 1: Extract Code Lists CLI Command

Add a `extract-code-lists` subcommand to `automapper-generator` that parses a MIG XML and writes all code definitions to a JSON file. This is reusable across format versions — run it once per MIG XML.

**Files:**
- Modify: `crates/automapper-generator/src/main.rs` — add `ExtractCodeLists` variant to `Commands` enum + handler
- Create: `crates/automapper-generator/src/codegen/code_list_extractor.rs` — extraction logic
- Modify: `crates/automapper-generator/src/codegen/mod.rs` — add `pub mod code_list_extractor;`
- Output: `crates/automapper-validation/src/generated/<fv>/code_lists.json` — generated artifact

### Step 1: Create the extractor module

Create `crates/automapper-generator/src/codegen/code_list_extractor.rs`:

```rust
//! Extracts all code lists from a MIG XML schema into a flat JSON file.
//!
//! Output format: `{ "<data_element_id>": { "name": "...", "codes": ["value1", "value2", ...] } }`
//!
//! This is run once per format version and the output is committed as a generated artifact.

use std::collections::BTreeMap;
use std::path::Path;

use mig_types::schema::common::CodeDefinition;
use mig_types::schema::mig::MigSchema;

use serde::{Deserialize, Serialize};

/// A single code list entry for a data element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeListEntry {
    /// Human-readable name of the data element (e.g., "Nachrichtentyp-Kennung").
    pub name: String,
    /// All valid code values for this data element.
    pub codes: Vec<CodeValueEntry>,
}

/// A single code value with its description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeValueEntry {
    /// The code value (e.g., "UTILMD", "Z16").
    pub value: String,
    /// Human-readable name/description.
    pub name: String,
}

/// Extract all code lists from a parsed MIG schema.
///
/// Returns a map from data element ID (e.g., "3225", "7111") to its code list.
/// Only data elements with at least one code are included.
pub fn extract_code_lists(mig: &MigSchema) -> BTreeMap<String, CodeListEntry> {
    let mut result: BTreeMap<String, CodeListEntry> = BTreeMap::new();

    fn visit_codes(
        de_id: &str,
        de_name: &str,
        codes: &[CodeDefinition],
        result: &mut BTreeMap<String, CodeListEntry>,
    ) {
        if codes.is_empty() {
            return;
        }
        let entry = result.entry(de_id.to_string()).or_insert_with(|| CodeListEntry {
            name: de_name.to_string(),
            codes: Vec::new(),
        });
        for code in codes {
            if !entry.codes.iter().any(|c| c.value == code.value) {
                entry.codes.push(CodeValueEntry {
                    value: code.value.clone(),
                    name: code.name.clone(),
                });
            }
        }
    }

    // Visit all segments recursively
    for seg in &mig.segments {
        visit_segment_codes(seg, &mut result, &visit_codes);
    }
    for group in &mig.segment_groups {
        visit_group_codes(group, &mut result, &visit_codes);
    }

    result
}

fn visit_segment_codes(
    seg: &mig_types::schema::mig::MigSegment,
    result: &mut BTreeMap<String, CodeListEntry>,
    visit: &dyn Fn(&str, &str, &[CodeDefinition], &mut BTreeMap<String, CodeListEntry>),
) {
    for de in &seg.data_elements {
        visit(&de.id, &de.name, &de.codes, result);
    }
    for comp in &seg.composites {
        for de in &comp.data_elements {
            visit(&de.id, &de.name, &de.codes, result);
        }
    }
}

fn visit_group_codes(
    group: &mig_types::schema::mig::MigSegmentGroup,
    result: &mut BTreeMap<String, CodeListEntry>,
    visit: &dyn Fn(&str, &str, &[CodeDefinition], &mut BTreeMap<String, CodeListEntry>),
) {
    for seg in &group.segments {
        visit_segment_codes(seg, result, visit);
    }
    for child in &group.nested_groups {
        visit_group_codes(child, result, visit);
    }
}

/// Write extracted code lists to a JSON file.
pub fn write_code_lists(
    code_lists: &BTreeMap<String, CodeListEntry>,
    output_path: &Path,
) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(code_lists)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, json)
}
```

### Step 2: Register the module

Add to `crates/automapper-generator/src/codegen/mod.rs`:

```rust
pub mod code_list_extractor;
```

### Step 3: Add CLI subcommand

Add to `Commands` enum in `crates/automapper-generator/src/main.rs`:

```rust
    /// Extract all code lists from a MIG XML into a JSON file.
    /// Run once per format version; output is committed as generated artifact.
    ExtractCodeLists {
        /// Path to MIG XML file
        #[arg(long)]
        mig_path: PathBuf,

        /// Output JSON file path (e.g., crates/automapper-validation/src/generated/fv2504/code_lists.json)
        #[arg(long)]
        output: PathBuf,
    },
```

Add the handler in the `match cli.command` block:

```rust
        Commands::ExtractCodeLists { mig_path, output } => {
            let mig = mig_assembly::parsing::parse_mig(&mig_path)?;
            let code_lists = automapper_generator::codegen::code_list_extractor::extract_code_lists(&mig);
            automapper_generator::codegen::code_list_extractor::write_code_lists(&code_lists, &output)?;
            println!(
                "Extracted {} code lists ({} total codes) to {}",
                code_lists.len(),
                code_lists.values().map(|e| e.codes.len()).sum::<usize>(),
                output.display()
            );
        }
```

### Step 4: Run for FV2504 message types

Generate code lists for each MIG XML. Example commands:

```bash
# UTILMD Strom
cargo run -p automapper-generator -- extract-code-lists \
  --mig-path xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml \
  --output crates/automapper-validation/src/generated/fv2504/utilmd_strom_code_lists.json

# ORDERS
cargo run -p automapper-generator -- extract-code-lists \
  --mig-path xml-migs-and-ahbs/FV2504/ORDERS_MIG_S2_1_Fehlerkorrektur_20250320.xml \
  --output crates/automapper-validation/src/generated/fv2504/orders_code_lists.json

# Repeat for each message type with conditions...
```

### Step 5: Verify output

```bash
# Check output is valid JSON with expected structure
python3 -c "
import json, sys
d = json.load(open(sys.argv[1]))
print(f'{len(d)} data elements with code lists')
for k, v in sorted(d.items())[:5]:
    print(f'  D{k} ({v[\"name\"]}): {len(v[\"codes\"])} codes')
" crates/automapper-validation/src/generated/fv2504/utilmd_strom_code_lists.json
```

### Step 6: Commit

```bash
git add crates/automapper-generator/src/codegen/code_list_extractor.rs \
       crates/automapper-generator/src/codegen/mod.rs \
       crates/automapper-generator/src/main.rs \
       crates/automapper-validation/src/generated/fv2504/*_code_lists.json
git commit -m "feat(generator): add extract-code-lists CLI for format-version-portable code list extraction"
```

---

## Task 2: CodeListProvider

Implement a provider that loads code list JSON and answers `code_list_membership_check`-style conditions.

**Files:**
- Modify: `crates/automapper-validation/src/eval/providers.rs` — add `CodeListProvider`
- Test: `crates/automapper-validation/src/eval/providers.rs` (inline `#[cfg(test)]` module)

### Step 1: Write failing tests

Add to `providers.rs` `#[cfg(test)]` module:

```rust
    #[test]
    fn test_code_list_provider_known_code() {
        let mut lists = std::collections::HashMap::new();
        lists.insert(
            "7111".to_string(),
            vec!["Z91".to_string(), "Z90".to_string(), "ZF0".to_string()],
        );
        let provider = CodeListProvider::new(lists);
        // Condition name format: "code_in_<de_id>:<value>"
        assert_eq!(
            provider.evaluate("code_in_7111:Z91"),
            ConditionResult::True
        );
        assert_eq!(
            provider.evaluate("code_in_7111:ZZZ"),
            ConditionResult::False
        );
        assert_eq!(
            provider.evaluate("code_in_9999:Z91"),
            ConditionResult::Unknown, // unknown list
        );
    }

    #[test]
    fn test_code_list_provider_loads_json() {
        let json = r#"{
            "7111": { "name": "Eigenschaft", "codes": [{"value": "Z91", "name": "MSB"}] },
            "3225": { "name": "Ort", "codes": [{"value": "Z16", "name": "MaLo"}] }
        }"#;
        let provider = CodeListProvider::from_json(json).unwrap();
        assert_eq!(provider.evaluate("code_in_7111:Z91"), ConditionResult::True);
        assert_eq!(provider.evaluate("code_in_3225:Z16"), ConditionResult::True);
        assert_eq!(provider.evaluate("code_in_3225:Z99"), ConditionResult::False);
    }
```

### Step 2: Run tests to verify they fail

```bash
cargo test -p automapper-validation -- code_list_provider
```
Expected: FAIL (type not found)

### Step 3: Implement CodeListProvider

Add to `providers.rs`:

```rust
use std::collections::{HashMap, HashSet};

/// Provider that checks whether a value belongs to a known code list.
///
/// Condition name format: `"code_in_<data_element_id>:<value>"`
/// Returns True if the value is in the list, False if not, Unknown if the list is unknown.
pub struct CodeListProvider {
    /// Map from data element ID to set of valid code values.
    lists: HashMap<String, HashSet<String>>,
}

impl CodeListProvider {
    /// Create from pre-built lists (data element ID → valid code values).
    pub fn new(lists: HashMap<String, Vec<String>>) -> Self {
        Self {
            lists: lists
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
        }
    }

    /// Load from the JSON format produced by `extract-code-lists` CLI.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        // Matches CodeListEntry structure from code_list_extractor
        #[derive(serde::Deserialize)]
        struct Entry {
            codes: Vec<CodeValue>,
        }
        #[derive(serde::Deserialize)]
        struct CodeValue {
            value: String,
        }

        let raw: HashMap<String, Entry> = serde_json::from_str(json)?;
        let lists = raw
            .into_iter()
            .map(|(k, v)| (k, v.codes.into_iter().map(|c| c.value).collect()))
            .collect();
        Ok(Self { lists })
    }

    /// Load from a JSON file path.
    pub fn from_json_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&json)?)
    }
}

impl ExternalConditionProvider for CodeListProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        // Parse "code_in_<de_id>:<value>"
        let rest = match condition_name.strip_prefix("code_in_") {
            Some(r) => r,
            None => return ConditionResult::Unknown,
        };
        let (de_id, value) = match rest.split_once(':') {
            Some(pair) => pair,
            None => return ConditionResult::Unknown,
        };
        match self.lists.get(de_id) {
            Some(set) => ConditionResult::from(set.contains(value)),
            None => ConditionResult::Unknown,
        }
    }
}
```

### Step 4: Run tests to verify they pass

```bash
cargo test -p automapper-validation -- code_list_provider
```

### Step 5: Commit

```bash
git add crates/automapper-validation/src/eval/providers.rs
git commit -m "feat(validation): add CodeListProvider for code list membership checks"
```

---

## Task 3: SectorProvider

Simple provider that answers sector-based conditions from a configured variant.

**Files:**
- Modify: `crates/automapper-validation/src/eval/providers.rs` — add `SectorProvider`

### Step 1: Write failing tests

```rust
    #[test]
    fn test_sector_provider_strom() {
        let provider = SectorProvider::new(Sector::Strom);
        assert_eq!(provider.evaluate("recipient_is_strom"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_is_gas"), ConditionResult::False);
        assert_eq!(provider.evaluate("mp_id_is_strom"), ConditionResult::True);
        assert_eq!(provider.evaluate("mp_id_is_gas"), ConditionResult::False);
        assert_eq!(provider.evaluate("market_location_is_electricity"), ConditionResult::True);
        assert_eq!(provider.evaluate("market_location_is_gas"), ConditionResult::False);
        assert_eq!(provider.evaluate("unrelated_condition"), ConditionResult::Unknown);
    }

    #[test]
    fn test_sector_provider_gas() {
        let provider = SectorProvider::new(Sector::Gas);
        assert_eq!(provider.evaluate("recipient_is_strom"), ConditionResult::False);
        assert_eq!(provider.evaluate("recipient_is_gas"), ConditionResult::True);
    }
```

### Step 2: Run tests to verify they fail

```bash
cargo test -p automapper-validation -- sector_provider
```

### Step 3: Implement SectorProvider

```rust
/// Energy sector (Strom = electricity, Gas = natural gas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sector {
    Strom,
    Gas,
}

/// Provider that resolves sector-based conditions from deployment configuration.
///
/// Handles conditions like `recipient_is_strom`, `recipient_is_gas`,
/// `mp_id_is_strom`, `market_location_is_electricity`, etc.
pub struct SectorProvider {
    sector: Sector,
}

impl SectorProvider {
    pub fn new(sector: Sector) -> Self {
        Self { sector }
    }

    /// Create from variant string (e.g., "Strom", "Gas").
    pub fn from_variant(variant: &str) -> Option<Self> {
        match variant {
            "Strom" => Some(Self::new(Sector::Strom)),
            "Gas" => Some(Self::new(Sector::Gas)),
            _ => None,
        }
    }
}

/// Condition names that indicate Strom sector.
const STROM_CONDITIONS: &[&str] = &[
    "recipient_is_strom",
    "recipient_market_sector_is_strom",
    "sender_is_strom",
    "mp_id_is_strom",
    "market_location_is_electricity",
    "location_is_strom",
    "metering_point_is_strom",
    "network_location_is_strom",
];

/// Condition names that indicate Gas sector.
const GAS_CONDITIONS: &[&str] = &[
    "recipient_is_gas",
    "recipient_market_sector_is_gas",
    "sender_is_gas",
    "mp_id_is_gas",
    "market_location_is_gas",
    "location_is_gas",
    "metering_point_is_gas",
    "network_location_is_gas",
    "recipient_is_msb_gas",
];

impl ExternalConditionProvider for SectorProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        if STROM_CONDITIONS.contains(&condition_name) {
            return ConditionResult::from(self.sector == Sector::Strom);
        }
        if GAS_CONDITIONS.contains(&condition_name) {
            return ConditionResult::from(self.sector == Sector::Gas);
        }
        ConditionResult::Unknown
    }
}
```

### Step 4: Run tests, commit

```bash
cargo test -p automapper-validation -- sector_provider
git add crates/automapper-validation/src/eval/providers.rs
git commit -m "feat(validation): add SectorProvider for Strom/Gas sector conditions"
```

---

## Task 4: MarketRoleProvider

Provider that resolves sender/recipient role conditions. Configured with known roles at construction time (from deployment config or NAD analysis).

**Files:**
- Modify: `crates/automapper-validation/src/eval/providers.rs` — add `MarketRoleProvider`

### Step 1: Write failing tests

```rust
    #[test]
    fn test_market_role_provider() {
        let provider = MarketRoleProvider::new(
            vec![MarketRole::LF],           // sender roles
            vec![MarketRole::NB, MarketRole::MSB], // recipient roles
        );
        assert_eq!(provider.evaluate("sender_is_lf"), ConditionResult::True);
        assert_eq!(provider.evaluate("sender_is_nb"), ConditionResult::False);
        assert_eq!(provider.evaluate("recipient_is_nb"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_is_msb"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_is_lf"), ConditionResult::False);
        assert_eq!(provider.evaluate("sender_is_uenb"), ConditionResult::False);
        assert_eq!(provider.evaluate("unrelated"), ConditionResult::Unknown);
    }

    #[test]
    fn test_market_role_provider_negated() {
        let provider = MarketRoleProvider::new(
            vec![MarketRole::MSB],
            vec![MarketRole::NB],
        );
        assert_eq!(provider.evaluate("sender_role_is_not_msb"), ConditionResult::False);
        assert_eq!(provider.evaluate("sender_role_is_not_lf"), ConditionResult::True);
        assert_eq!(provider.evaluate("recipient_role_is_not_nb"), ConditionResult::False);
    }
```

### Step 2: Run tests to verify they fail

```bash
cargo test -p automapper-validation -- market_role_provider
```

### Step 3: Implement MarketRoleProvider

```rust
/// Market participant roles in the German energy market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketRole {
    /// Lieferant (supplier)
    LF,
    /// Netzbetreiber (network operator)
    NB,
    /// Messstellenbetreiber (metering point operator)
    MSB,
    /// Messdienstleister (metering service provider)
    MDL,
    /// Übertragungsnetzbetreiber (transmission system operator)
    UENB,
    /// Bilanzkreisverantwortlicher (balance responsible party)
    BKV,
    /// Bilanzkoordinator (balance coordinator)
    BIKO,
    /// Einsatzverantwortlicher (dispatch responsible)
    ESA,
}

impl MarketRole {
    /// Parse from condition name suffix (e.g., "lf" → LF, "nb" → NB).
    fn from_suffix(s: &str) -> Option<Self> {
        match s {
            "lf" => Some(Self::LF),
            "nb" => Some(Self::NB),
            "msb" => Some(Self::MSB),
            "mdl" => Some(Self::MDL),
            "uenb" => Some(Self::UENB),
            "bkv" => Some(Self::BKV),
            "biko" => Some(Self::BIKO),
            "esa" => Some(Self::ESA),
            _ => None,
        }
    }
}

/// Provider that resolves sender/recipient market role conditions.
///
/// Handles patterns like:
/// - `sender_is_<role>` / `recipient_is_<role>` → True if role matches
/// - `sender_role_is_not_<role>` / `recipient_role_is_not_<role>` → negated
pub struct MarketRoleProvider {
    sender_roles: HashSet<MarketRole>,
    recipient_roles: HashSet<MarketRole>,
}

impl MarketRoleProvider {
    pub fn new(sender_roles: Vec<MarketRole>, recipient_roles: Vec<MarketRole>) -> Self {
        Self {
            sender_roles: sender_roles.into_iter().collect(),
            recipient_roles: recipient_roles.into_iter().collect(),
        }
    }
}

impl ExternalConditionProvider for MarketRoleProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        // Pattern: sender_is_<role>
        if let Some(role_str) = condition_name.strip_prefix("sender_is_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(self.sender_roles.contains(&role));
            }
        }
        // Pattern: recipient_is_<role>
        if let Some(role_str) = condition_name.strip_prefix("recipient_is_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(self.recipient_roles.contains(&role));
            }
        }
        // Pattern: sender_role_is_not_<role>
        if let Some(role_str) = condition_name.strip_prefix("sender_role_is_not_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(!self.sender_roles.contains(&role));
            }
        }
        // Pattern: recipient_role_is_not_<role>
        if let Some(role_str) = condition_name.strip_prefix("recipient_role_is_not_") {
            if let Some(role) = MarketRole::from_suffix(role_str) {
                return ConditionResult::from(!self.recipient_roles.contains(&role));
            }
        }
        ConditionResult::Unknown
    }
}
```

### Step 4: Run tests, commit

```bash
cargo test -p automapper-validation -- market_role_provider
git add crates/automapper-validation/src/eval/providers.rs
git commit -m "feat(validation): add MarketRoleProvider for sender/recipient role conditions"
```

---

## Task 5: Convenience Constructor for CompositeExternalProvider

Add a builder method to compose all static providers into one.

**Files:**
- Modify: `crates/automapper-validation/src/eval/providers.rs`

### Step 1: Write failing test

```rust
    #[test]
    fn test_composite_with_static_providers() {
        let composite = CompositeExternalProvider::with_defaults(
            Some(Sector::Strom),
            Some((vec![MarketRole::LF], vec![MarketRole::NB])),
            None, // no code list file
        );
        // Sector resolved
        assert_eq!(composite.evaluate("recipient_is_strom"), ConditionResult::True);
        // Market role resolved
        assert_eq!(composite.evaluate("sender_is_lf"), ConditionResult::True);
        // Unknown conditions still return Unknown
        assert_eq!(composite.evaluate("some_unknown"), ConditionResult::Unknown);
    }
```

### Step 2: Implement

```rust
impl CompositeExternalProvider {
    /// Build a composite provider with the standard static providers.
    ///
    /// - `sector`: If Some, adds a SectorProvider
    /// - `roles`: If Some((sender_roles, recipient_roles)), adds a MarketRoleProvider
    /// - `code_list_json`: If Some, adds a CodeListProvider from JSON string
    pub fn with_defaults(
        sector: Option<Sector>,
        roles: Option<(Vec<MarketRole>, Vec<MarketRole>)>,
        code_list_json: Option<&str>,
    ) -> Self {
        let mut providers: Vec<Box<dyn ExternalConditionProvider>> = Vec::new();

        if let Some(sector) = sector {
            providers.push(Box::new(SectorProvider::new(sector)));
        }
        if let Some((sender, recipient)) = roles {
            providers.push(Box::new(MarketRoleProvider::new(sender, recipient)));
        }
        if let Some(json) = code_list_json {
            if let Ok(provider) = CodeListProvider::from_json(json) {
                providers.push(Box::new(provider));
            }
        }

        Self::new(providers)
    }
}
```

### Step 3: Run tests, commit

```bash
cargo test -p automapper-validation -- composite_with_static
git add crates/automapper-validation/src/eval/providers.rs
git commit -m "feat(validation): add CompositeExternalProvider::with_defaults builder"
```

---

## Task 6: Re-export new types from eval module

**Files:**
- Modify: `crates/automapper-validation/src/eval.rs` — add re-exports

### Step 1: Add re-exports

Add to existing re-exports in `eval.rs`:

```rust
pub use providers::{CodeListProvider, SectorProvider, MarketRoleProvider, Sector, MarketRole};
```

### Step 2: Verify compilation

```bash
cargo check -p automapper-validation
```

### Step 3: Commit

```bash
git add crates/automapper-validation/src/eval.rs
git commit -m "feat(validation): re-export new provider types from eval module"
```

---

## Task 7: Generate Code Lists for All FV2504 Message Types

Run the CLI for each message type that has conditions. This creates the reusable JSON artifacts.

**Files:**
- Create: `crates/automapper-validation/src/generated/fv2504/*_code_lists.json` (multiple files)

### Step 1: Generate code lists

```bash
# Find all MIG XMLs for FV2504
for mig_xml in xml-migs-and-ahbs/FV2504/*_MIG_*.xml; do
  # Derive output name from filename
  basename=$(basename "$mig_xml" .xml | tr '[:upper:]' '[:lower:]' | sed 's/_mig_/_/' | sed 's/_s2_1.*$//')
  output="crates/automapper-validation/src/generated/fv2504/${basename}_code_lists.json"
  echo "Extracting: $mig_xml -> $output"
  cargo run -p automapper-generator -- extract-code-lists \
    --mig-path "$mig_xml" \
    --output "$output"
done
```

### Step 2: Verify all files generated

```bash
ls -la crates/automapper-validation/src/generated/fv2504/*_code_lists.json
# Should have one file per message type
```

### Step 3: Commit

```bash
git add crates/automapper-validation/src/generated/fv2504/*_code_lists.json
git commit -m "feat(validation): generate FV2504 code lists from MIG XML"
```

---

## Task 8: Inventory External Condition Names

Catalog all `ctx.external.evaluate("...")` names across generated code to verify our providers cover them.

### Step 1: Extract all external names

```bash
grep -roh 'ctx\.external\.evaluate("[^"]*")' \
  crates/automapper-validation/src/generated/fv2504/ \
  | sed 's/ctx\.external\.evaluate("\(.*\)")/\1/' \
  | sort -u > /tmp/external_names.txt

echo "Total unique names:"
wc -l /tmp/external_names.txt

echo "Covered by SectorProvider:"
grep -cE '(recipient_is_strom|recipient_is_gas|sender_is_strom|sender_is_gas|mp_id_is_strom|mp_id_is_gas|market_location_is_electricity|market_location_is_gas|recipient_market_sector_is_strom|recipient_market_sector_is_gas)' /tmp/external_names.txt || echo "0"

echo "Covered by MarketRoleProvider:"
grep -cE '(sender_is_(lf|nb|msb|mdl|uenb|bkv|biko|esa)|recipient_is_(lf|nb|msb|mdl|uenb|bkv|biko|esa)|sender_role_is_not_|recipient_role_is_not_)' /tmp/external_names.txt || echo "0"

echo "Remaining uncovered:"
grep -vE '(recipient_is_strom|recipient_is_gas|sender_is_strom|sender_is_gas|mp_id_is_|market_location_is_|recipient_market_sector_is_|sender_is_(lf|nb|msb|mdl|uenb|bkv|biko|esa)|recipient_is_(lf|nb|msb|mdl|uenb|bkv|biko|esa)|sender_role_is_not_|recipient_role_is_not_|code_in_)' /tmp/external_names.txt
```

### Step 2: Update provider condition name lists

Based on the inventory, update `STROM_CONDITIONS`, `GAS_CONDITIONS`, and `MarketRole::from_suffix` to cover all actual names used in generated code. The exact names will be known after running Step 1.

### Step 3: Re-run tests, commit any updates

```bash
cargo test -p automapper-validation
git add crates/automapper-validation/src/eval/providers.rs
git commit -m "fix(validation): align provider condition names with generated code"
```

---

## Task 9: Integration Test

End-to-end test that wires up all providers and evaluates a real condition expression.

**Files:**
- Create: `crates/automapper-validation/tests/external_providers_test.rs`

### Step 1: Write integration test

```rust
//! Integration test for external condition providers.

use automapper_validation::eval::{
    CodeListProvider, CompositeExternalProvider, MarketRole, MarketRoleProvider, Sector,
    SectorProvider,
};
use automapper_validation::eval::evaluator::{ConditionResult, ExternalConditionProvider};

#[test]
fn test_composite_provider_resolves_all_categories() {
    let sector = SectorProvider::new(Sector::Strom);
    let roles = MarketRoleProvider::new(vec![MarketRole::LF], vec![MarketRole::NB]);

    let composite = CompositeExternalProvider::new(vec![
        Box::new(sector),
        Box::new(roles),
    ]);

    // Sector conditions
    assert_eq!(composite.evaluate("recipient_is_strom"), ConditionResult::True);
    assert_eq!(composite.evaluate("recipient_is_gas"), ConditionResult::False);

    // Role conditions
    assert_eq!(composite.evaluate("sender_is_lf"), ConditionResult::True);
    assert_eq!(composite.evaluate("recipient_is_nb"), ConditionResult::True);
    assert_eq!(composite.evaluate("sender_is_nb"), ConditionResult::False);

    // Unknown conditions pass through
    assert_eq!(composite.evaluate("message_splitting"), ConditionResult::Unknown);
}
```

### Step 2: Run test

```bash
cargo test -p automapper-validation --test external_providers_test
```

### Step 3: Commit

```bash
git add crates/automapper-validation/tests/external_providers_test.rs
git commit -m "test(validation): add integration test for composite external providers"
```

---

## Format Version Migration Notes

When a new format version arrives (e.g., FV2510 → FV2604):

1. **Code lists**: Run `extract-code-lists` against the new MIG XML. The CLI is format-version-agnostic — it works on any MIG XML file.

```bash
cargo run -p automapper-generator -- extract-code-lists \
  --mig-path xml-migs-and-ahbs/FV2604/UTILMD_MIG_Strom_....xml \
  --output crates/automapper-validation/src/generated/fv2604/utilmd_strom_code_lists.json
```

2. **SectorProvider**: No changes needed — sector concept is format-version-independent.

3. **MarketRoleProvider**: No changes needed — role definitions are format-version-independent.

4. **Condition name alignment**: After regenerating conditions for the new FV, re-run Task 8's inventory script to check if new external condition names appear. Update provider name lists if needed.

---

## Summary

| Task | What | Conditions Covered | Effort |
|------|------|-------------------|--------|
| 1 | Extract code lists CLI | Infrastructure | Medium |
| 2 | CodeListProvider | ~46 | Small |
| 3 | SectorProvider | ~39 | Small |
| 4 | MarketRoleProvider | ~79 | Small |
| 5 | Composite builder | Convenience | Small |
| 6 | Re-exports | API surface | Trivial |
| 7 | Generate FV2504 code lists | Data | Small |
| 8 | Name inventory | Alignment | Small |
| 9 | Integration test | Verification | Small |

**Total: ~164 of 316 external conditions resolved (52%)**
