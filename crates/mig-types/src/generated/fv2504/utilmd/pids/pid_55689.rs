//! Auto-generated PID 55689 types.
//! Rückmeldung/Anfrage Daten der MaLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55689: Rückmeldung/Anfrage Daten der MaLo
/// Kommunikation: ÜNB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55689 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
