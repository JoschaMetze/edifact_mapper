//! Mapper for MabisZaehlpunkt (MaBiS metering point) business objects.
//!
//! Handles LOC+Z15 segments for MaBiS metering point identification.

use bo4e_extensions::{MabisZaehlpunkt, MabisZaehlpunktEdifact, WithValidity};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for MabisZaehlpunkt in UTILMD messages.
///
/// Supports multiple MabisZaehlpunkte per transaction. Each LOC+Z15 segment
/// creates a new entity.
pub struct MabisZaehlpunktMapper {
    zaehlpunkt_id: Option<String>,
    edifact: MabisZaehlpunktEdifact,
    has_data: bool,
    items: Vec<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>>,
    delimiters: EdifactDelimiters,
}

impl MabisZaehlpunktMapper {
    pub fn new() -> Self {
        Self {
            zaehlpunkt_id: None,
            edifact: MabisZaehlpunktEdifact::default(),
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
            let mz = MabisZaehlpunkt {
                zaehlpunkt_id: self.zaehlpunkt_id.take(),
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: mz,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for MabisZaehlpunktMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for MabisZaehlpunktMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z15"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                // Finalize previous entity before starting a new one
                self.finalize_current();
                self.zaehlpunkt_id = Some(id.to_string());
                self.edifact.raw_loc = Some(segment.to_raw_string(&self.delimiters));
                self.has_data = true;
            }
        }
    }
}

impl Builder<Vec<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>>> for MabisZaehlpunktMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>> {
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
    fn test_mabis_zaehlpunkt_mapper_loc_z15() {
        let mut mapper = MabisZaehlpunktMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z15"], vec!["MABIS001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.zaehlpunkt_id, Some("MABIS001".to_string()));
    }

    #[test]
    fn test_mabis_zaehlpunkt_mapper_multiple_z15() {
        let mut mapper = MabisZaehlpunktMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z15"], vec!["MABIS001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("LOC", vec![vec!["Z15"], vec!["MABIS002"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].data.zaehlpunkt_id, Some("MABIS001".to_string()));
        assert_eq!(result[1].data.zaehlpunkt_id, Some("MABIS002".to_string()));
    }

    #[test]
    fn test_mabis_zaehlpunkt_mapper_ignores_other_loc() {
        let mapper = MabisZaehlpunktMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_mabis_zaehlpunkt_mapper_empty_build() {
        let mut mapper = MabisZaehlpunktMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }
}
