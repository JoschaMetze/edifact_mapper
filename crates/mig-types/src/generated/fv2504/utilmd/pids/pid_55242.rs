//! Auto-generated PID 55242 types.
//! Abmeldung aus dem Modell 2
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55242: Abmeldung aus dem Modell 2
/// Kommunikation: NB (LPB) an NB (VNB)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55242 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
