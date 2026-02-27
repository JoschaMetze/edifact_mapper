//! Auto-generated PID 33003 types.
//! Strom Abweisung Kopf und Summe
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003Sg1 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid33003Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Währungsverwendung, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003Sg4 {
    pub cux: Option<OwnedSegment>,
}

/// SG5 — Dokumentenname, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003Sg5 {
    pub doc: Option<OwnedSegment>,
    pub dtm: Option<OwnedSegment>,
    pub moa: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub sg7: Vec<Pid33003Sg7>,
    pub sg7_e_0210_e_0259_e_0264_e_0266_e_0406_e_0407_e_0515_e_0517_e_0519_e_0521_e_0566_e_0568: Vec<Pid33003Sg7E_0210E_0259E_0264E_0266E_0406E_0407E_0515E_0517E_0519E_0521E_0566E_0568>,
}

/// SG7 — Code des Prüfschritts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003Sg7 {
    pub ajt: Option<OwnedSegment>,
}

/// SG7 — Code des Prüfschritts
/// Qualifiers: E_0210, E_0259, E_0264, E_0266, E_0406, E_0407, E_0515, E_0517, E_0519, E_0521, E_0566, E_0568
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003Sg7E_0210E_0259E_0264E_0266E_0406E_0407E_0515E_0517E_0519E_0521E_0566E_0568 {
    pub ajt: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// PID 33003: Strom Abweisung Kopf und Summe
/// Kommunikation: ReEmpf an ReErst
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33003 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub moa: OwnedSegment,
    pub rff: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid33003Sg1>,
    pub sg4: Vec<Pid33003Sg4>,
    pub sg5: Vec<Pid33003Sg5>,
}

impl Pid33003Sg1 {
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
        while let Some(item) = Pid33003Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid33003Sg3Ic {
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

impl Pid33003Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let cux = if peek_is(segments, cursor, "CUX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if cux.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            cux,
        })
    }
}

impl Pid33003Sg5 {
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
        let dtm = if peek_is(segments, cursor, "DTM") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let moa = if peek_is(segments, cursor, "MOA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if doc.is_none() && dtm.is_none() && moa.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg7 = Vec::new();
        while let Some(item) = Pid33003Sg7::from_segments(segments, cursor) {
            sg7.push(item);
        }
        let mut sg7_e_0210_e_0259_e_0264_e_0266_e_0406_e_0407_e_0515_e_0517_e_0519_e_0521_e_0566_e_0568 = Vec::new();
        while let Some(item) = Pid33003Sg7E_0210E_0259E_0264E_0266E_0406E_0407E_0515E_0517E_0519E_0521E_0566E_0568::from_segments(segments, cursor) {
            sg7_e_0210_e_0259_e_0264_e_0266_e_0406_e_0407_e_0515_e_0517_e_0519_e_0521_e_0566_e_0568.push(item);
        }
        Some(Self {
            doc,
            dtm,
            moa,
            rff,
            sg7,
            sg7_e_0210_e_0259_e_0264_e_0266_e_0406_e_0407_e_0515_e_0517_e_0519_e_0521_e_0566_e_0568,
        })
    }
}

impl Pid33003Sg7 {
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

impl Pid33003Sg7E_0210E_0259E_0264E_0266E_0406E_0407E_0515E_0517E_0519E_0521E_0566E_0568 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if ajt.is_none() && ftx.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            ajt,
            ftx,
            rff,
        })
    }
}

impl Pid33003 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let moa = expect_segment(segments, &mut cursor, "MOA")?.clone();
        let rff = expect_segment(segments, &mut cursor, "RFF")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let uns = expect_segment(segments, &mut cursor, "UNS")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid33003Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid33003Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }
        let mut sg5 = Vec::new();
        while let Some(item) = Pid33003Sg5::from_segments(segments, &mut cursor) {
            sg5.push(item);
        }

        Ok(Pid33003 {
            bgm,
            dtm,
            moa,
            rff,
            unh,
            uns,
            unt,
            sg1,
            sg4,
            sg5,
        })
    }
}
