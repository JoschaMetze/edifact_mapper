use crate::SegmentPosition;

/// A parsed EDIFACT segment that borrows from the input buffer.
///
/// Zero-copy design: all string data references the original input,
/// avoiding allocations during the parsing hot path.
///
/// # Structure
///
/// An EDIFACT segment like `NAD+Z04+9900123000002:500` has:
/// - `id`: `"NAD"`
/// - `elements[0]`: `["Z04"]` (simple element)
/// - `elements[1]`: `["9900123000002", "500"]` (composite element with 2 components)
#[derive(Debug, Clone)]
pub struct RawSegment<'a> {
    /// Segment identifier (e.g., "NAD", "LOC", "DTM").
    pub id: &'a str,
    /// Elements, where each element is a vector of component strings.
    /// `elements[i][j]` = component `j` of element `i`.
    pub elements: Vec<Vec<&'a str>>,
    /// Position metadata for this segment.
    pub position: SegmentPosition,
}

impl<'a> RawSegment<'a> {
    /// Creates a new RawSegment.
    pub fn new(id: &'a str, elements: Vec<Vec<&'a str>>, position: SegmentPosition) -> Self {
        Self {
            id,
            elements,
            position,
        }
    }

    /// Returns the number of elements (excluding the segment ID).
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    /// Gets the first component of element at `index`, or empty string if missing.
    ///
    /// This is a convenience method for accessing simple (non-composite) elements.
    pub fn get_element(&self, index: usize) -> &str {
        self.elements
            .get(index)
            .and_then(|e| e.first())
            .copied()
            .unwrap_or("")
    }

    /// Gets a specific component within an element, or empty string if missing.
    ///
    /// `element_index` is the 0-based element position.
    /// `component_index` is the 0-based component position within that element.
    pub fn get_component(&self, element_index: usize, component_index: usize) -> &str {
        self.elements
            .get(element_index)
            .and_then(|e| e.get(component_index))
            .copied()
            .unwrap_or("")
    }

    /// Returns all components of element at `index`, or empty slice if missing.
    pub fn get_components(&self, element_index: usize) -> &[&'a str] {
        self.elements
            .get(element_index)
            .map_or(&[], |e| e.as_slice())
    }

    /// Checks if the segment has the given ID (case-insensitive).
    pub fn is(&self, segment_id: &str) -> bool {
        self.id.eq_ignore_ascii_case(segment_id)
    }

    /// Reconstruct the raw segment string (without terminator) using the given delimiters.
    ///
    /// This produces `ID+elem1:comp1:comp2+elem2` format (without the trailing terminator).
    pub fn to_raw_string(&self, delimiters: &crate::EdifactDelimiters) -> String {
        let elem_sep = delimiters.element as char;
        let comp_sep = delimiters.component as char;

        let mut result = self.id.to_string();

        for element in &self.elements {
            result.push(elem_sep);
            // Preserve ALL components including trailing empty ones for roundtrip fidelity.
            // E.g. CAV+SA::::' must keep the trailing colons.
            for (j, component) in element.iter().enumerate() {
                if j > 0 {
                    result.push(comp_sep);
                }
                result.push_str(component);
            }
        }

        // Trim trailing empty elements (trailing element separators)
        while result.ends_with(elem_sep) {
            result.pop();
        }

        result
    }
}

