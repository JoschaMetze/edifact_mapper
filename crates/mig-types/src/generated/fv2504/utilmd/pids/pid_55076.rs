//! Auto-generated PID 55076 types.
//! Antwort auf Stammdaten-änderung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55076: Antwort auf Stammdaten-änderung
/// Kommunikation: UBA an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55076 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
