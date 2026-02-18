---
feature: edifact-core-implementation
epic: 7
title: "UTILMD Forward Mapping (EDIFACT -> BO4E)"
depends_on: [6]
estimated_tasks: 6
crate: automapper-core
---

# Epic 7: UTILMD Forward Mapping (EDIFACT -> BO4E)

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-core/src/`. All code must compile with `cargo check -p automapper-core`.

**Goal:** Implement the UTILMD forward mapping pipeline: `UtilmdCoordinator<V>` implementing `EdifactHandler` and `Coordinator`, plus all entity mappers (`MarktlokationMapper`, `MesslokationMapper`, `NetzlokationMapper`, `ZaehlerMapper`, `GeschaeftspartnerMapper`, `VertragMapper`, `ProzessdatenMapper`, `ZeitscheibeMapper`). Each mapper implements `SegmentHandler` and `Builder<T>` to parse EDIFACT segments into BO4E domain objects.

**Architecture:** The `UtilmdCoordinator<V>` is the central orchestrator. It implements `EdifactHandler` (from the parser crate) and routes each segment to registered mappers via `can_handle()` / `handle()`. On `on_message_end()`, it collects built objects from all mappers into a `UtilmdTransaktion`. The coordinator uses the `VersionConfig` trait to instantiate version-specific mappers at compile time. Mappers handle specific segment qualifiers (e.g., LOC+Z16 for Marktlokation, SEQ+Z03 for Zaehler). See design doc section 5 and C# `UtilmdCoordinator.cs`, `MarktlokationMapper.cs`.

**Tech Stack:** Rust, edifact-types, edifact-parser, bo4e-extensions, automapper-core traits, insta for snapshot testing, test-case for parameterized tests

---

## Task 1: ProzessdatenMapper (STS, DTM, RFF, FTX at Transaction Level)

**Files:**
- Create: `crates/automapper-core/src/mappers/mod.rs`
- Create: `crates/automapper-core/src/mappers/prozessdaten.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/mappers/mod.rs`:

```rust
//! Entity-specific mappers for UTILMD messages.
//!
//! Each mapper implements `SegmentHandler` + `Builder<T>` for one entity type.
//! The coordinator registers all mappers and routes segments to them.

pub mod prozessdaten;

pub use prozessdaten::ProzessdatenMapper;
```

Create `crates/automapper-core/src/mappers/prozessdaten.rs`:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_prozessdaten_mapper`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/lib.rs`:

```rust
//! Core automapper library.
//!
//! Coordinators, mappers, builders, and writers for bidirectional
//! EDIFACT <-> BO4E conversion. Supports streaming parsing with
//! format-version-dispatched mappers and parallel batch processing.

pub mod context;
pub mod coordinator;
pub mod error;
pub mod mappers;
pub mod traits;
pub mod version;

pub use context::TransactionContext;
pub use coordinator::{create_coordinator, detect_format_version, Coordinator};
pub use error::AutomapperError;
pub use traits::*;
pub use version::{FV2504, FV2510, VersionConfig, VersionPhantom};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_prozessdaten_mapper`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add ProzessdatenMapper for STS/DTM/RFF/FTX

Maps transaction-level segments to Prozessdaten: process dates
(DTM+137, 471, Z25, Z26), references (RFF+Z13, Z14), status (STS),
and free text (FTX+ACB). Supports format 303 and 102 date parsing.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: ZeitscheibeMapper (RFF+Z49/Z50/Z53, DTM+Z25/Z26)

**Files:**
- Create: `crates/automapper-core/src/mappers/zeitscheibe.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/mappers/zeitscheibe.rs`:

```rust
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
        let mut mapper = ZeitscheibeMapper::new();

        let dtm = RawSegment::new("DTM", vec![vec!["Z25", "202507010000", "303"]], pos());
        assert!(!mapper.can_handle(&dtm));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_zeitscheibe_mapper`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/mappers/mod.rs`:

```rust
//! Entity-specific mappers for UTILMD messages.

pub mod prozessdaten;
pub mod zeitscheibe;

pub use prozessdaten::ProzessdatenMapper;
pub use zeitscheibe::ZeitscheibeMapper;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_zeitscheibe_mapper`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add ZeitscheibeMapper for time slice references

Maps RFF+Z49/Z50/Z53 to Zeitscheibe IDs with DTM+Z25/Z26 date bounds.
Supports multiple Zeitscheiben per transaction for temporal data slicing.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: MarktlokationMapper (LOC+Z16, SEQ+Z01)

**Files:**
- Create: `crates/automapper-core/src/mappers/marktlokation.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/mappers/marktlokation.rs`:

```rust
//! Mapper for Marktlokation (market location) business objects.
//!
//! Handles:
//! - LOC+Z16: Market location identifier
//! - SEQ+Z01: Marktlokation data group (Zugeordneter Marktpartner, Spannungsebene)
//! - NAD+DP/Z63: Delivery address
//! - NAD+MS: Sparte extraction from code list qualifier
//!
//! Produces: `WithValidity<Marktlokation, MarktlokationEdifact>`
//!
//! Mirrors C# `MarktlokationMapper.cs`.

use bo4e_extensions::{
    Marktlokation, MarktlokationEdifact, WithValidity,
};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Marktlokation business objects in UTILMD messages.
///
/// Handles LOC+Z16 for location ID, NAD+DP/Z63 for address, NAD+MS for
/// Sparte, and SEQ+Z01 for Marktlokation-specific data.
pub struct MarktlokationMapper {
    marktlokations_id: Option<String>,
    sparte: Option<String>,
    strasse: Option<String>,
    hausnummer: Option<String>,
    postleitzahl: Option<String>,
    ort: Option<String>,
    landescode: Option<String>,
    netzebene: Option<String>,
    bilanzierungsmethode: Option<String>,
    edifact: MarktlokationEdifact,
    has_data: bool,
    in_seq_z01: bool,
}

impl MarktlokationMapper {
    /// Creates a new MarktlokationMapper.
    pub fn new() -> Self {
        Self {
            marktlokations_id: None,
            sparte: None,
            strasse: None,
            hausnummer: None,
            postleitzahl: None,
            ort: None,
            landescode: None,
            netzebene: None,
            bilanzierungsmethode: None,
            edifact: MarktlokationEdifact::default(),
            has_data: false,
            in_seq_z01: false,
        }
    }

    fn handle_loc(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        if qualifier != "Z16" {
            return;
        }
        let id = segment.get_component(1, 0);
        if !id.is_empty() {
            self.marktlokations_id = Some(id.to_string());
            self.has_data = true;
        }
    }

    fn handle_nad(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        match qualifier {
            "DP" | "Z63" | "Z59" | "Z60" => self.handle_nad_address(segment),
            "MS" => self.handle_nad_ms(segment),
            _ => {}
        }
    }

    fn handle_nad_address(&mut self, segment: &RawSegment) {
        // NAD+DP++++Bergstr.::1+Berlin++10115+DE'
        // C059 (element 4): street address
        //   Component 0: street, Component 2: house number
        // Element 5: city
        // Element 7: postal code
        // Element 8: country code
        let strasse = segment.get_component(4, 0);
        if !strasse.is_empty() {
            self.strasse = Some(strasse.to_string());
            self.has_data = true;
        }

        let hausnummer = segment.get_component(4, 2);
        if !hausnummer.is_empty() {
            self.hausnummer = Some(hausnummer.to_string());
        }

        let ort = segment.get_element(5);
        if !ort.is_empty() {
            self.ort = Some(ort.to_string());
        }

        let plz = segment.get_element(7);
        if !plz.is_empty() {
            self.postleitzahl = Some(plz.to_string());
        }

        let land = segment.get_element(8);
        if !land.is_empty() {
            self.landescode = Some(land.to_string());
        }
    }

    fn handle_nad_ms(&mut self, segment: &RawSegment) {
        // NAD+MS+9900000000001::293'
        // C082 component 1: code list qualifier (293=STROM, 332=GAS)
        let code_qualifier = segment.get_component(1, 1);
        if !code_qualifier.is_empty() {
            let sparte = match code_qualifier {
                "293" | "500" => Some("STROM"),
                "332" => Some("GAS"),
                _ => None,
            };
            if let Some(s) = sparte {
                self.sparte = Some(s.to_string());
                self.has_data = true;
            }
        }
    }

    fn handle_seq(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        // Reset SEQ state flags
        self.in_seq_z01 = false;

        match qualifier {
            "Z01" => {
                self.in_seq_z01 = true;
            }
            _ => {}
        }
    }
}

impl Default for MarktlokationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for MarktlokationMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "LOC" => segment.get_element(0) == "Z16",
            "NAD" => {
                let q = segment.get_element(0);
                matches!(q, "DP" | "Z63" | "Z59" | "Z60" | "MS")
            }
            "SEQ" => true, // Handle all SEQ to track context
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "LOC" => self.handle_loc(segment),
            "NAD" => self.handle_nad(segment),
            "SEQ" => self.handle_seq(segment),
            _ => {}
        }
    }
}

impl Builder<Option<WithValidity<Marktlokation, MarktlokationEdifact>>> for MarktlokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Marktlokation, MarktlokationEdifact>> {
        if !self.has_data {
            return None;
        }

        let ml = Marktlokation {
            marktlokations_id: self.marktlokations_id.take(),
            sparte: self.sparte.take(),
            lokationsadresse: if self.strasse.is_some() || self.ort.is_some() {
                Some(bo4e_extensions::Adresse {
                    strasse: self.strasse.take(),
                    hausnummer: self.hausnummer.take(),
                    postleitzahl: self.postleitzahl.take(),
                    ort: self.ort.take(),
                    landescode: self.landescode.take(),
                })
            } else {
                None
            },
            bilanzierungsmethode: self.bilanzierungsmethode.take(),
            netzebene: self.netzebene.take(),
        };

        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;

        Some(WithValidity {
            data: ml,
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
    fn test_marktlokation_mapper_loc_z16() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let loc = RawSegment::new(
            "LOC",
            vec![
                vec!["Z16"],
                vec!["DE00014545768S0000000000000003054"],
            ],
            pos(),
        );

        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);

        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.marktlokations_id,
            Some("DE00014545768S0000000000000003054".to_string())
        );
    }

    #[test]
    fn test_marktlokation_mapper_ignores_loc_z17() {
        let mut mapper = MarktlokationMapper::new();

        let loc = RawSegment::new("LOC", vec![vec!["Z17"], vec!["MELO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_marktlokation_mapper_nad_dp_address() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let nad = RawSegment::new(
            "NAD",
            vec![
                vec!["DP"],
                vec![],
                vec![],
                vec![],
                vec!["Bergstr.", "", "1"],
                vec!["Berlin"],
                vec![],
                vec!["10115"],
                vec!["DE"],
            ],
            pos(),
        );

        assert!(mapper.can_handle(&nad));
        mapper.handle(&nad, &mut ctx);

        let result = mapper.build().unwrap();
        let addr = result.data.lokationsadresse.unwrap();
        assert_eq!(addr.strasse, Some("Bergstr.".to_string()));
        assert_eq!(addr.hausnummer, Some("1".to_string()));
        assert_eq!(addr.ort, Some("Berlin".to_string()));
        assert_eq!(addr.postleitzahl, Some("10115".to_string()));
        assert_eq!(addr.landescode, Some("DE".to_string()));
    }

    #[test]
    fn test_marktlokation_mapper_nad_ms_sparte() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let nad = RawSegment::new(
            "NAD",
            vec![vec!["MS"], vec!["9900123000002", "", "293"]],
            pos(),
        );

        mapper.handle(&nad, &mut ctx);

        let result = mapper.build().unwrap();
        assert_eq!(result.data.sparte, Some("STROM".to_string()));
    }

    #[test]
    fn test_marktlokation_mapper_empty_returns_none() {
        let mut mapper = MarktlokationMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }

    #[test]
    fn test_marktlokation_mapper_reset() {
        let mut mapper = MarktlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");

        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["DE001"]], pos());
        mapper.handle(&loc, &mut ctx);
        assert!(!mapper.is_empty());

        mapper.reset();
        assert!(mapper.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_marktlokation_mapper`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/mappers/mod.rs`:

```rust
//! Entity-specific mappers for UTILMD messages.

