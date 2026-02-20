//! Auto-generated PID 55686 types.
//! Änderung Daten der Tranche
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55686: Änderung Daten der Tranche
/// Kommunikation: MSB an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55686 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
