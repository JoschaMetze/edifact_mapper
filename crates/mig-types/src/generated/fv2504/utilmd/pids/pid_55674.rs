//! Auto-generated PID 55674 types.
//! Abr.-Daten BK-Abr. erz. Malo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55674: Abr.-Daten BK-Abr. erz. Malo
/// Kommunikation: NB an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55674 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
