//! Auto-generated PID 55240 types.
//! Beendigung der Zuordnung zur MaLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55240: Beendigung der Zuordnung zur MaLo
/// Kommunikation: NB (VNB)an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55240 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
