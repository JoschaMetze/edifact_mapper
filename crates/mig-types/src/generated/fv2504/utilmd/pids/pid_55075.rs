//! Auto-generated PID 55075 types.
//! Stammdaten aufgrund einer Änderung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: DP
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg12Dp {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: VY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg12Vy {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55075Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_vy: Vec<Pid55075Sg12Vy>,
    pub sg12_dp: Vec<Pid55075Sg12Dp>,
    pub sg5_z16: Vec<Pid55075Sg5Z16>,
    pub sg5_z21: Vec<Pid55075Sg5Z21>,
    pub sg5_z17: Vec<Pid55075Sg5Z17>,
    pub sg6: Vec<Pid55075Sg6>,
    pub sg8_z01: Vec<Pid55075Sg8Z01>,
    pub sg8_z02: Vec<Pid55075Sg8Z02>,
    pub sg8_z15: Vec<Pid55075Sg8Z15>,
    pub sg8_z17: Vec<Pid55075Sg8Z17>,
    pub sg8_z03: Vec<Pid55075Sg8Z03>,
    pub sg8_z20: Vec<Pid55075Sg8Z20>,
    pub sg8_z04: Vec<Pid55075Sg8Z04>,
    pub sg8_z13: Vec<Pid55075Sg8Z13>,
    pub sg8_z14: Vec<Pid55075Sg8Z14>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg5Z21 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z01 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z02
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z02 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z03 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z04 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z13
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z13 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z14
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z14 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z15 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
    pub sg9: Vec<Pid55075Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z17 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg8Z20 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55075Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55075: Stammdaten aufgrund einer Änderung
/// Kommunikation: NB an UBA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55075Sg2>,
    pub sg4: Vec<Pid55075Sg4>,
}
