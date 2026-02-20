//! Auto-generated PID 55011 types.
//! Bestätigung Beendigung der Zuordnung
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55011: Bestätigung Beendigung der Zuordnung
/// Kommunikation: LF an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55011 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
