//! Segment cursor for tracking position during MIG-guided assembly.
//!
//! Lives in `mig-types` so that generated PID `from_segments()` impls
//! can use cursor helpers without depending on `mig-assembly`.

use crate::segment::OwnedSegment;

/// Error returned when an expected segment is not found at the cursor position.
#[derive(Debug)]
pub struct SegmentNotFound {
    pub expected: String,
}

impl std::fmt::Display for SegmentNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected segment '{}' not found", self.expected)
    }
}

impl std::error::Error for SegmentNotFound {}

/// A cursor that tracks position within a segment slice during assembly.
///
/// The cursor is the core state machine of the assembler. It advances
/// through segments as the MIG tree is matched against the input.
pub struct SegmentCursor {
    position: usize,
    total: usize,
}

impl SegmentCursor {
    pub fn new(total: usize) -> Self {
        Self { position: 0, total }
    }

    /// Current position in the segment list.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Number of segments remaining.
    pub fn remaining(&self) -> usize {
        self.total.saturating_sub(self.position)
    }

    /// Whether all segments have been consumed.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.total
    }

    /// Advance the cursor by one segment.
    pub fn advance(&mut self) {
        self.position += 1;
    }

    /// Save the current position for backtracking.
    pub fn save(&self) -> usize {
        self.position
    }

    /// Restore to a previously saved position.
    pub fn restore(&mut self, saved: usize) {
        self.position = saved;
    }
}

/// Check if the segment at the cursor's current position matches a tag.
pub fn peek_is(segments: &[OwnedSegment], cursor: &SegmentCursor, tag: &str) -> bool {
    if cursor.is_exhausted() {
        return false;
    }
    segments[cursor.position()].is(tag)
}

/// Consume the segment at the cursor's current position, advancing the cursor.
/// Returns None if the cursor is exhausted.
pub fn consume<'a>(
    segments: &'a [OwnedSegment],
    cursor: &mut SegmentCursor,
) -> Option<&'a OwnedSegment> {
    if cursor.is_exhausted() {
        return None;
    }
    let seg = &segments[cursor.position()];
    cursor.advance();
    Some(seg)
}

/// Consume the segment at cursor if it matches the expected tag.
/// Returns Err if exhausted or tag mismatch.
pub fn expect_segment<'a>(
    segments: &'a [OwnedSegment],
    cursor: &mut SegmentCursor,
    tag: &str,
) -> Result<&'a OwnedSegment, SegmentNotFound> {
    if cursor.is_exhausted() {
        return Err(SegmentNotFound {
            expected: tag.to_string(),
        });
    }
    let seg = &segments[cursor.position()];
    if !seg.is(tag) {
        return Err(SegmentNotFound {
            expected: tag.to_string(),
        });
    }
    cursor.advance();
    Ok(seg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segment(id: &str) -> OwnedSegment {
        OwnedSegment {
            id: id.to_string(),
            elements: vec![],
            segment_number: 0,
        }
    }

    #[test]
    fn test_cursor_peek_and_advance() {
        let mut cursor = SegmentCursor::new(4);

        assert_eq!(cursor.position(), 0);
        assert!(!cursor.is_exhausted());

        cursor.advance();
        assert_eq!(cursor.position(), 1);

        cursor.advance();
        cursor.advance();
        cursor.advance();
        assert!(cursor.is_exhausted());
    }

    #[test]
    fn test_cursor_remaining() {
        let mut cursor = SegmentCursor::new(5);
        assert_eq!(cursor.remaining(), 5);
        cursor.advance();
        assert_eq!(cursor.remaining(), 4);
    }

    #[test]
    fn test_cursor_save_restore() {
        let mut cursor = SegmentCursor::new(10);
        cursor.advance();
        cursor.advance();

        let saved = cursor.save();
        assert_eq!(saved, 2);

        cursor.advance();
        cursor.advance();
        assert_eq!(cursor.position(), 4);

        cursor.restore(saved);
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_cursor_empty() {
        let cursor = SegmentCursor::new(0);
        assert!(cursor.is_exhausted());
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn test_peek_is_helper() {
        let segments = vec![make_segment("NAD"), make_segment("IDE")];
        let cursor = SegmentCursor::new(segments.len());
        assert!(peek_is(&segments, &cursor, "NAD"));
        assert!(!peek_is(&segments, &cursor, "IDE"));
    }

    #[test]
    fn test_peek_is_exhausted() {
        let segments: Vec<OwnedSegment> = vec![];
        let cursor = SegmentCursor::new(0);
        assert!(!peek_is(&segments, &cursor, "NAD"));
    }

    #[test]
    fn test_consume_helper() {
        let segments = vec![make_segment("UNH"), make_segment("BGM")];
        let mut cursor = SegmentCursor::new(segments.len());

        let seg = consume(&segments, &mut cursor).unwrap();
        assert_eq!(seg.id, "UNH");
        assert_eq!(cursor.position(), 1);

        let seg = consume(&segments, &mut cursor).unwrap();
        assert_eq!(seg.id, "BGM");
        assert!(cursor.is_exhausted());

        assert!(consume(&segments, &mut cursor).is_none());
    }

    #[test]
    fn test_expect_segment_helper() {
        let segments = vec![make_segment("UNH"), make_segment("BGM")];
        let mut cursor = SegmentCursor::new(segments.len());
        let seg = expect_segment(&segments, &mut cursor, "UNH").unwrap();
        assert_eq!(seg.id, "UNH");
        assert_eq!(cursor.position(), 1);
    }

    #[test]
    fn test_expect_segment_wrong_tag() {
        let segments = vec![make_segment("UNH")];
        let mut cursor = SegmentCursor::new(segments.len());
        let result = expect_segment(&segments, &mut cursor, "BGM");
        assert!(result.is_err());
    }

    #[test]
    fn test_expect_segment_exhausted() {
        let segments: Vec<OwnedSegment> = vec![];
        let mut cursor = SegmentCursor::new(0);
        let result = expect_segment(&segments, &mut cursor, "UNH");
        assert!(result.is_err());
    }
}
