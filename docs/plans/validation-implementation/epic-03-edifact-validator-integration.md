---
feature: validation-implementation
epic: 3
title: "EdifactValidator & Integration"
depends_on: [validation-implementation/E02]
estimated_tasks: 5
crate: automapper-validation
status: in_progress
---

# Epic 3: EdifactValidator & Integration

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-validation/src/`. All code must compile with `cargo check -p automapper-validation`.

**Goal:** Implement the top-level `EdifactValidator` that orchestrates the full validation pipeline: parse EDIFACT input, detect message type and Pruefidentifikator, look up AHB rules, evaluate condition expressions per field/segment, and produce a `ValidationReport`. This ties together the parser (Feature 1), the condition expression parser (Epic 1), and the evaluator (Epic 2) into a usable validation API. This ports the C# types `EdifactValidator`, `ValidationReport`, `ValidationIssue`, `ValidationLevel`/`Severity`/`ValidationCategory`, `ErrorCodes`, and `AhbValidator`.

**Architecture:** The `EdifactValidator<E>` is generic over the `ConditionEvaluator` implementation, orchestrating a pipeline of EDIFACT parsing into `RawSegment` references, message type detection from UNH, AHB condition expression evaluation per field based on Pruefidentifikator, and `ValidationReport` production. Supports three validation levels (Structure, Conditions, Full) with typed `ValidationIssue` items categorized by Structure/Format/Code/AHB.

**Tech Stack:** thiserror 2.x, serde 1.x + serde_json 1.x, edifact-types (path dep), edifact-parser (path dep), automapper-core (path dep)

---

## Task 1: Define validation enums, structs, and error codes

### Description
Define the core validation types: `Severity`, `ValidationCategory`, `ValidationLevel`, `ValidationIssue`, `ValidationReport`, and the error code constants module. These are serializable types that form the public API of validation results.

### Implementation

**`crates/automapper-validation/src/validator.rs`**:
```rust
//! EDIFACT message validator.

mod codes;
mod issue;
mod level;
mod report;
mod validate;

pub use codes::ErrorCodes;
pub use issue::{Severity, ValidationCategory, ValidationIssue};
pub use level::ValidationLevel;
pub use report::ValidationReport;
pub use validate::EdifactValidator;
```

**`crates/automapper-validation/src/validator/level.rs`**:
```rust
//! Validation level configuration.

use serde::{Deserialize, Serialize};

/// Level of validation strictness.
///
/// Controls which checks are performed during validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Validate only EDIFACT structure: segment presence, ordering, and
    /// repetition counts against the MIG schema.
    Structure,

    /// Validate structure plus AHB condition expressions for the detected
    /// Pruefidentifikator. Requires a registered `ConditionEvaluator`.
    Conditions,

    /// Full validation: structure, conditions, format checks, and code
    /// value restrictions. The most thorough level.
    Full,
}

impl Default for ValidationLevel {
    fn default() -> Self {
        ValidationLevel::Full
    }
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
```

**`crates/automapper-validation/src/validator/issue.rs`**:
```rust
//! Validation issue types.

use edifact_types::SegmentPosition;
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
        }
    }

    /// Builder: set the segment position.
    pub fn with_position(mut self, position: SegmentPosition) -> Self {
        self.segment_position = Some(position);
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
            write!(f, " (segment #{}, byte {})", pos.segment_number, pos.byte_offset)?;
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
        let deserialized: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, "COD002");
        assert_eq!(deserialized.severity, Severity::Warning);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", ValidationCategory::Structure), "Structure");
        assert_eq!(format!("{}", ValidationCategory::Ahb), "AHB");
    }
}
```

**`crates/automapper-validation/src/validator/codes.rs`**:
```rust
//! Standard error codes for validation issues.
//!
//! Error codes follow a prefix convention:
//! - `STR0xx`: Structure validation
//! - `FMT0xx`: Format validation
//! - `COD0xx`: Code value validation
//! - `AHB0xx`: AHB condition rule validation

/// Standard error codes for validation issues.
pub struct ErrorCodes;

impl ErrorCodes {
    // --- Structure validation (STR001-STR099) ---

    /// A mandatory segment is missing.
    pub const MISSING_MANDATORY_SEGMENT: &'static str = "STR001";

    /// Segment repetitions exceed the MIG MaxRep count.
    pub const MAX_REPETITIONS_EXCEEDED: &'static str = "STR002";

    /// Unexpected segment found (not defined in MIG for this message type).
    pub const UNEXPECTED_SEGMENT: &'static str = "STR003";

    /// Segments are in wrong order according to the MIG.
    pub const WRONG_SEGMENT_ORDER: &'static str = "STR004";

    /// A mandatory segment group is missing.
    pub const MISSING_MANDATORY_GROUP: &'static str = "STR005";

    /// Segment group repetitions exceed MaxRep count.
    pub const GROUP_MAX_REP_EXCEEDED: &'static str = "STR006";

    // --- Format validation (FMT001-FMT099) ---

    /// Value exceeds maximum allowed length.
    pub const VALUE_TOO_LONG: &'static str = "FMT001";

    /// Value does not match required numeric format.
    pub const INVALID_NUMERIC_FORMAT: &'static str = "FMT002";

    /// Value does not match required alphanumeric format.
    pub const INVALID_ALPHANUMERIC_FORMAT: &'static str = "FMT003";

    /// Invalid date/time format.
    pub const INVALID_DATE_FORMAT: &'static str = "FMT004";

