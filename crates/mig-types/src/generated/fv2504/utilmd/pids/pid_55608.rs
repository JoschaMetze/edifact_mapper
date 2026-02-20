//! Auto-generated PID 55608 types.
//! Bestätigung Zuordnung des LF zur erz. MaLo/ Tranche
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55608Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ftx: Option<super::super::segments::SegFtx>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg6: Vec<Pid55608Sg6>,
    pub sg8_z79: Vec<Pid55608Sg8Z79>,
    pub sg8_zh0: Vec<Pid55608Sg8Zh0>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z79
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg8Z79 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55608Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZH0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608Sg8Zh0 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55608Sg10>,
}

/// PID 55608: Bestätigung Zuordnung des LF zur erz. MaLo/ Tranche
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55608Sg2>,
    pub sg4: Vec<Pid55608Sg4>,
}
