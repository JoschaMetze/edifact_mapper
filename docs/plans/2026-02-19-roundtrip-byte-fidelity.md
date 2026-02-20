# Roundtrip Byte Fidelity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `test_bo4e_roundtrip_fixture_byte_identical` pass for all 616 UTILMD fixture files — EDIFACT→BO4E→EDIFACT produces byte-identical output after normalization.

**Architecture:** Three-phase approach: (1) fix envelope/service segment roundtrip, (2) fix transaction-level segment parsing and writing gaps, (3) add passthrough storage for unmodeled segments so they survive roundtrip verbatim.

**Tech Stack:** Rust, automapper-core crate, bo4e-extensions crate, edifact-types crate

---

## Phase 1: Envelope Fidelity

### Task 1: Add missing fields to `Nachrichtendaten`

**Files:**
- Modify: `crates/bo4e-extensions/src/prozessdaten.rs:28-43`

**Step 1: Write failing test**

Add to the existing `#[cfg(test)]` module in `prozessdaten.rs`:

```rust
#[test]
fn test_nachrichtendaten_has_envelope_fields() {
    let nd = Nachrichtendaten {
        absender_unb_qualifier: Some("500".to_string()),
        empfaenger_unb_qualifier: Some("500".to_string()),
        unb_datum: Some("250331".to_string()),
        unb_zeit: Some("1329".to_string()),
        explicit_una: true,
        nachrichtentyp: Some("UTILMD:D:11A:UN:S2.1".to_string()),
        ..Default::default()
    };
    assert_eq!(nd.absender_unb_qualifier, Some("500".to_string()));
    assert_eq!(nd.unb_datum, Some("250331".to_string()));
    assert!(nd.explicit_una);
    assert_eq!(nd.nachrichtentyp.as_deref(), Some("UTILMD:D:11A:UN:S2.1"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p bo4e-extensions test_nachrichtendaten_has_envelope_fields`
Expected: FAIL — fields don't exist

**Step 3: Add fields to Nachrichtendaten**

Add these fields to `Nachrichtendaten` struct (after existing fields):

```rust
/// UNB sender identification code qualifier (e.g. "500"), element 1 component 1.
pub absender_unb_qualifier: Option<String>,
/// UNB recipient identification code qualifier (e.g. "500"), element 2 component 1.
pub empfaenger_unb_qualifier: Option<String>,
/// UNB preparation date (YYMMDD), element 3 component 0.
pub unb_datum: Option<String>,
/// UNB preparation time (HHMM), element 3 component 1.
pub unb_zeit: Option<String>,
/// Whether the original message had an explicit UNA service string.
#[serde(default)]
pub explicit_una: bool,
/// Original message type identifier from UNH (e.g. "UTILMD:D:11A:UN:S2.1").
pub nachrichtentyp: Option<String>,
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p bo4e-extensions test_nachrichtendaten_has_envelope_fields`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bo4e-extensions/src/prozessdaten.rs
git commit -m "feat(bo4e-extensions): add envelope roundtrip fields to Nachrichtendaten"
```

---

### Task 2: Parse UNB fields and UNA/UNH metadata in coordinator

**Files:**
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs` (handlers: `on_delimiters`, `on_interchange_start`, `on_message_start`)

**Step 1: Write failing test**

Add to `crates/automapper-core/tests/roundtrip_bo4e_test.rs` (synthetic tests section):

```rust
#[test]
fn test_envelope_fields_roundtrip() {
    // UNB with :500 qualifiers, explicit UNA, date/time
    let edifact = b"UNA:+.? '\
UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
DTM+137:202507011330:303'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TX001'\
UNT+8+MSG001'\
UNZ+1+REF001'";

    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(edifact).unwrap();

    let nd = &nachricht.nachrichtendaten;
    assert_eq!(nd.absender_unb_qualifier.as_deref(), Some("500"));
    assert_eq!(nd.empfaenger_unb_qualifier.as_deref(), Some("500"));
    assert_eq!(nd.unb_datum.as_deref(), Some("251217"));
    assert_eq!(nd.unb_zeit.as_deref(), Some("1229"));
    assert!(nd.explicit_una);
    assert_eq!(nd.nachrichtentyp.as_deref(), Some("UTILMD:D:11A:UN:S2.1"));
    assert_eq!(nd.erstellungsdatum.unwrap().format("%Y%m%d%H%M").to_string(), "202507011330");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test test_envelope_fields_roundtrip`
