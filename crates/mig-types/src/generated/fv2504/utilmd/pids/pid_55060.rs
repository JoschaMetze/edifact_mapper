//! Auto-generated PID 55060 types.
//! Antwort auf GDA
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z64
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg12Z64 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55060Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z64: Vec<Pid55060Sg12Z64>,
    pub sg5_z18: Vec<Pid55060Sg5Z18>,
    pub sg5_z16: Vec<Pid55060Sg5Z16>,
    pub sg5_z22: Vec<Pid55060Sg5Z22>,
    pub sg5_z20: Vec<Pid55060Sg5Z20>,
    pub sg5_z19: Vec<Pid55060Sg5Z19>,
    pub sg5_z21: Vec<Pid55060Sg5Z21>,
    pub sg5_z17: Vec<Pid55060Sg5Z17>,
    pub sg6: Vec<Pid55060Sg6>,
    pub sg8_zd5: Vec<Pid55060Sg8Zd5>,
    pub sg8_zd6: Vec<Pid55060Sg8Zd6>,
    pub sg8_zd7: Vec<Pid55060Sg8Zd7>,
    pub sg8_zd9: Vec<Pid55060Sg8Zd9>,
    pub sg8_ze0: Vec<Pid55060Sg8Ze0>,
    pub sg8_z98: Vec<Pid55060Sg8Z98>,
    pub sg8_ze2: Vec<Pid55060Sg8Ze2>,
    pub sg8_ze3: Vec<Pid55060Sg8Ze3>,
    pub sg8_ze4: Vec<Pid55060Sg8Ze4>,
    pub sg8_ze7: Vec<Pid55060Sg8Ze7>,
    pub sg8_ze8: Vec<Pid55060Sg8Ze8>,
    pub sg8_zf0: Vec<Pid55060Sg8Zf0>,
    pub sg8_zf1: Vec<Pid55060Sg8Zf1>,
    pub sg8_zf2: Vec<Pid55060Sg8Zf2>,
    pub sg8_zf3: Vec<Pid55060Sg8Zf3>,
    pub sg8_zf4: Vec<Pid55060Sg8Zf4>,
    pub sg8_zf5: Vec<Pid55060Sg8Zf5>,
    pub sg8_zf6: Vec<Pid55060Sg8Zf6>,
    pub sg8_zf7: Vec<Pid55060Sg8Zf7>,
    pub sg8_zf8: Vec<Pid55060Sg8Zf8>,
    pub sg8_zf9: Vec<Pid55060Sg8Zf9>,
    pub sg8_zg0: Vec<Pid55060Sg8Zg0>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z20 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z21 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z22
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg5Z22 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z98
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Z98 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
    pub sg9: Vec<Pid55060Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zd5 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zd6 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zd7 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD9
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zd9 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Ze0 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Ze2 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Ze3 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Ze4 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Ze7 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
    pub sg9: Vec<Pid55060Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Ze8 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf0 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
    pub sg9: Vec<Pid55060Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf1 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf2 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf3 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf4 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf5 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf7 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf8 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF9
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zf9 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg8Zg0 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55060Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55060: Antwort auf GDA
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55060 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55060Sg2>,
    pub sg4: Vec<Pid55060Sg4>,
}
