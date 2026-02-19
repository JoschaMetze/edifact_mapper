---
feature: missing-entity-mappers
epic: 3
title: "Coordinator Integration & Roundtrip Tests"
depends_on: [missing-entity-mappers/E01, missing-entity-mappers/E02]
estimated_tasks: 3
crate: automapper-core
status: in_progress
---

# Epic 3: Coordinator Integration & Roundtrip Tests

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-core/`. All code must compile with `cargo check -p automapper-core`.

**Goal:** Register all 7 new mappers in `UtilmdCoordinator`, update `write_transaction()` segment ordering to include new entities per MIG Counter/Nr, and extend roundtrip tests to verify all entity types survive the parse → generate → reparse cycle.

**Architecture:** The coordinator changes are mechanical: add mapper fields, route segments, collect results, reset on transaction boundary. The writer ordering must follow MIG Nr ordering exactly. Roundtrip tests construct a `UtilmdNachricht` with all 15 entity types and verify segment ordering + data preservation.

**Tech Stack:** Rust, automapper-core, bo4e-extensions

---

## Task 1: Register 7 Mappers in UtilmdCoordinator

**Files:**
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs`

**Step 1: Add mapper fields to struct**

Add 7 new mapper fields to `UtilmdCoordinator<V>` struct definition (after `zaehler_mapper`):

```rust
    steuerbare_ressource_mapper: SteuerbareRessourceMapper,
    technische_ressource_mapper: TechnischeRessourceMapper,
    tranche_mapper: TrancheMapper,
    mabis_zaehlpunkt_mapper: MabisZaehlpunktMapper,
    bilanzierung_mapper: BilanzierungMapper,
    produktpaket_mapper: ProduktpaketMapper,
    lokationszuordnung_mapper: LokationszuordnungMapper,
```

**Step 2: Initialize in `new()`**

Add to `Self { ... }` in `new()`:

```rust
            steuerbare_ressource_mapper: SteuerbareRessourceMapper::new(),
            technische_ressource_mapper: TechnischeRessourceMapper::new(),
            tranche_mapper: TrancheMapper::new(),
            mabis_zaehlpunkt_mapper: MabisZaehlpunktMapper::new(),
            bilanzierung_mapper: BilanzierungMapper::new(),
            produktpaket_mapper: ProduktpaketMapper::new(),
            lokationszuordnung_mapper: LokationszuordnungMapper::new(),
```

**Step 3: Route in `route_to_mappers()`**

Add after the `zaehler_mapper` routing block:

```rust
        if self.steuerbare_ressource_mapper.can_handle(segment) {
            self.steuerbare_ressource_mapper
                .handle(segment, &mut self.context);
        }
        if self.technische_ressource_mapper.can_handle(segment) {
            self.technische_ressource_mapper
                .handle(segment, &mut self.context);
        }
        if self.tranche_mapper.can_handle(segment) {
            self.tranche_mapper.handle(segment, &mut self.context);
        }
        if self.mabis_zaehlpunkt_mapper.can_handle(segment) {
            self.mabis_zaehlpunkt_mapper
                .handle(segment, &mut self.context);
        }
        if self.bilanzierung_mapper.can_handle(segment) {
            self.bilanzierung_mapper
                .handle(segment, &mut self.context);
        }
        if self.produktpaket_mapper.can_handle(segment) {
            self.produktpaket_mapper
                .handle(segment, &mut self.context);
        }
        if self.lokationszuordnung_mapper.can_handle(segment) {
            self.lokationszuordnung_mapper
                .handle(segment, &mut self.context);
        }
```

**Step 4: Collect in `collect_transaction()`**

Replace the `Vec::new()` / `None` placeholders with actual builder calls:

```rust
        // Replace:
        //   steuerbare_ressourcen: Vec::new(),
        // With:
            steuerbare_ressourcen: self
                .steuerbare_ressource_mapper
                .build()
                .into_iter()
                .collect(),
            technische_ressourcen: self
                .technische_ressource_mapper
                .build()
                .into_iter()
                .collect(),
            tranchen: self.tranche_mapper.build().into_iter().collect(),
            mabis_zaehlpunkte: self
                .mabis_zaehlpunkt_mapper
                .build()
                .into_iter()
                .collect(),

        // Replace:
        //   bilanzierung: None,
        // With:
            bilanzierung: self.bilanzierung_mapper.build(),

        // Replace:
        //   produktpakete: Vec::new(),
        // With:
            produktpakete: self.produktpaket_mapper.build(),

        // Replace:
        //   lokationszuordnungen: Vec::new(),
        // With:
            lokationszuordnungen: self.lokationszuordnung_mapper.build(),
```

