//! Auto-generated PID 21001 types.
//! Statusmeldung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21001Sg1 {
    pub nad: Option<OwnedSegment>,
    pub sg2_ic: Vec<Pid21001Sg2Ic>,
}

/// SG2 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21001Sg2Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Equipment, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21001Sg4 {
    pub eqd: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub sg6_172: Vec<Pid21001Sg6172>,
    pub sg7: Vec<Pid21001Sg7>,
}

/// SG6 — Ortsangabe, Qualifier
/// Qualifiers: 172
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21001Sg6172 {
    pub dtm: Option<OwnedSegment>,
    pub loc: Option<OwnedSegment>,
}

/// SG7 — Statuskategorie, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21001Sg7 {
    pub sts: Option<OwnedSegment>,
}

/// PID 21001: Statusmeldung
/// Kommunikation: NB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21001 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid21001Sg1>,
    pub sg4: Vec<Pid21001Sg4>,
}

impl Pid21001Sg1 {
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
        while let Some(item) = Pid21001Sg2Ic::from_segments(segments, cursor) {
            sg2_ic.push(item);
        }
        Some(Self {
            nad,
            sg2_ic,
        })
    }
}

impl Pid21001Sg2Ic {
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

impl Pid21001Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let eqd = if peek_is(segments, cursor, "EQD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if eqd.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg6_172 = Vec::new();
        while let Some(item) = Pid21001Sg6172::from_segments(segments, cursor) {
            sg6_172.push(item);
        }
        let mut sg7 = Vec::new();
        while let Some(item) = Pid21001Sg7::from_segments(segments, cursor) {
            sg7.push(item);
        }
        Some(Self {
            eqd,
            rff,
            sg6_172,
            sg7,
        })
    }
}

impl Pid21001Sg6172 {
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
        let loc = if peek_is(segments, cursor, "LOC") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            dtm,
            loc,
        })
    }
}

impl Pid21001Sg7 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let sts = if peek_is(segments, cursor, "STS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            sts,
        })
    }
}

impl Pid21001 {
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
        while let Some(item) = Pid21001Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid21001Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid21001 {
            bgm,
            dtm,
            unh,
            unt,
            sg1,
            sg4,
        })
    }
}