    /// Value is shorter than minimum required length.
    pub const VALUE_TOO_SHORT: &'static str = "FMT005";

    /// A required element is empty.
    pub const REQUIRED_ELEMENT_EMPTY: &'static str = "FMT006";

    // --- Code validation (COD001-COD099) ---

    /// Code value is not in the allowed code list for this element.
    pub const INVALID_CODE_VALUE: &'static str = "COD001";

    /// Code value is not allowed for this specific Pruefidentifikator.
    pub const CODE_NOT_ALLOWED_FOR_PID: &'static str = "COD002";

    // --- AHB validation (AHB001-AHB999) ---

    /// A field required by the AHB for this PID is missing.
    pub const MISSING_REQUIRED_FIELD: &'static str = "AHB001";

    /// A field is present but not allowed by the AHB for this PID.
    pub const FIELD_NOT_ALLOWED_FOR_PID: &'static str = "AHB002";

    /// A conditional AHB rule is violated.
    pub const CONDITIONAL_RULE_VIOLATION: &'static str = "AHB003";

    /// The Pruefidentifikator is unknown / not supported.
    pub const UNKNOWN_PRUEFIDENTIFIKATOR: &'static str = "AHB004";

    /// A condition expression could not be fully evaluated (Unknown result).
    pub const CONDITION_UNKNOWN: &'static str = "AHB005";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_prefixes() {
        assert!(ErrorCodes::MISSING_MANDATORY_SEGMENT.starts_with("STR"));
        assert!(ErrorCodes::VALUE_TOO_LONG.starts_with("FMT"));
        assert!(ErrorCodes::INVALID_CODE_VALUE.starts_with("COD"));
        assert!(ErrorCodes::MISSING_REQUIRED_FIELD.starts_with("AHB"));
    }

    #[test]
    fn test_all_codes_are_unique() {
        let codes = [
            ErrorCodes::MISSING_MANDATORY_SEGMENT,
            ErrorCodes::MAX_REPETITIONS_EXCEEDED,
            ErrorCodes::UNEXPECTED_SEGMENT,
            ErrorCodes::WRONG_SEGMENT_ORDER,
            ErrorCodes::MISSING_MANDATORY_GROUP,
            ErrorCodes::GROUP_MAX_REP_EXCEEDED,
            ErrorCodes::VALUE_TOO_LONG,
            ErrorCodes::INVALID_NUMERIC_FORMAT,
            ErrorCodes::INVALID_ALPHANUMERIC_FORMAT,
            ErrorCodes::INVALID_DATE_FORMAT,
            ErrorCodes::VALUE_TOO_SHORT,
            ErrorCodes::REQUIRED_ELEMENT_EMPTY,
            ErrorCodes::INVALID_CODE_VALUE,
            ErrorCodes::CODE_NOT_ALLOWED_FOR_PID,
            ErrorCodes::MISSING_REQUIRED_FIELD,
            ErrorCodes::FIELD_NOT_ALLOWED_FOR_PID,
            ErrorCodes::CONDITIONAL_RULE_VIOLATION,
            ErrorCodes::UNKNOWN_PRUEFIDENTIFIKATOR,
            ErrorCodes::CONDITION_UNKNOWN,
        ];

        let unique: std::collections::HashSet<&str> = codes.iter().copied().collect();
        assert_eq!(
            codes.len(),
            unique.len(),
            "Duplicate error codes found"
        );
    }
}
```

### Verification
```bash
cargo test -p automapper-validation validator::level::tests
cargo test -p automapper-validation validator::issue::tests
cargo test -p automapper-validation validator::codes::tests
```

### Commit
```
feat(automapper-validation): define validation types and error codes

ValidationLevel (Structure/Conditions/Full), Severity (Info/Warning/Error),
ValidationCategory (Structure/Format/Code/Ahb), ValidationIssue with builder
pattern, and ErrorCodes constants with STR/FMT/COD/AHB prefixes. All types
derive Serialize/Deserialize for API use.
```

---

## Task 2: Define ValidationReport

### Description
Implement the `ValidationReport` struct that aggregates validation issues with convenience accessors for errors, warnings, and validity checking.

### Implementation

**`crates/automapper-validation/src/validator/report.rs`**:
```rust
//! Validation report aggregating all issues from a validation run.

use serde::{Deserialize, Serialize};

use super::issue::{Severity, ValidationCategory, ValidationIssue};
use super::level::ValidationLevel;

