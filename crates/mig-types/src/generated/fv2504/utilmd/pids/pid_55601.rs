//! Auto-generated PID 55601 types.
//! Anmeldung neue erz. MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg12Z04 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg12Z09 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg12Z60 {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55601Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ftx: Option<super::super::segments::SegFtx>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z09: Vec<Pid55601Sg12Z09>,
    pub sg12_z04: Vec<Pid55601Sg12Z04>,
    pub sg12_z60: Vec<Pid55601Sg12Z60>,
    pub sg5_z16: Vec<Pid55601Sg5Z16>,
    pub sg5_z20: Vec<Pid55601Sg5Z20>,
    pub sg5_z19: Vec<Pid55601Sg5Z19>,
    pub sg5_z17: Vec<Pid55601Sg5Z17>,
    pub sg6: Vec<Pid55601Sg6>,
    pub sg8_z79: Vec<Pid55601Sg8Z79>,
    pub sg8_zh0: Vec<Pid55601Sg8Zh0>,
    pub sg8_z01: Vec<Pid55601Sg8Z01>,
    pub sg8_za4: Vec<Pid55601Sg8Za4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg5Z20 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg8Z01 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55601Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z79
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg8Z79 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55601Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg8Za4 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55601Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZH0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601Sg8Zh0 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55601Sg10>,
}

/// PID 55601: Anmeldung neue erz. MaLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55601 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55601Sg2>,
    pub sg4: Vec<Pid55601Sg4>,
}
