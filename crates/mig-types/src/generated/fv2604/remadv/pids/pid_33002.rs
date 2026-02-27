//! Auto-generated PID 33002 types.
//! Abweisung
//! Do not edit manually.

use serde::{Deserialize, Serialize};
use crate::segment::OwnedSegment;
use crate::cursor::{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment};

/// SG1 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002Sg1 {
    pub nad: Option<OwnedSegment>,
    pub sg3_ic: Vec<Pid33002Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002Sg3Ic {
    pub com: Option<OwnedSegment>,
    pub cta: Option<OwnedSegment>,
}

/// SG4 — Währungsverwendung, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002Sg4 {
    pub cux: Option<OwnedSegment>,
}

/// SG5 — Dokumentenname, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002Sg5 {
    pub doc: Option<OwnedSegment>,
    pub dtm: Option<OwnedSegment>,
    pub moa: Option<OwnedSegment>,
    pub rff: Option<OwnedSegment>,
    pub sg7: Vec<Pid33002Sg7>,
    pub sg7_e_0243_e_0259_e_0261_e_0267_e_0272_e_0275_e_0459_e_0503_e_0505_e_0506_e_0518_e_0522_e_0569_e_0804_e_0806_e_1007_e_1009_e_1010_e_3038_g_0079_g_0080_g_0081_g_0083_g_0084_g_0085_g_0086_gs_002: Vec<Pid33002Sg7E_0243E_0259E_0261E_0267E_0272E_0275E_0459E_0503E_0505E_0506E_0518E_0522E_0569E_0804E_0806E_1007E_1009E_1010E_3038G_0079G_0080G_0081G_0083G_0084G_0085G_0086Gs_002>,
}

/// SG7 — Code des Prüfschritts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002Sg7 {
    pub ajt: Option<OwnedSegment>,
}

/// SG7 — Code des Prüfschritts
/// Qualifiers: E_0243, E_0259, E_0261, E_0267, E_0272, E_0275, E_0459, E_0503, E_0505, E_0506, E_0518, E_0522, E_0569, E_0804, E_0806, E_1007, E_1009, E_1010, E_3038, G_0079, G_0080, G_0081, G_0083, G_0084, G_0085, G_0086, GS_002
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002Sg7E_0243E_0259E_0261E_0267E_0272E_0275E_0459E_0503E_0505E_0506E_0518E_0522E_0569E_0804E_0806E_1007E_1009E_1010E_3038G_0079G_0080G_0081G_0083G_0084G_0085G_0086Gs_002 {
    pub ajt: Option<OwnedSegment>,
    pub ftx: Option<OwnedSegment>,
}

/// PID 33002: Abweisung
/// Kommunikation: ReEmpf an ReErst
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid33002 {
    pub bgm: OwnedSegment,
    pub dtm: OwnedSegment,
    pub moa: OwnedSegment,
    pub rff: OwnedSegment,
    pub unh: OwnedSegment,
    pub uns: OwnedSegment,
    pub unt: OwnedSegment,
    pub sg1: Vec<Pid33002Sg1>,
    pub sg4: Vec<Pid33002Sg4>,
    pub sg5: Vec<Pid33002Sg5>,
}

impl Pid33002Sg1 {
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
        while let Some(item) = Pid33002Sg3Ic::from_segments(segments, cursor) {
            sg3_ic.push(item);
        }
        Some(Self {
            nad,
            sg3_ic,
        })
    }
}

impl Pid33002Sg3Ic {
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

impl Pid33002Sg4 {
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

impl Pid33002Sg5 {
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
        while let Some(item) = Pid33002Sg7::from_segments(segments, cursor) {
            sg7.push(item);
        }
        let mut sg7_e_0243_e_0259_e_0261_e_0267_e_0272_e_0275_e_0459_e_0503_e_0505_e_0506_e_0518_e_0522_e_0569_e_0804_e_0806_e_1007_e_1009_e_1010_e_3038_g_0079_g_0080_g_0081_g_0083_g_0084_g_0085_g_0086_gs_002 = Vec::new();
        while let Some(item) = Pid33002Sg7E_0243E_0259E_0261E_0267E_0272E_0275E_0459E_0503E_0505E_0506E_0518E_0522E_0569E_0804E_0806E_1007E_1009E_1010E_3038G_0079G_0080G_0081G_0083G_0084G_0085G_0086Gs_002::from_segments(segments, cursor) {
            sg7_e_0243_e_0259_e_0261_e_0267_e_0272_e_0275_e_0459_e_0503_e_0505_e_0506_e_0518_e_0522_e_0569_e_0804_e_0806_e_1007_e_1009_e_1010_e_3038_g_0079_g_0080_g_0081_g_0083_g_0084_g_0085_g_0086_gs_002.push(item);
        }
        Some(Self {
            doc,
            dtm,
            moa,
            rff,
            sg7,
            sg7_e_0243_e_0259_e_0261_e_0267_e_0272_e_0275_e_0459_e_0503_e_0505_e_0506_e_0518_e_0522_e_0569_e_0804_e_0806_e_1007_e_1009_e_1010_e_3038_g_0079_g_0080_g_0081_g_0083_g_0084_g_0085_g_0086_gs_002,
        })
    }
}

impl Pid33002Sg7 {
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

impl Pid33002Sg7E_0243E_0259E_0261E_0267E_0272E_0275E_0459E_0503E_0505E_0506E_0518E_0522E_0569E_0804E_0806E_1007E_1009E_1010E_3038G_0079G_0080G_0081G_0083G_0084G_0085G_0086Gs_002 {
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

impl Pid33002 {
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
        while let Some(item) = Pid33002Sg1::from_segments(segments, &mut cursor) {
            sg1.push(item);
        }
        let mut sg4 = Vec::new();
        while let Some(item) = Pid33002Sg4::from_segments(segments, &mut cursor) {
            sg4.push(item);
        }
        let mut sg5 = Vec::new();
        while let Some(item) = Pid33002Sg5::from_segments(segments, &mut cursor) {
            sg5.push(item);
        }

        Ok(Pid33002 {
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
