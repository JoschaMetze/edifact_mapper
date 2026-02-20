//! Auto-generated PID 55607 types.
//! (Ankündigung) Zuordnung des LF zur erz. MaLo/ Tranche
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55607: (Ankündigung) Zuordnung des LF zur erz. MaLo/ Tranche
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55607 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
