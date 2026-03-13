//! Auto-generated PID 55169 types.
//! Bestätigung Verpflicht-ungsanfrage
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg12Z03 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg12Z04 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z05
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg12Z05 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z07
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg12Z07 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg12Z08 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg12Z09 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55169Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z09: Vec<Pid55169Sg12Z09>,
    pub sg12_z04: Vec<Pid55169Sg12Z04>,
    pub sg12_z07: Vec<Pid55169Sg12Z07>,
    pub sg12_z08: Vec<Pid55169Sg12Z08>,
    pub sg12_z03: Vec<Pid55169Sg12Z03>,
    pub sg12_z05: Vec<Pid55169Sg12Z05>,
    pub sg5_z18: Vec<Pid55169Sg5Z18>,
    pub sg5_z16: Vec<Pid55169Sg5Z16>,
    pub sg5_z20: Vec<Pid55169Sg5Z20>,
    pub sg5_z19: Vec<Pid55169Sg5Z19>,
    pub sg5_z21: Vec<Pid55169Sg5Z21>,
    pub sg5_z17: Vec<Pid55169Sg5Z17>,
    pub sg6: Vec<Pid55169Sg6>,
    pub sg8_z78: Vec<Pid55169Sg8Z78>,
    pub sg8_z58: Vec<Pid55169Sg8Z58>,
    pub sg8_z51: Vec<Pid55169Sg8Z51>,
    pub sg8_z57: Vec<Pid55169Sg8Z57>,
    pub sg8_z60: Vec<Pid55169Sg8Z60>,
    pub sg8_z01: Vec<Pid55169Sg8Z01>,
    pub sg8_z27: Vec<Pid55169Sg8Z27>,
    pub sg8_z02: Vec<Pid55169Sg8Z02>,
    pub sg8_z59: Vec<Pid55169Sg8Z59>,
    pub sg8_z15: Vec<Pid55169Sg8Z15>,
    pub sg8_z16: Vec<Pid55169Sg8Z16>,
    pub sg8_z52: Vec<Pid55169Sg8Z52>,
    pub sg8_z62: Vec<Pid55169Sg8Z62>,
    pub sg8_z61: Vec<Pid55169Sg8Z61>,
    pub sg8_z18: Vec<Pid55169Sg8Z18>,
    pub sg8_z19: Vec<Pid55169Sg8Z19>,
    pub sg8_z03: Vec<Pid55169Sg8Z03>,
    pub sg8_z20: Vec<Pid55169Sg8Z20>,
    pub sg8_z04: Vec<Pid55169Sg8Z04>,
    pub sg8_z05: Vec<Pid55169Sg8Z05>,
    pub sg8_z06: Vec<Pid55169Sg8Z06>,
    pub sg8_z13: Vec<Pid55169Sg8Z13>,
    pub sg8_z14: Vec<Pid55169Sg8Z14>,
    pub sg8_z21: Vec<Pid55169Sg8Z21>,
    pub sg8_z08: Vec<Pid55169Sg8Z08>,
    pub sg8_z38: Vec<Pid55169Sg8Z38>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg5Z17 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg5Z18 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg5Z19 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg5Z20 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg5Z21 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg6 {
    pub dtm: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z01 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
    pub sg9: Vec<Pid55169Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z02
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z02 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z03 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z04 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z05
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z05 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z06
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z06 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z08 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z13
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z13 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z14
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z14 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z15 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
    pub sg9: Vec<Pid55169Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z16 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z18 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z19 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z20 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z21 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z27
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z27 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z38
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z38 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z51
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z51 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z52
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z52 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
    pub sg9: Vec<Pid55169Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z57
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z57 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z58
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z58 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z59
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z59 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z60 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z61
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z61 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z62
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z62 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55169Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z78
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg8Z78 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169Sg9 {
    pub qty: Option<OwnedSegment>,
}

/// PID 55169: Bestätigung Verpflicht-ungsanfrage
/// Kommunikation: gMSB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55169 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55169Sg2>,
    pub sg4: Vec<Pid55169Sg4>,
}

impl Pid55169Sg10 {
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

impl Pid55169Sg12Z03 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
            rff,
        })
    }
}

impl Pid55169Sg12Z04 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
            rff,
        })
    }
}

impl Pid55169Sg12Z05 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
            rff,
        })
    }
}

impl Pid55169Sg12Z07 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
            rff,
        })
    }
}

impl Pid55169Sg12Z08 {
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

impl Pid55169Sg12Z09 {
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
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if nad.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            nad,
            rff,
        })
    }
}

