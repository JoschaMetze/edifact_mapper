//! Validation level configuration.

use serde::{Deserialize, Serialize};

/// Level of validation strictness.
///
/// Controls which checks are performed during validation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Validate only EDIFACT structure: segment presence, ordering, and
    /// repetition counts against the MIG schema.
    #[serde(alias = "structure")]
    Structure,

    /// Validate structure plus AHB condition expressions for the detected
    /// Pruefidentifikator. Requires a registered `ConditionEvaluator`.
    #[serde(alias = "conditions")]
    Conditions,

    /// Full validation: structure, conditions, format checks, and code
    /// value restrictions. The most thorough level.
    #[default]
    #[serde(alias = "full")]
    Full,
}

impl std::fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationLevel::Structure => write!(f, "Structure"),
            ValidationLevel::Conditions => write!(f, "Conditions"),
            ValidationLevel::Full => write!(f, "Full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_full() {
        assert_eq!(ValidationLevel::default(), ValidationLevel::Full);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ValidationLevel::Structure), "Structure");
        assert_eq!(format!("{}", ValidationLevel::Conditions), "Conditions");
        assert_eq!(format!("{}", ValidationLevel::Full), "Full");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let level = ValidationLevel::Conditions;
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: ValidationLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}
