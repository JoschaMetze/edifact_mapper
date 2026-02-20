//! Auto-generated PID 55199 types.
//! Aktivierung ZP LF-AASZR
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55199: Aktivierung ZP LF-AASZR
/// Kommunikation: NB (ANB) an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55199 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
