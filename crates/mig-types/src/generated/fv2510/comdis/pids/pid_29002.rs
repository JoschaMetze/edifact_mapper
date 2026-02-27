//! Auto-generated PID 29002 types.
//! Ablehnung IFTSTA
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29002Sg1 {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
}

/// SG2 — Dokumentenname, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29002Sg2 {
    pub doc: Option<OwnedSegment>,
    pub sg3: Vec<Pid29002Sg3>,
    pub sg3_s_0108: Vec<Pid29002Sg3S_0108>,
}

/// SG3 — Anpassungsgrund, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29002Sg3 {
    pub ajt: Option<OwnedSegment>,
}

/// SG3 — Anpassungsgrund, Code
/// Qualifiers: S_0108
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29002Sg3S_0108 {
    pub ajt: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
}

/// PID 29002: Ablehnung IFTSTA
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29002 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub rff: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid29002Sg1>,
    pub sg2: Vec<Pid29002Sg2>,
}

impl Pid29002Sg1 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let com = if peek_is(segments, cursor, "COM") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let cta = if peek_is(segments, cursor, "CTA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let nad = if peek_is(segments, cursor, "NAD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if com.is_none() && cta.is_none() && nad.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            com,
            cta,
            nad,
        })
    }
}

impl Pid29002Sg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let doc = if peek_is(segments, cursor, "DOC") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if doc.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg3 = Vec::new();
        while let Some(item) = Pid29002Sg3::from_segments(segments, cursor) {
            sg3.push(item);
        }
        let mut sg3_s_0108 = Vec::new();
        while let Some(item) = Pid29002Sg3S_0108::from_segments(segments, cursor) {
            sg3_s_0108.push(item);
        }
        Some(Self {
            doc,
            sg3,
            sg3_s_0108,
        })
    }
}

impl Pid29002Sg3 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ajt = if peek_is(segments, cursor, "AJT") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ajt.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ajt,
        })
    }
}

impl Pid29002Sg3S_0108 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ajt = if peek_is(segments, cursor, "AJT") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let ftx = if peek_is(segments, cursor, "FTX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ajt.is_none() && ftx.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ajt,
            ftx,
        })
    }
}

impl Pid29002 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let rff = expect_segment(segments, &mut cursor, "RFF")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid29002Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid29002Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }

        Ok(Pid29002 {
            bgm,
            dtm,
            rff,
            unh,
            unt,
            sg1,
            sg2,
        })
    }
}