/// Complete validation report for an EDIFACT message.
///
/// Contains all issues found during validation, with convenience methods
/// for filtering by severity and checking overall validity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The detected EDIFACT message type (e.g., "UTILMD", "ORDERS").
    pub message_type: String,

    /// The detected Pruefidentifikator (e.g., "11001", "55001").
    pub pruefidentifikator: Option<String>,

    /// The detected format version (e.g., "FV2510").
    pub format_version: Option<String>,

    /// The validation level that was used.
    pub level: ValidationLevel,

    /// All validation issues found.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// Create a new empty validation report.
    pub fn new(message_type: impl Into<String>, level: ValidationLevel) -> Self {
        Self {
            message_type: message_type.into(),
            pruefidentifikator: None,
            format_version: None,
            level,
            issues: Vec::new(),
        }
    }

    /// Builder: set the Pruefidentifikator.
    pub fn with_pruefidentifikator(mut self, pid: impl Into<String>) -> Self {
        self.pruefidentifikator = Some(pid.into());
        self
    }

    /// Builder: set the format version.
    pub fn with_format_version(mut self, fv: impl Into<String>) -> Self {
        self.format_version = Some(fv.into());
        self
    }

    /// Add a validation issue.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Add multiple validation issues.
    pub fn add_issues(&mut self, issues: impl IntoIterator<Item = ValidationIssue>) {
        self.issues.extend(issues);
    }

    /// Returns `true` if there are no error-level issues.
    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    /// Returns the number of error-level issues.
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == Severity::Error).count()
    }

    /// Returns the number of warning-level issues.
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count()
    }

    /// Returns all error-level issues.
    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == Severity::Error)
    }

    /// Returns all warning-level issues.
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
    }

    /// Returns all info-level issues.
    pub fn infos(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == Severity::Info)
    }

    /// Returns issues filtered by category.
    pub fn by_category(&self, category: ValidationCategory) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(move |i| i.category == category)
    }

    /// Returns the total number of issues.
    pub fn total_issues(&self) -> usize {
        self.issues.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::issue::ValidationCategory;

    fn make_error(code: &str) -> ValidationIssue {
        ValidationIssue::new(Severity::Error, ValidationCategory::Ahb, code, "test error")
    }

    fn make_warning(code: &str) -> ValidationIssue {
        ValidationIssue::new(
            Severity::Warning,
            ValidationCategory::Structure,
            code,
            "test warning",
        )
    }

    fn make_info(code: &str) -> ValidationIssue {
        ValidationIssue::new(Severity::Info, ValidationCategory::Code, code, "test info")
    }

    #[test]
    fn test_empty_report_is_valid() {
        let report = ValidationReport::new("UTILMD", ValidationLevel::Full);
        assert!(report.is_valid());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
        assert_eq!(report.total_issues(), 0);
    }

    #[test]
    fn test_report_with_errors_is_invalid() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Full);
        report.add_issue(make_error("AHB001"));

        assert!(!report.is_valid());
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_report_with_only_warnings_is_valid() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Full);
        report.add_issue(make_warning("STR001"));

        assert!(report.is_valid());
        assert_eq!(report.warning_count(), 1);
        assert_eq!(report.error_count(), 0);
    }

    #[test]
    fn test_report_mixed_issues() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Full)
            .with_pruefidentifikator("11001")
            .with_format_version("FV2510");

        report.add_issue(make_error("AHB001"));
        report.add_issue(make_error("AHB003"));
        report.add_issue(make_warning("STR002"));
        report.add_issue(make_info("COD001"));

        assert!(!report.is_valid());
        assert_eq!(report.error_count(), 2);
        assert_eq!(report.warning_count(), 1);
        assert_eq!(report.total_issues(), 4);
        assert_eq!(report.errors().count(), 2);
        assert_eq!(report.warnings().count(), 1);
        assert_eq!(report.infos().count(), 1);
    }

    #[test]
    fn test_report_by_category() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Full);
        report.add_issue(make_error("AHB001"));
        report.add_issue(make_warning("STR002"));

        assert_eq!(report.by_category(ValidationCategory::Ahb).count(), 1);
        assert_eq!(report.by_category(ValidationCategory::Structure).count(), 1);
        assert_eq!(report.by_category(ValidationCategory::Format).count(), 0);
    }

    #[test]
    fn test_report_add_issues() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Full);
        let issues = vec![make_error("AHB001"), make_warning("STR001")];
        report.add_issues(issues);

        assert_eq!(report.total_issues(), 2);
    }

    #[test]
    fn test_report_serialization() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Conditions)
            .with_pruefidentifikator("11001")
            .with_format_version("FV2510");
        report.add_issue(make_error("AHB001"));

        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("UTILMD"));
        assert!(json.contains("11001"));
        assert!(json.contains("AHB001"));

        let deserialized: ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message_type, "UTILMD");
        assert_eq!(deserialized.pruefidentifikator.as_deref(), Some("11001"));
        assert_eq!(deserialized.total_issues(), 1);
    }
}
```

### Verification
```bash
cargo test -p automapper-validation validator::report::tests
```

### Commit
```
feat(automapper-validation): implement ValidationReport with issue aggregation

ValidationReport collects issues with filtering by severity and category,
validity checking, builder pattern for metadata, and full serde support.
```

---

## Task 3: Implement EdifactValidator

### Description
Implement the `EdifactValidator<E>` struct that orchestrates the complete validation pipeline. Given raw EDIFACT input, it:

1. Parses the EDIFACT content into `RawSegment` references
2. Detects the message type from UNH
3. Evaluates AHB condition expressions for each field based on the Pruefidentifikator
4. Produces a `ValidationReport` with all issues found

The validator is generic over the `ConditionEvaluator` implementation, allowing generated evaluators to be plugged in.

### Implementation

**`crates/automapper-validation/src/validator/validate.rs`**:
```rust
//! Main EdifactValidator implementation.

use crate::eval::{
    ConditionEvaluator, ConditionExprEvaluator, ConditionResult, EvaluationContext,
    ExternalConditionProvider,
};
use crate::error::ValidationError;
use crate::expr::ConditionParser;

use super::codes::ErrorCodes;
use super::issue::{Severity, ValidationCategory, ValidationIssue};
use super::level::ValidationLevel;
use super::report::ValidationReport;

/// AHB field definition for validation.
///
/// Represents a single field in an AHB rule table with its status
/// and allowed codes for a specific Pruefidentifikator.
#[derive(Debug, Clone)]
pub struct AhbFieldRule {
    /// Segment path (e.g., "SG2/NAD/C082/3039").
    pub segment_path: String,

