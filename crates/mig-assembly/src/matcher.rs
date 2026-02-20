//! Segment matching logic for MIG-guided assembly.
//!
//! Determines whether an OwnedSegment matches a MIG tree node based on
//! segment tag and optional qualifier values.

use crate::tokenize::OwnedSegment;

/// Check if a segment's tag matches the expected MIG segment ID (case-insensitive).
pub fn matches_segment_tag(segment_tag: &str, expected_tag: &str) -> bool {
    segment_tag.eq_ignore_ascii_case(expected_tag)
}

/// Check if a qualifier value matches the expected qualifier.
pub fn matches_qualifier(actual: &str, expected: &str) -> bool {
    actual.trim() == expected.trim()
}

/// Check if an OwnedSegment matches a MIG node, optionally checking a qualifier.
///
/// - `segment`: the owned EDIFACT segment
/// - `expected_tag`: the MIG segment ID (e.g., "NAD")
/// - `expected_qualifier`: if Some, the first element's first component must match
pub fn matches_mig_node(
    segment: &OwnedSegment,
    expected_tag: &str,
    expected_qualifier: Option<&str>,
) -> bool {
    if !matches_segment_tag(&segment.id, expected_tag) {
        return false;
    }
    match expected_qualifier {
        Some(q) => {
            let actual = segment.get_element(0);
            matches_qualifier(actual, q)
        }
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_segment_by_tag() {
        assert!(matches_segment_tag("NAD", "NAD"));
        assert!(!matches_segment_tag("NAD", "LOC"));
    }

    #[test]
    fn test_match_segment_tag_case_insensitive() {
        assert!(matches_segment_tag("nad", "NAD"));
        assert!(matches_segment_tag("Nad", "nad"));
    }

    #[test]
    fn test_match_segment_with_qualifier() {
        assert!(matches_qualifier("MS", "MS"));
        assert!(!matches_qualifier("MS", "MR"));
    }

    #[test]
    fn test_match_qualifier_trims_whitespace() {
        assert!(matches_qualifier(" MS ", "MS"));
        assert!(matches_qualifier("MS", " MS "));
    }

    #[test]
    fn test_matches_mig_node_tag_only() {
        let seg = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["Z04".to_string()]],
            segment_number: 1,
        };
        assert!(matches_mig_node(&seg, "NAD", None));
        assert!(!matches_mig_node(&seg, "LOC", None));
    }

    #[test]
    fn test_matches_mig_node_with_qualifier() {
        let seg = OwnedSegment {
            id: "NAD".to_string(),
            elements: vec![vec!["MS".to_string()]],
            segment_number: 1,
        };
        assert!(matches_mig_node(&seg, "NAD", Some("MS")));
        assert!(!matches_mig_node(&seg, "NAD", Some("MR")));
    }

    #[test]
    fn test_matches_mig_node_no_elements() {
        let seg = OwnedSegment {
            id: "UNH".to_string(),
            elements: vec![],
            segment_number: 1,
        };
        // No qualifier check — just tag
        assert!(matches_mig_node(&seg, "UNH", None));
        // Qualifier check fails because no elements
        assert!(!matches_mig_node(&seg, "UNH", Some("001")));
    }
}
