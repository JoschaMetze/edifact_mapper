//! Auto-generated code enums from MIG XML.
//! Do not edit manually.

#![allow(clippy::enum_variant_names, non_camel_case_types)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0001Qualifier {
    UNOC,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0001Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UNOC => write!(f, "UNOC"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0001Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "UNOC" => Self::UNOC,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0002Qualifier {
    _3,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0002Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_3 => write!(f, "3"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0002Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "3" => Self::_3,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0007Qualifier {
    _14,
    _500,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0007Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_14 => write!(f, "14"),
            Self::_500 => write!(f, "500"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0007Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "14" => Self::_14,
            "500" => Self::_500,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0029Qualifier {
    A,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0029Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0029Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "A" => Self::A,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0035Qualifier {
    _1,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0035Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_1 => write!(f, "1"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0035Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "1" => Self::_1,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0051Qualifier {
    UN,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0051Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UN => write!(f, "UN"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0051Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "UN" => Self::UN,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0052Qualifier {
    D,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0052Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::D => write!(f, "D"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0052Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "D" => Self::D,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0054Qualifier {
    _11A,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0054Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_11A => write!(f, "11A"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0054Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "11A" => Self::_11A,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0057Qualifier {
    S2_1,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0057Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::S2_1 => write!(f, "S2.1"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0057Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "S2.1" => Self::S2_1,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0065Qualifier {
    UTILMD,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0065Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UTILMD => write!(f, "UTILMD"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0065Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "UTILMD" => Self::UTILMD,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D0073Qualifier {
    C,
    F,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D0073Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::C => write!(f, "C"),
            Self::F => write!(f, "F"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D0073Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "C" => Self::C,
            "F" => Self::F,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D1001Qualifier {
    /// Dient der Mitteilung über die Aufnahme der Netznutzung an einer verbrauchenden Marktlokation z.B. bei Einzug oder Lieferantenwechsel, Aufnahme einer Direktvermarktung an einer erzeugenden Marktlokation oder Tranche, zur Übernahme des Messstellenbetriebes an einer Messlokation bei MSB-Wechsel.
    E01,
    /// Dient der Mitteilung über die Beendigung der Netznutzung an einer verbrauchenden Marktlokation z.B. bei Auszug oder Lieferantenwechsel, Beendigung einer Direktvermarktung an einer erzeugenden Marktlokation oder Tranche, zur Abgabe des Messstellenbetriebes an einer Messlokation bei MSB-Wechsel.
    E02,
    /// Dient der Mitteilung über die Änderungen von Stammdaten ohne dabei das Verhältnis Kunde, Lieferant und Messstellenbetreiber zu verändern. Z. B. Anpassung der Jahresverbrauchsprognose oder Namensänderung.
    E03,
    /// Dient der Übermittlung von Stammdaten zu einer Marktlokation oder Messlokation.
    Z14,
    /// Dient der Mitteilung über die Kündigung eines Liefer- oder Messstellenbetriebsvertrags zur Kommunikation zwischen zwei LF oder zwei MSB.
    E35,
    /// Diese Nachricht dient der Übermittlung von Hinweisen und zur Beendigung der Zuordnung eines LFA oder eines zukünftigen LF zu einer Marktlokation. Es erfordert keine Antwortnachricht.
    E44,
    /// Mit der Kategorie wird die Bilanzkreiszuordnungsliste nach MaBiS versendet.
    E40,
    /// Mit der Kategorie wird die Clearingliste nach MaBiS versendet.
    Z05,
    /// Unter dieser Kategorie werden die Aktivierungen und Deaktivierungen der Bilanzierungszeitreihen mit ihrer entsprechenden Kennzeichnung des Zeitreihentypen versendet.
    Z07,
    Z17,
    Z18,
    Z37,
    Z40,
    Z71,
    Z88,
    Z89,
    Z90,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D1001Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E01 => write!(f, "E01"),
            Self::E02 => write!(f, "E02"),
            Self::E03 => write!(f, "E03"),
            Self::Z14 => write!(f, "Z14"),
            Self::E35 => write!(f, "E35"),
            Self::E44 => write!(f, "E44"),
            Self::E40 => write!(f, "E40"),
            Self::Z05 => write!(f, "Z05"),
            Self::Z07 => write!(f, "Z07"),
            Self::Z17 => write!(f, "Z17"),
            Self::Z18 => write!(f, "Z18"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z40 => write!(f, "Z40"),
            Self::Z71 => write!(f, "Z71"),
            Self::Z88 => write!(f, "Z88"),
            Self::Z89 => write!(f, "Z89"),
            Self::Z90 => write!(f, "Z90"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D1001Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "E01" => Self::E01,
            "E02" => Self::E02,
            "E03" => Self::E03,
            "Z14" => Self::Z14,
            "E35" => Self::E35,
            "E44" => Self::E44,
            "E40" => Self::E40,
            "Z05" => Self::Z05,
            "Z07" => Self::Z07,
            "Z17" => Self::Z17,
            "Z18" => Self::Z18,
            "Z37" => Self::Z37,
            "Z40" => Self::Z40,
            "Z71" => Self::Z71,
            "Z88" => Self::Z88,
            "Z89" => Self::Z89,
            "Z90" => Self::Z90,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D1131Qualifier {
    E_0004,
    E_0014,
    E_0017,
    E_0039,
    E_0047,
    E_0049,
    E_0052,
    E_0068,
    E_0070,
    E_0096,
    E_0097,
    E_0104,
    E_0105,
    E_0009,
    E_0010,
    E_0011,
    E_0012,
    E_0015,
    E_0018,
    E_0020,
    E_0024,
    E_0027,
    E_0028,
    E_0034,
    E_0035,
    E_0071,
    E_0072,
    E_0078,
    E_0079,
    E_0102,
    E_0103,
    E_0408,
    E_0409,
    E_0410,
    E_0412,
    E_0415,
    E_0510,
    E_0511,
    E_0512,
    E_0513,
    E_0572,
    E_0574,
    E_0578,
    E_0583,
    E_0603,
    E_0604,
    E_0605,
    E_0606,
    E_0607,
    E_0608,
    E_0609,
    E_0610,
    E_0611,
    E_0612,
    E_0614,
    E_0615,
    E_0622,
    E_0623,
    E_0624,
    E_0639,
    S_0054,
    S_0055,
    S_0056,
    S_0059,
    S_0060,
    S_0063,
    S_0064,
    S_0086,
    S_0087,
    S_0090,
    S_0091,
    LAND,
    Z14,
    Z15,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D1131Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E_0004 => write!(f, "E_0004"),
            Self::E_0014 => write!(f, "E_0014"),
            Self::E_0017 => write!(f, "E_0017"),
            Self::E_0039 => write!(f, "E_0039"),
            Self::E_0047 => write!(f, "E_0047"),
            Self::E_0049 => write!(f, "E_0049"),
            Self::E_0052 => write!(f, "E_0052"),
            Self::E_0068 => write!(f, "E_0068"),
            Self::E_0070 => write!(f, "E_0070"),
            Self::E_0096 => write!(f, "E_0096"),
            Self::E_0097 => write!(f, "E_0097"),
            Self::E_0104 => write!(f, "E_0104"),
            Self::E_0105 => write!(f, "E_0105"),
            Self::E_0009 => write!(f, "E_0009"),
            Self::E_0010 => write!(f, "E_0010"),
            Self::E_0011 => write!(f, "E_0011"),
            Self::E_0012 => write!(f, "E_0012"),
            Self::E_0015 => write!(f, "E_0015"),
            Self::E_0018 => write!(f, "E_0018"),
            Self::E_0020 => write!(f, "E_0020"),
            Self::E_0024 => write!(f, "E_0024"),
            Self::E_0027 => write!(f, "E_0027"),
            Self::E_0028 => write!(f, "E_0028"),
            Self::E_0034 => write!(f, "E_0034"),
            Self::E_0035 => write!(f, "E_0035"),
            Self::E_0071 => write!(f, "E_0071"),
            Self::E_0072 => write!(f, "E_0072"),
            Self::E_0078 => write!(f, "E_0078"),
            Self::E_0079 => write!(f, "E_0079"),
            Self::E_0102 => write!(f, "E_0102"),
            Self::E_0103 => write!(f, "E_0103"),
            Self::E_0408 => write!(f, "E_0408"),
            Self::E_0409 => write!(f, "E_0409"),
            Self::E_0410 => write!(f, "E_0410"),
            Self::E_0412 => write!(f, "E_0412"),
            Self::E_0415 => write!(f, "E_0415"),
            Self::E_0510 => write!(f, "E_0510"),
            Self::E_0511 => write!(f, "E_0511"),
            Self::E_0512 => write!(f, "E_0512"),
            Self::E_0513 => write!(f, "E_0513"),
            Self::E_0572 => write!(f, "E_0572"),
            Self::E_0574 => write!(f, "E_0574"),
            Self::E_0578 => write!(f, "E_0578"),
            Self::E_0583 => write!(f, "E_0583"),
            Self::E_0603 => write!(f, "E_0603"),
            Self::E_0604 => write!(f, "E_0604"),
            Self::E_0605 => write!(f, "E_0605"),
            Self::E_0606 => write!(f, "E_0606"),
            Self::E_0607 => write!(f, "E_0607"),
            Self::E_0608 => write!(f, "E_0608"),
            Self::E_0609 => write!(f, "E_0609"),
            Self::E_0610 => write!(f, "E_0610"),
            Self::E_0611 => write!(f, "E_0611"),
            Self::E_0612 => write!(f, "E_0612"),
            Self::E_0614 => write!(f, "E_0614"),
            Self::E_0615 => write!(f, "E_0615"),
            Self::E_0622 => write!(f, "E_0622"),
            Self::E_0623 => write!(f, "E_0623"),
            Self::E_0624 => write!(f, "E_0624"),
            Self::E_0639 => write!(f, "E_0639"),
            Self::S_0054 => write!(f, "S_0054"),
            Self::S_0055 => write!(f, "S_0055"),
            Self::S_0056 => write!(f, "S_0056"),
            Self::S_0059 => write!(f, "S_0059"),
            Self::S_0060 => write!(f, "S_0060"),
            Self::S_0063 => write!(f, "S_0063"),
            Self::S_0064 => write!(f, "S_0064"),
            Self::S_0086 => write!(f, "S_0086"),
            Self::S_0087 => write!(f, "S_0087"),
            Self::S_0090 => write!(f, "S_0090"),
            Self::S_0091 => write!(f, "S_0091"),
            Self::LAND => write!(f, "LAND"),
            Self::Z14 => write!(f, "Z14"),
            Self::Z15 => write!(f, "Z15"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D1131Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "E_0004" => Self::E_0004,
            "E_0014" => Self::E_0014,
            "E_0017" => Self::E_0017,
            "E_0039" => Self::E_0039,
            "E_0047" => Self::E_0047,
            "E_0049" => Self::E_0049,
            "E_0052" => Self::E_0052,
            "E_0068" => Self::E_0068,
            "E_0070" => Self::E_0070,
            "E_0096" => Self::E_0096,
            "E_0097" => Self::E_0097,
            "E_0104" => Self::E_0104,
            "E_0105" => Self::E_0105,
            "E_0009" => Self::E_0009,
            "E_0010" => Self::E_0010,
            "E_0011" => Self::E_0011,
            "E_0012" => Self::E_0012,
            "E_0015" => Self::E_0015,
            "E_0018" => Self::E_0018,
            "E_0020" => Self::E_0020,
            "E_0024" => Self::E_0024,
            "E_0027" => Self::E_0027,
            "E_0028" => Self::E_0028,
            "E_0034" => Self::E_0034,
            "E_0035" => Self::E_0035,
            "E_0071" => Self::E_0071,
            "E_0072" => Self::E_0072,
            "E_0078" => Self::E_0078,
            "E_0079" => Self::E_0079,
            "E_0102" => Self::E_0102,
            "E_0103" => Self::E_0103,
            "E_0408" => Self::E_0408,
            "E_0409" => Self::E_0409,
            "E_0410" => Self::E_0410,
            "E_0412" => Self::E_0412,
            "E_0415" => Self::E_0415,
            "E_0510" => Self::E_0510,
            "E_0511" => Self::E_0511,
            "E_0512" => Self::E_0512,
            "E_0513" => Self::E_0513,
            "E_0572" => Self::E_0572,
            "E_0574" => Self::E_0574,
            "E_0578" => Self::E_0578,
            "E_0583" => Self::E_0583,
            "E_0603" => Self::E_0603,
            "E_0604" => Self::E_0604,
            "E_0605" => Self::E_0605,
            "E_0606" => Self::E_0606,
            "E_0607" => Self::E_0607,
            "E_0608" => Self::E_0608,
            "E_0609" => Self::E_0609,
            "E_0610" => Self::E_0610,
            "E_0611" => Self::E_0611,
            "E_0612" => Self::E_0612,
            "E_0614" => Self::E_0614,
            "E_0615" => Self::E_0615,
            "E_0622" => Self::E_0622,
            "E_0623" => Self::E_0623,
            "E_0624" => Self::E_0624,
            "E_0639" => Self::E_0639,
            "S_0054" => Self::S_0054,
            "S_0055" => Self::S_0055,
            "S_0056" => Self::S_0056,
            "S_0059" => Self::S_0059,
            "S_0060" => Self::S_0060,
            "S_0063" => Self::S_0063,
            "S_0064" => Self::S_0064,
            "S_0086" => Self::S_0086,
            "S_0087" => Self::S_0087,
            "S_0090" => Self::S_0090,
            "S_0091" => Self::S_0091,
            "LAND" => Self::LAND,
            "Z14" => Self::Z14,
            "Z15" => Self::Z15,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D1153Qualifier {
    AAV,
    Z13,
    TN,
    AUU,
    ACW,
    Z42,
    Z43,
    Z60,
    Z22,
    Z47,
    Z48,
    Z49,
    Z53,
    Z54,
    Z55,
    Z50,
    Z51,
    Z52,
    Z31,
    Z39,
    Z32,
    Z18,
    Z19,
    Z37,
    Z33,
    Z34,
    Z35,
    Z16,
    Z10,
    Z20,
    Z38,
    Z05,
    Z14,
    MG,
    AGK,
    Z12,
    AVE,
    Z46,
    AVC,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D1153Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AAV => write!(f, "AAV"),
            Self::Z13 => write!(f, "Z13"),
            Self::TN => write!(f, "TN"),
            Self::AUU => write!(f, "AUU"),
            Self::ACW => write!(f, "ACW"),
            Self::Z42 => write!(f, "Z42"),
            Self::Z43 => write!(f, "Z43"),
            Self::Z60 => write!(f, "Z60"),
            Self::Z22 => write!(f, "Z22"),
            Self::Z47 => write!(f, "Z47"),
            Self::Z48 => write!(f, "Z48"),
            Self::Z49 => write!(f, "Z49"),
            Self::Z53 => write!(f, "Z53"),
            Self::Z54 => write!(f, "Z54"),
            Self::Z55 => write!(f, "Z55"),
            Self::Z50 => write!(f, "Z50"),
            Self::Z51 => write!(f, "Z51"),
            Self::Z52 => write!(f, "Z52"),
            Self::Z31 => write!(f, "Z31"),
            Self::Z39 => write!(f, "Z39"),
            Self::Z32 => write!(f, "Z32"),
            Self::Z18 => write!(f, "Z18"),
            Self::Z19 => write!(f, "Z19"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z33 => write!(f, "Z33"),
            Self::Z34 => write!(f, "Z34"),
            Self::Z35 => write!(f, "Z35"),
            Self::Z16 => write!(f, "Z16"),
            Self::Z10 => write!(f, "Z10"),
            Self::Z20 => write!(f, "Z20"),
            Self::Z38 => write!(f, "Z38"),
            Self::Z05 => write!(f, "Z05"),
            Self::Z14 => write!(f, "Z14"),
            Self::MG => write!(f, "MG"),
            Self::AGK => write!(f, "AGK"),
            Self::Z12 => write!(f, "Z12"),
            Self::AVE => write!(f, "AVE"),
            Self::Z46 => write!(f, "Z46"),
            Self::AVC => write!(f, "AVC"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D1153Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "AAV" => Self::AAV,
            "Z13" => Self::Z13,
            "TN" => Self::TN,
            "AUU" => Self::AUU,
            "ACW" => Self::ACW,
            "Z42" => Self::Z42,
            "Z43" => Self::Z43,
            "Z60" => Self::Z60,
            "Z22" => Self::Z22,
            "Z47" => Self::Z47,
            "Z48" => Self::Z48,
            "Z49" => Self::Z49,
            "Z53" => Self::Z53,
            "Z54" => Self::Z54,
            "Z55" => Self::Z55,
            "Z50" => Self::Z50,
            "Z51" => Self::Z51,
            "Z52" => Self::Z52,
            "Z31" => Self::Z31,
            "Z39" => Self::Z39,
            "Z32" => Self::Z32,
            "Z18" => Self::Z18,
            "Z19" => Self::Z19,
            "Z37" => Self::Z37,
            "Z33" => Self::Z33,
            "Z34" => Self::Z34,
            "Z35" => Self::Z35,
            "Z16" => Self::Z16,
            "Z10" => Self::Z10,
            "Z20" => Self::Z20,
            "Z38" => Self::Z38,
            "Z05" => Self::Z05,
            "Z14" => Self::Z14,
            "MG" => Self::MG,
            "AGK" => Self::AGK,
            "Z12" => Self::Z12,
            "AVE" => Self::AVE,
            "Z46" => Self::Z46,
            "AVC" => Self::AVC,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D1154Qualifier {
    _55065,
    _55066,
    _55067,
    _55069,
    _55070,
    _55073,
    _55195,
    _55196,
    _55201,
    _55202,
    _55223,
    _55224,
    _55001,
    _55002,
    _55003,
    _55004,
    _55005,
    _55006,
    _55007,
    _55008,
    _55009,
    _55010,
    _55011,
    _55012,
    _55013,
    _55014,
    _55015,
    _55016,
    _55017,
    _55018,
    _55022,
    _55023,
    _55024,
    _55035,
    _55036,
    _55037,
    _55038,
    _55039,
    _55040,
    _55041,
    _55042,
    _55043,
    _55044,
    _55051,
    _55052,
    _55053,
    _55060,
    _55062,
    _55063,
    _55064,
    _55071,
    _55072,
    _55074,
    _55075,
    _55076,
    _55077,
    _55078,
    _55080,
    _55095,
    _55109,
    _55110,
    _55126,
    _55136,
    _55137,
    _55156,
    _55168,
    _55169,
    _55170,
    _55173,
    _55175,
    _55177,
    _55180,
    _55194,
    _55197,
    _55198,
    _55199,
    _55200,
    _55203,
    _55204,
    _55205,
    _55206,
    _55207,
    _55208,
    _55209,
    _55210,
    _55211,
    _55212,
    _55213,
    _55214,
    _55218,
    _55220,
    _55225,
    _55227,
    _55230,
    _55232,
    _55235,
    _55236,
    _55237,
    _55238,
    _55239,
    _55240,
    _55241,
    _55242,
    _55243,
    _55553,
    _55555,
    _55557,
    _55559,
    _55600,
    _55601,
    _55602,
    _55603,
    _55604,
    _55605,
    _55607,
    _55608,
    _55609,
    _55611,
    _55613,
    _55614,
    _55615,
    _55616,
    _55617,
    _55618,
    _55619,
    _55620,
    _55621,
    _55622,
    _55623,
    _55624,
    _55625,
    _55626,
    _55627,
    _55628,
    _55629,
    _55630,
    _55632,
    _55633,
    _55634,
    _55635,
    _55636,
    _55638,
    _55639,
    _55640,
    _55641,
    _55642,
    _55643,
    _55644,
    _55645,
    _55646,
    _55647,
    _55648,
    _55649,
    _55650,
    _55651,
    _55652,
    _55653,
    _55654,
    _55655,
    _55656,
    _55657,
    _55658,
    _55659,
    _55660,
    _55661,
    _55662,
    _55663,
    _55664,
    _55665,
    _55666,
    _55667,
    _55669,
    _55670,
    _55671,
    _55672,
    _55673,
    _55674,
    _55675,
    _55684,
    _55685,
    _55686,
    _55687,
    _55688,
    _55689,
    _55690,
    _55691,
    _55692,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D1154Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_55065 => write!(f, "55065"),
            Self::_55066 => write!(f, "55066"),
            Self::_55067 => write!(f, "55067"),
            Self::_55069 => write!(f, "55069"),
            Self::_55070 => write!(f, "55070"),
            Self::_55073 => write!(f, "55073"),
            Self::_55195 => write!(f, "55195"),
            Self::_55196 => write!(f, "55196"),
            Self::_55201 => write!(f, "55201"),
            Self::_55202 => write!(f, "55202"),
            Self::_55223 => write!(f, "55223"),
            Self::_55224 => write!(f, "55224"),
            Self::_55001 => write!(f, "55001"),
            Self::_55002 => write!(f, "55002"),
            Self::_55003 => write!(f, "55003"),
            Self::_55004 => write!(f, "55004"),
            Self::_55005 => write!(f, "55005"),
            Self::_55006 => write!(f, "55006"),
            Self::_55007 => write!(f, "55007"),
            Self::_55008 => write!(f, "55008"),
            Self::_55009 => write!(f, "55009"),
            Self::_55010 => write!(f, "55010"),
            Self::_55011 => write!(f, "55011"),
            Self::_55012 => write!(f, "55012"),
            Self::_55013 => write!(f, "55013"),
            Self::_55014 => write!(f, "55014"),
            Self::_55015 => write!(f, "55015"),
            Self::_55016 => write!(f, "55016"),
            Self::_55017 => write!(f, "55017"),
            Self::_55018 => write!(f, "55018"),
            Self::_55022 => write!(f, "55022"),
            Self::_55023 => write!(f, "55023"),
            Self::_55024 => write!(f, "55024"),
            Self::_55035 => write!(f, "55035"),
            Self::_55036 => write!(f, "55036"),
            Self::_55037 => write!(f, "55037"),
            Self::_55038 => write!(f, "55038"),
            Self::_55039 => write!(f, "55039"),
            Self::_55040 => write!(f, "55040"),
            Self::_55041 => write!(f, "55041"),
            Self::_55042 => write!(f, "55042"),
            Self::_55043 => write!(f, "55043"),
            Self::_55044 => write!(f, "55044"),
            Self::_55051 => write!(f, "55051"),
            Self::_55052 => write!(f, "55052"),
            Self::_55053 => write!(f, "55053"),
            Self::_55060 => write!(f, "55060"),
            Self::_55062 => write!(f, "55062"),
            Self::_55063 => write!(f, "55063"),
            Self::_55064 => write!(f, "55064"),
            Self::_55071 => write!(f, "55071"),
            Self::_55072 => write!(f, "55072"),
            Self::_55074 => write!(f, "55074"),
            Self::_55075 => write!(f, "55075"),
            Self::_55076 => write!(f, "55076"),
            Self::_55077 => write!(f, "55077"),
            Self::_55078 => write!(f, "55078"),
            Self::_55080 => write!(f, "55080"),
            Self::_55095 => write!(f, "55095"),
            Self::_55109 => write!(f, "55109"),
            Self::_55110 => write!(f, "55110"),
            Self::_55126 => write!(f, "55126"),
            Self::_55136 => write!(f, "55136"),
            Self::_55137 => write!(f, "55137"),
            Self::_55156 => write!(f, "55156"),
            Self::_55168 => write!(f, "55168"),
            Self::_55169 => write!(f, "55169"),
            Self::_55170 => write!(f, "55170"),
            Self::_55173 => write!(f, "55173"),
            Self::_55175 => write!(f, "55175"),
            Self::_55177 => write!(f, "55177"),
            Self::_55180 => write!(f, "55180"),
            Self::_55194 => write!(f, "55194"),
            Self::_55197 => write!(f, "55197"),
            Self::_55198 => write!(f, "55198"),
            Self::_55199 => write!(f, "55199"),
            Self::_55200 => write!(f, "55200"),
            Self::_55203 => write!(f, "55203"),
            Self::_55204 => write!(f, "55204"),
            Self::_55205 => write!(f, "55205"),
            Self::_55206 => write!(f, "55206"),
            Self::_55207 => write!(f, "55207"),
            Self::_55208 => write!(f, "55208"),
            Self::_55209 => write!(f, "55209"),
            Self::_55210 => write!(f, "55210"),
            Self::_55211 => write!(f, "55211"),
            Self::_55212 => write!(f, "55212"),
            Self::_55213 => write!(f, "55213"),
            Self::_55214 => write!(f, "55214"),
            Self::_55218 => write!(f, "55218"),
            Self::_55220 => write!(f, "55220"),
            Self::_55225 => write!(f, "55225"),
            Self::_55227 => write!(f, "55227"),
            Self::_55230 => write!(f, "55230"),
            Self::_55232 => write!(f, "55232"),
            Self::_55235 => write!(f, "55235"),
            Self::_55236 => write!(f, "55236"),
            Self::_55237 => write!(f, "55237"),
            Self::_55238 => write!(f, "55238"),
            Self::_55239 => write!(f, "55239"),
            Self::_55240 => write!(f, "55240"),
            Self::_55241 => write!(f, "55241"),
            Self::_55242 => write!(f, "55242"),
            Self::_55243 => write!(f, "55243"),
            Self::_55553 => write!(f, "55553"),
            Self::_55555 => write!(f, "55555"),
            Self::_55557 => write!(f, "55557"),
            Self::_55559 => write!(f, "55559"),
            Self::_55600 => write!(f, "55600"),
            Self::_55601 => write!(f, "55601"),
            Self::_55602 => write!(f, "55602"),
            Self::_55603 => write!(f, "55603"),
            Self::_55604 => write!(f, "55604"),
            Self::_55605 => write!(f, "55605"),
            Self::_55607 => write!(f, "55607"),
            Self::_55608 => write!(f, "55608"),
            Self::_55609 => write!(f, "55609"),
            Self::_55611 => write!(f, "55611"),
            Self::_55613 => write!(f, "55613"),
            Self::_55614 => write!(f, "55614"),
            Self::_55615 => write!(f, "55615"),
            Self::_55616 => write!(f, "55616"),
            Self::_55617 => write!(f, "55617"),
            Self::_55618 => write!(f, "55618"),
            Self::_55619 => write!(f, "55619"),
            Self::_55620 => write!(f, "55620"),
            Self::_55621 => write!(f, "55621"),
            Self::_55622 => write!(f, "55622"),
            Self::_55623 => write!(f, "55623"),
            Self::_55624 => write!(f, "55624"),
            Self::_55625 => write!(f, "55625"),
            Self::_55626 => write!(f, "55626"),
            Self::_55627 => write!(f, "55627"),
            Self::_55628 => write!(f, "55628"),
            Self::_55629 => write!(f, "55629"),
            Self::_55630 => write!(f, "55630"),
            Self::_55632 => write!(f, "55632"),
            Self::_55633 => write!(f, "55633"),
            Self::_55634 => write!(f, "55634"),
            Self::_55635 => write!(f, "55635"),
            Self::_55636 => write!(f, "55636"),
            Self::_55638 => write!(f, "55638"),
            Self::_55639 => write!(f, "55639"),
            Self::_55640 => write!(f, "55640"),
            Self::_55641 => write!(f, "55641"),
            Self::_55642 => write!(f, "55642"),
            Self::_55643 => write!(f, "55643"),
            Self::_55644 => write!(f, "55644"),
            Self::_55645 => write!(f, "55645"),
            Self::_55646 => write!(f, "55646"),
            Self::_55647 => write!(f, "55647"),
            Self::_55648 => write!(f, "55648"),
            Self::_55649 => write!(f, "55649"),
            Self::_55650 => write!(f, "55650"),
            Self::_55651 => write!(f, "55651"),
            Self::_55652 => write!(f, "55652"),
            Self::_55653 => write!(f, "55653"),
            Self::_55654 => write!(f, "55654"),
            Self::_55655 => write!(f, "55655"),
            Self::_55656 => write!(f, "55656"),
            Self::_55657 => write!(f, "55657"),
            Self::_55658 => write!(f, "55658"),
            Self::_55659 => write!(f, "55659"),
            Self::_55660 => write!(f, "55660"),
            Self::_55661 => write!(f, "55661"),
            Self::_55662 => write!(f, "55662"),
            Self::_55663 => write!(f, "55663"),
            Self::_55664 => write!(f, "55664"),
            Self::_55665 => write!(f, "55665"),
            Self::_55666 => write!(f, "55666"),
            Self::_55667 => write!(f, "55667"),
            Self::_55669 => write!(f, "55669"),
            Self::_55670 => write!(f, "55670"),
            Self::_55671 => write!(f, "55671"),
            Self::_55672 => write!(f, "55672"),
            Self::_55673 => write!(f, "55673"),
            Self::_55674 => write!(f, "55674"),
            Self::_55675 => write!(f, "55675"),
            Self::_55684 => write!(f, "55684"),
            Self::_55685 => write!(f, "55685"),
            Self::_55686 => write!(f, "55686"),
            Self::_55687 => write!(f, "55687"),
            Self::_55688 => write!(f, "55688"),
            Self::_55689 => write!(f, "55689"),
            Self::_55690 => write!(f, "55690"),
            Self::_55691 => write!(f, "55691"),
            Self::_55692 => write!(f, "55692"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D1154Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "55065" => Self::_55065,
            "55066" => Self::_55066,
            "55067" => Self::_55067,
            "55069" => Self::_55069,
            "55070" => Self::_55070,
            "55073" => Self::_55073,
            "55195" => Self::_55195,
            "55196" => Self::_55196,
            "55201" => Self::_55201,
            "55202" => Self::_55202,
            "55223" => Self::_55223,
            "55224" => Self::_55224,
            "55001" => Self::_55001,
            "55002" => Self::_55002,
            "55003" => Self::_55003,
            "55004" => Self::_55004,
            "55005" => Self::_55005,
            "55006" => Self::_55006,
            "55007" => Self::_55007,
            "55008" => Self::_55008,
            "55009" => Self::_55009,
            "55010" => Self::_55010,
            "55011" => Self::_55011,
            "55012" => Self::_55012,
            "55013" => Self::_55013,
            "55014" => Self::_55014,
            "55015" => Self::_55015,
            "55016" => Self::_55016,
            "55017" => Self::_55017,
            "55018" => Self::_55018,
            "55022" => Self::_55022,
            "55023" => Self::_55023,
            "55024" => Self::_55024,
            "55035" => Self::_55035,
            "55036" => Self::_55036,
            "55037" => Self::_55037,
            "55038" => Self::_55038,
            "55039" => Self::_55039,
            "55040" => Self::_55040,
            "55041" => Self::_55041,
            "55042" => Self::_55042,
            "55043" => Self::_55043,
            "55044" => Self::_55044,
            "55051" => Self::_55051,
            "55052" => Self::_55052,
            "55053" => Self::_55053,
            "55060" => Self::_55060,
            "55062" => Self::_55062,
            "55063" => Self::_55063,
            "55064" => Self::_55064,
            "55071" => Self::_55071,
            "55072" => Self::_55072,
            "55074" => Self::_55074,
            "55075" => Self::_55075,
            "55076" => Self::_55076,
            "55077" => Self::_55077,
            "55078" => Self::_55078,
            "55080" => Self::_55080,
            "55095" => Self::_55095,
            "55109" => Self::_55109,
            "55110" => Self::_55110,
            "55126" => Self::_55126,
            "55136" => Self::_55136,
            "55137" => Self::_55137,
            "55156" => Self::_55156,
            "55168" => Self::_55168,
            "55169" => Self::_55169,
            "55170" => Self::_55170,
            "55173" => Self::_55173,
            "55175" => Self::_55175,
            "55177" => Self::_55177,
            "55180" => Self::_55180,
            "55194" => Self::_55194,
            "55197" => Self::_55197,
            "55198" => Self::_55198,
            "55199" => Self::_55199,
            "55200" => Self::_55200,
            "55203" => Self::_55203,
            "55204" => Self::_55204,
            "55205" => Self::_55205,
            "55206" => Self::_55206,
            "55207" => Self::_55207,
            "55208" => Self::_55208,
            "55209" => Self::_55209,
            "55210" => Self::_55210,
            "55211" => Self::_55211,
            "55212" => Self::_55212,
            "55213" => Self::_55213,
            "55214" => Self::_55214,
            "55218" => Self::_55218,
            "55220" => Self::_55220,
            "55225" => Self::_55225,
            "55227" => Self::_55227,
            "55230" => Self::_55230,
            "55232" => Self::_55232,
            "55235" => Self::_55235,
            "55236" => Self::_55236,
            "55237" => Self::_55237,
            "55238" => Self::_55238,
            "55239" => Self::_55239,
            "55240" => Self::_55240,
            "55241" => Self::_55241,
            "55242" => Self::_55242,
            "55243" => Self::_55243,
            "55553" => Self::_55553,
            "55555" => Self::_55555,
            "55557" => Self::_55557,
            "55559" => Self::_55559,
            "55600" => Self::_55600,
            "55601" => Self::_55601,
            "55602" => Self::_55602,
            "55603" => Self::_55603,
            "55604" => Self::_55604,
            "55605" => Self::_55605,
            "55607" => Self::_55607,
            "55608" => Self::_55608,
            "55609" => Self::_55609,
            "55611" => Self::_55611,
            "55613" => Self::_55613,
            "55614" => Self::_55614,
            "55615" => Self::_55615,
            "55616" => Self::_55616,
            "55617" => Self::_55617,
            "55618" => Self::_55618,
            "55619" => Self::_55619,
            "55620" => Self::_55620,
            "55621" => Self::_55621,
            "55622" => Self::_55622,
            "55623" => Self::_55623,
            "55624" => Self::_55624,
            "55625" => Self::_55625,
            "55626" => Self::_55626,
            "55627" => Self::_55627,
            "55628" => Self::_55628,
            "55629" => Self::_55629,
            "55630" => Self::_55630,
            "55632" => Self::_55632,
            "55633" => Self::_55633,
            "55634" => Self::_55634,
            "55635" => Self::_55635,
            "55636" => Self::_55636,
            "55638" => Self::_55638,
            "55639" => Self::_55639,
            "55640" => Self::_55640,
            "55641" => Self::_55641,
            "55642" => Self::_55642,
            "55643" => Self::_55643,
            "55644" => Self::_55644,
            "55645" => Self::_55645,
            "55646" => Self::_55646,
            "55647" => Self::_55647,
            "55648" => Self::_55648,
            "55649" => Self::_55649,
            "55650" => Self::_55650,
            "55651" => Self::_55651,
            "55652" => Self::_55652,
            "55653" => Self::_55653,
            "55654" => Self::_55654,
            "55655" => Self::_55655,
            "55656" => Self::_55656,
            "55657" => Self::_55657,
            "55658" => Self::_55658,
            "55659" => Self::_55659,
            "55660" => Self::_55660,
            "55661" => Self::_55661,
            "55662" => Self::_55662,
            "55663" => Self::_55663,
            "55664" => Self::_55664,
            "55665" => Self::_55665,
            "55666" => Self::_55666,
            "55667" => Self::_55667,
            "55669" => Self::_55669,
            "55670" => Self::_55670,
            "55671" => Self::_55671,
            "55672" => Self::_55672,
            "55673" => Self::_55673,
            "55674" => Self::_55674,
            "55675" => Self::_55675,
            "55684" => Self::_55684,
            "55685" => Self::_55685,
            "55686" => Self::_55686,
            "55687" => Self::_55687,
            "55688" => Self::_55688,
            "55689" => Self::_55689,
            "55690" => Self::_55690,
            "55691" => Self::_55691,
            "55692" => Self::_55692,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D1229Qualifier {
    Z22,
    Z78,
    ZC7,
    ZC8,
    ZD5,
    Z58,
    ZC9,
    ZD0,
    ZD6,
    Z79,
    ZH0,
    Z51,
    ZA9,
    ZB0,
    ZD7,
    Z71,
    ZD8,
    ZH1,
    ZH2,
    Z57,
    ZA7,
    ZA8,
    ZD9,
    Z60,
    ZE0,
    ZG8,
    ZG9,
    Z01,
    Z80,
    Z81,
    Z98,
    Z29,
    Z45,
    Z82,
    Z83,
    Z84,
    Z96,
    Z97,
    ZE1,
    Z76,
    ZC5,
    ZC6,
    Z27,
    ZE2,
    Z02,
    ZA1,
    ZA2,
    ZE3,
    Z59,
    ZB5,
    ZB6,
    ZE4,
    Z44,
    ZD1,
    ZD2,
    ZE5,
    Z30,
    Z40,
    ZD3,
    ZD4,
    ZE6,
    Z15,
    Z94,
    Z95,
    ZE7,
    Z31,
    Z16,
    ZE8,
    Z17,
    Z99,
    ZA0,
    ZE9,
    Z32,
    Z52,
    ZF0,
    ZG4,
    ZG5,
    Z62,
    ZB1,
    ZB2,
    ZF1,
    Z61,
    ZB3,
    ZB4,
    ZF2,
    Z18,
    ZG6,
    ZG7,
    ZF3,
    Z19,
    ZF4,
    Z03,
    ZA3,
    ZA4,
    ZF5,
    Z20,
    ZA5,
    ZA6,
    ZF6,
    Z04,
    ZB9,
    ZC0,
    ZF7,
    Z05,
    ZB7,
    ZB8,
    ZF8,
    Z06,
    ZC1,
    ZC2,
    ZF9,
    Z13,
    ZC3,
    ZC4,
    ZG0,
    Z14,
    ZH3,
    ZH4,
    Z21,
    Z85,
    Z86,
    ZG1,
    Z33,
    Z08,
    Z87,
    Z88,
    ZG2,
    Z38,
    Z89,
    Z90,
    ZG3,
    Z23,
    Z24,
    Z25,
    Z47,
    Z72,
    Z48,
    Z49,
    Z75,
    Z92,
    Z93,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D1229Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z22 => write!(f, "Z22"),
            Self::Z78 => write!(f, "Z78"),
            Self::ZC7 => write!(f, "ZC7"),
            Self::ZC8 => write!(f, "ZC8"),
            Self::ZD5 => write!(f, "ZD5"),
            Self::Z58 => write!(f, "Z58"),
            Self::ZC9 => write!(f, "ZC9"),
            Self::ZD0 => write!(f, "ZD0"),
            Self::ZD6 => write!(f, "ZD6"),
            Self::Z79 => write!(f, "Z79"),
            Self::ZH0 => write!(f, "ZH0"),
            Self::Z51 => write!(f, "Z51"),
            Self::ZA9 => write!(f, "ZA9"),
            Self::ZB0 => write!(f, "ZB0"),
            Self::ZD7 => write!(f, "ZD7"),
            Self::Z71 => write!(f, "Z71"),
            Self::ZD8 => write!(f, "ZD8"),
            Self::ZH1 => write!(f, "ZH1"),
            Self::ZH2 => write!(f, "ZH2"),
            Self::Z57 => write!(f, "Z57"),
            Self::ZA7 => write!(f, "ZA7"),
            Self::ZA8 => write!(f, "ZA8"),
            Self::ZD9 => write!(f, "ZD9"),
            Self::Z60 => write!(f, "Z60"),
            Self::ZE0 => write!(f, "ZE0"),
            Self::ZG8 => write!(f, "ZG8"),
            Self::ZG9 => write!(f, "ZG9"),
            Self::Z01 => write!(f, "Z01"),
            Self::Z80 => write!(f, "Z80"),
            Self::Z81 => write!(f, "Z81"),
            Self::Z98 => write!(f, "Z98"),
            Self::Z29 => write!(f, "Z29"),
            Self::Z45 => write!(f, "Z45"),
            Self::Z82 => write!(f, "Z82"),
            Self::Z83 => write!(f, "Z83"),
            Self::Z84 => write!(f, "Z84"),
            Self::Z96 => write!(f, "Z96"),
            Self::Z97 => write!(f, "Z97"),
            Self::ZE1 => write!(f, "ZE1"),
            Self::Z76 => write!(f, "Z76"),
            Self::ZC5 => write!(f, "ZC5"),
            Self::ZC6 => write!(f, "ZC6"),
            Self::Z27 => write!(f, "Z27"),
            Self::ZE2 => write!(f, "ZE2"),
            Self::Z02 => write!(f, "Z02"),
            Self::ZA1 => write!(f, "ZA1"),
            Self::ZA2 => write!(f, "ZA2"),
            Self::ZE3 => write!(f, "ZE3"),
            Self::Z59 => write!(f, "Z59"),
            Self::ZB5 => write!(f, "ZB5"),
            Self::ZB6 => write!(f, "ZB6"),
            Self::ZE4 => write!(f, "ZE4"),
            Self::Z44 => write!(f, "Z44"),
            Self::ZD1 => write!(f, "ZD1"),
            Self::ZD2 => write!(f, "ZD2"),
            Self::ZE5 => write!(f, "ZE5"),
            Self::Z30 => write!(f, "Z30"),
            Self::Z40 => write!(f, "Z40"),
            Self::ZD3 => write!(f, "ZD3"),
            Self::ZD4 => write!(f, "ZD4"),
            Self::ZE6 => write!(f, "ZE6"),
            Self::Z15 => write!(f, "Z15"),
            Self::Z94 => write!(f, "Z94"),
            Self::Z95 => write!(f, "Z95"),
            Self::ZE7 => write!(f, "ZE7"),
            Self::Z31 => write!(f, "Z31"),
            Self::Z16 => write!(f, "Z16"),
            Self::ZE8 => write!(f, "ZE8"),
            Self::Z17 => write!(f, "Z17"),
            Self::Z99 => write!(f, "Z99"),
            Self::ZA0 => write!(f, "ZA0"),
            Self::ZE9 => write!(f, "ZE9"),
            Self::Z32 => write!(f, "Z32"),
            Self::Z52 => write!(f, "Z52"),
            Self::ZF0 => write!(f, "ZF0"),
            Self::ZG4 => write!(f, "ZG4"),
            Self::ZG5 => write!(f, "ZG5"),
            Self::Z62 => write!(f, "Z62"),
            Self::ZB1 => write!(f, "ZB1"),
            Self::ZB2 => write!(f, "ZB2"),
            Self::ZF1 => write!(f, "ZF1"),
            Self::Z61 => write!(f, "Z61"),
            Self::ZB3 => write!(f, "ZB3"),
            Self::ZB4 => write!(f, "ZB4"),
            Self::ZF2 => write!(f, "ZF2"),
            Self::Z18 => write!(f, "Z18"),
            Self::ZG6 => write!(f, "ZG6"),
            Self::ZG7 => write!(f, "ZG7"),
            Self::ZF3 => write!(f, "ZF3"),
            Self::Z19 => write!(f, "Z19"),
            Self::ZF4 => write!(f, "ZF4"),
            Self::Z03 => write!(f, "Z03"),
            Self::ZA3 => write!(f, "ZA3"),
            Self::ZA4 => write!(f, "ZA4"),
            Self::ZF5 => write!(f, "ZF5"),
            Self::Z20 => write!(f, "Z20"),
            Self::ZA5 => write!(f, "ZA5"),
            Self::ZA6 => write!(f, "ZA6"),
            Self::ZF6 => write!(f, "ZF6"),
            Self::Z04 => write!(f, "Z04"),
            Self::ZB9 => write!(f, "ZB9"),
            Self::ZC0 => write!(f, "ZC0"),
            Self::ZF7 => write!(f, "ZF7"),
            Self::Z05 => write!(f, "Z05"),
            Self::ZB7 => write!(f, "ZB7"),
            Self::ZB8 => write!(f, "ZB8"),
            Self::ZF8 => write!(f, "ZF8"),
            Self::Z06 => write!(f, "Z06"),
            Self::ZC1 => write!(f, "ZC1"),
            Self::ZC2 => write!(f, "ZC2"),
            Self::ZF9 => write!(f, "ZF9"),
            Self::Z13 => write!(f, "Z13"),
            Self::ZC3 => write!(f, "ZC3"),
            Self::ZC4 => write!(f, "ZC4"),
            Self::ZG0 => write!(f, "ZG0"),
            Self::Z14 => write!(f, "Z14"),
            Self::ZH3 => write!(f, "ZH3"),
            Self::ZH4 => write!(f, "ZH4"),
            Self::Z21 => write!(f, "Z21"),
            Self::Z85 => write!(f, "Z85"),
            Self::Z86 => write!(f, "Z86"),
            Self::ZG1 => write!(f, "ZG1"),
            Self::Z33 => write!(f, "Z33"),
            Self::Z08 => write!(f, "Z08"),
            Self::Z87 => write!(f, "Z87"),
            Self::Z88 => write!(f, "Z88"),
            Self::ZG2 => write!(f, "ZG2"),
            Self::Z38 => write!(f, "Z38"),
            Self::Z89 => write!(f, "Z89"),
            Self::Z90 => write!(f, "Z90"),
            Self::ZG3 => write!(f, "ZG3"),
            Self::Z23 => write!(f, "Z23"),
            Self::Z24 => write!(f, "Z24"),
            Self::Z25 => write!(f, "Z25"),
            Self::Z47 => write!(f, "Z47"),
            Self::Z72 => write!(f, "Z72"),
            Self::Z48 => write!(f, "Z48"),
            Self::Z49 => write!(f, "Z49"),
            Self::Z75 => write!(f, "Z75"),
            Self::Z92 => write!(f, "Z92"),
            Self::Z93 => write!(f, "Z93"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D1229Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z22" => Self::Z22,
            "Z78" => Self::Z78,
            "ZC7" => Self::ZC7,
            "ZC8" => Self::ZC8,
            "ZD5" => Self::ZD5,
            "Z58" => Self::Z58,
            "ZC9" => Self::ZC9,
            "ZD0" => Self::ZD0,
            "ZD6" => Self::ZD6,
            "Z79" => Self::Z79,
            "ZH0" => Self::ZH0,
            "Z51" => Self::Z51,
            "ZA9" => Self::ZA9,
            "ZB0" => Self::ZB0,
            "ZD7" => Self::ZD7,
            "Z71" => Self::Z71,
            "ZD8" => Self::ZD8,
            "ZH1" => Self::ZH1,
            "ZH2" => Self::ZH2,
            "Z57" => Self::Z57,
            "ZA7" => Self::ZA7,
            "ZA8" => Self::ZA8,
            "ZD9" => Self::ZD9,
            "Z60" => Self::Z60,
            "ZE0" => Self::ZE0,
            "ZG8" => Self::ZG8,
            "ZG9" => Self::ZG9,
            "Z01" => Self::Z01,
            "Z80" => Self::Z80,
            "Z81" => Self::Z81,
            "Z98" => Self::Z98,
            "Z29" => Self::Z29,
            "Z45" => Self::Z45,
            "Z82" => Self::Z82,
            "Z83" => Self::Z83,
            "Z84" => Self::Z84,
            "Z96" => Self::Z96,
            "Z97" => Self::Z97,
            "ZE1" => Self::ZE1,
            "Z76" => Self::Z76,
            "ZC5" => Self::ZC5,
            "ZC6" => Self::ZC6,
            "Z27" => Self::Z27,
            "ZE2" => Self::ZE2,
            "Z02" => Self::Z02,
            "ZA1" => Self::ZA1,
            "ZA2" => Self::ZA2,
            "ZE3" => Self::ZE3,
            "Z59" => Self::Z59,
            "ZB5" => Self::ZB5,
            "ZB6" => Self::ZB6,
            "ZE4" => Self::ZE4,
            "Z44" => Self::Z44,
            "ZD1" => Self::ZD1,
            "ZD2" => Self::ZD2,
            "ZE5" => Self::ZE5,
            "Z30" => Self::Z30,
            "Z40" => Self::Z40,
            "ZD3" => Self::ZD3,
            "ZD4" => Self::ZD4,
            "ZE6" => Self::ZE6,
            "Z15" => Self::Z15,
            "Z94" => Self::Z94,
            "Z95" => Self::Z95,
            "ZE7" => Self::ZE7,
            "Z31" => Self::Z31,
            "Z16" => Self::Z16,
            "ZE8" => Self::ZE8,
            "Z17" => Self::Z17,
            "Z99" => Self::Z99,
            "ZA0" => Self::ZA0,
            "ZE9" => Self::ZE9,
            "Z32" => Self::Z32,
            "Z52" => Self::Z52,
            "ZF0" => Self::ZF0,
            "ZG4" => Self::ZG4,
            "ZG5" => Self::ZG5,
            "Z62" => Self::Z62,
            "ZB1" => Self::ZB1,
            "ZB2" => Self::ZB2,
            "ZF1" => Self::ZF1,
            "Z61" => Self::Z61,
            "ZB3" => Self::ZB3,
            "ZB4" => Self::ZB4,
            "ZF2" => Self::ZF2,
            "Z18" => Self::Z18,
            "ZG6" => Self::ZG6,
            "ZG7" => Self::ZG7,
            "ZF3" => Self::ZF3,
            "Z19" => Self::Z19,
            "ZF4" => Self::ZF4,
            "Z03" => Self::Z03,
            "ZA3" => Self::ZA3,
            "ZA4" => Self::ZA4,
            "ZF5" => Self::ZF5,
            "Z20" => Self::Z20,
            "ZA5" => Self::ZA5,
            "ZA6" => Self::ZA6,
            "ZF6" => Self::ZF6,
            "Z04" => Self::Z04,
            "ZB9" => Self::ZB9,
            "ZC0" => Self::ZC0,
            "ZF7" => Self::ZF7,
            "Z05" => Self::Z05,
            "ZB7" => Self::ZB7,
            "ZB8" => Self::ZB8,
            "ZF8" => Self::ZF8,
            "Z06" => Self::Z06,
            "ZC1" => Self::ZC1,
            "ZC2" => Self::ZC2,
            "ZF9" => Self::ZF9,
            "Z13" => Self::Z13,
            "ZC3" => Self::ZC3,
            "ZC4" => Self::ZC4,
            "ZG0" => Self::ZG0,
            "Z14" => Self::Z14,
            "ZH3" => Self::ZH3,
            "ZH4" => Self::ZH4,
            "Z21" => Self::Z21,
            "Z85" => Self::Z85,
            "Z86" => Self::Z86,
            "ZG1" => Self::ZG1,
            "Z33" => Self::Z33,
            "Z08" => Self::Z08,
            "Z87" => Self::Z87,
            "Z88" => Self::Z88,
            "ZG2" => Self::ZG2,
            "Z38" => Self::Z38,
            "Z89" => Self::Z89,
            "Z90" => Self::Z90,
            "ZG3" => Self::ZG3,
            "Z23" => Self::Z23,
            "Z24" => Self::Z24,
            "Z25" => Self::Z25,
            "Z47" => Self::Z47,
            "Z72" => Self::Z72,
            "Z48" => Self::Z48,
            "Z49" => Self::Z49,
            "Z75" => Self::Z75,
            "Z92" => Self::Z92,
            "Z93" => Self::Z93,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D2005Qualifier {
    _137,
    _157,
    _76,
    _294,
    _92,
    _93,
    Z05,
    Z06,
    _471,
    _158,
    _159,
    _154,
    Z01,
    Z10,
    Z07,
    Z08,
    Z15,
    Z16,
    Z25,
    Z26,
    _752,
    Z21,
    Z09,
    Z22,
    _163,
    _164,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D2005Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_137 => write!(f, "137"),
            Self::_157 => write!(f, "157"),
            Self::_76 => write!(f, "76"),
            Self::_294 => write!(f, "294"),
            Self::_92 => write!(f, "92"),
            Self::_93 => write!(f, "93"),
            Self::Z05 => write!(f, "Z05"),
            Self::Z06 => write!(f, "Z06"),
            Self::_471 => write!(f, "471"),
            Self::_158 => write!(f, "158"),
            Self::_159 => write!(f, "159"),
            Self::_154 => write!(f, "154"),
            Self::Z01 => write!(f, "Z01"),
            Self::Z10 => write!(f, "Z10"),
            Self::Z07 => write!(f, "Z07"),
            Self::Z08 => write!(f, "Z08"),
            Self::Z15 => write!(f, "Z15"),
            Self::Z16 => write!(f, "Z16"),
            Self::Z25 => write!(f, "Z25"),
            Self::Z26 => write!(f, "Z26"),
            Self::_752 => write!(f, "752"),
            Self::Z21 => write!(f, "Z21"),
            Self::Z09 => write!(f, "Z09"),
            Self::Z22 => write!(f, "Z22"),
            Self::_163 => write!(f, "163"),
            Self::_164 => write!(f, "164"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D2005Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "137" => Self::_137,
            "157" => Self::_157,
            "76" => Self::_76,
            "294" => Self::_294,
            "92" => Self::_92,
            "93" => Self::_93,
            "Z05" => Self::Z05,
            "Z06" => Self::Z06,
            "471" => Self::_471,
            "158" => Self::_158,
            "159" => Self::_159,
            "154" => Self::_154,
            "Z01" => Self::Z01,
            "Z10" => Self::Z10,
            "Z07" => Self::Z07,
            "Z08" => Self::Z08,
            "Z15" => Self::Z15,
            "Z16" => Self::Z16,
            "Z25" => Self::Z25,
            "Z26" => Self::Z26,
            "752" => Self::_752,
            "Z21" => Self::Z21,
            "Z09" => Self::Z09,
            "Z22" => Self::Z22,
            "163" => Self::_163,
            "164" => Self::_164,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D2379Qualifier {
    _303,
    _610,
    _102,
    Z01,
    _106,
    _104,
    _602,
    _802,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D2379Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_303 => write!(f, "303"),
            Self::_610 => write!(f, "610"),
            Self::_102 => write!(f, "102"),
            Self::Z01 => write!(f, "Z01"),
            Self::_106 => write!(f, "106"),
            Self::_104 => write!(f, "104"),
            Self::_602 => write!(f, "602"),
            Self::_802 => write!(f, "802"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D2379Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "303" => Self::_303,
            "610" => Self::_610,
            "102" => Self::_102,
            "Z01" => Self::Z01,
            "106" => Self::_106,
            "104" => Self::_104,
            "602" => Self::_602,
            "802" => Self::_802,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D3035Qualifier {
    MS,
    MR,
    Z09,
    Z47,
    Z48,
    Z65,
    Z04,
    Z49,
    Z50,
    Z66,
    Z07,
    Z39,
    Z40,
    Z08,
    Z41,
    Z42,
    Z25,
    Z51,
    Z52,
    Z67,
    Z26,
    Z53,
    Z54,
    Z68,
    EO,
    Z55,
    Z56,
    Z69,
    DDO,
    Z57,
    Z58,
    Z70,
    VY,
    DP,
    Z59,
    Z60,
    Z63,
    Z03,
    Z43,
    Z44,
    Z64,
    Z05,
    Z45,
    Z46,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D3035Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MS => write!(f, "MS"),
            Self::MR => write!(f, "MR"),
            Self::Z09 => write!(f, "Z09"),
            Self::Z47 => write!(f, "Z47"),
            Self::Z48 => write!(f, "Z48"),
            Self::Z65 => write!(f, "Z65"),
            Self::Z04 => write!(f, "Z04"),
            Self::Z49 => write!(f, "Z49"),
            Self::Z50 => write!(f, "Z50"),
            Self::Z66 => write!(f, "Z66"),
            Self::Z07 => write!(f, "Z07"),
            Self::Z39 => write!(f, "Z39"),
            Self::Z40 => write!(f, "Z40"),
            Self::Z08 => write!(f, "Z08"),
            Self::Z41 => write!(f, "Z41"),
            Self::Z42 => write!(f, "Z42"),
            Self::Z25 => write!(f, "Z25"),
            Self::Z51 => write!(f, "Z51"),
            Self::Z52 => write!(f, "Z52"),
            Self::Z67 => write!(f, "Z67"),
            Self::Z26 => write!(f, "Z26"),
            Self::Z53 => write!(f, "Z53"),
            Self::Z54 => write!(f, "Z54"),
            Self::Z68 => write!(f, "Z68"),
            Self::EO => write!(f, "EO"),
            Self::Z55 => write!(f, "Z55"),
            Self::Z56 => write!(f, "Z56"),
            Self::Z69 => write!(f, "Z69"),
            Self::DDO => write!(f, "DDO"),
            Self::Z57 => write!(f, "Z57"),
            Self::Z58 => write!(f, "Z58"),
            Self::Z70 => write!(f, "Z70"),
            Self::VY => write!(f, "VY"),
            Self::DP => write!(f, "DP"),
            Self::Z59 => write!(f, "Z59"),
            Self::Z60 => write!(f, "Z60"),
            Self::Z63 => write!(f, "Z63"),
            Self::Z03 => write!(f, "Z03"),
            Self::Z43 => write!(f, "Z43"),
            Self::Z44 => write!(f, "Z44"),
            Self::Z64 => write!(f, "Z64"),
            Self::Z05 => write!(f, "Z05"),
            Self::Z45 => write!(f, "Z45"),
            Self::Z46 => write!(f, "Z46"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D3035Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "MS" => Self::MS,
            "MR" => Self::MR,
            "Z09" => Self::Z09,
            "Z47" => Self::Z47,
            "Z48" => Self::Z48,
            "Z65" => Self::Z65,
            "Z04" => Self::Z04,
            "Z49" => Self::Z49,
            "Z50" => Self::Z50,
            "Z66" => Self::Z66,
            "Z07" => Self::Z07,
            "Z39" => Self::Z39,
            "Z40" => Self::Z40,
            "Z08" => Self::Z08,
            "Z41" => Self::Z41,
            "Z42" => Self::Z42,
            "Z25" => Self::Z25,
            "Z51" => Self::Z51,
            "Z52" => Self::Z52,
            "Z67" => Self::Z67,
            "Z26" => Self::Z26,
            "Z53" => Self::Z53,
            "Z54" => Self::Z54,
            "Z68" => Self::Z68,
            "EO" => Self::EO,
            "Z55" => Self::Z55,
            "Z56" => Self::Z56,
            "Z69" => Self::Z69,
            "DDO" => Self::DDO,
            "Z57" => Self::Z57,
            "Z58" => Self::Z58,
            "Z70" => Self::Z70,
            "VY" => Self::VY,
            "DP" => Self::DP,
            "Z59" => Self::Z59,
            "Z60" => Self::Z60,
            "Z63" => Self::Z63,
            "Z03" => Self::Z03,
            "Z43" => Self::Z43,
            "Z44" => Self::Z44,
            "Z64" => Self::Z64,
            "Z05" => Self::Z05,
            "Z45" => Self::Z45,
            "Z46" => Self::Z46,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D3045Qualifier {
    Z01,
    Z02,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D3045Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z01 => write!(f, "Z01"),
            Self::Z02 => write!(f, "Z02"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D3045Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z01" => Self::Z01,
            "Z02" => Self::Z02,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D3055Qualifier {
    _9,
    _293,
    _332,
    _89,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D3055Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_9 => write!(f, "9"),
            Self::_293 => write!(f, "293"),
            Self::_332 => write!(f, "332"),
            Self::_89 => write!(f, "89"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D3055Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "9" => Self::_9,
            "293" => Self::_293,
            "332" => Self::_332,
            "89" => Self::_89,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D3139Qualifier {
    IC,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D3139Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IC => write!(f, "IC"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D3139Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "IC" => Self::IC,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D3155Qualifier {
    EM,
    FX,
    TE,
    AJ,
    AL,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D3155Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EM => write!(f, "EM"),
            Self::FX => write!(f, "FX"),
            Self::TE => write!(f, "TE"),
            Self::AJ => write!(f, "AJ"),
            Self::AL => write!(f, "AL"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D3155Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "EM" => Self::EM,
            "FX" => Self::FX,
            "TE" => Self::TE,
            "AJ" => Self::AJ,
            "AL" => Self::AL,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D3227Qualifier {
    Z15,
    Z18,
    Z16,
    Z22,
    Z20,
    Z19,
    Z21,
    Z17,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D3227Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z15 => write!(f, "Z15"),
            Self::Z18 => write!(f, "Z18"),
            Self::Z16 => write!(f, "Z16"),
            Self::Z22 => write!(f, "Z22"),
            Self::Z20 => write!(f, "Z20"),
            Self::Z19 => write!(f, "Z19"),
            Self::Z21 => write!(f, "Z21"),
            Self::Z17 => write!(f, "Z17"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D3227Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z15" => Self::Z15,
            "Z18" => Self::Z18,
            "Z16" => Self::Z16,
            "Z22" => Self::Z22,
            "Z20" => Self::Z20,
            "Z19" => Self::Z19,
            "Z21" => Self::Z21,
            "Z17" => Self::Z17,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D4051Qualifier {
    /// Der NB darf den LF der Marktlokation bzw. Tranche nur dann mit diesem Produktpaket zuordnen, wenn alle Produkte des Produktpakets zum Zuordnungsbeginn zur Anwendung kommen.
    Z01,
    /// Der NB darf den LF der Marktlokation bzw. Tranche dann mit diesem Produktpaket zuordnen, wenn alle oder auch nur ein Teil der Produkte des Produktpakets zum Zuordnungsbeginn zur Anwendung kommen.
    Z02,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D4051Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z01 => write!(f, "Z01"),
            Self::Z02 => write!(f, "Z02"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D4051Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z01" => Self::Z01,
            "Z02" => Self::Z02,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D4347Qualifier {
    _5,
    Z02,
    Z03,
    Z01,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D4347Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_5 => write!(f, "5"),
            Self::Z02 => write!(f, "Z02"),
            Self::Z03 => write!(f, "Z03"),
            Self::Z01 => write!(f, "Z01"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D4347Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "5" => Self::_5,
            "Z02" => Self::Z02,
            "Z03" => Self::Z03,
            "Z01" => Self::Z01,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D4451Qualifier {
    ACB,
    Z01,
    Z17,
    Z27,
    Z28,
    Z24,
    Z23,
    Z18,
    Z13,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D4451Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ACB => write!(f, "ACB"),
            Self::Z01 => write!(f, "Z01"),
            Self::Z17 => write!(f, "Z17"),
            Self::Z27 => write!(f, "Z27"),
            Self::Z28 => write!(f, "Z28"),
            Self::Z24 => write!(f, "Z24"),
            Self::Z23 => write!(f, "Z23"),
            Self::Z18 => write!(f, "Z18"),
            Self::Z13 => write!(f, "Z13"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D4451Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "ACB" => Self::ACB,
            "Z01" => Self::Z01,
            "Z17" => Self::Z17,
            "Z27" => Self::Z27,
            "Z28" => Self::Z28,
            "Z24" => Self::Z24,
            "Z23" => Self::Z23,
            "Z18" => Self::Z18,
            "Z13" => Self::Z13,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D6063Qualifier {
    Z10,
    _265,
    Z08,
    _31,
    Z07,
    Z32,
    Z09,
    Z38,
    Z16,
    Z34,
    Z35,
    Z37,
    Z33,
    Z46,
    _11,
    Z43,
    Z44,
    Z42,
    Z11,
    Z12,
    Z13,
    Z14,
    Z15,
    _79,
    Z17,
    Z36,
    Z45,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D6063Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z10 => write!(f, "Z10"),
            Self::_265 => write!(f, "265"),
            Self::Z08 => write!(f, "Z08"),
            Self::_31 => write!(f, "31"),
            Self::Z07 => write!(f, "Z07"),
            Self::Z32 => write!(f, "Z32"),
            Self::Z09 => write!(f, "Z09"),
            Self::Z38 => write!(f, "Z38"),
            Self::Z16 => write!(f, "Z16"),
            Self::Z34 => write!(f, "Z34"),
            Self::Z35 => write!(f, "Z35"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z33 => write!(f, "Z33"),
            Self::Z46 => write!(f, "Z46"),
            Self::_11 => write!(f, "11"),
            Self::Z43 => write!(f, "Z43"),
            Self::Z44 => write!(f, "Z44"),
            Self::Z42 => write!(f, "Z42"),
            Self::Z11 => write!(f, "Z11"),
            Self::Z12 => write!(f, "Z12"),
            Self::Z13 => write!(f, "Z13"),
            Self::Z14 => write!(f, "Z14"),
            Self::Z15 => write!(f, "Z15"),
            Self::_79 => write!(f, "79"),
            Self::Z17 => write!(f, "Z17"),
            Self::Z36 => write!(f, "Z36"),
            Self::Z45 => write!(f, "Z45"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D6063Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z10" => Self::Z10,
            "265" => Self::_265,
            "Z08" => Self::Z08,
            "31" => Self::_31,
            "Z07" => Self::Z07,
            "Z32" => Self::Z32,
            "Z09" => Self::Z09,
            "Z38" => Self::Z38,
            "Z16" => Self::Z16,
            "Z34" => Self::Z34,
            "Z35" => Self::Z35,
            "Z37" => Self::Z37,
            "Z33" => Self::Z33,
            "Z46" => Self::Z46,
            "11" => Self::_11,
            "Z43" => Self::Z43,
            "Z44" => Self::Z44,
            "Z42" => Self::Z42,
            "Z11" => Self::Z11,
            "Z12" => Self::Z12,
            "Z13" => Self::Z13,
            "Z14" => Self::Z14,
            "Z15" => Self::Z15,
            "79" => Self::_79,
            "Z17" => Self::Z17,
            "Z36" => Self::Z36,
            "Z45" => Self::Z45,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D6411Qualifier {
    Z16,
    KWH,
    KWT,
    H87,
    P1,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D6411Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z16 => write!(f, "Z16"),
            Self::KWH => write!(f, "KWH"),
            Self::KWT => write!(f, "KWT"),
            Self::H87 => write!(f, "H87"),
            Self::P1 => write!(f, "P1"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D6411Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z16" => Self::Z16,
            "KWH" => Self::KWH,
            "KWT" => Self::KWT,
            "H87" => Self::H87,
            "P1" => Self::P1,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7037Qualifier {
    /// Der Code ZF2 wird verwendet wenn kein Steuerkanal der Netzlokation zugeordnet ist und somit die Netzlokation nicht gesteuert werden kann.
    ZF2,
    /// Der Code ZF3 wird verwendet wenn ein Steuerkanal der Netzlokation zugeordnet ist und somit die Netzlokation gesteuert werden kann.
    ZF3,
    ZB3,
    /// Der NB rechnet die Blindarbeit an der genannten Netzlokation ab
    ZD9,
    /// Der NB rechnet die Blindarbeit an der genannten Netzlokation nicht ab. Dieser Zustand wird vom LF immer an einer Netzlokation angenommen bis eine Stammdatenänderung vom NB auf ZD9 "Abrechnung findet statt" erfolgt.
    ZE0,
    /// Der LF teilt hiermit die Bereitschaft zur Zahlung der Blindarbeit mit. Voraussetzung für eine tatsächliche Abrechnung gegenüber dem LF ist: Der NB hat über die Stammdatenprozessse dem LF mitgeteilt, dass er die Blindarbeit an der Netzlokation abrechnet und der NB hat die Abrechung der Blindarbeit auf den LF mit der entsprechenden SDÄ umstellt.
    ZE1,
    /// Der LF teilt hiermit keine Bereitschaft zur Zahlung der Blindarbeit mit.
    ZE2,
    ZA8,
    ZA7,
    ZF6,
    /// Marktlokation liefert Energie ins Netz. --> Erzeugende Marktlokation
    Z06,
    /// Marktlokation entnimmt Energie aus dem Netz. --> Verbrauchende Marktlokation
    Z07,
    E17,
    Z21,
    E03,
    Z83,
    ZA9,
    ZC0,
    ZA6,
    Z34,
    Z15,
    Z18,
    Z88,
    Z89,
    /// Veräußerungsform nach § 20 Abs. 1 Nr. 3 EEG 2014 („Einspeisevergütung nach § 37“) bzw. § 21 Abs. 2 EEG 2017 oder § 20 Abs. 1 Nr. 4 EEG 2014 („Einspeisevergütung nach § 38“) bzw. § 21b Abs. 1 Nr. 2 EEG 2017 (Ausfallvergütung)
    Z90,
    /// Veräußerungsform nach § 20 Abs. 1 Nr. 1 EEG 2014 („Geförderte Direktvermarktung") bzw. § 21 Abs. 1 Nr. 1 EEG 2017 (Marktprämie)
    Z91,
    /// Veräußerungsform ohne gesetzliche Vergütung (z.B. nach § 20 Abs. 1 Nr. 2 EEG 2014 („Sonstige Direktvermarktung") bzw. § 21b Abs. 1 Nr. 3 EEG 2017 ...)
    Z92,
    Z94,
    /// Der NB bestätigt mit der Anmeldung einer erzeugenden Marktlokation zur Direktvermarktung, dass die Anlage nicht mit einer Fernsteuerung ausgestattet ist und nicht fernsteuerbar ist. Die Voraussetzung zur Zahlung der Managementprämie für fernsteuerbare Anlagen ist nicht gegeben.
    Z96,
    /// Der NB bestätigt mit der Anmeldung einer erzeugenden Marktlokation zur Direktvermarktung, dass die Marktlokation mit einer Fernsteuerung ausgestattet, aber dem NB keine Information darüber vorliegt, dass der LF die Marktlokation fernsteuern kann. Die Voraussetzung zur Zahlung der Managementprämie für fernsteuerbare Marktlokation ist nicht gegeben.
    Z97,
    /// Der NB bestätigt mit der Anmeldung einer Marktlokation zur Direktvermarktung, dass die Marktlokation mit einer Fernsteuerung ausgestattet ist und der LF diese auch fernsteuern kann. Die Voraussetzung zur Zahlung der Managementprämie für fernsteuerbare Marktlokationen ist gegeben.
    Z98,
    ZC9,
    ZD0,
    ZE3,
    ZB9,
    ZD3,
    ZE9,
    ZF0,
    ZD8,
    ZE4,
    ZD1,
    ZD2,
    E04,
    Z82,
    Z81,
    ZC1,
    ZC2,
    ZC3,
    E13,
    Z28,
    E12,
    Z33,
    Z63,
    Z59,
    Z60,
    Z25,
    Z26,
    Z27,
    Z75,
    Z76,
    Z38,
    Z39,
    Z40,
    Z41,
    Z42,
    Z43,
    Z44,
    Z45,
    Z46,
    Z47,
    Z48,
    Z49,
    Z50,
    Z51,
    Z52,
    Z53,
    Z54,
    Z55,
    Z56,
    ZC6,
    ZC7,
    ZC8,
    E01,
    Z10,
    Z35,
    Z37,
    Z12,
    Z36,
    Z61,
    Z62,
    ZB4,
    ZB7,
    ZB6,
    ZB5,
    Z01,
    Z03,
    Z04,
    ZA5,
    /// Die Werte, die mit der OBIS Kennzahl des ZP der NGZ übermittelt werden, werden den Werten der selben OBIS Kennzahl der NZR zugerechnet
    ZE7,
    /// Die Werte, die mit der OBIS Kennzahl des ZP der NGZ übermittelt werden, werden den Werten der OBIS Kennzahl der NZR mit entgegengesetzter Lieferrichtung zugerechnet
    ZE8,
    /// Der Code ist anzuwenden, wenn der Lieferant eine Verringerung der Umlage nach EnFG für den Kunden an der genannten Marktlokation erwartet, da der Kunde die Voraussetzung nach EnFG erfüllt.
    ZF9,
    /// Der Code ist anzuwenden, wenn der Lieferant keine Verringerung der Umlage nach EnFG für den Kunden an der genannten Marktlokation erwartet, da der Kunde die Voraussetzung nach EnFG nicht erfüllt.
    ZG0,
    /// Der Code ist anzuwenden, wenn der Lieferant eine Verringerung der Umlage nach EnFG für den Kunden an der genannten Marktlokation nicht erwartet, da die Marktlokation die Voraussetzung nach EnFG nicht erfüllt.
    ZG1,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7037Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZF2 => write!(f, "ZF2"),
            Self::ZF3 => write!(f, "ZF3"),
            Self::ZB3 => write!(f, "ZB3"),
            Self::ZD9 => write!(f, "ZD9"),
            Self::ZE0 => write!(f, "ZE0"),
            Self::ZE1 => write!(f, "ZE1"),
            Self::ZE2 => write!(f, "ZE2"),
            Self::ZA8 => write!(f, "ZA8"),
            Self::ZA7 => write!(f, "ZA7"),
            Self::ZF6 => write!(f, "ZF6"),
            Self::Z06 => write!(f, "Z06"),
            Self::Z07 => write!(f, "Z07"),
            Self::E17 => write!(f, "E17"),
            Self::Z21 => write!(f, "Z21"),
            Self::E03 => write!(f, "E03"),
            Self::Z83 => write!(f, "Z83"),
            Self::ZA9 => write!(f, "ZA9"),
            Self::ZC0 => write!(f, "ZC0"),
            Self::ZA6 => write!(f, "ZA6"),
            Self::Z34 => write!(f, "Z34"),
            Self::Z15 => write!(f, "Z15"),
            Self::Z18 => write!(f, "Z18"),
            Self::Z88 => write!(f, "Z88"),
            Self::Z89 => write!(f, "Z89"),
            Self::Z90 => write!(f, "Z90"),
            Self::Z91 => write!(f, "Z91"),
            Self::Z92 => write!(f, "Z92"),
            Self::Z94 => write!(f, "Z94"),
            Self::Z96 => write!(f, "Z96"),
            Self::Z97 => write!(f, "Z97"),
            Self::Z98 => write!(f, "Z98"),
            Self::ZC9 => write!(f, "ZC9"),
            Self::ZD0 => write!(f, "ZD0"),
            Self::ZE3 => write!(f, "ZE3"),
            Self::ZB9 => write!(f, "ZB9"),
            Self::ZD3 => write!(f, "ZD3"),
            Self::ZE9 => write!(f, "ZE9"),
            Self::ZF0 => write!(f, "ZF0"),
            Self::ZD8 => write!(f, "ZD8"),
            Self::ZE4 => write!(f, "ZE4"),
            Self::ZD1 => write!(f, "ZD1"),
            Self::ZD2 => write!(f, "ZD2"),
            Self::E04 => write!(f, "E04"),
            Self::Z82 => write!(f, "Z82"),
            Self::Z81 => write!(f, "Z81"),
            Self::ZC1 => write!(f, "ZC1"),
            Self::ZC2 => write!(f, "ZC2"),
            Self::ZC3 => write!(f, "ZC3"),
            Self::E13 => write!(f, "E13"),
            Self::Z28 => write!(f, "Z28"),
            Self::E12 => write!(f, "E12"),
            Self::Z33 => write!(f, "Z33"),
            Self::Z63 => write!(f, "Z63"),
            Self::Z59 => write!(f, "Z59"),
            Self::Z60 => write!(f, "Z60"),
            Self::Z25 => write!(f, "Z25"),
            Self::Z26 => write!(f, "Z26"),
            Self::Z27 => write!(f, "Z27"),
            Self::Z75 => write!(f, "Z75"),
            Self::Z76 => write!(f, "Z76"),
            Self::Z38 => write!(f, "Z38"),
            Self::Z39 => write!(f, "Z39"),
            Self::Z40 => write!(f, "Z40"),
            Self::Z41 => write!(f, "Z41"),
            Self::Z42 => write!(f, "Z42"),
            Self::Z43 => write!(f, "Z43"),
            Self::Z44 => write!(f, "Z44"),
            Self::Z45 => write!(f, "Z45"),
            Self::Z46 => write!(f, "Z46"),
            Self::Z47 => write!(f, "Z47"),
            Self::Z48 => write!(f, "Z48"),
            Self::Z49 => write!(f, "Z49"),
            Self::Z50 => write!(f, "Z50"),
            Self::Z51 => write!(f, "Z51"),
            Self::Z52 => write!(f, "Z52"),
            Self::Z53 => write!(f, "Z53"),
            Self::Z54 => write!(f, "Z54"),
            Self::Z55 => write!(f, "Z55"),
            Self::Z56 => write!(f, "Z56"),
            Self::ZC6 => write!(f, "ZC6"),
            Self::ZC7 => write!(f, "ZC7"),
            Self::ZC8 => write!(f, "ZC8"),
            Self::E01 => write!(f, "E01"),
            Self::Z10 => write!(f, "Z10"),
            Self::Z35 => write!(f, "Z35"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z12 => write!(f, "Z12"),
            Self::Z36 => write!(f, "Z36"),
            Self::Z61 => write!(f, "Z61"),
            Self::Z62 => write!(f, "Z62"),
            Self::ZB4 => write!(f, "ZB4"),
            Self::ZB7 => write!(f, "ZB7"),
            Self::ZB6 => write!(f, "ZB6"),
            Self::ZB5 => write!(f, "ZB5"),
            Self::Z01 => write!(f, "Z01"),
            Self::Z03 => write!(f, "Z03"),
            Self::Z04 => write!(f, "Z04"),
            Self::ZA5 => write!(f, "ZA5"),
            Self::ZE7 => write!(f, "ZE7"),
            Self::ZE8 => write!(f, "ZE8"),
            Self::ZF9 => write!(f, "ZF9"),
            Self::ZG0 => write!(f, "ZG0"),
            Self::ZG1 => write!(f, "ZG1"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7037Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "ZF2" => Self::ZF2,
            "ZF3" => Self::ZF3,
            "ZB3" => Self::ZB3,
            "ZD9" => Self::ZD9,
            "ZE0" => Self::ZE0,
            "ZE1" => Self::ZE1,
            "ZE2" => Self::ZE2,
            "ZA8" => Self::ZA8,
            "ZA7" => Self::ZA7,
            "ZF6" => Self::ZF6,
            "Z06" => Self::Z06,
            "Z07" => Self::Z07,
            "E17" => Self::E17,
            "Z21" => Self::Z21,
            "E03" => Self::E03,
            "Z83" => Self::Z83,
            "ZA9" => Self::ZA9,
            "ZC0" => Self::ZC0,
            "ZA6" => Self::ZA6,
            "Z34" => Self::Z34,
            "Z15" => Self::Z15,
            "Z18" => Self::Z18,
            "Z88" => Self::Z88,
            "Z89" => Self::Z89,
            "Z90" => Self::Z90,
            "Z91" => Self::Z91,
            "Z92" => Self::Z92,
            "Z94" => Self::Z94,
            "Z96" => Self::Z96,
            "Z97" => Self::Z97,
            "Z98" => Self::Z98,
            "ZC9" => Self::ZC9,
            "ZD0" => Self::ZD0,
            "ZE3" => Self::ZE3,
            "ZB9" => Self::ZB9,
            "ZD3" => Self::ZD3,
            "ZE9" => Self::ZE9,
            "ZF0" => Self::ZF0,
            "ZD8" => Self::ZD8,
            "ZE4" => Self::ZE4,
            "ZD1" => Self::ZD1,
            "ZD2" => Self::ZD2,
            "E04" => Self::E04,
            "Z82" => Self::Z82,
            "Z81" => Self::Z81,
            "ZC1" => Self::ZC1,
            "ZC2" => Self::ZC2,
            "ZC3" => Self::ZC3,
            "E13" => Self::E13,
            "Z28" => Self::Z28,
            "E12" => Self::E12,
            "Z33" => Self::Z33,
            "Z63" => Self::Z63,
            "Z59" => Self::Z59,
            "Z60" => Self::Z60,
            "Z25" => Self::Z25,
            "Z26" => Self::Z26,
            "Z27" => Self::Z27,
            "Z75" => Self::Z75,
            "Z76" => Self::Z76,
            "Z38" => Self::Z38,
            "Z39" => Self::Z39,
            "Z40" => Self::Z40,
            "Z41" => Self::Z41,
            "Z42" => Self::Z42,
            "Z43" => Self::Z43,
            "Z44" => Self::Z44,
            "Z45" => Self::Z45,
            "Z46" => Self::Z46,
            "Z47" => Self::Z47,
            "Z48" => Self::Z48,
            "Z49" => Self::Z49,
            "Z50" => Self::Z50,
            "Z51" => Self::Z51,
            "Z52" => Self::Z52,
            "Z53" => Self::Z53,
            "Z54" => Self::Z54,
            "Z55" => Self::Z55,
            "Z56" => Self::Z56,
            "ZC6" => Self::ZC6,
            "ZC7" => Self::ZC7,
            "ZC8" => Self::ZC8,
            "E01" => Self::E01,
            "Z10" => Self::Z10,
            "Z35" => Self::Z35,
            "Z37" => Self::Z37,
            "Z12" => Self::Z12,
            "Z36" => Self::Z36,
            "Z61" => Self::Z61,
            "Z62" => Self::Z62,
            "ZB4" => Self::ZB4,
            "ZB7" => Self::ZB7,
            "ZB6" => Self::ZB6,
            "ZB5" => Self::ZB5,
            "Z01" => Self::Z01,
            "Z03" => Self::Z03,
            "Z04" => Self::Z04,
            "ZA5" => Self::ZA5,
            "ZE7" => Self::ZE7,
            "ZE8" => Self::ZE8,
            "ZF9" => Self::ZF9,
            "ZG0" => Self::ZG0,
            "ZG1" => Self::ZG1,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7059Qualifier {
    Z19,
    Z66,
    Z65,
    Z49,
    Z45,
    Z46,
    _11,
    Z53,
    Z30,
    Z18,
    Z20,
    _15,
    _6,
    Z31,
    Z22,
    Z23,
    Z24,
    Z36,
    Z42,
    ZA2,
    Z67,
    Z44,
    Z39,
    Z38,
    Z41,
    Z17,
    Z37,
    Z50,
    Z56,
    Z63,
    Z52,
    Z01,
    Z32,
    Z35,
    Z10,
    Z07,
    Z02,
    Z04,
    Z06,
    Z03,
    Z05,
    Z11,
    Z99,
    ZA0,
    Z25,
    Z48,
    Z28,
    Z29,
    Z61,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7059Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z19 => write!(f, "Z19"),
            Self::Z66 => write!(f, "Z66"),
            Self::Z65 => write!(f, "Z65"),
            Self::Z49 => write!(f, "Z49"),
            Self::Z45 => write!(f, "Z45"),
            Self::Z46 => write!(f, "Z46"),
            Self::_11 => write!(f, "11"),
            Self::Z53 => write!(f, "Z53"),
            Self::Z30 => write!(f, "Z30"),
            Self::Z18 => write!(f, "Z18"),
            Self::Z20 => write!(f, "Z20"),
            Self::_15 => write!(f, "15"),
            Self::_6 => write!(f, "6"),
            Self::Z31 => write!(f, "Z31"),
            Self::Z22 => write!(f, "Z22"),
            Self::Z23 => write!(f, "Z23"),
            Self::Z24 => write!(f, "Z24"),
            Self::Z36 => write!(f, "Z36"),
            Self::Z42 => write!(f, "Z42"),
            Self::ZA2 => write!(f, "ZA2"),
            Self::Z67 => write!(f, "Z67"),
            Self::Z44 => write!(f, "Z44"),
            Self::Z39 => write!(f, "Z39"),
            Self::Z38 => write!(f, "Z38"),
            Self::Z41 => write!(f, "Z41"),
            Self::Z17 => write!(f, "Z17"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z50 => write!(f, "Z50"),
            Self::Z56 => write!(f, "Z56"),
            Self::Z63 => write!(f, "Z63"),
            Self::Z52 => write!(f, "Z52"),
            Self::Z01 => write!(f, "Z01"),
            Self::Z32 => write!(f, "Z32"),
            Self::Z35 => write!(f, "Z35"),
            Self::Z10 => write!(f, "Z10"),
            Self::Z07 => write!(f, "Z07"),
            Self::Z02 => write!(f, "Z02"),
            Self::Z04 => write!(f, "Z04"),
            Self::Z06 => write!(f, "Z06"),
            Self::Z03 => write!(f, "Z03"),
            Self::Z05 => write!(f, "Z05"),
            Self::Z11 => write!(f, "Z11"),
            Self::Z99 => write!(f, "Z99"),
            Self::ZA0 => write!(f, "ZA0"),
            Self::Z25 => write!(f, "Z25"),
            Self::Z48 => write!(f, "Z48"),
            Self::Z28 => write!(f, "Z28"),
            Self::Z29 => write!(f, "Z29"),
            Self::Z61 => write!(f, "Z61"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7059Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z19" => Self::Z19,
            "Z66" => Self::Z66,
            "Z65" => Self::Z65,
            "Z49" => Self::Z49,
            "Z45" => Self::Z45,
            "Z46" => Self::Z46,
            "11" => Self::_11,
            "Z53" => Self::Z53,
            "Z30" => Self::Z30,
            "Z18" => Self::Z18,
            "Z20" => Self::Z20,
            "15" => Self::_15,
            "6" => Self::_6,
            "Z31" => Self::Z31,
            "Z22" => Self::Z22,
            "Z23" => Self::Z23,
            "Z24" => Self::Z24,
            "Z36" => Self::Z36,
            "Z42" => Self::Z42,
            "ZA2" => Self::ZA2,
            "Z67" => Self::Z67,
            "Z44" => Self::Z44,
            "Z39" => Self::Z39,
            "Z38" => Self::Z38,
            "Z41" => Self::Z41,
            "Z17" => Self::Z17,
            "Z37" => Self::Z37,
            "Z50" => Self::Z50,
            "Z56" => Self::Z56,
            "Z63" => Self::Z63,
            "Z52" => Self::Z52,
            "Z01" => Self::Z01,
            "Z32" => Self::Z32,
            "Z35" => Self::Z35,
            "Z10" => Self::Z10,
            "Z07" => Self::Z07,
            "Z02" => Self::Z02,
            "Z04" => Self::Z04,
            "Z06" => Self::Z06,
            "Z03" => Self::Z03,
            "Z05" => Self::Z05,
            "Z11" => Self::Z11,
            "Z99" => Self::Z99,
            "ZA0" => Self::ZA0,
            "Z25" => Self::Z25,
            "Z48" => Self::Z48,
            "Z28" => Self::Z28,
            "Z29" => Self::Z29,
            "Z61" => Self::Z61,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7110Qualifier {
    Z39,
    Z40,
    Z41,
    /// Der NB rechnet die Blindarbeit gegenüber dem Anschlussnutzer ab.
    Z36,
    /// Der NB rechnet die Blindarbeit gegenüber dem Lieferanten ab.
    Z37,
    /// Der NB hat noch nicht festgelegt gegenüber wem er die Blindarbeit abrechnet.
    Z38,
    Z19,
    Z20,
    Z08,
    Z09,
    Z10,
    Z11,
    Z12,
    Z13,
    Z06,
    Z07,
    Z01,
    Z02,
    Z03,
    Z04,
    Z05,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7110Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z39 => write!(f, "Z39"),
            Self::Z40 => write!(f, "Z40"),
            Self::Z41 => write!(f, "Z41"),
            Self::Z36 => write!(f, "Z36"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z38 => write!(f, "Z38"),
            Self::Z19 => write!(f, "Z19"),
            Self::Z20 => write!(f, "Z20"),
            Self::Z08 => write!(f, "Z08"),
            Self::Z09 => write!(f, "Z09"),
            Self::Z10 => write!(f, "Z10"),
            Self::Z11 => write!(f, "Z11"),
            Self::Z12 => write!(f, "Z12"),
            Self::Z13 => write!(f, "Z13"),
            Self::Z06 => write!(f, "Z06"),
            Self::Z07 => write!(f, "Z07"),
            Self::Z01 => write!(f, "Z01"),
            Self::Z02 => write!(f, "Z02"),
            Self::Z03 => write!(f, "Z03"),
            Self::Z04 => write!(f, "Z04"),
            Self::Z05 => write!(f, "Z05"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7110Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z39" => Self::Z39,
            "Z40" => Self::Z40,
            "Z41" => Self::Z41,
            "Z36" => Self::Z36,
            "Z37" => Self::Z37,
            "Z38" => Self::Z38,
            "Z19" => Self::Z19,
            "Z20" => Self::Z20,
            "Z08" => Self::Z08,
            "Z09" => Self::Z09,
            "Z10" => Self::Z10,
            "Z11" => Self::Z11,
            "Z12" => Self::Z12,
            "Z13" => Self::Z13,
            "Z06" => Self::Z06,
            "Z07" => Self::Z07,
            "Z01" => Self::Z01,
            "Z02" => Self::Z02,
            "Z03" => Self::Z03,
            "Z04" => Self::Z04,
            "Z05" => Self::Z05,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7111Qualifier {
    ZH9,
    ZV4,
    Z75,
    Z76,
    Z77,
    Z78,
    Z79,
    Z91,
    ZE4,
    ZD1,
    Z47,
    Z88,
    Z89,
    Z90,
    Z22,
    XYZ,
    E03,
    E04,
    E05,
    E06,
    E07,
    E08,
    E09,
    /// Anzuwenden wenn alle Messlokationen, die zur Erfassung der Energiewerte der Marktlokation erforderlich sind mit iMS ausgestattet sind.
    Z52,
    /// Anzuwenden wenn noch mindestens eine Messlokation, die zur Erfassung der Energiewerte der Marktlokation mit kME oder mME ausgestattet ist.
    Z53,
    /// Anzuwenden wenn keine Messlokation, zur Erfassung der Energiewerte der Marktlokation vorhanden ist (Pauschale-Marktlokation).
    Z68,
    E02,
    E14,
    Z36,
    Z33,
    Z34,
    Z35,
    Z37,
    Z46,
    Z74,
    Z73,
    ZA7,
    Z84,
    Z85,
    Z86,
    Z92,
    ZE1,
    ZB5,
    ZD9,
    ZE8,
    ZE9,
    ZB7,
    /// Hierunter ist Strom zu verstehen, der ausschließlich zum Betrieb von Endverbrauchsgeräten (z.B. Radio, Fernseher, Kühlschrank, Beleuchtung...) genutzt wird.
    Z64,
    /// Hierunter ist Strom zu verstehen, der zur Wärmebedarfsdeckung (z.B. Standspeicherheizung, Fußbodenspeicherheizungen, Wärmepumpe....) eingesetzt wird.
    Z65,
    ZE5,
    ZA8,
    ZB3,
    /// Netzbetreiber kann die Verbrauchseinrichtung einer Marktlokation unterbrechen. Es kommen gesonderte Netzentgelte für die Marktlokation zum Tragen.
    Z62,
    /// Netzbetreiber kann die Verbrauchseinrichtung einer Marktlokation nicht unterbrechen. Es kommen normale Netzentgelte zum Tragen.
    Z63,
    /// Hierunter fallen Heizungsanlagen bei denen das Speichermedium (z.B. Standspeicherheizkörper, Estrich als Fußbodenspeicher) während lastschwacher Zeiten aufgeladen wird und während des Tages Wärme abgeben.
    Z56,
    /// Wärmepumpen entziehen der Umwelt (Luft, Wasser oder Erdreich) Wärme und heben mit technischem Verfahren die Temperatur auf ein Temperaturniveau an, um Gebäude zu heizen. Der Stromverbrauch steht somit immer in zeitlicher Verbindung zur Wärmeerzeugung.
    Z57,
    /// Hierunter fallen Heizungsanlagen, die direkt und damit zeitgleich elektrische Energie in Wärme umwandeln (z.B. Konvektionsheizung, Infrarotheizung, Flächenheizung).
    Z61,
    ZV5,
    ZV6,
    ZV7,
    /// An der Marktlokation ist eine nicht öffentlliche Lademöglichkeit vorhanden
    ZE6,
    /// Es handelt sich um eine öffentliche Ladesäule mit ggf. mehreren Ladeanschlüssen an der Marktlokation
    Z87,
    /// Es handelt sich um mehr als eine öffentliche Ladesäule an der Marktlokation
    ZE7,
    ZF5,
    ZF6,
    ZG0,
    ZG1,
    ZG5,
    ZF7,
    ZF8,
    ZF9,
    ZG6,
    ZG8,
    ZG9,
    ZH0,
    ZH1,
    ZH2,
    ZH3,
    ZH4,
    ZH5,
    /// Dieser Code ist auszuwählen, wenn neben den genannten Technischen Ressourcen in der verbrauchenden Marktlokation weitere technische Einrichtungen (z. B. Kraft/ Licht) vorhanden sind
    ZH7,
    /// Dieser Code ist auszuwählen, wenn neben den genannten Technischen Ressourcen in der verbrauchenden Marktlokation keine weitere technische Einrichtung vorhanden ist.
    ZH8,
    ZF2,
    ZF0,
    ZB4,
    ZC9,
    AHZ,
    WSZ,
    LAZ,
    MAZ,
    MME,
    EHZ,
    IVA,
    Z30,
    ETZ,
    ZTZ,
    NTZ,
    ERZ,
    ZRZ,
    Z58,
    BKE,
    DPA,
    HUT,
    AMR,
    MMR,
    MIW,
    MPW,
    MBW,
    MUW,
    GSM,
    ETH,
    PLC,
    PST,
    DSL,
    LTE,
    RSU,
    TSU,
    Z95,
    Z96,
    Z97,
    Z98,
    Z99,
    ZA0,
    ZA1,
    ZA2,
    ZA3,
    ZA4,
    ZA5,
    ZA6,
    ZF1,
    ZG7,
    Z94,
    Z93,
    ZG3,
    ZG4,
    ZU5,
    ZU6,
    ZU7,
    ZU8,
    ZU9,
    ZV0,
    ZV1,
    ZV2,
    ZV3,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7111Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZH9 => write!(f, "ZH9"),
            Self::ZV4 => write!(f, "ZV4"),
            Self::Z75 => write!(f, "Z75"),
            Self::Z76 => write!(f, "Z76"),
            Self::Z77 => write!(f, "Z77"),
            Self::Z78 => write!(f, "Z78"),
            Self::Z79 => write!(f, "Z79"),
            Self::Z91 => write!(f, "Z91"),
            Self::ZE4 => write!(f, "ZE4"),
            Self::ZD1 => write!(f, "ZD1"),
            Self::Z47 => write!(f, "Z47"),
            Self::Z88 => write!(f, "Z88"),
            Self::Z89 => write!(f, "Z89"),
            Self::Z90 => write!(f, "Z90"),
            Self::Z22 => write!(f, "Z22"),
            Self::XYZ => write!(f, "XYZ"),
            Self::E03 => write!(f, "E03"),
            Self::E04 => write!(f, "E04"),
            Self::E05 => write!(f, "E05"),
            Self::E06 => write!(f, "E06"),
            Self::E07 => write!(f, "E07"),
            Self::E08 => write!(f, "E08"),
            Self::E09 => write!(f, "E09"),
            Self::Z52 => write!(f, "Z52"),
            Self::Z53 => write!(f, "Z53"),
            Self::Z68 => write!(f, "Z68"),
            Self::E02 => write!(f, "E02"),
            Self::E14 => write!(f, "E14"),
            Self::Z36 => write!(f, "Z36"),
            Self::Z33 => write!(f, "Z33"),
            Self::Z34 => write!(f, "Z34"),
            Self::Z35 => write!(f, "Z35"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z46 => write!(f, "Z46"),
            Self::Z74 => write!(f, "Z74"),
            Self::Z73 => write!(f, "Z73"),
            Self::ZA7 => write!(f, "ZA7"),
            Self::Z84 => write!(f, "Z84"),
            Self::Z85 => write!(f, "Z85"),
            Self::Z86 => write!(f, "Z86"),
            Self::Z92 => write!(f, "Z92"),
            Self::ZE1 => write!(f, "ZE1"),
            Self::ZB5 => write!(f, "ZB5"),
            Self::ZD9 => write!(f, "ZD9"),
            Self::ZE8 => write!(f, "ZE8"),
            Self::ZE9 => write!(f, "ZE9"),
            Self::ZB7 => write!(f, "ZB7"),
            Self::Z64 => write!(f, "Z64"),
            Self::Z65 => write!(f, "Z65"),
            Self::ZE5 => write!(f, "ZE5"),
            Self::ZA8 => write!(f, "ZA8"),
            Self::ZB3 => write!(f, "ZB3"),
            Self::Z62 => write!(f, "Z62"),
            Self::Z63 => write!(f, "Z63"),
            Self::Z56 => write!(f, "Z56"),
            Self::Z57 => write!(f, "Z57"),
            Self::Z61 => write!(f, "Z61"),
            Self::ZV5 => write!(f, "ZV5"),
            Self::ZV6 => write!(f, "ZV6"),
            Self::ZV7 => write!(f, "ZV7"),
            Self::ZE6 => write!(f, "ZE6"),
            Self::Z87 => write!(f, "Z87"),
            Self::ZE7 => write!(f, "ZE7"),
            Self::ZF5 => write!(f, "ZF5"),
            Self::ZF6 => write!(f, "ZF6"),
            Self::ZG0 => write!(f, "ZG0"),
            Self::ZG1 => write!(f, "ZG1"),
            Self::ZG5 => write!(f, "ZG5"),
            Self::ZF7 => write!(f, "ZF7"),
            Self::ZF8 => write!(f, "ZF8"),
            Self::ZF9 => write!(f, "ZF9"),
            Self::ZG6 => write!(f, "ZG6"),
            Self::ZG8 => write!(f, "ZG8"),
            Self::ZG9 => write!(f, "ZG9"),
            Self::ZH0 => write!(f, "ZH0"),
            Self::ZH1 => write!(f, "ZH1"),
            Self::ZH2 => write!(f, "ZH2"),
            Self::ZH3 => write!(f, "ZH3"),
            Self::ZH4 => write!(f, "ZH4"),
            Self::ZH5 => write!(f, "ZH5"),
            Self::ZH7 => write!(f, "ZH7"),
            Self::ZH8 => write!(f, "ZH8"),
            Self::ZF2 => write!(f, "ZF2"),
            Self::ZF0 => write!(f, "ZF0"),
            Self::ZB4 => write!(f, "ZB4"),
            Self::ZC9 => write!(f, "ZC9"),
            Self::AHZ => write!(f, "AHZ"),
            Self::WSZ => write!(f, "WSZ"),
            Self::LAZ => write!(f, "LAZ"),
            Self::MAZ => write!(f, "MAZ"),
            Self::MME => write!(f, "MME"),
            Self::EHZ => write!(f, "EHZ"),
            Self::IVA => write!(f, "IVA"),
            Self::Z30 => write!(f, "Z30"),
            Self::ETZ => write!(f, "ETZ"),
            Self::ZTZ => write!(f, "ZTZ"),
            Self::NTZ => write!(f, "NTZ"),
            Self::ERZ => write!(f, "ERZ"),
            Self::ZRZ => write!(f, "ZRZ"),
            Self::Z58 => write!(f, "Z58"),
            Self::BKE => write!(f, "BKE"),
            Self::DPA => write!(f, "DPA"),
            Self::HUT => write!(f, "HUT"),
            Self::AMR => write!(f, "AMR"),
            Self::MMR => write!(f, "MMR"),
            Self::MIW => write!(f, "MIW"),
            Self::MPW => write!(f, "MPW"),
            Self::MBW => write!(f, "MBW"),
            Self::MUW => write!(f, "MUW"),
            Self::GSM => write!(f, "GSM"),
            Self::ETH => write!(f, "ETH"),
            Self::PLC => write!(f, "PLC"),
            Self::PST => write!(f, "PST"),
            Self::DSL => write!(f, "DSL"),
            Self::LTE => write!(f, "LTE"),
            Self::RSU => write!(f, "RSU"),
            Self::TSU => write!(f, "TSU"),
            Self::Z95 => write!(f, "Z95"),
            Self::Z96 => write!(f, "Z96"),
            Self::Z97 => write!(f, "Z97"),
            Self::Z98 => write!(f, "Z98"),
            Self::Z99 => write!(f, "Z99"),
            Self::ZA0 => write!(f, "ZA0"),
            Self::ZA1 => write!(f, "ZA1"),
            Self::ZA2 => write!(f, "ZA2"),
            Self::ZA3 => write!(f, "ZA3"),
            Self::ZA4 => write!(f, "ZA4"),
            Self::ZA5 => write!(f, "ZA5"),
            Self::ZA6 => write!(f, "ZA6"),
            Self::ZF1 => write!(f, "ZF1"),
            Self::ZG7 => write!(f, "ZG7"),
            Self::Z94 => write!(f, "Z94"),
            Self::Z93 => write!(f, "Z93"),
            Self::ZG3 => write!(f, "ZG3"),
            Self::ZG4 => write!(f, "ZG4"),
            Self::ZU5 => write!(f, "ZU5"),
            Self::ZU6 => write!(f, "ZU6"),
            Self::ZU7 => write!(f, "ZU7"),
            Self::ZU8 => write!(f, "ZU8"),
            Self::ZU9 => write!(f, "ZU9"),
            Self::ZV0 => write!(f, "ZV0"),
            Self::ZV1 => write!(f, "ZV1"),
            Self::ZV2 => write!(f, "ZV2"),
            Self::ZV3 => write!(f, "ZV3"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7111Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "ZH9" => Self::ZH9,
            "ZV4" => Self::ZV4,
            "Z75" => Self::Z75,
            "Z76" => Self::Z76,
            "Z77" => Self::Z77,
            "Z78" => Self::Z78,
            "Z79" => Self::Z79,
            "Z91" => Self::Z91,
            "ZE4" => Self::ZE4,
            "ZD1" => Self::ZD1,
            "Z47" => Self::Z47,
            "Z88" => Self::Z88,
            "Z89" => Self::Z89,
            "Z90" => Self::Z90,
            "Z22" => Self::Z22,
            "XYZ" => Self::XYZ,
            "E03" => Self::E03,
            "E04" => Self::E04,
            "E05" => Self::E05,
            "E06" => Self::E06,
            "E07" => Self::E07,
            "E08" => Self::E08,
            "E09" => Self::E09,
            "Z52" => Self::Z52,
            "Z53" => Self::Z53,
            "Z68" => Self::Z68,
            "E02" => Self::E02,
            "E14" => Self::E14,
            "Z36" => Self::Z36,
            "Z33" => Self::Z33,
            "Z34" => Self::Z34,
            "Z35" => Self::Z35,
            "Z37" => Self::Z37,
            "Z46" => Self::Z46,
            "Z74" => Self::Z74,
            "Z73" => Self::Z73,
            "ZA7" => Self::ZA7,
            "Z84" => Self::Z84,
            "Z85" => Self::Z85,
            "Z86" => Self::Z86,
            "Z92" => Self::Z92,
            "ZE1" => Self::ZE1,
            "ZB5" => Self::ZB5,
            "ZD9" => Self::ZD9,
            "ZE8" => Self::ZE8,
            "ZE9" => Self::ZE9,
            "ZB7" => Self::ZB7,
            "Z64" => Self::Z64,
            "Z65" => Self::Z65,
            "ZE5" => Self::ZE5,
            "ZA8" => Self::ZA8,
            "ZB3" => Self::ZB3,
            "Z62" => Self::Z62,
            "Z63" => Self::Z63,
            "Z56" => Self::Z56,
            "Z57" => Self::Z57,
            "Z61" => Self::Z61,
            "ZV5" => Self::ZV5,
            "ZV6" => Self::ZV6,
            "ZV7" => Self::ZV7,
            "ZE6" => Self::ZE6,
            "Z87" => Self::Z87,
            "ZE7" => Self::ZE7,
            "ZF5" => Self::ZF5,
            "ZF6" => Self::ZF6,
            "ZG0" => Self::ZG0,
            "ZG1" => Self::ZG1,
            "ZG5" => Self::ZG5,
            "ZF7" => Self::ZF7,
            "ZF8" => Self::ZF8,
            "ZF9" => Self::ZF9,
            "ZG6" => Self::ZG6,
            "ZG8" => Self::ZG8,
            "ZG9" => Self::ZG9,
            "ZH0" => Self::ZH0,
            "ZH1" => Self::ZH1,
            "ZH2" => Self::ZH2,
            "ZH3" => Self::ZH3,
            "ZH4" => Self::ZH4,
            "ZH5" => Self::ZH5,
            "ZH7" => Self::ZH7,
            "ZH8" => Self::ZH8,
            "ZF2" => Self::ZF2,
            "ZF0" => Self::ZF0,
            "ZB4" => Self::ZB4,
            "ZC9" => Self::ZC9,
            "AHZ" => Self::AHZ,
            "WSZ" => Self::WSZ,
            "LAZ" => Self::LAZ,
            "MAZ" => Self::MAZ,
            "MME" => Self::MME,
            "EHZ" => Self::EHZ,
            "IVA" => Self::IVA,
            "Z30" => Self::Z30,
            "ETZ" => Self::ETZ,
            "ZTZ" => Self::ZTZ,
            "NTZ" => Self::NTZ,
            "ERZ" => Self::ERZ,
            "ZRZ" => Self::ZRZ,
            "Z58" => Self::Z58,
            "BKE" => Self::BKE,
            "DPA" => Self::DPA,
            "HUT" => Self::HUT,
            "AMR" => Self::AMR,
            "MMR" => Self::MMR,
            "MIW" => Self::MIW,
            "MPW" => Self::MPW,
            "MBW" => Self::MBW,
            "MUW" => Self::MUW,
            "GSM" => Self::GSM,
            "ETH" => Self::ETH,
            "PLC" => Self::PLC,
            "PST" => Self::PST,
            "DSL" => Self::DSL,
            "LTE" => Self::LTE,
            "RSU" => Self::RSU,
            "TSU" => Self::TSU,
            "Z95" => Self::Z95,
            "Z96" => Self::Z96,
            "Z97" => Self::Z97,
            "Z98" => Self::Z98,
            "Z99" => Self::Z99,
            "ZA0" => Self::ZA0,
            "ZA1" => Self::ZA1,
            "ZA2" => Self::ZA2,
            "ZA3" => Self::ZA3,
            "ZA4" => Self::ZA4,
            "ZA5" => Self::ZA5,
            "ZA6" => Self::ZA6,
            "ZF1" => Self::ZF1,
            "ZG7" => Self::ZG7,
            "Z94" => Self::Z94,
            "Z93" => Self::Z93,
            "ZG3" => Self::ZG3,
            "ZG4" => Self::ZG4,
            "ZU5" => Self::ZU5,
            "ZU6" => Self::ZU6,
            "ZU7" => Self::ZU7,
            "ZU8" => Self::ZU8,
            "ZU9" => Self::ZU9,
            "ZV0" => Self::ZV0,
            "ZV1" => Self::ZV1,
            "ZV2" => Self::ZV2,
            "ZV3" => Self::ZV3,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7140Qualifier {
    AUA,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7140Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AUA => write!(f, "AUA"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7140Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "AUA" => Self::AUA,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7143Qualifier {
    Z11,
    Z09,
    Z10,
    SRW,
    Z12,
    Z08,
    Z03,
    Z04,
    Z05,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7143Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z11 => write!(f, "Z11"),
            Self::Z09 => write!(f, "Z09"),
            Self::Z10 => write!(f, "Z10"),
            Self::SRW => write!(f, "SRW"),
            Self::Z12 => write!(f, "Z12"),
            Self::Z08 => write!(f, "Z08"),
            Self::Z03 => write!(f, "Z03"),
            Self::Z04 => write!(f, "Z04"),
            Self::Z05 => write!(f, "Z05"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7143Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z11" => Self::Z11,
            "Z09" => Self::Z09,
            "Z10" => Self::Z10,
            "SRW" => Self::SRW,
            "Z12" => Self::Z12,
            "Z08" => Self::Z08,
            "Z03" => Self::Z03,
            "Z04" => Self::Z04,
            "Z05" => Self::Z05,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7431Qualifier {
    _9,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7431Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_9 => write!(f, "9"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7431Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "9" => Self::_9,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7433Qualifier {
    Z04,
    Z06,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7433Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z04 => write!(f, "Z04"),
            Self::Z06 => write!(f, "Z06"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7433Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z04" => Self::Z04,
            "Z06" => Self::Z06,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D7495Qualifier {
    Z01,
    _24,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D7495Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Z01 => write!(f, "Z01"),
            Self::_24 => write!(f, "24"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D7495Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "Z01" => Self::Z01,
            "24" => Self::_24,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D9013Qualifier {
    /// Kunde verlässt oder bezieht eine schon bestehende Marktlokation
    E01,
    /// Kunde bezieht erstmals eine Marktlokation (z. B. Neubau)
    E02,
    /// - Kunde bleibt an der Marktlokation, hat nur den Marktpartner gewechselt - Marktpartner hat den Kunden gekündigt
    E03,
    /// Dient dem Rückruf von abgegebenen Meldungen
    E05,
    E06,
    Z02,
    Z15,
    /// Der NB informiert den LFN darüber, dass zum gewünschten Anmeldedatum noch ein anderer LFA der Marktlokation zugeordnet ist und deshalb eine Abmeldeanfrage an den LFA stellt.
    Z26,
    /// Kunde zieht aus der Marktlokation aus und die Marktlokation wird stillgelegt (bei allen anderen Auszügen ist E01 zu verwenden)
    Z33,
    /// Beim NB liegt nur eine Auszugsmeldung für die Marktlokation vor. Es erfolgt seitens des NB anschließend eine Meldung der Marktlokation an den E/G.
    Z36,
    /// Beim NB liegt eine neue Marktlokation ohne Lieferantenzuordnung vor. Daher erfolgt eine Meldung an den zuständigen LF für die EoG.
    Z37,
    /// Ein erstellter vorübergehender Anschluss wird aufgrund von fehlendem LF dem zuständigen LF für die EoG gemeldet.
    Z39,
    /// Verstreicht die gesetzliche 3 Monatsfrist der Ersatzversorgung ohne Aufnahme der Folgelieferung durch einen Lieferanten, kann der zuständigen LF die Marktlokation mit diesem Transaktionsgrund abmelden.
    Z41,
    ZC6,
    ZC7,
    ZC8,
    /// ZD9 wird angewendet in der Informationsmeldung des NB an den LF innerhalb der „GPKE“. Diese Meldung wird verwendet, wenn der EEG- bzw. KWK-G-Anlagenbetreiber über das entsprechende Formular 100% seiner Anlage wieder in die gesetzliche Förderung überführt hat.
    ZD9,
    ZG5,
    ZG6,
    ZG9,
    ZH0,
    ZH1,
    /// Vertrag zwischen Absender des Geschäftsvorfalls und Kunde wurde aufgehoben, wird z. B. verwendet wenn der Kunde den Vertrag widerruft.
    ZH2,
    ZE3,
    ZJ4,
    ZP3,
    ZP4,
    ZQ7,
    ZR9,
    ZT0,
    /// Anzuwenden wenn Ende wegen Kündigung durch bislang beliefernden LF (LFA) entsteht
    ZT4,
    /// Anzuwenden wenn Ende wegen - Kündigung durch den Kunden, - Kündigung durch LFN, - Keine Kündigung des Vertrages notwendig da Vertrag nur auf bestimmte Zeit gelaufen ist und von alleine ausläuft und - Kündigung durch Dritten z.B. unabhängige Berater entsteht.
    ZT5,
    ZT6,
    ZT7,
    ZU1,
    ZX2,
    ZX3,
    ZX4,
    ZX5,
    ZX6,
    ZX7,
    ZX8,
    ZX9,
    ZY0,
    ZY1,
    ZY2,
    ZY4,
    ZY5,
    ZY6,
    ZY7,
    ZY9,
    ZAM,
    ZAN,
    ZAO,
    ZZA,
    /// Der LFN wird einer Marktlokation vollständig zugeordnet (vollständige (100%ige) Zuordnung). Dieser Geschäftsvorfall ist auch für die Änderung von einer tranchierten Marktlokation in eine nicht tranchierte Marktlokation anzuwenden.
    ZW0,
    /// Der LFN wird einer bestehenden Tranche vollständig zugeordnet (vollständige (100%ige) Zuordnung). Dieser Geschäftsvorfall ist bei einem direkten Übergang, d. h. lückenlosem Zuordnungsende und -beginn und unter Beibehaltung der Tranche, anzuwenden.
    ZW1,
    /// Der LFN wird einer neu zu bildenden Tranche zugeordnet (anteiliger Zuordnungsvorgang unter Bildung neuer Tranchen
    ZW2,
    ZW3,
    ZW4,
    ZW5,
    ZW6,
    ZW7,
    ZW8,
    ZW9,
    ZX0,
    ZX1,
    ZAP,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D9013Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E01 => write!(f, "E01"),
            Self::E02 => write!(f, "E02"),
            Self::E03 => write!(f, "E03"),
            Self::E05 => write!(f, "E05"),
            Self::E06 => write!(f, "E06"),
            Self::Z02 => write!(f, "Z02"),
            Self::Z15 => write!(f, "Z15"),
            Self::Z26 => write!(f, "Z26"),
            Self::Z33 => write!(f, "Z33"),
            Self::Z36 => write!(f, "Z36"),
            Self::Z37 => write!(f, "Z37"),
            Self::Z39 => write!(f, "Z39"),
            Self::Z41 => write!(f, "Z41"),
            Self::ZC6 => write!(f, "ZC6"),
            Self::ZC7 => write!(f, "ZC7"),
            Self::ZC8 => write!(f, "ZC8"),
            Self::ZD9 => write!(f, "ZD9"),
            Self::ZG5 => write!(f, "ZG5"),
            Self::ZG6 => write!(f, "ZG6"),
            Self::ZG9 => write!(f, "ZG9"),
            Self::ZH0 => write!(f, "ZH0"),
            Self::ZH1 => write!(f, "ZH1"),
            Self::ZH2 => write!(f, "ZH2"),
            Self::ZE3 => write!(f, "ZE3"),
            Self::ZJ4 => write!(f, "ZJ4"),
            Self::ZP3 => write!(f, "ZP3"),
            Self::ZP4 => write!(f, "ZP4"),
            Self::ZQ7 => write!(f, "ZQ7"),
            Self::ZR9 => write!(f, "ZR9"),
            Self::ZT0 => write!(f, "ZT0"),
            Self::ZT4 => write!(f, "ZT4"),
            Self::ZT5 => write!(f, "ZT5"),
            Self::ZT6 => write!(f, "ZT6"),
            Self::ZT7 => write!(f, "ZT7"),
            Self::ZU1 => write!(f, "ZU1"),
            Self::ZX2 => write!(f, "ZX2"),
            Self::ZX3 => write!(f, "ZX3"),
            Self::ZX4 => write!(f, "ZX4"),
            Self::ZX5 => write!(f, "ZX5"),
            Self::ZX6 => write!(f, "ZX6"),
            Self::ZX7 => write!(f, "ZX7"),
            Self::ZX8 => write!(f, "ZX8"),
            Self::ZX9 => write!(f, "ZX9"),
            Self::ZY0 => write!(f, "ZY0"),
            Self::ZY1 => write!(f, "ZY1"),
            Self::ZY2 => write!(f, "ZY2"),
            Self::ZY4 => write!(f, "ZY4"),
            Self::ZY5 => write!(f, "ZY5"),
            Self::ZY6 => write!(f, "ZY6"),
            Self::ZY7 => write!(f, "ZY7"),
            Self::ZY9 => write!(f, "ZY9"),
            Self::ZAM => write!(f, "ZAM"),
            Self::ZAN => write!(f, "ZAN"),
            Self::ZAO => write!(f, "ZAO"),
            Self::ZZA => write!(f, "ZZA"),
            Self::ZW0 => write!(f, "ZW0"),
            Self::ZW1 => write!(f, "ZW1"),
            Self::ZW2 => write!(f, "ZW2"),
            Self::ZW3 => write!(f, "ZW3"),
            Self::ZW4 => write!(f, "ZW4"),
            Self::ZW5 => write!(f, "ZW5"),
            Self::ZW6 => write!(f, "ZW6"),
            Self::ZW7 => write!(f, "ZW7"),
            Self::ZW8 => write!(f, "ZW8"),
            Self::ZW9 => write!(f, "ZW9"),
            Self::ZX0 => write!(f, "ZX0"),
            Self::ZX1 => write!(f, "ZX1"),
            Self::ZAP => write!(f, "ZAP"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D9013Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "E01" => Self::E01,
            "E02" => Self::E02,
            "E03" => Self::E03,
            "E05" => Self::E05,
            "E06" => Self::E06,
            "Z02" => Self::Z02,
            "Z15" => Self::Z15,
            "Z26" => Self::Z26,
            "Z33" => Self::Z33,
            "Z36" => Self::Z36,
            "Z37" => Self::Z37,
            "Z39" => Self::Z39,
            "Z41" => Self::Z41,
            "ZC6" => Self::ZC6,
            "ZC7" => Self::ZC7,
            "ZC8" => Self::ZC8,
            "ZD9" => Self::ZD9,
            "ZG5" => Self::ZG5,
            "ZG6" => Self::ZG6,
            "ZG9" => Self::ZG9,
            "ZH0" => Self::ZH0,
            "ZH1" => Self::ZH1,
            "ZH2" => Self::ZH2,
            "ZE3" => Self::ZE3,
            "ZJ4" => Self::ZJ4,
            "ZP3" => Self::ZP3,
            "ZP4" => Self::ZP4,
            "ZQ7" => Self::ZQ7,
            "ZR9" => Self::ZR9,
            "ZT0" => Self::ZT0,
            "ZT4" => Self::ZT4,
            "ZT5" => Self::ZT5,
            "ZT6" => Self::ZT6,
            "ZT7" => Self::ZT7,
            "ZU1" => Self::ZU1,
            "ZX2" => Self::ZX2,
            "ZX3" => Self::ZX3,
            "ZX4" => Self::ZX4,
            "ZX5" => Self::ZX5,
            "ZX6" => Self::ZX6,
            "ZX7" => Self::ZX7,
            "ZX8" => Self::ZX8,
            "ZX9" => Self::ZX9,
            "ZY0" => Self::ZY0,
            "ZY1" => Self::ZY1,
            "ZY2" => Self::ZY2,
            "ZY4" => Self::ZY4,
            "ZY5" => Self::ZY5,
            "ZY6" => Self::ZY6,
            "ZY7" => Self::ZY7,
            "ZY9" => Self::ZY9,
            "ZAM" => Self::ZAM,
            "ZAN" => Self::ZAN,
            "ZAO" => Self::ZAO,
            "ZZA" => Self::ZZA,
            "ZW0" => Self::ZW0,
            "ZW1" => Self::ZW1,
            "ZW2" => Self::ZW2,
            "ZW3" => Self::ZW3,
            "ZW4" => Self::ZW4,
            "ZW5" => Self::ZW5,
            "ZW6" => Self::ZW6,
            "ZW7" => Self::ZW7,
            "ZW8" => Self::ZW8,
            "ZW9" => Self::ZW9,
            "ZX0" => Self::ZX0,
            "ZX1" => Self::ZX1,
            "ZAP" => Self::ZAP,
            other => Self::Unknown(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum D9015Qualifier {
    E01,
    _7,
    Z35,
    /// Unrecognized code value
    Unknown(String),
}

impl std::fmt::Display for D9015Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E01 => write!(f, "E01"),
            Self::_7 => write!(f, "7"),
            Self::Z35 => write!(f, "Z35"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for D9015Qualifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "E01" => Self::E01,
            "7" => Self::_7,
            "Z35" => Self::Z35,
            other => Self::Unknown(other.to_string()),
        })
    }
}
