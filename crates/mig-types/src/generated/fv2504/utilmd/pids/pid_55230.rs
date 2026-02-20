//! Auto-generated PID 55230 types.
//! Änderung Blindabr.-Daten der NeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg10 {
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55230Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z18: Vec<Pid55230Sg5Z18>,
    pub sg6: Vec<Pid55230Sg6>,
    pub sg8_z71: Vec<Pid55230Sg8Z71>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z71
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230Sg8Z71 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55230Sg10>,
}

/// PID 55230: Änderung Blindabr.-Daten der NeLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55230 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55230Sg2>,
    pub sg4: Vec<Pid55230Sg4>,
}
