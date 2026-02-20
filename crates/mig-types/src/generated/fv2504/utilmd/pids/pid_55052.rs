//! Auto-generated PID 55052 types.
//! Bestätigung Ende MSB
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55052: Bestätigung Ende MSB
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55052 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
