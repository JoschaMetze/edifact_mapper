#[cfg(test)]
use edifact_types::Control;
use edifact_types::{EdifactDelimiters, RawSegment, SegmentPosition};

use crate::error::ParseError;
use crate::handler::EdifactHandler;
use crate::segment_builder::SegmentBuilder as SegBuilder;
use crate::tokenizer::EdifactTokenizer;

/// Streaming EDIFACT parser.
///
/// Parses a byte slice by tokenizing it into segments and routing them
/// to an `EdifactHandler`. Service segments (UNB, UNH, UNT, UNZ) are
/// dispatched to specific handler methods in addition to `on_segment()`.
pub struct EdifactStreamParser;

impl EdifactStreamParser {
    /// Parse an EDIFACT interchange from a byte slice.
    ///
    /// This is the main synchronous entry point. It:
    /// 1. Detects UNA and determines delimiters
    /// 2. Tokenizes input into segments
    /// 3. Routes each segment to the handler
    /// 4. Stops if the handler returns `Control::Stop`
    pub fn parse(input: &[u8], handler: &mut dyn EdifactHandler) -> Result<(), ParseError> {
        // Step 1: Detect delimiters
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        handler.on_delimiters(&delimiters, has_una);

        // Step 2: Determine where actual content starts (after UNA if present)
        let content_start = if has_una { 9 } else { 0 };
        let content = &input[content_start..];

        // Step 3: Tokenize and process segments
        let tokenizer = EdifactTokenizer::new(delimiters);
        let seg_builder = SegBuilder::new(delimiters);

        let mut segment_number: u32 = 0;
        let mut message_number: u32 = 0;
        let mut byte_offset = content_start;

        for segment_str in tokenizer.tokenize_segments(content) {
            segment_number += 1;

            let position = SegmentPosition::new(segment_number, byte_offset, message_number);

            let Some(raw_segment) = seg_builder.build(segment_str, position) else {
                byte_offset += segment_str.len() + 1; // +1 for terminator
                continue;
            };

            // Skip UNA segments in content
            if raw_segment.is("UNA") {
                byte_offset += segment_str.len() + 1;
                segment_number -= 1; // don't count UNA
                continue;
            }

            let id_upper = raw_segment.id.to_ascii_uppercase();

            // Track message numbering
            if id_upper == "UNH" {
                message_number += 1;
            }

            // Rebuild position with correct message number
            let effective_message_number = if id_upper == "UNB" || id_upper == "UNZ" {
                0
            } else {
                message_number
            };
            let position =
                SegmentPosition::new(segment_number, byte_offset, effective_message_number);
            let raw_segment = RawSegment::new(raw_segment.id, raw_segment.elements, position);

            // Route service segments
            match id_upper.as_str() {
                "UNB" => {
                    if handler.on_interchange_start(&raw_segment).should_stop() {
                        return Ok(());
                    }
                }
                "UNH" => {
                    if handler.on_message_start(&raw_segment).should_stop() {
                        return Ok(());
                    }
                }
                "UNT" => {
                    handler.on_message_end(&raw_segment);
                }
                "UNZ" => {
                    handler.on_interchange_end(&raw_segment);
                }
                _ => {}
            }

            // Always call on_segment
            if handler.on_segment(&raw_segment).should_stop() {
                return Ok(());
            }

            byte_offset += segment_str.len() + 1; // +1 for terminator
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Handler that collects all event names in order.
    struct EventCollector {
        events: RefCell<Vec<String>>,
    }

    impl EventCollector {
        fn new() -> Self {
            Self {
                events: RefCell::new(Vec::new()),
            }
        }

        fn events(&self) -> Vec<String> {
            self.events.borrow().clone()
        }
    }

    impl EdifactHandler for EventCollector {
        fn on_delimiters(&mut self, _d: &EdifactDelimiters, explicit_una: bool) {
            self.events
                .borrow_mut()
                .push(format!("DELIMITERS(una={})", explicit_una));
        }

        fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
            self.events
                .borrow_mut()
                .push(format!("INTERCHANGE_START({})", unb.id));
            Control::Continue
        }

        fn on_message_start(&mut self, unh: &RawSegment) -> Control {
            self.events
                .borrow_mut()
                .push(format!("MESSAGE_START(ref={})", unh.get_element(0)));
            Control::Continue
        }

        fn on_segment(&mut self, seg: &RawSegment) -> Control {
            self.events
                .borrow_mut()
                .push(format!("SEGMENT({})", seg.id));
            Control::Continue
        }

        fn on_message_end(&mut self, _unt: &RawSegment) {
            self.events.borrow_mut().push("MESSAGE_END".to_string());
        }

        fn on_interchange_end(&mut self, _unz: &RawSegment) {
            self.events.borrow_mut().push("INTERCHANGE_END".to_string());
        }
    }

    #[test]
    fn test_parse_minimal_interchange() {
        let input = b"UNA:+.? 'UNB+UNOC:3+SENDER+RECEIVER+210101:1200+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'UNT+3+MSG001'UNZ+1+REF001'";

        let mut handler = EventCollector::new();
        EdifactStreamParser::parse(input, &mut handler).unwrap();

        let events = handler.events();
        assert_eq!(events[0], "DELIMITERS(una=true)");
        assert_eq!(events[1], "INTERCHANGE_START(UNB)");
        assert_eq!(events[2], "SEGMENT(UNB)");
        assert_eq!(events[3], "MESSAGE_START(ref=MSG001)");
        assert_eq!(events[4], "SEGMENT(UNH)");
        assert_eq!(events[5], "SEGMENT(BGM)");
        assert_eq!(events[6], "MESSAGE_END");
        assert_eq!(events[7], "SEGMENT(UNT)");
        assert_eq!(events[8], "INTERCHANGE_END");
        assert_eq!(events[9], "SEGMENT(UNZ)");
    }

    #[test]
    fn test_parse_without_una() {
        let input = b"UNB+UNOC:3+SENDER+RECEIVER'UNZ+0+REF'";

        let mut handler = EventCollector::new();
        EdifactStreamParser::parse(input, &mut handler).unwrap();

        let events = handler.events();
        assert_eq!(events[0], "DELIMITERS(una=false)");
        assert_eq!(events[1], "INTERCHANGE_START(UNB)");
    }

    #[test]
    fn test_parse_handler_stops_early() {
        struct StopOnBgm {
            segments_seen: Vec<String>,
        }
        impl EdifactHandler for StopOnBgm {
            fn on_segment(&mut self, seg: &RawSegment) -> Control {
                self.segments_seen.push(seg.id.to_string());
                if seg.is("BGM") {
                    Control::Stop
                } else {
                    Control::Continue
                }
            }
        }

        let input = b"UNA:+.? 'UNB+UNOC:3'UNH+001'BGM+E03'DTM+137:20250101'UNT+3+001'UNZ+1'";
        let mut handler = StopOnBgm {
            segments_seen: Vec::new(),
        };
        EdifactStreamParser::parse(input, &mut handler).unwrap();

        // Should have seen UNB, UNH, BGM but NOT DTM, UNT, UNZ
        assert_eq!(handler.segments_seen, vec!["UNB", "UNH", "BGM"]);
    }

    #[test]
    fn test_parse_message_numbering() {
        struct PositionTracker {
            positions: Vec<(String, u32)>,
        }
        impl EdifactHandler for PositionTracker {
            fn on_segment(&mut self, seg: &RawSegment) -> Control {
                self.positions
                    .push((seg.id.to_string(), seg.position.message_number));
                Control::Continue
            }
        }

        let input =
            b"UNA:+.? 'UNB+UNOC:3'UNH+001'BGM+E03'UNT+2+001'UNH+002'BGM+E03'UNT+2+002'UNZ+2'";
        let mut handler = PositionTracker {
            positions: Vec::new(),
        };
        EdifactStreamParser::parse(input, &mut handler).unwrap();

        // UNB is outside messages (message_number=0)
        assert_eq!(handler.positions[0], ("UNB".to_string(), 0));
        // First message
        assert_eq!(handler.positions[1], ("UNH".to_string(), 1));
        assert_eq!(handler.positions[2], ("BGM".to_string(), 1));
        assert_eq!(handler.positions[3], ("UNT".to_string(), 1));
        // Second message
        assert_eq!(handler.positions[4], ("UNH".to_string(), 2));
        assert_eq!(handler.positions[5], ("BGM".to_string(), 2));
        assert_eq!(handler.positions[6], ("UNT".to_string(), 2));
        // UNZ is outside messages
        assert_eq!(handler.positions[7], ("UNZ".to_string(), 0));
    }

    #[test]
    fn test_parse_empty_input() {
        struct NoOp;
        impl EdifactHandler for NoOp {}

        let mut handler = NoOp;
        let result = EdifactStreamParser::parse(b"", &mut handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_real_world_dtm_with_timezone() {
        struct DtmCollector {
            dtm_values: Vec<String>,
        }
        impl EdifactHandler for DtmCollector {
            fn on_segment(&mut self, seg: &RawSegment) -> Control {
                if seg.is("DTM") {
                    let qualifier = seg.get_component(0, 0);
                    let value = seg.get_component(0, 1);
                    self.dtm_values.push(format!("{}={}", qualifier, value));
                }
                Control::Continue
            }
        }

        let input = b"UNA:+.? 'UNB+UNOC:3'UNH+001'DTM+137:202506190130?+00:303'UNT+2+001'UNZ+1'";
        let mut handler = DtmCollector {
            dtm_values: Vec::new(),
        };
        EdifactStreamParser::parse(input, &mut handler).unwrap();

        assert_eq!(handler.dtm_values.len(), 1);
        assert_eq!(handler.dtm_values[0], "137=202506190130?+00");
    }

    mod fuzz {
        use super::*;
        use proptest::prelude::*;

        /// A handler that does nothing but exercises all callbacks.
        struct FuzzHandler {
            segment_count: usize,
        }

        impl EdifactHandler for FuzzHandler {
            fn on_delimiters(&mut self, _d: &EdifactDelimiters, _una: bool) {}

            fn on_interchange_start(&mut self, _unb: &RawSegment) -> Control {
                Control::Continue
            }

            fn on_message_start(&mut self, _unh: &RawSegment) -> Control {
                Control::Continue
            }

            fn on_segment(&mut self, _seg: &RawSegment) -> Control {
                self.segment_count += 1;
                if self.segment_count > 10_000 {
                    Control::Stop // safety valve for huge inputs
                } else {
                    Control::Continue
                }
            }

            fn on_message_end(&mut self, _unt: &RawSegment) {}
            fn on_interchange_end(&mut self, _unz: &RawSegment) {}

            fn on_error(&mut self, _error: ParseError) -> Control {
                Control::Continue // try to keep going
            }
        }

        proptest! {
            #[test]
            fn parser_never_panics_on_arbitrary_input(input in proptest::collection::vec(any::<u8>(), 0..1024)) {
                let mut handler = FuzzHandler { segment_count: 0 };
                // Must not panic — errors are OK, panics are NOT
                let _ = EdifactStreamParser::parse(&input, &mut handler);
            }

            #[test]
            fn parser_never_panics_on_ascii_input(input in "[A-Z0-9:+.?' \n\r]{0,512}") {
                let mut handler = FuzzHandler { segment_count: 0 };
                let _ = EdifactStreamParser::parse(input.as_bytes(), &mut handler);
            }

            #[test]
            fn parser_handles_valid_looking_messages(
                sender in "[A-Z0-9]{10,13}",
                receiver in "[A-Z0-9]{10,13}",
                ref_num in "[A-Z0-9]{5,10}",
            ) {
                let msg = format!(
                    "UNA:+.? 'UNB+UNOC:3+{}+{}+210101:1200+{}'UNZ+0+{}'",
                    sender, receiver, ref_num, ref_num,
                );
                let mut handler = FuzzHandler { segment_count: 0 };
                let result = EdifactStreamParser::parse(msg.as_bytes(), &mut handler);
                prop_assert!(result.is_ok());
                prop_assert!(handler.segment_count >= 2); // at least UNB and UNZ
            }
        }
    }
}
