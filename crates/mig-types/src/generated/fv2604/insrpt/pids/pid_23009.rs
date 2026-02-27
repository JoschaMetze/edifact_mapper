//! Auto-generated PID 23009 types.
//! Informations-meldung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid23009Sg2 {
    pub nad: Option<OwnedSegment>,
}

/// SG3 — Dokumentenname, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid23009Sg3 {
    pub doc: Option<OwnedSegment>,
    pub sg4: Vec<Pid23009Sg4>,
    pub sg7: Vec<Pid23009Sg7>,
}

/// SG4 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid23009Sg4 {
    pub rff: Option<OwnedSegment>,
}

/// SG7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid23009Sg7 {
    pub dtm: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg8: Vec<Pid23009Sg8>,
}

/// SG8 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid23009Sg8 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
}

/// PID 23009: Informations-meldung
/// Kommunikation: MSB an NB (Gas)/MSB (Strom)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid23009 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid23009Sg2>,
    pub sg3: Vec<Pid23009Sg3>,
}

impl Pid23009Sg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let nad = if peek_is(segments, cursor, "NAD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
        })
    }
}

impl Pid23009Sg3 {
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
        let mut sg4 = Vec::new();
        while let Some(item) = Pid23009Sg4::from_segments(segments, cursor) {
            sg4.push(item);
        }
        let mut sg7 = Vec::new();
        while let Some(item) = Pid23009Sg7::from_segments(segments, cursor) {
            sg7.push(item);
        }
        Some(Self {
            doc,
            sg4,
            sg7,
        })
    }
}

impl Pid23009Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            rff,
        })
    }
}

impl Pid23009Sg7 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let dtm = if peek_is(segments, cursor, "DTM") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let ftx = if peek_is(segments, cursor, "FTX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let lin = if peek_is(segments, cursor, "LIN") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let sts = if peek_is(segments, cursor, "STS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && ftx.is_none() && lin.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg8 = Vec::new();
        while let Some(item) = Pid23009Sg8::from_segments(segments, cursor) {
            sg8.push(item);
        }
        Some(Self {
            dtm,
            ftx,
            lin,
            sts,
            sg8,
        })
    }
}

impl Pid23009Sg8 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let loc = if peek_is(segments, cursor, "LOC") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let nad = if peek_is(segments, cursor, "NAD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if loc.is_none() && nad.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
            nad,
        })
    }
}

impl Pid23009 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid23009Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg3 = Vec::new();
        while let Some(item) = Pid23009Sg3::from_segments(segments, &mut cursor) {
            sg3.push(item);
        }

        Ok(Pid23009 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg3,
        })
    }
}
