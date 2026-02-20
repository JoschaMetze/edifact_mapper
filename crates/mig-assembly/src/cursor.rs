//! Segment cursor for tracking position during MIG-guided assembly.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
