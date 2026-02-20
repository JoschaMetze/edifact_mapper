//! Mapper for Zaehler (meter) business objects.
//!
//! Handles SEQ+Z03 (device data) and SEQ+Z79 (product package) segments.

use bo4e_extensions::{WithValidity, Zaehler, ZaehlerEdifact};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Zaehler in UTILMD messages.
///
/// Handles SEQ+Z03 for meter device data and related CCI/PIA segments.
/// Supports multiple Zaehler per transaction.
pub struct ZaehlerMapper {
    zaehlertyp: Option<String>,
    sparte: Option<String>,
    edifact: ZaehlerEdifact,
    has_data: bool,
    in_seq_z03: bool,
    items: Vec<WithValidity<Zaehler, ZaehlerEdifact>>,
    delimiters: EdifactDelimiters,
}

impl ZaehlerMapper {
    pub fn new() -> Self {
        Self {
            zaehlertyp: None,
            sparte: None,
            edifact: ZaehlerEdifact::default(),
            has_data: false,
            in_seq_z03: false,
            items: Vec::new(),
            delimiters: EdifactDelimiters::default(),
        }
    }

    /// Set delimiters for raw segment serialization.
    pub fn set_delimiters(&mut self, delimiters: EdifactDelimiters) {
        self.delimiters = delimiters;
    }

    /// Finalizes the current Zaehler and pushes it to the items list.
    fn finalize_current(&mut self) {
        if self.has_data {
            let z = Zaehler {
                zaehlernummer: None,
                zaehlertyp: self.zaehlertyp.take(),
                sparte: self.sparte.take(),
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: z,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for ZaehlerMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for ZaehlerMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "SEQ" => {
                // Handle Z03/Z79, but also claim other SEQs to reset in_seq_z03
                let q = segment.get_element(0);
                matches!(q, "Z03" | "Z79") || self.in_seq_z03
            }
            "RFF" => {
                if !self.in_seq_z03 {
                    return false;
                }
                let q = segment.get_component(0, 0);
                matches!(q, "Z19" | "Z14")
            }
            "QTY" => self.in_seq_z03,
            "CCI" | "CAV" => self.in_seq_z03,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                self.in_seq_z03 = qualifier == "Z03";
                if qualifier == "Z03" {
                    // Finalize previous Zaehler before starting a new one
                    self.finalize_current();
                    let sub_id = segment.get_element(1);
                    if !sub_id.is_empty() {
                        self.edifact.seq_sub_id = Some(sub_id.to_string());
                    }
                    self.has_data = true;
                }
                if qualifier == "Z79" {
                    let ref_val = segment.get_element(1);
                    if !ref_val.is_empty() {
                        self.edifact.produktpaket_id = Some(ref_val.to_string());
                        // Don't set has_data — Z79 is handled by ProduktpaketMapper.
                        // Only SEQ+Z03 should trigger Zaehler creation.
                    }
                }
            }
            "RFF" => {
                let qualifier = segment.get_component(0, 0);
                let value = segment.get_component(0, 1);
                if value.is_empty() {
                    return;
                }
                match qualifier {
                    "Z19" => {
                        self.edifact.referenz_messlokation = Some(value.to_string());
                        self.has_data = true;
                    }
                    "Z14" => {
                        self.edifact.referenz_gateway = Some(value.to_string());
                        self.has_data = true;
                    }
                    _ => {}
                }
            }
            "QTY" => {
                if !self.in_seq_z03 {
                    return;
                }
                let raw = segment.to_raw_string(&self.delimiters);
                self.edifact.raw_qty.push(raw);
                self.has_data = true;
            }
            "CCI" | "CAV" => {
                // Store raw CCI/CAV for roundtrip fidelity
                let raw = segment.to_raw_string(&self.delimiters);
                self.edifact.raw_cci_cav.push(raw);
                self.has_data = true;
            }
            _ => {}
        }
    }
}

impl Builder<Vec<WithValidity<Zaehler, ZaehlerEdifact>>> for ZaehlerMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Zaehler, ZaehlerEdifact>> {
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
    fn test_zaehler_mapper_seq_z03_rff() {
        let mut mapper = ZaehlerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z19", "MELO001"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].edifact.referenz_messlokation,
            Some("MELO001".to_string())
        );
    }

    #[test]
    fn test_zaehler_mapper_multiple_z03() {
        let mut mapper = ZaehlerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        // First Zaehler
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z19", "MELO001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec![], vec![], vec!["E13"]], pos()),
            &mut ctx,
        );
        mapper.handle(&RawSegment::new("CAV", vec![vec!["EHZ"]], pos()), &mut ctx);
        // Second Zaehler
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z19", "MELO002"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec![], vec![], vec!["E13"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CAV", vec![vec!["MME", "", "", "Z04"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].edifact.referenz_messlokation,
            Some("MELO001".to_string())
        );
        assert_eq!(result[0].edifact.raw_cci_cav.len(), 2);
        assert_eq!(
            result[1].edifact.referenz_messlokation,
            Some("MELO002".to_string())
        );
        assert_eq!(result[1].edifact.raw_cci_cav.len(), 2);
    }

    #[test]
    fn test_zaehler_mapper_cci_cav_roundtrip() {
        let mut mapper = ZaehlerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("RFF", vec![vec!["Z19", "MELO002"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec![], vec![], vec!["E13"]], pos()),
            &mut ctx,
        );
        mapper.handle(&RawSegment::new("CAV", vec![vec!["AHZ"]], pos()), &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edifact.raw_cci_cav.len(), 2);
    }
}
