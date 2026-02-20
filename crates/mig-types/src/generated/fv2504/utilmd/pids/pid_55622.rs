//! Auto-generated PID 55622 types.
//! Rückmeldung/Anfrage Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z51, Z52
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z51Z52 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z53, Z54
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z53Z54 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z55, Z56
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z55Z56 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z57, Z58
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z57Z58 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z59, Z60
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg12Z59Z60 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55622Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z51_z52: Vec<Pid55622Sg12Z51Z52>,
    pub sg12_z53_z54: Vec<Pid55622Sg12Z53Z54>,
    pub sg12_z55_z56: Vec<Pid55622Sg12Z55Z56>,
    pub sg12_z57_z58: Vec<Pid55622Sg12Z57Z58>,
    pub sg12_z59_z60: Vec<Pid55622Sg12Z59Z60>,
    pub sg5_z16: Vec<Pid55622Sg5Z16>,
    pub sg6: Vec<Pid55622Sg6>,
    pub sg8_z80_z81: Vec<Pid55622Sg8Z80Z81>,
    pub sg8_zd1_zd2: Vec<Pid55622Sg8Zd1Zd2>,
    pub sg8_zd3_zd4: Vec<Pid55622Sg8Zd3Zd4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z80, Z81
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg8Z80Z81 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55622Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD1, ZD2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg8Zd1Zd2 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55622Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZD3, ZD4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622Sg8Zd3Zd4 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
}

/// PID 55622: Rückmeldung/Anfrage Daten der MaLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55622 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55622Sg2>,
    pub sg4: Vec<Pid55622Sg4>,
}
