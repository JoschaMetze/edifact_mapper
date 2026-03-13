//! Auto-generated PID 44097 types.
//! Deklarationsliste
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097Sg10 {
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: VY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097Sg12Vy {
    pub nad: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097Sg2 {
    pub nad: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sg12_vy: Vec<Pid44097Sg12Vy>,
    pub sg6: Vec<Pid44097Sg6>,
    pub sg8_z26: Vec<Pid44097Sg8Z26>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z26
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097Sg8Z26 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid44097Sg10>,
}

/// PID 44097: Deklarationsliste
/// Kommunikation: MGV an BKV
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid44097 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid44097Sg2>,
    pub sg4: Vec<Pid44097Sg4>,
}

impl Pid44097Sg10 {
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

impl Pid44097Sg12Vy {
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

impl Pid44097Sg2 {
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

impl Pid44097Sg4 {
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
        if dtm.is_none() && ide.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg12_vy = Vec::new();
        while let Some(item) = Pid44097Sg12Vy::from_segments(segments, cursor) {
            sg12_vy.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid44097Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z26 = Vec::new();
        while let Some(item) = Pid44097Sg8Z26::from_segments(segments, cursor) {
            sg8_z26.push(item);
        }
        Some(Self {
            dtm,
            ide,
            sg12_vy,
            sg6,
            sg8_z26,
        })
    }
}

impl Pid44097Sg6 {
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

impl Pid44097Sg8Z26 {
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
        while let Some(item) = Pid44097Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            seq,
            sg10,
        })
    }
}

impl Pid44097 {
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
        while let Some(item) = Pid44097Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid44097Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid44097 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
