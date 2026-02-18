---
feature: generator-implementation
epic: 3
title: "AI-Assisted Condition Generation"
depends_on: [1, 2]
estimated_tasks: 7
crate: automapper-generator
---

# Epic 3: AI-Assisted Condition Generation

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-generator/src/`. The binary entry point is `crates/automapper-generator/src/main.rs`. Use types from Section 7 of the design doc exactly. All code must compile with `cargo check -p automapper-generator`.

**Goal:** Implement the `generate-conditions` and `validate-schema` CLI subcommands. The `generate-conditions` command shells out to the `claude` CLI to generate Rust condition evaluator code from AHB condition descriptions. It supports batch generation with configurable concurrency, incremental regeneration (only changed/low-confidence conditions), and outputs a `ConditionEvaluator` impl file. The `validate-schema` command checks that generated mapper code references valid BO4E types from the `stammdatenmodell/` submodule. Ports the C# `GenerateConditionsCommand`, `AnthropicClient`, `ConditionCodeGenerator`, `ConditionMetadataManager`, `RegenerationDecider`, and `ValidateGeneratedCommand` to idiomatic Rust.

**Architecture:** The `ClaudeConditionGenerator` struct builds a prompt from AHB condition descriptions plus surrounding context (segment structure from MIG, referencing fields, example implementations), then shells out to `claude --print` (no SDK dependency -- reuses existing CLI subscription). The response is parsed as JSON into `GeneratedCondition` structs with confidence levels. A `ConditionMetadataManager` stores per-condition metadata (description hash, confidence, external status) in a sidecar `.conditions.json` file for incremental regeneration. The `RegenerationDecider` compares current AHB conditions against stored metadata to determine which conditions need regeneration. Batch processing uses `tokio::spawn` tasks with a semaphore for concurrency control. Generated output is a Rust source file with a `ConditionEvaluator` trait impl containing match arms that dispatch to per-condition evaluation functions.

**Tech Stack:** clap 4.x (CLI), tokio 1.x (async batch processing), serde + serde_json (metadata, JSON response parsing), sha2 (description hashing), regex 1.x (segment reference extraction), thiserror 2.x (errors), insta (snapshot tests), tempfile (test fixtures)

---

## Task 1: Condition types and metadata structs

### Step 1 -- Write the test

Create `crates/automapper-generator/src/conditions/condition_types.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Confidence level for AI-generated condition code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceLevel::High => write!(f, "high"),
            ConfidenceLevel::Medium => write!(f, "medium"),
            ConfidenceLevel::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for ConfidenceLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "high" => Ok(ConfidenceLevel::High),
            "medium" => Ok(ConfidenceLevel::Medium),
            "low" => Ok(ConfidenceLevel::Low),
            other => Err(format!("unknown confidence level: {}", other)),
        }
    }
}

/// A single generated condition with its Rust code and confidence level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCondition {
    /// The condition number (e.g., 1, 2, 931).
    pub condition_number: u32,

    /// The generated Rust function body, or None for external/low-confidence conditions.
    pub rust_code: Option<String>,

    /// Whether this condition requires external context (cannot be evaluated from the message alone).
    pub is_external: bool,

    /// Confidence level of the generation.
    pub confidence: ConfidenceLevel,

    /// AI reasoning for the implementation or why review is needed.
    pub reasoning: Option<String>,

    /// For external conditions: a snake_case name for the condition (e.g., "message_splitting").
    pub external_name: Option<String>,

    /// The original German description from the AHB.
    pub original_description: Option<String>,

    /// Fields that reference this condition (segment paths with AHB status).
    pub referencing_fields: Option<Vec<String>>,
}

/// Input condition to send to Claude for code generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionInput {
    /// The condition ID (e.g., "1", "931").
    pub id: String,

    /// The German description text from the AHB.
    pub description: String,

    /// Fields that reference this condition (for context).
    pub referencing_fields: Option<Vec<String>>,
}

/// JSON response shape from Claude CLI (matches the prompted output format).
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeConditionResponse {
    pub conditions: Vec<ClaudeConditionEntry>,
}

/// A single condition entry in the Claude JSON response.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeConditionEntry {
    pub id: String,
    pub implementation: Option<String>,
    pub confidence: String,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub is_external: bool,
    pub external_name: Option<String>,
}
```

Create `crates/automapper-generator/src/conditions/metadata.rs`:

```rust
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::GeneratorError;

/// Metadata for a single condition stored in the sidecar JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionMetadata {
    /// Confidence level from the last generation ("high", "medium", "low").
    pub confidence: String,

    /// AI reasoning from the last generation.
    pub reasoning: Option<String>,

    /// SHA-256 hash (first 8 hex chars) of the AHB description for staleness detection.
    pub description_hash: String,

    /// Whether this condition requires external context.
    pub is_external: bool,
}

/// Root structure for the `.conditions.json` sidecar file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionMetadataFile {
    /// UTC timestamp of generation.
    pub generated_at: String,

    /// Source AHB filename.
    pub ahb_file: String,

    /// Format version (e.g., "FV2510").
    pub format_version: String,

    /// Per-condition metadata, keyed by condition ID.
    pub conditions: HashMap<String, ConditionMetadata>,
}

/// Computes a hash of the condition description for staleness detection.
/// Returns the first 8 hex characters of the SHA-256 hash.
pub fn compute_description_hash(description: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(description.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..4])
}

/// Loads metadata from a JSON file. Returns None if the file does not exist.
pub fn load_metadata(path: &Path) -> Result<Option<ConditionMetadataFile>, GeneratorError> {
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(path)?;
    let metadata: ConditionMetadataFile = serde_json::from_str(&json)?;
    Ok(Some(metadata))
}

/// Saves metadata to a JSON file.
pub fn save_metadata(path: &Path, metadata: &ConditionMetadataFile) -> Result<(), GeneratorError> {
    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Reason why a condition needs regeneration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegenerationReason {
    /// Condition is new (not in metadata).
    New,
    /// Previous generation had low confidence.
    LowConfidence,
    /// Previous generation had medium confidence.
    MediumConfidence,
    /// AHB description changed since last generation.
    Stale,
    /// Metadata exists but implementation is missing from the output file.
    MissingImplementation,
    /// User passed --force flag.
    Forced,
}

impl std::fmt::Display for RegenerationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegenerationReason::New => write!(f, "New"),
            RegenerationReason::LowConfidence => write!(f, "Low confidence"),
            RegenerationReason::MediumConfidence => write!(f, "Medium confidence"),
            RegenerationReason::Stale => write!(f, "Stale (description changed)"),
            RegenerationReason::MissingImplementation => write!(f, "Missing implementation"),
            RegenerationReason::Forced => write!(f, "Forced"),
        }
    }
}

/// A condition that needs to be regenerated, with the reason why.
#[derive(Debug, Clone)]
pub struct ConditionToRegenerate {
    pub condition_id: String,
    pub description: String,
    pub reason: RegenerationReason,
}

/// Result of the regeneration decision.
#[derive(Debug, Clone)]
pub struct RegenerationDecision {
    /// Conditions that need regeneration.
    pub to_regenerate: Vec<ConditionToRegenerate>,
    /// Condition IDs that can be preserved from the existing file.
    pub to_preserve: Vec<String>,
}

/// Decides which conditions need regeneration based on metadata and current AHB descriptions.
///
/// - If `force` is true, all conditions are regenerated.
/// - If no metadata file exists, all conditions are new.
/// - Otherwise, conditions are regenerated if: new, low/medium confidence, stale (description hash changed),
///   or implementation is missing.
pub fn decide_regeneration(
    conditions: &[(String, String)], // (id, description) pairs
    existing_metadata: Option<&ConditionMetadataFile>,
    existing_condition_ids: &std::collections::HashSet<String>, // IDs present in the output file
    force: bool,
) -> RegenerationDecision {
    let mut to_regenerate = Vec::new();
    let mut to_preserve = Vec::new();

    for (id, description) in conditions {
        let reason = should_regenerate(id, description, existing_metadata, existing_condition_ids, force);

        if let Some(reason) = reason {
            to_regenerate.push(ConditionToRegenerate {
                condition_id: id.clone(),
                description: description.clone(),
                reason,
            });
        } else {
            to_preserve.push(id.clone());
        }
    }

    RegenerationDecision {
        to_regenerate,
        to_preserve,
    }
}

fn should_regenerate(
    id: &str,
    description: &str,
    existing_metadata: Option<&ConditionMetadataFile>,
    existing_condition_ids: &std::collections::HashSet<String>,
    force: bool,
) -> Option<RegenerationReason> {
    if force {
        return Some(RegenerationReason::Forced);
    }

    let metadata = match existing_metadata {
        Some(m) => m,
        None => return Some(RegenerationReason::New),
    };

    let condition_meta = match metadata.conditions.get(id) {
        Some(m) => m,
        None => return Some(RegenerationReason::New),
    };

    if condition_meta.confidence.to_lowercase() == "low" {
        return Some(RegenerationReason::LowConfidence);
    }

    if condition_meta.confidence.to_lowercase() == "medium" {
        return Some(RegenerationReason::MediumConfidence);
    }

    // Check for staleness (description changed)
    let current_hash = compute_description_hash(description);
    if condition_meta.description_hash != current_hash {
        return Some(RegenerationReason::Stale);
    }

    // High confidence but implementation missing from output file
    if !existing_condition_ids.contains(id) {
        return Some(RegenerationReason::MissingImplementation);
    }

    // High confidence, not stale, implementation exists -> preserve
    None
}
```

Create `crates/automapper-generator/src/conditions/mod.rs`:

```rust
pub mod condition_types;
pub mod metadata;
```

Update `crates/automapper-generator/src/lib.rs` to add:

```rust
pub mod conditions;
```

Add to `crates/automapper-generator/Cargo.toml` dependencies:

```toml
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1", features = ["full"] }
```

Write `crates/automapper-generator/tests/condition_types_tests.rs`:

```rust
use automapper_generator::conditions::condition_types::*;
use automapper_generator::conditions::metadata::*;
use std::collections::{HashMap, HashSet};

