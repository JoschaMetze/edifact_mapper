//! Mapper for Netzlokation (network location) business objects.
//!
//! Handles LOC+Z18 segments for network location identification.

use bo4e_extensions::{Netzlokation, NetzlokationEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Netzlokation in UTILMD messages.
pub struct NetzlokationMapper {
    netzlokations_id: Option<String>,
    edifact: NetzlokationEdifact,
    has_data: bool,
}

impl NetzlokationMapper {
    pub fn new() -> Self {
        Self {
            netzlokations_id: None,
            edifact: NetzlokationEdifact::default(),
            has_data: false,
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
                self.netzlokations_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<Netzlokation, NetzlokationEdifact>>> for NetzlokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Netzlokation, NetzlokationEdifact>> {
        if !self.has_data {
            return None;
        }
        let nl = Netzlokation {
            netzlokations_id: self.netzlokations_id.take(),
            ..Default::default()
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: nl,
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
    fn test_netzlokation_mapper_loc_z18() {
        let mut mapper = NetzlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.netzlokations_id, Some("NELO001".to_string()));
    }
}
