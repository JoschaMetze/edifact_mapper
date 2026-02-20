//! Auto-generated PID 55665 types.
//! Rückmeldung/Anfrage Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55665Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z16: Vec<Pid55665Sg5Z16>,
    pub sg6: Vec<Pid55665Sg6>,
    pub sg8_za1_za2: Vec<Pid55665Sg8Za1Za2>,
    pub sg8_zb5_zb6: Vec<Pid55665Sg8Zb5Zb6>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA1, ZA2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg8Za1Za2 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55665Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZB5, ZB6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665Sg8Zb5Zb6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55665Sg10>,
}

/// PID 55665: Rückmeldung/Anfrage Daten der MaLo
/// Kommunikation: weiteren MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55665 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55665Sg2>,
    pub sg4: Vec<Pid55665Sg4>,
}
