//! Auto-generated PID 55109 types.
//! Änderung Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg12Z04 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z09
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg12Z09 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55109Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z09: Vec<Pid55109Sg12Z09>,
    pub sg12_z04: Vec<Pid55109Sg12Z04>,
    pub sg5_z16: Vec<Pid55109Sg5Z16>,
    pub sg6: Vec<Pid55109Sg6>,
    pub sg8_z01: Vec<Pid55109Sg8Z01>,
    pub sg8_z75: Vec<Pid55109Sg8Z75>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg8Z01 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55109Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z75
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109Sg8Z75 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55109Sg10>,
}

/// PID 55109: Änderung Daten der MaLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55109 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55109Sg2>,
    pub sg4: Vec<Pid55109Sg4>,
}
