//! Auto-generated PID 55691 types.
//! Änderung Paket-ID der Malo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg10 {
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: VY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg12Vy {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55691Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_vy: Vec<Pid55691Sg12Vy>,
    pub sg5_z16: Vec<Pid55691Sg5Z16>,
    pub sg6: Vec<Pid55691Sg6>,
    pub sg8_z01: Vec<Pid55691Sg8Z01>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691Sg8Z01 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55691Sg10>,
}

/// PID 55691: Änderung Paket-ID der Malo
/// Kommunikation: NBA an LF/ MSB / NBN / ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55691Sg2>,
    pub sg4: Vec<Pid55691Sg4>,
}
