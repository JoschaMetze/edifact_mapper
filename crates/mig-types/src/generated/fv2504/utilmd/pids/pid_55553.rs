//! Auto-generated PID 55553 types.
//! Daten auf individuelle Bestellung
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55553: Daten auf individuelle Bestellung
/// Kommunikation: MSB an NB / LF / MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55553 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
