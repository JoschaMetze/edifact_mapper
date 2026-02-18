---
feature: edifact-core-implementation
epic: 2
title: "edifact-types Crate"
depends_on: [1]
estimated_tasks: 5
crate: edifact-types
---

# Epic 2: edifact-types Crate

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/edifact-types/src/`. All code must compile with `cargo check -p edifact-types`.

**Goal:** Implement the zero-dependency `edifact-types` crate with all shared EDIFACT primitive types: `EdifactDelimiters`, `SegmentPosition`, `RawSegment<'a>`, and `Control` enum.

**Architecture:** This is the leaf crate with zero external dependencies. All types are designed for zero-copy parsing — `RawSegment<'a>` borrows string slices directly from the input buffer. `EdifactDelimiters` handles both default delimiters and UNA segment parsing. See design doc section 2.

**Tech Stack:** Rust 2021 edition, no external dependencies

---

## Task 1: EdifactDelimiters with Defaults

**Files:**
- Create: `crates/edifact-types/src/delimiters.rs`
- Modify: `crates/edifact-types/src/lib.rs`

**Step 1: Write the failing test**

Add to `crates/edifact-types/src/delimiters.rs`:

```rust
/// EDIFACT delimiter characters.
///
/// The six characters that control EDIFACT message structure. When no UNA
/// service string advice is present, the standard defaults apply:
/// - Component separator: `:` (colon)
/// - Element separator: `+` (plus)
/// - Decimal mark: `.` (period)
/// - Release character: `?` (question mark)
/// - Segment terminator: `'` (apostrophe)
/// - Reserved: ` ` (space)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdifactDelimiters {
    /// Component data element separator (default: `:`).
    pub component: u8,
    /// Data element separator (default: `+`).
    pub element: u8,
    /// Decimal mark (default: `.`).
    pub decimal: u8,
    /// Release character / escape (default: `?`).
    pub release: u8,
    /// Segment terminator (default: `'`).
    pub segment: u8,
    /// Reserved for future use (default: ` `).
    pub reserved: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_delimiters() {
        let d = EdifactDelimiters::default();
        assert_eq!(d.component, b':');
        assert_eq!(d.element, b'+');
        assert_eq!(d.decimal, b'.');
        assert_eq!(d.release, b'?');
        assert_eq!(d.segment, b'\'');
        assert_eq!(d.reserved, b' ');
    }

    #[test]
    fn test_delimiters_equality() {
        let a = EdifactDelimiters::default();
        let b = EdifactDelimiters::default();
        assert_eq!(a, b);
    }

    #[test]
    fn test_delimiters_debug() {
        let d = EdifactDelimiters::default();
        let debug = format!("{:?}", d);
        assert!(debug.contains("EdifactDelimiters"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-types test_default_delimiters`
Expected: FAIL — `Default` not implemented.

**Step 3: Write minimal implementation**

Complete `crates/edifact-types/src/delimiters.rs` with the Default impl:

```rust
impl Default for EdifactDelimiters {
    fn default() -> Self {
        Self {
            component: b':',
            element: b'+',
            decimal: b'.',
            release: b'?',
            segment: b'\'',
            reserved: b' ',
        }
    }
}

impl EdifactDelimiters {
    /// Standard EDIFACT delimiters (when no UNA segment is present).
    pub const STANDARD: Self = Self {
        component: b':',
        element: b'+',
        decimal: b'.',
        release: b'?',
        segment: b'\'',
        reserved: b' ',
    };

    /// Formats the delimiters as a UNA service string advice segment.
    ///
    /// Returns the 9-byte UNA string: `UNA:+.? '`
    pub fn to_una_string(&self) -> String {
        format!(
            "UNA{}{}{}{}{}{}",
            self.component as char,
            self.element as char,
            self.decimal as char,
            self.release as char,
            self.reserved as char,
            self.segment as char,
        )
    }
}

impl std::fmt::Display for EdifactDelimiters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UNA{}{}{}{}{}{}",
            self.component as char,
            self.element as char,
            self.decimal as char,
            self.release as char,
            self.reserved as char,
            self.segment as char,
        )
    }
}
```

Update `crates/edifact-types/src/lib.rs`:

```rust
//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.

