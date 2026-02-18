use serde::{Deserialize, Serialize};

/// Confidence level of a generated condition implementation.
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
            _ => Err(format!("unknown confidence level: '{}'", s)),
        }
    }
}

/// A generated condition from the Claude CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCondition {
    /// The condition number (e.g., 42).
    pub condition_number: u32,

    /// Generated Rust code body for the condition evaluator function.
    /// None for external or low-confidence conditions.
    pub rust_code: Option<String>,

    /// Whether this condition requires external context.
    pub is_external: bool,

    /// Confidence level of the generation.
    pub confidence: ConfidenceLevel,

    /// Reasoning from the AI about its implementation choice.
    pub reasoning: Option<String>,

    /// External name for conditions requiring runtime context (e.g., "message_splitting").
    pub external_name: Option<String>,

    /// Original AHB description text.
    pub original_description: Option<String>,

    /// Segment/field references from the AHB that use this condition.
    pub referencing_fields: Option<Vec<String>>,
}

/// Input condition for generation.
#[derive(Debug, Clone)]
pub struct ConditionInput {
    /// Condition ID (e.g., "1", "42").
    pub id: String,

    /// Original German AHB description.
    pub description: String,

    /// Segment/field references that use this condition.
    pub referencing_fields: Option<Vec<String>>,
}

/// Response structure from Claude CLI.
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeConditionResponse {
    pub conditions: Vec<ClaudeConditionEntry>,
}

/// A single condition entry in the Claude response.
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
