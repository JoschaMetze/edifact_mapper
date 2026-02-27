//! Auto-generated PID 19101 types.
//! Ablehnung der Anfrage Stammdaten
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid19101Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Code des Prüfschritts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid19101Sg2 {
    pub ajt: Option<OwnedSegment>,
}

/// SG3 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid19101Sg3 {
    pub nad: Option<OwnedSegment>,
    pub sg6_ic: Vec<Pid19101Sg6Ic>,
}

/// SG6 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid19101Sg6Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// PID 19101: Ablehnung der Anfrage Stammdaten
/// Kommunikation: NB an LF, MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid19101 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid19101Sg1>,
    pub sg2: Vec<Pid19101Sg2>,
    pub sg3: Vec<Pid19101Sg3>,
}

impl Pid19101Sg1 {
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

impl Pid19101Sg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ajt = if peek_is(segments, cursor, "AJT") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ajt.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ajt,
        })
    }
}

impl Pid19101Sg3 {
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
        let mut sg6_ic = Vec::new();
        while let Some(item) = Pid19101Sg6Ic::from_segments(segments, cursor) {
            sg6_ic.push(item);
        }
        Some(Self {
            nad,
            sg6_ic,
        })
    }
}

impl Pid19101Sg6Ic {
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

impl Pid19101 {
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
        while let Some(item) = Pid19101Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid19101Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg3 = Vec::new();
        while let Some(item) = Pid19101Sg3::from_segments(segments, &mut cursor) {
            sg3.push(item);
        }

        Ok(Pid19101 {
            bgm,
            dtm,
            unh,
            uns,
            unt,
            sg1,
            sg2,
            sg3,
        })
    }
}
