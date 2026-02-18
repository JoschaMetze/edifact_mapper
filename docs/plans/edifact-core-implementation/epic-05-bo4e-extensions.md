---
feature: edifact-core-implementation
epic: 5
title: "bo4e-extensions Crate"
depends_on: [edifact-core-implementation/E01]
estimated_tasks: 6
crate: bo4e-extensions
status: in_progress
---

# Epic 5: bo4e-extensions Crate

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/bo4e-extensions/src/`. All code must compile with `cargo check -p bo4e-extensions`.

**Goal:** Implement the `bo4e-extensions` crate with all domain types: `WithValidity<T, E>`, `Zeitraum`, EDIFACT companion types (`MarktlokationEdifact`, `ZaehlerEdifact`, etc.), `DataQuality`, `UtilmdNachricht`/`UtilmdTransaktion` containers, `Prozessdaten`, `Nachrichtendaten`, `Zeitscheibe`, `Bo4eUri`, and `LinkRegistry`.

**Architecture:** This crate bridges the standard BO4E types (from the `bo4e` crate, or defined here as placeholder structs until the external crate is available) with EDIFACT-specific functional domain data. The companion `*Edifact` types store data that exists in EDIFACT but has no home in standard BO4E. All types derive `Serialize`/`Deserialize` for JSON contract support. See design doc section 4.

**Tech Stack:** Rust, serde + serde_json, chrono, insta for snapshot testing

---

## Task 1: Placeholder BO4E Types

**Files:**
- Create: `crates/bo4e-extensions/src/bo4e_types.rs`
- Modify: `crates/bo4e-extensions/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/bo4e-extensions/src/bo4e_types.rs`:

```rust
//! Placeholder BO4E types.
//!
//! These will be replaced by imports from the `bo4e` crate once available.
//! For now, we define minimal structs that satisfy the API contract.

use serde::{Deserialize, Serialize};

/// Marktlokation — a market location in the German energy market.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Marktlokation {
    pub marktlokations_id: Option<String>,
    pub sparte: Option<String>,
    pub lokationsadresse: Option<Adresse>,
    pub bilanzierungsmethode: Option<String>,
    pub netzebene: Option<String>,
}

/// Messlokation — a metering location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Messlokation {
    pub messlokations_id: Option<String>,
    pub sparte: Option<String>,
    pub messlokationszaehler: Option<Vec<String>>,
}

/// Netzlokation — a network location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Netzlokation {
    pub netzlokations_id: Option<String>,
    pub sparte: Option<String>,
}

/// SteuerbareRessource — a controllable resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SteuerbareRessource {
    pub steuerbare_ressource_id: Option<String>,
}

/// TechnischeRessource — a technical resource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnischeRessource {
    pub technische_ressource_id: Option<String>,
}

/// Tranche — a tranche.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tranche {
    pub tranche_id: Option<String>,
}

/// MabisZaehlpunkt — a MaBiS metering point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MabisZaehlpunkt {
    pub zaehlpunkt_id: Option<String>,
}

/// Geschaeftspartner — a business partner.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Geschaeftspartner {
    pub name1: Option<String>,
    pub name2: Option<String>,
    pub gewerbekennzeichnung: Option<String>,
    pub geschaeftspartner_rolle: Option<Vec<String>>,
    pub partneradresse: Option<Adresse>,
}

/// Vertrag — a contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vertrag {
    pub vertragsnummer: Option<String>,
    pub vertragsart: Option<String>,
    pub vertragsbeginn: Option<String>,
    pub vertragsende: Option<String>,
}

/// Bilanzierung — balancing data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Bilanzierung {
    pub bilanzkreis: Option<String>,
    pub regelzone: Option<String>,
    pub bilanzierungsgebiet: Option<String>,
}

/// Zaehler — a meter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zaehler {
    pub zaehlernummer: Option<String>,
    pub zaehlertyp: Option<String>,
    pub sparte: Option<String>,
}

/// Produktpaket — a product package.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Produktpaket {
    pub produktpaket_id: Option<String>,
}

/// Lokationszuordnung — a location assignment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lokationszuordnung {
    pub marktlokations_id: Option<String>,
    pub messlokations_id: Option<String>,
}

/// Marktteilnehmer — a market participant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Marktteilnehmer {
    pub mp_id: Option<String>,
    pub marktrolle: Option<String>,
}

