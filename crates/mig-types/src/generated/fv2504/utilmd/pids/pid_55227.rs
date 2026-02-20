//! Auto-generated PID 55227 types.
//! Rückmeldung/Anfrage Blindabr.-Daten der NeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55227: Rückmeldung/Anfrage Blindabr.-Daten der NeLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55227 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
