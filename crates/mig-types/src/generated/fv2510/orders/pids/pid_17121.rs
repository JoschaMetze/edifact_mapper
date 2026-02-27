//! Auto-generated PID 17121 types.
//! Bestellung Änderung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg2 {
    pub loc: Option<OwnedSegment>,
    pub nad: Option<OwnedSegment>,
    pub sg5_ic: Vec<Pid17121Sg5Ic>,
}

/// SG29 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg29 {
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
    pub sg30_z39_z41: Vec<Pid17121Sg30Z39Z41>,
    pub sg30: Vec<Pid17121Sg30>,
    pub sg30_z35: Vec<Pid17121Sg30Z35>,
    pub sg34: Vec<Pid17121Sg34>,
}

/// SG30 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg30 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG30 — Klassentyp, Code
/// Qualifiers: Z35
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg30Z35 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG30 — Klassentyp, Code
/// Qualifiers: Z39, Z41
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg30Z39Z41 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG34 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg34 {
    pub rff: Option<OwnedSegment>,
}

/// SG5 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121Sg5Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// PID 17121: Bestellung Änderung
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid17121 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub imd: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid17121Sg1>,
    pub sg2: Vec<Pid17121Sg2>,
    pub sg29: Vec<Pid17121Sg29>,
}

impl Pid17121Sg1 {
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

impl Pid17121Sg2 {
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
        let mut sg5_ic = Vec::new();
        while let Some(item) = Pid17121Sg5Ic::from_segments(segments, cursor) {
            sg5_ic.push(item);
        }
        Some(Self {
            loc,
            nad,
            sg5_ic,
        })
    }
}

impl Pid17121Sg29 {
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
        let mut sg30_z39_z41 = Vec::new();
        while let Some(item) = Pid17121Sg30Z39Z41::from_segments(segments, cursor) {
            sg30_z39_z41.push(item);
        }
        let mut sg30 = Vec::new();
        while let Some(item) = Pid17121Sg30::from_segments(segments, cursor) {
            sg30.push(item);
        }
        let mut sg30_z35 = Vec::new();
        while let Some(item) = Pid17121Sg30Z35::from_segments(segments, cursor) {
            sg30_z35.push(item);
        }
        let mut sg34 = Vec::new();
        while let Some(item) = Pid17121Sg34::from_segments(segments, cursor) {
            sg34.push(item);
        }
        Some(Self {
            lin,
            pia,
            sg30_z39_z41,
            sg30,
            sg30_z35,
            sg34,
        })
    }
}

impl Pid17121Sg30 {
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

impl Pid17121Sg30Z35 {
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

impl Pid17121Sg30Z39Z41 {
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

impl Pid17121Sg34 {
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

impl Pid17121Sg5Ic {
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

impl Pid17121 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(
        segments: &[OwnedSegment],
    ) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let imd = expect_segment(segments, &mut cursor, "IMD")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let uns = expect_segment(segments, &mut cursor, "UNS")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg1 = Vec::new();
        while let Some(item) = Pid17121Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid17121Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg29 = Vec::new();
        while let Some(item) = Pid17121Sg29::from_segments(segments, &mut cursor) {
            sg29.push(item);
        }

        Ok(Pid17121 {
            bgm,
            dtm,
            imd,
            unh,
            uns,
            unt,
            sg1,
            sg2,
            sg29,
        })
    }
}
