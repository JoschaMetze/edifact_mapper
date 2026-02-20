//! Auto-generated PID 55074 types.
//! Stammdaten auf eine ORDERS
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55074: Stammdaten auf eine ORDERS
/// Kommunikation: NB an UBA
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55074 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg1: Vec<super::super::groups::Sg1>,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
