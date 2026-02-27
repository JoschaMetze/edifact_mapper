//! Auto-generated PID 15001 types.
//! Angebot Geräteübernahme
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG11 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg11 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
    pub sg14_ic: Vec<Pid15001Sg14Ic>,
}

/// SG14 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg14Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG27 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg27 {
    pub dtm: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
    pub gin: Option<OwnedSegment>,
    pub imd: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub sg28: Vec<Pid15001Sg28>,
    pub sg31: Vec<Pid15001Sg31>,
    pub sg32: Vec<Pid15001Sg32>,
    pub sg42_vy: Vec<Pid15001Sg42Vy>,
}

/// SG28 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg28 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG31 — Preis, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg31 {
    pub pri: Option<OwnedSegment>,
}

/// SG32 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg32 {
    pub rff: Option<OwnedSegment>,
}

/// SG4 — Währungsverwendung, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg4 {
    pub cux: Option<OwnedSegment>,
}

/// SG42 — Beteiligter, Qualifier
/// Qualifiers: VY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001Sg42Vy {
    pub nad: Option<OwnedSegment>,
}

/// PID 15001: Angebot Geräteübernahme
/// Kommunikation: MSBA an MSBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15001 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub imd: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid15001Sg1>,
    pub sg11: Vec<Pid15001Sg11>,
    pub sg27: Vec<Pid15001Sg27>,
    pub sg4: Vec<Pid15001Sg4>,
}

impl Pid15001Sg1 {
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

impl Pid15001Sg11 {
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
        while let Some(item) = Pid15001Sg14Ic::from_segments(segments, cursor) {
            sg14_ic.push(item);
        }
        Some(Self {
            loc,
            nad,
            sg14_ic,
        })
    }
}

impl Pid15001Sg14Ic {
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

impl Pid15001Sg27 {
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
        let gin = if peek_is(segments, cursor, "GIN") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let imd = if peek_is(segments, cursor, "IMD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let lin = if peek_is(segments, cursor, "LIN") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && ftx.is_none() && gin.is_none() && imd.is_none() && lin.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg28 = Vec::new();
        while let Some(item) = Pid15001Sg28::from_segments(segments, cursor) {
            sg28.push(item);
        }
        let mut sg31 = Vec::new();
        while let Some(item) = Pid15001Sg31::from_segments(segments, cursor) {
            sg31.push(item);
        }
        let mut sg32 = Vec::new();
        while let Some(item) = Pid15001Sg32::from_segments(segments, cursor) {
            sg32.push(item);
        }
        let mut sg42_vy = Vec::new();
        while let Some(item) = Pid15001Sg42Vy::from_segments(segments, cursor) {
            sg42_vy.push(item);
        }
        Some(Self {
            dtm,
            ftx,
            gin,
            imd,
            lin,
            sg28,
            sg31,
            sg32,
            sg42_vy,
        })
    }
}

impl Pid15001Sg28 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cav = if peek_is(segments, cursor, "CAV") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let cci = if peek_is(segments, cursor, "CCI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cav.is_none() && cci.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cav,
            cci,
        })
    }
}

impl Pid15001Sg31 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pri = if peek_is(segments, cursor, "PRI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pri.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            pri,
        })
    }
}

impl Pid15001Sg32 {
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

impl Pid15001Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cux = if peek_is(segments, cursor, "CUX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cux.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cux,
        })
    }
}

impl Pid15001Sg42Vy {
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

impl Pid15001 {
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
        while let Some(item) = Pid15001Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg11 = Vec::new();
        while let Some(item) = Pid15001Sg11::from_segments(segments, &mut cursor) {
            sg11.push(item);
        }
        let mut sg27 = Vec::new();
        while let Some(item) = Pid15001Sg27::from_segments(segments, &mut cursor) {
            sg27.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid15001Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid15001 {
            bgm,
            dtm,
            imd,
            unh,
            uns,
            unt,
            sg1,
            sg11,
            sg27,
            sg4,
        })
    }
}
