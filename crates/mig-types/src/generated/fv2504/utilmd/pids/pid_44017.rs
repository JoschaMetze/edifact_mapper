//! Auto-generated PID 44017 types.
//! Bestätigung Kündigung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: DP
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg12Dp {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg12Z09 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid44017Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z09: Vec<Pid44017Sg12Z09>,
    pub sg12_dp: Vec<Pid44017Sg12Dp>,
    pub sg5_172: Vec<Pid44017Sg5172>,
    pub sg6: Vec<Pid44017Sg6>,
    pub sg8_z01: Vec<Pid44017Sg8Z01>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: 172
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg5172 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg8Z01 {
    pub seq: Option<OwnedSegment>,
    pub sg9: Vec<Pid44017Sg9>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017Sg9 {
    pub qty: Option<OwnedSegment>,
}

/// PID 44017: Bestätigung Kündigung
/// Kommunikation: LFA an LFN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44017 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid44017Sg2>,
    pub sg4: Vec<Pid44017Sg4>,
}

impl Pid44017Sg12Dp {
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

impl Pid44017Sg12Z09 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
            rff,
        })
    }
}

impl Pid44017Sg2 {
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
        let mut sg3_ic = Vec::new();
        while let Some(item) = Pid44017Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid44017Sg3Ic {
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

impl Pid44017Sg4 {
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
        let ide = if peek_is(segments, cursor, "IDE") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let sts = if peek_is(segments, cursor, "STS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && ide.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg12_z09 = Vec::new();
        while let Some(item) = Pid44017Sg12Z09::from_segments(segments, cursor) {
            sg12_z09.push(item);
        }
        let mut sg12_dp = Vec::new();
        while let Some(item) = Pid44017Sg12Dp::from_segments(segments, cursor) {
            sg12_dp.push(item);
        }
        let mut sg5_172 = Vec::new();
        while let Some(item) = Pid44017Sg5172::from_segments(segments, cursor) {
            sg5_172.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid44017Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z01 = Vec::new();
        while let Some(item) = Pid44017Sg8Z01::from_segments(segments, cursor) {
            sg8_z01.push(item);
        }
        Some(Self {
            dtm,
            ide,
            sts,
            sg12_z09,
            sg12_dp,
            sg5_172,
            sg6,
            sg8_z01,
        })
    }
}

impl Pid44017Sg5172 {
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

impl Pid44017Sg6 {
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

impl Pid44017Sg8Z01 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid44017Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            seq,
            sg9,
        })
    }
}

impl Pid44017Sg9 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let qty = if peek_is(segments, cursor, "QTY") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if qty.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            qty,
        })
    }
}

impl Pid44017 {
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
        while let Some(item) = Pid44017Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid44017Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid44017 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
