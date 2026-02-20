//! Auto-generated PID 55559 types.
//! Rückmeldung/Anfrage MSB-Abr.-Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55559Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z16: Vec<Pid55559Sg5Z16>,
    pub sg6: Vec<Pid55559Sg6>,
    pub sg8_zc5_zc6: Vec<Pid55559Sg8Zc5Zc6>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZC5, ZC6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg8Zc5Zc6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg9: Vec<Pid55559Sg9>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55559: Rückmeldung/Anfrage MSB-Abr.-Daten der MaLo
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55559Sg2>,
    pub sg4: Vec<Pid55559Sg4>,
}
