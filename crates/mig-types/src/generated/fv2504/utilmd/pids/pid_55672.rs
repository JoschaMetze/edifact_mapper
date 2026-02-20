//! Auto-generated PID 55672 types.
//! Abr.-Daten BK-Abr. erz. Malo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55672Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z16: Vec<Pid55672Sg5Z16>,
    pub sg5_z21: Vec<Pid55672Sg5Z21>,
    pub sg6: Vec<Pid55672Sg6>,
    pub sg8_z01: Vec<Pid55672Sg8Z01>,
    pub sg8_z15: Vec<Pid55672Sg8Z15>,
    pub sg8_z21: Vec<Pid55672Sg8Z21>,
    pub sg8_z08: Vec<Pid55672Sg8Z08>,
    pub sg8_z38: Vec<Pid55672Sg8Z38>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg5Z21 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg8Z01 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55672Sg10>,
    pub sg9: Vec<Pid55672Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg8Z08 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55672Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg8Z15 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55672Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg8Z21 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55672Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z38
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg8Z38 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55672Sg10>,
}

/// SG9 — Arbeit / Leistung für tagesparameterabhängige Marktlokation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55672: Abr.-Daten BK-Abr. erz. Malo
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55672 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55672Sg2>,
    pub sg4: Vec<Pid55672Sg4>,
}
