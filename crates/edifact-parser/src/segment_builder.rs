use edifact_types::{EdifactDelimiters, RawSegment, SegmentPosition};

use crate::EdifactTokenizer;

/// Builds `RawSegment` instances from raw segment strings.
///
/// Takes the tokenized segment string (e.g., "NAD+Z04+9900123000002::293")
/// and splits it into the segment ID, elements, and components.
pub struct SegmentBuilder {
    tokenizer: EdifactTokenizer,
}

impl SegmentBuilder {
    /// Creates a new segment builder with the given delimiters.
    pub fn new(delimiters: EdifactDelimiters) -> Self {
        Self {
            tokenizer: EdifactTokenizer::new(delimiters),
        }
    }

    /// Parses a raw segment string into a `RawSegment`.
    ///
    /// The input is a single segment WITHOUT its terminator character.
    /// Example: `"NAD+Z04+9900123000002::293"`
    ///
    /// Returns `None` if the segment string is empty.
    pub fn build<'a>(
        &self,
        segment_str: &'a str,
        position: SegmentPosition,
    ) -> Option<RawSegment<'a>> {
        if segment_str.is_empty() {
            return None;
        }

        let mut elements_iter = self.tokenizer.tokenize_elements(segment_str);

        // First element is the segment ID
        let id = elements_iter.next()?;
        if id.is_empty() {
            return None;
        }

        // Remaining elements are data elements, each split into components
        let mut elements = Vec::new();
        for element_str in elements_iter {
            let components: Vec<&'a str> =
                self.tokenizer.tokenize_components(element_str).collect();
            elements.push(components);
        }

        Some(RawSegment::new(id, elements, position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(n: u32, offset: usize) -> SegmentPosition {
        SegmentPosition::new(n, offset, 1)
    }

    #[test]
    fn test_build_simple_segment() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder
            .build("UNH+00001+UTILMD:D:11A:UN:S2.1", pos(1, 0))
            .unwrap();

        assert_eq!(seg.id, "UNH");
        assert_eq!(seg.element_count(), 2);
        assert_eq!(seg.get_element(0), "00001");
        assert_eq!(seg.get_component(1, 0), "UTILMD");
        assert_eq!(seg.get_component(1, 1), "D");
        assert_eq!(seg.get_component(1, 2), "11A");
        assert_eq!(seg.get_component(1, 3), "UN");
        assert_eq!(seg.get_component(1, 4), "S2.1");
    }

    #[test]
    fn test_build_nad_segment() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder
            .build("NAD+Z04+9900123000002::293", pos(5, 100))
            .unwrap();

        assert_eq!(seg.id, "NAD");
        assert_eq!(seg.get_element(0), "Z04");
        assert_eq!(seg.get_component(1, 0), "9900123000002");
        assert_eq!(seg.get_component(1, 1), "");
        assert_eq!(seg.get_component(1, 2), "293");
    }

    #[test]
    fn test_build_dtm_with_escaped_plus() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder
            .build("DTM+137:202501010000?+01:303", pos(3, 50))
            .unwrap();

        assert_eq!(seg.id, "DTM");
        assert_eq!(seg.get_component(0, 0), "137");
        assert_eq!(seg.get_component(0, 1), "202501010000?+01");
        assert_eq!(seg.get_component(0, 2), "303");
    }

    #[test]
    fn test_build_segment_no_elements() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder.build("UNA", pos(1, 0)).unwrap();

        assert_eq!(seg.id, "UNA");
        assert_eq!(seg.element_count(), 0);
    }

    #[test]
    fn test_build_empty_input() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        assert!(builder.build("", pos(1, 0)).is_none());
    }

    #[test]
    fn test_build_loc_segment() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder
            .build("LOC+Z16+DE00014545768S0000000000000003054", pos(8, 200))
            .unwrap();

        assert_eq!(seg.id, "LOC");
        assert_eq!(seg.get_element(0), "Z16");
        assert_eq!(seg.get_element(1), "DE00014545768S0000000000000003054");
    }

    #[test]
    fn test_build_preserves_position() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder.build("BGM+E03+DOC001", pos(2, 42)).unwrap();

        assert_eq!(seg.position.segment_number, 2);
        assert_eq!(seg.position.byte_offset, 42);
        assert_eq!(seg.position.message_number, 1);
    }

    #[test]
    fn test_build_rff_segment() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder.build("RFF+Z13:TXREF001", pos(10, 300)).unwrap();

        assert_eq!(seg.id, "RFF");
        assert_eq!(seg.get_component(0, 0), "Z13");
        assert_eq!(seg.get_component(0, 1), "TXREF001");
    }
}
