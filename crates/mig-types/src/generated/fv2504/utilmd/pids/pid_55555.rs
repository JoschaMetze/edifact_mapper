//! Auto-generated PID 55555 types.
//! Rückmeldung/Anfrage Daten der individuellen Bestellung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55555Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg4 {
    pub ftx: Option<super::super::segments::SegFtx>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg6: Vec<Pid55555Sg6>,
    pub sg8_za1_za2: Vec<Pid55555Sg8Za1Za2>,
    pub sg8_za3_za4: Vec<Pid55555Sg8Za3Za4>,
    pub sg8_za5_za6: Vec<Pid55555Sg8Za5Za6>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA1, ZA2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg8Za1Za2 {
    pub pia: Option<super::super::segments::SegPia>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55555Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA3, ZA4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg8Za3Za4 {
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55555Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: ZA5, ZA6
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555Sg8Za5Za6 {
    pub pia: Option<super::super::segments::SegPia>,
    pub rff: Option<super::super::segments::SegRff>,
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55555Sg10>,
}

/// PID 55555: Rückmeldung/Anfrage Daten der individuellen Bestellung
/// Kommunikation: NB / LF / MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55555Sg2>,
    pub sg4: Vec<Pid55555Sg4>,
}
