//! Auto-generated PID 55126 types.
//! Abr.-Daten BK-Abr. verb. Malo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55126Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z16: Vec<Pid55126Sg5Z16>,
    pub sg6: Vec<Pid55126Sg6>,
    pub sg8_z01: Vec<Pid55126Sg8Z01>,
    pub sg8_z21: Vec<Pid55126Sg8Z21>,
    pub sg8_z08: Vec<Pid55126Sg8Z08>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg8Z01 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55126Sg10>,
    pub sg9: Vec<Pid55126Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg8Z08 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55126Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg8Z21 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55126Sg10>,
}

/// SG9 — Arbeit / Leistung für tagesparameterabhängige Marktlokation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55126: Abr.-Daten BK-Abr. verb. Malo
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55126Sg2>,
    pub sg4: Vec<Pid55126Sg4>,
}
