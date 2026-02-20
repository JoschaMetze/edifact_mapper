//! Mapper for Netzlokation (network location) business objects.
//!
//! Handles LOC+Z18 segments for network location identification.

use bo4e_extensions::{Netzlokation, NetzlokationEdifact, WithValidity};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Netzlokation in UTILMD messages.
///
/// Supports multiple Netzlokationen per transaction. Each LOC+Z18 segment
/// creates a new entity.
pub struct NetzlokationMapper {
    netzlokations_id: Option<String>,
    edifact: NetzlokationEdifact,
    has_data: bool,
    items: Vec<WithValidity<Netzlokation, NetzlokationEdifact>>,
    delimiters: EdifactDelimiters,
}

impl NetzlokationMapper {
    pub fn new() -> Self {
        Self {
            netzlokations_id: None,
            edifact: NetzlokationEdifact::default(),
            has_data: false,
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
            let nl = Netzlokation {
                netzlokations_id: self.netzlokations_id.take(),
                ..Default::default()
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: nl,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for NetzlokationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for NetzlokationMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z18"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                // Finalize previous entity before starting a new one
                self.finalize_current();
                self.netzlokations_id = Some(id.to_string());
                // Store raw LOC for roundtrip fidelity (preserves subcomponents like +++Z01 or :::N)
                let raw = segment.to_raw_string(&self.delimiters);
                self.edifact.raw_loc = Some(raw);
                self.has_data = true;
            }
        }
    }
}

impl Builder<Vec<WithValidity<Netzlokation, NetzlokationEdifact>>> for NetzlokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Netzlokation, NetzlokationEdifact>> {
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
    fn test_netzlokation_mapper_loc_z18() {
        let mut mapper = NetzlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.netzlokations_id, Some("NELO001".to_string()));
    }

    #[test]
    fn test_netzlokation_mapper_multiple_z18() {
        let mut mapper = NetzlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO002"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].data.netzlokations_id, Some("NELO001".to_string()));
        assert_eq!(result[1].data.netzlokations_id, Some("NELO002".to_string()));
    }

    #[test]
    fn test_netzlokation_mapper_empty_build() {
        let mut mapper = NetzlokationMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }
}
