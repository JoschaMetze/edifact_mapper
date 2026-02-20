//! Auto-generated PID 55239 types.
//! Antwort auf Anmeldung
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55239: Antwort auf Anmeldung
/// Kommunikation: NB (VNB)an NB (LPB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55239 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
