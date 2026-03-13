//! Auto-generated PID 55616 types.
//! Änderung Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: DDO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg12Ddo {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: DP
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg12Dp {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: EO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg12Eo {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z25
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg12Z25 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z26
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg12Z26 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55616Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg4 {
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z25: Vec<Pid55616Sg12Z25>,
    pub sg12_z26: Vec<Pid55616Sg12Z26>,
    pub sg12_eo: Vec<Pid55616Sg12Eo>,
    pub sg12_ddo: Vec<Pid55616Sg12Ddo>,
    pub sg12_dp: Vec<Pid55616Sg12Dp>,
    pub sg5_z16: Vec<Pid55616Sg5Z16>,
    pub sg6: Vec<Pid55616Sg6>,
    pub sg8_z01: Vec<Pid55616Sg8Z01>,
    pub sg8_z44: Vec<Pid55616Sg8Z44>,
    pub sg8_z40: Vec<Pid55616Sg8Z40>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg6 {
    pub dtm: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg8Z01 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55616Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z40
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg8Z40 {
    pub pia: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z44
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616Sg8Z44 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55616Sg10>,
}

/// PID 55616: Änderung Daten der MaLo
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55616 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55616Sg2>,
    pub sg4: Vec<Pid55616Sg4>,
}

impl Pid55616Sg10 {
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

impl Pid55616Sg12Ddo {
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

impl Pid55616Sg12Dp {
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

impl Pid55616Sg12Eo {
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

impl Pid55616Sg12Z25 {
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

impl Pid55616Sg12Z26 {
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

impl Pid55616Sg2 {
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
        while let Some(item) = Pid55616Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid55616Sg3Ic {
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

impl Pid55616Sg4 {
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
        let mut sg12_z25 = Vec::new();
        while let Some(item) = Pid55616Sg12Z25::from_segments(segments, cursor) {
            sg12_z25.push(item);
        }
        let mut sg12_z26 = Vec::new();
        while let Some(item) = Pid55616Sg12Z26::from_segments(segments, cursor) {
            sg12_z26.push(item);
        }
        let mut sg12_eo = Vec::new();
        while let Some(item) = Pid55616Sg12Eo::from_segments(segments, cursor) {
            sg12_eo.push(item);
        }
        let mut sg12_ddo = Vec::new();
        while let Some(item) = Pid55616Sg12Ddo::from_segments(segments, cursor) {
            sg12_ddo.push(item);
        }
        let mut sg12_dp = Vec::new();
        while let Some(item) = Pid55616Sg12Dp::from_segments(segments, cursor) {
            sg12_dp.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55616Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55616Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z01 = Vec::new();
        while let Some(item) = Pid55616Sg8Z01::from_segments(segments, cursor) {
            sg8_z01.push(item);
        }
        let mut sg8_z44 = Vec::new();
        while let Some(item) = Pid55616Sg8Z44::from_segments(segments, cursor) {
            sg8_z44.push(item);
        }
        let mut sg8_z40 = Vec::new();
        while let Some(item) = Pid55616Sg8Z40::from_segments(segments, cursor) {
            sg8_z40.push(item);
        }
        Some(Self {
            ide,
            sts,
            sg12_z25,
            sg12_z26,
            sg12_eo,
            sg12_ddo,
            sg12_dp,
            sg5_z16,
            sg6,
            sg8_z01,
            sg8_z44,
            sg8_z40,
        })
    }
}

impl Pid55616Sg5Z16 {
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

impl Pid55616Sg6 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            dtm,
            rff,
        })
    }
}

impl Pid55616Sg8Z01 {
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
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55616Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            seq,
            sg10,
        })
    }
}

impl Pid55616Sg8Z40 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
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
        Some(Self {
            pia,
            seq,
        })
    }
}

impl Pid55616Sg8Z44 {
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
        while let Some(item) = Pid55616Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55616 {
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
        while let Some(item) = Pid55616Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55616Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55616 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
