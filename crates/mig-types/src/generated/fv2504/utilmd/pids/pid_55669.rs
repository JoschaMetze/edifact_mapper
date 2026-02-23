//! Auto-generated PID 55669 types.
//! Rückmeldung/Anfrage Daten der MeLo
//! Do not edit manually.

use crate::cursor::{consume, expect_segment, peek_is, SegmentCursor, SegmentNotFound};
use crate::segment::OwnedSegment;
use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg10 {
    pub cav: Option<OwnedSegment>,
    pub cci: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z39, Z40
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z39Z40 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z41, Z42
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z41Z42 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z43, Z44
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z43Z44 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z45, Z46
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z45Z46 {
    pub nad: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg2 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid55669Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg4 {
    pub ide: Option<OwnedSegment>,
    pub sts: Option<OwnedSegment>,
    pub sg12_z39_z40: Vec<Pid55669Sg12Z39Z40>,
    pub sg12_z41_z42: Vec<Pid55669Sg12Z41Z42>,
    pub sg12_z43_z44: Vec<Pid55669Sg12Z43Z44>,
    pub sg12_z45_z46: Vec<Pid55669Sg12Z45Z46>,
    pub sg5_z17: Vec<Pid55669Sg5Z17>,
    pub sg6: Vec<Pid55669Sg6>,
    pub sg8_zg6_zg7: Vec<Pid55669Sg8Zg6Zg7>,
    pub sg8_za3_za4: Vec<Pid55669Sg8Za3Za4>,
    pub sg8_za5_za6: Vec<Pid55669Sg8Za5Za6>,
    pub sg8_zb9_zc0: Vec<Pid55669Sg8Zb9Zc0>,
    pub sg8_zb7_zb8: Vec<Pid55669Sg8Zb7Zb8>,
    pub sg8_zc1_zc2: Vec<Pid55669Sg8Zc1Zc2>,
    pub sg8_zc3_zc4: Vec<Pid55669Sg8Zc3Zc4>,
    pub sg8_zh3_zh4: Vec<Pid55669Sg8Zh3Zh4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg5Z17 {
    pub loc: Option<OwnedSegment>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg6 {
    pub dtm: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA3, ZA4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Za3Za4 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA5, ZA6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Za5Za6 {
    pub pia: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB7, ZB8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zb7Zb8 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB9, ZC0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zb9Zc0 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZC1, ZC2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zc1Zc2 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZC3, ZC4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zc3Zc4 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG6, ZG7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zg6Zg7 {
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZH3, ZH4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zh3Zh4 {
    pub rff: Option<OwnedSegment>,
    pub seq: Option<OwnedSegment>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// PID 55669: Rückmeldung/Anfrage Daten der MeLo
/// Kommunikation: weiterer MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub unh: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg2: Vec<Pid55669Sg2>,
    pub sg4: Vec<Pid55669Sg4>,
}

impl Pid55669Sg10 {
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

impl Pid55669Sg12Z39Z40 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55669Sg12Z41Z42 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55669Sg12Z43Z44 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55669Sg12Z45Z46 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { nad, rff })
    }
}

impl Pid55669Sg2 {
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
        while let Some(item) = Pid55669Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self { nad, sg3_ic })
    }
}

impl Pid55669Sg3Ic {
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

impl Pid55669Sg4 {
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
        let mut sg12_z39_z40 = Vec::new();
        while let Some(item) = Pid55669Sg12Z39Z40::from_segments(segments, cursor) {
            sg12_z39_z40.push(item);
        }
        let mut sg12_z41_z42 = Vec::new();
        while let Some(item) = Pid55669Sg12Z41Z42::from_segments(segments, cursor) {
            sg12_z41_z42.push(item);
        }
        let mut sg12_z43_z44 = Vec::new();
        while let Some(item) = Pid55669Sg12Z43Z44::from_segments(segments, cursor) {
            sg12_z43_z44.push(item);
        }
        let mut sg12_z45_z46 = Vec::new();
        while let Some(item) = Pid55669Sg12Z45Z46::from_segments(segments, cursor) {
            sg12_z45_z46.push(item);
        }
        let mut sg5_z17 = Vec::new();
        while let Some(item) = Pid55669Sg5Z17::from_segments(segments, cursor) {
            sg5_z17.push(item);
        }
        let mut sg6 = Vec::new();
        while let Some(item) = Pid55669Sg6::from_segments(segments, cursor) {
            sg6.push(item);
        }
        let mut sg8_zg6_zg7 = Vec::new();
        while let Some(item) = Pid55669Sg8Zg6Zg7::from_segments(segments, cursor) {
            sg8_zg6_zg7.push(item);
        }
        let mut sg8_za3_za4 = Vec::new();
        while let Some(item) = Pid55669Sg8Za3Za4::from_segments(segments, cursor) {
            sg8_za3_za4.push(item);
        }
        let mut sg8_za5_za6 = Vec::new();
        while let Some(item) = Pid55669Sg8Za5Za6::from_segments(segments, cursor) {
            sg8_za5_za6.push(item);
        }
        let mut sg8_zb9_zc0 = Vec::new();
        while let Some(item) = Pid55669Sg8Zb9Zc0::from_segments(segments, cursor) {
            sg8_zb9_zc0.push(item);
        }
        let mut sg8_zb7_zb8 = Vec::new();
        while let Some(item) = Pid55669Sg8Zb7Zb8::from_segments(segments, cursor) {
            sg8_zb7_zb8.push(item);
        }
        let mut sg8_zc1_zc2 = Vec::new();
        while let Some(item) = Pid55669Sg8Zc1Zc2::from_segments(segments, cursor) {
            sg8_zc1_zc2.push(item);
        }
        let mut sg8_zc3_zc4 = Vec::new();
        while let Some(item) = Pid55669Sg8Zc3Zc4::from_segments(segments, cursor) {
            sg8_zc3_zc4.push(item);
        }
        let mut sg8_zh3_zh4 = Vec::new();
        while let Some(item) = Pid55669Sg8Zh3Zh4::from_segments(segments, cursor) {
            sg8_zh3_zh4.push(item);
        }
        Some(Self {
            ide,
            sts,
            sg12_z39_z40,
            sg12_z41_z42,
            sg12_z43_z44,
            sg12_z45_z46,
            sg5_z17,
            sg6,
            sg8_zg6_zg7,
            sg8_za3_za4,
            sg8_za5_za6,
            sg8_zb9_zc0,
            sg8_zb7_zb8,
            sg8_zc1_zc2,
            sg8_zc3_zc4,
            sg8_zh3_zh4,
        })
    }
}

impl Pid55669Sg5Z17 {
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

impl Pid55669Sg6 {
    /// Try to assemble this group from segments at the cursor position.
    pub fn from_segments(segments: &[OwnedSegment], cursor: &mut SegmentCursor) -> Option<Self> {
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
        Some(Self { dtm, rff })
    }
}

impl Pid55669Sg8Za3Za4 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55669Sg8Za5Za6 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
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

impl Pid55669Sg8Zb7Zb8 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55669Sg8Zb9Zc0 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55669Sg8Zc1Zc2 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55669Sg8Zc3Zc4 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55669Sg8Zg6Zg7 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { seq, sg10 })
    }
}

impl Pid55669Sg8Zh3Zh4 {
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
        while let Some(item) = Pid55669Sg10::from_segments(segments, cursor) {
            sg10.push(item);
        }
        Some(Self { rff, seq, sg10 })
    }
}

impl Pid55669 {
    /// Assemble this PID from a pre-tokenized segment list.
    pub fn from_segments(segments: &[OwnedSegment]) -> Result<Self, SegmentNotFound> {
        let mut cursor = SegmentCursor::new(segments.len());

        let bgm = expect_segment(segments, &mut cursor, "BGM")?.clone();
        let dtm = expect_segment(segments, &mut cursor, "DTM")?.clone();
        let unh = expect_segment(segments, &mut cursor, "UNH")?.clone();
        let unt = expect_segment(segments, &mut cursor, "UNT")?.clone();
        let mut sg2 = Vec::new();
        while let Some(item) = Pid55669Sg2::from_segments(segments, &mut cursor) {
            sg2.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid55669Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }

        Ok(Pid55669 {
            bgm,
            dtm,
            unh,
            unt,
            sg2,
            sg4,
        })
    }
}
