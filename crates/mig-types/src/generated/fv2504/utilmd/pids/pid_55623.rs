//! Auto-generated PID 55623 types.
//! Rückmeldung/Anfrage Daten der TR
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55623: Rückmeldung/Anfrage Daten der TR
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55623 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
