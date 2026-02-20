//! Auto-generated PID 55622 types.
//! Rückmeldung/Anfrage Daten der MaLo
//! Do not edit manually.

use crate::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};
use crate::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z51, Z52
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z51Z52 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z53, Z54
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z53Z54 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z55, Z56
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z55Z56 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z57, Z58
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z57Z58 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z59, Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z59Z60 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55622Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg4 {
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z51_z52: Vec<Pid55622Sg12Z51Z52>,
    pub sg12_z53_z54: Vec<Pid55622Sg12Z53Z54>,
    pub sg12_z55_z56: Vec<Pid55622Sg12Z55Z56>,
    pub sg12_z57_z58: Vec<Pid55622Sg12Z57Z58>,
    pub sg12_z59_z60: Vec<Pid55622Sg12Z59Z60>,
    pub sg5_z16: Vec<Pid55622Sg5Z16>,
    pub sg6: Vec<Pid55622Sg6>,
    pub sg8_z80_z81: Vec<Pid55622Sg8Z80Z81>,
    pub sg8_zd1_zd2: Vec<Pid55622Sg8Zd1Zd2>,
    pub sg8_zd3_zd4: Vec<Pid55622Sg8Zd3Zd4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg6 {
    pub dtm: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z80, Z81
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg8Z80Z81 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55622Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD1, ZD2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg8Zd1Zd2 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55622Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD3, ZD4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg8Zd3Zd4 {
    pub pia: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// PID 55622: Rückmeldung/Anfrage Daten der MaLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55622Sg2>,
    pub sg4: Vec<Pid55622Sg4>,
}

impl Pid55622Sg10 {
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

impl Pid55622Sg12Z51Z52 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55622Sg12Z53Z54 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55622Sg12Z55Z56 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55622Sg12Z57Z58 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55622Sg12Z59Z60 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55622Sg2 {
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
        while let Some(item) = Pid55622Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self { nad, sg3_ic })
    }
}

impl Pid55622Sg3Ic {
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

impl Pid55622Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        let mut sg12_z51_z52 = Vec::new();
        while let Some(item) = Pid55622Sg12Z51Z52::from_segments(segments, cursor) {
            sg12_z51_z52.push(item);
        }
        let mut sg12_z53_z54 = Vec::new();
        while let Some(item) = Pid55622Sg12Z53Z54::from_segments(segments, cursor) {
            sg12_z53_z54.push(item);
        }
        let mut sg12_z55_z56 = Vec::new();
        while let Some(item) = Pid55622Sg12Z55Z56::from_segments(segments, cursor) {
            sg12_z55_z56.push(item);
        }
        let mut sg12_z57_z58 = Vec::new();
        while let Some(item) = Pid55622Sg12Z57Z58::from_segments(segments, cursor) {
            sg12_z57_z58.push(item);
        }
        let mut sg12_z59_z60 = Vec::new();
        while let Some(item) = Pid55622Sg12Z59Z60::from_segments(segments, cursor) {
            sg12_z59_z60.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55622Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55622Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z80_z81 = Vec::new();
        while let Some(item) = Pid55622Sg8Z80Z81::from_segments(segments, cursor) {
            sg8_z80_z81.push(item);
        }
        let mut sg8_zd1_zd2 = Vec::new();
        while let Some(item) = Pid55622Sg8Zd1Zd2::from_segments(segments, cursor) {
            sg8_zd1_zd2.push(item);
        }
        let mut sg8_zd3_zd4 = Vec::new();
        while let Some(item) = Pid55622Sg8Zd3Zd4::from_segments(segments, cursor) {
            sg8_zd3_zd4.push(item);
        }
        Some(Self {
            ide,
            sts,
            sg12_z51_z52,
            sg12_z53_z54,
            sg12_z55_z56,
            sg12_z57_z58,
            sg12_z59_z60,
            sg5_z16,
            sg6,
            sg8_z80_z81,
            sg8_zd1_zd2,
            sg8_zd3_zd4,
        })
    }
}

impl Pid55622Sg5Z16 {
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

impl Pid55622Sg6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
        let saved = cursor.save();
        let dtm = if peek_is(segments, cursor, "DTM") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self { dtm, rff })
    }
}

impl Pid55622Sg8Z80Z81 {
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
        while let Some(item) = Pid55622Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55622Sg8Zd1Zd2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55622Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55622Sg8Zd3Zd4 {
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
        Some(Self { pia, seq })
    }
}

impl Pid55622 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid55622Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55622Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55622 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