pub mod marktlokation;
pub mod prozessdaten;
pub mod zeitscheibe;

pub use marktlokation::MarktlokationMapper;
pub use prozessdaten::ProzessdatenMapper;
pub use zeitscheibe::ZeitscheibeMapper;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core test_marktlokation_mapper`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add MarktlokationMapper for LOC+Z16 and NAD

Maps LOC+Z16 to Marktlokation ID, NAD+DP/Z63 to address, NAD+MS to
Sparte. Tracks SEQ+Z01 context for Marktlokation data group.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: MesslokationMapper, NetzlokationMapper, GeschaeftspartnerMapper, VertragMapper, ZaehlerMapper

**Files:**
- Create: `crates/automapper-core/src/mappers/messlokation.rs`
- Create: `crates/automapper-core/src/mappers/netzlokation.rs`
- Create: `crates/automapper-core/src/mappers/geschaeftspartner.rs`
- Create: `crates/automapper-core/src/mappers/vertrag.rs`
- Create: `crates/automapper-core/src/mappers/zaehler.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/mappers/messlokation.rs`:

```rust
//! Mapper for Messlokation (metering location) business objects.
//!
//! Handles LOC+Z17 segments for metering location identification.

use bo4e_extensions::{Messlokation, MesslokationEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Messlokation in UTILMD messages.
pub struct MesslokationMapper {
    messlokations_id: Option<String>,
    edifact: MesslokationEdifact,
    has_data: bool,
}

impl MesslokationMapper {
    pub fn new() -> Self {
        Self {
            messlokations_id: None,
            edifact: MesslokationEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for MesslokationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for MesslokationMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z17"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.messlokations_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<Messlokation, MesslokationEdifact>>> for MesslokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Messlokation, MesslokationEdifact>> {
        if !self.has_data {
            return None;
        }
        let ml = Messlokation {
            messlokations_id: self.messlokations_id.take(),
            ..Default::default()
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: ml,
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
    fn test_messlokation_mapper_loc_z17() {
        let mut mapper = MesslokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z17"], vec!["DE00098765432100000000000000012"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.messlokations_id, Some("DE00098765432100000000000000012".to_string()));
    }

    #[test]
    fn test_messlokation_mapper_ignores_z16() {
        let mapper = MesslokationMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["ID"]], pos());
        assert!(!mapper.can_handle(&loc));
    }
}
```

Create `crates/automapper-core/src/mappers/netzlokation.rs`:

```rust
//! Mapper for Netzlokation (network location) business objects.
//!
//! Handles LOC+Z18 segments for network location identification.

use bo4e_extensions::{Netzlokation, NetzlokationEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Netzlokation in UTILMD messages.
pub struct NetzlokationMapper {
    netzlokations_id: Option<String>,
    edifact: NetzlokationEdifact,
    has_data: bool,
}

impl NetzlokationMapper {
    pub fn new() -> Self {
        Self {
            netzlokations_id: None,
            edifact: NetzlokationEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for NetzlokationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for NetzlokationMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z18"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.netzlokations_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<Netzlokation, NetzlokationEdifact>>> for NetzlokationMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Netzlokation, NetzlokationEdifact>> {
        if !self.has_data {
            return None;
        }
        let nl = Netzlokation {
            netzlokations_id: self.netzlokations_id.take(),
            ..Default::default()
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: nl,
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

    fn pos() -> SegmentPosition { SegmentPosition::new(1, 0, 1) }

    #[test]
    fn test_netzlokation_mapper_loc_z18() {
        let mut mapper = NetzlokationMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.netzlokations_id, Some("NELO001".to_string()));
    }
}
```

Create `crates/automapper-core/src/mappers/geschaeftspartner.rs`:

```rust
//! Mapper for Geschaeftspartner (business partner) from NAD segments.
//!
//! Handles NAD segments with party qualifiers (Z04, Z09, DP, etc.).

use bo4e_extensions::{
    Adresse, Geschaeftspartner, GeschaeftspartnerEdifact, WithValidity,
};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Geschaeftspartner in UTILMD messages.
///
/// Handles NAD segments with party qualifiers for business partners.
/// Each NAD with a party qualifier (Z04, Z09, Z48, Z50, etc.) creates
/// a separate Geschaeftspartner entry.
pub struct GeschaeftspartnerMapper {
    partners: Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>,
    has_data: bool,
}

impl GeschaeftspartnerMapper {
    pub fn new() -> Self {
        Self {
            partners: Vec::new(),
            has_data: false,
        }
    }

    /// NAD qualifiers that create Geschaeftspartner entries.
    fn is_party_qualifier(qualifier: &str) -> bool {
        matches!(
            qualifier,
            "Z04" | "Z09" | "Z48" | "Z50" | "Z25" | "Z26" | "EO" | "DDO"
        )
    }
}

impl Default for GeschaeftspartnerMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for GeschaeftspartnerMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        if segment.id != "NAD" {
            return false;
        }
        let qualifier = segment.get_element(0);
        Self::is_party_qualifier(qualifier)
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        let qualifier = segment.get_element(0);
        if !Self::is_party_qualifier(qualifier) {
            return;
        }

        // NAD+Z04+9900123000002::293+name1+++city++plz+DE'
        // C082 (element 1): party ID composite
        //   Component 0: party identification code
        //   Component 2: code list qualifier
        let party_id = segment.get_component(1, 0);
        let name1 = segment.get_element(2);

        // C059 (element 4): address
        let strasse = segment.get_component(4, 0);
        let hausnummer = segment.get_component(4, 2);
        let ort = segment.get_element(5);
        let plz = segment.get_element(7);
        let land = segment.get_element(8);

        let gp = Geschaeftspartner {
            name1: if !name1.is_empty() {
                Some(name1.to_string())
            } else if !party_id.is_empty() {
                Some(party_id.to_string())
            } else {
                None
            },
            partneradresse: if !strasse.is_empty() || !ort.is_empty() {
                Some(Adresse {
                    strasse: if strasse.is_empty() { None } else { Some(strasse.to_string()) },
                    hausnummer: if hausnummer.is_empty() { None } else { Some(hausnummer.to_string()) },
                    postleitzahl: if plz.is_empty() { None } else { Some(plz.to_string()) },
                    ort: if ort.is_empty() { None } else { Some(ort.to_string()) },
                    landescode: if land.is_empty() { None } else { Some(land.to_string()) },
                })
            } else {
                None
            },
            ..Default::default()
        };

        let edifact = GeschaeftspartnerEdifact {
            nad_qualifier: Some(qualifier.to_string()),
        };

        self.partners.push(WithValidity {
            data: gp,
            edifact,
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        });
        self.has_data = true;
    }
}

impl Builder<Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>>
    for GeschaeftspartnerMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>> {
        self.has_data = false;
        std::mem::take(&mut self.partners)
    }

    fn reset(&mut self) {
        self.partners.clear();
        self.has_data = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edifact_types::SegmentPosition;

    fn pos() -> SegmentPosition { SegmentPosition::new(1, 0, 1) }

    #[test]
    fn test_geschaeftspartner_mapper_nad_z04() {
        let mut mapper = GeschaeftspartnerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let nad = RawSegment::new("NAD", vec![vec!["Z04"], vec!["9900123000002", "", "293"]], pos());
        assert!(mapper.can_handle(&nad));
        mapper.handle(&nad, &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edifact.nad_qualifier, Some("Z04".to_string()));
        assert_eq!(result[0].data.name1, Some("9900123000002".to_string()));
    }

    #[test]
    fn test_geschaeftspartner_mapper_ignores_nad_ms() {
        let mapper = GeschaeftspartnerMapper::new();
        let nad = RawSegment::new("NAD", vec![vec!["MS"], vec!["ID"]], pos());
        assert!(!mapper.can_handle(&nad));
    }

    #[test]
    fn test_geschaeftspartner_mapper_multiple() {
        let mut mapper = GeschaeftspartnerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("NAD", vec![vec!["Z04"], vec!["PARTY1"]], pos()), &mut ctx);
        mapper.handle(&RawSegment::new("NAD", vec![vec!["Z09"], vec!["PARTY2"]], pos()), &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 2);
    }
}
```

Create `crates/automapper-core/src/mappers/vertrag.rs`:

```rust
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

    fn pos() -> SegmentPosition { SegmentPosition::new(1, 0, 1) }

    #[test]
    fn test_vertrag_mapper_seq_z18_haushaltskunde() {
        let mut mapper = VertragMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z18"]], pos()), &mut ctx);
        mapper.handle(&RawSegment::new("CCI", vec![vec!["Z15"], vec![], vec!["Z01"]], pos()), &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.edifact.haushaltskunde, Some(true));
    }

    #[test]
    fn test_vertrag_mapper_ignores_outside_z18() {
        let mut mapper = VertragMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z01"]], pos()), &mut ctx);
        mapper.handle(&RawSegment::new("CCI", vec![vec!["Z15"], vec![], vec!["Z01"]], pos()), &mut ctx);
        assert!(mapper.is_empty());
    }
}
```

Create `crates/automapper-core/src/mappers/zaehler.rs`:

```rust
//! Mapper for Zaehler (meter) business objects.
//!
//! Handles SEQ+Z03 (device data) and SEQ+Z79 (product package) segments.

use bo4e_extensions::{Zaehler, ZaehlerEdifact, WithValidity};
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

    fn pos() -> SegmentPosition { SegmentPosition::new(1, 0, 1) }

    #[test]
    fn test_zaehler_mapper_seq_z03_rff() {
        let mut mapper = ZaehlerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(&RawSegment::new("RFF", vec![vec!["Z19", "MELO001"]], pos()), &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edifact.referenz_messlokation, Some("MELO001".to_string()));
    }

    #[test]
    fn test_zaehler_mapper_pia_zaehlernummer() {
        let mut mapper = ZaehlerMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(&RawSegment::new("SEQ", vec![vec!["Z03"]], pos()), &mut ctx);
        mapper.handle(&RawSegment::new("PIA", vec![vec!["5"], vec!["ZAEHLER001"]], pos()), &mut ctx);
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.zaehlernummer, Some("ZAEHLER001".to_string()));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_messlokation_mapper`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/mappers/mod.rs`:

```rust
//! Entity-specific mappers for UTILMD messages.

pub mod geschaeftspartner;
pub mod marktlokation;
pub mod messlokation;
pub mod netzlokation;
pub mod prozessdaten;
pub mod vertrag;
pub mod zaehler;
pub mod zeitscheibe;

pub use geschaeftspartner::GeschaeftspartnerMapper;
pub use marktlokation::MarktlokationMapper;
pub use messlokation::MesslokationMapper;
pub use netzlokation::NetzlokationMapper;
pub use prozessdaten::ProzessdatenMapper;
pub use vertrag::VertragMapper;
pub use zaehler::ZaehlerMapper;
pub use zeitscheibe::ZeitscheibeMapper;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add Messlokation, Netzlokation, Geschaeftspartner, Vertrag, Zaehler mappers

MesslokationMapper handles LOC+Z17, NetzlokationMapper handles LOC+Z18,
GeschaeftspartnerMapper handles NAD party segments (Z04, Z09, etc.),
VertragMapper handles SEQ+Z18 with CCI, ZaehlerMapper handles SEQ+Z03.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: UtilmdCoordinator Implementation

**Files:**
- Create: `crates/automapper-core/src/utilmd_coordinator.rs`
- Modify: `crates/automapper-core/src/coordinator.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/utilmd_coordinator.rs`:

```rust
//! UTILMD-specific coordinator that orchestrates all mappers.
//!
//! Implements `EdifactHandler` and `Coordinator`. Routes segments to
//! registered mappers and collects built objects into `UtilmdTransaktion`.
//!
//! Mirrors C# `UtilmdCoordinator.cs`.

use std::marker::PhantomData;

use bo4e_extensions::{
    LinkRegistry, Marktteilnehmer, Nachrichtendaten, UtilmdTransaktion,
};
use edifact_types::{Control, EdifactDelimiters, RawSegment};
use edifact_parser::EdifactHandler;

use crate::context::TransactionContext;
use crate::coordinator::Coordinator;
use crate::error::AutomapperError;
use crate::mappers::*;
use crate::traits::{Builder, FormatVersion, SegmentHandler};
use crate::version::VersionConfig;

/// UTILMD coordinator that orchestrates all entity mappers.
///
/// Generic over `V: VersionConfig` for compile-time mapper dispatch.
/// Implements `EdifactHandler` for the streaming parser and `Coordinator`
/// for the high-level parse/generate API.
pub struct UtilmdCoordinator<V: VersionConfig> {
    context: TransactionContext,
    link_registry: LinkRegistry,

    // Mappers
    prozessdaten_mapper: ProzessdatenMapper,
    zeitscheibe_mapper: ZeitscheibeMapper,
    marktlokation_mapper: MarktlokationMapper,
    messlokation_mapper: MesslokationMapper,
    netzlokation_mapper: NetzlokationMapper,
    geschaeftspartner_mapper: GeschaeftspartnerMapper,
    vertrag_mapper: VertragMapper,
    zaehler_mapper: ZaehlerMapper,

    // Collected transactions
    transactions: Vec<UtilmdTransaktion>,

    // Nachrichtendaten from service segments
    nachrichtendaten: Nachrichtendaten,
    absender: Marktteilnehmer,
    empfaenger: Marktteilnehmer,

    // Current transaction state
    in_transaction: bool,
    current_transaction_id: Option<String>,

    _version: PhantomData<V>,
}

impl<V: VersionConfig> UtilmdCoordinator<V> {
    /// Creates a new UtilmdCoordinator.
    pub fn new() -> Self {
        Self {
            context: TransactionContext::new(V::VERSION.as_str()),
            link_registry: LinkRegistry::new(),
            prozessdaten_mapper: ProzessdatenMapper::new(),
            zeitscheibe_mapper: ZeitscheibeMapper::new(),
            marktlokation_mapper: MarktlokationMapper::new(),
            messlokation_mapper: MesslokationMapper::new(),
            netzlokation_mapper: NetzlokationMapper::new(),
            geschaeftspartner_mapper: GeschaeftspartnerMapper::new(),
            vertrag_mapper: VertragMapper::new(),
            zaehler_mapper: ZaehlerMapper::new(),
            transactions: Vec::new(),
            nachrichtendaten: Nachrichtendaten::default(),
            absender: Marktteilnehmer::default(),
            empfaenger: Marktteilnehmer::default(),
            in_transaction: false,
            current_transaction_id: None,
            _version: PhantomData,
        }
    }

    /// Routes a segment to all mappers that can handle it.
    fn route_to_mappers(&mut self, segment: &RawSegment) {
        if self.prozessdaten_mapper.can_handle(segment) {
            self.prozessdaten_mapper.handle(segment, &mut self.context);
        }
        if self.zeitscheibe_mapper.can_handle(segment) {
            self.zeitscheibe_mapper.handle(segment, &mut self.context);
        }
        if self.marktlokation_mapper.can_handle(segment) {
            self.marktlokation_mapper.handle(segment, &mut self.context);
        }
        if self.messlokation_mapper.can_handle(segment) {
            self.messlokation_mapper.handle(segment, &mut self.context);
        }
        if self.netzlokation_mapper.can_handle(segment) {
            self.netzlokation_mapper.handle(segment, &mut self.context);
        }
        if self.geschaeftspartner_mapper.can_handle(segment) {
            self.geschaeftspartner_mapper
                .handle(segment, &mut self.context);
        }
        if self.vertrag_mapper.can_handle(segment) {
            self.vertrag_mapper.handle(segment, &mut self.context);
        }
        if self.zaehler_mapper.can_handle(segment) {
            self.zaehler_mapper.handle(segment, &mut self.context);
        }
    }

    /// Handles IDE segment (transaction identifier).
    fn handle_ide(&mut self, segment: &RawSegment) {
        // IDE+24+transactionId'
        let qualifier = segment.get_element(0);
        if qualifier == "24" {
            let tx_id = segment.get_element(1);
            if !tx_id.is_empty() {
                self.current_transaction_id = Some(tx_id.to_string());
                self.in_transaction = true;
            }
        }
    }

    /// Handles NAD+MS (sender) and NAD+MR (recipient) at message level.
    fn handle_message_level_nad(&mut self, segment: &RawSegment) {
        let qualifier = segment.get_element(0);
        let mp_id = segment.get_component(1, 0);

        match qualifier {
            "MS" => {
                if !mp_id.is_empty() {
                    self.absender.mp_id = Some(mp_id.to_string());
                    self.context.set_sender_mp_id(mp_id);
                }
            }
            "MR" => {
                if !mp_id.is_empty() {
                    self.empfaenger.mp_id = Some(mp_id.to_string());
                    self.context.set_recipient_mp_id(mp_id);
                }
            }
            _ => {}
        }
    }

    /// Handles BGM segment (document type).
    fn handle_bgm(&mut self, segment: &RawSegment) {
        // BGM+E03+documentNumber'
        let kategorie = segment.get_element(0);
        if !kategorie.is_empty() {
            self.nachrichtendaten.kategorie = Some(kategorie.to_string());
        }
        let doc_nr = segment.get_element(1);
        if !doc_nr.is_empty() {
            self.nachrichtendaten.dokumentennummer = Some(doc_nr.to_string());
        }
    }

    /// Collects all built objects into a UtilmdTransaktion.
    fn collect_transaction(&mut self) -> UtilmdTransaktion {
        let prozessdaten = self.prozessdaten_mapper.build();
        let zeitscheiben = self.zeitscheibe_mapper.build();
        let marktlokationen = self
            .marktlokation_mapper
            .build()
            .into_iter()
            .collect();
        let messlokationen = self
            .messlokation_mapper
            .build()
            .into_iter()
            .collect();
        let netzlokationen = self
            .netzlokation_mapper
            .build()
            .into_iter()
            .collect();
        let parteien = self.geschaeftspartner_mapper.build();
        let vertrag = self.vertrag_mapper.build();
        let zaehler = self.zaehler_mapper.build();

        UtilmdTransaktion {
            transaktions_id: self.current_transaction_id.take().unwrap_or_default(),
            referenz_transaktions_id: None,
            absender: self.absender.clone(),
            empfaenger: self.empfaenger.clone(),
            prozessdaten,
            antwortstatus: None,
            zeitscheiben,
            marktlokationen,
            messlokationen,
            netzlokationen,
            steuerbare_ressourcen: Vec::new(),
            technische_ressourcen: Vec::new(),
            tranchen: Vec::new(),
            mabis_zaehlpunkte: Vec::new(),
            parteien,
            vertrag,
            bilanzierung: None,
            zaehler,
            produktpakete: Vec::new(),
            lokationszuordnungen: Vec::new(),
        }
    }

    /// Resets all mappers for a new transaction.
    fn reset_mappers(&mut self) {
        self.prozessdaten_mapper.reset();
        self.zeitscheibe_mapper.reset();
        self.marktlokation_mapper.reset();
        self.messlokation_mapper.reset();
        self.netzlokation_mapper.reset();
        self.geschaeftspartner_mapper.reset();
        self.vertrag_mapper.reset();
        self.zaehler_mapper.reset();
        self.in_transaction = false;
    }
}

impl<V: VersionConfig> Default for UtilmdCoordinator<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: VersionConfig> EdifactHandler for UtilmdCoordinator<V> {
    fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, _explicit_una: bool) {}

    fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
        // Extract sender/recipient from UNB
        let sender = unb.get_component(1, 0);
        if !sender.is_empty() {
            self.nachrichtendaten.absender_mp_id = Some(sender.to_string());
        }
        let recipient = unb.get_component(2, 0);
        if !recipient.is_empty() {
            self.nachrichtendaten.empfaenger_mp_id = Some(recipient.to_string());
        }
        let ref_nr = unb.get_element(4);
        if !ref_nr.is_empty() {
            self.nachrichtendaten.datenaustauschreferenz = Some(ref_nr.to_string());
        }
        Control::Continue
    }

    fn on_message_start(&mut self, unh: &RawSegment) -> Control {
        let msg_ref = unh.get_element(0);
        if !msg_ref.is_empty() {
            self.nachrichtendaten.nachrichtenreferenz = Some(msg_ref.to_string());
            self.context.set_message_reference(msg_ref);
        }
        Control::Continue
    }

    fn on_segment(&mut self, segment: &RawSegment) -> Control {
        match segment.id {
            "BGM" => self.handle_bgm(segment),
            "IDE" => self.handle_ide(segment),
            "NAD" => {
                let q = segment.get_element(0);
                if q == "MS" || q == "MR" {
                    self.handle_message_level_nad(segment);
                }
                // Also route to mappers (Geschaeftspartner handles party NADs)
                self.route_to_mappers(segment);
            }
            _ => {
                self.route_to_mappers(segment);
            }
        }
        Control::Continue
    }

    fn on_message_end(&mut self, _unt: &RawSegment) {
        // Collect the transaction if we have one
        if self.in_transaction || !self.prozessdaten_mapper.is_empty() {
            let tx = self.collect_transaction();
            self.transactions.push(tx);
            self.reset_mappers();
        }
    }

    fn on_interchange_end(&mut self, _unz: &RawSegment) {}
}

