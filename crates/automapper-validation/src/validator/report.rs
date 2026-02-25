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
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count()
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
    pub fn by_category(
        &self,
        category: ValidationCategory,
    ) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(move |i| i.category == category)
    }

    /// Returns the total number of issues.
    pub fn total_issues(&self) -> usize {
        self.issues.len()
    }

    /// Enrich all issues that have a `field_path` by resolving BO4E paths.
    ///
    /// The `resolver` closure maps an EDIFACT field path (e.g., "SG4/SG5/LOC/C517/3225")
    /// to a BO4E field path (e.g., "stammdaten.Marktlokation.marktlokationsId").
    /// Issues without a `field_path` or where the resolver returns `None` are left unchanged.
    pub fn enrich_bo4e_paths(&mut self, resolver: impl Fn(&str) -> Option<String>) {
        for issue in &mut self.issues {
            if let Some(ref edifact_path) = issue.field_path {
                issue.bo4e_path = resolver(edifact_path);
            }
        }
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
    fn test_enrich_bo4e_paths() {
        let mut report = ValidationReport::new("UTILMD", ValidationLevel::Full);
        report.add_issue(make_error("AHB001").with_field_path("SG4/SG5/LOC/C517/3225"));
        report.add_issue(make_warning("STR001").with_field_path("SG2/NAD/3035"));
        // Issue without field_path should be left alone
        report.add_issue(make_error("AHB002"));

        report.enrich_bo4e_paths(|path| match path {
            "SG4/SG5/LOC/C517/3225" => Some("stammdaten.Marktlokation.marktlokationsId".into()),
            "SG2/NAD/3035" => Some("stammdaten.Marktteilnehmer".into()),
            _ => None,
        });

        assert_eq!(
            report.issues[0].bo4e_path.as_deref(),
            Some("stammdaten.Marktlokation.marktlokationsId")
        );
        assert_eq!(
            report.issues[1].bo4e_path.as_deref(),
            Some("stammdaten.Marktteilnehmer")
        );
        // No field_path → no bo4e_path
        assert!(report.issues[2].bo4e_path.is_none());
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