mod delimiters;

pub use delimiters::EdifactDelimiters;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-types test_default_delimiters`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-types/
git commit -m "$(cat <<'EOF'
feat(edifact-types): add EdifactDelimiters with standard defaults

Implements the six EDIFACT delimiter characters with Default trait,
Display formatting as UNA string, and STANDARD const.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: UNA Parsing for EdifactDelimiters

**Files:**
- Modify: `crates/edifact-types/src/delimiters.rs`

**Step 1: Write the failing test**

Add to the `tests` module in `crates/edifact-types/src/delimiters.rs`:

```rust
    #[test]
    fn test_from_una_standard() {
        let una = b"UNA:+.? '";
        let d = EdifactDelimiters::from_una(una).unwrap();
        assert_eq!(d, EdifactDelimiters::default());
    }

    #[test]
    fn test_from_una_custom_delimiters() {
        let una = b"UNA;*.# |";
        let d = EdifactDelimiters::from_una(una).unwrap();
        assert_eq!(d.component, b';');
        assert_eq!(d.element, b'*');
        assert_eq!(d.decimal, b'.');
        assert_eq!(d.release, b'#');
        assert_eq!(d.reserved, b' ');
        assert_eq!(d.segment, b'|');
    }

    #[test]
    fn test_from_una_too_short() {
        let una = b"UNA:+.";
        assert!(EdifactDelimiters::from_una(una).is_err());
    }

    #[test]
    fn test_from_una_wrong_prefix() {
        let una = b"XXX:+.? '";
        assert!(EdifactDelimiters::from_una(una).is_err());
    }

    #[test]
    fn test_detect_with_una() {
        let input = b"UNA:+.? 'UNB+UNOC:3+sender+recipient'";
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        assert!(has_una);
        assert_eq!(delimiters, EdifactDelimiters::default());
    }

    #[test]
    fn test_detect_without_una() {
        let input = b"UNB+UNOC:3+sender+recipient'";
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        assert!(!has_una);
        assert_eq!(delimiters, EdifactDelimiters::default());
    }

    #[test]
    fn test_detect_empty_input() {
        let input = b"";
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        assert!(!has_una);
        assert_eq!(delimiters, EdifactDelimiters::default());
    }

    #[test]
    fn test_una_roundtrip() {
        let original = EdifactDelimiters {
            component: b';',
            element: b'*',
            decimal: b',',
            release: b'#',
            segment: b'!',
            reserved: b' ',
        };
        let una_string = original.to_una_string();
        let parsed = EdifactDelimiters::from_una(una_string.as_bytes()).unwrap();
        assert_eq!(original, parsed);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-types test_from_una`
Expected: FAIL — `from_una` method does not exist.

**Step 3: Write minimal implementation**

Add to the `impl EdifactDelimiters` block in `crates/edifact-types/src/delimiters.rs`:

```rust
    /// Parse delimiters from a UNA service string advice segment.
    ///
    /// The UNA segment is exactly 9 bytes: `UNA` followed by 6 delimiter characters.
    /// Format: `UNA<component><element><decimal><release><reserved><terminator>`
    ///
    /// # Errors
    ///
    /// Returns an error if the input is not exactly 9 bytes or does not start with `UNA`.
    pub fn from_una(una: &[u8]) -> Result<Self, UnaParseError> {
        if una.len() != 9 {
            return Err(UnaParseError::InvalidLength {
                expected: 9,
                actual: una.len(),
            });
        }

        if &una[0..3] != b"UNA" {
            return Err(UnaParseError::InvalidPrefix);
        }

        // UNA format positions:
        // 0-2: "UNA"
        // 3: component separator
        // 4: element separator
        // 5: decimal mark
        // 6: release character
        // 7: reserved
        // 8: segment terminator
        Ok(Self {
            component: una[3],
            element: una[4],
            decimal: una[5],
            release: una[6],
            reserved: una[7],
            segment: una[8],
        })
    }

    /// Detect delimiters from an EDIFACT message.
    ///
    /// If the message starts with a UNA segment, parses delimiters from it.
    /// Otherwise, returns the standard defaults.
    ///
    /// Returns `(has_una, delimiters)`.
    pub fn detect(input: &[u8]) -> (bool, Self) {
        if input.len() >= 9 && &input[0..3] == b"UNA" {
            match Self::from_una(&input[0..9]) {
                Ok(d) => (true, d),
                Err(_) => (false, Self::default()),
            }
        } else {
            (false, Self::default())
        }
    }
```

Add the error type before the `EdifactDelimiters` struct:

```rust
/// Error when parsing a UNA service string advice segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaParseError {
    /// UNA segment must be exactly 9 bytes.
    InvalidLength { expected: usize, actual: usize },
    /// UNA segment must start with "UNA".
    InvalidPrefix,
}

