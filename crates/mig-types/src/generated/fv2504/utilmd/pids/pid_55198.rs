//! Auto-generated PID 55198 types.
//! Deaktivierung tägliche AAÜZ
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55198: Deaktivierung tägliche AAÜZ
/// Kommunikation: NB (ANB) an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55198 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
