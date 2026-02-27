//! Auto-generated PID 44010 types.
//! Abmelde-anfrage des NB
//! Do not edit manually.

use crate::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};
use crate::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: VY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg12Vy {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg12Z09 {
    pub nad: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid44010Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z09: Vec<Pid44010Sg12Z09>,
    pub sg12_vy: Vec<Pid44010Sg12Vy>,
    pub sg5_172: Vec<Pid44010Sg5172>,
    pub sg6: Vec<Pid44010Sg6>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: 172
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg5172 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// PID 44010: Abmelde-anfrage des NB
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44010 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid44010Sg2>,
    pub sg4: Vec<Pid44010Sg4>,
}

impl Pid44010Sg12Vy {
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
        Some(Self { nad })
    }
}

impl Pid44010Sg12Z09 {
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
        Some(Self { nad })
    }
}

impl Pid44010Sg2 {
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
        while let Some(item) = Pid44010Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self { nad, sg3_ic })
    }
}

impl Pid44010Sg3Ic {
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

impl Pid44010Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid44010Sg12Z09::from_segments(segments, cursor) {
            sg12_z09.push(item);
        }
        let mut sg12_vy = Vec::new();
        while let Some(item) = Pid44010Sg12Vy::from_segments(segments, cursor) {
            sg12_vy.push(item);
        }
        let mut sg5_172 = Vec::new();
        while let Some(item) = Pid44010Sg5172::from_segments(segments, cursor) {
            sg5_172.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid44010Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        Some(Self {
            dtm,
            ide,
            sts,
            sg12_z09,
            sg12_vy,
            sg5_172,
            sg6,
        })
    }
}

impl Pid44010Sg5172 {
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

impl Pid44010Sg6 {
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

impl Pid44010 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid44010Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid44010Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid44010 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
