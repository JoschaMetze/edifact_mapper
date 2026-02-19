//! Mapper for Lokationszuordnung (location assignment) business objects.
//!
//! Handles SEQ+Z78 group and RFF segments for location bundle references.

use bo4e_extensions::{Lokationszuordnung, LokationszuordnungEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Lokationszuordnung in UTILMD messages.
///
/// Handles SEQ+Z78 for location bundle structure references.
/// RFF segments within the Z78 context contain referenced location IDs.
pub struct LokationszuordnungMapper {
    marktlokations_id: Option<String>,
    messlokations_id: Option<String>,
    edifact: LokationszuordnungEdifact,
    has_data: bool,
    in_seq_z78: bool,
    items: Vec<WithValidity<Lokationszuordnung, LokationszuordnungEdifact>>,
}

impl LokationszuordnungMapper {
    pub fn new() -> Self {
        Self {
            marktlokations_id: None,
            messlokations_id: None,
            edifact: LokationszuordnungEdifact::default(),
            has_data: false,
            in_seq_z78: false,
            items: Vec::new(),
        }
    }

    /// Finalizes the current item (if any) and pushes it to the items list.
    fn finalize_current(&mut self) {
        if self.has_data {
            let lz = Lokationszuordnung {
                marktlokations_id: self.marktlokations_id.take(),
                messlokations_id: self.messlokations_id.take(),
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: lz,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for LokationszuordnungMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for LokationszuordnungMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "SEQ" => {
                let q = segment.get_element(0);
                q == "Z78"
            }
            "RFF" => {
                if !self.in_seq_z78 {
                    return false;
                }
                let q = segment.get_component(0, 0);
                matches!(q, "Z18" | "Z19")
            }
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                if qualifier == "Z78" {
                    self.finalize_current();
                    self.in_seq_z78 = true;
                    self.has_data = true;
                    // Extract zuordnungstyp from SEQ element if present
                    let ref_val = segment.get_element(1);
                    if !ref_val.is_empty() {
                        self.edifact.zuordnungstyp = Some(ref_val.to_string());
                    }
                } else {
                    self.in_seq_z78 = false;
                }
            }
            "RFF" => {
                if !self.in_seq_z78 {
                    return;
                }
                let qualifier = segment.get_component(0, 0);
                let value = segment.get_component(0, 1);
                if value.is_empty() {
                    return;
                }
                match qualifier {
                    "Z18" => {
                        self.marktlokations_id = Some(value.to_string());
                    }
                    "Z19" => {
                        self.messlokations_id = Some(value.to_string());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl Builder<Vec<WithValidity<Lokationszuordnung, LokationszuordnungEdifact>>>
    for LokationszuordnungMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Lokationszuordnung, LokationszuordnungEdifact>> {
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
    fn test_lokationszuordnung_mapper_seq_z78_with_rff() {
        let mut mapper = LokationszuordnungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z78"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z18", "MALO001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z19", "MELO001"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].data.marktlokations_id,
            Some("MALO001".to_string())
        );
        assert_eq!(result[0].data.messlokations_id, Some("MELO001".to_string()));
    }

    #[test]
    fn test_lokationszuordnung_mapper_ignores_rff_outside_z78() {
        let mut mapper = LokationszuordnungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z18", "MALO001"]], pos()),
            &mut ctx,
        );
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_lokationszuordnung_mapper_empty_build() {
        let mut mapper = LokationszuordnungMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }
}
