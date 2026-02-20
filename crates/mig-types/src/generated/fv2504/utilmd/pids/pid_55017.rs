//! Auto-generated PID 55017 types.
//! Bestätigung Kündigung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55017: Bestätigung Kündigung
/// Kommunikation: LFA an LFN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55017 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