    /// Human-readable field name (e.g., "MP-ID des MSB").
    pub name: String,

    /// AHB status (e.g., "Muss [182] ∧ [152]", "X", "Kann").
    pub ahb_status: String,

    /// Allowed code values with their AHB status.
    pub codes: Vec<AhbCodeRule>,
}

/// An allowed code value within an AHB field rule.
#[derive(Debug, Clone)]
pub struct AhbCodeRule {
    /// The code value (e.g., "E01", "Z33").
    pub value: String,

    /// Description of the code (e.g., "Anmeldung").
    pub description: String,

    /// AHB status for this code (e.g., "X", "Muss").
    pub ahb_status: String,
}

/// AHB workflow definition for a specific Pruefidentifikator.
#[derive(Debug, Clone)]
pub struct AhbWorkflow {
    /// The Pruefidentifikator (e.g., "11001", "55001").
    pub pruefidentifikator: String,

    /// Description of the workflow.
    pub description: String,

    /// Communication direction (e.g., "NB an LF").
    pub communication_direction: Option<String>,

    /// All field rules for this workflow.
    pub fields: Vec<AhbFieldRule>,
}

/// Validates EDIFACT messages against AHB business rules.
///
/// The validator is generic over the `ConditionEvaluator` implementation,
/// which is typically generated from AHB XML schemas.
///
/// # Example
///
/// ```ignore
/// use automapper_validation::validator::EdifactValidator;
/// use automapper_validation::eval::NoOpExternalProvider;
///
/// let evaluator = UtilmdConditionEvaluatorFV2510::new();
/// let validator = EdifactValidator::new(evaluator);
/// let external = NoOpExternalProvider;
///
/// let report = validator.validate(
///     edifact_bytes,
///     ValidationLevel::Full,
///     &external,
///     Some(&ahb_workflow),
/// )?;
///
/// if !report.is_valid() {
///     for error in report.errors() {
///         eprintln!("{error}");
///     }
/// }
/// ```
pub struct EdifactValidator<E: ConditionEvaluator> {
    evaluator: E,
}

impl<E: ConditionEvaluator> EdifactValidator<E> {
    /// Create a new validator with the given condition evaluator.
    pub fn new(evaluator: E) -> Self {
        Self { evaluator }
    }

    /// Validate an EDIFACT message.
    ///
    /// # Arguments
    ///
    /// * `input` - Raw EDIFACT bytes
    /// * `level` - Validation strictness level
    /// * `external` - Provider for external conditions
    /// * `workflow` - Optional AHB workflow definition for the PID
    ///
    /// # Returns
    ///
    /// A `ValidationReport` with all issues found, or an error if
    /// the EDIFACT content could not be parsed at all.
    pub fn validate(
        &self,
        input: &[u8],
        level: ValidationLevel,
        external: &dyn ExternalConditionProvider,
        workflow: Option<&AhbWorkflow>,
    ) -> Result<ValidationReport, ValidationError> {
        let input_str = std::str::from_utf8(input).map_err(|_| {
            ValidationError::Parse(edifact_parser::ParseError::UnexpectedEof)
        })?;

        // Collect segments using the parser
        let segments = self.parse_segments(input_str)?;

        // Detect message type from UNH segment
        let message_type = self.detect_message_type(&segments).unwrap_or("UNKNOWN");

        // Build the report
        let mut report = ValidationReport::new(message_type, level)
            .with_format_version(self.evaluator.format_version());

        // Detect PID from RFF+Z13
        if let Some(pid) = self.detect_pruefidentifikator(&segments) {
            report.pruefidentifikator = Some(pid.to_string());
        }

        // Create evaluation context
        let ctx = EvaluationContext::new(
            report.pruefidentifikator.as_deref().unwrap_or(""),
            external,
            &segments,
        );

        // Structure validation (always performed)
        self.validate_structure(&segments, &mut report);

        // Condition validation (if level >= Conditions and workflow provided)
        if matches!(level, ValidationLevel::Conditions | ValidationLevel::Full) {
            if let Some(wf) = workflow {
                self.validate_conditions(wf, &ctx, &mut report);
            }
        }

        Ok(report)
    }

