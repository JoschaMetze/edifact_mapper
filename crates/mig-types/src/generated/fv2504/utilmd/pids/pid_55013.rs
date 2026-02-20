//! Auto-generated PID 55013 types.
//! Anmeldung / Zuordnung EOG
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z63
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z63 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z65
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z65 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z66
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z66 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z67
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z67 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z68
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z68 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z69
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z69 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z70
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg12Z70 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55013Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
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
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg5Z20 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z98
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Z98 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55013Sg10>,
    pub sg9: Vec<Pid55013Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Zd7 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55013Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Zf1 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55013Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZF3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg8Zf3 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55013Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55013: Anmeldung / Zuordnung EOG
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55013Sg2>,
    pub sg4: Vec<Pid55013Sg4>,
}