impl Pid55169Sg2 {
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
        while let Some(item) = Pid55169Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid55169Sg3Ic {
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

impl Pid55169Sg4 {
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
        let sts = if peek_is(segments, cursor, "STS") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if dtm.is_none() && ide.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg12_z09 = Vec::new();
        while let Some(item) = Pid55169Sg12Z09::from_segments(segments, cursor) {
            sg12_z09.push(item);
        }
        let mut sg12_z04 = Vec::new();
        while let Some(item) = Pid55169Sg12Z04::from_segments(segments, cursor) {
            sg12_z04.push(item);
        }
        let mut sg12_z07 = Vec::new();
        while let Some(item) = Pid55169Sg12Z07::from_segments(segments, cursor) {
            sg12_z07.push(item);
        }
        let mut sg12_z08 = Vec::new();
        while let Some(item) = Pid55169Sg12Z08::from_segments(segments, cursor) {
            sg12_z08.push(item);
        }
        let mut sg12_z03 = Vec::new();
        while let Some(item) = Pid55169Sg12Z03::from_segments(segments, cursor) {
            sg12_z03.push(item);
        }
        let mut sg12_z05 = Vec::new();
        while let Some(item) = Pid55169Sg12Z05::from_segments(segments, cursor) {
            sg12_z05.push(item);
        }
        let mut sg5_z18 = Vec::new();
        while let Some(item) = Pid55169Sg5Z18::from_segments(segments, cursor) {
            sg5_z18.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55169Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg5_z20 = Vec::new();
        while let Some(item) = Pid55169Sg5Z20::from_segments(segments, cursor) {
            sg5_z20.push(item);
        }
        let mut sg5_z19 = Vec::new();
        while let Some(item) = Pid55169Sg5Z19::from_segments(segments, cursor) {
            sg5_z19.push(item);
        }
        let mut sg5_z21 = Vec::new();
        while let Some(item) = Pid55169Sg5Z21::from_segments(segments, cursor) {
            sg5_z21.push(item);
        }
        let mut sg5_z17 = Vec::new();
        while let Some(item) = Pid55169Sg5Z17::from_segments(segments, cursor) {
            sg5_z17.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55169Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_z78 = Vec::new();
        while let Some(item) = Pid55169Sg8Z78::from_segments(segments, cursor) {
            sg8_z78.push(item);
        }
        let mut sg8_z58 = Vec::new();
        while let Some(item) = Pid55169Sg8Z58::from_segments(segments, cursor) {
            sg8_z58.push(item);
        }
        let mut sg8_z51 = Vec::new();
        while let Some(item) = Pid55169Sg8Z51::from_segments(segments, cursor) {
            sg8_z51.push(item);
        }
        let mut sg8_z57 = Vec::new();
        while let Some(item) = Pid55169Sg8Z57::from_segments(segments, cursor) {
            sg8_z57.push(item);
        }
        let mut sg8_z60 = Vec::new();
        while let Some(item) = Pid55169Sg8Z60::from_segments(segments, cursor) {
            sg8_z60.push(item);
        }
        let mut sg8_z01 = Vec::new();
        while let Some(item) = Pid55169Sg8Z01::from_segments(segments, cursor) {
            sg8_z01.push(item);
        }
        let mut sg8_z27 = Vec::new();
        while let Some(item) = Pid55169Sg8Z27::from_segments(segments, cursor) {
            sg8_z27.push(item);
        }
        let mut sg8_z02 = Vec::new();
        while let Some(item) = Pid55169Sg8Z02::from_segments(segments, cursor) {
            sg8_z02.push(item);
        }
        let mut sg8_z59 = Vec::new();
        while let Some(item) = Pid55169Sg8Z59::from_segments(segments, cursor) {
            sg8_z59.push(item);
        }
        let mut sg8_z15 = Vec::new();
        while let Some(item) = Pid55169Sg8Z15::from_segments(segments, cursor) {
            sg8_z15.push(item);
        }
        let mut sg8_z16 = Vec::new();
        while let Some(item) = Pid55169Sg8Z16::from_segments(segments, cursor) {
            sg8_z16.push(item);
        }
        let mut sg8_z52 = Vec::new();
        while let Some(item) = Pid55169Sg8Z52::from_segments(segments, cursor) {
            sg8_z52.push(item);
        }
        let mut sg8_z62 = Vec::new();
        while let Some(item) = Pid55169Sg8Z62::from_segments(segments, cursor) {
            sg8_z62.push(item);
        }
        let mut sg8_z61 = Vec::new();
        while let Some(item) = Pid55169Sg8Z61::from_segments(segments, cursor) {
            sg8_z61.push(item);
        }
        let mut sg8_z18 = Vec::new();
        while let Some(item) = Pid55169Sg8Z18::from_segments(segments, cursor) {
            sg8_z18.push(item);
        }
        let mut sg8_z19 = Vec::new();
        while let Some(item) = Pid55169Sg8Z19::from_segments(segments, cursor) {
            sg8_z19.push(item);
        }
        let mut sg8_z03 = Vec::new();
        while let Some(item) = Pid55169Sg8Z03::from_segments(segments, cursor) {
            sg8_z03.push(item);
        }
        let mut sg8_z20 = Vec::new();
        while let Some(item) = Pid55169Sg8Z20::from_segments(segments, cursor) {
            sg8_z20.push(item);
        }
        let mut sg8_z04 = Vec::new();
        while let Some(item) = Pid55169Sg8Z04::from_segments(segments, cursor) {
            sg8_z04.push(item);
        }
        let mut sg8_z05 = Vec::new();
        while let Some(item) = Pid55169Sg8Z05::from_segments(segments, cursor) {
            sg8_z05.push(item);
        }
        let mut sg8_z06 = Vec::new();
        while let Some(item) = Pid55169Sg8Z06::from_segments(segments, cursor) {
            sg8_z06.push(item);
        }
        let mut sg8_z13 = Vec::new();
        while let Some(item) = Pid55169Sg8Z13::from_segments(segments, cursor) {
            sg8_z13.push(item);
        }
        let mut sg8_z14 = Vec::new();
        while let Some(item) = Pid55169Sg8Z14::from_segments(segments, cursor) {
            sg8_z14.push(item);
        }
        let mut sg8_z21 = Vec::new();
        while let Some(item) = Pid55169Sg8Z21::from_segments(segments, cursor) {
            sg8_z21.push(item);
        }
        let mut sg8_z08 = Vec::new();
        while let Some(item) = Pid55169Sg8Z08::from_segments(segments, cursor) {
            sg8_z08.push(item);
        }
        let mut sg8_z38 = Vec::new();
        while let Some(item) = Pid55169Sg8Z38::from_segments(segments, cursor) {
            sg8_z38.push(item);
        }
        Some(Self {
            dtm,
            ide,
            sts,
            sg12_z09,
            sg12_z04,
            sg12_z07,
            sg12_z08,
            sg12_z03,
            sg12_z05,
            sg5_z18,
            sg5_z16,
            sg5_z20,
            sg5_z19,
            sg5_z21,
            sg5_z17,
            sg6,
            sg8_z78,
            sg8_z58,
            sg8_z51,
            sg8_z57,
            sg8_z60,
            sg8_z01,
            sg8_z27,
            sg8_z02,
            sg8_z59,
            sg8_z15,
            sg8_z16,
            sg8_z52,
            sg8_z62,
            sg8_z61,
            sg8_z18,
            sg8_z19,
            sg8_z03,
            sg8_z20,
            sg8_z04,
            sg8_z05,
            sg8_z06,
            sg8_z13,
            sg8_z14,
            sg8_z21,
            sg8_z08,
            sg8_z38,
        })
    }
}

impl Pid55169Sg5Z16 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid55169Sg5Z17 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid55169Sg5Z18 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid55169Sg5Z19 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid55169Sg5Z20 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid55169Sg5Z21 {
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
        if loc.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            loc,
        })
    }
}

