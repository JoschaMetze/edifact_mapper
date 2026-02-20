//! Auto-generated PID 55687 types.
//! Rückmeldung/Anfrage Daten der Tranche
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55687: Rückmeldung/Anfrage Daten der Tranche
/// Kommunikation: ÜNB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55687 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