/// Adresse — a postal address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Adresse {
    pub strasse: Option<String>,
    pub hausnummer: Option<String>,
    pub postleitzahl: Option<String>,
    pub ort: Option<String>,
    pub landescode: Option<String>,
}

/// Zaehlwerk — a meter register.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Zaehlwerk {
    pub obis_kennzahl: Option<String>,
    pub bezeichnung: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marktlokation_default() {
        let ml = Marktlokation::default();
        assert!(ml.marktlokations_id.is_none());
    }

    #[test]
    fn test_marktlokation_serde_roundtrip() {
        let ml = Marktlokation {
            marktlokations_id: Some("DE00014545768S0000000000000003054".to_string()),
            sparte: Some("STROM".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&ml).unwrap();
        let deserialized: Marktlokation = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.marktlokations_id,
            Some("DE00014545768S0000000000000003054".to_string())
        );
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_marktlokation`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

Update `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.

pub mod bo4e_types;

pub use bo4e_types::*;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/
git commit -m "$(cat <<'EOF'
feat(bo4e-extensions): add placeholder BO4E types with serde

Minimal structs for Marktlokation, Messlokation, Zaehler, Vertrag,
Geschaeftspartner, etc. All derive Serialize/Deserialize.
Will be replaced by the bo4e crate when available.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Zeitraum and DataQuality

**Files:**
- Create: `crates/bo4e-extensions/src/zeitraum.rs`
- Create: `crates/bo4e-extensions/src/data_quality.rs`
- Modify: `crates/bo4e-extensions/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/bo4e-extensions/src/zeitraum.rs`:

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// A time period with optional start and end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zeitraum {
    pub von: Option<NaiveDateTime>,
    pub bis: Option<NaiveDateTime>,
}

impl Zeitraum {
    /// Creates a new Zeitraum with the given start and end.
    pub fn new(von: Option<NaiveDateTime>, bis: Option<NaiveDateTime>) -> Self {
        Self { von, bis }
    }

    /// Returns true if this Zeitraum has both start and end set.
    pub fn is_bounded(&self) -> bool {
        self.von.is_some() && self.bis.is_some()
    }

    /// Returns true if neither start nor end is set.
    pub fn is_empty(&self) -> bool {
        self.von.is_none() && self.bis.is_none()
    }
}

