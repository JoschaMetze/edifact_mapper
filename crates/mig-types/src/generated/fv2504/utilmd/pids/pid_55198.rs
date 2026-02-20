//! Auto-generated PID 55198 types.
//! Deaktivierung tägliche AAÜZ
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55198Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sg5_z15: Vec<Pid55198Sg5Z15>,
    pub sg6: Vec<Pid55198Sg6>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z15
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198Sg5Z15 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// PID 55198: Deaktivierung tägliche AAÜZ
/// Kommunikation: NB (ANB) an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55198Sg2>,
    pub sg4: Vec<Pid55198Sg4>,
}
