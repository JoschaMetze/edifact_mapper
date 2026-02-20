//! Auto-generated PID 55669 types.
//! Rückmeldung/Anfrage Daten der MeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z39, Z40
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z39Z40 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z41, Z42
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z41Z42 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z43, Z44
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z43Z44 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z45, Z46
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg12Z45Z46 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55669Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z39_z40: Vec<Pid55669Sg12Z39Z40>,
    pub sg12_z41_z42: Vec<Pid55669Sg12Z41Z42>,
    pub sg12_z43_z44: Vec<Pid55669Sg12Z43Z44>,
    pub sg12_z45_z46: Vec<Pid55669Sg12Z45Z46>,
    pub sg5_z17: Vec<Pid55669Sg5Z17>,
    pub sg6: Vec<Pid55669Sg6>,
    pub sg8_zg6_zg7: Vec<Pid55669Sg8Zg6Zg7>,
    pub sg8_za3_za4: Vec<Pid55669Sg8Za3Za4>,
    pub sg8_za5_za6: Vec<Pid55669Sg8Za5Za6>,
    pub sg8_zb9_zc0: Vec<Pid55669Sg8Zb9Zc0>,
    pub sg8_zb7_zb8: Vec<Pid55669Sg8Zb7Zb8>,
    pub sg8_zc1_zc2: Vec<Pid55669Sg8Zc1Zc2>,
    pub sg8_zc3_zc4: Vec<Pid55669Sg8Zc3Zc4>,
    pub sg8_zh3_zh4: Vec<Pid55669Sg8Zh3Zh4>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z17
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg5Z17 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA3, ZA4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Za3Za4 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA5, ZA6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Za5Za6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB7, ZB8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zb7Zb8 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB9, ZC0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zb9Zc0 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZC1, ZC2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zc1Zc2 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZC3, ZC4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zc3Zc4 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZG6, ZG7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zg6Zg7 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZH3, ZH4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669Sg8Zh3Zh4 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55669Sg10>,
}

/// PID 55669: Rückmeldung/Anfrage Daten der MeLo
/// Kommunikation: weiterer MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55669 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55669Sg2>,
    pub sg4: Vec<Pid55669Sg4>,
}
