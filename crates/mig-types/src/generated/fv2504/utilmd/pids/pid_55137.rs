//! Auto-generated PID 55137 types.
//! Rückmeldung/Anfrage Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z47, Z48
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg12Z47Z48 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: Z49, Z50
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg12Z49Z50 {
    pub nad: Option<super::super::segments::SegNad>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55137Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg4 {
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_z47_z48: Vec<Pid55137Sg12Z47Z48>,
    pub sg12_z49_z50: Vec<Pid55137Sg12Z49Z50>,
    pub sg5_z16: Vec<Pid55137Sg5Z16>,
    pub sg6: Vec<Pid55137Sg6>,
    pub sg8_z80_z81: Vec<Pid55137Sg8Z80Z81>,
    pub sg8_z92_z93: Vec<Pid55137Sg8Z92Z93>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z80, Z81
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg8Z80Z81 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55137Sg10>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z92, Z93
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137Sg8Z92Z93 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55137Sg10>,
}

/// PID 55137: Rückmeldung/Anfrage Daten der MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55137 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55137Sg2>,
    pub sg4: Vec<Pid55137Sg4>,
}
