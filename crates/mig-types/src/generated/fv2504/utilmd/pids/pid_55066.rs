//! Auto-generated PID 55066 types.
//! Korrekturliste zur Lieferanten-clearingliste
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55066: Korrekturliste zur Lieferanten-clearingliste
/// Kommunikation: LF an NB/ ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55066 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
