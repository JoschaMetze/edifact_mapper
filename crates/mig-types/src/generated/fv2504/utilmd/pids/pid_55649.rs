//! Auto-generated PID 55649 types.
//! Änderung Daten der NeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55649: Änderung Daten der NeLo
/// Kommunikation: MSB an LF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55649 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
