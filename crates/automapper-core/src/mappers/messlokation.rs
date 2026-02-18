//! Mapper for Messlokation (metering location) business objects.
//!
//! Handles LOC+Z17 segments for metering location identification.

use bo4e_extensions::{Messlokation, MesslokationEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Messlokation in UTILMD messages.
pub struct MesslokationMapper {
    messlokations_id: Option<String>,
    edifact: MesslokationEdifact,
    has_data: bool,
}

impl MesslokationMapper {
    pub fn new() -> Self {
        Self {
            messlokations_id: None,
            edifact: MesslokationEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for MesslokationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for MesslokationMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z17"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.messlokations_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<Messlokation, MesslokationEdifact>>> for MesslokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Messlokation, MesslokationEdifact>> {
        if !self.has_data {
            return None;
        }
        let ml = Messlokation {
            messlokations_id: self.messlokations_id.take(),
            ..Default::default()
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: ml,
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
    fn test_messlokation_mapper_loc_z17() {
        let mut mapper = MesslokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new(
            "LOC",
            vec![vec!["Z17"], vec!["DE00098765432100000000000000012"]],
            pos(),
        );
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.messlokations_id,
            Some("DE00098765432100000000000000012".to_string())
        );
    }

    #[test]
    fn test_messlokation_mapper_ignores_z16() {
        let mapper = MesslokationMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["ID"]], pos());
        assert!(!mapper.can_handle(&loc));
    }
}
