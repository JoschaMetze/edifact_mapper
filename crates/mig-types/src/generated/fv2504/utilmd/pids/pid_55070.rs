//! Auto-generated PID 55070 types.
//! Clearingliste BAS
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55070: Clearingliste BAS
/// Kommunikation: BIKO an BKV
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55070 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<super::super::groups::Sg1>,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