impl Default for Zeitraum {
    fn default() -> Self {
        Self { von: None, bis: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_zeitraum_new() {
        let von = NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let bis = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let z = Zeitraum::new(Some(von), Some(bis));
        assert!(z.is_bounded());
        assert!(!z.is_empty());
    }

    #[test]
    fn test_zeitraum_default_is_empty() {
        let z = Zeitraum::default();
        assert!(z.is_empty());
        assert!(!z.is_bounded());
    }

    #[test]
    fn test_zeitraum_serde_roundtrip() {
        let von = NaiveDate::from_ymd_opt(2025, 6, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let z = Zeitraum::new(Some(von), None);
        let json = serde_json::to_string(&z).unwrap();
        let deserialized: Zeitraum = serde_json::from_str(&json).unwrap();
        assert_eq!(z, deserialized);
    }
}
```

Create `crates/bo4e-extensions/src/data_quality.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Data quality indicator for EDIFACT domain objects.
///
/// Indicates the completeness/reliability of data attached to a location
/// or other entity in the UTILMD message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataQuality {
    /// Complete data — all fields are present and verified.
    Vollstaendig,
    /// Expected data — fields are present but not yet confirmed.
    Erwartet,
    /// Data already exists in the system.
    ImSystemVorhanden,
    /// Informative only — data is provided for reference.
    Informativ,
}

impl DataQuality {
    /// Converts from an EDIFACT qualifier string.
    pub fn from_qualifier(qualifier: &str) -> Option<Self> {
        match qualifier {
            "Z36" | "VOLLSTAENDIG" => Some(Self::Vollstaendig),
            "Z34" | "ERWARTET" => Some(Self::Erwartet),
            "Z35" | "IM_SYSTEM_VORHANDEN" => Some(Self::ImSystemVorhanden),
            "Z33" | "INFORMATIV" => Some(Self::Informativ),
            _ => None,
        }
    }

    /// Converts to the EDIFACT qualifier string.
    pub fn to_qualifier(&self) -> &'static str {
        match self {
            Self::Vollstaendig => "Z36",
            Self::Erwartet => "Z34",
            Self::ImSystemVorhanden => "Z35",
            Self::Informativ => "Z33",
        }
    }
}

impl std::fmt::Display for DataQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vollstaendig => write!(f, "VOLLSTAENDIG"),
            Self::Erwartet => write!(f, "ERWARTET"),
            Self::ImSystemVorhanden => write!(f, "IM_SYSTEM_VORHANDEN"),
            Self::Informativ => write!(f, "INFORMATIV"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_quality_from_qualifier() {
        assert_eq!(
            DataQuality::from_qualifier("Z36"),
            Some(DataQuality::Vollstaendig)
        );
        assert_eq!(
            DataQuality::from_qualifier("Z33"),
            Some(DataQuality::Informativ)
        );
        assert_eq!(DataQuality::from_qualifier("XXX"), None);
    }

    #[test]
    fn test_data_quality_roundtrip() {
        for dq in [
            DataQuality::Vollstaendig,
            DataQuality::Erwartet,
            DataQuality::ImSystemVorhanden,
            DataQuality::Informativ,
        ] {
            let q = dq.to_qualifier();
            assert_eq!(DataQuality::from_qualifier(q), Some(dq));
        }
    }

    #[test]
    fn test_data_quality_serde() {
        let dq = DataQuality::Vollstaendig;
        let json = serde_json::to_string(&dq).unwrap();
        let deserialized: DataQuality = serde_json::from_str(&json).unwrap();
        assert_eq!(dq, deserialized);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_zeitraum`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

Update `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.

pub mod bo4e_types;
pub mod data_quality;
pub mod zeitraum;

pub use bo4e_types::*;
pub use data_quality::DataQuality;
pub use zeitraum::Zeitraum;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/
git commit -m "$(cat <<'EOF'
feat(bo4e-extensions): add Zeitraum and DataQuality types

Zeitraum wraps optional start/end NaiveDateTime.
DataQuality enum maps to EDIFACT qualifiers Z33-Z36.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: WithValidity Generic Wrapper

**Files:**
- Create: `crates/bo4e-extensions/src/with_validity.rs`
- Modify: `crates/bo4e-extensions/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/bo4e-extensions/src/with_validity.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::zeitraum::Zeitraum;

/// Wraps a BO4E business object with time validity and EDIFACT-specific context.
///
/// - `T` — the standard BO4E business object (pure data)
/// - `E` — the EDIFACT companion type (functional domain data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithValidity<T, E> {
    /// The pure BO4E business object.
    pub data: T,
    /// EDIFACT-specific functional domain data.
    pub edifact: E,
    /// Optional validity period.
    pub gueltigkeitszeitraum: Option<Zeitraum>,
    /// Reference to the original Zeitscheibe for roundtrip support.
    pub zeitscheibe_ref: Option<String>,
}

impl<T: Default, E: Default> Default for WithValidity<T, E> {
    fn default() -> Self {
        Self {
            data: T::default(),
            edifact: E::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        }
    }
}

impl<T, E: Default> WithValidity<T, E> {
    /// Creates a new WithValidity wrapping the given data with default EDIFACT context.
    pub fn new(data: T) -> Self {
        Self {
            data,
            edifact: E::default(),
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: None,
        }
    }

    /// Sets the validity period.
    pub fn with_zeitraum(mut self, zeitraum: Zeitraum) -> Self {
        self.gueltigkeitszeitraum = Some(zeitraum);
        self
    }

    /// Sets the Zeitscheibe reference.
    pub fn with_zeitscheibe_ref(mut self, zs_ref: String) -> Self {
        self.zeitscheibe_ref = Some(zs_ref);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bo4e_types::Marktlokation;
    use crate::data_quality::DataQuality;

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct TestEdifact {
        pub datenqualitaet: Option<DataQuality>,
        pub custom_field: Option<String>,
    }

    #[test]
    fn test_with_validity_new() {
        let ml = Marktlokation {
            marktlokations_id: Some("DE001".to_string()),
            ..Default::default()
        };
        let wv: WithValidity<Marktlokation, TestEdifact> = WithValidity::new(ml);

        assert_eq!(wv.data.marktlokations_id, Some("DE001".to_string()));
        assert!(wv.edifact.datenqualitaet.is_none());
        assert!(wv.gueltigkeitszeitraum.is_none());
        assert!(wv.zeitscheibe_ref.is_none());
    }

    #[test]
    fn test_with_validity_builder_pattern() {
        let wv: WithValidity<Marktlokation, TestEdifact> =
            WithValidity::new(Marktlokation::default())
                .with_zeitraum(Zeitraum::default())
                .with_zeitscheibe_ref("ZS001".to_string());

        assert!(wv.gueltigkeitszeitraum.is_some());
        assert_eq!(wv.zeitscheibe_ref, Some("ZS001".to_string()));
    }

    #[test]
    fn test_with_validity_serde_roundtrip() {
        let wv: WithValidity<Marktlokation, TestEdifact> = WithValidity {
            data: Marktlokation {
                marktlokations_id: Some("DE001".to_string()),
                ..Default::default()
            },
            edifact: TestEdifact {
                datenqualitaet: Some(DataQuality::Vollstaendig),
                custom_field: Some("test".to_string()),
            },
            gueltigkeitszeitraum: None,
            zeitscheibe_ref: Some("1".to_string()),
        };

        let json = serde_json::to_string_pretty(&wv).unwrap();
        let deserialized: WithValidity<Marktlokation, TestEdifact> =
            serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.data.marktlokations_id,
            Some("DE001".to_string())
        );
        assert_eq!(
            deserialized.edifact.datenqualitaet,
            Some(DataQuality::Vollstaendig)
        );
        assert_eq!(deserialized.zeitscheibe_ref, Some("1".to_string()));
    }

    #[test]
    fn test_with_validity_default() {
        let wv: WithValidity<Marktlokation, TestEdifact> = WithValidity::default();
        assert!(wv.data.marktlokations_id.is_none());
        assert!(wv.edifact.custom_field.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_with_validity`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

Update `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.

pub mod bo4e_types;
pub mod data_quality;
pub mod with_validity;
pub mod zeitraum;

pub use bo4e_types::*;
pub use data_quality::DataQuality;
pub use with_validity::WithValidity;
pub use zeitraum::Zeitraum;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/
git commit -m "$(cat <<'EOF'
feat(bo4e-extensions): add WithValidity<T, E> generic wrapper

Wraps a BO4E business object with EDIFACT companion data,
time validity (Zeitraum), and Zeitscheibe reference for roundtrip.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: EDIFACT Companion Types

**Files:**
- Create: `crates/bo4e-extensions/src/edifact_types.rs`
- Modify: `crates/bo4e-extensions/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/bo4e-extensions/src/edifact_types.rs`:

```rust
//! EDIFACT companion types that store functional domain data
//! not present in standard BO4E.

use serde::{Deserialize, Serialize};

use crate::data_quality::DataQuality;

/// EDIFACT companion for Marktlokation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarktlokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_netzlokation: Option<String>,
    pub vorgelagerte_lokations_ids: Option<Vec<LokationsTypZuordnung>>,
}

/// EDIFACT companion for Messlokation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MesslokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_netzlokation: Option<String>,
    pub vorgelagerte_lokations_ids: Option<Vec<LokationsTypZuordnung>>,
}

/// EDIFACT companion for Zaehler.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZaehlerEdifact {
    pub referenz_messlokation: Option<String>,
    pub referenz_gateway: Option<String>,
    pub produktpaket_id: Option<String>,
    pub is_smartmeter_gateway: Option<bool>,
    pub smartmeter_gateway_zuordnung: Option<String>,
}

