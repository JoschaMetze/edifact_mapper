//! Mapper for Geschaeftspartner (business partner) from NAD segments.
//!
//! Handles NAD segments with party qualifiers (Z04, Z09, DP, etc.).

use bo4e_extensions::{Adresse, Geschaeftspartner, GeschaeftspartnerEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Geschaeftspartner in UTILMD messages.
///
/// Handles NAD segments with party qualifiers for business partners.
/// Each NAD with a party qualifier (Z04, Z09, Z48, Z50, etc.) creates
/// a separate Geschaeftspartner entry.
pub struct GeschaeftspartnerMapper {
    partners: Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>,
    has_data: bool,
}

impl GeschaeftspartnerMapper {
    pub fn new() -> Self {
        Self {
            partners: Vec::new(),
            has_data: false,
        }
    }

    /// NAD qualifiers that create Geschaeftspartner entries.
    fn is_party_qualifier(qualifier: &str) -> bool {
        matches!(
            qualifier,
            "Z04" | "Z09" | "Z48" | "Z50" | "Z25" | "Z26" | "EO" | "DDO"
        )
    }
}

impl Default for GeschaeftspartnerMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for GeschaeftspartnerMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        if segment.id != "NAD" {
            return false;
        }
        let qualifier = segment.get_element(0);
        Self::is_party_qualifier(qualifier)
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        let qualifier = segment.get_element(0);
        if !Self::is_party_qualifier(qualifier) {
            return;
        }

        // NAD+Z04+9900123000002::293+name1+++city++plz+DE'
        // C082 (element 1): party ID composite
        //   Component 0: party identification code
        //   Component 2: code list qualifier
        let party_id = segment.get_component(1, 0);
        let name1 = segment.get_element(2);

        // C059 (element 4): address
        let strasse = segment.get_component(4, 0);
        let hausnummer = segment.get_component(4, 2);
        let ort = segment.get_element(5);
        let plz = segment.get_element(7);
        let land = segment.get_element(8);

        let gp = Geschaeftspartner {
            name1: if !name1.is_empty() {
                Some(name1.to_string())
            } else if !party_id.is_empty() {
                Some(party_id.to_string())
            } else {
                None
            },
            partneradresse: if !strasse.is_empty() || !ort.is_empty() {
                Some(Adresse {
                    strasse: if strasse.is_empty() {
                        None
                    } else {
                        Some(strasse.to_string())
                    },
                    hausnummer: if hausnummer.is_empty() {
                        None
                    } else {
                        Some(hausnummer.to_string())
                    },
                    postleitzahl: if plz.is_empty() {
                        None
                    } else {
                        Some(plz.to_string())
                    },
                    ort: if ort.is_empty() {
                        None
                    } else {
                        Some(ort.to_string())
                    },
                    landescode: if land.is_empty() {
                        None
                    } else {
                        Some(land.to_string())
                    },
                })
            } else {
                None
            },
            ..Default::default()
        };

        let edifact = GeschaeftspartnerEdifact {
            nad_qualifier: Some(qualifier.to_string()),
        };

        self.partners.push(WithValidity {
            data: gp,
            edifact,
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        });
        self.has_data = true;
    }
}

impl Builder<Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>>
    for GeschaeftspartnerMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>> {
        self.has_data = false;
        std::mem::take(&mut self.partners)
    }

    fn reset(&mut self) {
        self.partners.clear();
        self.has_data = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edifact_types::SegmentPosition;

    fn pos() -> SegmentPosition {
        SegmentPosition::new(1, 0, 1)
    }

    #[test]
    fn test_geschaeftspartner_mapper_nad_z04() {
        let mut mapper = GeschaeftspartnerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let nad = RawSegment::new(
            "NAD",
            vec![vec!["Z04"], vec!["9900123000002", "", "293"]],
            pos(),
        );
        assert!(mapper.can_handle(&nad));
        mapper.handle(&nad, &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edifact.nad_qualifier, Some("Z04".to_string()));
        assert_eq!(result[0].data.name1, Some("9900123000002".to_string()));
    }

    #[test]
    fn test_geschaeftspartner_mapper_ignores_nad_ms() {
        let mapper = GeschaeftspartnerMapper::new();
        let nad = RawSegment::new("NAD", vec![vec!["MS"], vec!["ID"]], pos());
        assert!(!mapper.can_handle(&nad));
    }

    #[test]
    fn test_geschaeftspartner_mapper_multiple() {
        let mut mapper = GeschaeftspartnerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("NAD", vec![vec!["Z04"], vec!["PARTY1"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("NAD", vec![vec!["Z09"], vec!["PARTY2"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 2);
    }
}