impl std::fmt::Display for UnaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(f, "UNA segment must be exactly {expected} bytes, got {actual}")
            }
            Self::InvalidPrefix => write!(f, "UNA segment must start with 'UNA'"),
        }
    }
}

impl std::error::Error for UnaParseError {}
```

Update `crates/edifact-types/src/lib.rs`:

```rust
//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.

mod delimiters;

pub use delimiters::{EdifactDelimiters, UnaParseError};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-types`
Expected: PASS — all tests pass.

**Step 5: Commit**

```bash
git add crates/edifact-types/
git commit -m "$(cat <<'EOF'
feat(edifact-types): add UNA parsing and delimiter detection

EdifactDelimiters::from_una() parses the 9-byte UNA header.
EdifactDelimiters::detect() auto-detects from message start.
Includes UnaParseError for invalid inputs and roundtrip test.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: SegmentPosition

**Files:**
- Create: `crates/edifact-types/src/position.rs`
- Modify: `crates/edifact-types/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-types/src/position.rs`:

```rust
/// Position metadata for a parsed EDIFACT segment.
///
/// Tracks where a segment was found in the input stream, enabling
/// error reporting with byte-level precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentPosition {
    /// 1-based segment number within the interchange.
    pub segment_number: u32,
    /// Byte offset from the start of the input.
    pub byte_offset: usize,
    /// 1-based message number within the interchange (0 for service segments UNB/UNZ).
    pub message_number: u32,
}

impl SegmentPosition {
    /// Creates a new segment position.
    pub fn new(segment_number: u32, byte_offset: usize, message_number: u32) -> Self {
        Self {
            segment_number,
            byte_offset,
            message_number,
        }
    }
}

impl std::fmt::Display for SegmentPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "segment {} at byte {} (message {})",
            self.segment_number, self.byte_offset, self.message_number
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_position_new() {
        let pos = SegmentPosition::new(5, 128, 1);
        assert_eq!(pos.segment_number, 5);
        assert_eq!(pos.byte_offset, 128);
        assert_eq!(pos.message_number, 1);
    }

    #[test]
    fn test_segment_position_display() {
        let pos = SegmentPosition::new(3, 42, 1);
        assert_eq!(pos.to_string(), "segment 3 at byte 42 (message 1)");
    }

    #[test]
    fn test_segment_position_service_segment() {
        let pos = SegmentPosition::new(1, 0, 0);
        assert_eq!(pos.message_number, 0);
    }

    #[test]
    fn test_segment_position_equality() {
        let a = SegmentPosition::new(1, 0, 1);
        let b = SegmentPosition::new(1, 0, 1);
        let c = SegmentPosition::new(2, 0, 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_segment_position_clone() {
        let pos = SegmentPosition::new(1, 100, 2);
        let cloned = pos;
        assert_eq!(pos, cloned);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-types test_segment_position`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the test file above. Update `crates/edifact-types/src/lib.rs`:

