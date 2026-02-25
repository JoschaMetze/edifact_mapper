//! Validation issue types.

use serde::{Deserialize, Serialize};

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational message, not a problem.
    Info,
    /// Warning that may indicate a problem but does not fail validation.
    Warning,
    /// Error that causes validation to fail.
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
        }
    }
}

/// Category of validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationCategory {
    /// Structural issues: missing segments, wrong order, MaxRep exceeded.
    Structure,
    /// Format issues: invalid data format (an..35, n13, dates).
    Format,
    /// Code issues: invalid code value not in allowed list.
    Code,
    /// AHB issues: PID-specific condition rule violations.
    Ahb,
}

impl std::fmt::Display for ValidationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationCategory::Structure => write!(f, "Structure"),
            ValidationCategory::Format => write!(f, "Format"),
            ValidationCategory::Code => write!(f, "Code"),
            ValidationCategory::Ahb => write!(f, "AHB"),
        }
    }
}

/// Serializable segment position for validation reports.
///
/// Mirrors `edifact_types::SegmentPosition` but with serde support,
/// since the edifact-types crate is intentionally zero-dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegmentPosition {
    /// 1-based segment number within the interchange.
    pub segment_number: u32,
    /// Byte offset from the start of the input.
    pub byte_offset: usize,
    /// 1-based message number within the interchange.
    pub message_number: u32,
}

impl From<edifact_types::SegmentPosition> for SegmentPosition {
    fn from(pos: edifact_types::SegmentPosition) -> Self {
        Self {
            segment_number: pos.segment_number,
            byte_offset: pos.byte_offset,
            message_number: pos.message_number,
        }
    }
}

/// A single validation issue found in an EDIFACT message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level of this issue.
    pub severity: Severity,

    /// Category of this issue.
    pub category: ValidationCategory,

    /// Machine-readable error code (e.g., "STR001", "AHB003").
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// Position in the EDIFACT message where the issue was found.
    pub segment_position: Option<SegmentPosition>,

    /// Field path within the segment (e.g., "SG2/NAD/C082/3039").
    pub field_path: Option<String>,

    /// The AHB rule that triggered this issue (e.g., "Muss [182] ∧ [152]").
    pub rule: Option<String>,

    /// The actual value found (if applicable).
    pub actual_value: Option<String>,

    /// The expected value (if applicable).
    pub expected_value: Option<String>,

    /// BO4E field path (e.g., "stammdaten.Marktlokation.marktlokationsId").
    /// Set when validation is triggered from BO4E input and errors can be
    /// traced back to the source BO4E structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bo4e_path: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue with the required fields.
    pub fn new(
        severity: Severity,
        category: ValidationCategory,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            code: code.into(),
            message: message.into(),
            segment_position: None,
            field_path: None,
            rule: None,
            actual_value: None,
            expected_value: None,
            bo4e_path: None,
        }
    }

    /// Builder: set the segment position.
    pub fn with_position(mut self, position: impl Into<SegmentPosition>) -> Self {
        self.segment_position = Some(position.into());
        self
    }

    /// Builder: set the field path.
    pub fn with_field_path(mut self, path: impl Into<String>) -> Self {
        self.field_path = Some(path.into());
        self
    }

    /// Builder: set the AHB rule.
    pub fn with_rule(mut self, rule: impl Into<String>) -> Self {
        self.rule = Some(rule.into());
        self
    }

    /// Builder: set the actual value.
    pub fn with_actual(mut self, value: impl Into<String>) -> Self {
        self.actual_value = Some(value.into());
        self
    }

    /// Builder: set the expected value.
    pub fn with_expected(mut self, value: impl Into<String>) -> Self {
        self.expected_value = Some(value.into());
        self
    }

    /// Builder: set the BO4E field path.
    pub fn with_bo4e_path(mut self, path: impl Into<String>) -> Self {
        self.bo4e_path = Some(path.into());
        self
    }

    /// Returns true if this is an error-level issue.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Returns true if this is a warning-level issue.
    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.code, self.message)?;
        if let Some(ref path) = self.field_path {
            write!(f, " at {path}")?;
        }
        if let Some(ref pos) = self.segment_position {
            write!(
                f,
                " (segment #{}, byte {})",
                pos.segment_number, pos.byte_offset
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_issue_builder() {
        let issue = ValidationIssue::new(
            Severity::Error,
            ValidationCategory::Ahb,
            "AHB001",
            "Required field missing",
        )
        .with_field_path("SG2/NAD/C082/3039")
        .with_rule("Muss [182] ∧ [152]")
        .with_position(SegmentPosition {
            segment_number: 5,
            byte_offset: 234,
            message_number: 1,
        });

        assert!(issue.is_error());
        assert!(!issue.is_warning());
        assert_eq!(issue.code, "AHB001");
        assert_eq!(issue.field_path.as_deref(), Some("SG2/NAD/C082/3039"));
        assert_eq!(issue.rule.as_deref(), Some("Muss [182] ∧ [152]"));
        assert_eq!(issue.segment_position.unwrap().segment_number, 5);
    }

    #[test]
    fn test_issue_display() {
        let issue = ValidationIssue::new(
            Severity::Error,
            ValidationCategory::Ahb,
            "AHB001",
            "Required field missing",
        )
        .with_field_path("NAD");

        let display = format!("{issue}");
        assert!(display.contains("[ERROR]"));
        assert!(display.contains("AHB001"));
        assert!(display.contains("Required field missing"));
        assert!(display.contains("at NAD"));
    }

    #[test]
    fn test_issue_serialization() {
        let issue = ValidationIssue::new(
            Severity::Warning,
            ValidationCategory::Code,
            "COD002",
            "Code not allowed for PID",
        );

        let json = serde_json::to_string_pretty(&issue).unwrap();
        // bo4e_path should be absent from JSON when None (skip_serializing_if)
        assert!(!json.contains("bo4e_path"));
        let deserialized: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, "COD002");
        assert_eq!(deserialized.severity, Severity::Warning);
        assert!(deserialized.bo4e_path.is_none());
    }

    #[test]
    fn test_bo4e_path_builder_and_serialization() {
        let issue = ValidationIssue::new(
            Severity::Error,
            ValidationCategory::Ahb,
            "AHB001",
            "Required field missing",
        )
        .with_field_path("SG4/SG5/LOC/C517/3225")
        .with_bo4e_path("stammdaten.Marktlokation.marktlokationsId");

        assert_eq!(
            issue.bo4e_path.as_deref(),
            Some("stammdaten.Marktlokation.marktlokationsId")
        );

        let json = serde_json::to_string_pretty(&issue).unwrap();
        assert!(json.contains("bo4e_path"));
        assert!(json.contains("stammdaten.Marktlokation.marktlokationsId"));

        let deserialized: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.bo4e_path.as_deref(),
            Some("stammdaten.Marktlokation.marktlokationsId")
        );
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", ValidationCategory::Structure), "Structure");
        assert_eq!(format!("{}", ValidationCategory::Ahb), "AHB");
    }

    #[test]
    fn test_position_from_edifact_types() {
        let edifact_pos = edifact_types::SegmentPosition::new(3, 100, 1);
        let pos: SegmentPosition = edifact_pos.into();
        assert_eq!(pos.segment_number, 3);
        assert_eq!(pos.byte_offset, 100);
        assert_eq!(pos.message_number, 1);
    }
}
