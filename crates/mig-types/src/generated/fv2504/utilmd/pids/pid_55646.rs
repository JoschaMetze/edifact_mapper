//! Auto-generated PID 55646 types.
//! Rückmeldung/Anfrage Daten der SR
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55646: Rückmeldung/Anfrage Daten der SR
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55646 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
