use serde::{Deserialize, Serialize};

/// Result of migrating an EDIFACT fixture between format versions.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// The migrated EDIFACT content as a string.
    pub edifact: String,
    /// Warnings about items requiring manual review.
    pub warnings: Vec<MigrationWarning>,
    /// Summary statistics.
    pub stats: MigrationStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationWarning {
    pub severity: WarningSeverity,
    pub message: String,
    /// The segment tag this warning relates to, if applicable.
    pub segment: Option<String>,
    /// The group this warning relates to, if applicable.
    pub group: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    /// Automatic action taken, informational only.
    Info,
    /// Automatic action taken but may need verification.
    Warning,
    /// Could not be handled automatically — requires manual review.
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct MigrationStats {
    pub segments_copied: usize,
    pub segments_removed: usize,
    pub segments_added: usize,
    pub codes_substituted: usize,
    pub manual_review_items: usize,
}

impl std::fmt::Display for MigrationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.severity {
            WarningSeverity::Info => "INFO",
            WarningSeverity::Warning => "WARNING",
            WarningSeverity::Error => "ERROR",
        };
        write!(f, "{}: {}", prefix, self.message)
    }
}
