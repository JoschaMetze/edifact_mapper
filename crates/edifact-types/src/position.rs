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
