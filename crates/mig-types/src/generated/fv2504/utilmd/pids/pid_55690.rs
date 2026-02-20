//! Auto-generated PID 55690 types.
//! Lokationsbündelstruktur und DB
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55690: Lokationsbündelstruktur und DB
/// Kommunikation: NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55690 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
