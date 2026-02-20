//! Auto-generated PID 55615 types.
//! Änderung Daten der NeLo
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55615: Änderung Daten der NeLo
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55615 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
