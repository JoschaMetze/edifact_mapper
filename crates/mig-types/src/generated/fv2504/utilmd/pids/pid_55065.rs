//! Auto-generated PID 55065 types.
//! Lieferanten-clearingliste
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55065: Lieferanten-clearingliste
/// Kommunikation: NB an LF ÜNB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55065 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<super::super::groups::Sg1>,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
