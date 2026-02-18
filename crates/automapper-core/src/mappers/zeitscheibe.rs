//! Mapper for Zeitscheibe (time slice) data.
//!
//! Handles:
//! - RFF+Z49: Zeitscheiben-Referenz (time slice reference)
//! - RFF+Z50: Zeitscheiben-Referenz (alternate)
//! - RFF+Z53: Zeitscheiben-Referenz (alternate)
//! - DTM+Z25/DTM+92: Zeitscheibe start date
//! - DTM+Z26/DTM+93: Zeitscheibe end date
//!
//! Produces: `Vec<Zeitscheibe>` from bo4e-extensions.

use bo4e_extensions::{Zeitraum, Zeitscheibe};
use chrono::NaiveDateTime;
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for UTILMD Zeitscheibe (time slice) data.
///
/// A UTILMD transaction can contain multiple Zeitscheiben (time slices),
/// each identified by an RFF+Z49/Z50/Z53 reference and bounded by
/// DTM+Z25/Z26 date segments. Entity objects (Marktlokation, Zaehler, etc.)
/// reference Zeitscheiben via their SEQ segment's second element.
pub struct ZeitscheibeMapper {
    zeitscheiben: Vec<Zeitscheibe>,
    current_id: Option<String>,
    current_von: Option<NaiveDateTime>,
    current_bis: Option<NaiveDateTime>,
    has_data: bool,
}

impl ZeitscheibeMapper {
    /// Creates a new ZeitscheibeMapper.
    pub fn new() -> Self {
        Self {
            zeitscheiben: Vec::new(),
            current_id: None,
            current_von: None,
            current_bis: None,
            has_data: false,
        }
    }

    /// Parses a DTM date value (same logic as ProzessdatenMapper).
    fn parse_dtm(value: &str, format_code: &str) -> Option<NaiveDateTime> {
        match format_code {
            "303" => {
                let clean = if let Some(pos) = value.find('?') {
                    &value[..pos]
                } else {
                    value
                };
                if clean.len() >= 12 {
                    NaiveDateTime::parse_from_str(&clean[..12], "%Y%m%d%H%M").ok()
                } else {
                    None
                }
            }
            "102" => {
                if value.len() >= 8 {
                    NaiveDateTime::parse_from_str(
                        &format!("{}0000", &value[..8]),
                        "%Y%m%d%H%M",
                    )
                    .ok()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Finalizes the current Zeitscheibe and starts a new one.
    fn finalize_current(&mut self) {
        if let Some(id) = self.current_id.take() {
            let zeitraum = if self.current_von.is_some() || self.current_bis.is_some() {
                Some(Zeitraum::new(self.current_von, self.current_bis))
            } else {
                None
            };
            self.zeitscheiben.push(Zeitscheibe {
                zeitscheiben_id: id,
                gueltigkeitszeitraum: zeitraum,
            });
        }
        self.current_von = None;
        self.current_bis = None;
    }

    fn handle_rff(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_component(0, 0);
        let value = segment.get_component(0, 1);

        if value.is_empty() {
            return;
        }

        match qualifier {
            "Z49" | "Z50" | "Z53" => {
                // New Zeitscheibe reference -- finalize previous if any
                self.finalize_current();
                self.current_id = Some(value.to_string());
                self.has_data = true;
            }
            _ => {}
        }
    }

    fn handle_dtm(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_component(0, 0);
        let value = segment.get_component(0, 1);
        let format_code = segment.get_component(0, 2);

        if value.is_empty() || self.current_id.is_none() {
            return;
        }

        let parsed = Self::parse_dtm(value, format_code);

        match qualifier {
            "Z25" | "92" => self.current_von = parsed,
            "Z26" | "93" => self.current_bis = parsed,
            _ => {}
        }
    }
}

impl Default for ZeitscheibeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for ZeitscheibeMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "RFF" => {
                let q = segment.get_component(0, 0);
                matches!(q, "Z49" | "Z50" | "Z53")
            }
            "DTM" => {
                // Only handle DTM in Zeitscheibe context (after RFF+Z49/Z50/Z53)
                if self.current_id.is_none() {
                    return false;
                }
                let q = segment.get_component(0, 0);
                matches!(q, "Z25" | "Z26" | "92" | "93")
            }
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "RFF" => self.handle_rff(segment),
            "DTM" => self.handle_dtm(segment),
            _ => {}
        }
    }
}

impl Builder<Vec<Zeitscheibe>> for ZeitscheibeMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Vec<Zeitscheibe> {
        // Finalize any pending Zeitscheibe
        self.finalize_current();
        let result = std::mem::take(&mut self.zeitscheiben);
        self.has_data = false;
        result
    }