/// EDIFACT companion for Geschaeftspartner.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeschaeftspartnerEdifact {
    pub nad_qualifier: Option<String>,
}

/// EDIFACT companion for Vertrag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VertragEdifact {
    pub haushaltskunde: Option<bool>,
    pub versorgungsart: Option<String>,
}

/// EDIFACT companion for Netzlokation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetzlokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_marktlokation: Option<String>,
    pub zugeordnete_messlokationen: Option<Vec<LokationsTypZuordnung>>,
}

/// EDIFACT companion for TechnischeRessource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnischeRessourceEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_marktlokation: Option<String>,
    pub referenz_steuerbare_ressource: Option<String>,
    pub referenz_messlokation: Option<String>,
}

/// EDIFACT companion for SteuerbareRessource.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SteuerbareRessourceEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub produktpaket_id: Option<String>,
}

/// EDIFACT companion for Tranche (placeholder).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrancheEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
}

/// EDIFACT companion for MabisZaehlpunkt (placeholder).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MabisZaehlpunktEdifact {
    pub zaehlpunkt_typ: Option<String>,
}

/// EDIFACT companion for Bilanzierung.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BilanzierungEdifact {
    pub temperatur_arbeit: Option<f64>,
    pub jahresverbrauchsprognose: Option<f64>,
}

