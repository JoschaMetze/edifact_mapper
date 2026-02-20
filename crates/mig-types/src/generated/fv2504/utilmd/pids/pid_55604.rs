//! Auto-generated PID 55604 types.
//! Ablehnung Anmeldung neue verb. MaLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55604: Ablehnung Anmeldung neue verb. MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55604 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