    /// Parse EDIFACT content into segments.
    fn parse_segments<'a>(
        &self,
        _input: &'a str,
    ) -> Result<Vec<edifact_types::RawSegment<'a>>, ValidationError> {
        // TODO: Use EdifactStreamParser from edifact-parser crate to parse segments.
        // For now, return empty vec. The actual parsing will be wired up when
        // Feature 1 (edifact-parser) is integrated.
        Ok(Vec::new())
    }

    /// Detect the message type from the UNH segment.
    fn detect_message_type<'a>(
        &self,
        segments: &'a [edifact_types::RawSegment<'a>],
    ) -> Option<&'a str> {
        segments
            .iter()
            .find(|s| s.id == "UNH")
            .and_then(|unh| unh.elements.get(1))
            .and_then(|e| e.first())
            .copied()
    }

    /// Detect the Pruefidentifikator from RFF+Z13.
    fn detect_pruefidentifikator<'a>(
        &self,
        segments: &'a [edifact_types::RawSegment<'a>],
    ) -> Option<&'a str> {
        segments.iter().find_map(|s| {
            if s.id != "RFF" {
                return None;
            }
            let qualifier = s.elements.get(0)?.get(0)?;
            if *qualifier == "Z13" {
                s.elements.get(0)?.get(1).copied()
            } else {
                None
            }
        })
    }

    /// Validate EDIFACT structure (segment presence, ordering).
    fn validate_structure(
        &self,
        _segments: &[edifact_types::RawSegment],
        _report: &mut ValidationReport,
    ) {
        // TODO: Implement MIG structure validation when MIG schema types
        // are available from automapper-generator. For now, this is a
        // placeholder that will be filled in when the generator crate
        // provides MigSchema types.
    }

    /// Validate AHB conditions for each field in the workflow.
    fn validate_conditions(
        &self,
        workflow: &AhbWorkflow,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        let expr_eval = ConditionExprEvaluator::new(&self.evaluator);

        for field in &workflow.fields {
            // Evaluate the AHB status condition expression
            let condition_result = expr_eval.evaluate_status(&field.ahb_status, ctx);

            match condition_result {
                ConditionResult::True => {
                    // Condition is met - field is required/applicable
                    if is_mandatory_status(&field.ahb_status) {
                        let segment_id = extract_segment_id(&field.segment_path);
                        if !ctx.has_segment(&segment_id) {
                            report.add_issue(
                                ValidationIssue::new(
                                    Severity::Error,
                                    ValidationCategory::Ahb,
                                    ErrorCodes::MISSING_REQUIRED_FIELD,
                                    format!(
                                        "Required field '{}' at {} is missing",
                                        field.name, field.segment_path
                                    ),
                                )
                                .with_field_path(&field.segment_path)
                                .with_rule(&field.ahb_status),
                            );
                        }
                    }

                    // Validate code values if field has code restrictions
                    self.validate_field_codes(field, ctx, report);
                }
                ConditionResult::False => {
                    // Condition not met - field not required, skip
                }
                ConditionResult::Unknown => {
                    // Cannot determine - add info-level warning
                    report.add_issue(
                        ValidationIssue::new(
                            Severity::Info,
                            ValidationCategory::Ahb,
                            ErrorCodes::CONDITION_UNKNOWN,
                            format!(
                                "Condition for field '{}' could not be fully evaluated (external conditions missing)",
                                field.name
                            ),
                        )
                        .with_field_path(&field.segment_path)
                        .with_rule(&field.ahb_status),
                    );
                }
            }
        }
    }

    /// Validate code values for a field against AHB allowed codes.
    fn validate_field_codes(
        &self,
        field: &AhbFieldRule,
        ctx: &EvaluationContext,
        report: &mut ValidationReport,
    ) {
        if field.codes.is_empty() {
            return;
        }

        let allowed_codes: Vec<&str> = field
            .codes
            .iter()
            .filter(|c| c.ahb_status == "X" || c.ahb_status.starts_with("Muss"))
            .map(|c| c.value.as_str())
            .collect();

        if allowed_codes.is_empty() {
            return;
        }

        let segment_id = extract_segment_id(&field.segment_path);
        let matching_segments = ctx.find_segments(&segment_id);

        for segment in matching_segments {
            if let Some(first_element) = segment.elements.first() {
                if let Some(code_value) = first_element.first() {
                    if !code_value.is_empty() && !allowed_codes.contains(code_value) {
                        report.add_issue(
                            ValidationIssue::new(
                                Severity::Error,
                                ValidationCategory::Code,
                                ErrorCodes::CODE_NOT_ALLOWED_FOR_PID,
                                format!(
                                    "Code '{}' is not allowed for this PID. Allowed: [{}]",
                                    code_value,
                                    allowed_codes.join(", ")
                                ),
                            )
                            .with_field_path(&field.segment_path)
                            .with_actual(*code_value)
                            .with_expected(allowed_codes.join(", ")),
                        );
                    }
                }
            }
        }
    }
}

/// Check if an AHB status is mandatory (Muss or X prefix).
fn is_mandatory_status(status: &str) -> bool {
    let trimmed = status.trim();
    trimmed.starts_with("Muss") || trimmed.starts_with('X')
}

