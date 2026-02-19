//! Mapper for Tranche business objects.
//!
//! Handles LOC+Z21 segments for tranche identification.

use bo4e_extensions::{Tranche, TrancheEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Tranche in UTILMD messages.
pub struct TrancheMapper {
    tranche_id: Option<String>,
    edifact: TrancheEdifact,
    has_data: bool,
}

impl TrancheMapper {
    pub fn new() -> Self {
        Self {
            tranche_id: None,
            edifact: TrancheEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for TrancheMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for TrancheMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z21"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.tranche_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<Tranche, TrancheEdifact>>> for TrancheMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Tranche, TrancheEdifact>> {
        if !self.has_data {
            return None;
        }
        let t = Tranche {
            tranche_id: self.tranche_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: t,
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
    fn test_tranche_mapper_loc_z21() {
        let mut mapper = TrancheMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z21"], vec!["TRANCHE001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.tranche_id, Some("TRANCHE001".to_string()));
    }

    #[test]
    fn test_tranche_mapper_ignores_other_loc() {
        let mapper = TrancheMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_tranche_mapper_empty_build() {
        let mut mapper = TrancheMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
