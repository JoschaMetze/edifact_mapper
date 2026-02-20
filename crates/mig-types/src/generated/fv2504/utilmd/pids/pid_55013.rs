//! Auto-generated PID 55013 types.
//! Anmeldung / Zuordnung EOG
//! Do not edit manually.

use crate::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};
use crate::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z63
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z63 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z65
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z65 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z66
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z66 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z67
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z67 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z68
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z68 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z69
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z69 {
    pub nad: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z70
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z70 {
    pub nad: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55013Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg4 {
    pub dtm: Option<OwnedSegment>,
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z65: Vec<Pid55013Sg12Z65>,
    pub sg12_z66: Vec<Pid55013Sg12Z66>,
    pub sg12_z67: Vec<Pid55013Sg12Z67>,
    pub sg12_z68: Vec<Pid55013Sg12Z68>,
    pub sg12_z69: Vec<Pid55013Sg12Z69>,
    pub sg12_z70: Vec<Pid55013Sg12Z70>,
    pub sg12_z63: Vec<Pid55013Sg12Z63>,
    pub sg5_z18: Vec<Pid55013Sg5Z18>,
    pub sg5_z16: Vec<Pid55013Sg5Z16>,
    pub sg5_z20: Vec<Pid55013Sg5Z20>,
    pub sg5_z19: Vec<Pid55013Sg5Z19>,
    pub sg5_z17: Vec<Pid55013Sg5Z17>,
    pub sg6: Vec<Pid55013Sg6>,
    pub sg8_zd7: Vec<Pid55013Sg8Zd7>,
    pub sg8_z98: Vec<Pid55013Sg8Z98>,
    pub sg8_zf1: Vec<Pid55013Sg8Zf1>,
    pub sg8_zf3: Vec<Pid55013Sg8Zf3>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z16 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z17 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z18 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z19 {
    pub loc: Option<OwnedSegment>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z20 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg6 {
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z98
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Z98 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55013Sg10>,
    pub sg9: Vec<Pid55013Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Zd7 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55013Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Zf1 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55013Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Zf3 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55013Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg9 {
    pub qty: Option<OwnedSegment>,
}

/// PID 55013: Anmeldung / Zuordnung EOG
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55013Sg2>,
    pub sg4: Vec<Pid55013Sg4>,
}

impl Pid55013Sg10 {
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

impl Pid55013Sg12Z63 {
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

impl Pid55013Sg12Z65 {
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

impl Pid55013Sg12Z66 {
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

impl Pid55013Sg12Z67 {
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

impl Pid55013Sg12Z68 {
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

impl Pid55013Sg12Z69 {
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

impl Pid55013Sg12Z70 {
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

impl Pid55013Sg2 {
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
        while let Some(item) = Pid55013Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self { nad, sg3_ic })
    }
}

impl Pid55013Sg3Ic {
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

impl Pid55013Sg4 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        let mut sg12_z65 = Vec::new();
        while let Some(item) = Pid55013Sg12Z65::from_segments(segments, cursor) {
            sg12_z65.push(item);
        }
        let mut sg12_z66 = Vec::new();
        while let Some(item) = Pid55013Sg12Z66::from_segments(segments, cursor) {
            sg12_z66.push(item);
        }
        let mut sg12_z67 = Vec::new();
        while let Some(item) = Pid55013Sg12Z67::from_segments(segments, cursor) {
            sg12_z67.push(item);
        }
        let mut sg12_z68 = Vec::new();
        while let Some(item) = Pid55013Sg12Z68::from_segments(segments, cursor) {
            sg12_z68.push(item);
        }
        let mut sg12_z69 = Vec::new();
        while let Some(item) = Pid55013Sg12Z69::from_segments(segments, cursor) {
            sg12_z69.push(item);
        }
        let mut sg12_z70 = Vec::new();
        while let Some(item) = Pid55013Sg12Z70::from_segments(segments, cursor) {
            sg12_z70.push(item);
        }
        let mut sg12_z63 = Vec::new();
        while let Some(item) = Pid55013Sg12Z63::from_segments(segments, cursor) {
            sg12_z63.push(item);
        }
        let mut sg5_z18 = Vec::new();
        while let Some(item) = Pid55013Sg5Z18::from_segments(segments, cursor) {
            sg5_z18.push(item);
        }
        let mut sg5_z16 = Vec::new();
        while let Some(item) = Pid55013Sg5Z16::from_segments(segments, cursor) {
            sg5_z16.push(item);
        }
        let mut sg5_z20 = Vec::new();
        while let Some(item) = Pid55013Sg5Z20::from_segments(segments, cursor) {
            sg5_z20.push(item);
        }
        let mut sg5_z19 = Vec::new();
        while let Some(item) = Pid55013Sg5Z19::from_segments(segments, cursor) {
            sg5_z19.push(item);
        }
        let mut sg5_z17 = Vec::new();
        while let Some(item) = Pid55013Sg5Z17::from_segments(segments, cursor) {
            sg5_z17.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55013Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_zd7 = Vec::new();
        while let Some(item) = Pid55013Sg8Zd7::from_segments(segments, cursor) {
            sg8_zd7.push(item);
        }
        let mut sg8_z98 = Vec::new();
        while let Some(item) = Pid55013Sg8Z98::from_segments(segments, cursor) {
            sg8_z98.push(item);
        }
        let mut sg8_zf1 = Vec::new();
        while let Some(item) = Pid55013Sg8Zf1::from_segments(segments, cursor) {
            sg8_zf1.push(item);
        }
        let mut sg8_zf3 = Vec::new();
        while let Some(item) = Pid55013Sg8Zf3::from_segments(segments, cursor) {
            sg8_zf3.push(item);
        }
        Some(Self {
            dtm,
            ide,
            sts,
            sg12_z65,
            sg12_z66,
            sg12_z67,
            sg12_z68,
            sg12_z69,
            sg12_z70,
            sg12_z63,
            sg5_z18,
            sg5_z16,
            sg5_z20,
            sg5_z19,
            sg5_z17,
            sg6,
            sg8_zd7,
            sg8_z98,
            sg8_zf1,
            sg8_zf3,
        })
    }
}

impl Pid55013Sg5Z16 {
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

impl Pid55013Sg5Z17 {
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

impl Pid55013Sg5Z18 {
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

impl Pid55013Sg5Z19 {
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

impl Pid55013Sg5Z20 {
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

impl Pid55013Sg6 {
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

impl Pid55013Sg8Z98 {
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
        while let Some(item) = Pid55013Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        let mut sg9 = Vec::new();
        while let Some(item) = Pid55013Sg9::from_segments(segments, cursor) {
            sg9.push(item);
        }
        Some(Self { seq, sg10, sg9 })
    }
}

impl Pid55013Sg8Zd7 {
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
        while let Some(item) = Pid55013Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55013Sg8Zf1 {
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
        while let Some(item) = Pid55013Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55013Sg8Zf3 {
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
        while let Some(item) = Pid55013Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55013Sg9 {
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

impl Pid55013 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid55013Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55013Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55013 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
