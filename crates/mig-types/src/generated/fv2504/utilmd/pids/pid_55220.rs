//! Auto-generated PID 55220 types.
//! Rückmeldung/Anfrage Abr.-Daten NNA
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55220: Rückmeldung/Anfrage Abr.-Daten NNA
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55220 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