```rust
//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.

mod delimiters;
mod position;

pub use delimiters::{EdifactDelimiters, UnaParseError};
pub use position::SegmentPosition;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-types`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-types/
git commit -m "$(cat <<'EOF'
feat(edifact-types): add SegmentPosition for tracking segment locations

Tracks segment number, byte offset, and message number for
precise error reporting and position-aware processing.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: RawSegment Zero-Copy Type

**Files:**
- Create: `crates/edifact-types/src/segment.rs`
- Modify: `crates/edifact-types/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-types/src/segment.rs`:

```rust
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
        self.elements.get(element_index).map_or(&[], |e| e.as_slice())
    }

    /// Checks if the segment has the given ID (case-insensitive).
    pub fn is(&self, segment_id: &str) -> bool {
        self.id.eq_ignore_ascii_case(segment_id)
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
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-types test_raw_segment`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the test file above. Update `crates/edifact-types/src/lib.rs`:

```rust
//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.
//!
//! # Types
//!
//! - [`EdifactDelimiters`] — the six delimiter characters
//! - [`SegmentPosition`] — byte offset and segment/message numbering
//! - [`RawSegment`] — zero-copy parsed segment borrowing from the input buffer
//! - [`Control`] — handler flow control (Continue / Stop)

mod delimiters;
mod position;
mod segment;

pub use delimiters::{EdifactDelimiters, UnaParseError};
pub use position::SegmentPosition;
pub use segment::RawSegment;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-types`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/edifact-types/
git commit -m "$(cat <<'EOF'
feat(edifact-types): add RawSegment zero-copy type

Zero-copy segment that borrows string slices from the input buffer.
Provides get_element(), get_component(), and Display formatting.
All accessor methods return empty string for out-of-bounds access.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Control Enum

**Files:**
- Create: `crates/edifact-types/src/control.rs`
- Modify: `crates/edifact-types/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/edifact-types/src/control.rs`:

```rust
/// Flow control signal returned by handler methods.
///
/// Handlers return this to tell the parser whether to continue
/// processing or stop early.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// Continue processing the next segment.
    Continue,
    /// Stop processing immediately.
    Stop,
}

impl Control {
    /// Returns `true` if this is `Control::Continue`.
    pub fn should_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Returns `true` if this is `Control::Stop`.
    pub fn should_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }
}

impl Default for Control {
    fn default() -> Self {
        Self::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_continue() {
        let c = Control::Continue;
        assert!(c.should_continue());
        assert!(!c.should_stop());
    }

    #[test]
    fn test_control_stop() {
        let c = Control::Stop;
        assert!(c.should_stop());
        assert!(!c.should_continue());
    }

    #[test]
    fn test_control_default_is_continue() {
        assert_eq!(Control::default(), Control::Continue);
    }

    #[test]
    fn test_control_equality() {
        assert_eq!(Control::Continue, Control::Continue);
        assert_eq!(Control::Stop, Control::Stop);
        assert_ne!(Control::Continue, Control::Stop);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p edifact-types test_control`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

The implementation is already in the file above. Update `crates/edifact-types/src/lib.rs`:

```rust
//! Shared EDIFACT primitive types.
//!
//! This crate defines the core data structures used across the EDIFACT parser
//! and automapper pipeline. It has zero external dependencies.
//!
//! # Types
//!
//! - [`EdifactDelimiters`] — the six delimiter characters
//! - [`SegmentPosition`] — byte offset and segment/message numbering
//! - [`RawSegment`] — zero-copy parsed segment borrowing from the input buffer
//! - [`Control`] — handler flow control (Continue / Stop)

mod control;
mod delimiters;
mod position;
mod segment;

pub use control::Control;
pub use delimiters::{EdifactDelimiters, UnaParseError};
pub use position::SegmentPosition;
pub use segment::RawSegment;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p edifact-types`
Expected: PASS — all 20+ tests pass.

**Step 5: Commit**

```bash
git add crates/edifact-types/
git commit -m "$(cat <<'EOF'
feat(edifact-types): add Control enum for handler flow control

Continue/Stop signal that handlers return to the parser.
Defaults to Continue. Completes the edifact-types public API.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```