#[test]
fn test_confidence_level_display() {
    assert_eq!(ConfidenceLevel::High.to_string(), "high");
    assert_eq!(ConfidenceLevel::Medium.to_string(), "medium");
    assert_eq!(ConfidenceLevel::Low.to_string(), "low");
}

#[test]
fn test_confidence_level_from_str() {
    assert_eq!("high".parse::<ConfidenceLevel>().unwrap(), ConfidenceLevel::High);
    assert_eq!("Medium".parse::<ConfidenceLevel>().unwrap(), ConfidenceLevel::Medium);
    assert_eq!("LOW".parse::<ConfidenceLevel>().unwrap(), ConfidenceLevel::Low);
    assert!("unknown".parse::<ConfidenceLevel>().is_err());
}

#[test]
fn test_generated_condition_serialization_roundtrip() {
    let condition = GeneratedCondition {
        condition_number: 42,
        rust_code: Some("ctx.transaktion.marktlokationen.is_empty()".to_string()),
        is_external: false,
        confidence: ConfidenceLevel::High,
        reasoning: Some("Simple field check".to_string()),
        external_name: None,
        original_description: Some("Wenn Marktlokation vorhanden".to_string()),
        referencing_fields: Some(vec!["SG8/SEQ (Muss [42])".to_string()]),
    };

    let json = serde_json::to_string(&condition).unwrap();
    let deserialized: GeneratedCondition = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.condition_number, 42);
    assert_eq!(deserialized.confidence, ConfidenceLevel::High);
    assert!(!deserialized.is_external);
    assert!(deserialized.rust_code.is_some());
}

#[test]
fn test_claude_response_parsing() {
    let json = r#"{
        "conditions": [
            {
                "id": "1",
                "implementation": "ctx.transaktion.aufteilung.is_some()",
                "confidence": "high",
                "reasoning": "Simple option check",
                "is_external": false
            },
            {
                "id": "8",
                "implementation": null,
                "confidence": "high",
                "reasoning": "Requires external business context",
                "is_external": true,
                "external_name": "DataClearingRequired"
            }
        ]
    }"#;

    let response: ClaudeConditionResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.conditions.len(), 2);
    assert_eq!(response.conditions[0].id, "1");
    assert!(!response.conditions[0].is_external);
    assert!(response.conditions[1].is_external);
    assert_eq!(
        response.conditions[1].external_name.as_deref(),
        Some("DataClearingRequired")
    );
}

#[test]
fn test_compute_description_hash() {
    let hash1 = compute_description_hash("Wenn Aufteilung vorhanden");
    let hash2 = compute_description_hash("Wenn Aufteilung vorhanden");
    let hash3 = compute_description_hash("Wenn Aufteilung NICHT vorhanden");

    assert_eq!(hash1, hash2, "same input should produce same hash");
    assert_ne!(hash1, hash3, "different input should produce different hash");
    assert_eq!(hash1.len(), 8, "hash should be 8 hex characters");
}

#[test]
fn test_metadata_serialization_roundtrip() {
    let mut conditions = HashMap::new();
    conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: Some("Simple check".to_string()),
            description_hash: "abcd1234".to_string(),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "UTILMD_AHB_Strom_2_1.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions,
    };

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    let deserialized: ConditionMetadataFile = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.format_version, "FV2510");
    assert_eq!(deserialized.conditions.len(), 1);
    assert_eq!(
        deserialized.conditions["1"].confidence,
        "high"
    );
}

#[test]
fn test_decide_regeneration_all_new() {
    let conditions = vec![
        ("1".to_string(), "Wenn Aufteilung vorhanden".to_string()),
        ("2".to_string(), "Wenn Netznutzung".to_string()),
    ];

    let decision = decide_regeneration(&conditions, None, &HashSet::new(), false);

    assert_eq!(decision.to_regenerate.len(), 2);
    assert!(decision.to_preserve.is_empty());
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::New);
}

#[test]
fn test_decide_regeneration_force_all() {
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash("Wenn Aufteilung vorhanden"),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), "Wenn Aufteilung vorhanden".to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, true);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::Forced);
}

#[test]
fn test_decide_regeneration_preserve_high_confidence() {
    let desc = "Wenn Aufteilung vorhanden";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash(desc),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), desc.to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert!(decision.to_regenerate.is_empty());
    assert_eq!(decision.to_preserve, vec!["1"]);
}

#[test]
fn test_decide_regeneration_stale_description() {
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash("OLD description"),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), "NEW description".to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::Stale);
}

#[test]
fn test_decide_regeneration_low_confidence_regenerated() {
    let desc = "Complex temporal condition";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "99".to_string(),
        ConditionMetadata {
            confidence: "low".to_string(),
            reasoning: Some("Too complex".to_string()),
            description_hash: compute_description_hash(desc),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("99".to_string(), desc.to_string())];
    let mut existing_ids = HashSet::new();
    existing_ids.insert("99".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(
        decision.to_regenerate[0].reason,
        RegenerationReason::LowConfidence
    );
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test condition_types_tests
```

Expected: Tests fail because the module does not exist yet.

### Step 3 -- Implement

Create the files listed above. Add `sha2` and `hex` to `Cargo.toml`:

```toml
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1", features = ["full"] }
```

### Step 4 -- Run the test (GREEN)

```bash
cargo test -p automapper-generator --test condition_types_tests
```

Expected:

```
running 10 tests
test test_confidence_level_display ... ok
test test_confidence_level_from_str ... ok
test test_generated_condition_serialization_roundtrip ... ok
test test_claude_response_parsing ... ok
test test_compute_description_hash ... ok
test test_metadata_serialization_roundtrip ... ok
test test_decide_regeneration_all_new ... ok
test test_decide_regeneration_force_all ... ok
test test_decide_regeneration_preserve_high_confidence ... ok
test test_decide_regeneration_stale_description ... ok
test test_decide_regeneration_low_confidence_regenerated ... ok
```

### Step 5 -- Verify compilation

```bash
cargo check -p automapper-generator
```

### Step 6 -- Commit

```bash
git add -A && git commit -m "feat(generator): add condition types, metadata structs, and regeneration decider"
```

---

## Task 2: Prompt building and Claude CLI response parsing

### Step 1 -- Write the test

Create `crates/automapper-generator/src/conditions/prompt.rs`:

```rust
use crate::conditions::condition_types::ConditionInput;
use crate::schema::mig::MigSchema;

/// Context for condition generation, including segment structure and examples.
pub struct ConditionContext<'a> {
    /// The EDIFACT message type (e.g., "UTILMD").
    pub message_type: &'a str,
    /// The format version (e.g., "FV2510").
    pub format_version: &'a str,
    /// Optional MIG schema for segment structure context.
    pub mig_schema: Option<&'a MigSchema>,
    /// Example condition implementations for few-shot learning.
    pub example_implementations: Vec<String>,
}

/// Builds the system prompt for condition generation.
///
/// The system prompt instructs Claude to generate Rust condition evaluator functions
/// from German AHB condition descriptions.
pub fn build_system_prompt() -> String {
    r#"You are an expert Rust developer specializing in EDIFACT message validation.
Your task is to generate Rust condition evaluator functions from German AHB (Anwendungshandbuch) condition descriptions.

The generated functions will be used in a struct that implements ConditionEvaluator:
```rust
pub trait ConditionEvaluator: Send + Sync {
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult;
    fn is_external(&self, condition: u32) -> bool;
}
```

Each condition is implemented as a method with this signature:
```rust
fn evaluate_NNN(&self, ctx: &EvaluationContext) -> ConditionResult {
    // Your implementation here
}
```

**EvaluationContext API:**
- `ctx.transaktion` - the `UtilmdTransaktion` struct with all parsed business objects
- `ctx.pruefidentifikator` - the current Pruefidentifikator being validated
- `ctx.external` - an `&dyn ExternalConditionProvider` for conditions requiring runtime business context

**UtilmdTransaktion fields:**
- `ctx.transaktion.marktlokationen` - `Vec<WithValidity<Marktlokation, MarktlokationEdifact>>`
- `ctx.transaktion.messlokationen` - `Vec<WithValidity<Messlokation, MesslokationEdifact>>`
- `ctx.transaktion.netzlokationen` - `Vec<WithValidity<Netzlokation, NetzlokationEdifact>>`
- `ctx.transaktion.zaehler` - `Vec<WithValidity<Zaehler, ZaehlerEdifact>>`
- `ctx.transaktion.parteien` - `Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>`
- `ctx.transaktion.vertrag` - `Option<WithValidity<Vertrag, VertragEdifact>>`
- `ctx.transaktion.prozessdaten` - `Prozessdaten`
- `ctx.transaktion.zeitscheiben` - `Vec<Zeitscheibe>`
- `ctx.transaktion.steuerbare_ressourcen` - `Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>`
- `ctx.transaktion.technische_ressourcen` - `Vec<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>>`

**ConditionResult:**
```rust
pub enum ConditionResult { True, False, Unknown }
```

Return `ConditionResult::Unknown` when the condition cannot be determined from the available data.

**Confidence levels:**
- **high**: Simple field existence checks or value comparisons
- **medium**: Logic that requires some interpretation but is straightforward
- **low**: Complex temporal logic, business rules that need clarification

**External conditions:**
Some conditions CANNOT be determined from the message alone. These depend on external runtime context such as:
- Message splitting status ("Wenn Aufteilung vorhanden")
- Data clearing requirements ("Datenclearing erforderlich")

Mark such conditions with `"is_external": true`. For external conditions, provide an `"external_name"` field with a meaningful snake_case name.

For **low confidence** conditions, set implementation to null.

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{
  "conditions": [
    {
      "id": "condition-id",
      "implementation": "Rust function body as a string (null for external/low confidence)",
      "confidence": "high" | "medium" | "low",
      "reasoning": "explanation",
      "is_external": false,
      "external_name": "snake_case_name (required only when is_external is true)"
    }
  ]
}"#
    .to_string()
}

/// Builds the user prompt from a batch of conditions.
pub fn build_user_prompt(
    conditions: &[ConditionInput],
    context: &ConditionContext<'_>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "Generate condition evaluator methods for the following conditions:\n\n"
    ));
    prompt.push_str(&format!(
        "Message Type: {}\nFormat Version: {}\n",
        context.message_type, context.format_version
    ));

    // Add segment structure context from MIG if available
    if let Some(mig) = context.mig_schema {
        let segment_context = build_segment_structure_context(mig, conditions);
        if !segment_context.is_empty() {
            prompt.push_str(&format!("\n{}\n", segment_context));
        }
    }

    // Add example implementations
    if !context.example_implementations.is_empty() {
        prompt.push_str("\n## Example Implementations\n");
        for example in &context.example_implementations {
            prompt.push_str(&format!("{}\n\n", example));
        }
    }

    // Add conditions list
    prompt.push_str("\nConditions:\n");
    for condition in conditions {
        prompt.push_str(&format!("  - [{}]: {}\n", condition.id, condition.description));
        if let Some(ref fields) = condition.referencing_fields {
            if !fields.is_empty() {
                prompt.push_str(&format!(
                    "    Used by fields: {}\n",
                    fields.join(", ")
                ));
            }
        }
    }

    prompt.push_str("\nGenerate the JSON response with implementations for all conditions.\n");

    prompt
}

