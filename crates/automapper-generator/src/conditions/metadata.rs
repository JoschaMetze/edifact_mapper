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
        let reason = should_regenerate(
            id,
            description,
            existing_metadata,
            existing_condition_ids,
            force,
        );

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