Expected: FAIL — fields are None

**Step 3: Update coordinator handlers**

In `utilmd_coordinator.rs`, update `on_delimiters`:

```rust
fn on_delimiters(&mut self, _delimiters: &EdifactDelimiters, explicit_una: bool) {
    self.nachrichtendaten.explicit_una = explicit_una;
}
```

Update `on_interchange_start` to capture qualifiers and date/time:

```rust
fn on_interchange_start(&mut self, unb: &RawSegment) -> Control {
    let sender = unb.get_component(1, 0);
    if !sender.is_empty() {
        self.nachrichtendaten.absender_mp_id = Some(sender.to_string());
    }
    let sender_qual = unb.get_component(1, 1);
    if !sender_qual.is_empty() {
        self.nachrichtendaten.absender_unb_qualifier = Some(sender_qual.to_string());
    }
    let recipient = unb.get_component(2, 0);
    if !recipient.is_empty() {
        self.nachrichtendaten.empfaenger_mp_id = Some(recipient.to_string());
    }
    let recipient_qual = unb.get_component(2, 1);
    if !recipient_qual.is_empty() {
        self.nachrichtendaten.empfaenger_unb_qualifier = Some(recipient_qual.to_string());
    }
    let date = unb.get_component(3, 0);
    if !date.is_empty() {
        self.nachrichtendaten.unb_datum = Some(date.to_string());
    }
    let time = unb.get_component(3, 1);
    if !time.is_empty() {
        self.nachrichtendaten.unb_zeit = Some(time.to_string());
    }
    let ref_nr = unb.get_element(4);
    if !ref_nr.is_empty() {
        self.nachrichtendaten.datenaustauschreferenz = Some(ref_nr.to_string());
    }
    Control::Continue
}
```

Update `on_message_start` to capture message type:

```rust
fn on_message_start(&mut self, unh: &RawSegment) -> Control {
    let msg_ref = unh.get_element(0);
    if !msg_ref.is_empty() {
        self.nachrichtendaten.nachrichtenreferenz = Some(msg_ref.to_string());
        self.context.set_message_reference(msg_ref);
    }
    // Capture original message type for roundtrip fidelity
    let msg_type = unh.get_element(1);
    if !msg_type.is_empty() {
        self.nachrichtendaten.nachrichtentyp = Some(msg_type.to_string());
    }
    Control::Continue
}
```

Add DTM+137 handling in `on_segment` — intercept message-level DTM+137 before routing to mappers:

```rust
fn on_segment(&mut self, segment: &RawSegment) -> Control {
    match segment.id {
        "BGM" => self.handle_bgm(segment),
        "IDE" => self.handle_ide(segment),
        "DTM" if !self.in_transaction => {
            // Message-level DTM (before any IDE) — capture into nachrichtendaten
            let qualifier = segment.get_component(0, 0);
            if qualifier == "137" {
                let value = segment.get_component(0, 1);
                let format_code = segment.get_component(0, 2);
                if let Some(dt) = ProzessdatenMapper::parse_dtm_value_pub(value, format_code) {
                    self.nachrichtendaten.erstellungsdatum = Some(dt);
                }
            }
            // Still route to mappers for any other message-level DTM
            self.route_to_mappers(segment);
        }
        "NAD" => {
            let q = segment.get_element(0);
            if q == "MS" || q == "MR" {
                self.handle_message_level_nad(segment);
            }
            self.route_to_mappers(segment);
        }
        _ => {
            self.route_to_mappers(segment);
        }
    }
    Control::Continue
}
```

Note: `parse_dtm_value` is currently private on ProzessdatenMapper. Either make it `pub(crate)` or extract it to a shared utility. The simplest approach is to add a `pub(crate)` wrapper:

In `crates/automapper-core/src/mappers/prozessdaten.rs`, add:

```rust
impl ProzessdatenMapper {
    /// Parse a DTM value+format pair into NaiveDateTime.
    /// Public for use by the coordinator (message-level DTM+137).
    pub(crate) fn parse_dtm_value_pub(value: &str, format_code: &str) -> Option<NaiveDateTime> {
        Self::parse_dtm_value(value, format_code)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test test_envelope_fields_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/src/utilmd_coordinator.rs crates/automapper-core/src/mappers/prozessdaten.rs
git commit -m "feat(automapper-core): parse UNB qualifiers, date/time, UNA flag, UNH type, DTM+137"
```