/// Extract the segment ID from a field path like "SG2/NAD/C082/3039" -> "NAD".
fn extract_segment_id(path: &str) -> String {
    for part in path.split('/') {
        // Skip segment group identifiers and composite/element identifiers
        if part.starts_with("SG") || part.starts_with("C_") || part.starts_with("D_") {
            continue;
        }
        // Return first 3-letter uppercase segment identifier
        if part.len() >= 3
            && part
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return part.to_string();
        }
    }
    // Fallback: return the last part
    path.split('/').last().unwrap_or(path).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{ConditionResult as CR, NoOpExternalProvider};
    use std::collections::HashMap;

    /// Mock evaluator for testing the validator.
    struct MockEvaluator {
        results: HashMap<u32, CR>,
    }

    impl MockEvaluator {
        fn new(results: Vec<(u32, CR)>) -> Self {
            Self {
                results: results.into_iter().collect(),
            }
        }

        fn all_true(ids: &[u32]) -> Self {
            Self::new(ids.iter().map(|&id| (id, CR::True)).collect())
        }
    }

    impl ConditionEvaluator for MockEvaluator {
        fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> CR {
            self.results.get(&condition).copied().unwrap_or(CR::Unknown)
        }
        fn is_external(&self, _condition: u32) -> bool {
            false
        }
        fn message_type(&self) -> &str {
            "UTILMD"
        }
        fn format_version(&self) -> &str {
            "FV2510"
        }
    }

    // === Helper function tests ===

    #[test]
    fn test_is_mandatory_status() {
        assert!(is_mandatory_status("Muss"));
        assert!(is_mandatory_status("Muss [182] ∧ [152]"));
        assert!(is_mandatory_status("X"));
        assert!(is_mandatory_status("X [567]"));
        assert!(!is_mandatory_status("Soll [1]"));
        assert!(!is_mandatory_status("Kann [1]"));
        assert!(!is_mandatory_status(""));
    }

    #[test]
    fn test_extract_segment_id_simple() {
        assert_eq!(extract_segment_id("NAD"), "NAD");
    }

    #[test]
    fn test_extract_segment_id_with_sg_prefix() {
        assert_eq!(extract_segment_id("SG2/NAD/C082/3039"), "NAD");
    }

    #[test]
    fn test_extract_segment_id_nested_sg() {
        assert_eq!(extract_segment_id("SG4/SG8/SEQ/C286/6350"), "SEQ");
    }

    // === Validator tests with mock data ===

    #[test]
    fn test_validate_missing_mandatory_field() {
        let evaluator = MockEvaluator::all_true(&[182, 152]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "SG2/NAD/C082/3039".to_string(),
                name: "MP-ID des MSB".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        // Validate empty EDIFACT (will have no segments)
        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Should have an error for missing mandatory field
        assert!(!report.is_valid());
        let errors: Vec<_> = report.errors().collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, ErrorCodes::MISSING_REQUIRED_FIELD);
        assert!(errors[0].message.contains("MP-ID des MSB"));
    }

    #[test]
    fn test_validate_condition_false_no_error() {
        // When condition evaluates to False, field is not required
        let evaluator = MockEvaluator::new(vec![(182, CR::True), (152, CR::False)]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "NAD".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Condition is false, so field is not required - no error
        assert!(report.is_valid());
    }

    #[test]
    fn test_validate_condition_unknown_adds_info() {
        // When condition is Unknown, add an info-level note
        let evaluator = MockEvaluator::new(vec![(182, CR::True)]);
        // 152 is not registered -> Unknown
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "NAD".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Should be valid (Unknown is not an error) but have an info issue
        assert!(report.is_valid());
        let infos: Vec<_> = report.infos().collect();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].code, ErrorCodes::CONDITION_UNKNOWN);
    }

    #[test]
    fn test_validate_structure_level_skips_conditions() {
        let evaluator = MockEvaluator::all_true(&[182, 152]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "11001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "NAD".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss [182] ∧ [152]".to_string(),
                codes: vec![],
            }],
        };

        // With Structure level, conditions are not checked
        let report = validator
            .validate(
                b"",
                ValidationLevel::Structure,
                &external,
                Some(&workflow),
            )
            .unwrap();

        // No AHB errors because conditions were not evaluated
        assert!(report.is_valid());
        assert_eq!(report.by_category(ValidationCategory::Ahb).count(), 0);
    }

    #[test]
    fn test_validate_no_workflow_no_condition_errors() {
        let evaluator = MockEvaluator::all_true(&[]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        // No workflow provided
        let report = validator
            .validate(b"", ValidationLevel::Full, &external, None)
            .unwrap();

        assert!(report.is_valid());
    }

    #[test]
    fn test_validate_bare_muss_always_required() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: Some("NB an LF".to_string()),
            fields: vec![AhbFieldRule {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Partnerrolle".to_string(),
                ahb_status: "Muss".to_string(), // No conditions
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Bare "Muss" with no conditions -> unconditionally required -> missing = error
        assert!(!report.is_valid());
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_validate_x_status_is_mandatory() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "DTM".to_string(),
                name: "Datum".to_string(),
                ahb_status: "X".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        assert!(!report.is_valid());
        let errors: Vec<_> = report.errors().collect();
        assert_eq!(errors[0].code, ErrorCodes::MISSING_REQUIRED_FIELD);
    }

    #[test]
    fn test_validate_soll_not_mandatory() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let workflow = AhbWorkflow {
            pruefidentifikator: "55001".to_string(),
            description: "Test".to_string(),
            communication_direction: None,
            fields: vec![AhbFieldRule {
                segment_path: "DTM".to_string(),
                name: "Datum".to_string(),
                ahb_status: "Soll".to_string(),
                codes: vec![],
            }],
        };

        let report = validator
            .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
            .unwrap();

        // Soll is not mandatory, so missing is not an error
        assert!(report.is_valid());
    }

    #[test]
    fn test_report_includes_metadata() {
        let evaluator = MockEvaluator::new(vec![]);
        let validator = EdifactValidator::new(evaluator);
        let external = NoOpExternalProvider;

        let report = validator
            .validate(b"", ValidationLevel::Full, &external, None)
            .unwrap();

        assert_eq!(report.format_version.as_deref(), Some("FV2510"));
        assert_eq!(report.level, ValidationLevel::Full);
    }
}
```

### Verification
```bash
cargo test -p automapper-validation validator::validate::tests
cargo test -p automapper-validation
cargo clippy -p automapper-validation -- -D warnings
```

### Commit
```
feat(automapper-validation): implement EdifactValidator with condition evaluation

EdifactValidator<E> validates EDIFACT messages against AHB rules. Evaluates
condition expressions per field, reports missing mandatory fields, validates
code values against PID-specific allowed lists, and handles Unknown conditions
with info-level warnings. Supports Structure, Conditions, and Full levels.
```

---

## Task 4: Full integration tests

### Description
Create comprehensive integration tests that exercise the complete validation pipeline end-to-end with realistic AHB workflow definitions.

### Implementation

**`crates/automapper-validation/tests/validator_integration.rs`**:
```rust
//! Full integration tests for the EdifactValidator.

