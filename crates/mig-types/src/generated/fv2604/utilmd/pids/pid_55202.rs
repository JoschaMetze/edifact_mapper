//! Auto-generated PID 55202 types.
//! Korrekturliste LF-AACL
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55202Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Identifikation einer Liste
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg4 {
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg5_z15: Vec<Pid55202Sg5Z15>,
    pub sg5_z16: Vec<Pid55202Sg5Z16>,
    pub sg5_z21: Vec<Pid55202Sg5Z21>,
    pub sg6: Vec<Pid55202Sg6>,
    pub sg8_z22: Vec<Pid55202Sg8Z22>,
    pub sg8_z01: Vec<Pid55202Sg8Z01>,
    pub sg8_z15: Vec<Pid55202Sg8Z15>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg5Z15 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg5Z21 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg8Z01 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55202Sg10>,
    pub sg9: Vec<Pid55202Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg8Z15 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg9: Vec<Pid55202Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z22
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg8Z22 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202Sg9 {
    pub qty: Option<OwnedSegment>,
}

/// PID 55202: Korrekturliste LF-AACL
/// Kommunikation: LF an NB (ANB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55202Sg2>,
    pub sg4: Vec<Pid55202Sg4>,
}

impl Pid55202Sg10 {
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

impl Pid55202Sg2 {
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
        while let Some(item) = Pid55202Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid55202Sg3Ic {
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

impl Pid55202Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
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
        if ide.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg5_z15 = Vec::new();
        while let Some(item) = Pid55202Sg5Z15::from_segments(segments, cursor) {
            sg5_z15.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55202Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg5_z21 = Vec::new();
        while let Some(item) = Pid55202Sg5Z21::from_segments(segments, cursor) {
            sg5_z21.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55202Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z22 = Vec::new();
        while let Some(item) = Pid55202Sg8Z22::from_segments(segments, cursor) {
            sg8_z22.push(item);
        }
        let mut sg8_z01 = Vec::new();
        while let Some(item) = Pid55202Sg8Z01::from_segments(segments, cursor) {
            sg8_z01.push(item);
        }
        let mut sg8_z15 = Vec::new();
        while let Some(item) = Pid55202Sg8Z15::from_segments(segments, cursor) {
            sg8_z15.push(item);
        }
        Some(Self {
            ide,
            sts,
            sg5_z15,
            sg5_z16,
            sg5_z21,
            sg6,
            sg8_z22,
            sg8_z01,
            sg8_z15,
        })
    }
}

impl Pid55202Sg5Z15 {
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

impl Pid55202Sg5Z16 {
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

impl Pid55202Sg5Z21 {
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

impl Pid55202Sg6 {
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

impl Pid55202Sg8Z01 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55202Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55202Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
            sg9,
        })
    }
}

impl Pid55202Sg8Z15 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55202Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg9,
        })
    }
}

impl Pid55202Sg8Z22 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            rff,
            seq,
        })
    }
}

impl Pid55202Sg9 {
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

impl Pid55202 {
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
        while let Some(item) = Pid55202Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55202Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55202 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
