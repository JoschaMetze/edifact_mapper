//! Auto-generated PID 55067 types.
//! Bilanzkreiszuordnungsliste
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55067: Bilanzkreiszuordnungsliste
/// Kommunikation: NB an BKV ÜNB an BKV
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55067 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<super::super::groups::Sg1>,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