use automapper_validation::eval::{
    ConditionEvaluator, ConditionResult, EvaluationContext, ExternalConditionProvider,
    NoOpExternalProvider,
};
use automapper_validation::validator::{
    EdifactValidator, ErrorCodes, Severity, ValidationCategory, ValidationIssue, ValidationLevel,
    ValidationReport,
};
use automapper_validation::validator::validate::{AhbCodeRule, AhbFieldRule, AhbWorkflow};
use std::collections::HashMap;

// === Test helpers ===

struct ConfigurableEvaluator {
    results: HashMap<u32, ConditionResult>,
    external_ids: Vec<u32>,
}

impl ConfigurableEvaluator {
    fn new() -> Self {
        Self {
            results: HashMap::new(),
            external_ids: Vec::new(),
        }
    }

    fn condition(mut self, id: u32, result: ConditionResult) -> Self {
        self.results.insert(id, result);
        self
    }

    fn external(mut self, id: u32) -> Self {
        self.external_ids.push(id);
        self
    }
}

impl ConditionEvaluator for ConfigurableEvaluator {
    fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
        self.results
            .get(&condition)
            .copied()
            .unwrap_or(ConditionResult::Unknown)
    }

    fn is_external(&self, condition: u32) -> bool {
        self.external_ids.contains(&condition)
    }

    fn message_type(&self) -> &str {
        "UTILMD"
    }

    fn format_version(&self) -> &str {
        "FV2510"
    }
}

struct FixedExternalProvider {
    results: HashMap<String, ConditionResult>,
}

impl FixedExternalProvider {
    fn new(results: Vec<(&str, ConditionResult)>) -> Self {
        Self {
            results: results
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }
}

impl ExternalConditionProvider for FixedExternalProvider {
    fn evaluate(&self, name: &str) -> ConditionResult {
        self.results
            .get(name)
            .copied()
            .unwrap_or(ConditionResult::Unknown)
    }
}

fn make_workflow(fields: Vec<AhbFieldRule>) -> AhbWorkflow {
    AhbWorkflow {
        pruefidentifikator: "11001".to_string(),
        description: "Lieferbeginn".to_string(),
        communication_direction: Some("LF an NB".to_string()),
        fields,
    }
}

fn simple_field(path: &str, name: &str, status: &str) -> AhbFieldRule {
    AhbFieldRule {
        segment_path: path.to_string(),
        name: name.to_string(),
        ahb_status: status.to_string(),
        codes: vec![],
    }
}

fn field_with_codes(
    path: &str,
    name: &str,
    status: &str,
    codes: Vec<(&str, &str)>,
) -> AhbFieldRule {
    AhbFieldRule {
        segment_path: path.to_string(),
        name: name.to_string(),
        ahb_status: status.to_string(),
        codes: codes
            .into_iter()
            .map(|(value, ahb)| AhbCodeRule {
                value: value.to_string(),
                description: format!("Code {value}"),
                ahb_status: ahb.to_string(),
            })
            .collect(),
    }
}

// === Test cases ===

#[test]
fn test_validate_utilmd_lieferbeginn_all_fields_present() {
    // When all conditions are met and all fields are present, no errors
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::True)
        .condition(6, ConditionResult::True);

    let workflow = make_workflow(vec![
        simple_field("SG2/NAD/3035", "Partnerrolle", "Muss"),
        simple_field("SG2/NAD/C082/3039", "MP-ID", "Muss [182] ∧ [152]"),
        simple_field("SG4/STS/C556/9013", "Transaktionsgrund", "Muss"),
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    // Note: with empty input, parse_segments returns empty vec,
    // so all mandatory fields will be missing
    let report = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
        .unwrap();

    // All Muss fields are missing -> errors
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 3);
}

#[test]
fn test_validate_conditional_fields_not_required_when_false() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::False)
        .condition(152, ConditionResult::True);

    let workflow = make_workflow(vec![
        simple_field("SG2/NAD/C082/3039", "MP-ID", "Muss [182] ∧ [152]"),
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
        .unwrap();

    // [182]=F makes AND false -> field not required -> no error
    assert!(report.is_valid());
}

#[test]
fn test_validate_mixed_mandatory_and_conditional_fields() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::False); // Condition false

    let workflow = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss"), // Always required
        simple_field("DTM", "Datum", "Muss [182] ∧ [152]"), // Condition false
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
        .unwrap();

    // NAD is missing (error), DTM condition is false (no error)
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 1);
    let error = report.errors().next().unwrap();
    assert!(error.message.contains("Partnerrolle"));
}

#[test]
fn test_validate_unknown_conditions_produce_info() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .external(8); // 8 is external and not registered -> Unknown

    let workflow = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss [182] ∧ [8]"),
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
        .unwrap();

    // [182]=T, [8]=Unknown -> AND = Unknown -> info, not error
    assert!(report.is_valid());
    let infos: Vec<_> = report.infos().collect();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].code, ErrorCodes::CONDITION_UNKNOWN);
}

#[test]
fn test_validate_xor_expression() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(102, ConditionResult::True)
        .condition(2006, ConditionResult::True)
        .condition(103, ConditionResult::False)
        .condition(2005, ConditionResult::False);

    let workflow = make_workflow(vec![simple_field(
        "DTM",
        "Datum",
        "Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])",
    )]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
        .unwrap();

    // XOR(T,F) = T -> field required, DTM missing -> error
    assert!(!report.is_valid());
    assert_eq!(report.error_count(), 1);
}

