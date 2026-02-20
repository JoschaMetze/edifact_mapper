//! Auto-generated PID 55653 types.
//! Änderung Daten der MeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z05
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg12Z05 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55653Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z05: Vec<Pid55653Sg12Z05>,
    pub sg5_z17: Vec<Pid55653Sg5Z17>,
    pub sg6: Vec<Pid55653Sg6>,
    pub sg8_z18: Vec<Pid55653Sg8Z18>,
    pub sg8_z03: Vec<Pid55653Sg8Z03>,
    pub sg8_z20: Vec<Pid55653Sg8Z20>,
    pub sg8_z04: Vec<Pid55653Sg8Z04>,
    pub sg8_z13: Vec<Pid55653Sg8Z13>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z03
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg8Z03 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55653Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z04
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg8Z04 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55653Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z13
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg8Z13 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55653Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg8Z18 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55653Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z20
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653Sg8Z20 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55653Sg10>,
}

/// PID 55653: Änderung Daten der MeLo
/// Kommunikation: MSB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55653 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55653Sg2>,
    pub sg4: Vec<Pid55653Sg4>,
}
