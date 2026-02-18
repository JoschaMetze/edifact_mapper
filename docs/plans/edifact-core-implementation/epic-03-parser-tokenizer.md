---
feature: edifact-core-implementation
epic: 3
title: "edifact-parser — Tokenizer & UNA"
depends_on: [edifact-core-implementation/E02]
estimated_tasks: 3
crate: edifact-parser
---

# Epic 3: edifact-parser — Tokenizer & UNA

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/edifact-parser/src/`. All code must compile with `cargo check -p edifact-parser`.

**Goal:** Implement the EDIFACT tokenizer that splits raw byte input into segments, elements, and components, handling UNA detection, release character escaping, and whitespace normalization.

**Architecture:** The tokenizer operates on `&[u8]` input and produces `RawSegment<'a>` references that borrow directly from the input. It first detects UNA to determine delimiters, then splits on segment terminators (respecting release character escaping), then splits each segment into elements and components. This mirrors the C# `EdifactTokenizer` class. See design doc section 3.

**Tech Stack:** Rust, edifact-types crate, thiserror for errors

---

## Task 1: Segment Tokenizer — Split on Segment Terminator

**Files:**
- Create: `crates/edifact-parser/src/tokenizer.rs`
- Modify: `crates/edifact-parser/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-parser/src/tokenizer.rs`:

```rust
use edifact_types::EdifactDelimiters;

/// Tokenizes raw EDIFACT byte input into segment strings.
///
/// Handles release character escaping, whitespace normalization (strips \r\n),
/// and UNA segment detection.
pub struct EdifactTokenizer {
    delimiters: EdifactDelimiters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_segments_simple() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"UNB+UNOC:3'UNH+00001'UNT+2+00001'UNZ+1'";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments, vec!["UNB+UNOC:3", "UNH+00001", "UNT+2+00001", "UNZ+1"]);
    }

    #[test]
    fn test_tokenize_segments_with_newlines() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"UNB+UNOC:3'\nUNH+00001'\r\nUNT+2+00001'\nUNZ+1'";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments, vec!["UNB+UNOC:3", "UNH+00001", "UNT+2+00001", "UNZ+1"]);
    }

    #[test]
    fn test_tokenize_segments_with_release_char() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        // ?'  is an escaped apostrophe — NOT a segment terminator
        let input = b"FTX+ACB+++text with ?'quotes?''";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "FTX+ACB+++text with ?'quotes?'");
    }

    #[test]
    fn test_tokenize_segments_empty_input() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_tokenize_segments_trailing_whitespace() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"UNH+00001'  \n  ";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments, vec!["UNH+00001"]);
    }

    #[test]
    fn test_tokenize_segments_custom_delimiter() {
        let delimiters = EdifactDelimiters {
            segment: b'!',
            ..EdifactDelimiters::default()
        };
        let tokenizer = EdifactTokenizer::new(delimiters);
        let input = b"UNB+UNOC:3!UNH+00001!";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments, vec!["UNB+UNOC:3", "UNH+00001"]);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-parser test_tokenize_segments`
Expected: FAIL — `EdifactTokenizer::new` and `tokenize_segments` do not exist.

**Step 3: Write minimal implementation**

Complete the implementation in `crates/edifact-parser/src/tokenizer.rs`:

```rust
impl EdifactTokenizer {
    /// Creates a new tokenizer with the given delimiters.
    pub fn new(delimiters: EdifactDelimiters) -> Self {
        Self { delimiters }
    }

    /// Returns the delimiters used by this tokenizer.
    pub fn delimiters(&self) -> &EdifactDelimiters {
        &self.delimiters
    }

    /// Tokenizes EDIFACT input into segment strings.
    ///
    /// Splits on segment terminator, respecting release character escaping.
    /// Strips `\r` and `\n` characters from the input (EDIFACT uses them
    /// only for readability).
    ///
    /// Each yielded string is a segment WITHOUT its terminator character.
    pub fn tokenize_segments<'a>(&self, input: &'a [u8]) -> SegmentIter<'a> {
        SegmentIter {
            input,
            pos: 0,
            segment_terminator: self.delimiters.segment,
            release_char: self.delimiters.release,
        }
    }

    /// Tokenizes a segment string into data elements.
    ///
    /// Splits on element separator, preserving release character escaping
    /// (unescaping happens at the component level).
    pub fn tokenize_elements<'a>(&self, segment: &'a str) -> ElementIter<'a> {
        ElementIter {
            input: segment,
            pos: 0,
            separator: self.delimiters.element as char,
            release: self.delimiters.release as char,
        }
    }

    /// Tokenizes a data element into components.
    ///
    /// Splits on component separator and unescapes release character sequences.
    pub fn tokenize_components<'a>(&self, element: &'a str) -> ComponentIter<'a> {
        ComponentIter {
            input: element,
            pos: 0,
            separator: self.delimiters.component as char,
            release: self.delimiters.release as char,
        }
    }
}

/// Iterator over segments in raw EDIFACT input bytes.
pub struct SegmentIter<'a> {
    input: &'a [u8],
    pos: usize,
    segment_terminator: u8,
    release_char: u8,
}

impl<'a> Iterator for SegmentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespace between segments
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if b == b'\r' || b == b'\n' || b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos >= self.input.len() {
            return None;
        }

        let start = self.pos;
        let mut i = self.pos;

        while i < self.input.len() {
            let b = self.input[i];

            // Skip \r and \n within segments (EDIFACT ignores them)
            if b == b'\r' || b == b'\n' {
                i += 1;
                continue;
            }

            // Check for release character — next byte is escaped
            if b == self.release_char && i + 1 < self.input.len() {
                i += 2; // skip release char and the escaped char
                continue;
            }

            if b == self.segment_terminator {
                // Found unescaped terminator
                let segment_bytes = &self.input[start..i];
                self.pos = i + 1;

                // Build segment string, stripping \r and \n
                let segment_str = strip_crlf(segment_bytes);
                if segment_str.is_empty() {
                    return self.next(); // skip empty segments
                }
                return Some(segment_str);
            }

            i += 1;
        }

        // Remaining content after last terminator (may be trailing whitespace)
        if start < self.input.len() {
            let segment_bytes = &self.input[start..];
            self.pos = self.input.len();
            let segment_str = strip_crlf(segment_bytes);
            if segment_str.is_empty() {
                return None;
            }
            return Some(segment_str);
        }

        None
    }
}

/// Converts a byte slice to a string, stripping \r and \n characters.
///
/// Returns the resulting &str. If the input has no CR/LF, returns a direct
/// reference. Otherwise we cannot return a zero-copy reference, so this is
/// a limitation — for segments containing embedded newlines, we need the
/// caller to handle it.
///
/// In practice, EDIFACT segments never contain embedded newlines as data
/// (they are only used as line separators between segments for readability).
/// So we can safely interpret the bytes as UTF-8 and trim.
fn strip_crlf(bytes: &[u8]) -> &str {
    // Fast path: try to interpret as UTF-8 and trim
    let s = std::str::from_utf8(bytes).unwrap_or("");
    s.trim_matches(|c: char| c == '\r' || c == '\n')
}

/// Iterator over elements within a segment string.
pub struct ElementIter<'a> {
    input: &'a str,
    pos: usize,
    separator: char,
    release: char,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.input.len() {
            return None;
        }

        let start = self.pos;
        let bytes = self.input.as_bytes();
        let mut i = self.pos;

        while i < bytes.len() {
            let ch = bytes[i] as char;

            // Release character escapes the next character
            if ch == self.release && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            if ch == self.separator {
                let element = &self.input[start..i];
                self.pos = i + 1;
                return Some(element);
            }

            i += 1;
        }

        // Return remaining content
        if start <= self.input.len() {
            let element = &self.input[start..];
            self.pos = self.input.len() + 1; // mark as exhausted
            return Some(element);
        }

        None
    }
}

/// Iterator over components within a data element.
pub struct ComponentIter<'a> {
    input: &'a str,
    pos: usize,
    separator: char,
    release: char,
}

impl<'a> Iterator for ComponentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.input.len() {
            return None;
        }

        let start = self.pos;
        let bytes = self.input.as_bytes();
        let mut i = self.pos;

        while i < bytes.len() {
            let ch = bytes[i] as char;

            // Release character escapes the next character
            if ch == self.release && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            if ch == self.separator {
                let component = &self.input[start..i];
                self.pos = i + 1;
                return Some(component);
            }

            i += 1;
        }

        // Return remaining content
        if start <= self.input.len() {
            let component = &self.input[start..];
            self.pos = self.input.len() + 1;
            return Some(component);
        }

        None
    }
}
```

Update `crates/edifact-parser/src/lib.rs`:

```rust
//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.
//!
//! This crate provides a standalone EDIFACT parser with no BO4E dependency.
//! It can be used by anyone in the Rust ecosystem for generic EDIFACT parsing.

mod tokenizer;

pub use tokenizer::EdifactTokenizer;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-parser test_tokenize_segments`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
feat(edifact-parser): add segment tokenizer with release char escaping

Splits raw bytes into segments respecting the release character.
Strips CR/LF between segments. Handles custom delimiters.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Element and Component Splitting

**Files:**
- Modify: `crates/edifact-parser/src/tokenizer.rs`

**Step 1: Write the failing test**

Add to the `tests` module in `crates/edifact-parser/src/tokenizer.rs`:

