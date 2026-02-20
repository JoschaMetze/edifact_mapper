//! Auto-generated PID 55205 types.
//! Weiterleitung Aktivierung ZP
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55205: Weiterleitung Aktivierung ZP
/// Kommunikation: BIKO an BKV (des LF)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55205 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
