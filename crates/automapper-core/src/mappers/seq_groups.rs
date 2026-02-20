//! Unified mapper for SEQ group segments (Z45, Z71, Z21, Z08, Z01, Z20).
//!
//! Tracks which SEQ group is currently active via context flags.
//! Routes CCI, CAV, PIA, QTY segments to the current active group.
//! On a new SEQ segment, finalizes the previous group and starts a new one.
//!
//! C# reference: MarktlokationMapper handles SEQ context flags and
//! routes CCI/CAV per group. This Rust implementation uses a dedicated mapper.

use bo4e_extensions::{
    CciEntry, GenericSeqGroup, SeqZ01Group, SeqZ08Group, SeqZ20Group, SeqZ21Group, SeqZ45Group,
    SeqZ71Group,
};
use edifact_types::{EdifactDelimiters, RawSegment};

use crate::context::TransactionContext;
use crate::traits::SegmentHandler;

/// Tracks the currently active SEQ group type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveSeqGroup {
    None,
    Z45,
    Z71,
    Z21,
    Z08,
    Z01,
    Z20,
    /// A SEQ group we don't have typed storage for — store raw segments.
    Generic,
}

/// Unified mapper for all SEQ group segments.
///
/// Routes CCI/CAV/PIA/QTY to the currently active SEQ group.
/// Finalizes groups on new SEQ or on build().
pub struct SeqGroupMapper {
    active_group: ActiveSeqGroup,
    delimiters: EdifactDelimiters,

    // Current group being built
    current_z45: SeqZ45Group,
    current_z71: SeqZ71Group,
    current_z21: SeqZ21Group,
    current_z08: SeqZ08Group,
    current_z01: SeqZ01Group,
    current_z20: SeqZ20Group,
    current_generic: GenericSeqGroup,

    // Completed groups
    pub z45_groups: Vec<SeqZ45Group>,
    pub z71_groups: Vec<SeqZ71Group>,
    pub z21_groups: Vec<SeqZ21Group>,
    pub z08_groups: Vec<SeqZ08Group>,
    pub z01_groups: Vec<SeqZ01Group>,
    pub z20_groups: Vec<SeqZ20Group>,
    pub generic_groups: Vec<GenericSeqGroup>,

    /// Order of SEQ groups as they appeared: (qualifier, index_within_type).
    pub group_order: Vec<(String, usize)>,

    has_data: bool,
}

impl SeqGroupMapper {
    pub fn new() -> Self {
        Self {
            active_group: ActiveSeqGroup::None,
            delimiters: EdifactDelimiters::default(),
            current_z45: SeqZ45Group::default(),
            current_z71: SeqZ71Group::default(),
            current_z21: SeqZ21Group::default(),
            current_z08: SeqZ08Group::default(),
            current_z01: SeqZ01Group::default(),
            current_z20: SeqZ20Group::default(),
            current_generic: GenericSeqGroup::default(),
            z45_groups: Vec::new(),
            z71_groups: Vec::new(),
            z21_groups: Vec::new(),
            z08_groups: Vec::new(),
            z01_groups: Vec::new(),
            z20_groups: Vec::new(),
            generic_groups: Vec::new(),
            group_order: Vec::new(),
            has_data: false,
        }
    }

    pub fn set_delimiters(&mut self, delimiters: EdifactDelimiters) {
        self.delimiters = delimiters;
    }

    /// Finalize the current active group and add it to the completed list.
    fn finalize_current(&mut self) {
        match self.active_group {
            ActiveSeqGroup::Z45 => {
                let idx = self.z45_groups.len();
                let group = std::mem::take(&mut self.current_z45);
                self.z45_groups.push(group);
                self.group_order.push(("Z45".to_string(), idx));
            }
            ActiveSeqGroup::Z71 => {
                let idx = self.z71_groups.len();
                let group = std::mem::take(&mut self.current_z71);
                self.z71_groups.push(group);
                self.group_order.push(("Z71".to_string(), idx));
            }
            ActiveSeqGroup::Z21 => {
                let idx = self.z21_groups.len();
                let group = std::mem::take(&mut self.current_z21);
                self.z21_groups.push(group);
                self.group_order.push(("Z21".to_string(), idx));
            }
            ActiveSeqGroup::Z08 => {
                let idx = self.z08_groups.len();
                let group = std::mem::take(&mut self.current_z08);
                self.z08_groups.push(group);
                self.group_order.push(("Z08".to_string(), idx));
            }
            ActiveSeqGroup::Z01 => {
                let idx = self.z01_groups.len();
                let group = std::mem::take(&mut self.current_z01);
                self.z01_groups.push(group);
                self.group_order.push(("Z01".to_string(), idx));
            }
            ActiveSeqGroup::Z20 => {
                let idx = self.z20_groups.len();
                let group = std::mem::take(&mut self.current_z20);
                self.z20_groups.push(group);
                self.group_order.push(("Z20".to_string(), idx));
            }
            ActiveSeqGroup::Generic => {
                if !self.current_generic.raw_segments.is_empty() {
                    let idx = self.generic_groups.len();
                    let group = std::mem::take(&mut self.current_generic);
                    self.generic_groups.push(group);
                    self.group_order.push(("_generic".to_string(), idx));
                }
            }
            ActiveSeqGroup::None => {}
        }
        self.active_group = ActiveSeqGroup::None;
    }

