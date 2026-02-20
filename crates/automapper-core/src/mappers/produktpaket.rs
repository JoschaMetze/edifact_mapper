//! Mapper for Produktpaket (product package) business objects.
//!
//! Handles SEQ+Z79 group and PIA segments for product package data.

use bo4e_extensions::{Produktpaket, ProduktpaketEdifact, WithValidity};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Produktpaket in UTILMD messages.
///
/// Handles SEQ+Z79 for product package identification. PIA segments within
/// the Z79 context contain the product name.
///
/// Note: ZaehlerMapper also handles SEQ+Z79 for its own produktpaket_id
/// reference. Both mappers receive the segment; this is fine because
/// `route_to_mappers` sends to all matching handlers.
pub struct ProduktpaketMapper {
    produktpaket_id: Option<String>,
    edifact: ProduktpaketEdifact,
    has_data: bool,
    in_seq_z79: bool,
    items: Vec<WithValidity<Produktpaket, ProduktpaketEdifact>>,
    delimiters: EdifactDelimiters,
}

impl ProduktpaketMapper {
    pub fn new() -> Self {
        Self {
            produktpaket_id: None,
            edifact: ProduktpaketEdifact::default(),
            has_data: false,
            in_seq_z79: false,
            items: Vec::new(),
            delimiters: EdifactDelimiters::default(),
        }
    }

    /// Set delimiters for raw segment serialization.
    pub fn set_delimiters(&mut self, delimiters: EdifactDelimiters) {
        self.delimiters = delimiters;
    }

    /// Finalizes the current item (if any) and pushes it to the items list.
    fn finalize_current(&mut self) {
        if self.has_data {
            let pp = Produktpaket {
                produktpaket_id: self.produktpaket_id.take(),
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: pp,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for ProduktpaketMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for ProduktpaketMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "SEQ" => {
                let q = segment.get_element(0);
                matches!(q, "Z79" | "ZH0") || self.in_seq_z79
            }
            "PIA" | "CCI" | "CAV" => self.in_seq_z79,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                if matches!(qualifier, "Z79" | "ZH0") {
                    // Finalize previous item before starting new one
                    self.finalize_current();
                    self.in_seq_z79 = true;
                    self.edifact.seq_qualifier = Some(qualifier.to_string());
                    // Extract produktpaket_id from SEQ element if present
                    let ref_val = segment.get_element(1);
                    if !ref_val.is_empty() {
                        self.produktpaket_id = Some(ref_val.to_string());
                        self.has_data = true;
                    }
                } else {
                    self.in_seq_z79 = false;
                }
            }
            "PIA" => {
                if !self.in_seq_z79 {
                    return;
                }
                // PIA+5+name:typ' -> product name and type
                let qualifier = segment.get_element(0);
                if qualifier == "5" {
                    let name = segment.get_component(1, 0);
                    if !name.is_empty() {
                        self.edifact.produktpaket_name = Some(name.to_string());
                        let raw = segment.to_raw_string(&self.delimiters);
                        self.edifact.raw_pia = Some(raw);
                        self.has_data = true;
                    }
                }
            }
            "CCI" | "CAV" => {
                if !self.in_seq_z79 {
                    return;
                }
                let raw = segment.to_raw_string(&self.delimiters);
                self.edifact.raw_cci_cav.push(raw);
                self.has_data = true;
            }
            _ => {}
        }
    }
}

impl Builder<Vec<WithValidity<Produktpaket, ProduktpaketEdifact>>> for ProduktpaketMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Produktpaket, ProduktpaketEdifact>> {
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
    fn test_produktpaket_mapper_seq_z79_with_pia() {
        let mut mapper = ProduktpaketMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z79"], vec!["PP001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("PIA", vec![vec!["5"], vec!["Grundversorgung"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.produktpaket_id, Some("PP001".to_string()));
        assert_eq!(
            result[0].edifact.produktpaket_name,
            Some("Grundversorgung".to_string())
        );
    }

    #[test]
    fn test_produktpaket_mapper_ignores_pia_outside_z79() {
        let mut mapper = ProduktpaketMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        // Set context to Z03 (not Z79)
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("PIA", vec![vec!["5"], vec!["SomeProduct"]], pos()),
            &mut ctx,
        );
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_produktpaket_mapper_empty_build() {
        let mut mapper = ProduktpaketMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }

    #[test]
    fn test_produktpaket_mapper_seq_z79_no_pia() {
        let mut mapper = ProduktpaketMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z79"], vec!["PP002"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.produktpaket_id, Some("PP002".to_string()));
        assert!(result[0].edifact.produktpaket_name.is_none());
    }
}
