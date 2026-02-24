//! EDIFACT tokenization helper — collects segments from input into a Vec.
//!
//! The `edifact-parser` crate uses an event-driven (SAX-style) API.
//! This module provides a convenience function that collects all parsed
//! segments into an owned `Vec<OwnedSegment>` for two-pass processing.
//!
//! `OwnedSegment` itself lives in `mig-types::segment` — re-exported here
//! for backward compatibility.

use std::sync::Arc;

use edifact_types::{Control, EdifactDelimiters, RawSegment};

// Re-export OwnedSegment from mig-types so existing `use crate::tokenize::OwnedSegment` paths work.
pub use mig_types::segment::OwnedSegment;

/// A single EDIFACT message (UNH...UNT) with its interchange envelope.
#[derive(Debug, Clone)]
pub struct MessageChunk {
    /// Interchange envelope segments (UNA, UNB) — shared across all messages via `Arc`.
    pub envelope: Arc<Vec<OwnedSegment>>,
    /// The UNH segment itself.
    pub unh: OwnedSegment,
    /// Segments between UNH and UNT (exclusive of both).
    pub body: Vec<OwnedSegment>,
    /// The UNT segment itself.
    pub unt: OwnedSegment,
}

impl MessageChunk {
    /// Reconstruct the full segment list for this message (envelope + UNH + body + UNT).
    /// This is the input format expected by `Assembler::assemble_generic()`.
    pub fn all_segments(&self) -> Vec<OwnedSegment> {
        let mut segs = Vec::with_capacity(self.envelope.len() + 2 + self.body.len());
        segs.extend_from_slice(&self.envelope);
        segs.push(self.unh.clone());
        segs.extend(self.body.iter().cloned());
        segs.push(self.unt.clone());
        segs
    }
}

/// A complete EDIFACT interchange split into per-message chunks.
#[derive(Debug, Clone)]
pub struct InterchangeChunks {
    /// Interchange envelope segments (UNA, UNB) — shared across all messages.
    pub envelope: Vec<OwnedSegment>,
    /// One entry per UNH/UNT pair.
    pub messages: Vec<MessageChunk>,
    /// The UNZ segment (interchange trailer), if present.
    pub unz: Option<OwnedSegment>,
}

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

