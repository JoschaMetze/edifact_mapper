//! Auto-generated PID 55210 types.
//! Antwort auf Aktivierung ZP
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55210: Antwort auf Aktivierung ZP
/// Kommunikation: BIKO an NB (ANB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55210 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
