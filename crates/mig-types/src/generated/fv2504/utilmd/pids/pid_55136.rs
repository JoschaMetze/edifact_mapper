//! Auto-generated PID 55136 types.
//! Rückmeldung/Anfrage Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z47, Z48
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg12Z47Z48 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z49, Z50
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg12Z49Z50 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55136Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg4 {
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z47_z48: Vec<Pid55136Sg12Z47Z48>,
    pub sg12_z49_z50: Vec<Pid55136Sg12Z49Z50>,
    pub sg5_z16: Vec<Pid55136Sg5Z16>,
    pub sg6: Vec<Pid55136Sg6>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136Sg6 {
    pub dtm: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// PID 55136: Rückmeldung/Anfrage Daten der MaLo
/// Kommunikation: MSB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55136 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55136Sg2>,
    pub sg4: Vec<Pid55136Sg4>,
}

impl Pid55136Sg12Z47Z48 {
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

impl Pid55136Sg12Z49Z50 {
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

impl Pid55136Sg2 {
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
        while let Some(item) = Pid55136Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid55136Sg3Ic {
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

impl Pid55136Sg4 {
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
        let mut sg12_z47_z48 = Vec::new();
        while let Some(item) = Pid55136Sg12Z47Z48::from_segments(segments, cursor) {
            sg12_z47_z48.push(item);
        }
        let mut sg12_z49_z50 = Vec::new();
        while let Some(item) = Pid55136Sg12Z49Z50::from_segments(segments, cursor) {
            sg12_z49_z50.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55136Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55136Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        Some(Self {
            ide,
            sts,
            sg12_z47_z48,
            sg12_z49_z50,
            sg5_z16,
            sg6,
        })
    }
}

impl Pid55136Sg5Z16 {
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

impl Pid55136Sg6 {
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

impl Pid55136 {
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
        while let Some(item) = Pid55136Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55136Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55136 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
