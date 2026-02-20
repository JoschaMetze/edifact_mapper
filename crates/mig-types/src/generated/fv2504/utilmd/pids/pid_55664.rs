//! Auto-generated PID 55664 types.
//! Rückmeldung/Anfrage Daten der NeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Klassentyp, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55664Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z18: Vec<Pid55664Sg5Z18>,
    pub sg6: Vec<Pid55664Sg6>,
    pub sg8_za9_zb0: Vec<Pid55664Sg8Za9Zb0>,
    pub sg8_za7_za8: Vec<Pid55664Sg8Za7Za8>,
    pub sg8_zg8_zg9: Vec<Pid55664Sg8Zg8Zg9>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z18
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg5Z18 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA7, ZA8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg8Za7Za8 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55664Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA9, ZB0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg8Za9Zb0 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55664Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG8, ZG9
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664Sg8Zg8Zg9 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55664Sg10>,
}

/// PID 55664: Rückmeldung/Anfrage Daten der NeLo
/// Kommunikation: weiterer MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55664 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55664Sg2>,
    pub sg4: Vec<Pid55664Sg4>,
}
