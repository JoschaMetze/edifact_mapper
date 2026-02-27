//! Auto-generated PID 15005 types.
//! Angebot Änderung Technik
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15005Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG11 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15005Sg11 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
    pub sg14_ic: Vec<Pid15005Sg14Ic>,
}

/// SG14 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15005Sg14Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG27 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15005Sg27 {
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
    pub sg31: Vec<Pid15005Sg31>,
}

/// SG31 — Preis, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15005Sg31 {
    pub pri: Option<OwnedSegment>,
    pub rng: Option<OwnedSegment>,
}

/// PID 15005: Angebot Änderung Technik
/// Kommunikation: MSB an NB, LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid15005 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid15005Sg1>,
    pub sg11: Vec<Pid15005Sg11>,
    pub sg27: Vec<Pid15005Sg27>,
}

impl Pid15005Sg1 {
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

impl Pid15005Sg11 {
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
        while let Some(item) = Pid15005Sg14Ic::from_segments(segments, cursor) {
            sg14_ic.push(item);
        }
        Some(Self {
            loc,
            nad,
            sg14_ic,
        })
    }
}

impl Pid15005Sg14Ic {
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

impl Pid15005Sg27 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let lin = if peek_is(segments, cursor, "LIN") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if lin.is_none() && pia.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg31 = Vec::new();
        while let Some(item) = Pid15005Sg31::from_segments(segments, cursor) {
            sg31.push(item);
        }
        Some(Self {
            lin,
            pia,
            sg31,
        })
    }
}

impl Pid15005Sg31 {
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
        let rng = if peek_is(segments, cursor, "RNG") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pri.is_none() && rng.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            pri,
            rng,
        })
    }
}

impl Pid15005 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let uns = expect_segment(segments, &mut cursor, "UNS")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid15005Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg11 = Vec::new();
        while let Some(item) = Pid15005Sg11::from_segments(segments, &mut cursor) {
            sg11.push(item);
        }
        let mut sg27 = Vec::new();
        while let Some(item) = Pid15005Sg27::from_segments(segments, &mut cursor) {
            sg27.push(item);
        }

        Ok(Pid15005 {
            bgm,
            dtm,
            unh,
            uns,
            unt,
            sg1,
            sg11,
            sg27,
        })
    }
}