**Step 5: Reset in `reset_mappers()`**

Add after `self.zaehler_mapper.reset();`:

```rust
        self.steuerbare_ressource_mapper.reset();
        self.technische_ressource_mapper.reset();
        self.tranche_mapper.reset();
        self.mabis_zaehlpunkt_mapper.reset();
        self.bilanzierung_mapper.reset();
        self.produktpaket_mapper.reset();
        self.lokationszuordnung_mapper.reset();
```

**Step 6: Update writer imports**

Add to the `use crate::writer::{ ... }` import at the top of the file:

```rust
    SteuerbareRessourceWriter, TechnischeRessourceWriter, TrancheWriter,
    MabisZaehlpunktWriter, BilanzierungWriter, ProduktpaketWriter, LokationszuordnungWriter,
```

**Step 7: Run existing tests to verify nothing broke**

Run: `cargo test -p automapper-core utilmd_coordinator`
Expected: All existing coordinator tests PASS

**Step 8: Commit**

```bash
git add crates/automapper-core/src/utilmd_coordinator.rs
git commit -m "feat(automapper-core): register 7 new entity mappers in UtilmdCoordinator"
```

---

## Task 2: Update write_transaction() Segment Ordering

**Files:**
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs`

**Step 1: Update `write_transaction()` SG5 LOC ordering**

Replace the current SG5 LOC block with the full MIG Nr ordering:

```rust
        // SG5: LOC segments (Counter=0320)
        // MIG order: Z18 (Nr 48), Z16 (Nr 49), Z20 (Nr 51), Z19 (Nr 52),
        //            Z21 (Nr 53), Z17 (Nr 54), Z15 (Nr 55)
        for nl in &tx.netzlokationen {
            NetzlokationWriter::write(doc, nl);
        }
        for ml in &tx.marktlokationen {
            MarktlokationWriter::write(doc, ml);
        }
        for tr in &tx.technische_ressourcen {
            TechnischeRessourceWriter::write(doc, tr);
        }
        for sr in &tx.steuerbare_ressourcen {
            SteuerbareRessourceWriter::write(doc, sr);
        }
        for t in &tx.tranchen {
            TrancheWriter::write(doc, t);
        }
        for ml in &tx.messlokationen {
            MesslokationWriter::write(doc, ml);
        }
        for mz in &tx.mabis_zaehlpunkte {
            MabisZaehlpunktWriter::write(doc, mz);
        }
```

**Step 2: Update `write_transaction()` SG8 SEQ ordering**

Replace the current SG8 SEQ block with the full MIG Nr ordering:

```rust
        // SG8: SEQ groups (Counter=0410)
        // MIG order: Z78 (Nr 74), Z79 (Nr 81), Z03 (Nr 311), Z18 (Nr 291)
        for lz in &tx.lokationszuordnungen {
            LokationszuordnungWriter::write(doc, lz);
        }
        for pp in &tx.produktpakete {
            ProduktpaketWriter::write(doc, pp);
        }
        // Bilanzierung (SEQ+Z98, between Z79 and Z03 in Nr order)
        if let Some(ref b) = tx.bilanzierung {
            BilanzierungWriter::write(doc, b);
        }
        for z in &tx.zaehler {
            ZaehlerWriter::write(doc, z);
        }
        if let Some(ref v) = tx.vertrag {
            VertragWriter::write(doc, v);
        }
```

**Step 3: Update `write_transaction()` doc comment**

Update the doc comment to reflect the full ordering:

```rust
    /// Writes a single transaction (SG4) to the document writer.
    ///
    /// MIG segment ordering within SG4 (by Counter):
    /// - 0190: IDE+24
    /// - 0230: DTM (process dates)
    /// - 0250: STS (transaction reason)
    /// - 0280: FTX (remarks)
    /// - 0320: SG5/LOC (Z18, Z16, Z20, Z19, Z21, Z17, Z15)
    /// - 0350: SG6/RFF (Z13, Z47+DTM)
    /// - 0410: SG8/SEQ (Z78, Z79, Z98, Z03, Z18)
    /// - 0570: SG12/NAD (DP address, geschaeftspartner)
