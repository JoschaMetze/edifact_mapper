//! EDIFACT tokenization helper — collects segments from input into a Vec.
//!
//! The `edifact-parser` crate uses an event-driven (SAX-style) API.
//! This module provides a convenience function that collects all parsed
//! segments into an owned `Vec<OwnedSegment>` for two-pass processing.
//!
//! `OwnedSegment` itself lives in `mig-types::segment` — re-exported here
//! for backward compatibility.

use edifact_types::{Control, EdifactDelimiters, RawSegment};

// Re-export OwnedSegment from mig-types so existing `use crate::tokenize::OwnedSegment` paths work.
pub use mig_types::segment::OwnedSegment;

/// Handler that collects all segments into owned copies.
struct SegmentCollector {
    segments: Vec<OwnedSegment>,
}

impl edifact_parser::EdifactHandler for SegmentCollector {
    fn on_segment(&mut self, segment: &RawSegment<'_>) -> Control {
        self.segments.push(OwnedSegment {
            id: segment.id.to_string(),
            elements: segment
                .elements
                .iter()
                .map(|e| e.iter().map(|c| c.to_string()).collect())
                .collect(),
            segment_number: segment.position.segment_number,
        });
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