/// EDIFACT companion for Produktpaket.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProduktpaketEdifact {
    pub produktpaket_name: Option<String>,
}

/// EDIFACT companion for Lokationszuordnung.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LokationszuordnungEdifact {
    pub zuordnungstyp: Option<String>,
}

/// A location type assignment (used in vorgelagerte_lokations_ids).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LokationsTypZuordnung {
    pub lokations_id: String,
    pub lokationstyp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marktlokation_edifact_default() {
        let e = MarktlokationEdifact::default();
        assert!(e.datenqualitaet.is_none());
        assert!(e.referenz_netzlokation.is_none());
    }

    #[test]
    fn test_zaehler_edifact_serde() {
        let e = ZaehlerEdifact {
            referenz_messlokation: Some("MELO001".to_string()),
            is_smartmeter_gateway: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_string(&e).unwrap();
        let deserialized: ZaehlerEdifact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.referenz_messlokation, Some("MELO001".to_string()));
        assert_eq!(deserialized.is_smartmeter_gateway, Some(true));
    }

    #[test]
    fn test_all_edifact_types_default() {
        // Verify all types implement Default
        let _ = MarktlokationEdifact::default();
        let _ = MesslokationEdifact::default();
        let _ = ZaehlerEdifact::default();
        let _ = GeschaeftspartnerEdifact::default();
        let _ = VertragEdifact::default();
        let _ = NetzlokationEdifact::default();
        let _ = TechnischeRessourceEdifact::default();
        let _ = SteuerbareRessourceEdifact::default();
        let _ = TrancheEdifact::default();
        let _ = MabisZaehlpunktEdifact::default();
        let _ = BilanzierungEdifact::default();
        let _ = ProduktpaketEdifact::default();
        let _ = LokationszuordnungEdifact::default();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_marktlokation_edifact`
Expected: FAIL — module not found.

**Step 3: Write minimal implementation**

Update `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.

pub mod bo4e_types;
pub mod data_quality;
pub mod edifact_types;
pub mod with_validity;
pub mod zeitraum;

pub use bo4e_types::*;
pub use data_quality::DataQuality;
pub use edifact_types::*;
pub use with_validity::WithValidity;
pub use zeitraum::Zeitraum;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/
git commit -m "$(cat <<'EOF'
feat(bo4e-extensions): add all EDIFACT companion types

MarktlokationEdifact, ZaehlerEdifact, GeschaeftspartnerEdifact,
VertragEdifact, NetzlokationEdifact, etc. All derive Default +
Serialize/Deserialize for JSON contract support.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Transaction Containers and Process Data

**Files:**
- Create: `crates/bo4e-extensions/src/transaction.rs`
- Create: `crates/bo4e-extensions/src/prozessdaten.rs`
- Modify: `crates/bo4e-extensions/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/bo4e-extensions/src/prozessdaten.rs`:

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Process-level metadata for UTILMD transactions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prozessdaten {
    pub transaktionsgrund: Option<String>,
    pub transaktionsgrund_ergaenzung: Option<String>,
    pub transaktionsgrund_ergaenzung_befristete_anmeldung: Option<String>,
    pub prozessdatum: Option<NaiveDateTime>,
    pub wirksamkeitsdatum: Option<NaiveDateTime>,
    pub vertragsbeginn: Option<NaiveDateTime>,
    pub vertragsende: Option<NaiveDateTime>,
    pub lieferbeginndatum_in_bearbeitung: Option<NaiveDateTime>,
    pub datum_naechste_bearbeitung: Option<NaiveDateTime>,
    pub tag_des_empfangs: Option<NaiveDateTime>,
    pub kuendigungsdatum_kunde: Option<NaiveDateTime>,
    pub geplanter_liefertermin: Option<NaiveDateTime>,
    pub verwendung_der_daten_ab: Option<NaiveDateTime>,
    pub verwendung_der_daten_bis: Option<NaiveDateTime>,
    pub referenz_vorgangsnummer: Option<String>,
    pub anfrage_referenz: Option<String>,
    pub geplantes_paket: Option<String>,
    pub bemerkung: Option<String>,
    pub andere_partei_mp_id: Option<String>,
}

/// Message-level metadata (from UNB/BGM/DTM segments).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nachrichtendaten {
    pub dokumentennummer: Option<String>,
    pub nachrichtenreferenz: Option<String>,
    pub absender_mp_id: Option<String>,
    pub empfaenger_mp_id: Option<String>,
    pub erstellungsdatum: Option<NaiveDateTime>,
    pub datenaustauschreferenz: Option<String>,
    pub pruefidentifikator: Option<String>,
    pub kategorie: Option<String>,
}