    fn reset(&mut self) {
        self.zeitscheiben.clear();
        self.current_id = None;
        self.current_von = None;
        self.current_bis = None;
        self.has_data = false;
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
    fn test_zeitscheibe_mapper_single() {
        let mut mapper = ZeitscheibeMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let rff = RawSegment::new("RFF", vec![vec!["Z49", "1"]], pos());
        let dtm_von = RawSegment::new("DTM", vec![vec!["Z25", "202507010000", "303"]], pos());
        let dtm_bis = RawSegment::new("DTM", vec![vec!["Z26", "202512310000", "303"]], pos());

        assert!(mapper.can_handle(&rff));
        mapper.handle(&rff, &mut ctx);
        mapper.handle(&dtm_von, &mut ctx);
        mapper.handle(&dtm_bis, &mut ctx);

        let zs = mapper.build();
        assert_eq!(zs.len(), 1);
        assert_eq!(zs[0].zeitscheiben_id, "1");
        assert!(zs[0].gueltigkeitszeitraum.is_some());
        let gz = zs[0].gueltigkeitszeitraum.as_ref().unwrap();
        assert!(gz.von.is_some());
        assert!(gz.bis.is_some());
    }

    #[test]
    fn test_zeitscheibe_mapper_multiple() {
        let mut mapper = ZeitscheibeMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // First Zeitscheibe
        mapper.handle(&RawSegment::new("RFF", vec![vec!["Z49", "1"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("DTM", vec![vec!["Z25", "202507010000", "303"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("DTM", vec![vec!["Z26", "202509300000", "303"]], pos()),
            &mut ctx,
        );

        // Second Zeitscheibe
        mapper.handle(&RawSegment::new("RFF", vec![vec!["Z49", "2"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("DTM", vec![vec!["Z25", "202510010000", "303"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("DTM", vec![vec!["Z26", "202512310000", "303"]], pos()),
            &mut ctx,
        );

        let zs = mapper.build();
        assert_eq!(zs.len(), 2);
        assert_eq!(zs[0].zeitscheiben_id, "1");
        assert_eq!(zs[1].zeitscheiben_id, "2");
    }

    #[test]
    fn test_zeitscheibe_mapper_rff_z50() {
        let mut mapper = ZeitscheibeMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        mapper.handle(&RawSegment::new("RFF", vec![vec!["Z50", "A"]], pos()), &mut ctx);

        let zs = mapper.build();
        assert_eq!(zs.len(), 1);
        assert_eq!(zs[0].zeitscheiben_id, "A");
    }

    #[test]
    fn test_zeitscheibe_mapper_reset() {
        let mut mapper = ZeitscheibeMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        mapper.handle(&RawSegment::new("RFF", vec![vec!["Z49", "1"]], pos()), &mut ctx);
        assert!(!mapper.is_empty());

        mapper.reset();
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_zeitscheibe_mapper_dtm_ignored_without_rff() {
        let mapper = ZeitscheibeMapper::new();

        let dtm = RawSegment::new("DTM", vec![vec!["Z25", "202507010000", "303"]], pos());
        assert!(!mapper.can_handle(&dtm));
    }
}
