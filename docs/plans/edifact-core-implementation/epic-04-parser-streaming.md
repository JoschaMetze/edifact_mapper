---
feature: edifact-core-implementation
epic: 4
title: "edifact-parser — Streaming Parser & Handler"
depends_on: [edifact-core-implementation/E03]
estimated_tasks: 5
crate: edifact-parser
---

# Epic 4: edifact-parser — Streaming Parser & Handler

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/edifact-parser/src/`. All code must compile with `cargo check -p edifact-parser`.

**Goal:** Implement the SAX-style streaming parser with the `EdifactHandler` trait, service segment routing, `ParseError` enum, integration tests with real EDIFACT fixtures, and proptest fuzzing.

**Architecture:** The `EdifactHandler` trait defines callback methods with default no-op implementations. `EdifactStreamParser::parse()` tokenizes the input, builds `RawSegment` instances, routes service segments (UNB, UNZ, UNH, UNT) to specific handler methods, and calls `on_segment()` for every segment. The handler returns `Control` to stop early. This mirrors the C# `EdifactStreamParser` + `IEdifactHandler` pattern. See design doc section 3.

**Tech Stack:** Rust, edifact-types, thiserror, proptest, test-case

---

## Task 1: ParseError Enum

**Files:**
- Create: `crates/edifact-parser/src/error.rs`
- Modify: `crates/edifact-parser/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-parser/src/error.rs`:

```rust
use edifact_types::SegmentPosition;

/// Errors that can occur during EDIFACT parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The UNA service string advice header is invalid.
    #[error("invalid UNA header at byte {offset}")]
    InvalidUna { offset: usize },

    /// A segment was not properly terminated.
    #[error("unterminated segment at byte {offset}")]
    UnterminatedSegment { offset: usize },

    /// The input ended unexpectedly.
    #[error("unexpected end of input")]
    UnexpectedEof,

    /// The input contains invalid UTF-8.
    #[error("invalid UTF-8 at byte {offset}: {source}")]
    InvalidUtf8 {
        offset: usize,
        #[source]
        source: std::str::Utf8Error,
    },

    /// A segment ID could not be determined.
    #[error("empty segment ID at byte {offset}")]
    EmptySegmentId { offset: usize },

    /// Handler returned Control::Stop.
    #[error("parsing stopped by handler at {position}")]
    StoppedByHandler { position: SegmentPosition },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display_invalid_una() {
        let err = ParseError::InvalidUna { offset: 0 };
        assert_eq!(err.to_string(), "invalid UNA header at byte 0");
    }

    #[test]
    fn test_parse_error_display_unterminated() {
        let err = ParseError::UnterminatedSegment { offset: 42 };
        assert_eq!(err.to_string(), "unterminated segment at byte 42");
    }

    #[test]
    fn test_parse_error_display_unexpected_eof() {
        let err = ParseError::UnexpectedEof;
        assert_eq!(err.to_string(), "unexpected end of input");
    }

    #[test]
    fn test_parse_error_display_stopped() {
        let err = ParseError::StoppedByHandler {
            position: SegmentPosition::new(3, 100, 1),
        };
        assert_eq!(
            err.to_string(),
            "parsing stopped by handler at segment 3 at byte 100 (message 1)"
        );
    }

    #[test]
    fn test_parse_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // ParseError contains Utf8Error which is Send+Sync
        // This ensures our error type can be used across threads
        assert_send_sync::<ParseError>();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-parser test_parse_error`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the file. Update `crates/edifact-parser/src/lib.rs`:

```rust
//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.
//!
//! This crate provides a standalone EDIFACT parser with no BO4E dependency.
//! It can be used by anyone in the Rust ecosystem for generic EDIFACT parsing.

mod error;
mod segment_builder;
mod tokenizer;