/// A time slice reference within a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zeitscheibe {
    pub zeitscheiben_id: String,
    pub gueltigkeitszeitraum: Option<crate::zeitraum::Zeitraum>,
}

/// Response status for answer messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Antwortstatus {
    pub status: Option<String>,
    pub grund: Option<String>,
    pub details: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prozessdaten_default() {
        let pd = Prozessdaten::default();
        assert!(pd.transaktionsgrund.is_none());
        assert!(pd.prozessdatum.is_none());
    }

    #[test]
    fn test_nachrichtendaten_serde() {
        let nd = Nachrichtendaten {
            dokumentennummer: Some("DOC001".to_string()),
            absender_mp_id: Some("9900123000002".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&nd).unwrap();
        let de: Nachrichtendaten = serde_json::from_str(&json).unwrap();
        assert_eq!(de.dokumentennummer, Some("DOC001".to_string()));
    }

    #[test]
    fn test_zeitscheibe_serde() {
        let zs = Zeitscheibe {
            zeitscheiben_id: "1".to_string(),
            gueltigkeitszeitraum: None,
        };
        let json = serde_json::to_string(&zs).unwrap();
        assert!(json.contains("\"zeitscheiben_id\":\"1\""));
    }
}
```

Create `crates/bo4e-extensions/src/transaction.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::bo4e_types::*;
use crate::edifact_types::*;
use crate::prozessdaten::*;
use crate::with_validity::WithValidity;

/// A complete UTILMD message containing one or more transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilmdNachricht {
    pub nachrichtendaten: Nachrichtendaten,
    pub dokumentennummer: String,
    pub kategorie: Option<String>,
    pub transaktionen: Vec<UtilmdTransaktion>,
}

/// A single UTILMD transaction (IDE segment group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilmdTransaktion {
    pub transaktions_id: String,
    pub referenz_transaktions_id: Option<String>,
    pub absender: Marktteilnehmer,
    pub empfaenger: Marktteilnehmer,
    pub prozessdaten: Prozessdaten,
    pub antwortstatus: Option<Antwortstatus>,
    pub zeitscheiben: Vec<Zeitscheibe>,
    pub marktlokationen: Vec<WithValidity<Marktlokation, MarktlokationEdifact>>,
    pub messlokationen: Vec<WithValidity<Messlokation, MesslokationEdifact>>,
    pub netzlokationen: Vec<WithValidity<Netzlokation, NetzlokationEdifact>>,
    pub steuerbare_ressourcen: Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>,
    pub technische_ressourcen: Vec<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>>,
    pub tranchen: Vec<WithValidity<Tranche, TrancheEdifact>>,
    pub mabis_zaehlpunkte: Vec<WithValidity<MabisZaehlpunkt, MabisZaehlpunktEdifact>>,
    pub parteien: Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>,
    pub vertrag: Option<WithValidity<Vertrag, VertragEdifact>>,
    pub bilanzierung: Option<WithValidity<Bilanzierung, BilanzierungEdifact>>,
    pub zaehler: Vec<WithValidity<Zaehler, ZaehlerEdifact>>,
    pub produktpakete: Vec<WithValidity<Produktpaket, ProduktpaketEdifact>>,
    pub lokationszuordnungen: Vec<WithValidity<Lokationszuordnung, LokationszuordnungEdifact>>,
}

