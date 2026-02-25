//! Evaluation context for condition evaluation.

use super::evaluator::ExternalConditionProvider;
use mig_types::segment::OwnedSegment;

/// Context passed to condition evaluators during evaluation.
///
/// Carries references to the transaction data and external condition
/// provider needed to evaluate AHB conditions.
pub struct EvaluationContext<'a> {
    /// The Pruefidentifikator (e.g., "11001", "55001") that identifies
    /// the specific AHB workflow being validated against.
    pub pruefidentifikator: &'a str,

    /// Provider for external conditions that depend on business context
    /// outside the EDIFACT message.
    pub external: &'a dyn ExternalConditionProvider,

    /// Parsed EDIFACT segments for direct segment inspection by condition
    /// evaluators. Conditions often need to check specific segment values.
    pub segments: &'a [OwnedSegment],
}

impl<'a> EvaluationContext<'a> {
    /// Create a new evaluation context.
    pub fn new(
        pruefidentifikator: &'a str,
        external: &'a dyn ExternalConditionProvider,
        segments: &'a [OwnedSegment],
    ) -> Self {
        Self {
            pruefidentifikator,
            external,
            segments,
        }
    }

    /// Find the first segment with the given ID.
    pub fn find_segment(&self, segment_id: &str) -> Option<&'a OwnedSegment> {
        self.segments.iter().find(|s| s.id == segment_id)
    }

    /// Find all segments with the given ID.
    pub fn find_segments(&self, segment_id: &str) -> Vec<&'a OwnedSegment> {
        self.segments
            .iter()
            .filter(|s| s.id == segment_id)
            .collect()
    }

    /// Find segments with a specific qualifier value on a given element.
    pub fn find_segments_with_qualifier(
        &self,
        segment_id: &str,
        element_index: usize,
        qualifier: &str,
    ) -> Vec<&'a OwnedSegment> {
        self.segments
            .iter()
            .filter(|s| {
                s.id == segment_id
                    && s.elements
                        .get(element_index)
                        .and_then(|e| e.first())
                        .is_some_and(|v| v == qualifier)
            })
            .collect()
    }

    /// Check if a segment with the given ID exists.
    pub fn has_segment(&self, segment_id: &str) -> bool {
        self.segments.iter().any(|s| s.id == segment_id)
    }
}

#[cfg(test)]
mod tests {
    use super::super::evaluator::NoOpExternalProvider;
    use super::*;

    fn make_segment(id: &str, elements: Vec<Vec<&str>>) -> OwnedSegment {
        OwnedSegment {
            id: id.to_string(),
            elements: elements
                .into_iter()
                .map(|e| e.into_iter().map(|c| c.to_string()).collect())
                .collect(),
            segment_number: 0,
        }
    }

    #[test]
    fn test_find_segment() {
        let segments = vec![
            make_segment("UNH", vec![vec!["test"]]),
            make_segment("NAD", vec![vec!["MS"], vec!["123456789", "", "293"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        assert!(ctx.find_segment("NAD").is_some());
        assert!(ctx.find_segment("DTM").is_none());
    }

    #[test]
    fn test_find_segments_with_qualifier() {
        let segments = vec![
            make_segment("NAD", vec![vec!["MS"], vec!["111"]]),
            make_segment("NAD", vec![vec!["MR"], vec!["222"]]),
            make_segment("NAD", vec![vec!["MS"], vec!["333"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        let ms_nads = ctx.find_segments_with_qualifier("NAD", 0, "MS");
        assert_eq!(ms_nads.len(), 2);
    }

    #[test]
    fn test_has_segment() {
        let segments = vec![make_segment("UNH", vec![vec!["test"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        assert!(ctx.has_segment("UNH"));
        assert!(!ctx.has_segment("NAD"));
    }
}
