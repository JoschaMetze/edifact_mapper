//! Auto-generated PID 55039 types.
//! Kündigung MSB
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55039: Kündigung MSB
/// Kommunikation: MSBN an MSBA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55039 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