impl Default for UtilmdTransaktion {
    fn default() -> Self {
        Self {
            transaktions_id: String::new(),
            referenz_transaktions_id: None,
            absender: Marktteilnehmer::default(),
            empfaenger: Marktteilnehmer::default(),
            prozessdaten: Prozessdaten::default(),
            antwortstatus: None,
            zeitscheiben: Vec::new(),
            marktlokationen: Vec::new(),
            messlokationen: Vec::new(),
            netzlokationen: Vec::new(),
            steuerbare_ressourcen: Vec::new(),
            technische_ressourcen: Vec::new(),
            tranchen: Vec::new(),
            mabis_zaehlpunkte: Vec::new(),
            parteien: Vec::new(),
            vertrag: None,
            bilanzierung: None,
            zaehler: Vec::new(),
            produktpakete: Vec::new(),
            lokationszuordnungen: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utilmd_transaktion_default() {
        let tx = UtilmdTransaktion::default();
        assert!(tx.transaktions_id.is_empty());
        assert!(tx.marktlokationen.is_empty());
        assert!(tx.vertrag.is_none());
    }

    #[test]
    fn test_utilmd_transaktion_serde_roundtrip() {
        let tx = UtilmdTransaktion {
            transaktions_id: "TX001".to_string(),
            absender: Marktteilnehmer {
                mp_id: Some("9900123".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&tx).unwrap();
        let de: UtilmdTransaktion = serde_json::from_str(&json).unwrap();
        assert_eq!(de.transaktions_id, "TX001");
        assert_eq!(de.absender.mp_id, Some("9900123".to_string()));
    }

    #[test]
    fn test_utilmd_nachricht_serde() {
        let msg = UtilmdNachricht {
            nachrichtendaten: Nachrichtendaten::default(),
            dokumentennummer: "DOC001".to_string(),
            kategorie: Some("E03".to_string()),
            transaktionen: vec![UtilmdTransaktion::default()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let de: UtilmdNachricht = serde_json::from_str(&json).unwrap();
        assert_eq!(de.dokumentennummer, "DOC001");
        assert_eq!(de.transaktionen.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_utilmd`
Expected: FAIL — modules not found.

**Step 3: Write minimal implementation**

Update `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.

pub mod bo4e_types;
pub mod data_quality;
pub mod edifact_types;
pub mod prozessdaten;
pub mod transaction;
pub mod with_validity;
pub mod zeitraum;

pub use bo4e_types::*;
pub use data_quality::DataQuality;
pub use edifact_types::*;
pub use prozessdaten::*;
pub use transaction::*;
pub use with_validity::WithValidity;
pub use zeitraum::Zeitraum;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/
git commit -m "$(cat <<'EOF'
feat(bo4e-extensions): add UtilmdTransaktion container and Prozessdaten

Complete transaction container with all entity collections using
WithValidity wrappers. Includes Prozessdaten, Nachrichtendaten,
Zeitscheibe, and Antwortstatus.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Bo4eUri and LinkRegistry

**Files:**
- Create: `crates/bo4e-extensions/src/uri.rs`
- Create: `crates/bo4e-extensions/src/link_registry.rs`
- Modify: `crates/bo4e-extensions/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/bo4e-extensions/src/uri.rs`:

```rust
use serde::{Deserialize, Serialize};

/// A URI identifying a BO4E business object.
///
/// Format: `bo4e://TypeName/Identifier`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bo4eUri(String);

impl Bo4eUri {
    /// Creates a new BO4E URI.
    pub fn new(type_name: &str, id: &str) -> Self {
        Self(format!("bo4e://{}/{}", type_name, id))
    }

    /// Extracts the type name from the URI.
    pub fn type_name(&self) -> &str {
        let after_scheme = &self.0["bo4e://".len()..];
        after_scheme.split('/').next().unwrap_or("")
    }

    /// Extracts the identifier from the URI.
    pub fn id(&self) -> &str {
        let after_scheme = &self.0["bo4e://".len()..];
        after_scheme.split('/').nth(1).unwrap_or("")
    }

    /// Attempts to parse a string as a Bo4eUri.
    pub fn parse(s: &str) -> Option<Self> {
        if !s.starts_with("bo4e://") {
            return None;
        }
        let path = &s["bo4e://".len()..];
        let slash = path.find('/')?;
        if slash == 0 || slash == path.len() - 1 {
            return None;
        }
        Some(Self(s.to_string()))
    }

    /// Returns the full URI string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Bo4eUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bo4e_uri_new() {
        let uri = Bo4eUri::new("Marktlokation", "DE001");
        assert_eq!(uri.to_string(), "bo4e://Marktlokation/DE001");
        assert_eq!(uri.type_name(), "Marktlokation");
        assert_eq!(uri.id(), "DE001");
    }

    #[test]
    fn test_bo4e_uri_parse() {
        let uri = Bo4eUri::parse("bo4e://Zaehler/Z001").unwrap();
        assert_eq!(uri.type_name(), "Zaehler");
        assert_eq!(uri.id(), "Z001");
    }

    #[test]
    fn test_bo4e_uri_parse_invalid() {
        assert!(Bo4eUri::parse("http://example.com").is_none());
        assert!(Bo4eUri::parse("bo4e://").is_none());
        assert!(Bo4eUri::parse("bo4e:///id").is_none());
        assert!(Bo4eUri::parse("bo4e://Type/").is_none());
    }

    #[test]
    fn test_bo4e_uri_equality() {
        let a = Bo4eUri::new("Marktlokation", "DE001");
        let b = Bo4eUri::new("Marktlokation", "DE001");
        let c = Bo4eUri::new("Marktlokation", "DE002");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_bo4e_uri_serde() {
        let uri = Bo4eUri::new("Geschaeftspartner", "GP001");
        let json = serde_json::to_string(&uri).unwrap();
        let de: Bo4eUri = serde_json::from_str(&json).unwrap();
        assert_eq!(uri, de);
    }
}
```

Create `crates/bo4e-extensions/src/link_registry.rs`:

```rust
use std::collections::HashMap;

use crate::uri::Bo4eUri;

/// Registry for managing links between BO4E objects within a transaction.
#[derive(Debug, Clone, Default)]
pub struct LinkRegistry {
    links: HashMap<Bo4eUri, Vec<Bo4eUri>>,
}

impl LinkRegistry {
    /// Creates a new empty link registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a link from source to target.
    pub fn add_link(&mut self, source: Bo4eUri, target: Bo4eUri) {
        self.links.entry(source).or_default().push(target);
    }

    /// Gets all links from a specific source.
    pub fn get_links_from(&self, source: &Bo4eUri) -> &[Bo4eUri] {
        self.links.get(source).map_or(&[], |v| v.as_slice())
    }

    /// Returns all links as a map.
    pub fn get_all_links(&self) -> &HashMap<Bo4eUri, Vec<Bo4eUri>> {
        &self.links
    }

    /// Clears all registered links.
    pub fn clear(&mut self) {
        self.links.clear();
    }

    /// Returns the number of source entries.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns true if no links are registered.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_registry_add_and_get() {
        let mut reg = LinkRegistry::new();
        let ml = Bo4eUri::new("Marktlokation", "ML001");
        let melo = Bo4eUri::new("Messlokation", "MELO001");
        let nelo = Bo4eUri::new("Netzlokation", "NELO001");

        reg.add_link(ml.clone(), melo.clone());
        reg.add_link(ml.clone(), nelo.clone());

        let links = reg.get_links_from(&ml);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], melo);
        assert_eq!(links[1], nelo);
    }

    #[test]
    fn test_link_registry_empty() {
        let reg = LinkRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);

        let ml = Bo4eUri::new("Marktlokation", "ML001");
        assert!(reg.get_links_from(&ml).is_empty());
    }

    #[test]
    fn test_link_registry_clear() {
        let mut reg = LinkRegistry::new();
        reg.add_link(
            Bo4eUri::new("A", "1"),
            Bo4eUri::new("B", "2"),
        );
        assert!(!reg.is_empty());

        reg.clear();
        assert!(reg.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_bo4e_uri`
Expected: FAIL — modules not found.

**Step 3: Write minimal implementation**

Update `crates/bo4e-extensions/src/lib.rs`:

```rust
//! BO4E extension types for EDIFACT mapping.

pub mod bo4e_types;
pub mod data_quality;
pub mod edifact_types;
pub mod link_registry;
pub mod prozessdaten;
pub mod transaction;
pub mod uri;
pub mod with_validity;
pub mod zeitraum;

pub use bo4e_types::*;
pub use data_quality::DataQuality;
pub use edifact_types::*;
pub use link_registry::LinkRegistry;
pub use prozessdaten::*;
pub use transaction::*;
pub use uri::Bo4eUri;
pub use with_validity::WithValidity;
pub use zeitraum::Zeitraum;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions`
Expected: PASS — all tests pass.

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/
git commit -m "$(cat <<'EOF'
feat(bo4e-extensions): add Bo4eUri and LinkRegistry

Bo4eUri provides bo4e://TypeName/Id format for object identification.
LinkRegistry manages source->target relationships between BO4E objects.
Completes the bo4e-extensions public API.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```
