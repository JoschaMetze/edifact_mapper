//! Auto-generated PID 55211 types.
//! Weiterleitung Aktivierung ZP
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55211: Weiterleitung Aktivierung ZP
/// Kommunikation: BIKO an BKV (des anfNB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55211 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
