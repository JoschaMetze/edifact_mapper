//! Auto-generated PID 55617 types.
//! Änderung Daten der TR
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55617: Änderung Daten der TR
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55617 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
