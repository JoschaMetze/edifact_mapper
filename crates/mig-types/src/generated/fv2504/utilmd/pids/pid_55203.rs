//! Auto-generated PID 55203 types.
//! Aktivierung ZP monatliche AAÜZ
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55203: Aktivierung ZP monatliche AAÜZ
/// Kommunikation: NB (ANB) an BIKO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55203 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
