//! Auto-generated PID 55041 types.
//! Ablehnung Kündigung MSB
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55041Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55041Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55041Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55041Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z17: Vec<Pid55041Sg5Z17>,
    pub sg6: Vec<Pid55041Sg6>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55041Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55041Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// PID 55041: Ablehnung Kündigung MSB
/// Kommunikation: MSBA an MSBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55041 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55041Sg2>,
    pub sg4: Vec<Pid55041Sg4>,
}
