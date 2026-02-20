//! Auto-generated PID 55012 types.
//! Ablehnung Beendigung der Zuordnung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55012: Ablehnung Beendigung der Zuordnung
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55012 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
