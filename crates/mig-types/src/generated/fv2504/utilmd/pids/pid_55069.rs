//! Auto-generated PID 55069 types.
//! Clearingliste DZR
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55069: Clearingliste DZR
/// Kommunikation: BIKO an NB/ ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55069 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<super::super::groups::Sg1>,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
