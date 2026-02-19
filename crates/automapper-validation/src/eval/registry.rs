//! Registry of condition evaluators keyed by (message_type, format_version).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::evaluator::ConditionEvaluator;

/// Global registry of condition evaluators.
///
/// Evaluators are registered at startup (typically from generated code)
/// and looked up at runtime based on the detected message type and format
/// version.
///
/// Thread-safe: uses `RwLock` for concurrent read access with exclusive
/// write access during registration.
pub struct EvaluatorRegistry {
    evaluators: RwLock<HashMap<(String, String), Arc<dyn ConditionEvaluator>>>,
}

impl EvaluatorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            evaluators: RwLock::new(HashMap::new()),
        }
    }

    /// Register a condition evaluator for a message type and format version.
    ///
    /// Overwrites any previously registered evaluator for the same key.
    pub fn register<E: ConditionEvaluator + 'static>(&self, evaluator: E) {
        let key = (
            evaluator.message_type().to_string(),
            evaluator.format_version().to_string(),
        );
        self.evaluators
            .write()
            .expect("registry lock poisoned")
            .insert(key, Arc::new(evaluator));
    }

    /// Register an already-Arc'd evaluator.
    pub fn register_arc(&self, evaluator: Arc<dyn ConditionEvaluator>) {
        let key = (
            evaluator.message_type().to_string(),
            evaluator.format_version().to_string(),
        );
        self.evaluators
            .write()
            .expect("registry lock poisoned")
            .insert(key, evaluator);
    }

    /// Look up an evaluator by message type and format version.
    ///
    /// Returns `None` if no evaluator is registered for the given key.
    pub fn get(
        &self,
        message_type: &str,
        format_version: &str,
    ) -> Option<Arc<dyn ConditionEvaluator>> {
        self.evaluators
            .read()
            .expect("registry lock poisoned")
            .get(&(message_type.to_string(), format_version.to_string()))
            .cloned()
    }

    /// Look up an evaluator, returning an error if not found.
    pub fn get_or_err(
        &self,
        message_type: &str,
        format_version: &str,
    ) -> Result<Arc<dyn ConditionEvaluator>, crate::error::ValidationError> {
        self.get(message_type, format_version).ok_or_else(|| {
            crate::error::ValidationError::NoEvaluator {
                message_type: message_type.to_string(),
                format_version: format_version.to_string(),
            }
        })
    }

    /// List all registered (message_type, format_version) keys.
    pub fn registered_keys(&self) -> Vec<(String, String)> {
        self.evaluators
            .read()
            .expect("registry lock poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Clear all registered evaluators. Primarily for testing.
    pub fn clear(&self) {
        self.evaluators
            .write()
            .expect("registry lock poisoned")
            .clear();
    }
}

impl Default for EvaluatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::context::EvaluationContext;
    use super::super::evaluator::ConditionResult;
    use super::*;

    /// Minimal evaluator for testing registry operations.
    struct TestEvaluator {
        msg_type: String,
        fmt_version: String,
    }

    impl TestEvaluator {
        fn new(msg_type: &str, fmt_version: &str) -> Self {
            Self {
                msg_type: msg_type.to_string(),
                fmt_version: fmt_version.to_string(),
            }
        }
    }

    impl ConditionEvaluator for TestEvaluator {
        fn evaluate(&self, _condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
            ConditionResult::Unknown
        }
        fn is_external(&self, _condition: u32) -> bool {
            false
        }
        fn message_type(&self) -> &str {
            &self.msg_type
        }
        fn format_version(&self) -> &str {
            &self.fmt_version
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));

        let eval = registry.get("UTILMD", "FV2510");
        assert!(eval.is_some());
        assert_eq!(eval.unwrap().message_type(), "UTILMD");
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let registry = EvaluatorRegistry::new();
        assert!(registry.get("UTILMD", "FV2510").is_none());
    }

    #[test]
    fn test_get_or_err_returns_error() {
        let registry = EvaluatorRegistry::new();
        let result = registry.get_or_err("UTILMD", "FV2510");
        assert!(result.is_err());
    }

    #[test]
    fn test_register_overwrites() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));

        let keys = registry.registered_keys();
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn test_multiple_registrations() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2504"));
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));
        registry.register(TestEvaluator::new("ORDERS", "FV2510"));

        let keys = registry.registered_keys();
        assert_eq!(keys.len(), 3);

        assert!(registry.get("UTILMD", "FV2504").is_some());
        assert!(registry.get("UTILMD", "FV2510").is_some());
        assert!(registry.get("ORDERS", "FV2510").is_some());
        assert!(registry.get("ORDERS", "FV2504").is_none());
    }

    #[test]
    fn test_clear() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));
        assert!(!registry.registered_keys().is_empty());

        registry.clear();
        assert!(registry.registered_keys().is_empty());
    }

    #[test]
    fn test_register_arc() {
        let registry = EvaluatorRegistry::new();
        let eval: Arc<dyn ConditionEvaluator> = Arc::new(TestEvaluator::new("UTILMD", "FV2510"));
        registry.register_arc(eval);

        assert!(registry.get("UTILMD", "FV2510").is_some());
    }
}
