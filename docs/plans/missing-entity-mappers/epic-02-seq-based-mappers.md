---
feature: missing-entity-mappers
epic: 2
title: "SEQ-Based Entity Mappers"
depends_on: []
estimated_tasks: 4
crate: automapper-core
status: in_progress
---

# Epic 2: SEQ-Based Entity Mappers

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-core/src/`. All code must compile with `cargo check -p automapper-core`.

**Goal:** Implement 3 SEQ-based entity mappers (Produktpaket, Lokationszuordnung, Bilanzierung) and their corresponding writers, following the VertragMapper/ZaehlerMapper pattern with SEQ context tracking.

**Architecture:** SEQ-based mappers track which SEQ group they're currently inside using a boolean flag. Subordinate segments (PIA, RFF, CCI, QTY) are only processed when the context flag is active. A new SEQ segment resets the context. Produktpaket uses `Builder<Vec<...>>` (multiple possible), Lokationszuordnung uses `Builder<Vec<...>>`, and Bilanzierung uses `Builder<Option<...>>` (at most one per transaction).

**Tech Stack:** Rust, edifact-types, bo4e-extensions, automapper-core traits

---

## Task 1: ProduktpaketMapper — SEQ+Z79

**Files:**
- Create: `crates/automapper-core/src/mappers/produktpaket.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the mapper file**

Create `crates/automapper-core/src/mappers/produktpaket.rs`:

```rust
//! Mapper for Produktpaket (product package) business objects.
//!
//! Handles SEQ+Z79 group and PIA segments for product package data.

use bo4e_extensions::{Produktpaket, ProduktpaketEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Produktpaket in UTILMD messages.
///
/// Handles SEQ+Z79 for product package identification. PIA segments within
/// the Z79 context contain the product name.
///
/// Note: ZaehlerMapper also handles SEQ+Z79 for its own produktpaket_id
/// reference. Both mappers receive the segment; this is fine because
/// `route_to_mappers` sends to all matching handlers.
pub struct ProduktpaketMapper {
    produktpaket_id: Option<String>,
    edifact: ProduktpaketEdifact,
    has_data: bool,
    in_seq_z79: bool,
    items: Vec<WithValidity<Produktpaket, ProduktpaketEdifact>>,
}

impl ProduktpaketMapper {
    pub fn new() -> Self {
        Self {
            produktpaket_id: None,
            edifact: ProduktpaketEdifact::default(),
            has_data: false,
            in_seq_z79: false,
            items: Vec::new(),
        }
    }

    /// Finalizes the current item (if any) and pushes it to the items list.
    fn finalize_current(&mut self) {
        if self.has_data {
            let pp = Produktpaket {
                produktpaket_id: self.produktpaket_id.take(),
            };
            let edifact = std::mem::take(&mut self.edifact);
            self.items.push(WithValidity {
                data: pp,
                edifact,
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            });
            self.has_data = false;
        }
    }
}

impl Default for ProduktpaketMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for ProduktpaketMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        match segment.id {
            "SEQ" => {
                let q = segment.get_element(0);
                q == "Z79"
            }
            "PIA" => self.in_seq_z79,
            _ => false,
        }
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        match segment.id {
            "SEQ" => {
                let qualifier = segment.get_element(0);
                if qualifier == "Z79" {
                    // Finalize previous item before starting new one
                    self.finalize_current();
                    self.in_seq_z79 = true;
                    // Extract produktpaket_id from SEQ element if present
                    let ref_val = segment.get_element(1);
                    if !ref_val.is_empty() {
                        self.produktpaket_id = Some(ref_val.to_string());
                        self.has_data = true;
                    }
                } else {
                    self.in_seq_z79 = false;
                }
            }
            "PIA" => {
                if !self.in_seq_z79 {
                    return;
                }
                // PIA+5+name' -> product name
                let qualifier = segment.get_element(0);
                if qualifier == "5" {
                    let name = segment.get_component(1, 0);
                    if !name.is_empty() {
                        self.edifact.produktpaket_name = Some(name.to_string());
                        self.has_data = true;
                    }
                }
            }
            _ => {}
        }
    }
}

impl Builder<Vec<WithValidity<Produktpaket, ProduktpaketEdifact>>> for ProduktpaketMapper {
    fn is_empty(&self) -> bool {
        !self.has_data && self.items.is_empty()
    }

    fn build(&mut self) -> Vec<WithValidity<Produktpaket, ProduktpaketEdifact>> {
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
    fn test_produktpaket_mapper_seq_z79_with_pia() {
        let mut mapper = ProduktpaketMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z79"], vec!["PP001"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("PIA", vec![vec!["5"], vec!["Grundversorgung"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.produktpaket_id, Some("PP001".to_string()));
        assert_eq!(
            result[0].edifact.produktpaket_name,
            Some("Grundversorgung".to_string())
        );
    }

    #[test]
    fn test_produktpaket_mapper_ignores_pia_outside_z79() {
        let mut mapper = ProduktpaketMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        // Set context to Z03 (not Z79)
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z03"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("PIA", vec![vec!["5"], vec!["SomeProduct"]], pos()),
            &mut ctx,
        );
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_produktpaket_mapper_empty_build() {
        let mut mapper = ProduktpaketMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_empty());
    }

    #[test]
    fn test_produktpaket_mapper_seq_z79_no_pia() {
        let mut mapper = ProduktpaketMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z79"], vec!["PP002"]], pos()),
            &mut ctx,
        );
        let result = mapper.build();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data.produktpaket_id, Some("PP002".to_string()));
        assert!(result[0].edifact.produktpaket_name.is_none());
    }
}
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod produktpaket;
pub use produktpaket::ProduktpaketMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core produktpaket`
Expected: 4 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/produktpaket.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add ProduktpaketMapper for SEQ+Z79"
```

---

## Task 2: LokationszuordnungMapper — SEQ+Z78

**Files:**
- Create: `crates/automapper-core/src/mappers/lokationszuordnung.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the mapper file**

