//! Mapper for Marktlokation (market location) business objects.
//!
//! Handles:
//! - LOC+Z16: Market location identifier
//! - SEQ+Z01: Marktlokation data group (Zugeordneter Marktpartner, Spannungsebene)
//! - NAD+DP/Z63: Delivery address
//! - NAD+MS: Sparte extraction from code list qualifier
//!
//! Produces: `Vec<WithValidity<Marktlokation, MarktlokationEdifact>>`
//!
//! Mirrors C# `MarktlokationMapper.cs`.

use bo4e_extensions::{Marktlokation, MarktlokationEdifact, WithValidity};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Marktlokation business objects in UTILMD messages.
///
/// Supports multiple Marktlokationen per transaction. Each LOC+Z16 segment
/// creates a new entity.
pub struct MarktlokationMapper {
    marktlokations_id: Option<String>,
    sparte: Option<String>,
    strasse: Option<String>,
    hausnummer: Option<String>,
    postleitzahl: Option<String>,
    ort: Option<String>,
    landescode: Option<String>,
    netzebene: Option<String>,
    bilanzierungsmethode: Option<String>,
    edifact: MarktlokationEdifact,
    has_data: bool,
    in_seq_z01: bool,
    after_nad_address: bool,
    items: Vec<WithValidity<Marktlokation, MarktlokationEdifact>>,
    delimiters: EdifactDelimiters,
}

impl MarktlokationMapper {
    /// Creates a new MarktlokationMapper.
    pub fn new() -> Self {
        Self {
            marktlokations_id: None,
            sparte: None,
            strasse: None,
            hausnummer: None,
            postleitzahl: None,
            ort: None,
            landescode: None,
            netzebene: None,
            bilanzierungsmethode: None,
            edifact: MarktlokationEdifact::default(),
            has_data: false,
            in_seq_z01: false,
            after_nad_address: false,
            items: Vec::new(),
            delimiters: EdifactDelimiters::default(),
        }
    }

    /// Set delimiters for raw segment serialization.
    pub fn set_delimiters(&mut self, delimiters: EdifactDelimiters) {
        self.delimiters = delimiters;
    }

    /// Finalizes the current item and pushes it to the items list.
    fn finalize_current(&mut self) {
        if self.has_data {
            let ml = Marktlokation {
                marktlokations_id: self.marktlokations_id.take(),
                sparte: self.sparte.clone(), // Clone: sparte applies to all MaLos
                lokationsadresse: if self.strasse.is_some() || self.ort.is_some() {
                    Some(bo4e_extensions::Adresse {
                        strasse: self.strasse.take(),
                        hausnummer: self.hausnummer.take(),
                        postleitzahl: self.postleitzahl.take(),
                        ort: self.ort.take(),
                        landescode: self.landescode.take(),
                    })
                } else {
                    None
                },
                bilanzierungsmethode: self.bilanzierungsmethode.take(),
                netzebene: self.netzebene.take(),
            };

            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: ml,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }

    fn handle_loc(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        if qualifier != "Z16" {
            return;
        }
        let id = segment.get_component(1, 0);
        if !id.is_empty() {
            // Finalize previous entity before starting a new one
            self.finalize_current();
            self.marktlokations_id = Some(id.to_string());
            self.edifact.raw_loc = Some(segment.to_raw_string(&self.delimiters));
            self.has_data = true;
        }
    }

    fn handle_nad(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        match qualifier {
            "DP" | "Z63" | "Z59" | "Z60" => self.handle_nad_address(segment),
            "MS" => self.handle_nad_ms(segment),
            _ => {}
        }
    }

    fn handle_nad_address(&mut self, segment: &RawSegment) {
        // Store raw NAD for roundtrip fidelity
        let raw = segment.to_raw_string(&self.delimiters);
        self.edifact.raw_nad_address.push(raw);
        self.after_nad_address = true;

        // NAD+DP++++Bergstr.::1+Berlin++10115+DE'
        // C059 (element 4): street address
        //   Component 0: street, Component 2: house number
        // Element 5: city
        // Element 7: postal code
        // Element 8: country code
        let strasse = segment.get_component(4, 0);
        if !strasse.is_empty() {
            self.strasse = Some(strasse.to_string());
            self.has_data = true;
        }

        let hausnummer = segment.get_component(4, 2);
        if !hausnummer.is_empty() {
            self.hausnummer = Some(hausnummer.to_string());
        }

        // Element 4 component 3: Ortsteil (district/sub-city)
        let _ortsteil = segment.get_component(4, 3);

        let ort = segment.get_element(5);
        if !ort.is_empty() {
            self.ort = Some(ort.to_string());
        }

        let plz = segment.get_element(7);
        if !plz.is_empty() {
            self.postleitzahl = Some(plz.to_string());
        }

        let land = segment.get_element(8);
        if !land.is_empty() {
            self.landescode = Some(land.to_string());
        }
    }

    fn handle_nad_ms(&mut self, segment: &RawSegment) {
        let code_qualifier = segment.get_component(1, 2);
        if !code_qualifier.is_empty() {
            let sparte = match code_qualifier {
                "293" | "500" => Some("STROM"),
                "332" => Some("GAS"),
                _ => None,
            };
            if let Some(s) = sparte {
                self.sparte = Some(s.to_string());
                // Don't set has_data — sparte is background info that applies to
                // all entities. Only LOC+Z16 and NAD+DP start actual entities.
            }
        }
    }

    fn handle_seq(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        // Reset SEQ state flags
        self.in_seq_z01 = false;

        if qualifier == "Z01" {
            self.in_seq_z01 = true;
        }
    }
}

impl Default for MarktlokationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for MarktlokationMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "LOC" => segment.get_element(0) == "Z16",
            "NAD" => {
                let q = segment.get_element(0);
                matches!(q, "DP" | "Z63" | "Z59" | "Z60" | "MS") || self.after_nad_address
            }
            "RFF" => self.after_nad_address,
            "SEQ" => true, // Handle all SEQ to track context
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "LOC" => {
                self.after_nad_address = false;
                self.handle_loc(segment);
            }
            "NAD" => {
                self.after_nad_address = false;
                self.handle_nad(segment);
            }
            "RFF" if self.after_nad_address => {
                // RFF following NAD+DP/Z63 — store as raw for roundtrip
                let raw = segment.to_raw_string(&self.delimiters);
                self.edifact.raw_nad_address.push(raw);
                // Keep after_nad_address true for additional RFFs
            }
            "SEQ" => {
                self.after_nad_address = false;
                self.handle_seq(segment);
            }
            _ => {}
        }
    }
}

