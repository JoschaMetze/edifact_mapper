//! Auto-generated PID 55658 types.
//! Rückmeldung/Anfrage Daten der MeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55658: Rückmeldung/Anfrage Daten der MeLo
/// Kommunikation: LF an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55658 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
