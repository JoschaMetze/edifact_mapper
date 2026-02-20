//! Auto-generated PID 55608 types.
//! Bestätigung Zuordnung des LF zur erz. MaLo/ Tranche
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55608: Bestätigung Zuordnung des LF zur erz. MaLo/ Tranche
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55608 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
