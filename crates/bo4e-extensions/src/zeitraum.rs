use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A time period with optional start and end.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Zeitraum {
    pub von: Option<NaiveDateTime>,
    pub bis: Option<NaiveDateTime>,
}

impl Zeitraum {
    /// Creates a new Zeitraum with the given start and end.
    pub fn new(von: Option<NaiveDateTime>, bis: Option<NaiveDateTime>) -> Self {
        Self { von, bis }
    }

    /// Returns true if this Zeitraum has both start and end set.
    pub fn is_bounded(&self) -> bool {
        self.von.is_some() && self.bis.is_some()
    }

    /// Returns true if neither start nor end is set.
    pub fn is_empty(&self) -> bool {
        self.von.is_none() && self.bis.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_zeitraum_new() {
        let von = NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let bis = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let z = Zeitraum::new(Some(von), Some(bis));
        assert!(z.is_bounded());
        assert!(!z.is_empty());
    }

    #[test]
    fn test_zeitraum_default_is_empty() {
        let z = Zeitraum::default();
        assert!(z.is_empty());
        assert!(!z.is_bounded());
    }

    #[test]
    fn test_zeitraum_serde_roundtrip() {
        let von = NaiveDate::from_ymd_opt(2025, 6, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let z = Zeitraum::new(Some(von), None);
        let json = serde_json::to_string(&z).unwrap();
        let deserialized: Zeitraum = serde_json::from_str(&json).unwrap();
        assert_eq!(z, deserialized);
    }
}
