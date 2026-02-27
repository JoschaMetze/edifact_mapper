//! Auto-generated PID 21043 types.
//! Bestellungsantwort / -mitteilung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21043Sg1 {
    pub nad: Option<OwnedSegment>,
    pub sg2_ic: Vec<Pid21043Sg2Ic>,
}

/// SG14
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21043Sg14 {
    pub cni: Option<OwnedSegment>,
    pub sg15: Vec<Pid21043Sg15>,
}

/// SG15 — Statuskategorie, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21043Sg15 {
    pub dtm: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg25: Vec<Pid21043Sg25>,
}

/// SG25
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21043Sg25 {
    pub ftx: Option<OwnedSegment>,
    pub gid: Option<OwnedSegment>,
}

/// SG2 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21043Sg2Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// PID 21043: Bestellungsantwort / -mitteilung
/// Kommunikation: MSB an LF / MSB / NB NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid21043 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid21043Sg1>,
    pub sg14: Vec<Pid21043Sg14>,
}

impl Pid21043Sg1 {
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
        let mut sg2_ic = Vec::new();
        while let Some(item) = Pid21043Sg2Ic::from_segments(segments, cursor) {
            sg2_ic.push(item);
        }
        Some(Self {
            nad,
            sg2_ic,
        })
    }
}

impl Pid21043Sg14 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cni = if peek_is(segments, cursor, "CNI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cni.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg15 = Vec::new();
        while let Some(item) = Pid21043Sg15::from_segments(segments, cursor) {
            sg15.push(item);
        }
        Some(Self {
            cni,
            sg15,
        })
    }
}

impl Pid21043Sg15 {
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
        let sts = if peek_is(segments, cursor, "STS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && rff.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg25 = Vec::new();
        while let Some(item) = Pid21043Sg25::from_segments(segments, cursor) {
            sg25.push(item);
        }
        Some(Self {
            dtm,
            rff,
            sts,
            sg25,
        })
    }
}

impl Pid21043Sg25 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let ftx = if peek_is(segments, cursor, "FTX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let gid = if peek_is(segments, cursor, "GID") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ftx.is_none() && gid.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ftx,
            gid,
        })
    }
}

impl Pid21043Sg2Ic {
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

impl Pid21043 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid21043Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg14 = Vec::new();
        while let Some(item) = Pid21043Sg14::from_segments(segments, &mut cursor) {
            sg14.push(item);
        }

        Ok(Pid21043 {
            bgm,
            dtm,
            unh,
            unt,
            sg1,
            sg14,
        })
    }
}