pub use error::ParseError;
pub use segment_builder::SegmentBuilder;
pub use tokenizer::EdifactTokenizer;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-parser test_parse_error`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
feat(edifact-parser): add ParseError enum with thiserror

Typed error variants for UNA parsing, unterminated segments,
unexpected EOF, invalid UTF-8, and handler-initiated stops.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: EdifactHandler Trait

**Files:**
- Create: `crates/edifact-parser/src/handler.rs`
- Modify: `crates/edifact-parser/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-parser/src/handler.rs`:

```rust
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
            self.events.push(format!("delimiters(una={})", explicit_una));
        }

        fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
            self.events
                .push(format!("interchange_start({})", unb.id));
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-parser test_default_handler`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the file. Update `crates/edifact-parser/src/lib.rs`:

```rust
//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.

mod error;
mod handler;
mod segment_builder;
mod tokenizer;

pub use error::ParseError;
pub use handler::EdifactHandler;
pub use segment_builder::SegmentBuilder;
pub use tokenizer::EdifactTokenizer;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-parser`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
feat(edifact-parser): add EdifactHandler trait with default impls

SAX-style handler with callbacks for delimiters, interchange start/end,
message start/end, and per-segment processing. All methods have
default no-op implementations.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: EdifactStreamParser::parse() — Main Entry Point

**Files:**
- Create: `crates/edifact-parser/src/parser.rs`
- Modify: `crates/edifact-parser/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-parser/src/parser.rs`:

```rust
use edifact_types::{Control, EdifactDelimiters, RawSegment, SegmentPosition};

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
            let raw_segment = RawSegment::new(
                raw_segment.id,
                raw_segment.elements,
                position,
            );

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
            self.events.borrow_mut().push(format!("SEGMENT({})", seg.id));
            Control::Continue
        }

        fn on_message_end(&mut self, _unt: &RawSegment) {
            self.events.borrow_mut().push("MESSAGE_END".to_string());
        }

        fn on_interchange_end(&mut self, _unz: &RawSegment) {
            self.events
                .borrow_mut()
                .push("INTERCHANGE_END".to_string());
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

        let input = b"UNA:+.? 'UNB+UNOC:3'UNH+001'BGM+E03'UNT+2+001'UNH+002'BGM+E03'UNT+2+002'UNZ+2'";
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
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-parser test_parse_minimal_interchange`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the file. Update `crates/edifact-parser/src/lib.rs`:

```rust
//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.

mod error;
mod handler;
mod parser;
mod segment_builder;
mod tokenizer;

pub use error::ParseError;
pub use handler::EdifactHandler;
pub use parser::EdifactStreamParser;
pub use segment_builder::SegmentBuilder;
pub use tokenizer::EdifactTokenizer;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-parser`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
feat(edifact-parser): add EdifactStreamParser with service segment routing

Main parse() entry point that detects UNA, tokenizes segments, and
routes UNB/UNH/UNT/UNZ to specific handler callbacks. Tracks message
numbering across multiple messages in an interchange.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Proptest Fuzzing for Parser Robustness

**Files:**
- Modify: `crates/edifact-parser/src/parser.rs`

**Step 1: Write the proptest**

Add to the tests in `crates/edifact-parser/src/parser.rs`:

```rust
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
```

**Step 2: Run proptest**

Run: `cargo test -p edifact-parser fuzz -- --nocapture`
Expected: PASS — no panics on any generated input.

**Step 3: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
test(edifact-parser): add proptest fuzzing for parser robustness

Property-based tests verify the parser never panics on arbitrary
byte input, random ASCII, or valid-looking EDIFACT messages.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Integration Test with Real EDIFACT Fixture

**Files:**
- Create: `crates/edifact-parser/tests/integration_parse.rs`

**Step 1: Write the integration test**

Create `crates/edifact-parser/tests/integration_parse.rs`:

```rust
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
```

**Step 2: Run integration test**

Run: `cargo test -p edifact-parser --test integration_parse`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/edifact-parser/tests/
git commit -m "$(cat <<'EOF'
test(edifact-parser): add integration tests for streaming parser

Tests a synthetic UTILMD message and a multi-message interchange.
Verifies event ordering, segment counting, and message numbering.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```
