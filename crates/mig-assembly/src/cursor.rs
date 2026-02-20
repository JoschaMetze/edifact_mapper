//! Segment cursor for tracking position during MIG-guided assembly.
//!
//! The core types live in `mig-types::cursor` — re-exported here for
//! backward compatibility so existing `use crate::cursor::*` paths work.

pub use mig_types::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};

/// Wrapper version of `expect_segment` that returns `AssemblyError` instead of `SegmentNotFound`.
///
/// This provides backward-compatible error types for callers that work with `AssemblyError`.
pub fn expect_segment_assembly<'a>(
    segments: &'a [mig_types::segment::OwnedSegment],
    cursor: &mut SegmentCursor,
    tag: &str,
) -> Result<&'a mig_types::segment::OwnedSegment, crate::AssemblyError> {
    expect_segment(segments, cursor, tag).map_err(|e| e.into())
}
