---
feature: missing-entity-mappers
epic: 1
title: "LOC-Based Entity Mappers"
depends_on: []
estimated_tasks: 5
crate: automapper-core
status: in_progress
---

# Epic 1: LOC-Based Entity Mappers

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-core/src/`. All code must compile with `cargo check -p automapper-core`.

**Goal:** Implement 4 LOC-based entity mappers (SteuerbareRessource, TechnischeRessource, Tranche, MabisZaehlpunkt) and their corresponding writers, following the NetzlokationMapper pattern exactly.

**Architecture:** Each mapper handles a single LOC+qualifier segment, extracts the location ID from the composite element, and builds a `WithValidity<T, TEdifact>`. Each writer emits a single `LOC+qualifier+id'` segment. All 4 are structurally identical to NetzlokationMapper/NetzlokationWriter, differing only in qualifier and types.

**Tech Stack:** Rust, edifact-types, bo4e-extensions, automapper-core traits

---

## Task 1: SteuerbareRessourceMapper — LOC+Z19

**Files:**
- Create: `crates/automapper-core/src/mappers/steuerbare_ressource.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/mappers/steuerbare_ressource.rs`:

```rust
//! Mapper for SteuerbareRessource (controllable resource) business objects.
//!
//! Handles LOC+Z19 segments for controllable resource identification.

use bo4e_extensions::{SteuerbareRessource, SteuerbareRessourceEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for SteuerbareRessource in UTILMD messages.
pub struct SteuerbareRessourceMapper {
    steuerbare_ressource_id: Option<String>,
    edifact: SteuerbareRessourceEdifact,
    has_data: bool,
}

impl SteuerbareRessourceMapper {
    pub fn new() -> Self {
        Self {
            steuerbare_ressource_id: None,
            edifact: SteuerbareRessourceEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for SteuerbareRessourceMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for SteuerbareRessourceMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z19"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.steuerbare_ressource_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>>
    for SteuerbareRessourceMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(
        &mut self,
    ) -> Option<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>> {
        if !self.has_data {
            return None;
        }
        let sr = SteuerbareRessource {
            steuerbare_ressource_id: self.steuerbare_ressource_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: sr,
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
    fn test_steuerbare_ressource_mapper_loc_z19() {
        let mut mapper = SteuerbareRessourceMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z19"], vec!["STRES001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.steuerbare_ressource_id,
            Some("STRES001".to_string())
        );
    }

    #[test]
    fn test_steuerbare_ressource_mapper_ignores_other_loc() {
        let mapper = SteuerbareRessourceMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_steuerbare_ressource_mapper_empty_build() {
        let mut mapper = SteuerbareRessourceMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod steuerbare_ressource;
pub use steuerbare_ressource::SteuerbareRessourceMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core steuerbare_ressource`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/steuerbare_ressource.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add SteuerbareRessourceMapper for LOC+Z19"
```

---

## Task 2: TechnischeRessourceMapper — LOC+Z20

**Files:**
- Create: `crates/automapper-core/src/mappers/technische_ressource.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the mapper file**

Create `crates/automapper-core/src/mappers/technische_ressource.rs`:

```rust
//! Mapper for TechnischeRessource (technical resource) business objects.
//!
//! Handles LOC+Z20 segments for technical resource identification.

use bo4e_extensions::{TechnischeRessource, TechnischeRessourceEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for TechnischeRessource in UTILMD messages.
pub struct TechnischeRessourceMapper {
    technische_ressource_id: Option<String>,
    edifact: TechnischeRessourceEdifact,
    has_data: bool,
}

impl TechnischeRessourceMapper {
    pub fn new() -> Self {
        Self {
            technische_ressource_id: None,
            edifact: TechnischeRessourceEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for TechnischeRessourceMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for TechnischeRessourceMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z20"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.technische_ressource_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>>>
    for TechnischeRessourceMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(
        &mut self,
    ) -> Option<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>> {
        if !self.has_data {
            return None;
        }
        let tr = TechnischeRessource {
            technische_ressource_id: self.technische_ressource_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: tr,
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
    fn test_technische_ressource_mapper_loc_z20() {
        let mut mapper = TechnischeRessourceMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z20"], vec!["TECRES001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(
            result.data.technische_ressource_id,
            Some("TECRES001".to_string())
        );
    }

    #[test]
    fn test_technische_ressource_mapper_ignores_other_loc() {
        let mapper = TechnischeRessourceMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_technische_ressource_mapper_empty_build() {
        let mut mapper = TechnischeRessourceMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod technische_ressource;
pub use technische_ressource::TechnischeRessourceMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core technische_ressource`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/technische_ressource.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add TechnischeRessourceMapper for LOC+Z20"
```

---

## Task 3: TrancheMapper — LOC+Z21

**Files:**
- Create: `crates/automapper-core/src/mappers/tranche.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the mapper file**

Create `crates/automapper-core/src/mappers/tranche.rs`:

```rust
//! Mapper for Tranche business objects.
//!
//! Handles LOC+Z21 segments for tranche identification.

use bo4e_extensions::{Tranche, TrancheEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for Tranche in UTILMD messages.
pub struct TrancheMapper {
    tranche_id: Option<String>,
    edifact: TrancheEdifact,
    has_data: bool,
}

impl TrancheMapper {
    pub fn new() -> Self {
        Self {
            tranche_id: None,
            edifact: TrancheEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for TrancheMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for TrancheMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z21"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.tranche_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<Tranche, TrancheEdifact>>> for TrancheMapper {
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<Tranche, TrancheEdifact>> {
        if !self.has_data {
            return None;
        }
        let t = Tranche {
            tranche_id: self.tranche_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: t,
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
    fn test_tranche_mapper_loc_z21() {
        let mut mapper = TrancheMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z21"], vec!["TRANCHE001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.tranche_id, Some("TRANCHE001".to_string()));
    }

    #[test]
    fn test_tranche_mapper_ignores_other_loc() {
        let mapper = TrancheMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z16"], vec!["MALO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_tranche_mapper_empty_build() {
        let mut mapper = TrancheMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod tranche;
pub use tranche::TrancheMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core tranche`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/tranche.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add TrancheMapper for LOC+Z21"
```

---

## Task 4: MabisZaehlpunktMapper — LOC+Z15

**Files:**
- Create: `crates/automapper-core/src/mappers/mabis_zaehlpunkt.rs`
- Modify: `crates/automapper-core/src/mappers/mod.rs`

**Step 1: Write the mapper file**

Create `crates/automapper-core/src/mappers/mabis_zaehlpunkt.rs`:

```rust
//! Mapper for MabisZaehlpunkt (MaBiS metering point) business objects.
//!
//! Handles LOC+Z15 segments for MaBiS metering point identification.

use bo4e_extensions::{MabisZaehlpunkt, MabisZaehlpunktEdifact, WithValidity};
use edifact_types::RawSegment;

use crate::context::TransactionContext;
use crate::traits::{Builder, SegmentHandler};

/// Mapper for MabisZaehlpunkt in UTILMD messages.
pub struct MabisZaehlpunktMapper {
    zaehlpunkt_id: Option<String>,
    edifact: MabisZaehlpunktEdifact,
    has_data: bool,
}

impl MabisZaehlpunktMapper {
    pub fn new() -> Self {
        Self {
            zaehlpunkt_id: None,
            edifact: MabisZaehlpunktEdifact::default(),
            has_data: false,
        }
    }
}

impl Default for MabisZaehlpunktMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentHandler for MabisZaehlpunktMapper {
    fn can_handle(&self, segment: &RawSegment) -> bool {
        segment.id == "LOC" && segment.get_element(0) == "Z15"
    }

    fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
        if segment.id == "LOC" {
            let id = segment.get_component(1, 0);
            if !id.is_empty() {
                self.zaehlpunkt_id = Some(id.to_string());
                self.has_data = true;
            }
        }
    }
}