```

**Step 4: Run tests**

Run: `cargo test -p automapper-core`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/src/utilmd_coordinator.rs
git commit -m "feat(automapper-core): update write_transaction() with full MIG segment ordering"
```

---

## Task 3: Extend Roundtrip Tests

**Files:**
- Modify: `crates/automapper-core/tests/roundtrip_bo4e_test.rs`
- Modify: `docs/mig-segment-ordering.md`

**Step 1: Add new entity types to imports**

Add to the `use bo4e_extensions::{...}` import:

```rust
    Bilanzierung, BilanzierungEdifact,
    Lokationszuordnung, LokationszuordnungEdifact,
    MabisZaehlpunkt, MabisZaehlpunktEdifact,
    Produktpaket, ProduktpaketEdifact,
    SteuerbareRessource, SteuerbareRessourceEdifact,
    TechnischeRessource, TechnischeRessourceEdifact,
    Tranche, TrancheEdifact,
    Vertrag, VertragEdifact,
```

**Step 2: Add new LOC segments to `MINIMAL_EDIFACT`**

Update the constant to include new LOC qualifiers (in MIG Nr order):

```rust
const MINIMAL_EDIFACT: &[u8] = b"UNA:+.? '\
UNB+UNOC:3+9900123000002+9900456000001+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202507011330:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TXID001'\
LOC+Z18+NELO001'\
LOC+Z16+DE00014545768S0000000000000003054'\
LOC+Z20+TECRES001'\
LOC+Z19+STRES001'\
LOC+Z21+TRANCHE001'\
LOC+Z17+MELO001'\
LOC+Z15+MABIS001'\
STS+7+E01'\
UNT+16+MSG001'\
UNZ+1+REF001'";
```

**Step 3: Update `test_bo4e_roundtrip_synthetic_parse_generate()`**

After the existing location count assertions, add:

```rust
    assert_eq!(tx.steuerbare_ressourcen.len(), 1);
    assert_eq!(tx.technische_ressourcen.len(), 1);
    assert_eq!(tx.tranchen.len(), 1);
    assert_eq!(tx.mabis_zaehlpunkte.len(), 1);

    // ... in step 3 verification:
    assert!(output.contains("LOC+Z20+TECRES001'"), "LOC+Z20 technische_ressource");
    assert!(output.contains("LOC+Z19+STRES001'"), "LOC+Z19 steuerbare_ressource");
    assert!(output.contains("LOC+Z21+TRANCHE001'"), "LOC+Z21 tranche");
    assert!(output.contains("LOC+Z15+MABIS001'"), "LOC+Z15 mabis_zaehlpunkt");
```

**Step 4: Update `test_bo4e_roundtrip_synthetic_reparse()`**

Add entity count comparisons after the existing ones:

```rust
    assert_eq!(tx1.steuerbare_ressourcen.len(), tx2.steuerbare_ressourcen.len());
    assert_eq!(tx1.technische_ressourcen.len(), tx2.technische_ressourcen.len());
    assert_eq!(tx1.tranchen.len(), tx2.tranchen.len());
    assert_eq!(tx1.mabis_zaehlpunkte.len(), tx2.mabis_zaehlpunkte.len());
```

**Step 5: Add new entities to `test_bo4e_roundtrip_construct_and_generate()`**

Add all 7 new entity types to the constructed `UtilmdTransaktion`. Add them to the `..Default::default()` area:

