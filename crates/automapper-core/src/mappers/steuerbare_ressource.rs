//! Mapper for SteuerbareRessource (controllable resource) business objects.
//!
//! Handles LOC+Z19 segments for controllable resource identification.

use bo4e_extensions::{SteuerbareRessource, SteuerbareRessourceEdifact, WithValidity};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for SteuerbareRessource in UTILMD messages.
///
/// Supports multiple SteuerbareRessourcen per transaction. Each LOC+Z19 segment
/// creates a new entity.
pub struct SteuerbareRessourceMapper {
    steuerbare_ressource_id: Option<String>,
    edifact: SteuerbareRessourceEdifact,
    has_data: bool,
    items: Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>,
    delimiters: EdifactDelimiters,
}

impl SteuerbareRessourceMapper {
    pub fn new() -> Self {
        Self {
            steuerbare_ressource_id: None,
            edifact: SteuerbareRessourceEdifact::default(),
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
            let sr = SteuerbareRessource {
                steuerbare_ressource_id: self.steuerbare_ressource_id.take(),
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: sr,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for SteuerbareRessourceMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for SteuerbareRessourceMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z19"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                // Finalize previous entity before starting a new one
                self.finalize_current();
                self.steuerbare_ressource_id = Some(id.to_string());
                self.edifact.raw_loc = Some(segment.to_raw_string(&self.delimiters));
                self.has_data = true;
            }
        }
    }
}

impl Builder<Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>>
    for SteuerbareRessourceMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>> {
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
    fn test_steuerbare_ressource_mapper_loc_z19() {
        let mut mapper = SteuerbareRessourceMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z19"], vec!["STRES001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].data.steuerbare_ressource_id,
            Some("STRES001".to_string())
        );
    }

    #[test]
    fn test_steuerbare_ressource_mapper_multiple_z19() {
        let mut mapper = SteuerbareRessourceMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z19"], vec!["STRES001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z19"], vec!["STRES002"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].data.steuerbare_ressource_id,
            Some("STRES001".to_string())
        );
        assert_eq!(
            result[1].data.steuerbare_ressource_id,
            Some("STRES002".to_string())
        );
    }

    #[test]
    fn test_steuerbare_ressource_mapper_ignores_other_loc() {
        let mapper = SteuerbareRessourceMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_steuerbare_ressource_mapper_empty_build() {
        let mut mapper = SteuerbareRessourceMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }
}
