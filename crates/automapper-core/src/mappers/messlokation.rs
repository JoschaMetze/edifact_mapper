//! Mapper for Messlokation (metering location) business objects.
//!
//! Handles LOC+Z17 segments for metering location identification.

use bo4e_extensions::{Messlokation, MesslokationEdifact, WithValidity};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Messlokation in UTILMD messages.
///
/// Supports multiple Messlokationen per transaction. Each LOC+Z17 segment
/// creates a new entity.
pub struct MesslokationMapper {
    messlokations_id: Option<String>,
    edifact: MesslokationEdifact,
    has_data: bool,
    items: Vec<WithValidity<Messlokation, MesslokationEdifact>>,
    delimiters: EdifactDelimiters,
}

impl MesslokationMapper {
    pub fn new() -> Self {
        Self {
            messlokations_id: None,
            edifact: MesslokationEdifact::default(),
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
            let ml = Messlokation {
                messlokations_id: self.messlokations_id.take(),
                ..Default::default()
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
                // Finalize previous entity before starting a new one
                self.finalize_current();
                self.messlokations_id = Some(id.to_string());
                let raw = segment.to_raw_string(&self.delimiters);
                self.edifact.raw_loc = Some(raw);
                self.has_data = true;
            }
        }
    }
}

impl Builder<Vec<WithValidity<Messlokation, MesslokationEdifact>>> for MesslokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Messlokation, MesslokationEdifact>> {
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
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].data.messlokations_id,
            Some("DE00098765432100000000000000012".to_string())
        );
    }

    #[test]
    fn test_messlokation_mapper_multiple_z17() {
        let mut mapper = MesslokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z17"], vec!["MELO001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z17"], vec!["MELO002"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].data.messlokations_id, Some("MELO001".to_string()));
        assert_eq!(result[1].data.messlokations_id, Some("MELO002".to_string()));
    }

    #[test]
    fn test_messlokation_mapper_ignores_z16() {
        let mapper = MesslokationMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["ID"]], pos());
        assert!(!mapper.can_handle(&loc));
    }
}
