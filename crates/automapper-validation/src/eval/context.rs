//! Evaluation context for condition evaluation.

use super::evaluator::{ConditionResult, ExternalConditionProvider};
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
    fn find_segments_in_group(&self, _: &str, _: &[&str], _: usize) -> Vec<OwnedSegment> {
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

    /// Get the group navigator, if one is set.
    pub fn navigator(&self) -> Option<&'a dyn GroupNavigator> {
        self.navigator
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
        !self
            .find_segments_in_group(segment_id, group_path, instance_index)
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

    /// Count child group repetitions within a specific parent group instance.
    /// Returns 0 if no navigator is set.
    pub fn child_group_instance_count(
        &self,
        parent_path: &[&str],
        parent_instance: usize,
        child_group_id: &str,
    ) -> usize {
        match self.navigator {
            Some(nav) => {
                nav.child_group_instance_count(parent_path, parent_instance, child_group_id)
            }
            None => 0,
        }
    }

    /// Find segments in a child group within a specific parent group instance.
    /// Returns empty if no navigator is set.
    pub fn find_segments_in_child_group(
        &self,
        segment_id: &str,
        parent_path: &[&str],
        parent_instance: usize,
        child_group_id: &str,
        child_instance: usize,
    ) -> Vec<OwnedSegment> {
        match self.navigator {
            Some(nav) => nav.find_segments_in_child_group(
                segment_id,
                parent_path,
                parent_instance,
                child_group_id,
                child_instance,
            ),
            None => Vec::new(),
        }
    }

    /// Extract a single value from the first matching segment in a group instance.
    /// Returns None if no navigator is set or value not found.
    pub fn extract_value_in_group(
        &self,
        segment_id: &str,
        element_index: usize,
        component_index: usize,
        group_path: &[&str],
        instance_index: usize,
    ) -> Option<String> {
        self.navigator?.extract_value_in_group(
            segment_id,
            element_index,
            component_index,
            group_path,
            instance_index,
        )
    }

    // --- High-level condition helpers ---
    // These reduce generated condition evaluator boilerplate by ~50%.

    /// Check if any segment with the given tag + qualifier exists (message-wide).
    /// Returns `True` if found, `False` if not.
    pub fn has_qualifier(
        &self,
        tag: &str,
        element_index: usize,
        qualifier: &str,
    ) -> ConditionResult {
        ConditionResult::from(
            !self
                .find_segments_with_qualifier(tag, element_index, qualifier)
                .is_empty(),
        )
    }

    /// Check if a segment with given tag + qualifier does NOT exist (message-wide).
    /// Returns `True` if absent, `False` if present.
    pub fn lacks_qualifier(
        &self,
        tag: &str,
        element_index: usize,
        qualifier: &str,
    ) -> ConditionResult {
        ConditionResult::from(
            self.find_segments_with_qualifier(tag, element_index, qualifier)
                .is_empty(),
        )
    }

    /// Check if any segment with the given tag + qualifier has a specific sub-element value.
    ///
    /// Finds segments matching `tag` with `elements[qual_elem][0] == qualifier`,
    /// then checks if `elements[value_elem][value_comp]` matches any of `values`.
    pub fn has_qualified_value(
        &self,
        tag: &str,
        qual_elem: usize,
        qualifier: &str,
        value_elem: usize,
        value_comp: usize,
        values: &[&str],
    ) -> ConditionResult {
        let segments = self.find_segments_with_qualifier(tag, qual_elem, qualifier);
        if segments.is_empty() {
            return ConditionResult::Unknown;
        }
        for seg in &segments {
            if let Some(v) = seg
                .elements
                .get(value_elem)
                .and_then(|e| e.get(value_comp))
                .map(|s| s.as_str())
            {
                if values.contains(&v) {
                    return ConditionResult::True;
                }
            }
        }
        ConditionResult::False
    }

    /// Group-scoped qualifier existence check with message-wide fallback.
    ///
    /// Checks if any group instance at `group_path` contains a segment matching
    /// `tag` with `elements[element_index][0] == qualifier`. Falls back to
    /// message-wide search if no group navigator is available.
    pub fn any_group_has_qualifier(
        &self,
        tag: &str,
        element_index: usize,
        qualifier: &str,
        group_path: &[&str],
    ) -> ConditionResult {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            for i in 0..instance_count {
                if !self
                    .find_segments_with_qualifier_in_group(
                        tag,
                        element_index,
                        qualifier,
                        group_path,
                        i,
                    )
                    .is_empty()
                {
                    return ConditionResult::True;
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide search
        self.has_qualifier(tag, element_index, qualifier)
    }

    /// Group-scoped segment existence check (any tag match, no qualifier).
    ///
    /// Checks if any group instance at `group_path` contains a segment with
    /// `elements[element_index][0]` matching any of `qualifiers`. Falls back
    /// to message-wide search if no group navigator is available.
    pub fn any_group_has_any_qualifier(
        &self,
        tag: &str,
        element_index: usize,
        qualifiers: &[&str],
        group_path: &[&str],
    ) -> ConditionResult {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            for i in 0..instance_count {
                let segs = self.find_segments_in_group(tag, group_path, i);
                if segs.iter().any(|seg| {
                    seg.elements
                        .get(element_index)
                        .and_then(|e| e.first())
                        .map(|s| s.as_str())
                        .is_some_and(|v| qualifiers.contains(&v))
                }) {
                    return ConditionResult::True;
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide search
        let found = self.find_segments(tag).iter().any(|seg| {
            seg.elements
                .get(element_index)
                .and_then(|e| e.first())
                .map(|s| s.as_str())
                .is_some_and(|v| qualifiers.contains(&v))
        });
        ConditionResult::from(found)
    }

    /// Group-scoped check for sub-element value within qualified segments.
    ///
    /// For each group instance at `group_path`, checks if a segment matching
    /// `tag` with `elements[qual_elem][0] == qualifier` has
    /// `elements[value_elem][value_comp]` in `values`. Falls back to message-wide.
    pub fn any_group_has_qualified_value(
        &self,
        tag: &str,
        qual_elem: usize,
        qualifier: &str,
        value_elem: usize,
        value_comp: usize,
        values: &[&str],
        group_path: &[&str],
    ) -> ConditionResult {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            for i in 0..instance_count {
                let segs = self.find_segments_with_qualifier_in_group(
                    tag, qual_elem, qualifier, group_path, i,
                );
                for seg in &segs {
                    if seg
                        .elements
                        .get(value_elem)
                        .and_then(|e| e.get(value_comp))
                        .map(|s| s.as_str())
                        .is_some_and(|v| values.contains(&v))
                    {
                        return ConditionResult::True;
                    }
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide search
        self.has_qualified_value(tag, qual_elem, qualifier, value_elem, value_comp, values)
    }

    // --- Parent-child group navigation helpers ---

    /// Pattern A: Check if parent group instances matching a qualifier have a child
    /// group containing a specific qualifier.
    ///
    /// Example: "In the SG8 with SEQ+Z98, does its SG10 child have CCI+Z23?"
    ///
    /// Falls back to message-wide search if no navigator is available.
    #[allow(clippy::too_many_arguments)]
    pub fn filtered_parent_child_has_qualifier(
        &self,
        parent_path: &[&str],
        parent_tag: &str,
        parent_elem: usize,
        parent_qual: &str,
        child_group_id: &str,
        child_tag: &str,
        child_elem: usize,
        child_qual: &str,
    ) -> ConditionResult {
        let parent_count = self.group_instance_count(parent_path);
        if parent_count > 0 {
            for pi in 0..parent_count {
                // Check if this parent instance has the required qualifier
                let parent_segs = self.find_segments_with_qualifier_in_group(
                    parent_tag,
                    parent_elem,
                    parent_qual,
                    parent_path,
                    pi,
                );
                if parent_segs.is_empty() {
                    continue;
                }
                // Check child group instances for the child qualifier
                let child_count = self.child_group_instance_count(parent_path, pi, child_group_id);
                for ci in 0..child_count {
                    let child_segs = self.find_segments_in_child_group(
                        child_tag,
                        parent_path,
                        pi,
                        child_group_id,
                        ci,
                    );
                    if child_segs.iter().any(|s| {
                        s.elements
                            .get(child_elem)
                            .and_then(|e| e.first())
                            .is_some_and(|v| v == child_qual)
                    }) {
                        return ConditionResult::True;
                    }
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide — check both qualifiers exist independently
        let has_parent = !self
            .find_segments_with_qualifier(parent_tag, parent_elem, parent_qual)
            .is_empty();
        let has_child = !self
            .find_segments_with_qualifier(child_tag, child_elem, child_qual)
            .is_empty();
        ConditionResult::from(has_parent && has_child)
    }

    /// Pattern B: Check if any group instance has one qualifier present but another absent.
    ///
    /// Example: "In any SG8, SEQ+Z59 is present but CCI+11 is absent"
    ///
    /// Falls back to message-wide search if no navigator is available.
    #[allow(clippy::too_many_arguments)]
    pub fn any_group_has_qualifier_without(
        &self,
        present_tag: &str,
        present_elem: usize,
        present_qual: &str,
        absent_tag: &str,
        absent_elem: usize,
        absent_qual: &str,
        group_path: &[&str],
    ) -> ConditionResult {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            for i in 0..instance_count {
                let has_present = !self
                    .find_segments_with_qualifier_in_group(
                        present_tag,
                        present_elem,
                        present_qual,
                        group_path,
                        i,
                    )
                    .is_empty();
                let has_absent = !self
                    .find_segments_with_qualifier_in_group(
                        absent_tag,
                        absent_elem,
                        absent_qual,
                        group_path,
                        i,
                    )
                    .is_empty();
                if has_present && !has_absent {
                    return ConditionResult::True;
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide
        let has_present = !self
            .find_segments_with_qualifier(present_tag, present_elem, present_qual)
            .is_empty();
        let has_absent = !self
            .find_segments_with_qualifier(absent_tag, absent_elem, absent_qual)
            .is_empty();
        ConditionResult::from(has_present && !has_absent)
    }

    /// Pattern C helper: Collect all values at a specific element+component across group instances.
    ///
    /// Returns `(instance_index, value)` pairs for non-empty values.
    pub fn collect_group_values(
        &self,
        tag: &str,
        elem: usize,
        comp: usize,
        group_path: &[&str],
    ) -> Vec<(usize, String)> {
        let instance_count = self.group_instance_count(group_path);
        let mut results = Vec::new();
        for i in 0..instance_count {
            if let Some(val) = self
                .navigator
                .and_then(|nav| nav.extract_value_in_group(tag, elem, comp, group_path, i))
            {
                if !val.is_empty() {
                    results.push((i, val));
                }
            }
        }
        results
    }

    /// Pattern C: Check if a value from one group path matches a value in another group path.
    ///
    /// Example: "Zeitraum-ID in SG6 RFF+Z49 matches reference in SG8 SEQ.c286"
    ///
    /// Finds qualified segments in source_path, extracts their value, then checks if
    /// any instance in target_path has the same value at the target position.
    ///
    /// Falls back to message-wide search if no navigator is available.
    #[allow(clippy::too_many_arguments)]
    pub fn groups_share_qualified_value(
        &self,
        source_tag: &str,
        source_qual_elem: usize,
        source_qual: &str,
        source_value_elem: usize,
        source_value_comp: usize,
        source_path: &[&str],
        target_tag: &str,
        target_elem: usize,
        target_comp: usize,
        target_path: &[&str],
    ) -> ConditionResult {
        let source_count = self.group_instance_count(source_path);
        let target_count = self.group_instance_count(target_path);
        if source_count > 0 && target_count > 0 {
            // Collect source values from qualified segments
            let mut source_values = Vec::new();
            for si in 0..source_count {
                let segs = self.find_segments_with_qualifier_in_group(
                    source_tag,
                    source_qual_elem,
                    source_qual,
                    source_path,
                    si,
                );
                for seg in &segs {
                    if let Some(val) = seg
                        .elements
                        .get(source_value_elem)
                        .and_then(|e| e.get(source_value_comp))
                    {
                        if !val.is_empty() {
                            source_values.push(val.clone());
                        }
                    }
                }
            }
            if source_values.is_empty() {
                return ConditionResult::Unknown;
            }
            // Check if any target instance has a matching value
            let target_values =
                self.collect_group_values(target_tag, target_elem, target_comp, target_path);
            for (_, tv) in &target_values {
                if source_values.iter().any(|sv| sv == tv) {
                    return ConditionResult::True;
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide search
        let source_segs =
            self.find_segments_with_qualifier(source_tag, source_qual_elem, source_qual);
        let source_vals: Vec<&str> = source_segs
            .iter()
            .filter_map(|s| {
                s.elements
                    .get(source_value_elem)
                    .and_then(|e| e.get(source_value_comp))
                    .map(|v| v.as_str())
            })
            .filter(|v| !v.is_empty())
            .collect();
        if source_vals.is_empty() {
            return ConditionResult::Unknown;
        }
        let target_segs = self.find_segments(target_tag);
        let has_match = target_segs.iter().any(|s| {
            s.elements
                .get(target_elem)
                .and_then(|e| e.get(target_comp))
                .map(|v| v.as_str())
                .is_some_and(|v| source_vals.contains(&v))
        });
        ConditionResult::from(has_match)
    }

    /// Group-scoped co-occurrence check: two segment conditions must both be true
    /// in the same group instance.
    ///
    /// For each group instance, checks that:
    /// 1. A segment with `tag_a` has `elements[elem_a][0]` in `quals_a`
    /// 2. A segment with `tag_b` has `elements[elem_b][comp_b]` in `vals_b`
    ///
    /// Falls back to message-wide search.
    #[allow(clippy::too_many_arguments)]
    pub fn any_group_has_co_occurrence(
        &self,
        tag_a: &str,
        elem_a: usize,
        quals_a: &[&str],
        tag_b: &str,
        elem_b: usize,
        comp_b: usize,
        vals_b: &[&str],
        group_path: &[&str],
    ) -> ConditionResult {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            for i in 0..instance_count {
                let a_present = self
                    .find_segments_in_group(tag_a, group_path, i)
                    .iter()
                    .any(|seg| {
                        seg.elements
                            .get(elem_a)
                            .and_then(|e| e.first())
                            .map(|s| s.as_str())
                            .is_some_and(|v| quals_a.contains(&v))
                    });
                let b_present = self
                    .find_segments_in_group(tag_b, group_path, i)
                    .iter()
                    .any(|seg| {
                        seg.elements
                            .get(elem_b)
                            .and_then(|e| e.get(comp_b))
                            .map(|s| s.as_str())
                            .is_some_and(|v| vals_b.contains(&v))
                    });
                if a_present && b_present {
                    return ConditionResult::True;
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide
        let a_found = self.find_segments(tag_a).iter().any(|seg| {
            seg.elements
                .get(elem_a)
                .and_then(|e| e.first())
                .map(|s| s.as_str())
                .is_some_and(|v| quals_a.contains(&v))
        });
        if !a_found {
            return ConditionResult::False;
        }
        let b_found = self.find_segments(tag_b).iter().any(|seg| {
            seg.elements
                .get(elem_b)
                .and_then(|e| e.get(comp_b))
                .map(|s| s.as_str())
                .is_some_and(|v| vals_b.contains(&v))
        });
        ConditionResult::from(b_found)
    }

    // --- Multi-element matching helpers ---

    /// Check if any segment with the given tag has ALL specified element/component values.
    ///
    /// Each check is `(element_index, component_index, expected_value)`.
    /// Returns `True` if a matching segment exists, `False` if segments exist but none match,
    /// `Unknown` if no segments with the tag exist.
    ///
    /// Example: `ctx.has_segment_matching("STS", &[(0, 0, "Z20"), (1, 0, "Z32"), (2, 0, "A99")])`
    pub fn has_segment_matching(
        &self,
        tag: &str,
        checks: &[(usize, usize, &str)],
    ) -> ConditionResult {
        let segments = self.find_segments(tag);
        if segments.is_empty() {
            return ConditionResult::Unknown;
        }
        let found = segments.iter().any(|seg| {
            checks.iter().all(|(elem, comp, val)| {
                seg.elements
                    .get(*elem)
                    .and_then(|e| e.get(*comp))
                    .is_some_and(|v| v == val)
            })
        });
        ConditionResult::from(found)
    }

    /// Group-scoped multi-element match with message-wide fallback.
    ///
    /// Checks if any group instance at `group_path` contains a segment with `tag`
    /// where ALL element/component checks match.
    pub fn has_segment_matching_in_group(
        &self,
        tag: &str,
        checks: &[(usize, usize, &str)],
        group_path: &[&str],
    ) -> ConditionResult {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            for i in 0..instance_count {
                let segs = self.find_segments_in_group(tag, group_path, i);
                if segs.iter().any(|seg| {
                    checks.iter().all(|(elem, comp, val)| {
                        seg.elements
                            .get(*elem)
                            .and_then(|e| e.get(*comp))
                            .is_some_and(|v| v == val)
                    })
                }) {
                    return ConditionResult::True;
                }
            }
            return ConditionResult::False;
        }
        // Fallback: message-wide
        self.has_segment_matching(tag, checks)
    }

    // --- DTM date comparison helpers ---

    /// Check if a DTM segment with the given qualifier has a value >= threshold.
    ///
    /// Both the DTM value and threshold should be in EDIFACT format 303 (CCYYMMDDHHMM).
    /// Returns `Unknown` if no DTM with the qualifier exists.
    pub fn dtm_ge(&self, qualifier: &str, threshold: &str) -> ConditionResult {
        let segs = self.find_segments_with_qualifier("DTM", 0, qualifier);
        match segs.first() {
            Some(dtm) => match dtm.elements.first().and_then(|e| e.get(1)) {
                Some(val) => ConditionResult::from(val.as_str() >= threshold),
                None => ConditionResult::Unknown,
            },
            None => ConditionResult::Unknown,
        }
    }

    /// Check if a DTM segment with the given qualifier has a value < threshold.
    pub fn dtm_lt(&self, qualifier: &str, threshold: &str) -> ConditionResult {
        let segs = self.find_segments_with_qualifier("DTM", 0, qualifier);
        match segs.first() {
            Some(dtm) => match dtm.elements.first().and_then(|e| e.get(1)) {
                Some(val) => ConditionResult::from((val.as_str()) < threshold),
                None => ConditionResult::Unknown,
            },
            None => ConditionResult::Unknown,
        }
    }

    /// Check if a DTM segment with the given qualifier has a value <= threshold.
    pub fn dtm_le(&self, qualifier: &str, threshold: &str) -> ConditionResult {
        let segs = self.find_segments_with_qualifier("DTM", 0, qualifier);
        match segs.first() {
            Some(dtm) => match dtm.elements.first().and_then(|e| e.get(1)) {
                Some(val) => ConditionResult::from(val.as_str() <= threshold),
                None => ConditionResult::Unknown,
            },
            None => ConditionResult::Unknown,
        }
    }

    // --- Group-scoped cardinality helpers ---

    /// Count segments matching tag + qualifier within a group path (across all instances).
    ///
    /// Returns the total count across all group instances.
    /// Falls back to message-wide count if no navigator.
    pub fn count_qualified_in_group(
        &self,
        tag: &str,
        element_index: usize,
        qualifier: &str,
        group_path: &[&str],
    ) -> usize {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            let mut total = 0;
            for i in 0..instance_count {
                total += self
                    .find_segments_with_qualifier_in_group(
                        tag,
                        element_index,
                        qualifier,
                        group_path,
                        i,
                    )
                    .len();
            }
            return total;
        }
        // Fallback: message-wide
        self.find_segments_with_qualifier(tag, element_index, qualifier)
            .len()
    }

    /// Count segments matching tag (any qualifier) within a group path.
    pub fn count_in_group(&self, tag: &str, group_path: &[&str]) -> usize {
        let instance_count = self.group_instance_count(group_path);
        if instance_count > 0 {
            let mut total = 0;
            for i in 0..instance_count {
                total += self.find_segments_in_group(tag, group_path, i).len();
            }
            return total;
        }
        // Fallback: message-wide
        self.find_segments(tag).len()
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
        /// Children: (parent_path, parent_instance, child_group_id, child_instance, segments)
        children: Vec<(Vec<String>, usize, String, usize, Vec<OwnedSegment>)>,
    }

    impl MockGroupNavigator {
        fn new() -> Self {
            Self {
                groups: vec![],
                children: vec![],
            }
        }
        fn with_group(mut self, path: &[&str], instance: usize, segs: Vec<OwnedSegment>) -> Self {
            self.groups
                .push((path.iter().map(|s| s.to_string()).collect(), instance, segs));
            self
        }
        fn with_child_group(
            mut self,
            parent_path: &[&str],
            parent_instance: usize,
            child_id: &str,
            child_instance: usize,
            segs: Vec<OwnedSegment>,
        ) -> Self {
            self.children.push((
                parent_path.iter().map(|s| s.to_string()).collect(),
                parent_instance,
                child_id.to_string(),
                child_instance,
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
        fn child_group_instance_count(
            &self,
            parent_path: &[&str],
            parent_instance: usize,
            child_group_id: &str,
        ) -> usize {
            self.children
                .iter()
                .filter(|(pp, pi, cid, _, _)| {
                    let ps: Vec<&str> = pp.iter().map(|s| s.as_str()).collect();
                    ps.as_slice() == parent_path && *pi == parent_instance && cid == child_group_id
                })
                .count()
        }
        fn find_segments_in_child_group(
            &self,
            segment_id: &str,
            parent_path: &[&str],
            parent_instance: usize,
            child_group_id: &str,
            child_instance: usize,
        ) -> Vec<OwnedSegment> {
            self.children
                .iter()
                .find(|(pp, pi, cid, ci, _)| {
                    let ps: Vec<&str> = pp.iter().map(|s| s.as_str()).collect();
                    ps.as_slice() == parent_path
                        && *pi == parent_instance
                        && cid == child_group_id
                        && *ci == child_instance
                })
                .map(|(_, _, _, _, segs)| {
                    segs.iter()
                        .filter(|s| s.id == segment_id)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        }
        fn extract_value_in_group(
            &self,
            segment_id: &str,
            element_index: usize,
            component_index: usize,
            group_path: &[&str],
            instance_index: usize,
        ) -> Option<String> {
            let segs = self.find_instance(group_path, instance_index)?;
            let seg = segs.iter().find(|s| s.id == segment_id)?;
            seg.elements
                .get(element_index)?
                .get(component_index)
                .cloned()
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
        let result = ctx.find_segments_with_qualifier_in_group("SEQ", 0, "Z98", &["SG4", "SG8"], 0);
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

    // --- High-level helper tests ---

    #[test]
    fn test_has_qualifier() {
        let segments = vec![
            make_segment("NAD", vec![vec!["MS"], vec!["111"]]),
            make_segment("NAD", vec![vec!["MR"], vec!["222"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        assert_eq!(ctx.has_qualifier("NAD", 0, "MS"), ConditionResult::True);
        assert_eq!(ctx.has_qualifier("NAD", 0, "DP"), ConditionResult::False);
    }

    #[test]
    fn test_lacks_qualifier() {
        let segments = vec![make_segment("DTM", vec![vec!["92", "2025"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        assert_eq!(ctx.lacks_qualifier("DTM", 0, "93"), ConditionResult::True);
        assert_eq!(ctx.lacks_qualifier("DTM", 0, "92"), ConditionResult::False);
    }

    #[test]
    fn test_has_qualified_value() {
        let segments = vec![make_segment("STS", vec![vec!["7"], vec![], vec!["ZG9"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(
            ctx.has_qualified_value("STS", 0, "7", 2, 0, &["ZG9", "ZH1", "ZH2"]),
            ConditionResult::True,
        );
        assert_eq!(
            ctx.has_qualified_value("STS", 0, "7", 2, 0, &["E01"]),
            ConditionResult::False,
        );
        // No STS+E01 → Unknown
        assert_eq!(
            ctx.has_qualified_value("STS", 0, "E01", 2, 0, &["Z01"]),
            ConditionResult::Unknown,
        );
    }

    #[test]
    fn test_any_group_has_qualifier() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![make_segment("SEQ", vec![vec!["Z01"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                1,
                vec![make_segment("SEQ", vec![vec!["Z98"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        assert_eq!(
            ctx.any_group_has_qualifier("SEQ", 0, "Z98", &["SG4", "SG8"]),
            ConditionResult::True,
        );
        assert_eq!(
            ctx.any_group_has_qualifier("SEQ", 0, "Z99", &["SG4", "SG8"]),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_any_group_has_qualifier_fallback() {
        // No navigator — falls back to message-wide search
        let segments = vec![make_segment("SEQ", vec![vec!["Z98"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(
            ctx.any_group_has_qualifier("SEQ", 0, "Z98", &["SG4", "SG8"]),
            ConditionResult::True,
        );
    }

    #[test]
    fn test_any_group_has_any_qualifier() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new().with_group(
            &["SG4", "SG8"],
            0,
            vec![make_segment("SEQ", vec![vec!["Z80"]])],
        );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        assert_eq!(
            ctx.any_group_has_any_qualifier("SEQ", 0, &["Z01", "Z80", "Z81"], &["SG4", "SG8"]),
            ConditionResult::True,
        );
        assert_eq!(
            ctx.any_group_has_any_qualifier("SEQ", 0, &["Z98"], &["SG4", "SG8"]),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_any_group_has_co_occurrence() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new().with_group(
            &["SG4", "SG8"],
            0,
            vec![
                make_segment("SEQ", vec![vec!["Z01"]]),
                make_segment("CCI", vec![vec!["Z30"], vec![], vec!["Z07"]]),
            ],
        );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        assert_eq!(
            ctx.any_group_has_co_occurrence(
                "SEQ",
                0,
                &["Z01"],
                "CCI",
                2,
                0,
                &["Z07"],
                &["SG4", "SG8"],
            ),
            ConditionResult::True,
        );
        // Wrong CCI value
        assert_eq!(
            ctx.any_group_has_co_occurrence(
                "SEQ",
                0,
                &["Z01"],
                "CCI",
                2,
                0,
                &["ZC0"],
                &["SG4", "SG8"],
            ),
            ConditionResult::False,
        );
    }

    // --- Parent-child navigation tests ---

    #[test]
    fn test_filtered_parent_child_has_qualifier() {
        let external = NoOpExternalProvider;
        // SG8[0] has SEQ+Z98, with SG10 child having CCI+Z23
        // SG8[1] has SEQ+Z01, no SG10 children
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
            )
            .with_child_group(
                &["SG4", "SG8"],
                0,
                "SG10",
                0,
                vec![make_segment("CCI", vec![vec!["Z23"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        // SG8 with SEQ+Z98 has SG10 child with CCI+Z23 → True
        assert_eq!(
            ctx.filtered_parent_child_has_qualifier(
                &["SG4", "SG8"],
                "SEQ",
                0,
                "Z98",
                "SG10",
                "CCI",
                0,
                "Z23",
            ),
            ConditionResult::True,
        );
        // SG8 with SEQ+Z01 has no SG10 children → False
        assert_eq!(
            ctx.filtered_parent_child_has_qualifier(
                &["SG4", "SG8"],
                "SEQ",
                0,
                "Z01",
                "SG10",
                "CCI",
                0,
                "Z23",
            ),
            ConditionResult::False,
        );
        // Wrong child qualifier → False
        assert_eq!(
            ctx.filtered_parent_child_has_qualifier(
                &["SG4", "SG8"],
                "SEQ",
                0,
                "Z98",
                "SG10",
                "CCI",
                0,
                "Z99",
            ),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_filtered_parent_child_fallback() {
        // No navigator — falls back to message-wide
        let segments = vec![
            make_segment("SEQ", vec![vec!["Z98"]]),
            make_segment("CCI", vec![vec!["Z23"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(
            ctx.filtered_parent_child_has_qualifier(
                &["SG4", "SG8"],
                "SEQ",
                0,
                "Z98",
                "SG10",
                "CCI",
                0,
                "Z23",
            ),
            ConditionResult::True,
        );
        // Missing child qualifier in message-wide → False
        assert_eq!(
            ctx.filtered_parent_child_has_qualifier(
                &["SG4", "SG8"],
                "SEQ",
                0,
                "Z98",
                "SG10",
                "CCI",
                0,
                "Z99",
            ),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_any_group_has_qualifier_without() {
        let external = NoOpExternalProvider;
        // SG8[0]: SEQ+Z59 present, CCI+11 absent
        // SG8[1]: SEQ+Z01 present, CCI+11 present
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![make_segment("SEQ", vec![vec!["Z59"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                1,
                vec![
                    make_segment("SEQ", vec![vec!["Z01"]]),
                    make_segment("CCI", vec![vec!["11"]]),
                ],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        // SG8[0] has SEQ+Z59 without CCI+11 → True
        assert_eq!(
            ctx.any_group_has_qualifier_without("SEQ", 0, "Z59", "CCI", 0, "11", &["SG4", "SG8"]),
            ConditionResult::True,
        );
        // Looking for SEQ+Z01 without CCI+11 → False (SG8[1] has both)
        assert_eq!(
            ctx.any_group_has_qualifier_without("SEQ", 0, "Z01", "CCI", 0, "11", &["SG4", "SG8"]),
            ConditionResult::False,
        );
        // Looking for SEQ+Z99 (doesn't exist) → False
        assert_eq!(
            ctx.any_group_has_qualifier_without("SEQ", 0, "Z99", "CCI", 0, "11", &["SG4", "SG8"]),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_any_group_has_qualifier_without_fallback() {
        let segments = vec![make_segment("SEQ", vec![vec!["Z59"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        // Message-wide: SEQ+Z59 present, CCI+11 absent → True
        assert_eq!(
            ctx.any_group_has_qualifier_without("SEQ", 0, "Z59", "CCI", 0, "11", &["SG4", "SG8"]),
            ConditionResult::True,
        );
    }

    #[test]
    fn test_collect_group_values() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG6"],
                0,
                vec![make_segment("RFF", vec![vec!["Z49", "REF001"]])],
            )
            .with_group(
                &["SG4", "SG6"],
                1,
                vec![make_segment("RFF", vec![vec!["Z49", "REF002"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        let values = ctx.collect_group_values("RFF", 0, 1, &["SG4", "SG6"]);
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], (0, "REF001".to_string()));
        assert_eq!(values[1], (1, "REF002".to_string()));
    }

    #[test]
    fn test_groups_share_qualified_value() {
        let external = NoOpExternalProvider;
        // SG6[0]: RFF+Z49 with value "TS001"
        // SG8[0]: SEQ with c286 value "TS001" (matches)
        // SG8[1]: SEQ with c286 value "TS999" (no match)
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG6"],
                0,
                vec![make_segment("RFF", vec![vec!["Z49", "TS001"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![make_segment("SEQ", vec![vec!["Z98"], vec!["TS001"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                1,
                vec![make_segment("SEQ", vec![vec!["Z01"], vec!["TS999"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        // RFF+Z49 value "TS001" matches SEQ value at [1][0] → True
        assert_eq!(
            ctx.groups_share_qualified_value(
                "RFF",
                0,
                "Z49",
                0,
                1,
                &["SG4", "SG6"],
                "SEQ",
                1,
                0,
                &["SG4", "SG8"],
            ),
            ConditionResult::True,
        );
    }

    #[test]
    fn test_groups_share_qualified_value_no_match() {
        let external = NoOpExternalProvider;
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG6"],
                0,
                vec![make_segment("RFF", vec![vec!["Z49", "TS001"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![make_segment("SEQ", vec![vec!["Z98"], vec!["TS999"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        // No matching value → False
        assert_eq!(
            ctx.groups_share_qualified_value(
                "RFF",
                0,
                "Z49",
                0,
                1,
                &["SG4", "SG6"],
                "SEQ",
                1,
                0,
                &["SG4", "SG8"],
            ),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_groups_share_qualified_value_no_source() {
        let external = NoOpExternalProvider;
        // No RFF+Z49 at all
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG6"],
                0,
                vec![make_segment("RFF", vec![vec!["Z13", "55001"]])],
            )
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![make_segment("SEQ", vec![vec!["Z98"], vec!["TS001"]])],
            );
        let ctx = EvaluationContext::with_navigator("55001", &external, &[], &nav);

        // No source qualifier match → Unknown
        assert_eq!(
            ctx.groups_share_qualified_value(
                "RFF",
                0,
                "Z49",
                0,
                1,
                &["SG4", "SG6"],
                "SEQ",
                1,
                0,
                &["SG4", "SG8"],
            ),
            ConditionResult::Unknown,
        );
    }

    #[test]
    fn test_groups_share_qualified_value_fallback() {
        // No navigator — falls back to message-wide
        let segments = vec![
            make_segment("RFF", vec![vec!["Z49", "TS001"]]),
            make_segment("SEQ", vec![vec!["Z98"], vec!["TS001"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(
            ctx.groups_share_qualified_value(
                "RFF",
                0,
                "Z49",
                0,
                1,
                &["SG4", "SG6"],
                "SEQ",
                1,
                0,
                &["SG4", "SG8"],
            ),
            ConditionResult::True,
        );
    }

    #[test]
    fn test_child_group_pass_throughs_no_navigator() {
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &[]);

        assert_eq!(
            ctx.child_group_instance_count(&["SG4", "SG8"], 0, "SG10"),
            0
        );
        assert!(ctx
            .find_segments_in_child_group("CCI", &["SG4", "SG8"], 0, "SG10", 0)
            .is_empty());
        assert_eq!(
            ctx.extract_value_in_group("SEQ", 0, 0, &["SG4", "SG8"], 0),
            None
        );
    }

    // --- has_segment_matching tests ---

    #[test]
    fn test_has_segment_matching_found() {
        let segments = vec![
            make_segment("STS", vec![vec!["7"], vec!["E01"], vec!["ZW4"]]),
            make_segment("STS", vec![vec!["Z20"], vec!["Z32"], vec!["A99"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(
            ctx.has_segment_matching("STS", &[(0, 0, "Z20"), (1, 0, "Z32"), (2, 0, "A99")]),
            ConditionResult::True,
        );
    }

    #[test]
    fn test_has_segment_matching_not_found() {
        let segments = vec![make_segment(
            "STS",
            vec![vec!["7"], vec!["E01"], vec!["ZW4"]],
        )];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(
            ctx.has_segment_matching("STS", &[(0, 0, "Z20"), (1, 0, "Z32")]),
            ConditionResult::False,
        );
    }

    #[test]
    fn test_has_segment_matching_no_segments() {
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &[]);

        assert_eq!(
            ctx.has_segment_matching("STS", &[(0, 0, "Z20")]),
            ConditionResult::Unknown,
        );
    }

    #[test]
    fn test_has_segment_matching_in_group() {
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4"],
                0,
                vec![make_segment("STS", vec![vec!["7"], vec!["E01"]])],
            )
            .with_group(
                &["SG4"],
                1,
                vec![make_segment("STS", vec![vec!["Z20"], vec!["Z32"]])],
            );
        let segments = vec![
            make_segment("STS", vec![vec!["7"], vec!["E01"]]),
            make_segment("STS", vec![vec!["Z20"], vec!["Z32"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::with_navigator("55001", &external, &segments, &nav);

        assert_eq!(
            ctx.has_segment_matching_in_group("STS", &[(0, 0, "Z20"), (1, 0, "Z32")], &["SG4"]),
            ConditionResult::True,
        );
    }

    // --- DTM comparison tests ---

    #[test]
    fn test_dtm_ge() {
        let segments = vec![make_segment(
            "DTM",
            vec![vec!["137", "202601010000", "303"]],
        )];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(ctx.dtm_ge("137", "202601010000"), ConditionResult::True);
        assert_eq!(ctx.dtm_ge("137", "202501010000"), ConditionResult::True);
        assert_eq!(ctx.dtm_ge("137", "202701010000"), ConditionResult::False);
        assert_eq!(ctx.dtm_ge("999", "202601010000"), ConditionResult::Unknown);
    }

    #[test]
    fn test_dtm_lt() {
        let segments = vec![make_segment(
            "DTM",
            vec![vec!["137", "202601010000", "303"]],
        )];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(ctx.dtm_lt("137", "202701010000"), ConditionResult::True);
        assert_eq!(ctx.dtm_lt("137", "202601010000"), ConditionResult::False);
        assert_eq!(ctx.dtm_lt("137", "202501010000"), ConditionResult::False);
    }

    #[test]
    fn test_dtm_le() {
        let segments = vec![make_segment(
            "DTM",
            vec![vec!["137", "202601010000", "303"]],
        )];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        assert_eq!(ctx.dtm_le("137", "202601010000"), ConditionResult::True);
        assert_eq!(ctx.dtm_le("137", "202701010000"), ConditionResult::True);
        assert_eq!(ctx.dtm_le("137", "202501010000"), ConditionResult::False);
    }

    // --- Count helpers tests ---

    #[test]
    fn test_count_qualified_in_group() {
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![
                    make_segment("CCI", vec![vec!["Z23"]]),
                    make_segment("CCI", vec![vec!["Z30"]]),
                ],
            )
            .with_group(
                &["SG4", "SG8"],
                1,
                vec![make_segment("CCI", vec![vec!["Z23"]])],
            );
        let segments = vec![
            make_segment("CCI", vec![vec!["Z23"]]),
            make_segment("CCI", vec![vec!["Z30"]]),
            make_segment("CCI", vec![vec!["Z23"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::with_navigator("55001", &external, &segments, &nav);

        assert_eq!(
            ctx.count_qualified_in_group("CCI", 0, "Z23", &["SG4", "SG8"]),
            2
        );
        assert_eq!(
            ctx.count_qualified_in_group("CCI", 0, "Z30", &["SG4", "SG8"]),
            1
        );
        assert_eq!(
            ctx.count_qualified_in_group("CCI", 0, "Z99", &["SG4", "SG8"]),
            0
        );
    }

    #[test]
    fn test_count_in_group() {
        let nav = MockGroupNavigator::new()
            .with_group(
                &["SG4", "SG8"],
                0,
                vec![
                    make_segment("SEQ", vec![vec!["Z98"]]),
                    make_segment("CCI", vec![vec!["Z23"]]),
                ],
            )
            .with_group(
                &["SG4", "SG8"],
                1,
                vec![make_segment("SEQ", vec![vec!["Z01"]])],
            );
        let segments = vec![
            make_segment("SEQ", vec![vec!["Z98"]]),
            make_segment("CCI", vec![vec!["Z23"]]),
            make_segment("SEQ", vec![vec!["Z01"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::with_navigator("55001", &external, &segments, &nav);

        assert_eq!(ctx.count_in_group("SEQ", &["SG4", "SG8"]), 2);
        assert_eq!(ctx.count_in_group("CCI", &["SG4", "SG8"]), 1);
        assert_eq!(ctx.count_in_group("DTM", &["SG4", "SG8"]), 0);
    }

    #[test]
    fn test_count_fallback_no_navigator() {
        let segments = vec![
            make_segment("CCI", vec![vec!["Z23"]]),
            make_segment("CCI", vec![vec!["Z30"]]),
            make_segment("CCI", vec![vec!["Z23"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("55001", &external, &segments);

        // Falls back to message-wide count
        assert_eq!(
            ctx.count_qualified_in_group("CCI", 0, "Z23", &["SG4", "SG8"]),
            2
        );
        assert_eq!(ctx.count_in_group("CCI", &["SG4", "SG8"]), 3);
    }
}
