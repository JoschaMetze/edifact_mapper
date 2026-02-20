//! Auto-generated PID 55073 types.
//! Übermittlung der Profildefinitionen
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55073: Übermittlung der Profildefinitionen
/// Kommunikation: NB an LF/ MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55073 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
