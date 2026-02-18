//! Mapper for process-level data (STS, DTM, RFF, FTX at transaction level).
//!
//! Handles:
//! - STS segments: transaction reason and status
//! - DTM segments: process dates (DTM+137, DTM+471, DTM+Z25, DTM+Z26, etc.)
//! - RFF segments: reference numbers (RFF+Z13, RFF+Z14, etc.)
//! - FTX segments: free text remarks (FTX+ACB)
//!
//! Produces: `Prozessdaten` from bo4e-extensions.

use bo4e_extensions::Prozessdaten;
use chrono::NaiveDateTime;
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for UTILMD process-level data.
///
/// This mapper handles STS, DTM, RFF, and FTX segments at the transaction
/// level (outside of entity-specific SEQ groups). It populates the
/// `Prozessdaten` struct with dates, references, and status information.
pub struct ProzessdatenMapper {
    prozessdaten: Prozessdaten,
    has_data: bool,
}

impl ProzessdatenMapper {
    /// Creates a new ProzessdatenMapper.
    pub fn new() -> Self {
        Self {
            prozessdaten: Prozessdaten::default(),
            has_data: false,
        }
    }

    /// Parses a DTM date value in EDIFACT format.
    ///
    /// Supports formats:
    /// - `303`: `CCYYMMDDHHmm` (12 chars) with optional timezone
    /// - `102`: `CCYYMMDD` (8 chars)
    fn parse_dtm_value(value: &str, format_code: &str) -> Option<NaiveDateTime> {
        match format_code {
            "303" => {
                // Strip timezone suffix like ?+00 if present
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

    fn handle_sts(&mut self, segment: &RawSegment) {
        // STS+E01+transaktionsgrund::Z44+ergaenzung' or STS+7+grund::codeList'
        // Element 0: status type (E01 = process status)
        // Element 1: composite with transaction reason
        //   - Component 0: code
        //   - Component 2: code list qualifier
        let grund_code = segment.get_component(1, 0);
        if !grund_code.is_empty() {
            self.prozessdaten.transaktionsgrund = Some(grund_code.to_string());
            self.has_data = true;
        }

        // Element 2: supplementary reason
        let ergaenzung = segment.get_component(2, 0);
        if !ergaenzung.is_empty() {
            self.prozessdaten.transaktionsgrund_ergaenzung = Some(ergaenzung.to_string());
        }
    }

    fn handle_dtm(&mut self, segment: &RawSegment) {
        // DTM+qualifier:value:format'
        // C507 composite: element 0
        //   - Component 0: qualifier (137, 471, Z25, Z26, etc.)
        //   - Component 1: date value
        //   - Component 2: format code (303, 102)
        let qualifier = segment.get_component(0, 0);
        let value = segment.get_component(0, 1);
        let format_code = segment.get_component(0, 2);

        if value.is_empty() {
            return;
        }

        let parsed = Self::parse_dtm_value(value, format_code);

        match qualifier {
            "137" => self.prozessdaten.prozessdatum = parsed,
            "471" => self.prozessdaten.wirksamkeitsdatum = parsed,
            "Z25" | "92" => self.prozessdaten.vertragsbeginn = parsed,
            "Z26" | "93" => self.prozessdaten.vertragsende = parsed,
            "Z42" => self.prozessdaten.lieferbeginndatum_in_bearbeitung = parsed,
            "Z43" => self.prozessdaten.datum_naechste_bearbeitung = parsed,
            "Z51" => self.prozessdaten.tag_des_empfangs = parsed,
            "Z52" => self.prozessdaten.kuendigungsdatum_kunde = parsed,
            "Z53" => self.prozessdaten.geplanter_liefertermin = parsed,
            _ => return, // Ignore unknown qualifiers
        }
        self.has_data = true;
    }

    fn handle_rff(&mut self, segment: &RawSegment) {
        // RFF+qualifier:value'
        // C506 composite: element 0
        //   - Component 0: qualifier (Z13, Z14, etc.)
        //   - Component 1: reference value
        let qualifier = segment.get_component(0, 0);
        let value = segment.get_component(0, 1);

        if value.is_empty() {
            return;
        }

        match qualifier {
            "Z13" => {
                self.prozessdaten.referenz_vorgangsnummer = Some(value.to_string());
                self.has_data = true;
            }
            "Z14" => {
                self.prozessdaten.anfrage_referenz = Some(value.to_string());
                self.has_data = true;
            }
            _ => {} // Other RFF qualifiers handled by other mappers
        }
    }

    fn handle_ftx(&mut self, segment: &RawSegment) {
        // FTX+ACB+++text1:text2:text3:text4:text5'
        // Element 0: text subject qualifier (ACB = additional information)
        // Element 3: C108 composite with text lines
        //   - Components 0-4: text lines
        let qualifier = segment.get_element(0);
        if qualifier != "ACB" {
            return;
        }

        let text = segment.get_component(3, 0);
        if !text.is_empty() {
            self.prozessdaten.bemerkung = Some(text.to_string());
            self.has_data = true;
        }
    }
}

impl Default for ProzessdatenMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for ProzessdatenMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        matches!(segment.id, "STS" | "DTM" | "RFF" | "FTX")
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "STS" => self.handle_sts(segment),
            "DTM" => self.handle_dtm(segment),
            "RFF" => self.handle_rff(segment),
            "FTX" => self.handle_ftx(segment),
            _ => {}
        }
    }
}

