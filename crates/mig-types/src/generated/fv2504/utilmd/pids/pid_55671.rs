//! Auto-generated PID 55671 types.
//! Rückmeldung auf Stammdaten BK-Treue
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55671: Rückmeldung auf Stammdaten BK-Treue
/// Kommunikation: ÜNB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55671 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
