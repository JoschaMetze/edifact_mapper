//! Auto-generated PID 55110 types.
//! Änderung Daten der MaLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55110: Änderung Daten der MaLo
/// Kommunikation: LF an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55110 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