impl Builder<Prozessdaten> for ProzessdatenMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Prozessdaten {
        let result = std::mem::take(&mut self.prozessdaten);
        self.has_data = false;
        result
    }

    fn reset(&mut self) {
        self.prozessdaten = Prozessdaten::default();
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
    fn test_prozessdaten_mapper_sts() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let sts = RawSegment::new(
            "STS",
            vec![
                vec!["E01"],
                vec!["E01", "", "Z44"],
                vec!["Z01"],
            ],
            pos(),
        );

        assert!(mapper.can_handle(&sts));
        mapper.handle(&sts, &mut ctx);

        assert!(!mapper.is_empty());
        let pd = mapper.build();
        assert_eq!(pd.transaktionsgrund, Some("E01".to_string()));
        assert_eq!(pd.transaktionsgrund_ergaenzung, Some("Z01".to_string()));
    }

    #[test]
    fn test_prozessdaten_mapper_dtm_137() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let dtm = RawSegment::new(
            "DTM",
            vec![vec!["137", "202506190130", "303"]],
            pos(),
        );

        mapper.handle(&dtm, &mut ctx);

        let pd = mapper.build();
        assert!(pd.prozessdatum.is_some());
        let dt = pd.prozessdatum.unwrap();
        assert_eq!(dt.format("%Y%m%d%H%M").to_string(), "202506190130");
    }

    #[test]
    fn test_prozessdaten_mapper_dtm_with_timezone() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // DTM+137:202506190130?+00:303'
        let dtm = RawSegment::new(
            "DTM",
            vec![vec!["137", "202506190130?+00", "303"]],
            pos(),
        );

        mapper.handle(&dtm, &mut ctx);

        let pd = mapper.build();
        assert!(pd.prozessdatum.is_some());
    }

    #[test]
    fn test_prozessdaten_mapper_dtm_102_format() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let dtm = RawSegment::new(
            "DTM",
            vec![vec!["471", "20250701", "102"]],
            pos(),
        );

        mapper.handle(&dtm, &mut ctx);

        let pd = mapper.build();
        assert!(pd.wirksamkeitsdatum.is_some());
    }

    #[test]
    fn test_prozessdaten_mapper_rff() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let rff = RawSegment::new(
            "RFF",
            vec![vec!["Z13", "VORGANGSNUMMER001"]],
            pos(),
        );

        mapper.handle(&rff, &mut ctx);

        let pd = mapper.build();
        assert_eq!(
            pd.referenz_vorgangsnummer,
            Some("VORGANGSNUMMER001".to_string())
        );
    }

    #[test]
    fn test_prozessdaten_mapper_ftx() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let ftx = RawSegment::new(
            "FTX",
            vec![vec!["ACB"], vec![], vec![], vec!["Test remark"]],
            pos(),
        );

        mapper.handle(&ftx, &mut ctx);

        let pd = mapper.build();
        assert_eq!(pd.bemerkung, Some("Test remark".to_string()));
    }

    #[test]
    fn test_prozessdaten_mapper_reset() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let sts = RawSegment::new("STS", vec![vec!["E01"], vec!["E01"]], pos());
        mapper.handle(&sts, &mut ctx);
        assert!(!mapper.is_empty());

        mapper.reset();
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_prozessdaten_mapper_ignores_unknown_dtm() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let dtm = RawSegment::new(
            "DTM",
            vec![vec!["999", "20250701", "102"]],
            pos(),
        );

        mapper.handle(&dtm, &mut ctx);
        assert!(mapper.is_empty());
    }
}
