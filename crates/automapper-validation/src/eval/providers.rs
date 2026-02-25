//! Concrete [`ExternalConditionProvider`] implementations.
//!
//! - [`MapExternalProvider`]: wraps a `HashMap<String, bool>` for simple lookup.
//! - [`CompositeExternalProvider`]: chains multiple providers, returning the first
//!   non-[`Unknown`](super::ConditionResult::Unknown) result.

use std::collections::HashMap;

use super::evaluator::{ConditionResult, ExternalConditionProvider};

/// An [`ExternalConditionProvider`] backed by a `HashMap<String, bool>`.
///
/// Returns `True`/`False` for keys present in the map and `Unknown` for
/// missing keys. This is the simplest way for API callers to supply
/// external condition values.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use automapper_validation::{MapExternalProvider, ConditionResult};
/// use automapper_validation::eval::ExternalConditionProvider;
///
/// let mut conditions = HashMap::new();
/// conditions.insert("DateKnown".to_string(), true);
///
/// let provider = MapExternalProvider::new(conditions);
/// assert_eq!(provider.evaluate("DateKnown"), ConditionResult::True);
/// assert_eq!(provider.evaluate("Unknown"), ConditionResult::Unknown);
/// ```
pub struct MapExternalProvider {
    conditions: HashMap<String, bool>,
}

impl MapExternalProvider {
    /// Creates a new `MapExternalProvider` from the given condition map.
    pub fn new(conditions: HashMap<String, bool>) -> Self {
        Self { conditions }
    }
}

impl ExternalConditionProvider for MapExternalProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        match self.conditions.get(condition_name) {
            Some(true) => ConditionResult::True,
            Some(false) => ConditionResult::False,
            None => ConditionResult::Unknown,
        }
    }
}

/// An [`ExternalConditionProvider`] that delegates to multiple providers in order.
///
/// For each `evaluate()` call, providers are consulted in sequence. The first
/// provider that returns a non-[`Unknown`](ConditionResult::Unknown) result wins.
/// If all providers return `Unknown` (or there are no providers), `Unknown` is
/// returned.
///
/// This is useful for layering: e.g., a caller-supplied map on top of a
/// system-default provider.
pub struct CompositeExternalProvider {
    providers: Vec<Box<dyn ExternalConditionProvider>>,
}

impl CompositeExternalProvider {
    /// Creates a new `CompositeExternalProvider` from the given provider list.
    ///
    /// Providers are consulted in the order they appear in the vector.
    pub fn new(providers: Vec<Box<dyn ExternalConditionProvider>>) -> Self {
        Self { providers }
    }
}

impl ExternalConditionProvider for CompositeExternalProvider {
    fn evaluate(&self, condition_name: &str) -> ConditionResult {
        for provider in &self.providers {
            let result = provider.evaluate(condition_name);
            if !result.is_unknown() {
                return result;
            }
        }
        ConditionResult::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- MapExternalProvider tests ----

    #[test]
    fn map_provider_returns_true_for_true_entry() {
        let mut conditions = HashMap::new();
        conditions.insert("DateKnown".to_string(), true);
        let provider = MapExternalProvider::new(conditions);

        assert_eq!(provider.evaluate("DateKnown"), ConditionResult::True);
    }

    #[test]
    fn map_provider_returns_false_for_false_entry() {
        let mut conditions = HashMap::new();
        conditions.insert("MessageSplitting".to_string(), false);
        let provider = MapExternalProvider::new(conditions);

        assert_eq!(
            provider.evaluate("MessageSplitting"),
            ConditionResult::False
        );
    }

    #[test]
    fn map_provider_returns_unknown_for_missing_key() {
        let mut conditions = HashMap::new();
        conditions.insert("DateKnown".to_string(), true);
        let provider = MapExternalProvider::new(conditions);

        assert_eq!(provider.evaluate("NonExistent"), ConditionResult::Unknown);
    }

    #[test]
    fn map_provider_empty_map_returns_unknown() {
        let provider = MapExternalProvider::new(HashMap::new());

        assert_eq!(provider.evaluate("Anything"), ConditionResult::Unknown);
    }

    // ---- CompositeExternalProvider tests ----

    #[test]
    fn composite_first_known_wins() {
        // Provider 1 knows "A" = true, but not "B"
        let mut p1_map = HashMap::new();
        p1_map.insert("A".to_string(), true);
        let p1 = MapExternalProvider::new(p1_map);

        // Provider 2 knows "B" = false, but not "A"
        let mut p2_map = HashMap::new();
        p2_map.insert("B".to_string(), false);
        let p2 = MapExternalProvider::new(p2_map);

        let composite = CompositeExternalProvider::new(vec![Box::new(p1), Box::new(p2)]);

        // "A" resolved by p1
        assert_eq!(composite.evaluate("A"), ConditionResult::True);
        // "B" not in p1 (Unknown), resolved by p2
        assert_eq!(composite.evaluate("B"), ConditionResult::False);
    }

    #[test]
    fn composite_all_unknown_returns_unknown() {
        // Two providers, neither knows "X"
        let p1 = MapExternalProvider::new(HashMap::new());
        let p2 = MapExternalProvider::new(HashMap::new());

        let composite = CompositeExternalProvider::new(vec![Box::new(p1), Box::new(p2)]);

        assert_eq!(composite.evaluate("X"), ConditionResult::Unknown);
    }

    #[test]
    fn composite_empty_returns_unknown() {
        let composite = CompositeExternalProvider::new(vec![]);

        assert_eq!(composite.evaluate("Anything"), ConditionResult::Unknown);
    }
}
