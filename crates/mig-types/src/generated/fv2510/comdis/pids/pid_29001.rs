//! Auto-generated PID 29001 types.
//! Ablehnung REMADV
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29001Sg1 {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
}

/// SG2 — Dokumentenname, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29001Sg2 {
    pub doc: Option<OwnedSegment>,
    pub moa: Option<OwnedSegment>,
    pub sg3: Vec<Pid29001Sg3>,
    pub sg3_e_0265_e_0271_e_0274_e_0504_e_0516_e_0520_e_0567_e_1008_s_0109: Vec<Pid29001Sg3E_0265E_0271E_0274E_0504E_0516E_0520E_0567E_1008S_0109>,
}

/// SG3 — Anpassungsgrund, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29001Sg3 {
    pub ajt: Option<OwnedSegment>,
}

/// SG3 — Anpassungsgrund, Code
/// Qualifiers: E_0265, E_0271, E_0274, E_0504, E_0516, E_0520, E_0567, E_1008, S_0109
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29001Sg3E_0265E_0271E_0274E_0504E_0516E_0520E_0567E_1008S_0109 {
    pub ajt: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
}

/// PID 29001: Ablehnung REMADV
/// Kommunikation: NB an LF MSB an LF, NB, ESA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid29001 {
    pub bgm: OwnedSegment,
    pub cux: OwnedSegment,
    pub dtm: OwnedSegment,
    pub rff: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid29001Sg1>,
    pub sg2: Vec<Pid29001Sg2>,
}

impl Pid29001Sg1 {
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
        let nad = if peek_is(segments, cursor, "NAD") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if com.is_none() && cta.is_none() && nad.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            com,
            cta,
            nad,
        })
    }
}

impl Pid29001Sg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let doc = if peek_is(segments, cursor, "DOC") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let moa = if peek_is(segments, cursor, "MOA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if doc.is_none() && moa.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg3 = Vec::new();
        while let Some(item) = Pid29001Sg3::from_segments(segments, cursor) {
            sg3.push(item);
        }
        let mut sg3_e_0265_e_0271_e_0274_e_0504_e_0516_e_0520_e_0567_e_1008_s_0109 = Vec::new();
        while let Some(item) = Pid29001Sg3E_0265E_0271E_0274E_0504E_0516E_0520E_0567E_1008S_0109::from_segments(segments, cursor) {
            sg3_e_0265_e_0271_e_0274_e_0504_e_0516_e_0520_e_0567_e_1008_s_0109.push(item);
        }
        Some(Self {
            doc,
            moa,
            sg3,
            sg3_e_0265_e_0271_e_0274_e_0504_e_0516_e_0520_e_0567_e_1008_s_0109,
        })
    }
}

impl Pid29001Sg3 {
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

impl Pid29001Sg3E_0265E_0271E_0274E_0504E_0516E_0520E_0567E_1008S_0109 {
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
        let ftx = if peek_is(segments, cursor, "FTX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ajt.is_none() && ftx.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ajt,
            ftx,
        })
    }
}

impl Pid29001 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let cux = expect_segment(segments, &mut cursor, "CUX")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let rff = expect_segment(segments, &mut cursor, "RFF")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid29001Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid29001Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }

        Ok(Pid29001 {
            bgm,
            cux,
            dtm,
            rff,
            unh,
            unt,
            sg1,
            sg2,
        })
    }
}
