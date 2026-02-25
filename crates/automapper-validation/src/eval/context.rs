//! Evaluation context for condition evaluation.

use super::evaluator::ExternalConditionProvider;
use mig_types::navigator::GroupNavigator;
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

    /// Optional group navigator for group-scoped condition queries.
    /// When None, group-scoped methods return empty / false / 0.
    pub navigator: Option<&'a dyn GroupNavigator>,
}

/// A no-op group navigator that returns empty results for all queries.
pub struct NoOpGroupNavigator;

impl GroupNavigator for NoOpGroupNavigator {
    fn find_segments_in_group(
        &self,
        _: &str,
        _: &[&str],
        _: usize,
    ) -> Vec<OwnedSegment> {
        Vec::new()
    }
    fn find_segments_with_qualifier_in_group(
        &self,
        _: &str,
        _: usize,
        _: &str,
        _: &[&str],
        _: usize,
    ) -> Vec<OwnedSegment> {
        Vec::new()
    }
    fn group_instance_count(&self, _: &[&str]) -> usize {
        0
    }
}

impl<'a> EvaluationContext<'a> {
    /// Create a new evaluation context (without group navigator).
    pub fn new(
        pruefidentifikator: &'a str,
        external: &'a dyn ExternalConditionProvider,
        segments: &'a [OwnedSegment],
    ) -> Self {
        Self {
            pruefidentifikator,
            external,
            segments,
            navigator: None,
        }
    }

    /// Create a new evaluation context with a group navigator.
    pub fn with_navigator(
        pruefidentifikator: &'a str,
        external: &'a dyn ExternalConditionProvider,
        segments: &'a [OwnedSegment],
        navigator: &'a dyn GroupNavigator,
    ) -> Self {
        Self {
            pruefidentifikator,
            external,
            segments,
            navigator: Some(navigator),
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

    /// Find all segments with the given tag within a specific group instance.
    /// Returns empty if no navigator is set.
    pub fn find_segments_in_group(
        &self,
        segment_id: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> Vec<OwnedSegment> {
        match self.navigator {
            Some(nav) => nav.find_segments_in_group(segment_id, group_path, instance_index),
            None => Vec::new(),
        }
    }

    /// Find segments matching a tag + qualifier within a group instance.
    /// Returns empty if no navigator is set.
    pub fn find_segments_with_qualifier_in_group(
        &self,
        segment_id: &str,
        element_index: usize,
        qualifier: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> Vec<OwnedSegment> {
        match self.navigator {
            Some(nav) => nav.find_segments_with_qualifier_in_group(
                segment_id,
                element_index,
                qualifier,
                group_path,
                instance_index,
            ),
            None => Vec::new(),
        }
    }

    /// Check if a segment exists in a specific group instance.
    /// Returns false if no navigator is set.
    pub fn has_segment_in_group(
        &self,
        segment_id: &str,
        group_path: &[&str],
        instance_index: usize,
    ) -> bool {
        !self.find_segments_in_group(segment_id, group_path, instance_index)
            .is_empty()
    }

    /// Count repetitions of a group at the given path.
    /// Returns 0 if no navigator is set.
    pub fn group_instance_count(&self, group_path: &[&str]) -> usize {
        match self.navigator {
            Some(nav) => nav.group_instance_count(group_path),
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::evaluator::NoOpExternalProvider;
    use super::*;
    use mig_types::navigator::GroupNavigator;

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

    // --- Mock navigator for testing ---
    struct MockGroupNavigator {
        groups: Vec<(Vec<String>, usize, Vec<OwnedSegment>)>,
    }

    impl MockGroupNavigator {
        fn new() -> Self {
            Self { groups: vec![] }
        }
        fn with_group(
            mut self,
            path: &[&str],
            instance: usize,
            segs: Vec<OwnedSegment>,
        ) -> Self {
            self.groups.push((
                path.iter().map(|s| s.to_string()).collect(),
                instance,
                segs,
            ));
            self
        }
        fn find_instance(&self, group_path: &[&str], idx: usize) -> Option<&[OwnedSegment]> {
            self.groups
                .iter()
                .find(|(p, i, _)| {
                    let ps: Vec<&str> = p.iter().map(|s| s.as_str()).collect();
                    ps.as_slice() == group_path && *i == idx
                })
                .map(|(_, _, segs)| segs.as_slice())
        }
    }

    impl GroupNavigator for MockGroupNavigator {
        fn find_segments_in_group(
            &self,
            segment_id: &str,
            group_path: &[&str],
            instance_index: usize,
        ) -> Vec<OwnedSegment> {
            self.find_instance(group_path, instance_index)
                .map(|segs| {
                    segs.iter()
                        .filter(|s| s.id == segment_id)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        }
        fn find_segments_with_qualifier_in_group(
            &self,
            segment_id: &str,
            element_index: usize,
            qualifier: &str,
            group_path: &[&str],
            instance_index: usize,
        ) -> Vec<OwnedSegment> {
            self.find_segments_in_group(segment_id, group_path, instance_index)
                .into_iter()
                .filter(|s| {
                    s.elements
                        .get(element_index)
                        .and_then(|e| e.first())
                        .is_some_and(|v| v == qualifier)
                })
                .collect()
        }
        fn group_instance_count(&self, group_path: &[&str]) -> usize {
            self.groups
                .iter()
                .filter(|(p, _, _)| {
                    let ps: Vec<&str> = p.iter().map(|s| s.as_str()).collect();
                    ps.as_slice() == group_path
                })
                .count()
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

    // --- Group navigator tests ---

    #[test]
    fn test_no_navigator_group_find_returns_empty() {
        let segments = vec![make_segment("SEQ", vec![vec!["Z98"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);
        assert!(ctx
            .find_segments_in_group("SEQ", &["SG4", "SG8"], 0)
            .is_empty());
    }

    #[test]
    fn test_no_navigator_group_instance_count_zero() {
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &[]);
        assert_eq!(ctx.group_instance_count(&["SG4"]), 0);
    }

    #[test]
    fn test_with_navigator_finds_segments_in_group() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new().with_group(
            &["SG4", "SG8"],
            0,
            vec![
                make_segment("SEQ", vec![vec!["Z98"]]),
                make_segment("CCI", vec![vec!["Z30"], vec![], vec!["Z07"]]),
            ],
        );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);
        let result = ctx.find_segments_in_group("SEQ", &["SG4", "SG8"], 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "SEQ");
    }

    #[test]
    fn test_with_navigator_qualifier_in_group() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new().with_group(
            &["SG4", "SG8"],
            0,
            vec![
                make_segment("SEQ", vec![vec!["Z98"]]),
                make_segment("SEQ", vec![vec!["Z01"]]),
            ],
        );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);
        let result =
            ctx.find_segments_with_qualifier_in_group("SEQ", 0, "Z98", &["SG4", "SG8"], 0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_group_instance_count_with_navigator() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![make_segment("SEQ", vec![vec!["Z98"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                1,
                vec![make_segment("SEQ", vec![vec!["Z01"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);
        assert_eq!(ctx.group_instance_count(&["SG4", "SG8"]), 2);
    }

    #[test]
    fn test_has_segment_in_group() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new().with_group(
            &["SG4", "SG8"],
            0,
            vec![make_segment("SEQ", vec![vec!["Z98"]])],
        );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);
        assert!(ctx.has_segment_in_group("SEQ", &["SG4", "SG8"], 0));
        assert!(!ctx.has_segment_in_group("CCI", &["SG4", "SG8"], 0));
        assert!(!ctx.has_segment_in_group("SEQ", &["SG4", "SG5"], 0));
    }
}
