//! Auto-generated PID 17128 types.
//! Reklamation einer Konfiguration
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg5_ic: Vec<Pid17128Sg5Ic>,
}

/// SG29 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg29 {
    pub ftx: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
    pub sg30_z52: Vec<Pid17128Sg30Z52>,
    pub sg30_z53: Vec<Pid17128Sg30Z53>,
    pub sg30_z54: Vec<Pid17128Sg30Z54>,
    pub sg30_z60: Vec<Pid17128Sg30Z60>,
}

/// SG30 — Klassentyp, Code
/// Qualifiers: Z52
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg30Z52 {
    pub cci: Option<OwnedSegment>,
}

/// SG30 — Klassentyp, Code
/// Qualifiers: Z53
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg30Z53 {
    pub cci: Option<OwnedSegment>,
}

/// SG30 — Klassentyp, Code
/// Qualifiers: Z54
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg30Z54 {
    pub cci: Option<OwnedSegment>,
}

/// SG30 — Klassentyp, Code
/// Qualifiers: Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg30Z60 {
    pub cci: Option<OwnedSegment>,
}

/// SG5 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128Sg5Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// PID 17128: Reklamation einer Konfiguration
/// Kommunikation: NB, LF, MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17128 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid17128Sg1>,
    pub sg2: Vec<Pid17128Sg2>,
    pub sg29: Vec<Pid17128Sg29>,
}

impl Pid17128Sg1 {
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

impl Pid17128Sg2 {
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
        let mut sg5_ic = Vec::new();
        while let Some(item) = Pid17128Sg5Ic::from_segments(segments, cursor) {
            sg5_ic.push(item);
        }
        Some(Self {
            nad,
            sg5_ic,
        })
    }
}

impl Pid17128Sg29 {
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
        if ftx.is_none() && lin.is_none() && pia.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg30_z52 = Vec::new();
        while let Some(item) = Pid17128Sg30Z52::from_segments(segments, cursor) {
            sg30_z52.push(item);
        }
        let mut sg30_z53 = Vec::new();
        while let Some(item) = Pid17128Sg30Z53::from_segments(segments, cursor) {
            sg30_z53.push(item);
        }
        let mut sg30_z54 = Vec::new();
        while let Some(item) = Pid17128Sg30Z54::from_segments(segments, cursor) {
            sg30_z54.push(item);
        }
        let mut sg30_z60 = Vec::new();
        while let Some(item) = Pid17128Sg30Z60::from_segments(segments, cursor) {
            sg30_z60.push(item);
        }
        Some(Self {
            ftx,
            lin,
            pia,
            sg30_z52,
            sg30_z53,
            sg30_z54,
            sg30_z60,
        })
    }
}

impl Pid17128Sg30Z52 {
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

impl Pid17128Sg30Z53 {
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

impl Pid17128Sg30Z54 {
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

impl Pid17128Sg30Z60 {
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

impl Pid17128Sg5Ic {
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

impl Pid17128 {
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
        while let Some(item) = Pid17128Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid17128Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg29 = Vec::new();
        while let Some(item) = Pid17128Sg29::from_segments(segments, &mut cursor) {
            sg29.push(item);
        }

        Ok(Pid17128 {
            bgm,
            dtm,
            unh,
            uns,
            unt,
            sg1,
            sg2,
            sg29,
        })
    }
}
