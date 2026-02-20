//! Auto-generated PID 55691 types.
//! Änderung Paket-ID der Malo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55691: Änderung Paket-ID der Malo
/// Kommunikation: NBA an LF/ MSB / NBN / ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55691 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
