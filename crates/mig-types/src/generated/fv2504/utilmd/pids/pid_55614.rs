//! Auto-generated PID 55614 types.
//! Rückmeldung/Anfrage Abr.-Daten BK-Abr. verb. MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG10 — Merkmal, Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg10 {
    pub cav: Option<super::super::segments::SegCav>,
    pub cci: Option<super::super::segments::SegCci>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55614Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg4 {
    pub ftx: Option<super::super::segments::SegFtx>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg5_z16: Vec<Pid55614Sg5Z16>,
    pub sg6: Vec<Pid55614Sg6>,
    pub sg8_z80_z81: Vec<Pid55614Sg8Z80Z81>,
    pub sg8_z85_z86: Vec<Pid55614Sg8Z85Z86>,
}

/// SG5 — Ortsangabe, Qualifier
/// Qualifiers: Z16
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg5Z16 {
    pub loc: Option<super::super::segments::SegLoc>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg6 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub rff: Option<super::super::segments::SegRff>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z80, Z81
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg8Z80Z81 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55614Sg10>,
    pub sg9: Vec<Pid55614Sg9>,
}

/// SG8 — Handlung, Code
/// Qualifiers: Z85, Z86
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg8Z85Z86 {
    pub seq: Option<super::super::segments::SegSeq>,
    pub sg10: Vec<Pid55614Sg10>,
}

/// SG9 — Menge, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614Sg9 {
    pub qty: Option<super::super::segments::SegQty>,
}

/// PID 55614: Rückmeldung/Anfrage Abr.-Daten BK-Abr. verb. MaLo
/// Kommunikation: ÜNB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55614 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55614Sg2>,
    pub sg4: Vec<Pid55614Sg4>,
}
