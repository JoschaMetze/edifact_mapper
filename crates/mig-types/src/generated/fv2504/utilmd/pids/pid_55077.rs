//! Auto-generated PID 55077 types.
//! Anmeldung erz. MaLo
//! Do not edit manually.

use crate::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};
use crate::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55077Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg5_z16: Vec<Pid55077Sg5Z16>,
    pub sg5_z21: Vec<Pid55077Sg5Z21>,
    pub sg6: Vec<Pid55077Sg6>,
    pub sg8_z79: Vec<Pid55077Sg8Z79>,
    pub sg8_zh0: Vec<Pid55077Sg8Zh0>,
    pub sg8_z01: Vec<Pid55077Sg8Z01>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg5Z21 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg8Z01 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55077Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z79
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg8Z79 {
    pub pia: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55077Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZH0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077Sg8Zh0 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55077Sg10>,
}

/// PID 55077: Anmeldung erz. MaLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55077Sg2>,
    pub sg4: Vec<Pid55077Sg4>,
}

impl Pid55077Sg10 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { cav, cci })
    }
}

impl Pid55077Sg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55077Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self { nad, sg3_ic })
    }
}

impl Pid55077Sg3Ic {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { com, cta })
    }
}

impl Pid55077Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        if dtm.is_none() && ftx.is_none() && ide.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55077Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg5_z21 = Vec::new();
        while let Some(item) = Pid55077Sg5Z21::from_segments(segments, cursor) {
            sg5_z21.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55077Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z79 = Vec::new();
        while let Some(item) = Pid55077Sg8Z79::from_segments(segments, cursor) {
            sg8_z79.push(item);
        }
        let mut sg8_zh0 = Vec::new();
        while let Some(item) = Pid55077Sg8Zh0::from_segments(segments, cursor) {
            sg8_zh0.push(item);
        }
        let mut sg8_z01 = Vec::new();
        while let Some(item) = Pid55077Sg8Z01::from_segments(segments, cursor) {
            sg8_z01.push(item);
        }
        Some(Self {
            dtm,
            ftx,
            ide,
            sts,
            sg5_z16,
            sg5_z21,
            sg6,
            sg8_z79,
            sg8_zh0,
            sg8_z01,
        })
    }
}

impl Pid55077Sg5Z16 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55077Sg5Z21 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55077Sg6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { rff })
    }
}

impl Pid55077Sg8Z01 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55077Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55077Sg8Z79 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55077Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { pia, seq, sg10 })
    }
}

impl Pid55077Sg8Zh0 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55077Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55077 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid55077Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55077Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55077 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
