use edifact_types::{Control, EdifactDelimiters, RawSegment};

use crate::ParseError;

/// Trait for handling EDIFACT parsing events.
///
/// All methods have default no-op implementations, so implementors
/// only need to override the callbacks they care about.
///
/// # Event Order
///
/// For a typical EDIFACT interchange:
/// 1. `on_delimiters()` — always called first
/// 2. `on_interchange_start()` — when UNB is encountered
/// 3. `on_message_start()` — when UNH is encountered
/// 4. `on_segment()` — for EVERY segment (including UNB, UNH, UNT, UNZ)
/// 5. `on_message_end()` — when UNT is encountered
/// 6. `on_interchange_end()` — when UNZ is encountered
pub trait EdifactHandler {
    /// Called when delimiters are determined (from UNA or defaults).
    fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, _explicit_una: bool) {}

    /// Called when an interchange begins (UNB segment).
    fn on_interchange_start(&mut self, _unb: &RawSegment) -> Control {
        Control::Continue
    }

    /// Called when a message begins (UNH segment).
    fn on_message_start(&mut self, _unh: &RawSegment) -> Control {
        Control::Continue
    }

    /// Called for every segment in the interchange.
    ///
    /// This is called for ALL segments, including service segments
    /// (UNB, UNH, UNT, UNZ). The specific `on_*` methods are called
    /// BEFORE `on_segment()` for service segments.
    fn on_segment(&mut self, _segment: &RawSegment) -> Control {
        Control::Continue
    }

    /// Called when a message ends (UNT segment).
    fn on_message_end(&mut self, _unt: &RawSegment) {}

    /// Called when an interchange ends (UNZ segment).
    fn on_interchange_end(&mut self, _unz: &RawSegment) {}

    /// Called when a parsing error occurs.
    ///
    /// Return `Control::Continue` to attempt recovery, or
    /// `Control::Stop` to abort parsing.
    fn on_error(&mut self, _error: ParseError) -> Control {
        Control::Stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edifact_types::SegmentPosition;

    /// A test handler that records all events.
    struct RecordingHandler {
        events: Vec<String>,
    }

    impl RecordingHandler {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl EdifactHandler for RecordingHandler {
        fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, explicit_una: bool) {
            self.events
                .push(format!("delimiters(una={})", explicit_una));
        }

        fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
            self.events.push(format!("interchange_start({})", unb.id));
            Control::Continue
        }

        fn on_message_start(&mut self, unh: &RawSegment) -> Control {
            self.events.push(format!("message_start({})", unh.id));
            Control::Continue
        }

        fn on_segment(&mut self, segment: &RawSegment) -> Control {
            self.events.push(format!("segment({})", segment.id));
            Control::Continue
        }

        fn on_message_end(&mut self, unt: &RawSegment) {
            self.events.push(format!("message_end({})", unt.id));
        }

        fn on_interchange_end(&mut self, unz: &RawSegment) {
            self.events.push(format!("interchange_end({})", unz.id));
        }
    }

    #[test]
    fn test_default_handler_compiles() {
        struct EmptyHandler;
        impl EdifactHandler for EmptyHandler {}

        let mut handler = EmptyHandler;
        let pos = SegmentPosition::new(1, 0, 0);
        let seg = RawSegment::new("UNB", vec![], pos);

        // All defaults should work
        handler.on_delimiters(&EdifactDelimiters::default(), false);
        assert_eq!(handler.on_interchange_start(&seg), Control::Continue);
        assert_eq!(handler.on_message_start(&seg), Control::Continue);
        assert_eq!(handler.on_segment(&seg), Control::Continue);
        handler.on_message_end(&seg);
        handler.on_interchange_end(&seg);
    }

    #[test]
    fn test_recording_handler() {
        let mut handler = RecordingHandler::new();
        let pos = SegmentPosition::new(1, 0, 0);

        handler.on_delimiters(&EdifactDelimiters::default(), true);
        handler.on_interchange_start(&RawSegment::new("UNB", vec![], pos));
        handler.on_segment(&RawSegment::new("UNB", vec![], pos));

        assert_eq!(handler.events.len(), 3);
        assert_eq!(handler.events[0], "delimiters(una=true)");
        assert_eq!(handler.events[1], "interchange_start(UNB)");
        assert_eq!(handler.events[2], "segment(UNB)");
    }

    #[test]
    fn test_handler_stop_control() {
        struct StopOnSecondSegment {
            count: usize,
        }
        impl EdifactHandler for StopOnSecondSegment {
            fn on_segment(&mut self, _segment: &RawSegment) -> Control {
                self.count += 1;
                if self.count >= 2 {
                    Control::Stop
                } else {
                    Control::Continue
                }
            }
        }

        let mut handler = StopOnSecondSegment { count: 0 };
        let pos = SegmentPosition::new(1, 0, 1);

        assert_eq!(
            handler.on_segment(&RawSegment::new("BGM", vec![], pos)),
            Control::Continue
        );
        assert_eq!(
            handler.on_segment(&RawSegment::new("DTM", vec![], pos)),
            Control::Stop
        );
    }
}
