//! Auto-generated PID 55177 types.
//! Rückmeldung/Anfrage Lokationsbündelstruktur
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55177: Rückmeldung/Anfrage Lokationsbündelstruktur
/// Kommunikation: MSB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55177 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