impl Builder<Option<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>>>
    for MabisZaehlpunktMapper
{
    fn is_empty(&self) -> bool {
        !self.has_data
    }

    fn build(&mut self) -> Option<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>> {
        if !self.has_data {
            return None;
        }
        let mz = MabisZaehlpunkt {
            zaehlpunkt_id: self.zaehlpunkt_id.take(),
        };
        let edifact = std::mem::take(&mut self.edifact);
        self.has_data = false;
        Some(WithValidity {
            data: mz,
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
    fn test_mabis_zaehlpunkt_mapper_loc_z15() {
        let mut mapper = MabisZaehlpunktMapper::new();
        let mut ctx = TransactionContext::new("FV2504");
        let loc = RawSegment::new("LOC", vec![vec!["Z15"], vec!["MABIS001"]], pos());
        assert!(mapper.can_handle(&loc));
        mapper.handle(&loc, &mut ctx);
        let result = mapper.build().unwrap();
        assert_eq!(result.data.zaehlpunkt_id, Some("MABIS001".to_string()));
    }

    #[test]
    fn test_mabis_zaehlpunkt_mapper_ignores_other_loc() {
        let mapper = MabisZaehlpunktMapper::new();
        let loc = RawSegment::new("LOC", vec![vec!["Z18"], vec!["NELO001"]], pos());
        assert!(!mapper.can_handle(&loc));
    }

    #[test]
    fn test_mabis_zaehlpunkt_mapper_empty_build() {
        let mut mapper = MabisZaehlpunktMapper::new();
        assert!(mapper.is_empty());
        assert!(mapper.build().is_none());
    }
}
```

**Step 2: Add module to `mappers/mod.rs`**

Add to `crates/automapper-core/src/mappers/mod.rs`:

```rust
pub mod mabis_zaehlpunkt;
pub use mabis_zaehlpunkt::MabisZaehlpunktMapper;
```

**Step 3: Run test to verify it passes**

Run: `cargo test -p automapper-core mabis_zaehlpunkt`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/mappers/mabis_zaehlpunkt.rs crates/automapper-core/src/mappers/mod.rs
git commit -m "feat(automapper-core): add MabisZaehlpunktMapper for LOC+Z15"
```

---

## Task 5: LOC-Based Entity Writers

**Files:**
- Modify: `crates/automapper-core/src/writer/entity_writers.rs`

**Step 1: Write failing tests for all 4 writers**

Add these test cases to the `#[cfg(test)] mod tests` block in `entity_writers.rs`:

```rust
#[test]
fn test_steuerbare_ressource_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let sr = WithValidity {
        data: SteuerbareRessource {
            steuerbare_ressource_id: Some("STRES001".to_string()),
        },
        edifact: SteuerbareRessourceEdifact::default(),
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    SteuerbareRessourceWriter::write(&mut doc, &sr);
    doc.end_message();
    doc.end_interchange();

    assert!(doc.output().contains("LOC+Z19+STRES001'"));
}

#[test]
fn test_technische_ressource_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let tr = WithValidity {
        data: TechnischeRessource {
            technische_ressource_id: Some("TECRES001".to_string()),
        },
        edifact: TechnischeRessourceEdifact::default(),
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    TechnischeRessourceWriter::write(&mut doc, &tr);
    doc.end_message();
    doc.end_interchange();

    assert!(doc.output().contains("LOC+Z20+TECRES001'"));
}

#[test]
fn test_tranche_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let t = WithValidity {
        data: Tranche {
            tranche_id: Some("TRANCHE001".to_string()),
        },
        edifact: TrancheEdifact::default(),
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    TrancheWriter::write(&mut doc, &t);
    doc.end_message();
    doc.end_interchange();

    assert!(doc.output().contains("LOC+Z21+TRANCHE001'"));
}

#[test]
fn test_mabis_zaehlpunkt_writer() {
    let mut doc = EdifactDocumentWriter::with_delimiters(EdifactDelimiters::default(), false);
    doc.begin_interchange("S", "R", "REF", "D", "T");
    doc.begin_message("M", "TYPE");

    let mz = WithValidity {
        data: MabisZaehlpunkt {
            zaehlpunkt_id: Some("MABIS001".to_string()),
        },
        edifact: MabisZaehlpunktEdifact::default(),
        gueltigkeitszeitraum: None,
        zeitscheibe_ref: None,
    };

    MabisZaehlpunktWriter::write(&mut doc, &mz);
    doc.end_message();
    doc.end_interchange();

    assert!(doc.output().contains("LOC+Z15+MABIS001'"));
}
```

**Step 2: Write the 4 writer structs**

Add these before the `#[cfg(test)]` block in `entity_writers.rs`:

```rust
/// Writes a SteuerbareRessource to EDIFACT segments.
pub struct SteuerbareRessourceWriter;

impl SteuerbareRessourceWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        sr: &WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>,
    ) {
        if let Some(ref id) = sr.data.steuerbare_ressource_id {
            doc.write_segment("LOC", &["Z19", id]);
        }
    }
}

/// Writes a TechnischeRessource to EDIFACT segments.
pub struct TechnischeRessourceWriter;

impl TechnischeRessourceWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        tr: &WithValidity<TechnischeRessource, TechnischeRessourceEdifact>,
    ) {
        if let Some(ref id) = tr.data.technische_ressource_id {
            doc.write_segment("LOC", &["Z20", id]);
        }
    }
}

/// Writes a Tranche to EDIFACT segments.
pub struct TrancheWriter;

impl TrancheWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        t: &WithValidity<Tranche, TrancheEdifact>,
    ) {
        if let Some(ref id) = t.data.tranche_id {
            doc.write_segment("LOC", &["Z21", id]);
        }
    }
}

/// Writes a MabisZaehlpunkt to EDIFACT segments.
pub struct MabisZaehlpunktWriter;

impl MabisZaehlpunktWriter {
    pub fn write(
        doc: &mut EdifactDocumentWriter,
        mz: &WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>,
    ) {
        if let Some(ref id) = mz.data.zaehlpunkt_id {
            doc.write_segment("LOC", &["Z15", id]);
        }
    }
}
```

**Step 3: Update writer `mod.rs` re-exports**

Add to `crates/automapper-core/src/writer/mod.rs`:

```rust
pub use entity_writers::{
    SteuerbareRessourceWriter, TechnischeRessourceWriter, TrancheWriter, MabisZaehlpunktWriter,
};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p automapper-core entity_writers`
Expected: All writer tests PASS (existing + 4 new)

**Step 5: Commit**

```bash
git add crates/automapper-core/src/writer/entity_writers.rs crates/automapper-core/src/writer/mod.rs
git commit -m "feat(automapper-core): add writers for LOC-based entities (Z19, Z20, Z21, Z15)"
```
