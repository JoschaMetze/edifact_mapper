//! Mapper for SteuerbareRessource (controllable resource) business objects.
//!
//! Handles LOC+Z19 segments for controllable resource identification.

use bo4e_extensions::{SteuerbareRessource, SteuerbareRessourceEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for SteuerbareRessource in UTILMD messages.
pub struct SteuerbareRessourceMapper {
    steuerbare_ressource_id: Option<String>,
    edifact: SteuerbareRessourceEdifact,
    has_data: bool,
}

impl SteuerbareRessourceMapper {
    pub fn new() -> Self {
        Self {
            steuerbare_ressource_id: None,
            edifact: SteuerbareRessourceEdifact::default(),
            has_data: false,
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
                self.steuerbare_ressource_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>>
    for SteuerbareRessourceMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(
        &mut self,
    ) -> Option<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>> {
        if !self.has_data {
            return None;
        }
        let sr = SteuerbareRessource {
            steuerbare_ressource_id: self.steuerbare_ressource_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: sr,
            edifact,
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        })
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
        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.steuerbare_ressource_id,
            Some("STRES001".to_string())
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
        assert!(mapper.build().is_none());
    }
}
