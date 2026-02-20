//! Auto-generated PID 55658 types.
//! Rückmeldung/Anfrage Daten der MeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z45, Z46
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg12Z45Z46 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55658Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z45_z46: Vec<Pid55658Sg12Z45Z46>,
    pub sg5_z17: Vec<Pid55658Sg5Z17>,
    pub sg6: Vec<Pid55658Sg6>,
    pub sg8_zg6_zg7: Vec<Pid55658Sg8Zg6Zg7>,
    pub sg8_za3_za4: Vec<Pid55658Sg8Za3Za4>,
    pub sg8_za5_za6: Vec<Pid55658Sg8Za5Za6>,
    pub sg8_zb9_zc0: Vec<Pid55658Sg8Zb9Zc0>,
    pub sg8_zc3_zc4: Vec<Pid55658Sg8Zc3Zc4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA3, ZA4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg8Za3Za4 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55658Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA5, ZA6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg8Za5Za6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55658Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB9, ZC0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg8Zb9Zc0 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55658Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZC3, ZC4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg8Zc3Zc4 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55658Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG6, ZG7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658Sg8Zg6Zg7 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55658Sg10>,
}

/// PID 55658: Rückmeldung/Anfrage Daten der MeLo
/// Kommunikation: LF an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55658Sg2>,
    pub sg4: Vec<Pid55658Sg4>,
}
