//! Auto-generated PID 55095 types.
//! Antwort auf GDA erz. MaLo
//! Do not edit manually.

use crate::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};
use crate::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z63
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg12Z63 {
    pub nad: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55095Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg4 {
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z63: Vec<Pid55095Sg12Z63>,
    pub sg5_z18: Vec<Pid55095Sg5Z18>,
    pub sg5_z16: Vec<Pid55095Sg5Z16>,
    pub sg5_z20: Vec<Pid55095Sg5Z20>,
    pub sg5_z19: Vec<Pid55095Sg5Z19>,
    pub sg5_z21: Vec<Pid55095Sg5Z21>,
    pub sg5_z17: Vec<Pid55095Sg5Z17>,
    pub sg6: Vec<Pid55095Sg6>,
    pub sg8_zd5: Vec<Pid55095Sg8Zd5>,
    pub sg8_zd6: Vec<Pid55095Sg8Zd6>,
    pub sg8_zd7: Vec<Pid55095Sg8Zd7>,
    pub sg8_ze0: Vec<Pid55095Sg8Ze0>,
    pub sg8_z98: Vec<Pid55095Sg8Z98>,
    pub sg8_ze3: Vec<Pid55095Sg8Ze3>,
    pub sg8_ze4: Vec<Pid55095Sg8Ze4>,
    pub sg8_ze6: Vec<Pid55095Sg8Ze6>,
    pub sg8_ze7: Vec<Pid55095Sg8Ze7>,
    pub sg8_ze9: Vec<Pid55095Sg8Ze9>,
    pub sg8_zf0: Vec<Pid55095Sg8Zf0>,
    pub sg8_zf1: Vec<Pid55095Sg8Zf1>,
    pub sg8_zf2: Vec<Pid55095Sg8Zf2>,
    pub sg8_zf3: Vec<Pid55095Sg8Zf3>,
    pub sg8_zf5: Vec<Pid55095Sg8Zf5>,
    pub sg8_zf6: Vec<Pid55095Sg8Zf6>,
    pub sg8_zg0: Vec<Pid55095Sg8Zg0>,
    pub sg8_zg1: Vec<Pid55095Sg8Zg1>,
    pub sg8_zg2: Vec<Pid55095Sg8Zg2>,
    pub sg8_zg3: Vec<Pid55095Sg8Zg3>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg5Z17 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg5Z18 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg5Z19 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg5Z20 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg5Z21 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z98
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Z98 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
    pub sg9: Vec<Pid55095Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zd5 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zd6 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zd7 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Ze0 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Ze3 {
    pub pia: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Ze4 {
    pub pia: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Ze6 {
    pub pia: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Ze7 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE9
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Ze9 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zf0 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
    pub sg9: Vec<Pid55095Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zf1 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zf2 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zf3 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zf5 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zf6 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zg0 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zg1 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zg2 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg8Zg3 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55095Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095Sg9 {
    pub qty: Option<OwnedSegment>,
}

/// PID 55095: Antwort auf GDA erz. MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55095 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55095Sg2>,
    pub sg4: Vec<Pid55095Sg4>,
}

impl Pid55095Sg10 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { cav, cci })
    }
}

impl Pid55095Sg12Z63 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad })
    }
}

impl Pid55095Sg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self { nad, sg3_ic })
    }
}

impl Pid55095Sg3Ic {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { com, cta })
    }
}

