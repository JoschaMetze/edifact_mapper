//! Auto-generated PID 55195 types.
//! Bilanzierungs-gebiets-clearing-liste
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG1 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg1 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55195Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sg5_z15: Vec<Pid55195Sg5Z15>,
    pub sg5_z16: Vec<Pid55195Sg5Z16>,
    pub sg5_z21: Vec<Pid55195Sg5Z21>,
    pub sg6: Vec<Pid55195Sg6>,
    pub sg8_z22: Vec<Pid55195Sg8Z22>,
    pub sg8_z01: Vec<Pid55195Sg8Z01>,
    pub sg8_z02: Vec<Pid55195Sg8Z02>,
    pub sg8_z15: Vec<Pid55195Sg8Z15>,
    pub sg8_z17: Vec<Pid55195Sg8Z17>,
    pub sg8_z21: Vec<Pid55195Sg8Z21>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg5Z15 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg5Z21 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg8Z01 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55195Sg10>,
    pub sg9: Vec<Pid55195Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z02
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg8Z02 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg8Z15 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55195Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg8Z17 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg8Z21 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55195Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z22
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg8Z22 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55195: Bilanzierungs-gebiets-clearing-liste
/// Kommunikation: ÜNB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<Pid55195Sg1>,
    pub sg2: Vec<Pid55195Sg2>,
    pub sg4: Vec<Pid55195Sg4>,
}
