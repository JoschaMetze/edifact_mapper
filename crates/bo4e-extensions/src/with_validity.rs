use serde::{Deserialize, Serialize};

use crate::zeitraum::Zeitraum;

/// Wraps a BO4E business object with time validity and EDIFACT-specific context.
///
/// - `T` — the standard BO4E business object (pure data)
/// - `E` — the EDIFACT companion type (functional domain data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithValidity<T, E> {
    /// The pure BO4E business object.
    pub data: T,
    /// EDIFACT-specific functional domain data.
    pub edifact: E,
    /// Optional validity period.
    pub gueltigkeitszeitraum: Option<Zeitraum>,
    /// Reference to the original Zeitscheibe for roundtrip support.
    pub zeitscheibe_ref: Option<String>,
}

impl<T: Default, E: Default> Default for WithValidity<T, E> {
    fn default() -> Self {
        Self {
            data: T::default(),
            edifact: E::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        }
    }
}

impl<T, E: Default> WithValidity<T, E> {
    /// Creates a new WithValidity wrapping the given data with default EDIFACT context.
    pub fn new(data: T) -> Self {
        Self {
            data,
            edifact: E::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        }
    }

    /// Sets the validity period.
    pub fn with_zeitraum(mut self, zeitraum: Zeitraum) -> Self {
        self.gueltigkeitszeitraum = Some(zeitraum);
        self
    }

    /// Sets the Zeitscheibe reference.
    pub fn with_zeitscheibe_ref(mut self, zs_ref: String) -> Self {
        self.zeitscheibe_ref = Some(zs_ref);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bo4e_types::Marktlokation;
    use crate::data_quality::DataQuality;

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct TestEdifact {
        pub datenqualitaet: Option<DataQuality>,
        pub custom_field: Option<String>,
    }

    #[test]
    fn test_with_validity_new() {
        let ml = Marktlokation {
            marktlokations_id: Some("DE001".to_string()),
            ..Default::default()
        };
        let wv: WithValidity<Marktlokation, TestEdifact> = WithValidity::new(ml);

        assert_eq!(wv.data.marktlokations_id, Some("DE001".to_string()));
        assert!(wv.edifact.datenqualitaet.is_none());
        assert!(wv.gueltigkeitszeitraum.is_none());
        assert!(wv.zeitscheibe_ref.is_none());
    }

    #[test]
    fn test_with_validity_builder_pattern() {
        let wv: WithValidity<Marktlokation, TestEdifact> =
            WithValidity::new(Marktlokation::default())
                .with_zeitraum(Zeitraum::default())
                .with_zeitscheibe_ref("ZS001".to_string());

        assert!(wv.gueltigkeitszeitraum.is_some());
        assert_eq!(wv.zeitscheibe_ref, Some("ZS001".to_string()));
    }

    #[test]
    fn test_with_validity_serde_roundtrip() {
        let wv: WithValidity<Marktlokation, TestEdifact> = WithValidity {
            data: Marktlokation {
                marktlokations_id: Some("DE001".to_string()),
                ..Default::default()
            },
            edifact: TestEdifact {
                datenqualitaet: Some(DataQuality::Vollstaendig),
                custom_field: Some("test".to_string()),
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: Some("1".to_string()),
        };

        let json = serde_json::to_string_pretty(&wv).unwrap();
        let deserialized: WithValidity<Marktlokation, TestEdifact> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.data.marktlokations_id,
            Some("DE001".to_string())
        );
        assert_eq!(
            deserialized.edifact.datenqualitaet,
            Some(DataQuality::Vollstaendig)
        );
        assert_eq!(deserialized.zeitscheibe_ref, Some("1".to_string()));
    }

    #[test]
    fn test_with_validity_default() {
        let wv: WithValidity<Marktlokation, TestEdifact> = WithValidity::default();
        assert!(wv.data.marktlokations_id.is_none());
        assert!(wv.edifact.custom_field.is_none());
    }
}