Create `crates/automapper-core/src/mappers/lokationszuordnung.rs`:

```rust
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
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z78"]], pos()),
            &mut ctx,
        );
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
        assert_eq!(
            result[0].data.messlokations_id,
            Some("MELO001".to_string())
        );
    }

    #[test]
    fn test_lokationszuordnung_mapper_ignores_rff_outside_z78() {
        let mut mapper = LokationszuordnungMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z03"]], pos()),
            &mut ctx,
        );
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
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod lokationszuordnung;
pub use lokationszuordnung::LokationszuordnungMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core lokationszuordnung`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/lokationszuordnung.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add LokationszuordnungMapper for SEQ+Z78"
```

---

## Task 3: BilanzierungMapper — SEQ+Z98/Z81

**Files:**
- Create: `crates/automapper-core/src/mappers/bilanzierung.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the mapper file**

Create `crates/automapper-core/src/mappers/bilanzierung.rs`:

```rust
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
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z98"]], pos()),
            &mut ctx,
        );
        mapper.handle(
            &RawSegment::new("CCI", vec![vec!["Z20"], vec![], vec!["11YN20---------Z"]], pos()),
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
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z98"]], pos()),
            &mut ctx,
        );
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
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z81"]], pos()),
            &mut ctx,
        );
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
        mapper.handle(
            &RawSegment::new("SEQ", vec![vec!["Z03"]], pos()),
            &mut ctx,
        );
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
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod bilanzierung;
pub use bilanzierung::BilanzierungMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core bilanzierung`
Expected: 5 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/bilanzierung.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add BilanzierungMapper for SEQ+Z98/Z81"
```

---

## Task 4: SEQ-Based Entity Writers

**Files:**
- Modify: `crates/automapper-core/src/writer/entity_writers.rs`
- Modify: `crates/automapper-core/src/writer/mod.rs`

**Step 1: Write failing tests for all 3 writers**

Add these test cases to the `#[cfg(test)] mod tests` block in `entity_writers.rs`:

```rust
#[test]
fn test_produktpaket_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let pp = WithValidity {
        data: Produktpaket {
            produktpaket_id: Some("PP001".to_string()),
        },
        edifact: ProduktpaketEdifact {
            produktpaket_name: Some("Grundversorgung".to_string()),
        },
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    ProduktpaketWriter::write(&mut doc, &pp);
    doc.end_message();
    doc.end_interchange();

    let output = doc.output();
    assert!(output.contains("SEQ+Z79+PP001'"));
    assert!(output.contains("PIA+5+Grundversorgung'"));
}

#[test]
fn test_lokationszuordnung_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let lz = WithValidity {
        data: Lokationszuordnung {
            marktlokations_id: Some("MALO001".to_string()),
            messlokations_id: Some("MELO001".to_string()),
        },
        edifact: LokationszuordnungEdifact::default(),
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    LokationszuordnungWriter::write(&mut doc, &lz);
    doc.end_message();
    doc.end_interchange();

    let output = doc.output();
    assert!(output.contains("SEQ+Z78'"));
    assert!(output.contains("RFF+Z18:MALO001'"));
    assert!(output.contains("RFF+Z19:MELO001'"));
}

#[test]
fn test_bilanzierung_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let b = WithValidity {
        data: Bilanzierung {
            bilanzkreis: Some("11YN20---------Z".to_string()),
            regelzone: None,
            bilanzierungsgebiet: None,
        },
        edifact: BilanzierungEdifact {
            jahresverbrauchsprognose: Some(12345.67),
            temperatur_arbeit: None,
        },
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    BilanzierungWriter::write(&mut doc, &b);
    doc.end_message();
    doc.end_interchange();

    let output = doc.output();
    assert!(output.contains("SEQ+Z98'"));
    assert!(output.contains("CCI+Z20++11YN20---------Z'"));
    assert!(output.contains("QTY+Z09:12345.67'"));
}
```

**Step 2: Write the 3 writer structs**

Add these before the `#[cfg(test)]` block in `entity_writers.rs`:

```rust
/// Writes a Produktpaket to EDIFACT segments.
pub struct ProduktpaketWriter;

impl ProduktpaketWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        pp: &WithValidity<Produktpaket, ProduktpaketEdifact>,
    ) {
        // SEQ+Z79+produktpaket_id'
        let id = pp.data.produktpaket_id.as_deref().unwrap_or("");
        doc.write_segment("SEQ", &["Z79", id]);

        // PIA+5+produktpaket_name'
        if let Some(ref name) = pp.edifact.produktpaket_name {
            doc.write_segment("PIA", &["5", name]);
        }
    }
}

/// Writes a Lokationszuordnung to EDIFACT segments.
pub struct LokationszuordnungWriter;

impl LokationszuordnungWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        lz: &WithValidity<Lokationszuordnung, LokationszuordnungEdifact>,
    ) {
        // SEQ+Z78'
        doc.write_segment("SEQ", &["Z78"]);

        // RFF+Z18:marktlokations_id'
        if let Some(ref malo_id) = lz.data.marktlokations_id {
            doc.write_segment_with_composites("RFF", &[&["Z18", malo_id]]);
        }

        // RFF+Z19:messlokations_id'
        if let Some(ref melo_id) = lz.data.messlokations_id {
            doc.write_segment_with_composites("RFF", &[&["Z19", melo_id]]);
        }
    }
}

/// Writes Bilanzierung data to EDIFACT segments.
pub struct BilanzierungWriter;

impl BilanzierungWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        b: &WithValidity<Bilanzierung, BilanzierungEdifact>,
    ) {
        // SEQ+Z98'
        doc.write_segment("SEQ", &["Z98"]);

        // CCI+Z20++bilanzkreis'
        if let Some(ref bk) = b.data.bilanzkreis {
            doc.write_segment("CCI", &["Z20", "", bk]);
        }

        // CCI+Z21++regelzone'
        if let Some(ref rz) = b.data.regelzone {
            doc.write_segment("CCI", &["Z21", "", rz]);
        }

        // CCI+Z22++bilanzierungsgebiet'
        if let Some(ref bg) = b.data.bilanzierungsgebiet {
            doc.write_segment("CCI", &["Z22", "", bg]);
        }

        // QTY+Z09:jahresverbrauchsprognose'
        if let Some(jvp) = b.edifact.jahresverbrauchsprognose {
            let value = format!("{jvp}");
            doc.write_segment_with_composites("QTY", &[&["Z09", &value]]);
        }

        // QTY+265:temperatur_arbeit'
        if let Some(ta) = b.edifact.temperatur_arbeit {
            let value = format!("{ta}");
            doc.write_segment_with_composites("QTY", &[&["265", &value]]);
        }
    }
}
```

**Step 3: Update writer `mod.rs` re-exports**

Add to the existing re-exports in `crates/automapper-core/src/writer/mod.rs`:

```rust
pub use entity_writers::{ProduktpaketWriter, LokationszuordnungWriter, BilanzierungWriter};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p automapper-core entity_writers`
Expected: All writer tests PASS (existing + 3 new)

**Step 5: Commit**

```bash
git add crates/automapper-core/src/writer/entity_writers.rs crates/automapper-core/src/writer/mod.rs
git commit -m "feat(automapper-core): add writers for SEQ-based entities (Z79, Z78, Z98)"
```
