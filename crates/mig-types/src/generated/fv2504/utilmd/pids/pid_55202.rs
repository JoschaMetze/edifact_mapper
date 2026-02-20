//! Auto-generated PID 55202 types.
//! Korrekturliste LF-AACL
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55202: Korrekturliste LF-AACL
/// Kommunikation: LF an NB (ANB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55202 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
