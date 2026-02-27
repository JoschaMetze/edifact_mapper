//! Auto-generated PID  types.
//! Syntaxfehler-meldung in der Nachricht
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Nachrichten-Referenznummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PidSg1 {
    pub ucm: Option<OwnedSegment>,
    pub sg2: Vec<PidSg2>,
    pub sg2_13_15_16_22_35_36: Vec<PidSg2131516223536>,
}

/// SG2 — Segmentposition in der Nachricht
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PidSg2 {
    pub ucs: Option<OwnedSegment>,
}

/// SG2 — Segmentposition in der Nachricht
/// Qualifiers: 13, 15, 16, 22, 35, 36
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PidSg2131516223536 {
    pub ucd: Option<OwnedSegment>,
    pub ucs: Option<OwnedSegment>,
}

/// PID : Syntaxfehler-meldung in der Nachricht
/// Kommunikation: 
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid {
    pub uci: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<PidSg1>,
}

impl PidSg1 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ucm = if peek_is(segments, cursor, "UCM") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ucm.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg2 = Vec::new();
        while let Some(item) = PidSg2::from_segments(segments, cursor) {
            sg2.push(item);
        }
        let mut sg2_13_15_16_22_35_36 = Vec::new();
        while let Some(item) = PidSg2131516223536::from_segments(segments, cursor) {
            sg2_13_15_16_22_35_36.push(item);
        }
        Some(Self {
            ucm,
            sg2,
            sg2_13_15_16_22_35_36,
        })
    }
}

impl PidSg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ucs = if peek_is(segments, cursor, "UCS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ucs.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ucs,
        })
    }
}

impl PidSg2131516223536 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ucd = if peek_is(segments, cursor, "UCD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let ucs = if peek_is(segments, cursor, "UCS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ucd.is_none() && ucs.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ucd,
            ucs,
        })
    }
}

impl Pid {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let uci = expect_segment(segments, &mut cursor, "UCI")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = PidSg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }

        Ok(Pid {
            uci,
            unh,
            unt,
            sg1,
        })
    }
}
