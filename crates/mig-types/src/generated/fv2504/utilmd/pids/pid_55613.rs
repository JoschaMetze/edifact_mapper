//! Auto-generated PID 55613 types.
//! Abr.-Daten BK-Abr. verb. MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55613Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z16: Vec<Pid55613Sg5Z16>,
    pub sg6: Vec<Pid55613Sg6>,
    pub sg8_z01_z98: Vec<Pid55613Sg8Z01Z98>,
    pub sg8_z21: Vec<Pid55613Sg8Z21>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z01, Z98
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg8Z01Z98 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55613Sg10>,
    pub sg9: Vec<Pid55613Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z21
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg8Z21 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55613Sg10>,
}

/// SG9 — Veranschlagte Jahresmenge gesamt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55613: Abr.-Daten BK-Abr. verb. MaLo
/// Kommunikation: NB an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55613 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55613Sg2>,
    pub sg4: Vec<Pid55613Sg4>,
}
