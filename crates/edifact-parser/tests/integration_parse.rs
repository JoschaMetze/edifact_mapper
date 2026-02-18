//! Integration tests for the EDIFACT parser using real fixture files.

use edifact_parser::{EdifactHandler, EdifactStreamParser};
use edifact_types::{Control, EdifactDelimiters, RawSegment};

/// Handler that counts segments and messages.
struct CountingHandler {
    delimiter_calls: usize,
    interchange_starts: usize,
    interchange_ends: usize,
    message_starts: usize,
    message_ends: usize,
    total_segments: usize,
    segment_ids: Vec<String>,
    has_explicit_una: bool,
}

impl CountingHandler {
    fn new() -> Self {
        Self {
            delimiter_calls: 0,
            interchange_starts: 0,
            interchange_ends: 0,
            message_starts: 0,
            message_ends: 0,
            total_segments: 0,
            segment_ids: Vec::new(),
            has_explicit_una: false,
        }
    }
}

impl EdifactHandler for CountingHandler {
    fn on_delimiters(&mut self, _d: &EdifactDelimiters, explicit_una: bool) {
        self.delimiter_calls += 1;
        self.has_explicit_una = explicit_una;
    }

    fn on_interchange_start(&mut self, _unb: &RawSegment) -> Control {
        self.interchange_starts += 1;
        Control::Continue
    }

    fn on_interchange_end(&mut self, _unz: &RawSegment) {
        self.interchange_ends += 1;
    }

    fn on_message_start(&mut self, _unh: &RawSegment) -> Control {
        self.message_starts += 1;
        Control::Continue
    }

    fn on_message_end(&mut self, _unt: &RawSegment) {
        self.message_ends += 1;
    }

    fn on_segment(&mut self, seg: &RawSegment) -> Control {
        self.total_segments += 1;
        self.segment_ids.push(seg.id.to_string());
        Control::Continue
    }
}

#[test]
fn test_parse_synthetic_utilmd() {
    // A synthetic but realistic UTILMD message
    let input = br#"UNA:+.? '
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+GEN0001'
UNH+GEN0001MSG+UTILMD:D:11A:UN:S2.1'
BGM+E03+DOC001'
DTM+137:202506190130?+00:303'
NAD+MS+9900123000002::293'
NAD+MR+9900456000001::293'
IDE+24+TXID001'
LOC+Z16+DE00014545768S0000000000000003054'
UNT+8+GEN0001MSG'
UNZ+1+GEN0001'"#;

    let mut handler = CountingHandler::new();
    let result = EdifactStreamParser::parse(input, &mut handler);
    assert!(result.is_ok());

    assert_eq!(handler.delimiter_calls, 1);
    assert!(handler.has_explicit_una);
    assert_eq!(handler.interchange_starts, 1);
    assert_eq!(handler.interchange_ends, 1);
    assert_eq!(handler.message_starts, 1);
    assert_eq!(handler.message_ends, 1);

    // UNB + UNH + BGM + DTM + NAD + NAD + IDE + LOC + UNT + UNZ = 10 segments
    assert_eq!(handler.total_segments, 10);

    // Verify segment order
    assert_eq!(handler.segment_ids[0], "UNB");
    assert_eq!(handler.segment_ids[1], "UNH");
    assert_eq!(handler.segment_ids[2], "BGM");
    assert_eq!(handler.segment_ids[3], "DTM");
    assert_eq!(handler.segment_ids[4], "NAD");
    assert_eq!(handler.segment_ids[5], "NAD");
    assert_eq!(handler.segment_ids[6], "IDE");
    assert_eq!(handler.segment_ids[7], "LOC");
    assert_eq!(handler.segment_ids[8], "UNT");
    assert_eq!(handler.segment_ids[9], "UNZ");
}

#[test]
fn test_parse_multi_message_interchange() {
    let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN:S2.1'BGM+E03'UNT+2+001'UNH+002+UTILMD:D:11A:UN:S2.1'BGM+E03'UNT+2+002'UNZ+2'";

    let mut handler = CountingHandler::new();
    EdifactStreamParser::parse(input, &mut handler).unwrap();

    assert_eq!(handler.interchange_starts, 1);
    assert_eq!(handler.interchange_ends, 1);
    assert_eq!(handler.message_starts, 2);
    assert_eq!(handler.message_ends, 2);
}
