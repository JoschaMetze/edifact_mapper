//! Auto-generated PID 21009 types.
//! Status-meldung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21009Sg1 {
    pub nad: Option<OwnedSegment>,
    pub sg2_ic: Vec<Pid21009Sg2Ic>,
}

/// SG14
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21009Sg14 {
    pub cni: Option<OwnedSegment>,
    pub loc: Option<OwnedSegment>,
    pub sg15: Vec<Pid21009Sg15>,
}

/// SG15 — Statuskategorie, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21009Sg15 {
    pub rff: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
}

/// SG2 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21009Sg2Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// PID 21009: Status-meldung
/// Kommunikation: MSBN an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21009 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid21009Sg1>,
    pub sg14: Vec<Pid21009Sg14>,
}

impl Pid21009Sg1 {
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
        let mut sg2_ic = Vec::new();
        while let Some(item) = Pid21009Sg2Ic::from_segments(segments, cursor) {
            sg2_ic.push(item);
        }
        Some(Self {
            nad,
            sg2_ic,
        })
    }
}

impl Pid21009Sg14 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cni = if peek_is(segments, cursor, "CNI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let loc = if peek_is(segments, cursor, "LOC") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cni.is_none() && loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg15 = Vec::new();
        while let Some(item) = Pid21009Sg15::from_segments(segments, cursor) {
            sg15.push(item);
        }
        Some(Self {
            cni,
            loc,
            sg15,
        })
    }
}

impl Pid21009Sg15 {
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
        let sts = if peek_is(segments, cursor, "STS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            rff,
            sts,
        })
    }
}

impl Pid21009Sg2Ic {
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

impl Pid21009 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid21009Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg14 = Vec::new();
        while let Some(item) = Pid21009Sg14::from_segments(segments, &mut cursor) {
            sg14.push(item);
        }

        Ok(Pid21009 {
            bgm,
            dtm,
            unh,
            unt,
            sg1,
            sg14,
        })
    }
}
