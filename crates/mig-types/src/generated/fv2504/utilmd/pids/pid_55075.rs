//! Auto-generated PID 55075 types.
//! Stammdaten aufgrund einer Änderung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55075: Stammdaten aufgrund einer Änderung
/// Kommunikation: NB an UBA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55075 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
