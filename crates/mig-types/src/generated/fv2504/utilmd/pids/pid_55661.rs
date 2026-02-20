//! Auto-generated PID 55661 types.
//! Änderung Daten der SR
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55661Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z19: Vec<Pid55661Sg5Z19>,
    pub sg6: Vec<Pid55661Sg6>,
    pub sg8_z62: Vec<Pid55661Sg8Z62>,
    pub sg8_z61: Vec<Pid55661Sg8Z61>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z61
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg8Z61 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55661Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z62
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661Sg8Z62 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55661Sg10>,
}

/// PID 55661: Änderung Daten der SR
/// Kommunikation: MSB an weiteren MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55661 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55661Sg2>,
    pub sg4: Vec<Pid55661Sg4>,
}
