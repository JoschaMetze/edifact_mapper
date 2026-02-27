//! Auto-generated PID 44116 types.
//! Änderung vom MSB mit Abhängig keiten
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid44116Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg5_172: Vec<Pid44116Sg5172>,
    pub sg6: Vec<Pid44116Sg6>,
    pub sg8_z03: Vec<Pid44116Sg8Z03>,
    pub sg8_z50: Vec<Pid44116Sg8Z50>,
    pub sg8_z09: Vec<Pid44116Sg8Z09>,
    pub sg8_z20: Vec<Pid44116Sg8Z20>,
    pub sg8_z05: Vec<Pid44116Sg8Z05>,
    pub sg8_z13: Vec<Pid44116Sg8Z13>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: 172
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg5172 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg8Z03 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44116Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z05
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg8Z05 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44116Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg8Z09 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44116Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z13
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg8Z13 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44116Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg8Z20 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44116Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z50
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116Sg8Z50 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44116Sg10>,
}

/// PID 44116: Änderung vom MSB mit Abhängig keiten
/// Kommunikation: MSB an NB [Verteiler]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44116 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid44116Sg2>,
    pub sg4: Vec<Pid44116Sg4>,
}

impl Pid44116Sg10 {
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

impl Pid44116Sg2 {
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
        while let Some(item) = Pid44116Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid44116Sg3Ic {
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

impl Pid44116Sg4 {
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
        let mut sg5_172 = Vec::new();
        while let Some(item) = Pid44116Sg5172::from_segments(segments, cursor) {
            sg5_172.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid44116Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z03 = Vec::new();
        while let Some(item) = Pid44116Sg8Z03::from_segments(segments, cursor) {
            sg8_z03.push(item);
        }
        let mut sg8_z50 = Vec::new();
        while let Some(item) = Pid44116Sg8Z50::from_segments(segments, cursor) {
            sg8_z50.push(item);
        }
        let mut sg8_z09 = Vec::new();
        while let Some(item) = Pid44116Sg8Z09::from_segments(segments, cursor) {
            sg8_z09.push(item);
        }
        let mut sg8_z20 = Vec::new();
        while let Some(item) = Pid44116Sg8Z20::from_segments(segments, cursor) {
            sg8_z20.push(item);
        }
        let mut sg8_z05 = Vec::new();
        while let Some(item) = Pid44116Sg8Z05::from_segments(segments, cursor) {
            sg8_z05.push(item);
        }
        let mut sg8_z13 = Vec::new();
        while let Some(item) = Pid44116Sg8Z13::from_segments(segments, cursor) {
            sg8_z13.push(item);
        }
        Some(Self {
            dtm,
            ide,
            sts,
            sg5_172,
            sg6,
            sg8_z03,
            sg8_z50,
            sg8_z09,
            sg8_z20,
            sg8_z05,
            sg8_z13,
        })
    }
}

impl Pid44116Sg5172 {
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

impl Pid44116Sg6 {
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

impl Pid44116Sg8Z03 {
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
        while let Some(item) = Pid44116Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid44116Sg8Z05 {
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
        while let Some(item) = Pid44116Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid44116Sg8Z09 {
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
        while let Some(item) = Pid44116Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid44116Sg8Z13 {
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
        while let Some(item) = Pid44116Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            seq,
            sg10,
        })
    }
}

impl Pid44116Sg8Z20 {
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
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid44116Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid44116Sg8Z50 {
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
        while let Some(item) = Pid44116Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            seq,
            sg10,
        })
    }
}

impl Pid44116 {
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
        while let Some(item) = Pid44116Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid44116Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid44116 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
