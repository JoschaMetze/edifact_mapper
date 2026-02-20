//! Auto-generated PID 55035 types.
//! Antwort auf GDA verb. MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z63
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg12Z63 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55035Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z63: Vec<Pid55035Sg12Z63>,
    pub sg5_z18: Vec<Pid55035Sg5Z18>,
    pub sg5_z16: Vec<Pid55035Sg5Z16>,
    pub sg5_z22: Vec<Pid55035Sg5Z22>,
    pub sg5_z20: Vec<Pid55035Sg5Z20>,
    pub sg5_z19: Vec<Pid55035Sg5Z19>,
    pub sg5_z17: Vec<Pid55035Sg5Z17>,
    pub sg6: Vec<Pid55035Sg6>,
    pub sg8_zd5: Vec<Pid55035Sg8Zd5>,
    pub sg8_zd6: Vec<Pid55035Sg8Zd6>,
    pub sg8_zd7: Vec<Pid55035Sg8Zd7>,
    pub sg8_ze0: Vec<Pid55035Sg8Ze0>,
    pub sg8_z98: Vec<Pid55035Sg8Z98>,
    pub sg8_ze1: Vec<Pid55035Sg8Ze1>,
    pub sg8_ze3: Vec<Pid55035Sg8Ze3>,
    pub sg8_ze4: Vec<Pid55035Sg8Ze4>,
    pub sg8_ze5: Vec<Pid55035Sg8Ze5>,
    pub sg8_zf0: Vec<Pid55035Sg8Zf0>,
    pub sg8_zf1: Vec<Pid55035Sg8Zf1>,
    pub sg8_zf2: Vec<Pid55035Sg8Zf2>,
    pub sg8_zf3: Vec<Pid55035Sg8Zf3>,
    pub sg8_zf5: Vec<Pid55035Sg8Zf5>,
    pub sg8_zf6: Vec<Pid55035Sg8Zf6>,
    pub sg8_zg0: Vec<Pid55035Sg8Zg0>,
    pub sg8_zg1: Vec<Pid55035Sg8Zg1>,
    pub sg8_zg2: Vec<Pid55035Sg8Zg2>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg5Z20 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z22
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg5Z22 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z98
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Z98 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
    pub sg9: Vec<Pid55035Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zd5 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zd6 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zd7 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Ze0 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Ze1 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
    pub sg9: Vec<Pid55035Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Ze3 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Ze4 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZE5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Ze5 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zf0 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
    pub sg9: Vec<Pid55035Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zf1 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zf2 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zf3 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zf5 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zf6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zg0 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zg1 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg8Zg2 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55035Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55035: Antwort auf GDA verb. MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55035Sg2>,
    pub sg4: Vec<Pid55035Sg4>,
}