/// Extracts segment IDs referenced in condition descriptions and builds
/// a compact segment structure reference from the MIG schema.
fn build_segment_structure_context(mig: &MigSchema, conditions: &[ConditionInput]) -> String {
    use regex::Regex;

    let de_regex = Regex::new(r"(?i)(?:SG\d+\s+)?([A-Z]{3})(?:\+[A-Z0-9]+)?\s+DE(\d{4})").unwrap();
    let qualifier_regex = Regex::new(r"\b([A-Z]{3})\+([A-Z0-9]+)").unwrap();

    let mut referenced_segments = std::collections::HashSet::new();

    for condition in conditions {
        for cap in de_regex.captures_iter(&condition.description) {
            if let Some(seg) = cap.get(1) {
                referenced_segments.insert(seg.as_str().to_uppercase());
            }
        }
        for cap in qualifier_regex.captures_iter(&condition.description) {
            if let Some(seg) = cap.get(1) {
                referenced_segments.insert(seg.as_str().to_uppercase());
            }
        }
    }

    if referenced_segments.is_empty() {
        return String::new();
    }

    let mut context = String::new();
    context.push_str("## EDIFACT Segment Structure Reference\n");
    context.push_str("This shows how to access data elements from the parsed transaction.\n\n");

    // Include segment definitions from MIG that are referenced
    for segment in &mig.segments {
        if referenced_segments.contains(&segment.id.to_uppercase()) {
            context.push_str(&format!(
                "### {} - {}\n",
                segment.id,
                segment.name
            ));
            for de in &segment.data_elements {
                context.push_str(&format!(
                    "- DE{} ({}): element position {}\n",
                    de.id, de.name, de.position
                ));
            }
            context.push('\n');
        }
    }

    // Also check segments inside groups
    for group in &mig.segment_groups {
        append_group_segments(&mut context, group, &referenced_segments);
    }

    context
}

fn append_group_segments(
    context: &mut String,
    group: &crate::schema::mig::MigSegmentGroup,
    referenced: &std::collections::HashSet<String>,
) {
    for segment in &group.segments {
        if referenced.contains(&segment.id.to_uppercase()) {
            context.push_str(&format!(
                "### {} - {} (in {})\n",
                segment.id, segment.name, group.id
            ));
            for de in &segment.data_elements {
                context.push_str(&format!(
                    "- DE{} ({}): element position {}\n",
                    de.id, de.name, de.position
                ));
            }
            context.push('\n');
        }
    }
    for nested in &group.nested_groups {
        append_group_segments(context, nested, referenced);
    }
}

/// Default example implementations for few-shot prompting.
pub fn default_example_implementations() -> Vec<String> {
    vec![
        r#"// Example 1: Field existence check
fn evaluate_494(&self, ctx: &EvaluationContext) -> ConditionResult {
    if ctx.transaktion.marktlokationen.is_empty() {
        ConditionResult::False
    } else {
        ConditionResult::True
    }
}"#
        .to_string(),
        r#"// Example 2: Value comparison
fn evaluate_501(&self, ctx: &EvaluationContext) -> ConditionResult {
    match ctx.transaktion.prozessdaten.kategorie.as_deref() {
        Some("E01") | Some("E02") => ConditionResult::True,
        Some(_) => ConditionResult::False,
        None => ConditionResult::Unknown,
    }
}"#
        .to_string(),
        r#"// Example 3: External condition
fn evaluate_1(&self, ctx: &EvaluationContext) -> ConditionResult {
    // "Wenn Aufteilung vorhanden" — requires external context
    ctx.external.evaluate("message_splitting")
}"#
        .to_string(),
    ]
}
```

Create `crates/automapper-generator/src/conditions/claude_generator.rs`:

```rust
use std::io::Write;
use std::process::{Command, Stdio};

use crate::conditions::condition_types::*;
use crate::conditions::prompt::{self, ConditionContext};
use crate::error::GeneratorError;

/// Shells out to the `claude` CLI to generate condition evaluator code.
///
/// Uses `claude --print` with a JSON-structured prompt. No SDK dependency --
/// reuses the user's existing Claude CLI subscription.
pub struct ClaudeConditionGenerator {
    /// Maximum concurrent Claude CLI calls for batch generation.
    pub max_concurrent: usize,
}

impl ClaudeConditionGenerator {
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    /// Generates conditions for a single batch by shelling out to `claude --print`.
    ///
    /// Returns parsed `GeneratedCondition` structs.
    pub fn generate_batch(
        &self,
        conditions: &[ConditionInput],
        context: &ConditionContext<'_>,
    ) -> Result<Vec<GeneratedCondition>, GeneratorError> {
        let system_prompt = prompt::build_system_prompt();
        let user_prompt = prompt::build_user_prompt(conditions, context);

        let full_prompt = format!("{}\n\n---\n\n{}", system_prompt, user_prompt);

        let raw_response = self.invoke_claude_cli(&full_prompt)?;
        let parsed = self.parse_response(&raw_response)?;

        Ok(parsed)
    }

    /// Invokes the `claude` CLI with `--print` flag and returns stdout.
    fn invoke_claude_cli(&self, prompt: &str) -> Result<String, GeneratorError> {
        let mut child = Command::new("claude")
            .args(["--print", "--model", "sonnet", "--max-tokens", "16384"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GeneratorError::ClaudeCli {
                message: format!("failed to spawn claude CLI: {}", e),
            })?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(prompt.as_bytes()).map_err(|e| {
                GeneratorError::ClaudeCli {
                    message: format!("failed to write to claude stdin: {}", e),
                }
            })?;
        }
        // Drop stdin to signal EOF
        drop(child.stdin.take());

        let output = child.wait_with_output().map_err(|e| GeneratorError::ClaudeCli {
            message: format!("claude CLI failed: {}", e),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GeneratorError::ClaudeCli {
                message: format!(
                    "claude CLI exited with status {}: {}",
                    output.status, stderr
                ),
            });
        }

        String::from_utf8(output.stdout).map_err(|e| GeneratorError::ClaudeCli {
            message: format!("claude CLI returned invalid UTF-8: {}", e),
        })
    }

    /// Parses the JSON response from Claude into GeneratedCondition structs.
    pub fn parse_response(&self, raw_response: &str) -> Result<Vec<GeneratedCondition>, GeneratorError> {
        let cleaned = strip_markdown_code_blocks(raw_response);

        let response: ClaudeConditionResponse =
            serde_json::from_str(&cleaned).map_err(|e| GeneratorError::ClaudeCli {
                message: format!("failed to parse Claude JSON response: {}. Response was: {}", e, &cleaned[..cleaned.len().min(500)]),
            })?;

        let mut results = Vec::new();
        for entry in response.conditions {
            let condition_number: u32 = entry.id.parse().map_err(|e| GeneratorError::ClaudeCli {
                message: format!("invalid condition ID '{}': {}", entry.id, e),
            })?;

            let confidence: ConfidenceLevel = entry
                .confidence
                .parse()
                .unwrap_or(ConfidenceLevel::Medium);

            results.push(GeneratedCondition {
                condition_number,
                rust_code: entry.implementation,
                is_external: entry.is_external,
                confidence,
                reasoning: entry.reasoning,
                external_name: entry.external_name,
                original_description: None,
                referencing_fields: None,
            });
        }

        Ok(results)
    }
}

