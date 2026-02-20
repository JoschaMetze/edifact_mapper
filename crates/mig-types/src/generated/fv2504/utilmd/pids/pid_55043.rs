//! Auto-generated PID 55043 types.
//! Bestätigung Anmeldung MSB
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg12Z03 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg12Z04 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z05
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg12Z05 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z07
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg12Z07 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg12Z08 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg12Z09 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55043Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z09: Vec<Pid55043Sg12Z09>,
    pub sg12_z04: Vec<Pid55043Sg12Z04>,
    pub sg12_z07: Vec<Pid55043Sg12Z07>,
    pub sg12_z08: Vec<Pid55043Sg12Z08>,
    pub sg12_z03: Vec<Pid55043Sg12Z03>,
    pub sg12_z05: Vec<Pid55043Sg12Z05>,
    pub sg5_z18: Vec<Pid55043Sg5Z18>,
    pub sg5_z16: Vec<Pid55043Sg5Z16>,
    pub sg5_z22: Vec<Pid55043Sg5Z22>,
    pub sg5_z20: Vec<Pid55043Sg5Z20>,
    pub sg5_z19: Vec<Pid55043Sg5Z19>,
    pub sg5_z21: Vec<Pid55043Sg5Z21>,
    pub sg5_z17: Vec<Pid55043Sg5Z17>,
    pub sg6: Vec<Pid55043Sg6>,
    pub sg8_z78: Vec<Pid55043Sg8Z78>,
    pub sg8_z58: Vec<Pid55043Sg8Z58>,
    pub sg8_z51: Vec<Pid55043Sg8Z51>,
    pub sg8_z57: Vec<Pid55043Sg8Z57>,
    pub sg8_z60: Vec<Pid55043Sg8Z60>,
    pub sg8_z01: Vec<Pid55043Sg8Z01>,
    pub sg8_z27: Vec<Pid55043Sg8Z27>,
    pub sg8_z02: Vec<Pid55043Sg8Z02>,
    pub sg8_z59: Vec<Pid55043Sg8Z59>,
    pub sg8_z15: Vec<Pid55043Sg8Z15>,
    pub sg8_z16: Vec<Pid55043Sg8Z16>,
    pub sg8_z52: Vec<Pid55043Sg8Z52>,
    pub sg8_z62: Vec<Pid55043Sg8Z62>,
    pub sg8_z61: Vec<Pid55043Sg8Z61>,
    pub sg8_z18: Vec<Pid55043Sg8Z18>,
    pub sg8_z19: Vec<Pid55043Sg8Z19>,
    pub sg8_z03: Vec<Pid55043Sg8Z03>,
    pub sg8_z20: Vec<Pid55043Sg8Z20>,
    pub sg8_z04: Vec<Pid55043Sg8Z04>,
    pub sg8_z05: Vec<Pid55043Sg8Z05>,
    pub sg8_z06: Vec<Pid55043Sg8Z06>,
    pub sg8_z13: Vec<Pid55043Sg8Z13>,
    pub sg8_z14: Vec<Pid55043Sg8Z14>,
    pub sg8_z21: Vec<Pid55043Sg8Z21>,
    pub sg8_z08: Vec<Pid55043Sg8Z08>,
    pub sg8_z38: Vec<Pid55043Sg8Z38>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z20 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z21 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z22
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg5Z22 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z01 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
    pub sg9: Vec<Pid55043Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z02
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z02 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z03 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z04 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z05
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z05 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z06
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z06 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z08 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z13
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z13 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z14
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z14 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z15 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
    pub sg9: Vec<Pid55043Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z16 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z18 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z19 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z20 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z21 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z27
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z27 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z38
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z38 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z51
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z51 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z52
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z52 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
    pub sg9: Vec<Pid55043Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z57
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z57 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z58
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z58 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z59
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z59 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z60 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z61
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z61 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z62
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z62 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55043Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z78
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg8Z78 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55043: Bestätigung Anmeldung MSB
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55043 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55043Sg2>,
    pub sg4: Vec<Pid55043Sg4>,
}