    fn handle_seq(&mut self, segment: &RawSegment) {
        // Finalize previous group
        self.finalize_current();

        let qualifier = segment.get_element(0);
        let zeitscheibe_ref = segment.get_element(1);
        let zs_ref = if zeitscheibe_ref.is_empty() {
            None
        } else {
            Some(zeitscheibe_ref.to_string())
        };

        self.has_data = true;

        match qualifier {
            "Z45" => {
                self.active_group = ActiveSeqGroup::Z45;
                self.current_z45.zeitscheibe_ref = zs_ref;
            }
            "Z71" => {
                self.active_group = ActiveSeqGroup::Z71;
                self.current_z71.zeitscheibe_ref = zs_ref;
            }
            "Z21" => {
                self.active_group = ActiveSeqGroup::Z21;
                self.current_z21.zeitscheibe_ref = zs_ref;
            }
            "Z08" => {
                self.active_group = ActiveSeqGroup::Z08;
                self.current_z08.zeitscheibe_ref = zs_ref;
            }
            "Z01" => {
                self.active_group = ActiveSeqGroup::Z01;
                self.current_z01.zeitscheibe_ref = zs_ref;
            }
            "Z20" => {
                self.active_group = ActiveSeqGroup::Z20;
                self.current_z20.zeitscheibe_ref = zs_ref;
            }
            // These qualifiers are handled by dedicated mappers.
            // We still record their position in the group order.
            "Z03" | "Z78" | "Z79" | "ZH0" | "Z98" | "Z81" => {
                self.group_order.push((qualifier.to_string(), 0));
                self.active_group = ActiveSeqGroup::None;
            }
            _ => {
                // Generic SEQ group: store raw segments
                self.active_group = ActiveSeqGroup::Generic;
                self.current_generic.qualifier = qualifier.to_string();
                self.current_generic.zeitscheibe_ref = zs_ref;
                // Store the SEQ segment itself
                let raw = segment.to_raw_string(&self.delimiters);
                self.current_generic.raw_segments.push(raw);
            }
        }
    }