impl Pid55169Sg6 {
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
        if dtm.is_none() && rff.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            dtm,
            rff,
        })
    }
}

impl Pid55169Sg8Z01 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55169Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
            sg9,
        })
    }
}

impl Pid55169Sg8Z02 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z03 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z04 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z05 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z06 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z08 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z13 {
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
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z14 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z15 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55169Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
            sg9,
        })
    }
}

impl Pid55169Sg8Z16 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z18 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z19 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z20 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z21 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z27 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z38 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z51 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z52 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55169Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
            sg9,
        })
    }
}

impl Pid55169Sg8Z57 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z58 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            rff,
            seq,
        })
    }
}

impl Pid55169Sg8Z59 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z60 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z61 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let rff = if peek_is(segments, cursor, "RFF") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            pia,
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z62 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55169Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self {
            rff,
            seq,
            sg10,
        })
    }
}

impl Pid55169Sg8Z78 {
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
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if rff.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            rff,
            seq,
        })
    }
}

impl Pid55169Sg9 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(
        segments: &[OwnedSegment],
        cursor: &mut SegmentCursor,
    ) -> Option<Self> {
        let saved = cursor.save();
        let qty = if peek_is(segments, cursor, "QTY") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if qty.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self {
            qty,
        })
    }
}

impl Pid55169 {
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
        while let Some(item) = Pid55169Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55169Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55169 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
