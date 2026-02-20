//! Auto-generated PID 55018 types.
//! Ablehnung Kündigung
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55018: Ablehnung Kündigung
/// Kommunikation: LFA an LFN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55018 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
