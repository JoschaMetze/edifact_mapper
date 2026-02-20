//! Auto-generated PID 55013 types.
//! Anmeldung / Zuordnung EOG
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55013: Anmeldung / Zuordnung EOG
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55013 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
