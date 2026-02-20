//! Auto-generated PID 55238 types.
//! Anmeldung in Modell 2
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55238: Anmeldung in Modell 2
/// Kommunikation: NB (LPB) an NB (VNB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55238 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
