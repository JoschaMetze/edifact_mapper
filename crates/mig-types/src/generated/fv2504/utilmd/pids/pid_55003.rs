//! Auto-generated PID 55003 types.
//! Ablehnung Anmeldung verb. MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// SG12 — Beteiligter, Qualifier
/// Qualifiers: VY
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55003Sg12Vy {
    pub nad: Option<super::super::segments::SegNad>,
}

/// SG2 — Beteiligter, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55003Sg2 {
    pub nad: Option<super::super::segments::SegNad>,
    pub sg3_ic: Vec<Pid55003Sg3Ic>,
}

/// SG3 — Funktion des Ansprechpartners, Code
/// Qualifiers: IC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55003Sg3Ic {
    pub com: Option<super::super::segments::SegCom>,
    pub cta: Option<super::super::segments::SegCta>,
}

/// SG4 — Objekt, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55003Sg4 {
    pub dtm: Option<super::super::segments::SegDtm>,
    pub ftx: Option<super::super::segments::SegFtx>,
    pub ide: Option<super::super::segments::SegIde>,
    pub sts: Option<super::super::segments::SegSts>,
    pub sg12_vy: Vec<Pid55003Sg12Vy>,
    pub sg6: Vec<Pid55003Sg6>,
}

/// SG6 — Referenz, Qualifier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55003Sg6 {
    pub rff: Option<super::super::segments::SegRff>,
}

/// PID 55003: Ablehnung Anmeldung verb. MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55003 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<Pid55003Sg2>,
    pub sg4: Vec<Pid55003Sg4>,
}
