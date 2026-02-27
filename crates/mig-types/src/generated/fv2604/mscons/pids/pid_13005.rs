//! Auto-generated PID 13005 types.
//! EEG-Überführungs-ZR
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg4_ic: Vec<Pid13005Sg4Ic>,
}

/// SG4 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg4Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG5 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg5 {
    pub nad: Option<OwnedSegment>,
    pub sg6_237: Vec<Pid13005Sg6237>,
    pub sg6_107: Vec<Pid13005Sg6107>,
}

/// SG6 — Ortsangabe, Qualifier
/// Qualifiers: 107
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg6107 {
    pub dtm: Option<OwnedSegment>,
    pub loc: Option<OwnedSegment>,
    pub sg8: Vec<Pid13005Sg8>,
    pub sg9: Vec<Pid13005Sg9>,
}

/// SG6 — Ortsangabe, Qualifier
/// Qualifiers: 237
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg6237 {
    pub loc: Option<OwnedSegment>,
}

/// SG8 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg8 {
    pub cci: Option<OwnedSegment>,
}

/// SG9
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005Sg9 {
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
}

/// PID 13005: EEG-Überführungs-ZR
/// Kommunikation: 
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid13005 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unb: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub unz: OwnedSegment,
    pub sg1: Vec<Pid13005Sg1>,
    pub sg2: Vec<Pid13005Sg2>,
    pub sg5: Vec<Pid13005Sg5>,
}

impl Pid13005Sg1 {
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

impl Pid13005Sg2 {
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
        let mut sg4_ic = Vec::new();
        while let Some(item) = Pid13005Sg4Ic::from_segments(segments, cursor) {
            sg4_ic.push(item);
        }
        Some(Self {
            nad,
            sg4_ic,
        })
    }
}

impl Pid13005Sg4Ic {
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

impl Pid13005Sg5 {
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
        let mut sg6_237 = Vec::new();
        while let Some(item) = Pid13005Sg6237::from_segments(segments, cursor) {
            sg6_237.push(item);
        }
        let mut sg6_107 = Vec::new();
        while let Some(item) = Pid13005Sg6107::from_segments(segments, cursor) {
            sg6_107.push(item);
        }
        Some(Self {
            nad,
            sg6_237,
            sg6_107,
        })
    }
}

impl Pid13005Sg6107 {
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
        let mut sg8 = Vec::new();
        while let Some(item) = Pid13005Sg8::from_segments(segments, cursor) {
            sg8.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid13005Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            dtm,
            loc,
            sg8,
            sg9,
        })
    }
}

impl Pid13005Sg6237 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid13005Sg8 {
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

impl Pid13005Sg9 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
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
        if lin.is_none() && pia.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            lin,
            pia,
        })
    }
}

impl Pid13005 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unb = expect_segment(segments, &mut cursor, "UNB")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let uns = expect_segment(segments, &mut cursor, "UNS")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let unz = expect_segment(segments, &mut cursor, "UNZ")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid13005Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid13005Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg5 = Vec::new();
        while let Some(item) = Pid13005Sg5::from_segments(segments, &mut cursor) {
            sg5.push(item);
        }

        Ok(Pid13005 {
            bgm,
            dtm,
            unb,
            unh,
            uns,
            unt,
            unz,
            sg1,
            sg2,
            sg5,
        })
    }
}