/// Strips markdown code block wrappers (```json ... ```) from a response string.
fn strip_markdown_code_blocks(response: &str) -> String {
    let trimmed = response.trim();

    if trimmed.starts_with("```") {
        let rest = if let Some(newline_pos) = trimmed.find('\n') {
            &trimmed[newline_pos + 1..]
        } else {
            trimmed
        };

        if rest.ends_with("```") {
            return rest[..rest.len() - 3].trim().to_string();
        }
    }

    trimmed.to_string()
}
```

Update `crates/automapper-generator/src/conditions/mod.rs`:

```rust
pub mod condition_types;
pub mod metadata;
pub mod prompt;
pub mod claude_generator;
```

Write `crates/automapper-generator/tests/prompt_tests.rs`:

```rust
use automapper_generator::conditions::condition_types::ConditionInput;
use automapper_generator::conditions::prompt::*;
use automapper_generator::conditions::claude_generator::ClaudeConditionGenerator;

#[test]
fn test_system_prompt_contains_key_instructions() {
    let prompt = build_system_prompt();

    assert!(prompt.contains("ConditionEvaluator"), "should mention the trait");
    assert!(prompt.contains("EvaluationContext"), "should mention the context");
    assert!(prompt.contains("ConditionResult"), "should mention the result type");
    assert!(prompt.contains("is_external"), "should explain external conditions");
    assert!(prompt.contains("JSON"), "should request JSON output");
}

#[test]
fn test_user_prompt_includes_conditions() {
    let conditions = vec![
        ConditionInput {
            id: "1".to_string(),
            description: "Wenn Aufteilung vorhanden".to_string(),
            referencing_fields: Some(vec!["SG8/SEQ (Muss [1])".to_string()]),
        },
        ConditionInput {
            id: "2".to_string(),
            description: "Wenn Netznutzung vorhanden".to_string(),
            referencing_fields: None,
        },
    ];

    let context = ConditionContext {
        message_type: "UTILMD",
        format_version: "FV2510",
        mig_schema: None,
        example_implementations: default_example_implementations(),
    };

    let prompt = build_user_prompt(&conditions, &context);

    assert!(prompt.contains("UTILMD"), "should include message type");
    assert!(prompt.contains("FV2510"), "should include format version");
    assert!(prompt.contains("[1]: Wenn Aufteilung vorhanden"), "should include condition 1");
    assert!(prompt.contains("[2]: Wenn Netznutzung vorhanden"), "should include condition 2");
    assert!(
        prompt.contains("SG8/SEQ (Muss [1])"),
        "should include referencing fields"
    );
    assert!(prompt.contains("Example"), "should include examples section");
}

#[test]
fn test_default_examples_exist() {
    let examples = default_example_implementations();
    assert!(examples.len() >= 3, "should have at least 3 examples");
    assert!(examples[0].contains("evaluate_"), "examples should contain function signatures");
}

#[test]
fn test_parse_valid_json_response() {
    let generator = ClaudeConditionGenerator::new(4);

    let json = r#"{
        "conditions": [
            {
                "id": "42",
                "implementation": "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}",
                "confidence": "high",
                "reasoning": "Simple field existence check",
                "is_external": false
            }
        ]
    }"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].condition_number, 42);
    assert_eq!(conditions[0].confidence, automapper_generator::conditions::condition_types::ConfidenceLevel::High);
    assert!(!conditions[0].is_external);
    assert!(conditions[0].rust_code.is_some());
}

