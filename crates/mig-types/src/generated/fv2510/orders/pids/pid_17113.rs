//! Auto-generated PID 17113 types.
//! Reklamation von Werten
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17113Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17113Sg2 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
    pub sg3: Vec<Pid17113Sg3>,
    pub sg5_ic: Vec<Pid17113Sg5Ic>,
}

/// SG29 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17113Sg29 {
    pub dtm: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
}

/// SG3 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17113Sg3 {
    pub rff: Option<OwnedSegment>,
}

/// SG5 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17113Sg5Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// PID 17113: Reklamation von Werten
/// Kommunikation: LF, NB, ÜNB, MSB an MSB (Strom), LF an NB (Gas)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17113 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub imd: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid17113Sg1>,
    pub sg2: Vec<Pid17113Sg2>,
    pub sg29: Vec<Pid17113Sg29>,
}

impl Pid17113Sg1 {
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

impl Pid17113Sg2 {
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
        let mut sg3 = Vec::new();
        while let Some(item) = Pid17113Sg3::from_segments(segments, cursor) {
            sg3.push(item);
        }
        let mut sg5_ic = Vec::new();
        while let Some(item) = Pid17113Sg5Ic::from_segments(segments, cursor) {
            sg5_ic.push(item);
        }
        Some(Self {
            loc,
            nad,
            sg3,
            sg5_ic,
        })
    }
}

impl Pid17113Sg29 {
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
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && ftx.is_none() && lin.is_none() && pia.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            dtm,
            ftx,
            lin,
            pia,
        })
    }
}

impl Pid17113Sg3 {
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

impl Pid17113Sg5Ic {
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
        if com.is_none() && cta.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            com,
            cta,
        })
    }
}

impl Pid17113 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let imd = expect_segment(segments, &mut cursor, "IMD")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let uns = expect_segment(segments, &mut cursor, "UNS")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid17113Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid17113Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg29 = Vec::new();
        while let Some(item) = Pid17113Sg29::from_segments(segments, &mut cursor) {
            sg29.push(item);
        }

        Ok(Pid17113 {
            bgm,
            dtm,
            imd,
            unh,
            uns,
            unt,
            sg1,
            sg2,
            sg29,
        })
    }
}
