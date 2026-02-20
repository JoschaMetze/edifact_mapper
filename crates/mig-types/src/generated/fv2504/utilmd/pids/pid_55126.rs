//! Auto-generated PID 55126 types.
//! Abr.-Daten BK-Abr. verb. Malo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55126: Abr.-Daten BK-Abr. verb. Malo
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55126 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