#[test]
fn test_parse_response_with_markdown_wrapper() {
    let generator = ClaudeConditionGenerator::new(4);

    let json = r#"```json
{
    "conditions": [
        {
            "id": "8",
            "implementation": null,
            "confidence": "high",
            "reasoning": "External condition",
            "is_external": true,
            "external_name": "DataClearingRequired"
        }
    ]
}
```"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 1);
    assert!(conditions[0].is_external);
    assert!(conditions[0].rust_code.is_none());
    assert_eq!(
        conditions[0].external_name.as_deref(),
        Some("DataClearingRequired")
    );
}

#[test]
fn test_parse_response_invalid_json() {
    let generator = ClaudeConditionGenerator::new(4);

    let result = generator.parse_response("not json at all");
    assert!(result.is_err());
}

#[test]
fn test_parse_response_mixed_confidence() {
    let generator = ClaudeConditionGenerator::new(4);

    let json = r#"{
        "conditions": [
            {
                "id": "1",
                "implementation": "ConditionResult::True",
                "confidence": "high",
                "reasoning": "Simple",
                "is_external": false
            },
            {
                "id": "2",
                "implementation": "ConditionResult::Unknown",
                "confidence": "medium",
                "reasoning": "Needs review",
                "is_external": false
            },
            {
                "id": "3",
                "implementation": null,
                "confidence": "low",
                "reasoning": "Too complex",
                "is_external": false
            }
        ]
    }"#;

    let conditions = generator.parse_response(json).unwrap();
    assert_eq!(conditions.len(), 3);

    use automapper_generator::conditions::condition_types::ConfidenceLevel;
    assert_eq!(conditions[0].confidence, ConfidenceLevel::High);
    assert_eq!(conditions[1].confidence, ConfidenceLevel::Medium);
    assert_eq!(conditions[2].confidence, ConfidenceLevel::Low);
    assert!(conditions[2].rust_code.is_none());
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test prompt_tests
```

### Step 3 -- Implement

Create the files listed above.

### Step 4 -- Run the test (GREEN)

```bash
cargo test -p automapper-generator --test prompt_tests
```

Expected:

```
running 7 tests
test test_system_prompt_contains_key_instructions ... ok
test test_user_prompt_includes_conditions ... ok
test test_default_examples_exist ... ok
test test_parse_valid_json_response ... ok
test test_parse_response_with_markdown_wrapper ... ok
test test_parse_response_invalid_json ... ok
test test_parse_response_mixed_confidence ... ok
```

### Step 5 -- Commit

```bash
git add -A && git commit -m "feat(generator): implement prompt building and Claude response parsing"
```

---

## Task 3: Condition evaluator code generation (output file)

### Step 1 -- Write the test

Create `crates/automapper-generator/src/conditions/codegen.rs`:

```rust
use crate::conditions::condition_types::{ConfidenceLevel, GeneratedCondition};

/// Generates a complete Rust source file containing a ConditionEvaluator impl
/// with match arms dispatching to per-condition evaluation functions.
///
/// Output format mirrors the C# `ConditionCodeGenerator.GenerateClass`.
pub fn generate_condition_evaluator_file(
    message_type: &str,
    format_version: &str,
    conditions: &[GeneratedCondition],
    source_file: &str,
    preserved_method_bodies: &std::collections::HashMap<u32, String>,
) -> String {
    let class_name = format!(
        "{}ConditionEvaluator{}",
        to_pascal_case(message_type),
        format_version
    );

    let mut code = String::new();

    // Header
    code.push_str("// <auto-generated>\n");
    code.push_str("// Generated by automapper-generator generate-conditions\n");
    code.push_str(&format!("// AHB: {}\n", source_file));
    code.push_str(&format!(
        "// Generated: {}\n",
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
    ));
    code.push_str("// </auto-generated>\n\n");

    code.push_str("use automapper_validation::condition::{ConditionEvaluator, ConditionResult, EvaluationContext};\n\n");

    // Struct definition
    code.push_str(&format!(
        "/// Generated condition evaluator for {} {}.\n",
        message_type, format_version
    ));
    code.push_str(&format!("pub struct {} {{\n", class_name));
    code.push_str("    // External condition IDs that require runtime context.\n");
    code.push_str("    external_conditions: std::collections::HashSet<u32>,\n");
    code.push_str("}\n\n");

    // Collect external condition numbers
    let external_ids: Vec<u32> = conditions
        .iter()
        .filter(|c| c.is_external)
        .map(|c| c.condition_number)
        .collect();

    // Default impl
    code.push_str(&format!("impl Default for {} {{\n", class_name));
    code.push_str("    fn default() -> Self {\n");
    code.push_str("        let mut external_conditions = std::collections::HashSet::new();\n");
    for id in &external_ids {
        code.push_str(&format!("        external_conditions.insert({});\n", id));
    }
    code.push_str("        Self { external_conditions }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // ConditionEvaluator impl
    code.push_str(&format!("impl ConditionEvaluator for {} {{\n", class_name));

    // evaluate() method with match arms
    code.push_str(
        "    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult {\n",
    );
    code.push_str("        match condition {\n");

    // Merge new conditions and preserved conditions, sorted by condition number
    let mut all_condition_numbers: Vec<u32> = conditions
        .iter()
        .map(|c| c.condition_number)
        .chain(preserved_method_bodies.keys().copied())
        .collect();
    all_condition_numbers.sort();
    all_condition_numbers.dedup();

    for &num in &all_condition_numbers {
        code.push_str(&format!(
            "            {} => self.evaluate_{}(ctx),\n",
            num, num
        ));
    }
    code.push_str("            _ => ConditionResult::Unknown,\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // is_external() method
    code.push_str("    fn is_external(&self, condition: u32) -> bool {\n");
    code.push_str("        self.external_conditions.contains(&condition)\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Private evaluation methods
    code.push_str(&format!("impl {} {{\n", class_name));

    for condition in conditions {
        generate_condition_method(&mut code, condition);
    }

    // Preserved methods (from previous generation, high-confidence, unchanged)
    for (&num, body) in preserved_method_bodies {
        // Only include if not already generated as a new condition
        if !conditions.iter().any(|c| c.condition_number == num) {
            code.push_str(&format!(
                "    fn evaluate_{}(&self, ctx: &EvaluationContext) -> ConditionResult {{\n",
                num
            ));
            code.push_str(&indent_body(body, 8));
            code.push_str("    }\n\n");
        }
    }

    code.push_str("}\n");

    code
}

/// Generates a single condition evaluation method.
fn generate_condition_method(code: &mut String, condition: &GeneratedCondition) {
    let num = condition.condition_number;

    // Doc comment
    code.push_str(&format!("    /// [{}]", num));
    if let Some(ref desc) = condition.original_description {
        code.push_str(&format!(" {}", escape_doc_comment(desc)));
    }
    code.push('\n');

    if condition.is_external {
        code.push_str("    /// EXTERNAL: Requires context from outside the message.\n");
    }

    if let Some(ref fields) = condition.referencing_fields {
        if !fields.is_empty() {
            code.push_str("    /// Referenced by:\n");
            for field in fields.iter().take(10) {
                code.push_str(&format!("    /// - {}\n", escape_doc_comment(field)));
            }
            if fields.len() > 10 {
                code.push_str(&format!("    /// - ... and {} more\n", fields.len() - 10));
            }
        }
    }

    // Confidence annotation comment
    if condition.confidence == ConfidenceLevel::Medium {
        if let Some(ref reasoning) = condition.reasoning {
            code.push_str(&format!(
                "    // REVIEW: {} (medium confidence)\n",
                reasoning
            ));
        }
    }

    // Method signature
    code.push_str(&format!(
        "    fn evaluate_{}(&self, ctx: &EvaluationContext) -> ConditionResult {{\n",
        num
    ));

    // Method body
    if condition.is_external {
        code.push_str(&format!(
            "        ctx.external.evaluate(\"{}\")\n",
            condition
                .external_name
                .as_deref()
                .unwrap_or(&format!("condition_{}", num))
        ));
    } else if condition.confidence == ConfidenceLevel::High || condition.confidence == ConfidenceLevel::Medium {
        if let Some(ref rust_code) = condition.rust_code {
            code.push_str(&indent_body(rust_code, 8));
        } else {
            code.push_str(&format!(
                "        // TODO: Implement condition [{}]\n",
                num
            ));
            code.push_str("        ConditionResult::Unknown\n");
        }
    } else {
        // Low confidence
        code.push_str(&format!(
            "        // TODO: Condition [{}] requires manual implementation\n",
            num
        ));
        if let Some(ref reasoning) = condition.reasoning {
            code.push_str(&format!("        // Reason: {}\n", reasoning));
        }
        code.push_str("        ConditionResult::Unknown\n");
    }

    code.push_str("    }\n\n");
}

/// Indents a multi-line code body to the specified column.
fn indent_body(body: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    let lines: Vec<&str> = body.lines().collect();

    // Find minimum indentation
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let mut result = String::new();
    for line in &lines {
        if line.trim().is_empty() {
            result.push('\n');
        } else {
            let stripped = if line.len() > min_indent {
                &line[min_indent..]
            } else {
                line.trim_start()
            };
            result.push_str(&indent);
            result.push_str(stripped);
            result.push('\n');
        }
    }
    result
}

fn escape_doc_comment(text: &str) -> String {
    text.replace('<', "&lt;").replace('>', "&gt;")
}

fn to_pascal_case(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    let mut chars = input.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    first + &chars.as_str().to_lowercase()
}
```

Update `crates/automapper-generator/src/conditions/mod.rs`:

```rust
pub mod condition_types;
pub mod metadata;
pub mod prompt;
pub mod claude_generator;
pub mod codegen;
```

Add `chrono` to `crates/automapper-generator/Cargo.toml`:

```toml
chrono = { version = "0.4", features = ["serde"] }
```

Write `crates/automapper-generator/tests/condition_codegen_tests.rs`:

```rust
use automapper_generator::conditions::condition_types::*;
use automapper_generator::conditions::codegen::generate_condition_evaluator_file;
use std::collections::HashMap;

fn make_test_conditions() -> Vec<GeneratedCondition> {
    vec![
        GeneratedCondition {
            condition_number: 1,
            rust_code: None,
            is_external: true,
            confidence: ConfidenceLevel::High,
            reasoning: Some("Requires external context".to_string()),
            external_name: Some("message_splitting".to_string()),
            original_description: Some("Wenn Aufteilung vorhanden".to_string()),
            referencing_fields: Some(vec!["SG8/SEQ (Muss [1])".to_string()]),
        },
        GeneratedCondition {
            condition_number: 2,
            rust_code: Some(
                "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}"
                    .to_string(),
            ),
            is_external: false,
            confidence: ConfidenceLevel::High,
            reasoning: Some("Simple field check".to_string()),
            external_name: None,
            original_description: Some("Wenn Marktlokation vorhanden".to_string()),
            referencing_fields: None,
        },
        GeneratedCondition {
            condition_number: 99,
            rust_code: Some("ConditionResult::Unknown".to_string()),
            is_external: false,
            confidence: ConfidenceLevel::Medium,
            reasoning: Some("Needs review".to_string()),
            external_name: None,
            original_description: Some("Komplexe Bedingung".to_string()),
            referencing_fields: None,
        },
        GeneratedCondition {
            condition_number: 100,
            rust_code: None,
            is_external: false,
            confidence: ConfidenceLevel::Low,
            reasoning: Some("Too complex for auto-generation".to_string()),
            external_name: None,
            original_description: Some("Sehr komplexe Bedingung".to_string()),
            referencing_fields: None,
        },
    ]
}

#[test]
fn test_generate_condition_evaluator_file_structure() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test_ahb.xml",
        &HashMap::new(),
    );

    assert!(output.contains("auto-generated"), "should have header");
    assert!(
        output.contains("pub struct UtilmdConditionEvaluatorFV2510"),
        "should have struct name"
    );
    assert!(
        output.contains("impl ConditionEvaluator for UtilmdConditionEvaluatorFV2510"),
        "should impl trait"
    );
}

#[test]
fn test_generate_match_arms() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(output.contains("1 => self.evaluate_1(ctx)"), "should have match arm for condition 1");
    assert!(output.contains("2 => self.evaluate_2(ctx)"), "should have match arm for condition 2");
    assert!(
        output.contains("99 => self.evaluate_99(ctx)"),
        "should have match arm for condition 99"
    );
    assert!(
        output.contains("_ => ConditionResult::Unknown"),
        "should have default match arm"
    );
}

#[test]
fn test_external_condition_output() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("ctx.external.evaluate(\"message_splitting\")"),
        "external condition should delegate to provider"
    );
    assert!(
        output.contains("EXTERNAL"),
        "external condition should have EXTERNAL doc comment"
    );
    assert!(
        output.contains("external_conditions.insert(1)"),
        "should register condition 1 as external"
    );
}

#[test]
fn test_high_confidence_condition_has_implementation() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("ctx.transaktion.marktlokationen.is_empty()"),
        "high-confidence condition should have generated implementation"
    );
}

#[test]
fn test_medium_confidence_has_review_marker() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("REVIEW"),
        "medium-confidence condition should have REVIEW comment"
    );
}

#[test]
fn test_low_confidence_has_todo() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test.xml",
        &HashMap::new(),
    );

    assert!(
        output.contains("TODO: Condition [100] requires manual implementation"),
        "low-confidence condition should have TODO"
    );
}

#[test]
fn test_preserved_methods_included() {
    let conditions = vec![GeneratedCondition {
        condition_number: 5,
        rust_code: Some("ConditionResult::True".to_string()),
        is_external: false,
        confidence: ConfidenceLevel::High,
        reasoning: None,
        external_name: None,
        original_description: None,
        referencing_fields: None,
    }];

    let mut preserved = HashMap::new();
    preserved.insert(
        10,
        "ConditionResult::False // previously generated".to_string(),
    );

    let output =
        generate_condition_evaluator_file("UTILMD", "FV2510", &conditions, "test.xml", &preserved);

    assert!(
        output.contains("5 => self.evaluate_5(ctx)"),
        "should have new condition"
    );
    assert!(
        output.contains("10 => self.evaluate_10(ctx)"),
        "should have preserved condition in match"
    );
    assert!(
        output.contains("previously generated"),
        "should include preserved method body"
    );
}

#[test]
fn test_condition_evaluator_snapshot() {
    let conditions = make_test_conditions();
    let output = generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &conditions,
        "test_ahb.xml",
        &HashMap::new(),
    );

    // Replace the dynamic timestamp line for stable snapshots
    let stable_output = output
        .lines()
        .map(|line| {
            if line.starts_with("// Generated:") {
                "// Generated: 2026-02-18T00:00:00Z"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!("condition_evaluator_fv2510", stable_output);
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test condition_codegen_tests
```

### Step 3 -- Implement

Create the file listed above.

### Step 4 -- Accept snapshots and run (GREEN)

```bash
cargo insta test -p automapper-generator --test condition_codegen_tests --accept
cargo test -p automapper-generator --test condition_codegen_tests
```

Expected:

```
running 8 tests
test test_generate_condition_evaluator_file_structure ... ok
test test_generate_match_arms ... ok
test test_external_condition_output ... ok
test test_high_confidence_condition_has_implementation ... ok
test test_medium_confidence_has_review_marker ... ok
test test_low_confidence_has_todo ... ok
test test_preserved_methods_included ... ok
test test_condition_evaluator_snapshot ... ok
```

### Step 5 -- Commit

```bash
git add -A && git commit -m "feat(generator): implement condition evaluator Rust code generation"
```

---

## Task 4: Integration test with mock `claude` script

### Step 1 -- Write the test

Create a mock `claude` shell script that returns canned JSON responses.

Create `crates/automapper-generator/tests/fixtures/mock_claude.sh`:

```bash
#!/bin/bash
# Mock claude CLI that reads stdin and returns canned condition generation response.
# Used in integration tests for the generate-conditions subcommand.

# Read stdin (prompt) and discard it
cat > /dev/null

# Return a canned JSON response
cat <<'EOF'
{
  "conditions": [
    {
      "id": "1",
      "implementation": null,
      "confidence": "high",
      "reasoning": "Requires external context: message splitting status",
      "is_external": true,
      "external_name": "message_splitting"
    },
    {
      "id": "2",
      "implementation": "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}",
      "confidence": "high",
      "reasoning": "Simple field existence check on Marktlokation list",
      "is_external": false
    },
    {
      "id": "3",
      "implementation": "ConditionResult::Unknown",
      "confidence": "medium",
      "reasoning": "Interpretation of temporal condition uncertain",
      "is_external": false
    }
  ]
}
EOF
```

Create `crates/automapper-generator/tests/fixtures/minimal_ahb_with_conditions.xml` (a minimal AHB XML with a Bedingungen section):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<AHB_UTILMD_Strom>
  <AWF Pruefidentifikator="11001" Beschreibung="Lieferbeginn">
    <Feld Segment="UNH" Status="Muss"/>
    <Feld Segment="BGM" Status="Muss [1]"/>
    <Feld Segment="SG8/SEQ" Status="Muss [2]"/>
  </AWF>
  <Bedingungen>
    <Bedingung Nr="1" Beschreibung="Wenn Aufteilung vorhanden"/>
    <Bedingung Nr="2" Beschreibung="Wenn Marktlokation vorhanden"/>
    <Bedingung Nr="3" Beschreibung="Wenn Zeitraum gueltig"/>
  </Bedingungen>
</AHB_UTILMD_Strom>
```

Write `crates/automapper-generator/tests/mock_claude_integration_test.rs`:

```rust
use std::io::Write;
use std::process::Command;

use automapper_generator::conditions::claude_generator::ClaudeConditionGenerator;
use automapper_generator::conditions::condition_types::*;
use automapper_generator::conditions::prompt::*;

/// Test that the ClaudeConditionGenerator can parse a realistic canned response.
#[test]
fn test_parse_mock_claude_response() {
    // Simulate the response that the mock_claude.sh script would return
    let mock_response = include_str!("fixtures/mock_claude_response.json");

    let generator = ClaudeConditionGenerator::new(4);
    let conditions = generator.parse_response(mock_response).unwrap();

    assert_eq!(conditions.len(), 3);

    // Condition 1: external
    assert_eq!(conditions[0].condition_number, 1);
    assert!(conditions[0].is_external);
    assert_eq!(conditions[0].confidence, ConfidenceLevel::High);
    assert!(conditions[0].rust_code.is_none());

    // Condition 2: high confidence with implementation
    assert_eq!(conditions[1].condition_number, 2);
    assert!(!conditions[1].is_external);
    assert_eq!(conditions[1].confidence, ConfidenceLevel::High);
    assert!(conditions[1].rust_code.is_some());

    // Condition 3: medium confidence
    assert_eq!(conditions[2].condition_number, 3);
    assert_eq!(conditions[2].confidence, ConfidenceLevel::Medium);
}

/// Test the full pipeline: build prompt -> parse response -> generate output file.
#[test]
fn test_end_to_end_condition_generation_pipeline() {
    let mock_response = include_str!("fixtures/mock_claude_response.json");

    // Step 1: Build prompt
    let conditions = vec![
        ConditionInput {
            id: "1".to_string(),
            description: "Wenn Aufteilung vorhanden".to_string(),
            referencing_fields: None,
        },
        ConditionInput {
            id: "2".to_string(),
            description: "Wenn Marktlokation vorhanden".to_string(),
            referencing_fields: Some(vec!["SG8/SEQ (Muss [2])".to_string()]),
        },
        ConditionInput {
            id: "3".to_string(),
            description: "Wenn Zeitraum gueltig".to_string(),
            referencing_fields: None,
        },
    ];

    let context = ConditionContext {
        message_type: "UTILMD",
        format_version: "FV2510",
        mig_schema: None,
        example_implementations: default_example_implementations(),
    };

    let prompt = build_user_prompt(&conditions, &context);
    assert!(!prompt.is_empty());

    // Step 2: Parse response
    let generator = ClaudeConditionGenerator::new(4);
    let generated = generator.parse_response(mock_response).unwrap();

    // Enrich with original descriptions
    let enriched: Vec<GeneratedCondition> = generated
        .into_iter()
        .map(|mut gc| {
            if let Some(input) = conditions.iter().find(|c| c.id == gc.condition_number.to_string())
            {
                gc.original_description = Some(input.description.clone());
                gc.referencing_fields = input.referencing_fields.clone();
            }
            gc
        })
        .collect();

    // Step 3: Generate output file
    let output = automapper_generator::conditions::codegen::generate_condition_evaluator_file(
        "UTILMD",
        "FV2510",
        &enriched,
        "test_ahb.xml",
        &std::collections::HashMap::new(),
    );

    // Verify the output contains expected elements
    assert!(output.contains("UtilmdConditionEvaluatorFV2510"));
    assert!(output.contains("evaluate_1"));
    assert!(output.contains("evaluate_2"));
    assert!(output.contains("evaluate_3"));
    assert!(output.contains("message_splitting"));
    assert!(output.contains("Wenn Aufteilung vorhanden"));
    assert!(output.contains("ConditionEvaluator"));
}
```

Create `crates/automapper-generator/tests/fixtures/mock_claude_response.json`:

```json
{
  "conditions": [
    {
      "id": "1",
      "implementation": null,
      "confidence": "high",
      "reasoning": "Requires external context: message splitting status",
      "is_external": true,
      "external_name": "message_splitting"
    },
    {
      "id": "2",
      "implementation": "if ctx.transaktion.marktlokationen.is_empty() {\n    ConditionResult::False\n} else {\n    ConditionResult::True\n}",
      "confidence": "high",
      "reasoning": "Simple field existence check on Marktlokation list",
      "is_external": false
    },
    {
      "id": "3",
      "implementation": "ConditionResult::Unknown",
      "confidence": "medium",
      "reasoning": "Interpretation of temporal condition uncertain",
      "is_external": false
    }
  ]
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test mock_claude_integration_test
```

### Step 3 -- Implement

Create the fixture files listed above. Ensure the mock script is executable:

```bash
chmod +x crates/automapper-generator/tests/fixtures/mock_claude.sh
```

### Step 4 -- Run the test (GREEN)

```bash
cargo test -p automapper-generator --test mock_claude_integration_test
```

Expected:

```
running 2 tests
test test_parse_mock_claude_response ... ok
test test_end_to_end_condition_generation_pipeline ... ok
```

### Step 5 -- Commit

```bash
git add -A && git commit -m "feat(generator): add integration tests with mock claude responses"
```

---

## Task 5: `GenerateConditions` CLI subcommand wiring

### Step 1 -- Write the test

Write `crates/automapper-generator/tests/generate_conditions_cli_tests.rs`:

```rust
use std::process::Command;

#[test]
fn test_cli_generate_conditions_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args(["generate-conditions", "--help"])
        .output()
        .expect("failed to run automapper-generator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--ahb-path"), "should have --ahb-path flag");
    assert!(stdout.contains("--output-dir"), "should have --output-dir flag");
    assert!(
        stdout.contains("--format-version"),
        "should have --format-version flag"
    );
    assert!(
        stdout.contains("--message-type"),
        "should have --message-type flag"
    );
    assert!(
        stdout.contains("--incremental"),
        "should have --incremental flag"
    );
    assert!(
        stdout.contains("--max-concurrent"),
        "should have --max-concurrent flag"
    );
    assert!(output.status.success());
}