---

### Task 3: Update writer to use new envelope fields

**Files:**
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs` (`generate_impl`)
- Modify: `crates/automapper-core/src/writer/document_writer.rs` (`begin_interchange`)

**Step 1: Write failing test**

Add to `crates/automapper-core/tests/roundtrip_bo4e_test.rs`:

```rust
#[test]
fn test_envelope_writer_preserves_qualifiers() {
    let edifact = b"UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'\
UNH+MSG001+UTILMD:D:11A:UN:S2.1'\
BGM+E03+DOC001'\
NAD+MS+9900123000002::293'\
NAD+MR+9900456000001::293'\
IDE+24+TX001'\
UNT+6+MSG001'\
UNZ+1+REF001'";

    let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
    let nachricht = coord.parse_nachricht(edifact).unwrap();
    let output = String::from_utf8(coord.generate(&nachricht).unwrap()).unwrap();

    // No UNA (original had none)
    assert!(output.starts_with("UNB+"), "should NOT start with UNA");
    // UNB should preserve :500 qualifiers and date/time
    assert!(output.contains("UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'"));
    // UNH should preserve original message type
    assert!(output.contains("UNH+MSG001+UTILMD:D:11A:UN:S2.1'"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test test_envelope_writer_preserves_qualifiers`
Expected: FAIL — UNA is always written, qualifiers missing

**Step 3: Update `begin_interchange` to accept composites**

In `document_writer.rs`, change `begin_interchange` signature to accept sender/recipient as composites:

```rust
pub fn begin_interchange(
    &mut self,
    sender_id: &str,
    sender_qualifier: Option<&str>,
    recipient_id: &str,
    recipient_qualifier: Option<&str>,
    reference: &str,
    date: &str,
    time: &str,
    write_una: bool,
) {
    if write_una {
        self.write_una();
    }

    self.interchange_ref = Some(reference.to_string());
    self.message_count = 0;
    self.in_interchange = true;

    self.writer.begin_segment("UNB");
    self.writer.begin_composite();
    self.writer.add_component("UNOC");
    self.writer.add_component("3");
    self.writer.end_composite();
    // Sender composite
    self.writer.begin_composite();
    self.writer.add_component(sender_id);
    if let Some(q) = sender_qualifier {
        self.writer.add_component(q);
    }
    self.writer.end_composite();
    // Recipient composite
    self.writer.begin_composite();
    self.writer.add_component(recipient_id);
    if let Some(q) = recipient_qualifier {
        self.writer.add_component(q);
    }
    self.writer.end_composite();
    // Date/time composite
    self.writer.begin_composite();
    self.writer.add_component(date);
    self.writer.add_component(time);
    self.writer.end_composite();
    self.writer.add_element(reference);
    self.writer.end_segment();
}
```

Remove the `write_una` field from the struct — UNA writing is now controlled by the caller via the parameter. Update `new()` and `with_delimiters()` accordingly.

**Step 4: Update `generate_impl` to use new fields**

In `utilmd_coordinator.rs`, update `generate_impl`:

```rust
fn generate_impl(
    nachricht: &UtilmdNachricht,
    format_version: FormatVersion,
) -> Result<Vec<u8>, AutomapperError> {
    let nd = &nachricht.nachrichtendaten;

    // Use UNB date/time from parsed data, fallback to erstellungsdatum
    let (date_str, time_str) = if nd.unb_datum.is_some() || nd.unb_zeit.is_some() {
        (
            nd.unb_datum.clone().unwrap_or_default(),
            nd.unb_zeit.clone().unwrap_or_default(),
        )
    } else if let Some(ref dt) = nd.erstellungsdatum {
        (dt.format("%y%m%d").to_string(), dt.format("%H%M").to_string())
    } else {
        (String::new(), String::new())
    };

    let mut doc = EdifactDocumentWriter::new();

    doc.begin_interchange(
        nd.absender_mp_id.as_deref().unwrap_or(""),
        nd.absender_unb_qualifier.as_deref(),
        nd.empfaenger_mp_id.as_deref().unwrap_or(""),
        nd.empfaenger_unb_qualifier.as_deref(),
        nd.datenaustauschreferenz.as_deref().unwrap_or(""),
        &date_str,
        &time_str,
        nd.explicit_una,
    );

    // Use original message type string if available, else derive from format version
    let msg_type = nd.nachrichtentyp.as_deref()
        .unwrap_or(format_version.message_type_string());

    doc.begin_message(
        nd.nachrichtenreferenz.as_deref().unwrap_or(""),
        msg_type,
    );

    // ... rest unchanged
```

**Step 5: Fix all existing callers of `begin_interchange`**

Update tests in `document_writer.rs` that call `begin_interchange` to pass the new parameters.

**Step 6: Run tests**

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test test_envelope_writer_preserves_qualifiers`
Expected: PASS

Run: `cargo test --workspace`
Expected: all pass (existing tests updated)

**Step 7: Commit**

```bash
git add crates/automapper-core/src/writer/document_writer.rs crates/automapper-core/src/utilmd_coordinator.rs crates/automapper-core/tests/roundtrip_bo4e_test.rs
git commit -m "feat(automapper-core): writer preserves UNB qualifiers, date/time, UNA, UNH type"
```

---

## Phase 2: Transaction-Level Segment Gaps

### Task 4: Fix STS parsing — transaktionsgrund is in element 2 for S2.1 format

The real S2.1 STS format is `STS+7++E01+ZW4+E03` where:
- Element 0: `7` (status type)
- Element 1: empty
- Element 2: transaktionsgrund (e.g. `E01`)
- Element 3: ergaenzung (e.g. `ZW4`)
- Element 4: `transaktionsgrund_ergaenzung_befristete_anmeldung` (e.g. `E03`)

Current code reads element 1 for transaktionsgrund → gets empty string.

**Files:**
- Modify: `crates/automapper-core/src/mappers/prozessdaten.rs` (`handle_sts`)
- Modify: `crates/automapper-core/src/writer/entity_writers.rs` (`ProzessdatenWriter::write`)

**Step 1: Write failing test**

Add to the test module in `prozessdaten.rs`:

```rust
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

    let pd = mapper.build();
    assert_eq!(pd.transaktionsgrund, Some("E01".to_string()));
    assert_eq!(pd.transaktionsgrund_ergaenzung, Some("ZW4".to_string()));
    assert_eq!(pd.transaktionsgrund_ergaenzung_befristete_anmeldung, Some("E03".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_prozessdaten_mapper_sts_s21_format`
Expected: FAIL — transaktionsgrund is None

**Step 3: Fix `handle_sts`**

Update `handle_sts` in `prozessdaten.rs`:

```rust
fn handle_sts(&mut self, segment: &RawSegment) {
    // S2.1 format: STS+7++grund+ergaenzung+befristete_anmeldung
    // Legacy format: STS+grund_code+composite_with_grund+ergaenzung
    //
    // Detect by checking element 0: if "7", use S2.1 parsing (elements 2,3,4).
    // Otherwise, use legacy parsing (elements 1,2).
    let status_type = segment.get_element(0);

    if status_type == "7" || status_type == "E01" {
        // S2.1 format: STS+7++grund+ergaenzung+befristete_anmeldung
        // or answer: STS+E01++status:details
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
            self.prozessdaten.transaktionsgrund_ergaenzung_befristete_anmeldung =
                Some(befristet.to_string());
        }
    } else {
        // Legacy format: STS+E01+grund::codelist+ergaenzung
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
```

**Step 4: Fix STS writer to match S2.1 format**

Update `ProzessdatenWriter::write` in `entity_writers.rs`:

```rust
// STS segment (Counter=0250, Nr 00035)
// S2.1 format: STS+7++transaktionsgrund+ergaenzung+befristete_anmeldung
if let Some(ref grund) = pd.transaktionsgrund {
    let w = doc.segment_writer();
    w.begin_segment("STS");
    w.add_element("7");
    w.add_empty_element(); // empty element 1 per S2.1 format
    w.add_element(grund);
    if let Some(ref erg) = pd.transaktionsgrund_ergaenzung {
        w.add_element(erg);
    }
    if let Some(ref befr) = pd.transaktionsgrund_ergaenzung_befristete_anmeldung {
        if pd.transaktionsgrund_ergaenzung.is_none() {
            w.add_empty_element(); // empty ergaenzung placeholder
        }
        w.add_element(befr);
    }
    w.end_segment();
    doc.message_segment_count_increment();
}
```

**Step 5: Update existing STS tests** — the old tests use the legacy format; keep them but add the S2.1 tests alongside.

**Step 6: Run tests**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/automapper-core/src/mappers/prozessdaten.rs crates/automapper-core/src/writer/entity_writers.rs
git commit -m "fix(automapper-core): STS parsing/writing for S2.1 format (element 2 not 1)"
```

---

### Task 5: Add missing DTM qualifiers to ProzessdatenWriter

DTM+Z51 (tag_des_empfangs), DTM+Z52 (kuendigungsdatum_kunde), DTM+Z53 (geplanter_liefertermin) are parsed but not written.

**Files:**
- Modify: `crates/automapper-core/src/writer/entity_writers.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_prozessdaten_writer_all_dtm_qualifiers() {
    let pd = Prozessdaten {
        tag_des_empfangs: Some(dt),
        kuendigungsdatum_kunde: Some(dt),
        geplanter_liefertermin: Some(dt),
        ..Default::default()
    };
    ProzessdatenWriter::write(&mut doc, &pd);
    let output = doc.output();
    assert!(output.contains("DTM+Z51:"));
    assert!(output.contains("DTM+Z52:"));
    assert!(output.contains("DTM+Z53:"));
}
```

**Step 2: Add the missing DTM writes**

After the existing DTM writes in `ProzessdatenWriter::write`, add:

```rust
if let Some(ref dt) = pd.tag_des_empfangs {
    Self::write_dtm(doc, "Z51", dt);
}
if let Some(ref dt) = pd.kuendigungsdatum_kunde {
    Self::write_dtm(doc, "Z52", dt);
}
if let Some(ref dt) = pd.geplanter_liefertermin {
    Self::write_dtm(doc, "Z53", dt);
}
```

Also add DTM qualifiers `verwendung_der_daten_ab` and `verwendung_der_daten_bis` if they exist in the struct.

**Step 3: Run tests**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/automapper-core/src/writer/entity_writers.rs
git commit -m "feat(automapper-core): write DTM+Z51/Z52/Z53 in ProzessdatenWriter"
```

---

### Task 6: Preserve DTM format code and timezone for roundtrip

Original DTM values have format codes `303` or `102`, and some have timezone suffixes like `?+00`. The writer always uses `303` without timezone. For byte-identical roundtrip, we need to preserve the original format.

**Files:**
- Modify: `crates/bo4e-extensions/src/prozessdaten.rs` — add raw DTM metadata
- Modify: `crates/automapper-core/src/mappers/prozessdaten.rs` — capture raw DTM strings
- Modify: `crates/automapper-core/src/writer/entity_writers.rs` — write raw DTM when available

**Step 1: Add raw DTM storage to Prozessdaten**

```rust
/// Raw DTM values for roundtrip fidelity. Maps qualifier → "value:format" string.
/// E.g. "137" → "202503311329?+00:303"
#[serde(default, skip_serializing_if = "HashMap::is_empty")]
pub raw_dtm: HashMap<String, String>,
```

**Step 2: Capture raw DTM in mapper**

When `handle_dtm` is called, also store the raw composite string:

```rust
fn handle_dtm(&mut self, segment: &RawSegment) {
    let qualifier = segment.get_component(0, 0);
    let value = segment.get_component(0, 1);
    let format_code = segment.get_component(0, 2);
    // Store raw for roundtrip
    if !value.is_empty() {
        let raw = if format_code.is_empty() {
            value.to_string()
        } else {
            format!("{}:{}", value, format_code)
        };
        self.prozessdaten.raw_dtm.insert(qualifier.to_string(), raw);
    }
    // ... existing parsing continues
```

**Step 3: Use raw DTM in writer when available**

```rust
fn write_dtm_raw_or_parsed(doc: &mut EdifactDocumentWriter, qualifier: &str, dt: &NaiveDateTime, raw_dtm: &HashMap<String, String>) {
    if let Some(raw) = raw_dtm.get(qualifier) {
        // Write raw preserved value
        doc.write_segment_with_composites("DTM", &[&[qualifier, /* split raw */]]);
    } else {
        Self::write_dtm(doc, qualifier, dt);
    }
}
```

**Step 4: Run tests, commit**

---

### Task 7: Preserve DTM+137 raw value at message level for Nachrichtendaten

Similar to Task 6 but for the message-level DTM+137 captured by the coordinator.

**Files:**
- Modify: `crates/bo4e-extensions/src/prozessdaten.rs` — add `raw_nachrichtendatum: Option<String>` to Nachrichtendaten
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs` — capture raw value
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs` — use raw value in generate_impl

---

## Phase 3: Segment Passthrough

### Task 8: Add passthrough segment storage to UtilmdTransaktion

For unmodeled segments (46+ SEQ qualifiers, IMD, standalone CCI/CAV, etc.), store the raw segment text so it can be replayed during generation.

**Files:**
- Modify: `crates/bo4e-extensions/src/nachricht.rs` (UtilmdTransaktion)
- Modify: `crates/bo4e-extensions/src/prozessdaten.rs` (Nachrichtendaten — message-level passthrough)

**Step 1: Design passthrough storage**

Each passthrough segment needs a zone marker indicating where it should be replayed during generation:

```rust
/// Zone within an EDIFACT message where a segment appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentZone {
    /// Message header (before first IDE): DTM, IMD, etc.
    MessageHeader,
    /// Transaction header (after IDE, before LOC): DTM, STS, FTX area
    TransactionHeader,
    /// SG5: LOC area
    Locations,
    /// SG6: RFF area
    References,
    /// SG8: SEQ groups area
    Sequences,
    /// SG12: NAD parties area
    Parties,
}

/// A raw segment preserved for roundtrip fidelity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassthroughSegment {
    /// The full raw segment text (without terminator), e.g. "CCI+Z30++Z07"
    pub raw: String,
    pub zone: SegmentZone,
}
```

Add to `UtilmdTransaktion`:

```rust
/// Segments not handled by any mapper, preserved for roundtrip.
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub passthrough_segments: Vec<PassthroughSegment>,
```

Add to `Nachrichtendaten`:

```rust
/// Message-level passthrough segments (before first IDE).
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub passthrough_segments: Vec<PassthroughSegment>,
```

**Step 2: Commit**

---

### Task 9: Capture unhandled segments in coordinator

**Files:**
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs`

**Step 1: Track current zone**

Add a `current_zone: SegmentZone` field to the coordinator that advances as we encounter zone-transition segments (IDE → TransactionHeader, LOC → Locations, RFF after LOC → References, SEQ → Sequences, NAD after SEQ → Parties).

**Step 2: Capture passthrough in `route_to_mappers`**

After routing a segment to mappers, check if any mapper handled it. If not, store it as passthrough:

```rust
fn route_to_mappers(&mut self, segment: &RawSegment) {
    // Update zone based on segment type
    self.update_zone(segment);

    let handled = /* check if any mapper handles it */;

    if !handled {
        let raw = segment.to_raw_string(&self.delimiters);
        let ps = PassthroughSegment {
            raw,
            zone: self.current_zone,
        };
        if self.in_transaction {
            self.passthrough_segments.push(ps);
        } else {
            self.nachrichtendaten.passthrough_segments.push(ps);
        }
    }
}
```

**Note:** This requires a way to reconstruct the raw segment string from a `RawSegment`. Add a `to_raw_string` method to `RawSegment` or capture the raw text during parsing.

**Step 3: Run tests, commit**

---

### Task 10: Replay passthrough segments during generation

**Files:**
- Modify: `crates/automapper-core/src/utilmd_coordinator.rs` (`generate_impl`, `write_transaction`)

**Step 1: Write passthrough replay helper**

```rust
fn write_passthrough(doc: &mut EdifactDocumentWriter, segments: &[PassthroughSegment], zone: SegmentZone) {
    for ps in segments.iter().filter(|s| s.zone == zone) {
        doc.write_raw_segment(&ps.raw);
    }
}
```

**Step 2: Interleave passthrough in `write_transaction`**

After each zone's modeled segments, replay passthrough for that zone:

```rust
fn write_transaction(doc: &mut EdifactDocumentWriter, tx: &UtilmdTransaktion) {
    doc.write_segment("IDE", &["24", &tx.transaktions_id]);

    ProzessdatenWriter::write(doc, &tx.prozessdaten);
    Self::write_passthrough(doc, &tx.passthrough_segments, SegmentZone::TransactionHeader);

    // SG5: LOC segments
    for nl in &tx.netzlokationen { NetzlokationWriter::write(doc, nl); }
    // ... other LOC writers ...
    Self::write_passthrough(doc, &tx.passthrough_segments, SegmentZone::Locations);

    // SG6: RFF references
    ProzessdatenWriter::write_references(doc, &tx.prozessdaten);
    ZeitscheibeWriter::write(doc, &tx.zeitscheiben);
    Self::write_passthrough(doc, &tx.passthrough_segments, SegmentZone::References);

    // SG8: SEQ groups
    for lz in &tx.lokationszuordnungen { LokationszuordnungWriter::write(doc, lz); }
    // ... other SEQ writers ...
    Self::write_passthrough(doc, &tx.passthrough_segments, SegmentZone::Sequences);

    // SG12: NAD parties
    for ml in &tx.marktlokationen { MarktlokationWriter::write_address(doc, ml); }
    for gp in &tx.parteien { GeschaeftspartnerWriter::write(doc, gp); }
    Self::write_passthrough(doc, &tx.passthrough_segments, SegmentZone::Parties);
}
```

**Step 3: Also replay message-level passthrough**

In `generate_impl`, after NAD+MS/MR and before the transaction loop:

```rust
// Message-level passthrough (IMD, etc.)
for ps in &nd.passthrough_segments {
    doc.write_raw_segment(&ps.raw);
}
```

**Step 4: Add `write_raw_segment` to document writer**

```rust
pub fn write_raw_segment(&mut self, raw: &str) {
    self.writer.buffer.push_str(raw);
    self.writer.buffer.push(self.delimiters.segment_terminator);
    self.message_segment_count += 1;
}
```

**Step 5: Run byte-identical test**

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test test_bo4e_roundtrip_fixture_byte_identical -- --nocapture`
Expected: significant improvement, many files now match

**Step 6: Commit**

---

### Task 11: Capture raw segment text during parsing

The passthrough mechanism needs to reconstruct raw segment text from `RawSegment`. Two approaches:

**Option A (simpler):** Store the raw segment string in `RawSegment` during parsing. Add a `raw: Option<String>` field. The parser already has the raw bytes — capture them.

**Option B (no parser change):** Reconstruct from `RawSegment` fields using delimiters. Add `fn to_raw_string(&self, delimiters: &EdifactDelimiters) -> String` to `RawSegment`.

Prefer Option B to avoid changing the parser's zero-copy design.

**Files:**
- Modify: `crates/edifact-types/src/segment.rs` — add `to_raw_string`

---

### Task 12: Run byte-identical test and fix remaining differences

After all phases, run the full byte-identical test and debug any remaining differences.

Run: `cargo test -p automapper-core --test roundtrip_bo4e_test test_bo4e_roundtrip_fixture_byte_identical -- --nocapture`

Expected categories of remaining differences:
- Entity-writer segments with partially preserved data (e.g. NAD with full name/address components)
- SEQ groups that ARE modeled but whose writers produce simplified output
- Edge cases with escape sequences or custom delimiters

These can be addressed incrementally by enriching companion types and writers.

---

## Implementation Order

1. **Task 1** → Nachrichtendaten fields (bo4e-extensions)
2. **Task 2** → Parse envelope fields (automapper-core coordinator)
3. **Task 3** → Writer uses envelope fields (automapper-core writer)
4. **Task 4** → Fix STS parsing/writing for S2.1 format
5. **Task 5** → Missing DTM qualifiers in writer
6. **Task 6** → Raw DTM preservation
7. **Task 7** → Message-level DTM+137 raw preservation
8. **Task 8** → Passthrough storage types
9. **Task 9** → Capture unhandled segments
10. **Task 10** → Replay passthrough in writer
11. **Task 11** → Raw segment text reconstruction
12. **Task 12** → Integration test and remaining fixes

## Testing Strategy

- Each task has unit tests for the specific change
- `test_bo4e_roundtrip_fixture_byte_identical` is the integration gate
- Run after each phase to measure progress: Phase 1 → envelope matches, Phase 2 → transaction header matches, Phase 3 → full match
- Existing 558 tests must continue to pass throughout
