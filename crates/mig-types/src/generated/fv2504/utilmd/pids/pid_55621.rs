//! Auto-generated PID 55621 types.
//! Rückmeldung/Anfrage Daten zur NeLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55621: Rückmeldung/Anfrage Daten zur NeLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55621 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