```rust
    #[test]
    fn test_tokenize_elements() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let elements: Vec<&str> = tokenizer.tokenize_elements("NAD+Z04+9900123000002:500").collect();
        assert_eq!(elements, vec!["NAD", "Z04", "9900123000002:500"]);
    }

    #[test]
    fn test_tokenize_elements_escaped_plus() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let elements: Vec<&str> = tokenizer.tokenize_elements("FTX+ACB+++value with ?+plus").collect();
        // ?+ is escaped, so it should NOT split
        assert_eq!(elements, vec!["FTX", "ACB", "", "value with ?+plus"]);
    }

    #[test]
    fn test_tokenize_components() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let components: Vec<&str> = tokenizer.tokenize_components("UTILMD:D:11A:UN:S2.1").collect();
        assert_eq!(components, vec!["UTILMD", "D", "11A", "UN", "S2.1"]);
    }

    #[test]
    fn test_tokenize_components_escaped_colon() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let components: Vec<&str> = tokenizer.tokenize_components("value?:with:colon").collect();
        // ?: is escaped, so "value?:with" is one component
        assert_eq!(components, vec!["value?:with", "colon"]);
    }

    #[test]
    fn test_tokenize_components_empty() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let components: Vec<&str> = tokenizer.tokenize_components("Z04::500").collect();
        assert_eq!(components, vec!["Z04", "", "500"]);
    }

    #[test]
    fn test_full_tokenization_pipeline() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"NAD+Z04+9900123000002::293'DTM+137:202501010000?+01:303'";

        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments.len(), 2);

        // Parse first segment: NAD+Z04+9900123000002::293
        let elements: Vec<&str> = tokenizer.tokenize_elements(segments[0]).collect();
        assert_eq!(elements, vec!["NAD", "Z04", "9900123000002::293"]);

        // Parse composite element: 9900123000002::293
        let components: Vec<&str> = tokenizer.tokenize_components(elements[2]).collect();
        assert_eq!(components, vec!["9900123000002", "", "293"]);

        // Parse second segment: DTM+137:202501010000?+01:303
        let dtm_elements: Vec<&str> = tokenizer.tokenize_elements(segments[1]).collect();
        assert_eq!(dtm_elements, vec!["DTM", "137:202501010000?+01:303"]);

        // Parse DTM composite (note: ?+ is escaped at element level, kept as-is)
        let dtm_components: Vec<&str> =
            tokenizer.tokenize_components(dtm_elements[1]).collect();
        assert_eq!(dtm_components, vec!["137", "202501010000?+01", "303"]);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-parser test_tokenize_elements`
Expected: These should already pass since we implemented the iterators in Task 1. If not, debug.

**Step 3: Verify all pass**

Run: `cargo test -p edifact-parser`
Expected: PASS — all tokenizer tests pass.

**Step 4: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
test(edifact-parser): add element/component tokenization tests

Verifies the full tokenization pipeline: segments -> elements -> components.
Tests escaped delimiters, empty components, and real DTM+137 patterns.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Segment Builder — Convert Raw Tokens to RawSegment

**Files:**
- Create: `crates/edifact-parser/src/segment_builder.rs`
- Modify: `crates/edifact-parser/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-parser/src/segment_builder.rs`:

```rust
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
    pub fn build<'a>(&self, segment_str: &'a str, position: SegmentPosition) -> Option<RawSegment<'a>> {
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
            let components: Vec<&'a str> = self.tokenizer.tokenize_components(element_str).collect();
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
        let seg = builder.build("UNH+00001+UTILMD:D:11A:UN:S2.1", pos(1, 0)).unwrap();

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
        let seg = builder.build("NAD+Z04+9900123000002::293", pos(5, 100)).unwrap();

        assert_eq!(seg.id, "NAD");
        assert_eq!(seg.get_element(0), "Z04");
        assert_eq!(seg.get_component(1, 0), "9900123000002");
        assert_eq!(seg.get_component(1, 1), "");
        assert_eq!(seg.get_component(1, 2), "293");
    }

    #[test]
    fn test_build_dtm_with_escaped_plus() {
        let builder = SegmentBuilder::new(EdifactDelimiters::default());
        let seg = builder.build("DTM+137:202501010000?+01:303", pos(3, 50)).unwrap();

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
        let seg = builder.build("LOC+Z16+DE00014545768S0000000000000003054", pos(8, 200)).unwrap();

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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-parser test_build_simple_segment`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the file above. Update `crates/edifact-parser/src/lib.rs`:

```rust
//! Streaming EDIFACT tokenizer and SAX-style event-driven parser.
//!
//! This crate provides a standalone EDIFACT parser with no BO4E dependency.
//! It can be used by anyone in the Rust ecosystem for generic EDIFACT parsing.

mod segment_builder;
mod tokenizer;

pub use segment_builder::SegmentBuilder;
pub use tokenizer::EdifactTokenizer;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-parser`
Expected: PASS — all tests pass.

**Step 5: Commit**

```bash
git add crates/edifact-parser/
git commit -m "$(cat <<'EOF'
feat(edifact-parser): add SegmentBuilder to convert tokens to RawSegment

Parses raw segment strings into structured RawSegment instances with
segment ID, elements (split on +), and components (split on :).
Preserves release character escaping in component data.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 20 |
| Passed | 20 |
| Failed | 0 |
| Skipped | 0 |

Files tested:
- `crates/edifact-parser/src/tokenizer.rs` (12 tests: segment splitting, element splitting, component splitting, release char escaping, custom delimiters, full pipeline)
- `crates/edifact-parser/src/segment_builder.rs` (8 tests: simple/composite segments, escaped delimiters, position preservation, edge cases)
