//! Mapper for Zaehler (meter) business objects.
//!
//! Handles SEQ+Z03 (device data) and SEQ+Z79 (product package) segments.

use bo4e_extensions::{WithValidity, Zaehler, ZaehlerEdifact};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Zaehler in UTILMD messages.
///
/// Handles SEQ+Z03 for meter device data and related CCI/PIA segments.
pub struct ZaehlerMapper {
    zaehlernummer: Option<String>,
    zaehlertyp: Option<String>,
    sparte: Option<String>,
    edifact: ZaehlerEdifact,
    has_data: bool,
    in_seq_z03: bool,
}

impl ZaehlerMapper {
    pub fn new() -> Self {
        Self {
            zaehlernummer: None,
            zaehlertyp: None,
            sparte: None,
            edifact: ZaehlerEdifact::default(),
            has_data: false,
            in_seq_z03: false,
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
                let q = segment.get_element(0);
                matches!(q, "Z03" | "Z79")
            }
            "RFF" => {
                if !self.in_seq_z03 {
                    return false;
                }
                let q = segment.get_component(0, 0);
                matches!(q, "Z19" | "Z14")
            }
            "PIA" => self.in_seq_z03,
            "CCI" => self.in_seq_z03,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                self.in_seq_z03 = qualifier == "Z03";
                if qualifier == "Z79" {
                    let ref_val = segment.get_element(1);
                    if !ref_val.is_empty() {
                        self.edifact.produktpaket_id = Some(ref_val.to_string());
                        self.has_data = true;
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
            "PIA" => {
                // PIA+5+zaehlernummer:codeList'
                let qualifier = segment.get_element(0);
                if qualifier == "5" {
                    let nummer = segment.get_component(1, 0);
                    if !nummer.is_empty() {
                        self.zaehlernummer = Some(nummer.to_string());
                        self.has_data = true;
                    }
                }
            }
            _ => {}
        }
    }
}

impl Builder<Vec<WithValidity<Zaehler, ZaehlerEdifact>>> for ZaehlerMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Vec<WithValidity<Zaehler, ZaehlerEdifact>> {
        if !self.has_data {
            return Vec::new();
        }
        let z = Zaehler {
            zaehlernummer: self.zaehlernummer.take(),
            zaehlertyp: self.zaehlertyp.take(),
            sparte: self.sparte.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        vec![WithValidity {
            data: z,
            edifact,
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        }]
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
    fn test_zaehler_mapper_pia_zaehlernummer() {
        let mut mapper = ZaehlerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("PIA", vec![vec!["5"], vec!["ZAEHLER001"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.zaehlernummer, Some("ZAEHLER001".to_string()));
    }
}