impl<V: VersionConfig> Coordinator for UtilmdCoordinator<V> {
    fn parse(&mut self, input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError> {
        edifact_parser::EdifactStreamParser::parse(input, self)?;
        Ok(std::mem::take(&mut self.transactions))
    }

    fn generate(
        &self,
        _transaktion: &UtilmdTransaktion,
    ) -> Result<Vec<u8>, AutomapperError> {
        // Will be implemented in Epic 8 (Writer)
        Ok(Vec::new())
    }

    fn format_version(&self) -> FormatVersion {
        V::VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{FV2504, FV2510};

    #[test]
    fn test_utilmd_coordinator_new() {
        let coord = UtilmdCoordinator::<FV2504>::new();
        assert_eq!(coord.format_version(), FormatVersion::FV2504);
        assert!(coord.transactions.is_empty());
    }

    #[test]
    fn test_utilmd_coordinator_fv2510() {
        let coord = UtilmdCoordinator::<FV2510>::new();
        assert_eq!(coord.format_version(), FormatVersion::FV2510);
    }

    #[test]
    fn test_utilmd_coordinator_parse_minimal() {
        let input = b"UNA:+.? 'UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'NAD+MS+9900123000002::293'NAD+MR+9900456000001::293'IDE+24+TXID001'LOC+Z16+DE00014545768S0000000000000003054'STS+E01+E01'UNT+8+MSG001'UNZ+1+REF001'";

        let mut coord = UtilmdCoordinator::<FV2504>::new();
        let result = coord.parse(input).unwrap();

        assert_eq!(result.len(), 1);
        let tx = &result[0];
        assert_eq!(tx.transaktions_id, "TXID001");
        assert_eq!(tx.absender.mp_id, Some("9900123000002".to_string()));
        assert_eq!(tx.empfaenger.mp_id, Some("9900456000001".to_string()));
        assert_eq!(tx.marktlokationen.len(), 1);
        assert_eq!(
            tx.marktlokationen[0].data.marktlokations_id,
            Some("DE00014545768S0000000000000003054".to_string())
        );
    }

    #[test]
    fn test_utilmd_coordinator_parse_with_messlokation() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R+251217:1229+REF'UNH+MSG+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC'IDE+24+TX1'LOC+Z16+MALO001'LOC+Z17+MELO001'LOC+Z18+NELO001'UNT+7+MSG'UNZ+1+REF'";

        let mut coord = UtilmdCoordinator::<FV2504>::new();
        let result = coord.parse(input).unwrap();

        assert_eq!(result.len(), 1);
        let tx = &result[0];
        assert_eq!(tx.marktlokationen.len(), 1);
        assert_eq!(tx.messlokationen.len(), 1);
        assert_eq!(tx.netzlokationen.len(), 1);
        assert_eq!(
            tx.messlokationen[0].data.messlokations_id,
            Some("MELO001".to_string())
        );
        assert_eq!(
            tx.netzlokationen[0].data.netzlokations_id,
            Some("NELO001".to_string())
        );
    }

    #[test]
    fn test_utilmd_coordinator_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<UtilmdCoordinator<FV2504>>();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_utilmd_coordinator`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/coordinator.rs` to use `UtilmdCoordinator` in `create_coordinator`:

Replace the `StubCoordinator` usage in `create_coordinator` with:

```rust
use crate::utilmd_coordinator::UtilmdCoordinator;
use crate::version::{FV2504, FV2510};

/// Creates a coordinator for the specified format version.
pub fn create_coordinator(fv: FormatVersion) -> Result<Box<dyn Coordinator>, AutomapperError> {
    match fv {
        FormatVersion::FV2504 => Ok(Box::new(UtilmdCoordinator::<FV2504>::new())),
        FormatVersion::FV2510 => Ok(Box::new(UtilmdCoordinator::<FV2510>::new())),
    }
}
```

Update `crates/automapper-core/src/lib.rs`:

```rust
//! Core automapper library.

pub mod context;
pub mod coordinator;
pub mod error;
pub mod mappers;
pub mod traits;
pub mod utilmd_coordinator;
pub mod version;

pub use context::TransactionContext;
pub use coordinator::{create_coordinator, detect_format_version, Coordinator};
pub use error::AutomapperError;
pub use traits::*;
pub use utilmd_coordinator::UtilmdCoordinator;
pub use version::{FV2504, FV2510, VersionConfig, VersionPhantom};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add UtilmdCoordinator with full mapper orchestration

UtilmdCoordinator<V> implements EdifactHandler and Coordinator. Routes
segments to all registered mappers (Prozessdaten, Zeitscheibe, Marktlokation,
Messlokation, Netzlokation, Geschaeftspartner, Vertrag, Zaehler).
Collects built objects into UtilmdTransaktion on message end.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Forward Mapping Integration Tests

**Files:**
- Create: `crates/automapper-core/tests/forward_mapping_test.rs`

**Step 1: Write the integration test**

Create `crates/automapper-core/tests/forward_mapping_test.rs`:

```rust
//! Integration tests for UTILMD forward mapping (EDIFACT -> BO4E).
//!
//! Tests parse synthetic but realistic EDIFACT messages and verify
//! the resulting UtilmdTransaktion structure.

use automapper_core::{create_coordinator, FormatVersion, UtilmdCoordinator, FV2504};

/// A synthetic UTILMD message with multiple entity types.
const SYNTHETIC_UTILMD: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+GEN0001'\
UNH+GEN0001MSG+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202506190130:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
STS+E01+E01::Z44'\
DTM+137:202507010000:303'\
DTM+471:202508010000:303'\
RFF+Z13:VORGANGS001'\
RFF+Z49:1'\
DTM+Z25:202507010000:303'\
DTM+Z26:202512310000:303'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z17+DE00098765432100000000000000012'\
LOC+Z18+NELO00000000001'\
NAD+Z04+9900999000003::293'\
FTX+ACB+++Testbemerkung'\
UNT+18+GEN0001MSG'\
UNZ+1+GEN0001'";

#[test]
fn test_forward_mapping_synthetic_utilmd() {
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let result = coord.parse(SYNTHETIC_UTILMD).unwrap();

    assert_eq!(result.len(), 1, "should produce one transaction");
    let tx = &result[0];

    // Transaction ID
    assert_eq!(tx.transaktions_id, "TXID001");

    // Absender/Empfaenger
    assert_eq!(tx.absender.mp_id, Some("9900123000002".to_string()));
    assert_eq!(tx.empfaenger.mp_id, Some("9900456000001".to_string()));

    // Prozessdaten
    assert_eq!(
        tx.prozessdaten.transaktionsgrund,
        Some("E01".to_string())
    );
    assert!(tx.prozessdaten.prozessdatum.is_some());
    assert!(tx.prozessdaten.wirksamkeitsdatum.is_some());
    assert_eq!(
        tx.prozessdaten.referenz_vorgangsnummer,
        Some("VORGANGS001".to_string())
    );
    assert_eq!(
        tx.prozessdaten.bemerkung,
        Some("Testbemerkung".to_string())
    );

    // Zeitscheiben
    assert_eq!(tx.zeitscheiben.len(), 1);
    assert_eq!(tx.zeitscheiben[0].zeitscheiben_id, "1");
    assert!(tx.zeitscheiben[0].gueltigkeitszeitraum.is_some());

    // Marktlokation
    assert_eq!(tx.marktlokationen.len(), 1);
    assert_eq!(
        tx.marktlokationen[0].data.marktlokations_id,
        Some("DE00014545768S0000000000000003054".to_string())
    );

    // Messlokation
    assert_eq!(tx.messlokationen.len(), 1);
    assert_eq!(
        tx.messlokationen[0].data.messlokations_id,
        Some("DE00098765432100000000000000012".to_string())
    );

    // Netzlokation
    assert_eq!(tx.netzlokationen.len(), 1);
    assert_eq!(
        tx.netzlokationen[0].data.netzlokations_id,
        Some("NELO00000000001".to_string())
    );

    // Geschaeftspartner
    assert_eq!(tx.parteien.len(), 1);
    assert_eq!(
        tx.parteien[0].edifact.nad_qualifier,
        Some("Z04".to_string())
    );
}

#[test]
fn test_forward_mapping_via_create_coordinator() {
    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let result = coord.parse(SYNTHETIC_UTILMD).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].transaktions_id, "TXID001");
}

#[test]
fn test_forward_mapping_empty_input() {
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let result = coord.parse(b"").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_forward_mapping_nad_sparte() {
    let input = b"UNA:+.? 'UNB+UNOC:3+S+R+D+REF'UNH+M+UTILMD:D:11A:UN:S2.1'BGM+E03+D'NAD+MS+9900123::293'IDE+24+TX1'LOC+Z16+MALO1'UNT+6+M'UNZ+1+REF'";
    let mut coord = UtilmdCoordinator::<FV2504>::new();
    let result = coord.parse(input).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].marktlokationen[0].data.sparte,
        Some("STROM".to_string())
    );
}
```

**Step 2: Run integration test**

Run: `cargo test -p automapper-core --test forward_mapping_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/automapper-core/tests/
git commit -m "$(cat <<'EOF'
test(automapper-core): add forward mapping integration tests

Tests the complete EDIFACT -> BO4E pipeline with synthetic UTILMD
messages. Verifies transaction extraction, Prozessdaten, Zeitscheiben,
Marktlokation, Messlokation, Netzlokation, and Geschaeftspartner mapping.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```
