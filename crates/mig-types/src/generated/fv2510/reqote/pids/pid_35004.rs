//! Auto-generated PID 35004 types.
//! Anfrage einer Konfiguration
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG11 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg11 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
    pub sg14_ic: Vec<Pid35004Sg14Ic>,
}

/// SG14 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg14Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG27 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg27 {
    pub ftx: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
    pub sg28_z52: Vec<Pid35004Sg28Z52>,
    pub sg28_z53: Vec<Pid35004Sg28Z53>,
    pub sg28_z54: Vec<Pid35004Sg28Z54>,
    pub sg28_z60: Vec<Pid35004Sg28Z60>,
}

/// SG28 — Klassentyp, Code
/// Qualifiers: Z52
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg28Z52 {
    pub cci: Option<OwnedSegment>,
}

/// SG28 — Klassentyp, Code
/// Qualifiers: Z53
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg28Z53 {
    pub cci: Option<OwnedSegment>,
}

/// SG28 — Klassentyp, Code
/// Qualifiers: Z54
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg28Z54 {
    pub cci: Option<OwnedSegment>,
}

/// SG28 — Klassentyp, Code
/// Qualifiers: Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004Sg28Z60 {
    pub cci: Option<OwnedSegment>,
}

/// PID 35004: Anfrage einer Konfiguration
/// Kommunikation: NB, LF an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid35004 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub imd: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid35004Sg1>,
    pub sg11: Vec<Pid35004Sg11>,
    pub sg27: Vec<Pid35004Sg27>,
}

impl Pid35004Sg1 {
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

impl Pid35004Sg11 {
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
        let mut sg14_ic = Vec::new();
        while let Some(item) = Pid35004Sg14Ic::from_segments(segments, cursor) {
            sg14_ic.push(item);
        }
        Some(Self {
            loc,
            nad,
            sg14_ic,
        })
    }
}

impl Pid35004Sg14Ic {
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

impl Pid35004Sg27 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
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
        if ftx.is_none() && lin.is_none() && pia.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg28_z52 = Vec::new();
        while let Some(item) = Pid35004Sg28Z52::from_segments(segments, cursor) {
            sg28_z52.push(item);
        }
        let mut sg28_z53 = Vec::new();
        while let Some(item) = Pid35004Sg28Z53::from_segments(segments, cursor) {
            sg28_z53.push(item);
        }
        let mut sg28_z54 = Vec::new();
        while let Some(item) = Pid35004Sg28Z54::from_segments(segments, cursor) {
            sg28_z54.push(item);
        }
        let mut sg28_z60 = Vec::new();
        while let Some(item) = Pid35004Sg28Z60::from_segments(segments, cursor) {
            sg28_z60.push(item);
        }
        Some(Self {
            ftx,
            lin,
            pia,
            sg28_z52,
            sg28_z53,
            sg28_z54,
            sg28_z60,
        })
    }
}

impl Pid35004Sg28Z52 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cci = if peek_is(segments, cursor, "CCI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cci.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cci,
        })
    }
}

impl Pid35004Sg28Z53 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cci = if peek_is(segments, cursor, "CCI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cci.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cci,
        })
    }
}

impl Pid35004Sg28Z54 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cci = if peek_is(segments, cursor, "CCI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cci.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cci,
        })
    }
}

impl Pid35004Sg28Z60 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cci = if peek_is(segments, cursor, "CCI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cci.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cci,
        })
    }
}

impl Pid35004 {
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
        while let Some(item) = Pid35004Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg11 = Vec::new();
        while let Some(item) = Pid35004Sg11::from_segments(segments, &mut cursor) {
            sg11.push(item);
        }
        let mut sg27 = Vec::new();
        while let Some(item) = Pid35004Sg27::from_segments(segments, &mut cursor) {
            sg27.push(item);
        }

        Ok(Pid35004 {
            bgm,
            dtm,
            imd,
            unh,
            uns,
            unt,
            sg1,
            sg11,
            sg27,
        })
    }
}
