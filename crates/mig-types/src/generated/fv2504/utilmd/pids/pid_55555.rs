//! Auto-generated PID 55555 types.
//! Rückmeldung/Anfrage Daten der individuellen Bestellung
//! Do not edit manually.

use serde::{Serialize, Deserialize};

/// PID 55555: Rückmeldung/Anfrage Daten der individuellen Bestellung
/// Kommunikation: NB / LF / MSB an MSB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55555 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
