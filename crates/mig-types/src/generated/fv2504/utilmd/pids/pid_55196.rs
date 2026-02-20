//! Auto-generated PID 55196 types.
//! Antwort auf Bilanzierungs-gebiets-clearing-liste
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55196: Antwort auf Bilanzierungs-gebiets-clearing-liste
/// Kommunikation: NB an ÜNB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55196 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