/// Split a flat segment list into per-message chunks at UNH/UNT boundaries.
///
/// Each message gets a copy of the interchange envelope (UNB and any segments
/// before the first UNH) so it can be independently assembled.
///
/// # Errors
///
/// Returns an error if no UNH/UNT pairs are found.
pub fn split_messages(
    segments: Vec<OwnedSegment>,
) -> Result<InterchangeChunks, crate::AssemblyError> {
    let mut envelope: Vec<OwnedSegment> = Vec::with_capacity(4);
    // Collect (unh, body, unt) tuples first, then wrap with shared envelope Arc.
    let mut raw_messages: Vec<(OwnedSegment, Vec<OwnedSegment>, OwnedSegment)> = Vec::new();
    let mut unz: Option<OwnedSegment> = None;

    // State machine
    let mut current_unh: Option<OwnedSegment> = None;
    let mut current_body: Vec<OwnedSegment> = Vec::with_capacity(32);
    let mut seen_first_unh = false;

    for seg in segments {
        let id_upper = seg.id.to_uppercase();
        match id_upper.as_str() {
            "UNH" => {
                seen_first_unh = true;
                current_unh = Some(seg);
                current_body.clear();
            }
            "UNT" => {
                if let Some(unh) = current_unh.take() {
                    raw_messages.push((unh, std::mem::take(&mut current_body), seg));
                }
            }
            "UNZ" => {
                unz = Some(seg);
            }
            _ => {
                if seen_first_unh {
                    current_body.push(seg);
                } else {
                    envelope.push(seg);
                }
            }
        }
    }

    if raw_messages.is_empty() {
        return Err(crate::AssemblyError::ParseError(
            "No UNH/UNT message pairs found in interchange".to_string(),
        ));
    }

    // Share the envelope via Arc across all messages to avoid N clones.
    let envelope_arc = Arc::new(envelope);
    let messages = raw_messages
        .into_iter()
        .map(|(unh, body, unt)| MessageChunk {
            envelope: Arc::clone(&envelope_arc),
            unh,
            body,
            unt,
        })
        .collect();

    Ok(InterchangeChunks {
        envelope: (*envelope_arc).clone(),
        messages,
        unz,
    })
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
    fn test_message_chunk_struct_exists() {
        let chunk = MessageChunk {
            envelope: Arc::new(vec![]),
            unh: OwnedSegment {
                id: "UNH".to_string(),
                elements: vec![],
                segment_number: 0,
            },
            body: vec![],
            unt: OwnedSegment {
                id: "UNT".to_string(),
                elements: vec![],
                segment_number: 1,
            },
        };
        assert_eq!(chunk.unh.id, "UNH");
        assert_eq!(chunk.unt.id, "UNT");
        assert!(chunk.envelope.is_empty());
        assert!(chunk.body.is_empty());
    }

    #[test]
    fn test_interchange_chunks_struct_exists() {
        let chunks = InterchangeChunks {
            envelope: vec![],
            messages: vec![],
            unz: None,
        };
        assert!(chunks.messages.is_empty());
        assert!(chunks.unz.is_none());
    }

    #[test]
    fn test_split_messages_single_message() {
        let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'UNT+3+MSG001'UNZ+1+REF001'";
        let segments = parse_to_segments(input).unwrap();
        let chunks = split_messages(segments).unwrap();

        assert_eq!(chunks.messages.len(), 1);
        assert_eq!(chunks.envelope.len(), 1); // UNB only (UNA not emitted by parser)
        assert!(chunks.unz.is_some());

        let msg = &chunks.messages[0];
        assert!(msg.unh.is("UNH"));
        assert!(msg.unt.is("UNT"));
        assert_eq!(msg.body.len(), 1); // BGM only
        assert!(msg.body[0].is("BGM"));

        // all_segments() should reconstruct: UNB, UNH, BGM, UNT
        let all = msg.all_segments();
        assert_eq!(all.len(), 4);
        assert!(all[0].is("UNB"));
        assert!(all[1].is("UNH"));
        assert!(all[2].is("BGM"));
        assert!(all[3].is("UNT"));
    }

    #[test]
    fn test_split_messages_two_messages() {
        let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+001+UTILMD:D:11A:UN:S2.1'BGM+E01+DOC001'UNT+2+001'UNH+002+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC002'DTM+137:20250101:102'UNT+3+002'UNZ+2+REF001'";
        let segments = parse_to_segments(input).unwrap();
        let chunks = split_messages(segments).unwrap();

        assert_eq!(chunks.messages.len(), 2);

        // First message: UNH, BGM, UNT
        let msg1 = &chunks.messages[0];
        assert_eq!(msg1.unh.get_element(0), "001");
        assert_eq!(msg1.body.len(), 1);
        assert!(msg1.body[0].is("BGM"));

        // Second message: UNH, BGM, DTM, UNT
        let msg2 = &chunks.messages[1];
        assert_eq!(msg2.unh.get_element(0), "002");
        assert_eq!(msg2.body.len(), 2);
        assert!(msg2.body[0].is("BGM"));
        assert!(msg2.body[1].is("DTM"));

        // Both messages share the same envelope
        assert_eq!(msg1.envelope.len(), msg2.envelope.len());
        assert!(msg1.envelope[0].is("UNB"));
    }

    #[test]
    fn test_split_messages_envelope_preserved_per_message() {
        // Each message's all_segments() should start with envelope
        let input = b"UNA:+.? 'UNB+UNOC:3+SEND+RECV+210101:1200+REF'UNH+001+UTILMD:D:11A:UN:S2.1'UNT+1+001'UNH+002+UTILMD:D:11A:UN:S2.1'UNT+1+002'UNZ+2+REF'";
        let segments = parse_to_segments(input).unwrap();
        let chunks = split_messages(segments).unwrap();

        for msg in &chunks.messages {
            let all = msg.all_segments();
            assert!(all[0].is("UNB"), "First segment should be UNB");
            assert!(all[1].is("UNH"), "Second segment should be UNH");
            assert!(all.last().unwrap().is("UNT"), "Last segment should be UNT");
        }
    }

    #[test]
    fn test_split_messages_no_messages_errors() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R+210101:1200+REF'UNZ+0+REF'";
        let segments = parse_to_segments(input).unwrap();
        let result = split_messages(segments);
        assert!(result.is_err());
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