```rust
            steuerbare_ressourcen: vec![WithValidity {
                data: SteuerbareRessource {
                    steuerbare_ressource_id: Some("STRES_GEN_001".to_string()),
                },
                edifact: SteuerbareRessourceEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            technische_ressourcen: vec![WithValidity {
                data: TechnischeRessource {
                    technische_ressource_id: Some("TECRES_GEN_001".to_string()),
                },
                edifact: TechnischeRessourceEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            tranchen: vec![WithValidity {
                data: Tranche {
                    tranche_id: Some("TRANCHE_GEN_001".to_string()),
                },
                edifact: TrancheEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            mabis_zaehlpunkte: vec![WithValidity {
                data: MabisZaehlpunkt {
                    zaehlpunkt_id: Some("MABIS_GEN_001".to_string()),
                },
                edifact: MabisZaehlpunktEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            bilanzierung: Some(WithValidity {
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
            }),
            produktpakete: vec![WithValidity {
                data: Produktpaket {
                    produktpaket_id: Some("PP_GEN_001".to_string()),
                },
                edifact: ProduktpaketEdifact {
                    produktpaket_name: Some("Grundversorgung".to_string()),
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            lokationszuordnungen: vec![WithValidity {
                data: Lokationszuordnung {
                    marktlokations_id: Some("MALO_LZ_001".to_string()),
                    messlokations_id: Some("MELO_LZ_001".to_string()),
                },
                edifact: LokationszuordnungEdifact::default(),
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }],
            vertrag: Some(WithValidity {
                data: Vertrag::default(),
                edifact: VertragEdifact {
                    haushaltskunde: Some(true),
                    versorgungsart: None,
                },
                gueltigkeitszeitraum: None,
                zeitscheibe_ref: None,
            }),
```

Add output assertions for new entities:

```rust
    // New LOC segments
    assert!(output.contains("LOC+Z20+TECRES_GEN_001'"));
    assert!(output.contains("LOC+Z19+STRES_GEN_001'"));
    assert!(output.contains("LOC+Z21+TRANCHE_GEN_001'"));
    assert!(output.contains("LOC+Z15+MABIS_GEN_001'"));

    // New SEQ segments
    assert!(output.contains("SEQ+Z78'"));
    assert!(output.contains("RFF+Z18:MALO_LZ_001'"));
    assert!(output.contains("RFF+Z19:MELO_LZ_001'"));
    assert!(output.contains("SEQ+Z79+PP_GEN_001'"));
    assert!(output.contains("PIA+5+Grundversorgung'"));
    assert!(output.contains("SEQ+Z98'"));
    assert!(output.contains("CCI+Z20++11YN20---------Z'"));
    assert!(output.contains("QTY+Z09:12345.67'"));
```

Add LOC ordering assertions (full MIG Nr order):

```rust
    // Full SG5 LOC MIG ordering: Z18 < Z16 < Z20 < Z19 < Z21 < Z17 < Z15
    let loc_z20_pos = output.find("LOC+Z20").unwrap();
    let loc_z19_pos = output.find("LOC+Z19").unwrap();
    let loc_z21_pos = output.find("LOC+Z21").unwrap();
    let loc_z15_pos = output.find("LOC+Z15").unwrap();

    assert!(loc_z18_pos < loc_z16_pos, "LOC+Z18 (Nr 48) before LOC+Z16 (Nr 49)");
    assert!(loc_z16_pos < loc_z20_pos, "LOC+Z16 (Nr 49) before LOC+Z20 (Nr 51)");
    assert!(loc_z20_pos < loc_z19_pos, "LOC+Z20 (Nr 51) before LOC+Z19 (Nr 52)");
    assert!(loc_z19_pos < loc_z21_pos, "LOC+Z19 (Nr 52) before LOC+Z21 (Nr 53)");
    assert!(loc_z21_pos < loc_z17_pos, "LOC+Z21 (Nr 53) before LOC+Z17 (Nr 54)");
    assert!(loc_z17_pos < loc_z15_pos, "LOC+Z17 (Nr 54) before LOC+Z15 (Nr 55)");
```

Add SEQ ordering assertions:

```rust
    // SG8 SEQ ordering: Z78 < Z79 < Z98 < Z03 < Z18
    let seq_z78_pos = output.find("SEQ+Z78").unwrap();
    let seq_z79_pos = output.find("SEQ+Z79").unwrap();
    let seq_z98_pos = output.find("SEQ+Z98").unwrap();

    assert!(seq_z78_pos < seq_z79_pos, "SEQ+Z78 (Nr 74) before SEQ+Z79 (Nr 81)");
    assert!(seq_z79_pos < seq_z98_pos, "SEQ+Z79 (Nr 81) before SEQ+Z98 (Bilanzierung)");
    assert!(seq_z98_pos < seq_z03_pos, "SEQ+Z98 before SEQ+Z03 (Nr 311)");
```

