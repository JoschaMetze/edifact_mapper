//! Mapper for TechnischeRessource (technical resource) business objects.
//!
//! Handles LOC+Z20 segments for technical resource identification.

use bo4e_extensions::{TechnischeRessource, TechnischeRessourceEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for TechnischeRessource in UTILMD messages.
pub struct TechnischeRessourceMapper {
    technische_ressource_id: Option<String>,
    edifact: TechnischeRessourceEdifact,
    has_data: bool,
}

impl TechnischeRessourceMapper {
    pub fn new() -> Self {
        Self {
            technische_ressource_id: None,
            edifact: TechnischeRessourceEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for TechnischeRessourceMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for TechnischeRessourceMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z20"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.technische_ressource_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>>>
    for TechnischeRessourceMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(
        &mut self,
    ) -> Option<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>> {
        if !self.has_data {
            return None;
        }
        let tr = TechnischeRessource {
            technische_ressource_id: self.technische_ressource_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: tr,
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
    fn test_technische_ressource_mapper_loc_z20() {
        let mut mapper = TechnischeRessourceMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z20"], vec!["TECRES001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.technische_ressource_id,
            Some("TECRES001".to_string())
        );
    }

    #[test]
    fn test_technische_ressource_mapper_ignores_other_loc() {
        let mapper = TechnischeRessourceMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_technische_ressource_mapper_empty_build() {
        let mut mapper = TechnischeRessourceMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
