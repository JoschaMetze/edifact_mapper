//! EDIFACT tokenization helper — collects segments from input into a Vec.
//!
//! The `edifact-parser` crate uses an event-driven (SAX-style) API.
//! This module provides a convenience function that collects all parsed
//! segments into an owned `Vec<OwnedSegment>` for two-pass processing.

use edifact_types::{Control, EdifactDelimiters, RawSegment};

/// An owned version of `RawSegment` — stores String data instead of borrows.
///
/// Used for the two-pass assembler: pass 1 collects segments into this
/// type, pass 2 consumes them guided by the MIG schema.
#[derive(Debug, Clone)]
pub struct OwnedSegment {
    /// Segment identifier (e.g., "NAD", "LOC", "DTM").
    pub id: String,
    /// Elements, where each element is a vector of component strings.
    /// `elements[i][j]` = component `j` of element `i`.
    pub elements: Vec<Vec<String>>,
    /// 1-based segment number within the message.
    pub segment_number: u32,
}

impl OwnedSegment {
    /// Creates an OwnedSegment from a borrowed RawSegment.
    pub fn from_raw(raw: &RawSegment<'_>) -> Self {
        Self {
            id: raw.id.to_string(),
            elements: raw
                .elements
                .iter()
                .map(|e| e.iter().map(|c| c.to_string()).collect())
                .collect(),
            segment_number: raw.position.segment_number,
        }
    }

    /// Gets the first component of element at `index`, or empty string if missing.
    pub fn get_element(&self, index: usize) -> &str {
        self.elements
            .get(index)
            .and_then(|e| e.first())
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Gets a specific component within an element, or empty string if missing.
    pub fn get_component(&self, element_index: usize, component_index: usize) -> &str {
        self.elements
            .get(element_index)
            .and_then(|e| e.get(component_index))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Checks if the segment has the given ID (case-insensitive).
    pub fn is(&self, segment_id: &str) -> bool {
        self.id.eq_ignore_ascii_case(segment_id)
    }
}

/// Handler that collects all segments into owned copies.
struct SegmentCollector {
    segments: Vec<OwnedSegment>,
}

impl edifact_parser::EdifactHandler for SegmentCollector {
    fn on_segment(&mut self, segment: &RawSegment<'_>) -> Control {
        self.segments.push(OwnedSegment::from_raw(segment));
        Control::Continue
    }

    fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, _explicit_una: bool) {}

    fn on_interchange_start(&mut self, _unb: &RawSegment<'_>) -> Control {
        Control::Continue
    }

    fn on_message_start(&mut self, _unh: &RawSegment<'_>) -> Control {
        Control::Continue
    }

    fn on_message_end(&mut self, _unt: &RawSegment<'_>) {}

    fn on_interchange_end(&mut self, _unz: &RawSegment<'_>) {}
}

/// Parse an EDIFACT message into a list of owned segments.
///
/// This is "pass 1" of the two-pass assembler. It uses the streaming
/// `edifact-parser` to tokenize the input and collects all segments
/// into owned data structures suitable for random access.
pub fn parse_to_segments(input: &[u8]) -> Result<Vec<OwnedSegment>, crate::AssemblyError> {
    let mut collector = SegmentCollector {
        segments: Vec::new(),
    };
    edifact_parser::EdifactStreamParser::parse(input, &mut collector)
        .map_err(|e| crate::AssemblyError::ParseError(e.to_string()))?;
    Ok(collector.segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_to_segments_minimal() {
        let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'UNT+3+MSG001'UNZ+1+REF001'";
        let segments = parse_to_segments(input).unwrap();

        // Should have UNB, UNH, BGM, UNT, UNZ
        assert_eq!(segments.len(), 5);
        assert!(segments[0].is("UNB"));
        assert!(segments[1].is("UNH"));
        assert!(segments[2].is("BGM"));
        assert!(segments[3].is("UNT"));
        assert!(segments[4].is("UNZ"));
    }

    #[test]
    fn test_parse_to_segments_element_access() {
        let input = b"UNA:+.? 'UNB+UNOC:3'UNH+001+UTILMD:D:11A'BGM+E03+DOC001'UNT+2+001'UNZ+1'";
        let segments = parse_to_segments(input).unwrap();

        let bgm = &segments[2];
        assert_eq!(bgm.id, "BGM");
        assert_eq!(bgm.get_element(0), "E03");
        assert_eq!(bgm.get_element(1), "DOC001");
        assert_eq!(bgm.get_element(99), "");
    }

    #[test]
    fn test_parse_to_segments_composite_access() {
        let input = b"UNA:+.? 'UNH+001+UTILMD:D:11A:UN:S2.1'UNT+1+001'";
        let segments = parse_to_segments(input).unwrap();

        let unh = &segments[0]; // UNH
        assert_eq!(unh.get_component(1, 0), "UTILMD");
        assert_eq!(unh.get_component(1, 1), "D");
        assert_eq!(unh.get_component(1, 4), "S2.1");
    }

    #[test]
    fn test_owned_segment_is_case_insensitive() {
        let input = b"UNA:+.? 'UNB+UNOC:3'UNZ+0'";
        let segments = parse_to_segments(input).unwrap();
        assert!(segments[0].is("unb"));
        assert!(segments[0].is("UNB"));
        assert!(segments[0].is("Unb"));
    }
}