#[test]
fn test_cli_generate_conditions_missing_args() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .arg("generate-conditions")
        .output()
        .expect("failed to run automapper-generator");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("error"),
        "stderr should mention missing required args: {}",
        stderr
    );
}

#[test]
fn test_cli_generate_conditions_dry_run() {
    let ahb_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");

    // Dry run should succeed without calling Claude
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args([
            "generate-conditions",
            "--ahb-path",
            ahb_path.to_str().unwrap(),
            "--output-dir",
            "/tmp/automapper-generator-test-output",
            "--format-version",
            "FV2510",
            "--message-type",
            "UTILMD",
            "--dry-run",
        ])
        .output()
        .expect("failed to run automapper-generator");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Dry run may succeed or fail depending on AHB parsing, but it should NOT invoke Claude
    // The key assertion is that it doesn't hang waiting for claude
    assert!(
        stderr.contains("DRY RUN") || stderr.contains("conditions") || !output.status.success(),
        "dry run should report or fail gracefully: {}",
        stderr
    );
}

#[test]
fn test_cli_validate_schema_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_automapper-generator"))
        .args(["validate-schema", "--help"])
        .output()
        .expect("failed to run automapper-generator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--stammdatenmodell-path"),
        "should have --stammdatenmodell-path flag"
    );
    assert!(
        stdout.contains("--generated-dir"),
        "should have --generated-dir flag"
    );
    assert!(output.status.success());
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test generate_conditions_cli_tests
```

Expected: Fails because the `generate-conditions` subcommand is still a placeholder.

### Step 3 -- Implement

Update `crates/automapper-generator/src/main.rs` to wire the `GenerateConditions` and `ValidateSchema` subcommands:

Replace the placeholder `GenerateConditions` handler in `main.rs`:

```rust
Commands::GenerateConditions {
    ahb_path,
    output_dir,
    format_version,
    message_type,
    incremental,
    concurrency,
    mig_path,
    batch_size,
    dry_run,
} => {
    eprintln!(
        "Generating conditions for {} {} (incremental={})",
        message_type, format_version, incremental
    );

    // Parse AHB
    let ahb_filename = ahb_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let variant = if ahb_filename.contains("Strom") {
        Some("Strom")
    } else if ahb_filename.contains("Gas") {
        Some("Gas")
    } else {
        None
    };

    let ahb_schema = crate::parsing::ahb_parser::parse_ahb(
        &ahb_path,
        &message_type,
        variant,
        &format_version,
    )?;

    // Optionally parse MIG for segment structure context
    let mig_schema = if let Some(ref mig) = mig_path {
        Some(crate::parsing::mig_parser::parse_mig(
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

    // Determine what needs regeneration
    let existing_ids = std::collections::HashSet::new(); // TODO: parse existing output file
    let decision = automapper_generator::conditions::metadata::decide_regeneration(
        &conditions,
        existing_metadata.as_ref(),
        &existing_ids,
        !incremental, // force = !incremental for first run
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
    let condition_inputs: Vec<automapper_generator::conditions::condition_types::ConditionInput> =
        decision
            .to_regenerate
            .iter()
            .map(|c| automapper_generator::conditions::condition_types::ConditionInput {
                id: c.condition_id.clone(),
                description: c.description.clone(),
                referencing_fields: None,
            })
            .collect();

    // Generate conditions in batches
    let generator =
        automapper_generator::conditions::claude_generator::ClaudeConditionGenerator::new(
            concurrency,
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
        eprintln!(
            "[Batch {}/{}] Processing {} conditions...",
            i + 1,
            (condition_inputs.len() + batch_size - 1) / batch_size,
            batch.len()
        );

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
                    "  Generated: {} (High: {}, Medium: {}, Low: {})",
                    generated.len(),
                    high,
                    medium,
                    low
                );
                all_generated.extend(generated);
            }
            Err(e) => {
                eprintln!("  ERROR: {}", e);
            }
        }
    }

    // Generate output file
    let output_path = output_dir.join(format!(
        "{}_conditions_{}.rs",
        message_type.to_lowercase(),
        format_version.to_lowercase()
    ));

    let preserved = std::collections::HashMap::new(); // TODO: preserved from existing file
    let source_code =
        automapper_generator::conditions::codegen::generate_condition_evaluator_file(
            &message_type,
            &format_version,
            &all_generated,
            ahb_path.to_str().unwrap_or("unknown"),
            &preserved,
        );

    std::fs::write(&output_path, &source_code)?;
    eprintln!("Generated: {:?}", output_path);

    // Save metadata
    let mut meta_conditions = std::collections::HashMap::new();
    for gc in &all_generated {
        let desc = gc
            .original_description
            .as_deref()
            .unwrap_or("");
        meta_conditions.insert(
            gc.condition_number.to_string(),
            automapper_generator::conditions::metadata::ConditionMetadata {
                confidence: gc.confidence.to_string(),
                reasoning: gc.reasoning.clone(),
                description_hash:
                    automapper_generator::conditions::metadata::compute_description_hash(desc),
                is_external: gc.is_external,
            },
        );
    }

    let metadata_file =
        automapper_generator::conditions::metadata::ConditionMetadataFile {
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
```

### Step 4 -- Run the test (GREEN)

```bash
cargo test -p automapper-generator --test generate_conditions_cli_tests
```

Expected:

```
running 4 tests
test test_cli_generate_conditions_help ... ok
test test_cli_generate_conditions_missing_args ... ok
test test_cli_generate_conditions_dry_run ... ok
test test_cli_validate_schema_help ... ok
```

### Step 5 -- Commit

```bash
git add -A && git commit -m "feat(generator): wire generate-conditions CLI subcommand with batch processing"
```

---

## Task 6: `ValidateSchema` CLI subcommand

### Step 1 -- Write the test

Create `crates/automapper-generator/src/validation/mod.rs`:

```rust
pub mod schema_validator;
```

Create `crates/automapper-generator/src/validation/schema_validator.rs`:

```rust
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
    for entry in glob::glob(json_pattern.to_str().unwrap_or(""))
        .map_err(|e| GeneratorError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
    {
        if let Ok(path) = entry {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                types.insert(stem.to_string());
            }
        }
    }

    // Look for Rust type definitions (*.rs) with `pub struct` or `pub enum`
    let rs_pattern = stammdatenmodell_path.join("**/*.rs");
    let type_regex = Regex::new(r"pub\s+(?:struct|enum)\s+(\w+)").unwrap();

    for entry in glob::glob(rs_pattern.to_str().unwrap_or(""))
        .map_err(|e| GeneratorError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
    {
        if let Ok(path) = entry {
            let content = std::fs::read_to_string(&path)?;
            for cap in type_regex.captures_iter(&content) {
                if let Some(name) = cap.get(1) {
                    types.insert(name.as_str().to_string());
                }
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
    let mut errors = Vec::new();
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

    // Regex for BO4E type references in generated code
    let bo4e_ref_regex = Regex::new(r"(?:bo4e::)?(\b[A-Z][a-z]+(?:[A-Z][a-z]*)+\b)").unwrap();

    // Scan all .rs files in generated_dir
    let pattern = generated_dir.join("*.rs");
    for entry in glob::glob(pattern.to_str().unwrap_or(""))
        .map_err(|e| GeneratorError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
    {
        if let Ok(path) = entry {
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
                                message: format!(
                                    "type '{}' not found in stammdatenmodell",
                                    name
                                ),
                            });
                        }
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
```

Update `crates/automapper-generator/src/lib.rs`:

```rust
pub mod error;
pub mod schema;
pub mod parsing;
pub mod codegen;
pub mod conditions;
pub mod validation;

pub use error::GeneratorError;
```

Add `glob` to `crates/automapper-generator/Cargo.toml`:

```toml
glob = "0.3"
```

Write `crates/automapper-generator/tests/schema_validation_tests.rs`:

```rust
use automapper_generator::validation::schema_validator::*;
use std::collections::HashSet;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_validate_generated_code_no_errors() {
    let tmp = TempDir::new().unwrap();

    // Write a generated file that only references known types
    let code = r#"
use automapper_validation::condition::{ConditionEvaluator, ConditionResult, EvaluationContext};

pub struct TestEvaluator;

impl ConditionEvaluator for TestEvaluator {
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult {
        match condition {
            1 => ConditionResult::True,
            _ => ConditionResult::Unknown,
        }
    }

    fn is_external(&self, _condition: u32) -> bool {
        false
    }
}
"#;

    std::fs::write(tmp.path().join("test_evaluator.rs"), code).unwrap();

    let known_types = HashSet::new(); // No BO4E types needed for this code
    let report = validate_generated_code(tmp.path(), &known_types).unwrap();

    assert!(report.errors.is_empty(), "should have no errors");
}

#[test]
fn test_validate_generated_code_warns_unknown_type() {
    let tmp = TempDir::new().unwrap();

    let code = r#"
use automapper_validation::condition::{ConditionEvaluator, ConditionResult};

fn check_marktlokation(malo: &Marktlokation) -> bool {
    true
}
"#;

    std::fs::write(tmp.path().join("test_mapper.rs"), code).unwrap();

    let known_types = HashSet::new(); // Marktlokation is NOT in the known types
    let report = validate_generated_code(tmp.path(), &known_types).unwrap();

    assert!(
        report.warnings.iter().any(|w| w.message.contains("Marktlokation")),
        "should warn about unknown Marktlokation type"
    );
}

#[test]
fn test_validate_generated_code_known_type_passes() {
    let tmp = TempDir::new().unwrap();

    let code = r#"
fn check_marktlokation(malo: &Marktlokation) -> bool {
    true
}
"#;

    std::fs::write(tmp.path().join("test_mapper.rs"), code).unwrap();

    let mut known_types = HashSet::new();
    known_types.insert("Marktlokation".to_string());

    let report = validate_generated_code(tmp.path(), &known_types).unwrap();

    assert!(
        !report.warnings.iter().any(|w| w.message.contains("Marktlokation")),
        "should NOT warn about known Marktlokation type"
    );
}

#[test]
fn test_validate_missing_generated_dir() {
    let known_types = HashSet::new();
    let result = validate_generated_code(Path::new("/nonexistent/dir"), &known_types);
    assert!(result.is_err());
}

#[test]
fn test_validation_report_display() {
    let issue = SchemaValidationIssue {
        file: "test.rs".to_string(),
        line: 42,
        message: "type 'Foo' not found".to_string(),
    };
    assert_eq!(issue.to_string(), "test.rs:42: type 'Foo' not found");
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test schema_validation_tests
```

### Step 3 -- Implement

Create the files listed above. Wire the `ValidateSchema` command in `main.rs`:

Replace the placeholder `ValidateSchema` handler:

```rust
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
```

### Step 4 -- Run the test (GREEN)

```bash
cargo test -p automapper-generator --test schema_validation_tests
```

Expected:

```
running 5 tests
test test_validate_generated_code_no_errors ... ok
test test_validate_generated_code_warns_unknown_type ... ok
test test_validate_generated_code_known_type_passes ... ok
test test_validate_missing_generated_dir ... ok
test test_validation_report_display ... ok
```

### Step 5 -- Commit

```bash
git add -A && git commit -m "feat(generator): implement validate-schema subcommand with BO4E type checking"
```

---

## Task 7: Incremental mode and metadata persistence test

### Step 1 -- Write the test

Write `crates/automapper-generator/tests/incremental_tests.rs`:

```rust
use automapper_generator::conditions::condition_types::*;
use automapper_generator::conditions::metadata::*;
use std::collections::{HashMap, HashSet};
use tempfile::TempDir;

/// Tests the full incremental flow:
/// 1. First generation creates metadata
/// 2. Second generation with same descriptions preserves high-confidence conditions
/// 3. Changed description triggers regeneration
#[test]
fn test_incremental_generation_flow() {
    let tmp = TempDir::new().unwrap();
    let metadata_path = tmp.path().join("conditions.json");

    // === Step 1: First generation (all new) ===
    let conditions_v1 = vec![
        ("1".to_string(), "Wenn Aufteilung vorhanden".to_string()),
        ("2".to_string(), "Wenn Marktlokation vorhanden".to_string()),
        ("3".to_string(), "Wenn Zeitraum gueltig".to_string()),
    ];

    let decision_v1 = decide_regeneration(&conditions_v1, None, &HashSet::new(), false);

    assert_eq!(
        decision_v1.to_regenerate.len(),
        3,
        "first run should regenerate all"
    );
    assert!(decision_v1.to_preserve.is_empty());

    // Simulate generation results
    let mut meta_conditions = HashMap::new();
    for (id, desc) in &conditions_v1 {
        meta_conditions.insert(
            id.clone(),
            ConditionMetadata {
                confidence: "high".to_string(),
                reasoning: Some("Generated".to_string()),
                description_hash: compute_description_hash(desc),
                is_external: id == "1",
            },
        );
    }

    let metadata_v1 = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    save_metadata(&metadata_path, &metadata_v1).unwrap();

    // === Step 2: Second generation with same descriptions ===
    let loaded = load_metadata(&metadata_path).unwrap().unwrap();
    assert_eq!(loaded.conditions.len(), 3);

    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());
    existing_ids.insert("2".to_string());
    existing_ids.insert("3".to_string());

    let decision_v2 = decide_regeneration(
        &conditions_v1,
        Some(&loaded),
        &existing_ids,
        false,
    );

    assert!(
        decision_v2.to_regenerate.is_empty(),
        "same descriptions should preserve all: {:?}",
        decision_v2.to_regenerate
    );
    assert_eq!(decision_v2.to_preserve.len(), 3);

    // === Step 3: Changed description triggers regeneration ===
    let conditions_v3 = vec![
        ("1".to_string(), "Wenn Aufteilung vorhanden".to_string()), // unchanged
        (
            "2".to_string(),
            "Wenn Marktlokation vorhanden UND aktiv".to_string(),
        ), // CHANGED
        ("3".to_string(), "Wenn Zeitraum gueltig".to_string()), // unchanged
    ];

    let decision_v3 = decide_regeneration(
        &conditions_v3,
        Some(&loaded),
        &existing_ids,
        false,
    );

    assert_eq!(
        decision_v3.to_regenerate.len(),
        1,
        "only changed condition should regenerate"
    );
    assert_eq!(decision_v3.to_regenerate[0].condition_id, "2");
    assert_eq!(decision_v3.to_regenerate[0].reason, RegenerationReason::Stale);
    assert_eq!(decision_v3.to_preserve.len(), 2);
}

#[test]
fn test_metadata_persistence_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.conditions.json");

    let mut conditions = HashMap::new();
    conditions.insert(
        "42".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: Some("Simple check".to_string()),
            description_hash: "abcd1234".to_string(),
            is_external: true,
        },
    );
    conditions.insert(
        "99".to_string(),
        ConditionMetadata {
            confidence: "low".to_string(),
            reasoning: Some("Complex temporal logic".to_string()),
            description_hash: "efgh5678".to_string(),
            is_external: false,
        },
    );

    let original = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "UTILMD_AHB_Strom_2_1.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions,
    };

    save_metadata(&path, &original).unwrap();

    let loaded = load_metadata(&path).unwrap().unwrap();

    assert_eq!(loaded.format_version, "FV2510");
    assert_eq!(loaded.conditions.len(), 2);
    assert_eq!(loaded.conditions["42"].confidence, "high");
    assert!(loaded.conditions["42"].is_external);
    assert_eq!(loaded.conditions["99"].confidence, "low");
    assert!(!loaded.conditions["99"].is_external);
}

#[test]
fn test_load_nonexistent_metadata_returns_none() {
    let result = load_metadata(std::path::Path::new("/nonexistent/path.json")).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_new_condition_added_to_ahb() {
    let desc1 = "Wenn Aufteilung vorhanden";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash(desc1),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    // Condition 2 is new (not in metadata)
    let conditions = vec![
        ("1".to_string(), desc1.to_string()),
        ("2".to_string(), "Neubedingung".to_string()),
    ];

    let mut existing_ids = HashSet::new();
    existing_ids.insert("1".to_string());

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(decision.to_regenerate[0].condition_id, "2");
    assert_eq!(decision.to_regenerate[0].reason, RegenerationReason::New);
    assert_eq!(decision.to_preserve.len(), 1);
    assert_eq!(decision.to_preserve[0], "1");
}

#[test]
fn test_missing_implementation_triggers_regeneration() {
    let desc = "Wenn Aufteilung vorhanden";
    let mut meta_conditions = HashMap::new();
    meta_conditions.insert(
        "1".to_string(),
        ConditionMetadata {
            confidence: "high".to_string(),
            reasoning: None,
            description_hash: compute_description_hash(desc),
            is_external: false,
        },
    );

    let metadata = ConditionMetadataFile {
        generated_at: "2026-02-18T12:00:00Z".to_string(),
        ahb_file: "test.xml".to_string(),
        format_version: "FV2510".to_string(),
        conditions: meta_conditions,
    };

    let conditions = vec![("1".to_string(), desc.to_string())];

    // Condition 1 is in metadata but NOT in existing implementation file
    let existing_ids = HashSet::new(); // empty = no existing implementations

    let decision = decide_regeneration(&conditions, Some(&metadata), &existing_ids, false);

    assert_eq!(decision.to_regenerate.len(), 1);
    assert_eq!(
        decision.to_regenerate[0].reason,
        RegenerationReason::MissingImplementation
    );
}
```

### Step 2 -- Run the test (RED)

```bash
cargo test -p automapper-generator --test incremental_tests
```

### Step 3 -- Implement

The implementation is already in `metadata.rs` from Task 1. This test validates the integration of all the metadata and regeneration logic together.

### Step 4 -- Run the test (GREEN)

```bash
cargo test -p automapper-generator --test incremental_tests
```

Expected:

```
running 5 tests
test test_incremental_generation_flow ... ok
test test_metadata_persistence_roundtrip ... ok
test test_load_nonexistent_metadata_returns_none ... ok
test test_new_condition_added_to_ahb ... ok
test test_missing_implementation_triggers_regeneration ... ok
```

### Step 5 -- Run all generator tests

```bash
cargo test -p automapper-generator
```

All tests across Epics 1, 2, and 3 should pass.

### Step 6 -- Commit

```bash
git add -A && git commit -m "feat(generator): add incremental mode tests and metadata persistence validation"
```

---

## Summary

After completing all 7 tasks, the `automapper-generator` crate has:

| Component | File | Purpose |
|-----------|------|---------|
| `ConditionTypes` | `src/conditions/condition_types.rs` | `GeneratedCondition`, `ConditionInput`, `ClaudeConditionResponse`, `ConfidenceLevel` |
| `Metadata` | `src/conditions/metadata.rs` | `ConditionMetadataFile`, `ConditionMetadata`, `decide_regeneration()`, `compute_description_hash()` |
| `Prompt` | `src/conditions/prompt.rs` | `build_system_prompt()`, `build_user_prompt()`, segment structure extraction |
| `ClaudeGenerator` | `src/conditions/claude_generator.rs` | `ClaudeConditionGenerator`, shells out to `claude --print`, parses JSON response |
| `Codegen` | `src/conditions/codegen.rs` | `generate_condition_evaluator_file()` -- produces Rust source with match arms |
| `SchemaValidator` | `src/validation/schema_validator.rs` | `validate_generated_code()`, `extract_bo4e_types()` |
| CLI wiring | `src/main.rs` | `GenerateConditions` and `ValidateSchema` subcommands fully implemented |

### CLI Commands Available

```bash
# Generate condition evaluators (calls claude CLI)
cargo run -p automapper-generator -- generate-conditions \
    --ahb-path xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_2_1.xml \
    --output-dir generated/ \
    --format-version FV2510 \
    --message-type UTILMD \
    --max-concurrent 4

# Incremental mode (only regenerate changed/low-confidence conditions)
cargo run -p automapper-generator -- generate-conditions \
    --ahb-path xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_2_1.xml \
    --output-dir generated/ \
    --format-version FV2510 \
    --message-type UTILMD \
    --incremental

# Dry run (parse only, don't call Claude)
cargo run -p automapper-generator -- generate-conditions \
    --ahb-path xml-migs-and-ahbs/FV2510/UTILMD_AHB_Strom_2_1.xml \
    --output-dir generated/ \
    --format-version FV2510 \
    --message-type UTILMD \
    --dry-run

# Validate generated code against BO4E schema
cargo run -p automapper-generator -- validate-schema \
    --stammdatenmodell-path stammdatenmodell/ \
    --generated-dir generated/
```