    fn handle_cci(&mut self, segment: &RawSegment) {
        let raw = segment.to_raw_string(&self.delimiters);

        let qualifier = {
            let el0 = segment.get_element(0);
            if el0.is_empty() {
                None
            } else {
                Some(el0.to_string())
            }
        };
        let additional = {
            let el1 = segment.get_element(1);
            if el1.is_empty() {
                None
            } else {
                Some(el1.to_string())
            }
        };
        let code = {
            let c = segment.get_component(2, 0);
            if c.is_empty() {
                None
            } else {
                Some(c.to_string())
            }
        };

        let entry = CciEntry {
            qualifier,
            additional_qualifier: additional,
            characteristic_code: code,
        };

        match self.active_group {
            ActiveSeqGroup::Z45 => {
                self.current_z45.cci_segments.push(entry);
                self.current_z45.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z71 => {
                self.current_z71.cci_segments.push(entry);
                self.current_z71.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z21 => {
                self.current_z21.cci_segments.push(entry);
                self.current_z21.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z08 => {
                self.current_z08.cci_segments.push(entry);
                self.current_z08.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z01 => {
                self.current_z01.cci_segments.push(entry);
                self.current_z01.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z20 => {
                self.current_z20.cci_segments.push(entry);
                self.current_z20.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Generic => {
                self.current_generic.raw_segments.push(raw);
            }
            ActiveSeqGroup::None => {} // CCI outside SEQ group — handled by other mappers
        }
    }

    fn handle_cav(&mut self, segment: &RawSegment) {
        let raw = segment.to_raw_string(&self.delimiters);
        let value = segment.get_component(0, 0);
        let val = if value.is_empty() {
            raw.clone()
        } else {
            value.to_string()
        };

        match self.active_group {
            ActiveSeqGroup::Z45 => {
                self.current_z45.cav_segments.push(val);
                self.current_z45.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z71 => {
                self.current_z71.cav_segments.push(val);
                self.current_z71.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z21 => {
                self.current_z21.cav_segments.push(val);
                self.current_z21.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z08 => {
                self.current_z08.cav_segments.push(val);
                self.current_z08.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z01 => {
                self.current_z01.cav_segments.push(val);
                self.current_z01.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z20 => {
                self.current_z20.cav_segments.push(val);
                self.current_z20.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Generic => {
                self.current_generic.raw_segments.push(raw);
            }
            ActiveSeqGroup::None => {}
        }
    }

    fn handle_pia(&mut self, segment: &RawSegment) {
        // PIA+Z02+artikelId:artikelIdTyp'
        let qualifier = segment.get_element(0);
        let raw = segment.to_raw_string(&self.delimiters);

        if qualifier == "Z02" {
            let artikel_id = segment.get_component(1, 0);
            let artikel_id_typ = segment.get_component(1, 1);

            match self.active_group {
                ActiveSeqGroup::Z45 => {
                    if !artikel_id.is_empty() {
                        self.current_z45.artikel_id = Some(artikel_id.to_string());
                    }
                    if !artikel_id_typ.is_empty() {
                        self.current_z45.artikel_id_typ = Some(artikel_id_typ.to_string());
                    }
                    self.current_z45.raw_cci_cav.push(raw);
                }
                ActiveSeqGroup::Z71 => {
                    if !artikel_id.is_empty() {
                        self.current_z71.artikel_id = Some(artikel_id.to_string());
                    }
                    if !artikel_id_typ.is_empty() {
                        self.current_z71.artikel_id_typ = Some(artikel_id_typ.to_string());
                    }
                    self.current_z71.raw_cci_cav.push(raw);
                }
                ActiveSeqGroup::Z20 => {
                    self.current_z20.pia_segments.push(raw.clone());
                    self.current_z20.raw_cci_cav.push(raw);
                }
                _ => {
                    // PIA in other groups — store as raw in generic
                    if self.active_group == ActiveSeqGroup::Generic {
                        self.current_generic.raw_segments.push(raw);
                    }
                }
            }
        } else if self.active_group == ActiveSeqGroup::Z20 {
            self.current_z20.pia_segments.push(raw.clone());
            self.current_z20.raw_cci_cav.push(raw);
        } else if self.active_group == ActiveSeqGroup::Generic {
            self.current_generic.raw_segments.push(raw);
        }
    }

    fn handle_qty(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_component(0, 0);
        let value = segment.get_component(0, 1);
        let unit = segment.get_component(0, 2);

        let raw_composite = if unit.is_empty() {
            format!("{}:{}", qualifier, value)
        } else {
            format!("{}:{}:{}", qualifier, value, unit)
        };

        let raw = segment.to_raw_string(&self.delimiters);

        match self.active_group {
            ActiveSeqGroup::Z45 => {
                match qualifier {
                    "Z38" => self.current_z45.wandlerfaktor = Some(raw_composite),
                    "Z16" => self.current_z45.vorkommastelle = Some(raw_composite),
                    "Z37" => self.current_z45.nachkommastelle = Some(raw_composite),
                    _ => {}
                }
                // Always store raw for roundtrip fidelity
                self.current_z45.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Z01 => {
                self.current_z01.qty_segments.push(raw_composite);
                self.current_z01.raw_cci_cav.push(raw);
            }
            ActiveSeqGroup::Generic => {
                self.current_generic.raw_segments.push(raw);
            }
            _ => {} // QTY in other groups
        }
    }

    /// Called by other mappers or coordinator to handle segments that
    /// might be within a SEQ group context.
    fn handle_other_in_generic(&mut self, segment: &RawSegment) {
        if self.active_group == ActiveSeqGroup::Generic {
            let raw = segment.to_raw_string(&self.delimiters);
            self.current_generic.raw_segments.push(raw);
        }
    }

    /// Build all groups — finalizes any open group.
    /// Returns (z45, z71, z21, z08, z01, z20, generic, group_order).
    #[allow(clippy::type_complexity)]
    pub fn build_all(
        &mut self,
    ) -> (
        Vec<SeqZ45Group>,
        Vec<SeqZ71Group>,
        Vec<SeqZ21Group>,
        Vec<SeqZ08Group>,
        Vec<SeqZ01Group>,
        Vec<SeqZ20Group>,
        Vec<GenericSeqGroup>,
        Vec<(String, usize)>,
    ) {
        self.finalize_current();
        (
            std::mem::take(&mut self.z45_groups),
            std::mem::take(&mut self.z71_groups),
            std::mem::take(&mut self.z21_groups),
            std::mem::take(&mut self.z08_groups),
            std::mem::take(&mut self.z01_groups),
            std::mem::take(&mut self.z20_groups),
            std::mem::take(&mut self.generic_groups),
            std::mem::take(&mut self.group_order),
        )
    }

    pub fn is_empty(&self) -> bool {
        !self.has_data
    }

    pub fn reset(&mut self) {
        self.active_group = ActiveSeqGroup::None;
        self.current_z45 = SeqZ45Group::default();
        self.current_z71 = SeqZ71Group::default();
        self.current_z21 = SeqZ21Group::default();
        self.current_z08 = SeqZ08Group::default();
        self.current_z01 = SeqZ01Group::default();
        self.current_z20 = SeqZ20Group::default();
        self.current_generic = GenericSeqGroup::default();
        self.z45_groups.clear();
        self.z71_groups.clear();
        self.z21_groups.clear();
        self.z08_groups.clear();
        self.z01_groups.clear();
        self.z20_groups.clear();
        self.generic_groups.clear();
        self.group_order.clear();
        self.has_data = false;
    }

    /// Returns the currently active SEQ group type, if any.
    pub fn active_group_type(&self) -> Option<&'static str> {
        match self.active_group {
            ActiveSeqGroup::Z45 => Some("Z45"),
            ActiveSeqGroup::Z71 => Some("Z71"),
            ActiveSeqGroup::Z21 => Some("Z21"),
            ActiveSeqGroup::Z08 => Some("Z08"),
            ActiveSeqGroup::Z01 => Some("Z01"),
            ActiveSeqGroup::Z20 => Some("Z20"),
            ActiveSeqGroup::Generic => Some("Generic"),
            ActiveSeqGroup::None => None,
        }
    }
}

impl Default for SeqGroupMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for SeqGroupMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            // Handle ALL SEQ segments to properly finalize previous groups.
            // Segments with dedicated mappers (Z03, Z78, etc.) set active_group = None
            // in handle_seq, which prevents CCI/CAV capture in that context.
            "SEQ" => true,
            // CCI/CAV/PIA/QTY within an active group
            "CCI" | "CAV" | "PIA" | "QTY" => self.active_group != ActiveSeqGroup::None,
            // RFF within an active SEQ group
            "RFF" => matches!(
                self.active_group,
                ActiveSeqGroup::Z01
                    | ActiveSeqGroup::Z20
                    | ActiveSeqGroup::Z21
                    | ActiveSeqGroup::Z08
                    | ActiveSeqGroup::Generic
            ),
            // NAD/UNS ends all SEQ groups — finalize active group
            "NAD" | "UNS" => self.active_group != ActiveSeqGroup::None,
            // DTM/IMD within a generic group
            "DTM" | "IMD" => self.active_group == ActiveSeqGroup::Generic,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            // NAD/UNS ends the SEQ zone — finalize any active group
            "NAD" | "UNS" => {
                self.finalize_current();
            }
            "SEQ" => self.handle_seq(segment),
            "CCI" => self.handle_cci(segment),
            "CAV" => self.handle_cav(segment),
            "PIA" => self.handle_pia(segment),
            "QTY" => self.handle_qty(segment),
            "RFF" if self.active_group == ActiveSeqGroup::Z01 => {
                let raw = segment.to_raw_string(&self.delimiters);
                self.current_z01.rff_segments.push(raw.clone());
                self.current_z01.raw_cci_cav.push(raw);
            }
            "RFF" if self.active_group == ActiveSeqGroup::Z20 => {
                let raw = segment.to_raw_string(&self.delimiters);
                self.current_z20.rff_segments.push(raw.clone());
                self.current_z20.raw_cci_cav.push(raw);
            }
            "RFF" if self.active_group == ActiveSeqGroup::Z21 => {
                let raw = segment.to_raw_string(&self.delimiters);
                self.current_z21.rff_segments.push(raw.clone());
                self.current_z21.raw_cci_cav.push(raw);
            }
            "RFF" if self.active_group == ActiveSeqGroup::Z08 => {
                let raw = segment.to_raw_string(&self.delimiters);
                self.current_z08.rff_segments.push(raw.clone());
                self.current_z08.raw_cci_cav.push(raw);
            }
            _ => self.handle_other_in_generic(segment),
        }
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
    fn test_seq_z45_basic() {
        let mut mapper = SeqGroupMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // SEQ+Z45+1'
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z45"], vec!["1"]], pos()),
            &mut ctx,
        );
        // PIA+Z02+ART001:Z09'
        mapper.handle(
            &RawSegment::new("PIA", vec![vec!["Z02"], vec!["ART001", "Z09"]], pos()),
            &mut ctx,
        );
        // CCI+Z30++Z07'
        mapper.handle(
            &RawSegment::new("CCI", vec![vec!["Z30"], vec![], vec!["Z07"]], pos()),
            &mut ctx,
        );
        // CAV+Z74'
        mapper.handle(&RawSegment::new("CAV", vec![vec!["Z74"]], pos()), &mut ctx);

        let (z45, _, _, _, _, _, _, _) = mapper.build_all();
        assert_eq!(z45.len(), 1);
        assert_eq!(z45[0].zeitscheibe_ref, Some("1".to_string()));
        assert_eq!(z45[0].artikel_id, Some("ART001".to_string()));
        assert_eq!(z45[0].artikel_id_typ, Some("Z09".to_string()));
        assert_eq!(z45[0].cci_segments.len(), 1);
        assert_eq!(z45[0].cav_segments.len(), 1);
    }

    #[test]
    fn test_seq_z01_with_cci_cav() {
        let mut mapper = SeqGroupMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z01"]], pos()), &mut ctx);
        mapper.handle(
            &RawSegment::new("CCI", vec![vec![], vec![], vec!["ZB3"]], pos()),
            &mut ctx,
        );
        mapper.handle(&RawSegment::new("CAV", vec![vec!["Z91"]], pos()), &mut ctx);

        let (_, _, _, _, z01, _, _, _) = mapper.build_all();
        assert_eq!(z01.len(), 1);
        assert_eq!(z01[0].cci_segments.len(), 1);
        assert_eq!(
            z01[0].cci_segments[0].characteristic_code,
            Some("ZB3".to_string())
        );
        assert_eq!(z01[0].cav_segments, vec!["Z91"]);
    }

    #[test]
    fn test_multiple_seq_groups() {
        let mut mapper = SeqGroupMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // First Z01
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z01"], vec!["1"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec![], vec![], vec!["ZB3"]], pos()),
            &mut ctx,
        );
        // Second Z01
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z01"], vec!["2"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec![], vec![], vec!["E03"]], pos()),
            &mut ctx,
        );
        // Z45
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z45"]], pos()), &mut ctx);

        let (z45, _, _, _, z01, _, _, _) = mapper.build_all();
        assert_eq!(z01.len(), 2);
        assert_eq!(z01[0].zeitscheibe_ref, Some("1".to_string()));
        assert_eq!(z01[1].zeitscheibe_ref, Some("2".to_string()));
        assert_eq!(z45.len(), 1);
    }

    #[test]
    fn test_reset() {
        let mut mapper = SeqGroupMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z01"]], pos()), &mut ctx);
        assert!(!mapper.is_empty());

        mapper.reset();
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_generic_seq_group() {
        let mut mapper = SeqGroupMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        // SEQ+Z02 is a generic group
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z02"], vec!["1"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec!["Z30"], vec![], vec!["Z07"]], pos()),
            &mut ctx,
        );
        mapper.handle(&RawSegment::new("CAV", vec![vec!["Z74"]], pos()), &mut ctx);

        let (_, _, _, _, _, _, generic, _) = mapper.build_all();
        assert_eq!(generic.len(), 1);
        assert_eq!(generic[0].qualifier, "Z02");
        // SEQ raw + CCI raw + CAV raw
        assert_eq!(generic[0].raw_segments.len(), 3);
    }
}
