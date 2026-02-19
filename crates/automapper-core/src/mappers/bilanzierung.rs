//! Mapper for Bilanzierung (balancing/settlement) business objects.
//!
//! Handles SEQ+Z98 and CCI/QTY segments for balancing data.
//! Minimal implementation covering modeled fields only.

use bo4e_extensions::{Bilanzierung, BilanzierungEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Bilanzierung in UTILMD messages.
///
/// Handles SEQ+Z98 (or Z81) for settlement/balancing data.
/// Subordinate segments:
/// - CCI+Z20: bilanzkreis
/// - QTY+Z09: jahresverbrauchsprognose
/// - QTY+265: temperatur_arbeit
pub struct BilanzierungMapper {
    bilanzkreis: Option<String>,
    regelzone: Option<String>,
    bilanzierungsgebiet: Option<String>,
    edifact: BilanzierungEdifact,
    has_data: bool,
    in_bilanzierung_seq: bool,
}

impl BilanzierungMapper {
    pub fn new() -> Self {
        Self {
            bilanzkreis: None,
            regelzone: None,
            bilanzierungsgebiet: None,
            edifact: BilanzierungEdifact::default(),
            has_data: false,
            in_bilanzierung_seq: false,
        }
    }
}

impl Default for BilanzierungMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for BilanzierungMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "SEQ" => {
                let q = segment.get_element(0);
                matches!(q, "Z98" | "Z81")
            }
            "CCI" | "CAV" | "QTY" => self.in_bilanzierung_seq,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                self.in_bilanzierung_seq = matches!(qualifier, "Z98" | "Z81");
            }
            "CCI" => {
                if !self.in_bilanzierung_seq {
                    return;
                }
                let first = segment.get_element(0);
                let code = segment.get_element(2);
                // CCI+Z20++bilanzkreis
                if first == "Z20" && !code.is_empty() {
                    self.bilanzkreis = Some(code.to_string());
                    self.has_data = true;
                }
                // CCI+Z21++regelzone
                if first == "Z21" && !code.is_empty() {
                    self.regelzone = Some(code.to_string());
                    self.has_data = true;
                }
                // CCI+Z22++bilanzierungsgebiet
                if first == "Z22" && !code.is_empty() {
                    self.bilanzierungsgebiet = Some(code.to_string());
                    self.has_data = true;
                }
            }
            "QTY" => {
                if !self.in_bilanzierung_seq {
                    return;
                }
                let qualifier = segment.get_component(0, 0);
                let value = segment.get_component(0, 1);
                if value.is_empty() {
                    return;
                }
                match qualifier {
                    "Z09" => {
                        if let Ok(v) = value.parse::<f64>() {
                            self.edifact.jahresverbrauchsprognose = Some(v);
                            self.has_data = true;
                        }
                    }
                    "265" => {
                        if let Ok(v) = value.parse::<f64>() {
                            self.edifact.temperatur_arbeit = Some(v);
                            self.has_data = true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl Builder<Option<WithValidity<Bilanzierung, BilanzierungEdifact>>> for BilanzierungMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Bilanzierung, BilanzierungEdifact>> {
        if !self.has_data {
            return None;
        }
        let b = Bilanzierung {
            bilanzkreis: self.bilanzkreis.take(),
            regelzone: self.regelzone.take(),
            bilanzierungsgebiet: self.bilanzierungsgebiet.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: b,
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
    fn test_bilanzierung_mapper_cci_z20_bilanzkreis() {
        let mut mapper = BilanzierungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z98"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new(
                "CCI",
                vec![vec!["Z20"], vec![], vec!["11YN20---------Z"]],
                pos(),
            ),
            &mut ctx,
        );
        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.bilanzkreis,
            Some("11YN20---------Z".to_string())
        );
    }

    #[test]
    fn test_bilanzierung_mapper_qty_jahresverbrauchsprognose() {
        let mut mapper = BilanzierungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z98"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("QTY", vec![vec!["Z09", "12345.67"]], pos()),
            &mut ctx,
        );
        let result = mapper.build().unwrap();
        assert!((result.edifact.jahresverbrauchsprognose.unwrap() - 12345.67).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bilanzierung_mapper_qty_temperatur_arbeit() {
        let mut mapper = BilanzierungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z81"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("QTY", vec![vec!["265", "9876.54"]], pos()),
            &mut ctx,
        );
        let result = mapper.build().unwrap();
        assert!((result.edifact.temperatur_arbeit.unwrap() - 9876.54).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bilanzierung_mapper_ignores_outside_seq() {
        let mut mapper = BilanzierungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("CCI", vec![vec!["Z20"], vec![], vec!["BK001"]], pos()),
            &mut ctx,
        );
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_bilanzierung_mapper_empty_build() {
        let mut mapper = BilanzierungMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
