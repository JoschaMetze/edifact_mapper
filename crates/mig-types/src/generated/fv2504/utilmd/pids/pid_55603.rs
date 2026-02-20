//! Auto-generated PID 55603 types.
//! Bestätigung Anmeldung neue erz. MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55603: Bestätigung Anmeldung neue erz. MaLo
/// Kommunikation: NB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55603 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
