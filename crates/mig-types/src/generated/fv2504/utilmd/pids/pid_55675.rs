//! Auto-generated PID 55675 types.
//! Rückmeldung/Anfrage Abr.-Daten BK-Abr. erz. Malo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55675: Rückmeldung/Anfrage Abr.-Daten BK-Abr. erz. Malo
/// Kommunikation: ÜNB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55675 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
