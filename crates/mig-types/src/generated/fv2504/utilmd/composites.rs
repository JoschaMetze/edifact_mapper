//! Auto-generated composite structs from MIG XML.
//! Do not edit manually.

use super::enums::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC002 {
    /// Hier wird die Kategorie der gesamten Nachricht für alle Vorgänge angegeben:
    pub d1001: D1001Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC056 {
    pub d3413: Option<String>,
    pub d3412: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC058 {
    pub d3124: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC059 {
    pub d3042_1: String,
    pub d3042_2: Option<String>,
    pub d3042_3: Option<String>,
    pub d3042_4: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC076 {
    pub d3148: String,
    pub d3155: D3155Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC080 {
    pub d3036_1: String,
    pub d3036_2: Option<String>,
    pub d3036_3: Option<String>,
    pub d3036_4: Option<String>,
    pub d3036_5: Option<String>,
    pub d3045: D3045Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC082 {
    pub d3039: String,
    pub d1131: Option<D1131Qualifier>,
    pub d3055: D3055Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC106 {
    /// EDI-Nachrichtennummer vergeben vom Absender des Dokuments
    pub d1004: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC107 {
    pub d4441: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC108 {
    pub d4440_1: String,
    pub d4440_2: Option<String>,
    pub d4440_3: Option<String>,
    pub d4440_4: Option<String>,
    pub d4440_5: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC186 {
    pub d6063: D6063Qualifier,
    /// Mengenangabe
    pub d6060: String,
    pub d6411: D6411Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC206 {
    pub d7402: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC212 {
    pub d7140: D7140Qualifier,
    pub d7143: D7143Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC240 {
    pub d7037: D7037Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC286 {
    pub d1050: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC502 {
    pub d6313: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC506 {
    pub d1153: D1153Qualifier,
    pub d1154: D1154Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC507 {
    pub d2005: D2005Qualifier,
    pub d2380: String,
    pub d2379: D2379Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC517 {
    pub d3225: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC543 {
    pub d7431: D7431Qualifier,
    pub d7433: D7433Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC555 {
    pub d4405: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC556 {
    pub d9013: D9013Qualifier,
    pub d1131: D1131Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC601 {
    pub d9015: D9015Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC819 {
    pub d3229: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeC889 {
    pub d7111: D7111Qualifier,
    pub d1131: Option<D1131Qualifier>,
    pub d3055: Option<D3055Qualifier>,
    pub d7110: D7110Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS001 {
    pub d0001: D0001Qualifier,
    /// 3 = Syntax-Versionsnummer 3
    pub d0002: D0002Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS002 {
    /// Internationale Lokationsnummer (n13) oder BDEW-Codenummer
    pub d0004: String,
    pub d0007: D0007Qualifier,
    pub d0008: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS003 {
    /// Internationale Lokationsnummer (n13) oder BDEW-Codenummer
    pub d0010: String,
    /// 14 = GS1 (ehem. EAN International)
    pub d0007: D0007Qualifier,
    pub d0014: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS004 {
    /// JJMMTT
    pub d0017: String,
    /// HHMM
    pub d0019: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS005 {
    pub d0022: String,
    pub d0025: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS009 {
    pub d0065: D0065Qualifier,
    pub d0052: D0052Qualifier,
    pub d0054: D0054Qualifier,
    pub d0051: D0051Qualifier,
    pub d0057: D0057Qualifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeS010 {
    /// Laufende Nummer bei Aufteilung von Nachrichten
    pub d0070: String,
    pub d0073: Option<D0073Qualifier>,
}
