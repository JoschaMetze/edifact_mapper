//! Auto-generated PID 55632 types.
//! Änderung Daten der MeLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55632: Änderung Daten der MeLo
/// Kommunikation: NB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55632 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
