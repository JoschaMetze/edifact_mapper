//! Auto-generated PID 55559 types.
//! Rückmeldung/Anfrage MSB-Abr.-Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55559: Rückmeldung/Anfrage MSB-Abr.-Daten der MaLo
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55559 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
