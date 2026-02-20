//! Auto-generated PID 55073 types.
//! Übermittlung der Profildefinitionen
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55073Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg4 {
    pub ftx: Option<super::super::segments::SegFtx>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sg6: Vec<Pid55073Sg6>,
    pub sg8_z21: Vec<Pid55073Sg8Z21>,
    pub sg8_z08: Vec<Pid55073Sg8Z08>,
    pub sg8_z38: Vec<Pid55073Sg8Z38>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z08
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg8Z08 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55073Sg10>,
    pub sg9: Vec<Pid55073Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg8Z21 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55073Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z38
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg8Z38 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55073Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55073: Übermittlung der Profildefinitionen
/// Kommunikation: NB an LF/ MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55073Sg2>,
    pub sg4: Vec<Pid55073Sg4>,
}
