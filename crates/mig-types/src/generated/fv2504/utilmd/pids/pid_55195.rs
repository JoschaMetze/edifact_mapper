//! Auto-generated PID 55195 types.
//! Bilanzierungs-gebiets-clearing-liste
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55195: Bilanzierungs-gebiets-clearing-liste
/// Kommunikation: ÜNB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55195 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<super::super::groups::Sg1>,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
