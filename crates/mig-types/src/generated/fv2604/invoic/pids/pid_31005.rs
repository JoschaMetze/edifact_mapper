//! Auto-generated PID 31005 types.
//! MMM-Rechnung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg2 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
    pub sg3: Vec<Pid31005Sg3>,
    pub sg5_ic: Vec<Pid31005Sg5Ic>,
}

/// SG26 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg26 {
    pub dtm: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub qty: Option<OwnedSegment>,
    pub sg27: Vec<Pid31005Sg27>,
    pub sg29: Vec<Pid31005Sg29>,
    pub sg34_7: Vec<Pid31005Sg347>,
}

/// SG27 — Geldbetrag, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg27 {
    pub moa: Option<OwnedSegment>,
}

/// SG29 — Preis, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg29 {
    pub pri: Option<OwnedSegment>,
}

/// SG3 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg3 {
    pub rff: Option<OwnedSegment>,
}

/// SG34 — Zoll-/Steuer-/Gebührenfunktion, Qualifier
/// Qualifiers: 7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg347 {
    pub tax: Option<OwnedSegment>,
}

/// SG50 — Geldbetrag, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg50 {
    pub moa: Option<OwnedSegment>,
}

/// SG52 — Zoll-/Steuer-/Gebührenfunktion, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg52 {
    pub moa: Option<OwnedSegment>,
    pub tax: Option<OwnedSegment>,
}

/// SG5 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg5Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG7 — Währungsverwendung, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg7 {
    pub cux: Option<OwnedSegment>,
}

/// SG8 — Zahlungsbedingung, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005Sg8 {
    pub dtm: Option<OwnedSegment>,
    pub pyt: Option<OwnedSegment>,
}

/// PID 31005: MMM-Rechnung
/// Kommunikation: ReErst an ReEmpf
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid31005 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub ftx: OwnedSegment,
    pub imd: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid31005Sg1>,
    pub sg2: Vec<Pid31005Sg2>,
    pub sg26: Vec<Pid31005Sg26>,
    pub sg50: Vec<Pid31005Sg50>,
    pub sg52: Vec<Pid31005Sg52>,
    pub sg7: Vec<Pid31005Sg7>,
    pub sg8: Vec<Pid31005Sg8>,
}

impl Pid31005Sg1 {
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

impl Pid31005Sg2 {
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
        let mut sg3 = Vec::new();
        while let Some(item) = Pid31005Sg3::from_segments(segments, cursor) {
            sg3.push(item);
        }
        let mut sg5_ic = Vec::new();
        while let Some(item) = Pid31005Sg5Ic::from_segments(segments, cursor) {
            sg5_ic.push(item);
        }
        Some(Self {
            loc,
            nad,
            sg3,
            sg5_ic,
        })
    }
}

impl Pid31005Sg26 {
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
        let lin = if peek_is(segments, cursor, "LIN") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let qty = if peek_is(segments, cursor, "QTY") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && lin.is_none() && qty.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg27 = Vec::new();
        while let Some(item) = Pid31005Sg27::from_segments(segments, cursor) {
            sg27.push(item);
        }
        let mut sg29 = Vec::new();
        while let Some(item) = Pid31005Sg29::from_segments(segments, cursor) {
            sg29.push(item);
        }
        let mut sg34_7 = Vec::new();
        while let Some(item) = Pid31005Sg347::from_segments(segments, cursor) {
            sg34_7.push(item);
        }
        Some(Self {
            dtm,
            lin,
            qty,
            sg27,
            sg29,
            sg34_7,
        })
    }
}

impl Pid31005Sg27 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let moa = if peek_is(segments, cursor, "MOA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if moa.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            moa,
        })
    }
}

impl Pid31005Sg29 {
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
        if pri.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            pri,
        })
    }
}

impl Pid31005Sg3 {
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

impl Pid31005Sg347 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let tax = if peek_is(segments, cursor, "TAX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if tax.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            tax,
        })
    }
}

impl Pid31005Sg50 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let moa = if peek_is(segments, cursor, "MOA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if moa.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            moa,
        })
    }
}

impl Pid31005Sg52 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let moa = if peek_is(segments, cursor, "MOA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let tax = if peek_is(segments, cursor, "TAX") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if moa.is_none() && tax.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            moa,
            tax,
        })
    }
}

impl Pid31005Sg5Ic {
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

impl Pid31005Sg7 {
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

impl Pid31005Sg8 {
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
        let pyt = if peek_is(segments, cursor, "PYT") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && pyt.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            dtm,
            pyt,
        })
    }
}

impl Pid31005 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let ftx = expect_segment(segments, &mut cursor, "FTX")?.clone();
        let imd = expect_segment(segments, &mut cursor, "IMD")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let uns = expect_segment(segments, &mut cursor, "UNS")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid31005Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid31005Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg26 = Vec::new();
        while let Some(item) = Pid31005Sg26::from_segments(segments, &mut cursor) {
            sg26.push(item);
        }
        let mut sg50 = Vec::new();
        while let Some(item) = Pid31005Sg50::from_segments(segments, &mut cursor) {
            sg50.push(item);
        }
        let mut sg52 = Vec::new();
        while let Some(item) = Pid31005Sg52::from_segments(segments, &mut cursor) {
            sg52.push(item);
        }
        let mut sg7 = Vec::new();
        while let Some(item) = Pid31005Sg7::from_segments(segments, &mut cursor) {
            sg7.push(item);
        }
        let mut sg8 = Vec::new();
        while let Some(item) = Pid31005Sg8::from_segments(segments, &mut cursor) {
            sg8.push(item);
        }

        Ok(Pid31005 {
            bgm,
            dtm,
            ftx,
            imd,
            unh,
            uns,
            unt,
            sg1,
            sg2,
            sg26,
            sg50,
            sg52,
            sg7,
            sg8,
        })
    }
}
