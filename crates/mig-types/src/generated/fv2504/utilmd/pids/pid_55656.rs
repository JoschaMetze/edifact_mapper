//! Auto-generated PID 55656 types.
//! Rückmeldung/Anfrage Daten der SR
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55656Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z19: Vec<Pid55656Sg5Z19>,
    pub sg6: Vec<Pid55656Sg6>,
    pub sg8_zb1_zb2: Vec<Pid55656Sg8Zb1Zb2>,
    pub sg8_zb3_zb4: Vec<Pid55656Sg8Zb3Zb4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z19
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg5Z19 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB1, ZB2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg8Zb1Zb2 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55656Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB3, ZB4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656Sg8Zb3Zb4 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55656Sg10>,
}

/// PID 55656: Rückmeldung/Anfrage Daten der SR
/// Kommunikation: LF an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55656 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55656Sg2>,
    pub sg4: Vec<Pid55656Sg4>,
}
