//! Auto-generated PID 55035 types.
//! Antwort auf GDA verb. MaLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55035: Antwort auf GDA verb. MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55035 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
