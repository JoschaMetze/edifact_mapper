//! Auto-generated PID 55639 types.
//! Änderung Daten der NeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55639Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z18: Vec<Pid55639Sg5Z18>,
    pub sg6: Vec<Pid55639Sg6>,
    pub sg8_z51: Vec<Pid55639Sg8Z51>,
    pub sg8_z57: Vec<Pid55639Sg8Z57>,
    pub sg8_z60: Vec<Pid55639Sg8Z60>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z51
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg8Z51 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55639Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z57
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg8Z57 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55639Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639Sg8Z60 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55639Sg10>,
}

/// PID 55639: Änderung Daten der NeLo
/// Kommunikation: MSB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55639 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55639Sg2>,
    pub sg4: Vec<Pid55639Sg4>,
}
