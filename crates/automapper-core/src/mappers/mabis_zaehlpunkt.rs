//! Mapper for MabisZaehlpunkt (MaBiS metering point) business objects.
//!
//! Handles LOC+Z15 segments for MaBiS metering point identification.

use bo4e_extensions::{MabisZaehlpunkt, MabisZaehlpunktEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for MabisZaehlpunkt in UTILMD messages.
pub struct MabisZaehlpunktMapper {
    zaehlpunkt_id: Option<String>,
    edifact: MabisZaehlpunktEdifact,
    has_data: bool,
}

impl MabisZaehlpunktMapper {
    pub fn new() -> Self {
        Self {
            zaehlpunkt_id: None,
            edifact: MabisZaehlpunktEdifact::default(),
            has_data: false,
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
                self.zaehlpunkt_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>>>
    for MabisZaehlpunktMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>> {
        if !self.has_data {
            return None;
        }
        let mz = MabisZaehlpunkt {
            zaehlpunkt_id: self.zaehlpunkt_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: mz,
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
    fn test_mabis_zaehlpunkt_mapper_loc_z15() {
        let mut mapper = MabisZaehlpunktMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z15"], vec!["MABIS001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.zaehlpunkt_id, Some("MABIS001".to_string()));
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
        assert!(mapper.build().is_none());
    }
}