impl Builder<Vec<WithValidity<Marktlokation, MarktlokationEdifact>>> for MarktlokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Marktlokation, MarktlokationEdifact>> {
        self.finalize_current();
        std::mem::take(&mut self.items)
    }

    fn reset(&mut self) {
        *self = Self::new();
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
    fn test_marktlokation_mapper_loc_z16() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let loc = RawSegment::new(
            "LOC",
            vec![vec!["Z16"], vec!["DE00014545768S0000000000000003054"]],
            pos(),
        );

        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);

        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].data.marktlokations_id,
            Some("DE00014545768S0000000000000003054".to_string())
        );
    }

    #[test]
    fn test_marktlokation_mapper_multiple_z16() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let loc1 = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO001"]], pos());
        let loc2 = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO002"]], pos());

        mapper.handle(&loc1, &mut ctx);
        mapper.handle(&loc2, &mut ctx);

        let result = mapper.build();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].data.marktlokations_id,
            Some("MALO001".to_string())
        );
        assert_eq!(
            result[1].data.marktlokations_id,
            Some("MALO002".to_string())
        );
    }

    #[test]
    fn test_marktlokation_mapper_ignores_loc_z17() {
        let mapper = MarktlokationMapper::new();

        let loc = RawSegment::new("LOC", vec![vec!["Z17"], vec!["MELO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_marktlokation_mapper_nad_dp_address() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let nad = RawSegment::new(
            "NAD",
            vec![
                vec!["DP"],
                vec![],
                vec![],
                vec![],
                vec!["Bergstr.", "", "1"],
                vec!["Berlin"],
                vec![],
                vec!["10115"],
                vec!["DE"],
            ],
            pos(),
        );

        assert!(mapper.can_handle(&nad));
        mapper.handle(&nad, &mut ctx);

        let result = mapper.build();
        assert_eq!(result.len(), 1);
        let addr = result[0].data.lokationsadresse.as_ref().unwrap();
        assert_eq!(addr.strasse, Some("Bergstr.".to_string()));
        assert_eq!(addr.hausnummer, Some("1".to_string()));
        assert_eq!(addr.ort, Some("Berlin".to_string()));
        assert_eq!(addr.postleitzahl, Some("10115".to_string()));
        assert_eq!(addr.landescode, Some("DE".to_string()));
    }

    #[test]
    fn test_marktlokation_mapper_nad_ms_sparte() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // NAD+MS alone doesn't create an entity (needs LOC+Z16 or NAD+DP)
        let nad = RawSegment::new(
            "NAD",
            vec![vec!["MS"], vec!["9900123000002", "", "293"]],
            pos(),
        );
        mapper.handle(&nad, &mut ctx);

        // Sparte is stored but no entity yet
        assert!(mapper.build().is_empty());

        // After reset, feed NAD+MS then LOC+Z16 → entity with sparte
        mapper.reset();
        mapper.handle(&nad, &mut ctx);
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO001"]], pos());
        mapper.handle(&loc, &mut ctx);

        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.sparte, Some("STROM".to_string()));
        assert_eq!(
            result[0].data.marktlokations_id,
            Some("MALO001".to_string())
        );
    }

    #[test]
    fn test_marktlokation_mapper_empty_returns_empty() {
        let mut mapper = MarktlokationMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }

    #[test]
    fn test_marktlokation_mapper_reset() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["DE001"]], pos());
        mapper.handle(&loc, &mut ctx);
        assert!(!mapper.is_empty());

        mapper.reset();
        assert!(mapper.is_empty());
    }
}