#[test]
fn test_validate_structure_level_ignores_conditions() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(182, ConditionResult::True)
        .condition(152, ConditionResult::True);

    let workflow = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss [182] ∧ [152]"),
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator
        .validate(
            b"",
            ValidationLevel::Structure,
            &external,
            Some(&workflow),
        )
        .unwrap();

    // Structure level does not check AHB conditions
    assert_eq!(report.by_category(ValidationCategory::Ahb).count(), 0);
}

#[test]
fn test_validate_report_serialization() {
    let evaluator = ConfigurableEvaluator::new();
    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let workflow = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss"),
    ]);

    let report = validator
        .validate(b"", ValidationLevel::Full, &external, Some(&workflow))
        .unwrap();

    // Serialize to JSON and back
    let json = serde_json::to_string_pretty(&report).unwrap();
    let deserialized: ValidationReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.format_version.as_deref(), Some("FV2510"));
    assert_eq!(deserialized.level, ValidationLevel::Full);
    assert_eq!(deserialized.total_issues(), report.total_issues());
}

#[test]
fn test_validate_kann_field_not_mandatory() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(570, ConditionResult::True);

    let workflow = make_workflow(vec![
        simple_field("FTX", "Freitext", "Kann [570]"),
    ]);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    let report = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&workflow))
        .unwrap();

    // "Kann" is not mandatory even when condition is True
    assert!(report.is_valid());
}

#[test]
fn test_validate_multiple_workflows_same_validator() {
    let evaluator = ConfigurableEvaluator::new()
        .condition(1, ConditionResult::True);

    let validator = EdifactValidator::new(evaluator);
    let external = NoOpExternalProvider;

    // First workflow
    let wf1 = make_workflow(vec![
        simple_field("NAD", "Partnerrolle", "Muss"),
    ]);
    let report1 = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&wf1))
        .unwrap();
    assert_eq!(report1.error_count(), 1);

    // Second workflow with different fields
    let wf2 = make_workflow(vec![
        simple_field("DTM", "Datum", "Muss"),
        simple_field("BGM", "Nachrichtentyp", "Muss"),
    ]);
    let report2 = validator
        .validate(b"", ValidationLevel::Conditions, &external, Some(&wf2))
        .unwrap();
    assert_eq!(report2.error_count(), 2);
}
```

### Verification
```bash
cargo test -p automapper-validation --test validator_integration
cargo test -p automapper-validation
cargo clippy -p automapper-validation -- -D warnings
```

### Commit
```
test(automapper-validation): add full validator integration tests

End-to-end tests for EdifactValidator with realistic AHB workflows.
Covers mandatory fields, conditional fields, Unknown propagation, XOR
expressions, validation levels, Kann vs Muss handling, and report
serialization.
```

---

## Task 5: Re-export public API and final cleanup

### Description
Ensure the crate has a clean public API with appropriate re-exports in `lib.rs`. Run all tests and clippy to confirm everything compiles and passes.

### Implementation

Update **`crates/automapper-validation/src/lib.rs`**:
```rust
//! AHB condition expression parsing, evaluation, and EDIFACT message validation.
//!
//! This crate provides three layers of functionality:
//!
//! 1. **Expression parsing** ([`expr`]): Parses AHB status strings like
//!    `"Muss [182] ∧ [6] ∧ [570]"` into a [`ConditionExpr`] AST.
//!
//! 2. **Condition evaluation** ([`eval`]): Evaluates condition expressions
//!    using a [`ConditionEvaluator`] trait with three-valued logic
//!    (True/False/Unknown) for graceful handling of external conditions.
//!
//! 3. **Message validation** ([`validator`]): Validates EDIFACT messages
//!    against AHB rules, producing a [`ValidationReport`] with typed issues.
//!
//! # Quick Start
//!
//! ```ignore
//! use automapper_validation::expr::{ConditionParser, ConditionExpr};
//! use automapper_validation::eval::{ConditionExprEvaluator, ConditionResult};
//! use automapper_validation::validator::{EdifactValidator, ValidationLevel};
//!
//! // Parse a condition expression
//! let expr = ConditionParser::parse("Muss [182] ∧ [152]").unwrap();
//!
//! // Evaluate using a condition evaluator
//! let validator = EdifactValidator::new(my_evaluator);
//! let report = validator.validate(edifact_bytes, ValidationLevel::Full, &external, None)?;
//! ```

pub mod error;
pub mod eval;
pub mod expr;
pub mod validator;

// Re-export key types at crate root for convenience
pub use error::{ParseError, ValidationError};
pub use eval::{ConditionEvaluator, ConditionExprEvaluator, ConditionResult, EvaluationContext};
pub use expr::{ConditionExpr, ConditionParser};
pub use validator::{
    EdifactValidator, ErrorCodes, Severity, ValidationCategory, ValidationIssue, ValidationLevel,
    ValidationReport,
};
```

### Verification
```bash
cargo check -p automapper-validation
cargo test -p automapper-validation
cargo clippy -p automapper-validation -- -D warnings
cargo doc -p automapper-validation --no-deps
```

### Commit
```
feat(automapper-validation): finalize public API with crate-level re-exports

Clean public API re-exporting key types at crate root: ConditionExpr,
ConditionParser, ConditionEvaluator, ConditionResult, EdifactValidator,
ValidationReport, ValidationIssue, and all supporting types. Full
rustdoc with module-level documentation and usage examples.
```
