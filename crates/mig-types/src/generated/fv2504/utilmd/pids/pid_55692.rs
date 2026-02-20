//! Auto-generated PID 55692 types.
//! Rückmeldung/Anfrage Paket-ID der Malo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55692: Rückmeldung/Anfrage Paket-ID der Malo
/// Kommunikation: LF/ MSB / ÜNB an NBA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55692 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
