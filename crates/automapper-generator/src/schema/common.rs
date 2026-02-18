use serde::{Deserialize, Serialize};

/// Cardinality of a segment or group in the MIG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cardinality {
    /// Mandatory (M) — must appear exactly once or as specified.
    Mandatory,
    /// Required (R) — must appear.
    Required,
    /// Dependent (D) — depends on other segments.
    Dependent,
    /// Optional (O) — may appear.
    Optional,
    /// Not used (N) — must not appear.
    NotUsed,
    /// Conditional (C) — conditional on context.
    Conditional,
}

impl Cardinality {
    /// Parse from a status string (e.g., "M", "C", "R", "D", "O", "N").
    pub fn from_status(status: &str) -> Self {
        match status.trim() {
            "M" => Cardinality::Mandatory,
            "R" => Cardinality::Required,
            "D" => Cardinality::Dependent,
            "O" => Cardinality::Optional,
            "N" => Cardinality::NotUsed,
            "C" => Cardinality::Conditional,
            _ => Cardinality::Conditional, // Default for unknown
        }
    }

    /// Whether this cardinality means the element is required.
    pub fn is_required(&self) -> bool {
        matches!(self, Cardinality::Mandatory | Cardinality::Required)
    }
}

/// An allowed code value for a data element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeDefinition {
    /// The code value (e.g., "ORDERS", "E40").
    pub value: String,
    /// Human-readable name of the code.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
}

/// EDIFACT data element format type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdifactDataType {
    Alphabetic,
    Numeric,
    Alphanumeric,
}

/// Parsed EDIFACT format specification (e.g., "an..35", "n13").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdifactFormat {
    pub data_type: EdifactDataType,
    /// Minimum length (None for variable-length formats like "an..35").
    pub min_length: Option<usize>,
    /// Maximum length.
    pub max_length: usize,
}

impl EdifactFormat {
    /// Parse an EDIFACT format string (e.g., "an..35", "n13", "a3").
    pub fn parse(format: &str) -> Option<Self> {
        let format = format.trim();
        if format.is_empty() {
            return None;
        }

        // Regex-free parsing: extract type prefix, optional "..", and length
        let (type_str, rest) = if let Some(rest) = format.strip_prefix("an") {
            ("an", rest)
        } else if let Some(rest) = format.strip_prefix('a') {
            ("a", rest)
        } else if let Some(rest) = format.strip_prefix('n') {
            ("n", rest)
        } else {
            return None;
        };

        let data_type = match type_str {
            "a" => EdifactDataType::Alphabetic,
            "n" => EdifactDataType::Numeric,
            "an" => EdifactDataType::Alphanumeric,
            _ => return None,
        };

        let (is_variable, length_str) = if let Some(stripped) = rest.strip_prefix("..") {
            (true, stripped)
        } else {
            (false, rest)
        };

        let max_length: usize = length_str.parse().ok()?;
        let min_length = if is_variable { None } else { Some(max_length) };

        Some(EdifactFormat {
            data_type,
            min_length,
            max_length,
        })
    }
}
