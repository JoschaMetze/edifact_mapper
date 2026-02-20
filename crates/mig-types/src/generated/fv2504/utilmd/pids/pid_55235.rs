//! Auto-generated PID 55235 types.
//! Zuordnung ZP der NGZ zur NZR
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55235: Zuordnung ZP der NGZ zur NZR
/// Kommunikation: NB an NB NB an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55235 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
