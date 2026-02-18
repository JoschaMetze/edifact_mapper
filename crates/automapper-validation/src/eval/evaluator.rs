//! Core condition evaluation traits.

use super::context::EvaluationContext;

/// Three-valued result of evaluating a single condition.
///
/// Unlike the C# implementation which uses `bool`, we use three-valued logic
/// to support partial evaluation when external conditions are unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionResult {
    /// The condition is satisfied.
    True,
    /// The condition is not satisfied.
    False,
    /// The condition cannot be determined (e.g., external condition without a provider).
    Unknown,
}

impl ConditionResult {
    /// Returns `true` if this is `ConditionResult::True`.
    pub fn is_true(self) -> bool {
        matches!(self, ConditionResult::True)
    }

    /// Returns `true` if this is `ConditionResult::False`.
    pub fn is_false(self) -> bool {
        matches!(self, ConditionResult::False)
    }

    /// Returns `true` if this is `ConditionResult::Unknown`.
    pub fn is_unknown(self) -> bool {
        matches!(self, ConditionResult::Unknown)
    }

    /// Converts to `Option<bool>`: True -> Some(true), False -> Some(false), Unknown -> None.
    pub fn to_option(self) -> Option<bool> {
        match self {
            ConditionResult::True => Some(true),
            ConditionResult::False => Some(false),
            ConditionResult::Unknown => None,
        }
    }
}

impl From<bool> for ConditionResult {
    fn from(value: bool) -> Self {
        if value {
            ConditionResult::True
        } else {
            ConditionResult::False
        }
    }
}

impl std::fmt::Display for ConditionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionResult::True => write!(f, "True"),
            ConditionResult::False => write!(f, "False"),
            ConditionResult::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Evaluates individual AHB conditions by number.
///
/// Implementations are typically generated from AHB XML schemas (one per
/// message type and format version). Each condition number maps to a
/// specific business rule check.
pub trait ConditionEvaluator: Send + Sync {
    /// Evaluate a single condition by number.
    ///
    /// Returns `ConditionResult::Unknown` for unrecognized condition numbers
    /// or conditions that require unavailable external context.
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult;

    /// Returns `true` if the given condition requires external context
    /// (i.e., cannot be determined from the EDIFACT message alone).
    fn is_external(&self, condition: u32) -> bool;

    /// Returns the message type this evaluator handles (e.g., "UTILMD").
    fn message_type(&self) -> &str;

    /// Returns the format version this evaluator handles (e.g., "FV2510").
    fn format_version(&self) -> &str;
}

/// Provider for external conditions that depend on context outside the EDIFACT message.
///
/// External conditions are things like:
/// - [1] "Wenn Aufteilung vorhanden" (message splitting status)
/// - [14] "Wenn Datum bekannt" (whether a date is known)
/// - [30] "Wenn Antwort auf Aktivierung" (response to activation)
///
/// These cannot be determined from the EDIFACT content alone and require
/// business context from the calling system.
pub trait ExternalConditionProvider: Send + Sync {
    /// Evaluate an external condition by name.
    ///
    /// The `condition_name` corresponds to the speaking name from the
    /// generated external conditions constants (e.g., "MessageSplitting",
    /// "DateKnown").
    fn evaluate(&self, condition_name: &str) -> ConditionResult;
}

/// A no-op external condition provider that returns `Unknown` for everything.
///
/// Useful when no external context is available — conditions will propagate
/// as `Unknown` through the expression evaluator.
pub struct NoOpExternalProvider;

impl ExternalConditionProvider for NoOpExternalProvider {
    fn evaluate(&self, _condition_name: &str) -> ConditionResult {
        ConditionResult::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_result_is_methods() {
        assert!(ConditionResult::True.is_true());
        assert!(!ConditionResult::True.is_false());
        assert!(!ConditionResult::True.is_unknown());

        assert!(!ConditionResult::False.is_true());
        assert!(ConditionResult::False.is_false());

        assert!(ConditionResult::Unknown.is_unknown());
    }

    #[test]
    fn test_condition_result_to_option() {
        assert_eq!(ConditionResult::True.to_option(), Some(true));
        assert_eq!(ConditionResult::False.to_option(), Some(false));
        assert_eq!(ConditionResult::Unknown.to_option(), None);
    }

    #[test]
    fn test_condition_result_from_bool() {
        assert_eq!(ConditionResult::from(true), ConditionResult::True);
        assert_eq!(ConditionResult::from(false), ConditionResult::False);
    }

    #[test]
    fn test_condition_result_display() {
        assert_eq!(format!("{}", ConditionResult::True), "True");
        assert_eq!(format!("{}", ConditionResult::False), "False");
        assert_eq!(format!("{}", ConditionResult::Unknown), "Unknown");
    }

    #[test]
    fn test_noop_external_provider() {
        let provider = NoOpExternalProvider;
        assert_eq!(
            provider.evaluate("MessageSplitting"),
            ConditionResult::Unknown
        );
        assert_eq!(provider.evaluate("anything"), ConditionResult::Unknown);
    }
}