impl<'a> std::fmt::Display for RawSegment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)?;
        for element in &self.elements {
            write!(f, "+")?;
            for (j, component) in element.iter().enumerate() {
                if j > 0 {
                    write!(f, ":")?;
                }
                write!(f, "{component}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_position() -> SegmentPosition {
        SegmentPosition::new(1, 0, 1)
    }

    #[test]
    fn test_raw_segment_simple() {
        let seg = RawSegment::new(
            "UNH",
            vec![vec!["00001"], vec!["UTILMD", "D", "11A", "UN", "S2.1"]],
            make_position(),
        );
        assert_eq!(seg.id, "UNH");
        assert_eq!(seg.element_count(), 2);
        assert_eq!(seg.get_element(0), "00001");
        assert_eq!(seg.get_component(1, 0), "UTILMD");
        assert_eq!(seg.get_component(1, 4), "S2.1");
    }

    #[test]
    fn test_raw_segment_get_element_out_of_bounds() {
        let seg = RawSegment::new("BGM", vec![vec!["E03"]], make_position());
        assert_eq!(seg.get_element(0), "E03");
        assert_eq!(seg.get_element(1), "");
        assert_eq!(seg.get_element(99), "");
    }

    #[test]
    fn test_raw_segment_get_component_out_of_bounds() {
        let seg = RawSegment::new("NAD", vec![vec!["Z04", "123"]], make_position());
        assert_eq!(seg.get_component(0, 0), "Z04");
        assert_eq!(seg.get_component(0, 1), "123");
        assert_eq!(seg.get_component(0, 2), "");
        assert_eq!(seg.get_component(1, 0), "");
    }

    #[test]
    fn test_raw_segment_display() {
        let seg = RawSegment::new(
            "NAD",
            vec![vec!["Z04"], vec!["9900123000002", "500"]],
            make_position(),
        );
        assert_eq!(seg.to_string(), "NAD+Z04+9900123000002:500");
    }

    #[test]
    fn test_raw_segment_display_no_elements() {
        let seg = RawSegment::new("UNA", vec![], make_position());
        assert_eq!(seg.to_string(), "UNA");
    }

    #[test]
    fn test_raw_segment_is_case_insensitive() {
        let seg = RawSegment::new("NAD", vec![], make_position());
        assert!(seg.is("NAD"));
        assert!(seg.is("nad"));
        assert!(seg.is("Nad"));
        assert!(!seg.is("LOC"));
    }

    #[test]
    fn test_raw_segment_get_components() {
        let seg = RawSegment::new(
            "DTM",
            vec![vec!["137", "202501010000+01", "303"]],
            make_position(),
        );
        let components = seg.get_components(0);
        assert_eq!(components, &["137", "202501010000+01", "303"]);
        assert!(seg.get_components(1).is_empty());
    }

    #[test]
    fn test_raw_segment_zero_copy_lifetime() {
        let input = String::from("NAD+Z04+9900123000002:500");
        let seg = RawSegment::new(
            &input[0..3],
            vec![vec![&input[4..7]], vec![&input[8..21], &input[22..25]]],
            make_position(),
        );
        // Verify that the segment borrows from the input
        assert_eq!(seg.id, "NAD");
        assert_eq!(seg.get_element(0), "Z04");
        assert_eq!(seg.get_component(1, 0), "9900123000002");
        assert_eq!(seg.get_component(1, 1), "500");
    }

    #[test]
    fn test_raw_segment_clone() {
        let seg = RawSegment::new("LOC", vec![vec!["Z16", "DE00014545768"]], make_position());
        let cloned = seg.clone();
        assert_eq!(seg.id, cloned.id);
        assert_eq!(seg.elements, cloned.elements);
        assert_eq!(seg.position, cloned.position);
    }

    #[test]
    fn test_raw_segment_to_raw_string() {
        let seg = RawSegment::new(
            "LOC",
            vec![vec!["Z16"], vec!["DE00014545768S0000000000000003054"]],
            make_position(),
        );
        let delimiters = crate::EdifactDelimiters::default();
        assert_eq!(
            seg.to_raw_string(&delimiters),
            "LOC+Z16+DE00014545768S0000000000000003054"
        );
    }

    #[test]
    fn test_raw_segment_to_raw_string_composite() {
        let seg = RawSegment::new(
            "DTM",
            vec![vec!["137", "202507011330", "303"]],
            make_position(),
        );
        let delimiters = crate::EdifactDelimiters::default();
        assert_eq!(seg.to_raw_string(&delimiters), "DTM+137:202507011330:303");
    }

    #[test]
    fn test_raw_segment_to_raw_string_no_elements() {
        let seg = RawSegment::new("UNA", vec![], make_position());
        let delimiters = crate::EdifactDelimiters::default();
        assert_eq!(seg.to_raw_string(&delimiters), "UNA");
    }

    #[test]
    fn test_raw_segment_to_raw_string_trailing_empty_components() {
        // Segment like "CCI+Z30++Z07" where element[1] is empty
        let seg = RawSegment::new(
            "CCI",
            vec![vec!["Z30"], vec![""], vec!["Z07"]],
            make_position(),
        );
        let delimiters = crate::EdifactDelimiters::default();
        assert_eq!(seg.to_raw_string(&delimiters), "CCI+Z30++Z07");
    }

    #[test]
    fn test_raw_segment_to_raw_string_trailing_empty_elements() {
        // Trailing empty elements should be trimmed
        let seg = RawSegment::new(
            "BGM",
            vec![vec!["E03"], vec![""], vec![""]],
            make_position(),
        );
        let delimiters = crate::EdifactDelimiters::default();
        assert_eq!(seg.to_raw_string(&delimiters), "BGM+E03");
    }
}