Add reparse count assertions for new entities:

```rust
    assert_eq!(tx2.steuerbare_ressourcen.len(), 1);
    assert_eq!(tx2.technische_ressourcen.len(), 1);
    assert_eq!(tx2.tranchen.len(), 1);
    assert_eq!(tx2.mabis_zaehlpunkte.len(), 1);
    assert!(tx2.bilanzierung.is_some());
    assert_eq!(tx2.produktpakete.len(), 1);
    assert_eq!(tx2.lokationszuordnungen.len(), 1);
```

**Step 6: Extend fixture reparse test**

In `test_bo4e_roundtrip_fixture_reparse()`, add entity count comparisons inside the per-transaction loop:

```rust
            // Compare all entity counts (not just marktlokationen)
            let entity_checks = [
                ("messlokationen", tx1.messlokationen.len(), tx2.messlokationen.len()),
                ("netzlokationen", tx1.netzlokationen.len(), tx2.netzlokationen.len()),
                ("steuerbare_ressourcen", tx1.steuerbare_ressourcen.len(), tx2.steuerbare_ressourcen.len()),
                ("technische_ressourcen", tx1.technische_ressourcen.len(), tx2.technische_ressourcen.len()),
                ("tranchen", tx1.tranchen.len(), tx2.tranchen.len()),
                ("mabis_zaehlpunkte", tx1.mabis_zaehlpunkte.len(), tx2.mabis_zaehlpunkte.len()),
                ("zaehler", tx1.zaehler.len(), tx2.zaehler.len()),
                ("produktpakete", tx1.produktpakete.len(), tx2.produktpakete.len()),
                ("lokationszuordnungen", tx1.lokationszuordnungen.len(), tx2.lokationszuordnungen.len()),
                ("parteien", tx1.parteien.len(), tx2.parteien.len()),
            ];
            for (name, c1, c2) in entity_checks {
                if c1 != c2 {
                    reparse_fail.push(format!(
                        "{} tx[{}]: {} count {} vs {}",
                        rel.display(), i, name, c1, c2
                    ));
                    file_ok = false;
                }
            }
```

**Step 7: Run all tests**

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test`
Expected: All roundtrip tests PASS

Run: `cargo test --workspace`
Expected: All workspace tests PASS

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings

Run: `cargo fmt --all -- --check`
Expected: No formatting issues

**Step 8: Update MIG ordering doc**

Update the "Simplified ordering for generate()" section in `docs/mig-segment-ordering.md`:

```
  --- SG5: Lokationen (Ctr=0320) ---
  LOC+Z18             (netzlokationen)                   Nr=00048
  LOC+Z16             (marktlokationen)                  Nr=00049
  LOC+Z20             (technische_ressourcen)             Nr=00051
  LOC+Z19             (steuerbare_ressourcen)             Nr=00052
  LOC+Z21             (tranchen)                          Nr=00053
  LOC+Z17             (messlokationen)                   Nr=00054
  LOC+Z15             (mabis_zaehlpunkte)                Nr=00055
  --- SG6: Referenzen (Ctr=0350) ---
  RFF+Z13             (referenz_vorgangsnummer)          Nr=00056
  RFF+Z47 + DTM+Z25/Z26  (zeitscheiben)                 Nr=00066
  --- SG8: Sequenzgruppen (Ctr=0410) ---
  SEQ+Z78             (lokationszuordnungen)             Nr=00074
  SEQ+Z79             (produktpakete)                    Nr=00081
  SEQ+Z98             (bilanzierung)                     Nr≈00200
  SEQ+Z01             (marktlokation data)               Nr=00114
  SEQ+Z18             (messlokation/vertrag data)        Nr=00291
  SEQ+Z03             (zaehler)                          Nr=00311
  --- SG12: Parteien (Ctr=0570) ---
  NAD+DP              (marktlokation address)            Nr=00518
  NAD+qualifier       (geschaeftspartner)                Nr=varies
```

**Step 9: Commit**

```bash
git add crates/automapper-core/tests/roundtrip_bo4e_test.rs docs/mig-segment-ordering.md
git commit -m "test(automapper-core): extend roundtrip tests for all 15 entity types"
```