impl Pid55095Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
        let saved = cursor.save();
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
        if ide.is_none() && sts.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg12_z63 = Vec::new();
        while let Some(item) = Pid55095Sg12Z63::from_segments(segments, cursor) {
            sg12_z63.push(item);
        }
        let mut sg5_z18 = Vec::new();
        while let Some(item) = Pid55095Sg5Z18::from_segments(segments, cursor) {
            sg5_z18.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55095Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg5_z20 = Vec::new();
        while let Some(item) = Pid55095Sg5Z20::from_segments(segments, cursor) {
            sg5_z20.push(item);
        }
        let mut sg5_z19 = Vec::new();
        while let Some(item) = Pid55095Sg5Z19::from_segments(segments, cursor) {
            sg5_z19.push(item);
        }
        let mut sg5_z21 = Vec::new();
        while let Some(item) = Pid55095Sg5Z21::from_segments(segments, cursor) {
            sg5_z21.push(item);
        }
        let mut sg5_z17 = Vec::new();
        while let Some(item) = Pid55095Sg5Z17::from_segments(segments, cursor) {
            sg5_z17.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55095Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_zd5 = Vec::new();
        while let Some(item) = Pid55095Sg8Zd5::from_segments(segments, cursor) {
            sg8_zd5.push(item);
        }
        let mut sg8_zd6 = Vec::new();
        while let Some(item) = Pid55095Sg8Zd6::from_segments(segments, cursor) {
            sg8_zd6.push(item);
        }
        let mut sg8_zd7 = Vec::new();
        while let Some(item) = Pid55095Sg8Zd7::from_segments(segments, cursor) {
            sg8_zd7.push(item);
        }
        let mut sg8_ze0 = Vec::new();
        while let Some(item) = Pid55095Sg8Ze0::from_segments(segments, cursor) {
            sg8_ze0.push(item);
        }
        let mut sg8_z98 = Vec::new();
        while let Some(item) = Pid55095Sg8Z98::from_segments(segments, cursor) {
            sg8_z98.push(item);
        }
        let mut sg8_ze3 = Vec::new();
        while let Some(item) = Pid55095Sg8Ze3::from_segments(segments, cursor) {
            sg8_ze3.push(item);
        }
        let mut sg8_ze4 = Vec::new();
        while let Some(item) = Pid55095Sg8Ze4::from_segments(segments, cursor) {
            sg8_ze4.push(item);
        }
        let mut sg8_ze6 = Vec::new();
        while let Some(item) = Pid55095Sg8Ze6::from_segments(segments, cursor) {
            sg8_ze6.push(item);
        }
        let mut sg8_ze7 = Vec::new();
        while let Some(item) = Pid55095Sg8Ze7::from_segments(segments, cursor) {
            sg8_ze7.push(item);
        }
        let mut sg8_ze9 = Vec::new();
        while let Some(item) = Pid55095Sg8Ze9::from_segments(segments, cursor) {
            sg8_ze9.push(item);
        }
        let mut sg8_zf0 = Vec::new();
        while let Some(item) = Pid55095Sg8Zf0::from_segments(segments, cursor) {
            sg8_zf0.push(item);
        }
        let mut sg8_zf1 = Vec::new();
        while let Some(item) = Pid55095Sg8Zf1::from_segments(segments, cursor) {
            sg8_zf1.push(item);
        }
        let mut sg8_zf2 = Vec::new();
        while let Some(item) = Pid55095Sg8Zf2::from_segments(segments, cursor) {
            sg8_zf2.push(item);
        }
        let mut sg8_zf3 = Vec::new();
        while let Some(item) = Pid55095Sg8Zf3::from_segments(segments, cursor) {
            sg8_zf3.push(item);
        }
        let mut sg8_zf5 = Vec::new();
        while let Some(item) = Pid55095Sg8Zf5::from_segments(segments, cursor) {
            sg8_zf5.push(item);
        }
        let mut sg8_zf6 = Vec::new();
        while let Some(item) = Pid55095Sg8Zf6::from_segments(segments, cursor) {
            sg8_zf6.push(item);
        }
        let mut sg8_zg0 = Vec::new();
        while let Some(item) = Pid55095Sg8Zg0::from_segments(segments, cursor) {
            sg8_zg0.push(item);
        }
        let mut sg8_zg1 = Vec::new();
        while let Some(item) = Pid55095Sg8Zg1::from_segments(segments, cursor) {
            sg8_zg1.push(item);
        }
        let mut sg8_zg2 = Vec::new();
        while let Some(item) = Pid55095Sg8Zg2::from_segments(segments, cursor) {
            sg8_zg2.push(item);
        }
        let mut sg8_zg3 = Vec::new();
        while let Some(item) = Pid55095Sg8Zg3::from_segments(segments, cursor) {
            sg8_zg3.push(item);
        }
        Some(Self {
            ide,
            sts,
            sg12_z63,
            sg5_z18,
            sg5_z16,
            sg5_z20,
            sg5_z19,
            sg5_z21,
            sg5_z17,
            sg6,
            sg8_zd5,
            sg8_zd6,
            sg8_zd7,
            sg8_ze0,
            sg8_z98,
            sg8_ze3,
            sg8_ze4,
            sg8_ze6,
            sg8_ze7,
            sg8_ze9,
            sg8_zf0,
            sg8_zf1,
            sg8_zf2,
            sg8_zf3,
            sg8_zf5,
            sg8_zf6,
            sg8_zg0,
            sg8_zg1,
            sg8_zg2,
            sg8_zg3,
        })
    }
}

impl Pid55095Sg5Z16 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55095Sg5Z17 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55095Sg5Z18 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55095Sg5Z19 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55095Sg5Z20 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55095Sg5Z21 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { loc })
    }
}

impl Pid55095Sg6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { rff })
    }
}

impl Pid55095Sg8Z98 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55095Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self { seq, sg10, sg9 })
    }
}

impl Pid55095Sg8Zd5 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { rff, seq })
    }
}

impl Pid55095Sg8Zd6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { rff, seq })
    }
}

impl Pid55095Sg8Zd7 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55095Sg8Ze0 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
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

impl Pid55095Sg8Ze3 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { pia, seq, sg10 })
    }
}

impl Pid55095Sg8Ze4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        let mut sg10 = Vec::new();
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { pia, seq, sg10 })
    }
}

impl Pid55095Sg8Ze6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
        let saved = cursor.save();
        let pia = if peek_is(segments, cursor, "PIA") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        let seq = if peek_is(segments, cursor, "SEQ") {
            Some(consume(segments, cursor)?.clone())
        } else {
            None
        };
        if pia.is_none() && seq.is_none() {
            cursor.restore(saved);
            return None;
        }
        Some(Self { pia, seq })
    }
}

impl Pid55095Sg8Ze7 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { rff, seq })
    }
}

impl Pid55095Sg8Ze9 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { pia, rff, seq })
    }
}

impl Pid55095Sg8Zf0 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55095Sg9::from_segments(segments, cursor) {
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

impl Pid55095Sg8Zf1 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55095Sg8Zf2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
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

impl Pid55095Sg8Zf3 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55095Sg8Zf5 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55095Sg8Zf6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
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

impl Pid55095Sg8Zg0 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55095Sg8Zg1 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55095Sg8Zg2 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55095Sg8Zg3 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        while let Some(item) = Pid55095Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55095Sg9 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { qty })
    }
}

impl Pid55095 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid55095Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55095Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55095 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
