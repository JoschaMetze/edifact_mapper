//! Auto-generated PID 55662 types.
//! Änderung Daten der Tranche
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55662: Änderung Daten der Tranche
/// Kommunikation: MSB an weiteren MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55662 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
