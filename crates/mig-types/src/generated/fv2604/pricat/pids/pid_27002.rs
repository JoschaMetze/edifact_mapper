//! Auto-generated PID 27002 types.
//! Preisblätter MSB-Leistungen
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg1 {
    pub rff: Option<OwnedSegment>,
}

/// SG17 — Produktgruppen-Art, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg17 {
    pub pgi: Option<OwnedSegment>,
    pub sg36: Vec<Pid27002Sg36>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg4_ic: Vec<Pid27002Sg4Ic>,
}

/// SG36 — Positionsnummer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg36 {
    pub imd: Option<OwnedSegment>,
    pub lin: Option<OwnedSegment>,
    pub pia: Option<OwnedSegment>,
    pub sg40: Vec<Pid27002Sg40>,
}

/// SG40 — Preis, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg40 {
    pub pri: Option<OwnedSegment>,
    pub rng: Option<OwnedSegment>,
}

/// SG4 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg4Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG6 — Währungsverwendung, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002Sg6 {
    pub cux: Option<OwnedSegment>,
}

/// PID 27002: Preisblätter MSB-Leistungen
/// Kommunikation: MSB an LF / NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid27002 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid27002Sg1>,
    pub sg17: Vec<Pid27002Sg17>,
    pub sg2: Vec<Pid27002Sg2>,
    pub sg6: Vec<Pid27002Sg6>,
}

impl Pid27002Sg1 {
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

impl Pid27002Sg17 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pgi = if peek_is(segments, cursor, "PGI") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pgi.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg36 = Vec::new();
        while let Some(item) = Pid27002Sg36::from_segments(segments, cursor) {
            sg36.push(item);
        }
        Some(Self {
            pgi,
            sg36,
        })
    }
}

impl Pid27002Sg2 {
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
        let mut sg4_ic = Vec::new();
        while let Some(item) = Pid27002Sg4Ic::from_segments(segments, cursor) {
            sg4_ic.push(item);
        }
        Some(Self {
            nad,
            sg4_ic,
        })
    }
}

impl Pid27002Sg36 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let imd = if peek_is(segments, cursor, "IMD") {
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
        if imd.is_none() && lin.is_none() && pia.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg40 = Vec::new();
        while let Some(item) = Pid27002Sg40::from_segments(segments, cursor) {
            sg40.push(item);
        }
        Some(Self {
            imd,
            lin,
            pia,
            sg40,
        })
    }
}

impl Pid27002Sg40 {
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

impl Pid27002Sg4Ic {
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

impl Pid27002Sg6 {
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

impl Pid27002 {
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
        while let Some(item) = Pid27002Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg17 = Vec::new();
        while let Some(item) = Pid27002Sg17::from_segments(segments, &mut cursor) {
            sg17.push(item);
        }
        let mut sg2 = Vec::new();
        while let Some(item) = Pid27002Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid27002Sg6::from_segments(segments, &mut cursor) {
            sg6.push(item);
        }

        Ok(Pid27002 {
            bgm,
            dtm,
            unh,
            unt,
            sg1,
            sg17,
            sg2,
            sg6,
        })
    }
}
