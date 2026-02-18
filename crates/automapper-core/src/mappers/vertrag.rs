//! Mapper for Vertrag (contract) business objects.
//!
//! Handles SEQ+Z18 group and CCI segments for contract data.

use bo4e_extensions::{Vertrag, VertragEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Vertrag in UTILMD messages.
pub struct VertragMapper {
    vertragsnummer: Option<String>,
    edifact: VertragEdifact,
    has_data: bool,
    in_seq_z18: bool,
}

impl VertragMapper {
    pub fn new() -> Self {
        Self {
            vertragsnummer: None,
            edifact: VertragEdifact::default(),
            has_data: false,
            in_seq_z18: false,
        }
    }
}

impl Default for VertragMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for VertragMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "SEQ" => true,
            "CCI" => self.in_seq_z18,
            "CAV" => self.in_seq_z18,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                self.in_seq_z18 = qualifier == "Z18";
            }
            "CCI" => {
                if !self.in_seq_z18 {
                    return;
                }
                let first = segment.get_element(0);
                let code2 = segment.get_element(2);
                // CCI+Z15++Z01/Z02 -> Haushaltskunde
                if first == "Z15" && !code2.is_empty() {
                    self.edifact.haushaltskunde = Some(code2 == "Z01");
                    self.has_data = true;
                }
                // CCI+Z36++code -> Versorgungsart
                if first == "Z36" && !code2.is_empty() {
                    self.edifact.versorgungsart = Some(code2.to_string());
                    self.has_data = true;
                }
            }
            _ => {}
        }
    }
}

impl Builder<Option<WithValidity<Vertrag, VertragEdifact>>> for VertragMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Vertrag, VertragEdifact>> {
        if !self.has_data {
            return None;
        }
        let v = Vertrag {
            vertragsnummer: self.vertragsnummer.take(),
            ..Default::default()
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: v,
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
    fn test_vertrag_mapper_seq_z18_haushaltskunde() {
        let mut mapper = VertragMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z18"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec!["Z15"], vec![], vec!["Z01"]], pos()),
            &mut ctx,
        );
        let result = mapper.build().unwrap();
        assert_eq!(result.edifact.haushaltskunde, Some(true));
    }

    #[test]
    fn test_vertrag_mapper_ignores_outside_z18() {
        let mut mapper = VertragMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z01"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec!["Z15"], vec![], vec!["Z01"]], pos()),
            &mut ctx,
        );
        assert!(mapper.is_empty());
    }
}
