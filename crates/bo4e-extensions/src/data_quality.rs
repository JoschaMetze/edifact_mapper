use serde::{Deserialize, Serialize};

/// Data quality indicator for EDIFACT domain objects.
///
/// Indicates the completeness/reliability of data attached to a location
/// or other entity in the UTILMD message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataQuality {
    /// Complete data — all fields are present and verified.
    Vollstaendig,
    /// Expected data — fields are present but not yet confirmed.
    Erwartet,
    /// Data already exists in the system.
    ImSystemVorhanden,
    /// Informative only — data is provided for reference.
    Informativ,
}

impl DataQuality {
    /// Converts from an EDIFACT qualifier string.
    pub fn from_qualifier(qualifier: &str) -> Option<Self> {
        match qualifier {
            "Z36" | "VOLLSTAENDIG" => Some(Self::Vollstaendig),
            "Z34" | "ERWARTET" => Some(Self::Erwartet),
            "Z35" | "IM_SYSTEM_VORHANDEN" => Some(Self::ImSystemVorhanden),
            "Z33" | "INFORMATIV" => Some(Self::Informativ),
            _ => None,
        }
    }

    /// Converts to the EDIFACT qualifier string.
    pub fn to_qualifier(&self) -> &'static str {
        match self {
            Self::Vollstaendig => "Z36",
            Self::Erwartet => "Z34",
            Self::ImSystemVorhanden => "Z35",
            Self::Informativ => "Z33",
        }
    }
}

impl std::fmt::Display for DataQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vollstaendig => write!(f, "VOLLSTAENDIG"),
            Self::Erwartet => write!(f, "ERWARTET"),
            Self::ImSystemVorhanden => write!(f, "IM_SYSTEM_VORHANDEN"),
            Self::Informativ => write!(f, "INFORMATIV"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_quality_from_qualifier() {
        assert_eq!(
            DataQuality::from_qualifier("Z36"),
            Some(DataQuality::Vollstaendig)
        );
        assert_eq!(
            DataQuality::from_qualifier("Z33"),
            Some(DataQuality::Informativ)
        );
        assert_eq!(DataQuality::from_qualifier("XXX"), None);
    }

    #[test]
    fn test_data_quality_roundtrip() {
        for dq in [
            DataQuality::Vollstaendig,
            DataQuality::Erwartet,
            DataQuality::ImSystemVorhanden,
            DataQuality::Informativ,
        ] {
            let q = dq.to_qualifier();
            assert_eq!(DataQuality::from_qualifier(q), Some(dq));
        }
    }

    #[test]
    fn test_data_quality_serde() {
        let dq = DataQuality::Vollstaendig;
        let json = serde_json::to_string(&dq).unwrap();
        let deserialized: DataQuality = serde_json::from_str(&json).unwrap();
        assert_eq!(dq, deserialized);
    }
}
