//! Auto-generated PID 55218 types.
//! Abr.-Daten NNA
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55218: Abr.-Daten NNA
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55218 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
