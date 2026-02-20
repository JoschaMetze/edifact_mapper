//! Auto-generated PID 55625 types.
//! Rückmeldung/Anfrage Daten der Tranche
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55625: Rückmeldung/Anfrage Daten der Tranche
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55625 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
