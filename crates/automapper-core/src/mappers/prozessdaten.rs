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

/// Parse a DTM date/time value with format code into NaiveDateTime.
/// Supports format codes 303 (CCYYMMDDHHmm) and 102 (CCYYMMDD).
pub(crate) fn parse_edifact_dtm(value: &str, format_code: &str) -> Option<NaiveDateTime> {
    match format_code {
        // 303: CCYYMMDDHHmm (optionally with ?+00 timezone suffix)
        // 203: CCYYMMDDHHmm (same format, different semantic)
        "303" | "203" => {
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
                NaiveDateTime::parse_from_str(&format!("{}0000", &value[..8]), "%Y%m%d%H%M").ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Mapper for UTILMD process-level data.
///
/// This mapper handles STS, DTM, RFF, and FTX segments at the transaction
/// level (outside of entity-specific SEQ groups). It populates the
/// `Prozessdaten` struct with dates, references, and status information.
pub struct ProzessdatenMapper {
    prozessdaten: Prozessdaten,
    has_data: bool,
    delimiters: edifact_types::EdifactDelimiters,
    /// Once a SEQ segment is seen, stop claiming RFF/DTM (they belong to SEQ groups).
    in_seq_zone: bool,
    /// Once RFF+Z47/Z48/Z49/Z50/Z53 is seen, DTM+Z25/Z26 belong to Zeitscheibe, not process level.
    in_zeitscheibe_zone: bool,
}

impl ProzessdatenMapper {
    /// Creates a new ProzessdatenMapper.
    pub fn new() -> Self {
        Self {
            prozessdaten: Prozessdaten::default(),
            has_data: false,
            delimiters: edifact_types::EdifactDelimiters::default(),
            in_seq_zone: false,
            in_zeitscheibe_zone: false,
        }
    }

    /// Called by the coordinator to notify that a SEQ segment has been seen.
    pub fn notify_seq_entered(&mut self) {
        self.in_seq_zone = true;
    }

    /// Set the delimiters for raw segment serialization.
    pub fn set_delimiters(&mut self, delimiters: edifact_types::EdifactDelimiters) {
        self.delimiters = delimiters;
    }

    /// Set LOC+Z22 schlafende Marktlokation ID.
    pub fn set_schlafende_marktlokation(&mut self, id: &str) {
        self.prozessdaten.schlafende_marktlokation_id = Some(id.to_string());
        self.has_data = true;
    }

    /// Set NAD+VY andere Partei MP-ID and code qualifier.
    pub fn set_andere_partei(&mut self, mp_id: &str, code_qualifier: &str) {
        self.prozessdaten.andere_partei_mp_id = Some(mp_id.to_string());
        if !code_qualifier.is_empty() {
            self.prozessdaten.andere_partei_code_qualifier = Some(code_qualifier.to_string());
        }
        self.has_data = true;
    }

    /// Set RFF segment following NAD+VY.
    pub fn set_andere_partei_rff(&mut self, raw: &str) {
        self.prozessdaten.andere_partei_rff = Some(raw.to_string());
        self.has_data = true;
    }

    /// Parses a DTM date value in EDIFACT format.
    ///
    /// Delegates to the standalone [`parse_edifact_dtm`] function.
    fn parse_dtm_value(value: &str, format_code: &str) -> Option<NaiveDateTime> {
        parse_edifact_dtm(value, format_code)
    }

    fn handle_sts(&mut self, segment: &RawSegment, delimiters: &edifact_types::EdifactDelimiters) {
        let el0 = segment.get_element(0);

        if el0 == "7" {
            // S2.1 format: STS+7++grund+ergaenzung+befristete_anmeldung
            let grund = segment.get_component(2, 0);
            if !grund.is_empty() {
                self.prozessdaten.transaktionsgrund = Some(grund.to_string());
                self.has_data = true;
            }
            let ergaenzung = segment.get_component(3, 0);
            if !ergaenzung.is_empty() {
                self.prozessdaten.transaktionsgrund_ergaenzung = Some(ergaenzung.to_string());
            }
            let befristet = segment.get_component(4, 0);
            if !befristet.is_empty() {
                self.prozessdaten
                    .transaktionsgrund_ergaenzung_befristete_anmeldung =
                    Some(befristet.to_string());
            }
        } else if el0 == "E01" || el0 == "Z17" || el0 == "Z18" || el0 == "Z35" {
            // STS+E01/Z17/Z18/Z35: Store raw for roundtrip fidelity
            let raw = segment.to_raw_string(delimiters);
            self.prozessdaten.sts_raw.push(raw);
            self.has_data = true;
        } else {
            // Legacy format: STS+grund+composite+ergaenzung
            let grund_code = segment.get_component(1, 0);
            if !grund_code.is_empty() {
                self.prozessdaten.transaktionsgrund = Some(grund_code.to_string());
                self.has_data = true;
            }
            let ergaenzung = segment.get_component(2, 0);
            if !ergaenzung.is_empty() {
                self.prozessdaten.transaktionsgrund_ergaenzung = Some(ergaenzung.to_string());
            }
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

        // Z25/Z26/752 after Zeitscheibe zone (RFF+Z49/Z50/Z52/Z53/Z47) — attach to last zeitscheibe ref
        if self.in_zeitscheibe_zone && matches!(qualifier, "Z25" | "Z26" | "752") {
            let raw = segment.to_raw_string(&self.delimiters);
            if let Some(last_ref) = self.prozessdaten.zeitscheibe_refs.last_mut() {
                last_ref.raw_dtms.push(raw);
            }
            self.has_data = true;
            return;
        }

        // Store DTM composite value keyed by qualifier for roundtrip fidelity
        let raw = if format_code.is_empty() {
            value.to_string()
        } else {
            format!("{}:{}", value, format_code)
        };
        self.prozessdaten.raw_dtm.insert(qualifier.to_string(), raw);
        // Store ordered raw segment string for process-level DTMs only.
        // Reference-section DTMs (155, 752, 672, Z20, Z21, Z09, Z22) are written
        // separately via REFERENCE_DTM_ORDER and should NOT be replayed here.
        if !matches!(
            qualifier,
            "155" | "752" | "672" | "Z20" | "Z21" | "Z09" | "Z22"
        ) {
            self.prozessdaten
                .raw_process_dtms
                .push(segment.to_raw_string(&self.delimiters));
        }
        self.has_data = true;

        let parsed = Self::parse_dtm_value(value, format_code);

        match qualifier {
            "137" => self.prozessdaten.prozessdatum = parsed,
            "471" => self.prozessdaten.wirksamkeitsdatum = parsed,
            "92" => self.prozessdaten.vertragsbeginn = parsed,
            "93" => self.prozessdaten.vertragsende = parsed,
            "Z42" => self.prozessdaten.lieferbeginndatum_in_bearbeitung = parsed,
            "Z43" => self.prozessdaten.datum_naechste_bearbeitung = parsed,
            "Z51" => self.prozessdaten.tag_des_empfangs = parsed,
            "Z52" => self.prozessdaten.kuendigungsdatum_kunde = parsed,
            "Z53" => self.prozessdaten.geplanter_liefertermin = parsed,
            "157" => self.prozessdaten.dtm_157 = parsed,
            "Z01" => self.prozessdaten.dtm_z01 = parsed,
            "76" => self.prozessdaten.dtm_76 = parsed,
            "154" => self.prozessdaten.dtm_154 = parsed,
            "Z05" => self.prozessdaten.dtm_z05 = parsed,
            "752" => self.prozessdaten.dtm_752 = parsed,
            "158" => self.prozessdaten.dtm_158 = parsed,
            "159" => self.prozessdaten.dtm_159 = parsed,
            "672" => self.prozessdaten.dtm_672 = parsed,
            "155" => self.prozessdaten.dtm_155 = parsed,
            "Z25" => self.prozessdaten.verwendung_der_daten_ab = parsed,
            "Z26" => self.prozessdaten.verwendung_der_daten_bis = parsed,
            _ => {}
        }
    }

    /// Build the composite value string from all components after the qualifier.
    /// E.g., for RFF+Z49::1, returns "::1". For RFF+Z13:55126, returns ":55126".
    fn rff_composite_value(
        segment: &RawSegment,
        delimiters: &edifact_types::EdifactDelimiters,
    ) -> String {
        let components = segment.get_components(0);
        if components.len() <= 1 {
            return String::new();
        }
        let mut result = String::new();
        for comp in &components[1..] {
            result.push(delimiters.component as char);
            result.push_str(comp);
        }
        result
    }

    fn handle_rff(&mut self, segment: &RawSegment) {
        // RFF+qualifier:value' or RFF+qualifier::value' (3-component)
        let qualifier = segment.get_component(0, 0);
        let value = segment.get_component(0, 1);
        // Build full composite value for qualifiers that may have 3+ components
        let full_composite = Self::rff_composite_value(segment, &self.delimiters);

        match qualifier {
            "Z13" => {
                self.prozessdaten.referenz_vorgangsnummer = Some(value.to_string());
                self.has_data = true;
            }
            "Z14" => {
                self.prozessdaten.anfrage_referenz = Some(value.to_string());
                self.has_data = true;
            }
            "AGI" => {
                self.prozessdaten.vorgangsnummer = Some(value.to_string());
                self.has_data = true;
            }
            "ACW" => {
                self.prozessdaten.referenz_vorgangsnummer_acw = Some(value.to_string());
                self.has_data = true;
            }
            "AAV" => {
                self.prozessdaten.anfrage_referenz_aav = Some(value.to_string());
                self.has_data = true;
            }
            "TN" => {
                self.prozessdaten.referenz_transaktions_id = Some(value.to_string());
                self.has_data = true;
            }
            "Z18" => {
                // RFF+Z18 can be valueless or with value
                self.prozessdaten.rff_z18 = Some(value.to_string());
                self.has_data = true;
            }
            "Z49" | "Z50" | "Z52" | "Z53" => {
                // Z49/Z50/Z52/Z53 start Zeitscheibe reference blocks
                let raw_rff = segment.to_raw_string(&self.delimiters);
                self.prozessdaten.zeitscheibe_refs.push(
                    bo4e_extensions::prozessdaten::ZeitscheibeRef {
                        raw_rff,
                        raw_dtms: Vec::new(),
                    },
                );
                self.in_zeitscheibe_zone = true;
                self.has_data = true;
            }
            "Z01" => {
                self.prozessdaten.rff_z01 = Some(full_composite);
                self.has_data = true;
            }
            "Z31" => {
                self.prozessdaten.rff_z31 = Some(full_composite);
                self.has_data = true;
            }
            "Z39" => {
                self.prozessdaten.rff_z39 = Some(value.to_string());
                self.has_data = true;
            }
            "Z42" => {
                self.prozessdaten.rff_z42 = Some(value.to_string());
                self.has_data = true;
            }
            "Z43" => {
                self.prozessdaten.rff_z43 = Some(value.to_string());
                self.has_data = true;
            }
            "Z60" => {
                self.prozessdaten.rff_z60 = Some(value.to_string());
                self.has_data = true;
            }
            "Z47" | "Z48" => {
                // Z47/Z48: Zeitscheibe reference — store raw for roundtrip fidelity
                let raw_rff = segment.to_raw_string(&self.delimiters);
                self.prozessdaten.zeitscheibe_refs.push(
                    bo4e_extensions::prozessdaten::ZeitscheibeRef {
                        raw_rff,
                        raw_dtms: Vec::new(),
                    },
                );
                self.in_zeitscheibe_zone = true;
                self.has_data = true;
            }
            _ => {} // Other RFF qualifiers handled by other mappers
        }
    }

    fn handle_imd(&mut self, segment: &RawSegment, delimiters: &edifact_types::EdifactDelimiters) {
        let raw = segment.to_raw_string(delimiters);
        self.prozessdaten.raw_imd.push(raw);
        self.has_data = true;
    }

    fn handle_ftx(&mut self, segment: &RawSegment, delimiters: &edifact_types::EdifactDelimiters) {
        let qualifier = segment.get_element(0);

        // Store raw FTX for roundtrip fidelity
        let raw = segment.to_raw_string(delimiters);
        self.prozessdaten.raw_ftx.push(raw);
        self.has_data = true;

        // Also parse ACB into bemerkung field for semantic access
        if qualifier == "ACB" {
            let text = segment.get_component(3, 0);
            if !text.is_empty() {
                self.prozessdaten.bemerkung = Some(text.to_string());
            }
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
        match segment.id {
            // STS/FTX/IMD are always process-level
            "STS" | "FTX" | "IMD" => true,
            // RFF/DTM: only at process level (before any SEQ)
            "RFF" | "DTM" => !self.in_seq_zone,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        let delimiters = self.delimiters;
        match segment.id {
            "STS" => self.handle_sts(segment, &delimiters),
            "DTM" => self.handle_dtm(segment),
            "RFF" => self.handle_rff(segment),
            "FTX" => self.handle_ftx(segment, &delimiters),
            "IMD" => self.handle_imd(segment, &delimiters),
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
        self.in_seq_zone = false;
        self.in_zeitscheibe_zone = false;
        // delimiters are preserved across resets
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
    fn test_prozessdaten_mapper_sts_e01_raw() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let sts = RawSegment::new(
            "STS",
            vec![vec!["E01"], vec![], vec!["A01", "E_0408", "", "1"]],
            pos(),
        );

        assert!(mapper.can_handle(&sts));
        mapper.handle(&sts, &mut ctx);

        assert!(!mapper.is_empty());
        let pd = mapper.build();
        // STS+E01 is stored as raw, not parsed into transaktionsgrund
        assert_eq!(pd.sts_raw.len(), 1);
    }

    #[test]
    fn test_prozessdaten_mapper_dtm_137() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let dtm = RawSegment::new("DTM", vec![vec!["137", "202506190130", "303"]], pos());

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
        let dtm = RawSegment::new("DTM", vec![vec!["137", "202506190130?+00", "303"]], pos());

        mapper.handle(&dtm, &mut ctx);

        let pd = mapper.build();
        assert!(pd.prozessdatum.is_some());
    }

    #[test]
    fn test_prozessdaten_mapper_dtm_102_format() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let dtm = RawSegment::new("DTM", vec![vec!["471", "20250701", "102"]], pos());

        mapper.handle(&dtm, &mut ctx);

        let pd = mapper.build();
        assert!(pd.wirksamkeitsdatum.is_some());
    }

    #[test]
    fn test_prozessdaten_mapper_rff() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let rff = RawSegment::new("RFF", vec![vec!["Z13", "VORGANGSNUMMER001"]], pos());

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
    fn test_prozessdaten_mapper_sts_s21_format() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // Real S2.1 format: STS+7++E01+ZW4+E03
        let sts = RawSegment::new(
            "STS",
            vec![vec!["7"], vec![], vec!["E01"], vec!["ZW4"], vec!["E03"]],
            pos(),
        );
        mapper.handle(&sts, &mut ctx);

        assert!(!mapper.is_empty());
        let pd = mapper.build();
        assert_eq!(pd.transaktionsgrund, Some("E01".to_string()));
        assert_eq!(pd.transaktionsgrund_ergaenzung, Some("ZW4".to_string()));
        assert_eq!(
            pd.transaktionsgrund_ergaenzung_befristete_anmeldung,
            Some("E03".to_string())
        );
    }

    #[test]
    fn test_prozessdaten_mapper_sts_s21_partial() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // STS+7++E01 (no ergaenzung)
        let sts = RawSegment::new("STS", vec![vec!["7"], vec![], vec!["E01"]], pos());
        mapper.handle(&sts, &mut ctx);

        assert!(!mapper.is_empty());
        let pd = mapper.build();
        assert_eq!(pd.transaktionsgrund, Some("E01".to_string()));
        assert!(pd.transaktionsgrund_ergaenzung.is_none());
        assert!(pd
            .transaktionsgrund_ergaenzung_befristete_anmeldung
            .is_none());
    }

    #[test]
    fn test_prozessdaten_mapper_reset() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let sts = RawSegment::new("STS", vec![vec!["7"], vec![], vec!["E01"]], pos());
        mapper.handle(&sts, &mut ctx);
        assert!(!mapper.is_empty());

        mapper.reset();
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_prozessdaten_mapper_stores_unknown_dtm_raw() {
        let mut mapper = ProzessdatenMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let dtm = RawSegment::new("DTM", vec![vec!["999", "20250701", "102"]], pos());

        mapper.handle(&dtm, &mut ctx);
        // Unknown DTM qualifiers are stored in raw_dtm for roundtrip fidelity
        assert!(!mapper.is_empty());
        let pd = mapper.build();
        assert_eq!(pd.raw_dtm.get("999"), Some(&"20250701:102".to_string()));
    }
}
