//! Auto-generated PID 55225 types.
//! Änderung Blindabr.-Daten der NeLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55225: Änderung Blindabr.-Daten der NeLo
/// Kommunikation: NB an LF NBA an NBN
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55225 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
