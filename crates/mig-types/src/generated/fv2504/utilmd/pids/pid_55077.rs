//! Auto-generated PID 55077 types.
//! Anmeldung erz. MaLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55077: Anmeldung erz. MaLo
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55077 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
