# UTILMD Gas (FV2504) Mapping Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Map all 95 UTILMD Gas PIDs (44001–44182) with TOML mappings and byte-identical roundtrip tests.

**Architecture:** Incremental approach — build 44001 as the reference PID with full TOML mappings, extract common/ patterns, then bulk-generate remaining 93 PIDs. Gas shares the same UTILMD MIG/schema directory as Strom but uses different AHB XML and different SG8/SG12 group variants.

**Tech Stack:** TOML mapping files, Rust test infrastructure (MessageTypeConfig), `generate-fixture` CLI for fixture generation, `schema-lookup` CLI for schema inspection.

---

## Scope Summary

| Category | Count | Description |
|---|---|---|
| Simple (only SG5/SG6) | 39 PIDs | LOC+172 + RFF/DTM only, no SG8/SG12 |
| Medium (< 10 groups) | 44 PIDs | Some SG8 and/or SG12 variants |
| Complex (10+ groups) | 12 PIDs | 44001 (11), 44002 (18), 44013 (22) biggest |

**Gas-specific characteristics:**
- 2 SG5 variants: `sg5_172` (95 PIDs), `sg5_z08` (2 PIDs)
- 14 SG8 variants: Z01–Z50 (completely different from Strom)
- 12 SG12/NAD variants: DDO, DP, EO, VY, Z03–Z09, Z25, Z26
- 6 SG6 RFF qualifiers: Z13 (all), TN (47), Z18 (11), Z19 (7), AAV (2), ACW (1)
- Gas MIG: `UTILMD_MIG_Gas_G1_0a_außerordendliche_20240726.xml`
- Gas AHB: `UTILMD_AHB_Gas_1_0a_außerordentliche_20240726.xml`
- Schema dir: shared `crates/mig-types/src/generated/fv2504/utilmd/pids/` (44* files already exist)
- Generated fixtures: all 95 already exist in `fixtures/generated/fv2504/utilmd/44*.edi`

---

## Task 1: Create UTILMD_Gas Mapping Directory Structure

**Files:**
- Create: `mappings/FV2504/UTILMD_Gas/message/` (directory)
- Create: `mappings/FV2504/UTILMD_Gas/common/` (directory)

**Step 1: Create directory structure**

```bash
mkdir -p mappings/FV2504/UTILMD_Gas/message
mkdir -p mappings/FV2504/UTILMD_Gas/common
```

**Step 2: Commit**

```bash
git add mappings/FV2504/UTILMD_Gas/
git commit -m "chore: create UTILMD_Gas mapping directory structure"
```

---

## Task 2: Create Gas Test Infrastructure

**Files:**
- Create: `crates/mig-bo4e/tests/common/utilmd_gas.rs`
- Modify: `crates/mig-bo4e/tests/common/mod.rs` (add `pub mod utilmd_gas;`)

**Step 1: Create the Gas test module**

Create `crates/mig-bo4e/tests/common/utilmd_gas.rs` following the pattern from `comdis.rs` / `remadv.rs`:

```rust
//! Test utilities for UTILMD Gas message type.

use super::test_utils::MessageTypeConfig;
use mig_bo4e::engine::MappingEngine;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_types::schema::mig::MigSchema;
use std::path::{Path, PathBuf};

pub const MIG_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Gas_G1_0a_außerordendliche_20240726.xml";
pub const AHB_XML_PATH: &str =
    "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Gas_1_0a_außerordentliche_20240726.xml";
pub const FIXTURE_DIR: &str = "../../example_market_communication_bo4e_transactions/UTILMD/FV2504";
pub const GENERATED_FIXTURE_DIR: &str = "../../fixtures/generated/fv2504/utilmd";
pub const MAPPINGS_BASE: &str = "../../mappings/FV2504/UTILMD_Gas";
pub const SCHEMA_DIR: &str = "../../crates/mig-types/src/generated/fv2504/utilmd/pids";

const CONFIG: MessageTypeConfig = MessageTypeConfig {
    mig_xml_path: MIG_XML_PATH,
    ahb_xml_path: AHB_XML_PATH,
    fixture_dir: FIXTURE_DIR,
    mappings_base: MAPPINGS_BASE,
    schema_dir: SCHEMA_DIR,
    message_type: "UTILMD",
    variant: Some("Gas"),
    tx_group: "SG4",
    format_version: "FV2504",
};

pub fn path_resolver() -> PathResolver {
    CONFIG.path_resolver()
}

pub fn message_dir() -> PathBuf {
    CONFIG.message_dir()
}

pub fn common_dir() -> PathBuf {
    CONFIG.common_dir()
}

pub fn pid_dir(pid: &str) -> PathBuf {
    CONFIG.pid_dir(pid)
}

pub fn schema_index(pid: &str) -> PidSchemaIndex {
    CONFIG.schema_index(pid)
}

pub fn load_pid_filtered_mig(pid_id: &str) -> Option<MigSchema> {
    CONFIG.load_pid_filtered_mig(pid_id)
}

pub fn discover_generated_fixture(pid: &str) -> Option<PathBuf> {
    let path = Path::new(GENERATED_FIXTURE_DIR).join(format!("{pid}.edi"));
    if path.exists() { Some(path) } else { None }
}

pub fn load_split_engines(pid: &str) -> (MappingEngine, MappingEngine) {
    CONFIG.load_split_engines(pid)
}

pub fn run_full_roundtrip(pid: &str) {
    CONFIG.run_full_roundtrip(pid);
}

pub fn run_full_roundtrip_with_skip(pid: &str, known_incomplete: &[&str]) {
    CONFIG.run_full_roundtrip_with_skip(pid, known_incomplete);
}
```

**Step 2: Register the module in mod.rs**

Add `pub mod utilmd_gas;` to `crates/mig-bo4e/tests/common/mod.rs`.

**Step 3: Verify compilation**

```bash
cargo check -p mig-bo4e --tests
```

**Step 4: Commit**

```bash
git add crates/mig-bo4e/tests/common/utilmd_gas.rs crates/mig-bo4e/tests/common/mod.rs
git commit -m "feat(utilmd-gas): add test infrastructure for UTILMD Gas variant"
```

---

## Task 3: Create Gas Message-Level TOMLs

Message-level TOMLs (BGM, DTM, NAD MS/MR, CTA/COM) are structurally identical to Strom. Copy from `UTILMD_Strom/message/` with minor adjustments.

**Files:**
- Create: `mappings/FV2504/UTILMD_Gas/message/nachricht.toml`
- Create: `mappings/FV2504/UTILMD_Gas/message/marktteilnehmer.toml`
- Create: `mappings/FV2504/UTILMD_Gas/message/kontakt.toml`

**Step 1: Create nachricht.toml** (identical to Strom)

```toml
[meta]
entity = "Nachricht"
bo4e_type = "Nachricht"
source_group = ""
source_path = ""

[fields]
"bgm.0" = "nachrichtentyp"
"bgm.1" = "nachrichtennummer"
"dtm[137].c507.d2005" = { target = "", default = "137" }
"dtm[137].c507.d2380" = "erstellungsdatum"
"dtm[137].c507.d2379" = { target = "", default = "303" }
```

**Step 2: Create marktteilnehmer.toml**

Gas uses d3055 codes 9 (GS1) and 332 (DVGW) — no 293 (BDEW) or 500 (GLN). Copy Strom's file but update the enum_map.

```toml
[meta]
entity = "Marktteilnehmer"
bo4e_type = "Marktteilnehmer"
source_group = "SG2"
source_path = "sg2"

[fields]
"nad.d3035" = "marktrolle"
"nad.c082.d3039" = "rollencodenummer"
"nad.c082.d1131" = "codelisteCode"

[fields."nad.c082.d3055"]
target = "rollencodetyp"
enum_map = { "9" = "GS1", "293" = "BDEW", "332" = "DVGW", "500" = "GLN" }
```

Note: Keep all 4 enum_map entries (9, 293, 332, 500) for robustness — the engine auto-omits unused ones.

**Step 3: Create kontakt.toml** (identical to Strom)

```toml
[meta]
entity = "Kontakt"
bo4e_type = "Kontakt"
source_group = "SG2.SG3"
source_path = "sg2.sg3_ic"
discriminator = "CTA.d3139=IC"

[fields]
"cta.d3139" = { target = "", default = "IC" }
"cta.c056.d3413" = "kontaktNummer"
"cta.c056.d3412" = "kontaktName"
"com.c076.d3148" = "kommunikationsnummer"
"com.c076.d3155" = "kommunikationsart"
```

**Step 4: Verify TOML syntax**

```bash
cargo test -p mig-bo4e -- --test-threads=1 2>&1 | head -5
```

**Step 5: Commit**

```bash
git add mappings/FV2504/UTILMD_Gas/message/
git commit -m "feat(utilmd-gas): add message-level TOMLs (nachricht, marktteilnehmer, kontakt)"
```

---

## Task 4: Create Gas Common TOMLs

Gas needs common/ patterns for groups shared across many PIDs. Based on analysis:
- `sg5_172` (95 PIDs) — LOC+172 network node
- `_20_rff_z13.toml` (95 PIDs) — RFF+Z13 Prüfidentifikator
- `_21_rff_tn.toml` (47 PIDs) — RFF+TN Transaktionsnummer
- `geschaeftspartner.toml` (41 PIDs) — SG12 NAD, all qualifiers

**Files:**
- Create: `mappings/FV2504/UTILMD_Gas/common/sg5_172.toml`
- Create: `mappings/FV2504/UTILMD_Gas/common/_20_rff_z13.toml`
- Create: `mappings/FV2504/UTILMD_Gas/common/_21_rff_tn.toml`
- Create: `mappings/FV2504/UTILMD_Gas/common/geschaeftspartner.toml`

**Step 1: Create sg5_172.toml** — Meldepunkt/network node

```toml
[meta]
entity = "Meldepunkt"
bo4e_type = "Meldepunkt"
source_group = "SG4.SG5"
source_path = "sg4.sg5_172"
discriminator = "LOC.d3227=172"

[fields]
"loc.c517.d3225" = "meldepunktId"
"loc.c517.d1131" = "lokationCodelisteCode"
"loc.c517.d3055" = "lokationCodepflegeCode"

"loc.d3227" = { target = "", default = "172" }
```

Note: Gas LOC+172 is "Meldepunkt" (reporting point / network node), not "Marktlokation". The LOC has 3 data components: d3225 (ID), d1131 (codelist), d3055 (code maintenance). Check schema to verify if d3224 (zeitraumId) exists.

**Step 2: Create _20_rff_z13.toml** — Prüfidentifikator (same as Strom)

```toml
[meta]
entity = "Prozessdaten"
bo4e_type = "Prozessdaten"
source_group = "SG4.SG6"
source_path = "sg4.sg6"
discriminator = "RFF.c506.d1153=Z13"

[fields]
"rff.c506.d1153" = { target = "", default = "Z13" }
"rff.c506.d1154" = "pruefidentifikator"
```

**Step 3: Create _21_rff_tn.toml** — Transaktionsnummer

```toml
[meta]
entity = "Prozessdaten"
bo4e_type = "Prozessdaten"
source_group = "SG4.SG6"
source_path = "sg4.sg6"
discriminator = "RFF.c506.d1153=TN"

[fields]
"rff.c506.d1153" = { target = "", default = "TN" }
"rff.c506.d1154" = "transaktionsnummer"
```

**Step 4: Create geschaeftspartner.toml**

Follow the NAD entity reuse pattern from CLAUDE.md — one file for ALL SG12 NAD qualifiers, no discriminator → auto-array. Maps the superset of all NAD fields.

```toml
[meta]
entity = "Geschaeftspartner"
bo4e_type = "Geschaeftspartner"
companion_type = "GeschaeftspartnerEdifact"
source_group = "SG4.SG12"
source_path = "sg4.sg12"

[fields]
"nad.c082.d3039" = "identifikation"
"nad.c080.d3036" = "name1"
"nad.c080.d3036_2" = "name2"
"nad.c080.d3036_3" = "namenszusatz1"
"nad.c080.d3036_5" = "anrede"
"nad.c058.d3124" = "zusatzinfo"
"nad.c058.d3124_2" = "zusatzinfo2"
"nad.c058.d3124_3" = "zusatzinfo3"
"nad.c058.d3124_4" = "zusatzinfo4"
"nad.c058.d3124_5" = "zusatzinfo5"
"nad.c059.d3042" = "strasse"
"nad.c059.d3042_2" = "strasseZusatz"
"nad.c059.d3042_3" = "hausnummer"
"nad.c059.d3042_4" = "adresszusatz"
"nad.d3164" = "ort"
"nad.d3251" = "postleitzahl"
"nad.d3207" = "land"
"nad.c819.d3229" = "region"

[companion_fields]
"nad.d3035" = "nad_qualifier"
"nad.c082.d1131" = "codelist_code"
"nad.c082.d3055" = "codepflege_code"
"nad.c080.d3036_4" = "name_format_code"
"nad.c080.d3045" = "anredeCode"
```

Note: Some SG12 variants (Z05, Z09) have child RFF segments. These will need per-PID TOML files that override this common template. The engine's `load_with_common` will replace common SG12 defs when a PID-specific SG12 file exists with matching source_group.

**Step 5: Commit**

```bash
git add mappings/FV2504/UTILMD_Gas/common/
git commit -m "feat(utilmd-gas): add common TOMLs (sg5_172, rff_z13, rff_tn, geschaeftspartner)"
```

---

## Task 5: Map PID 44001 — Reference Implementation (Anmeldung NN)

PID 44001 is the first reference PID (11 SG4 groups). It covers: SG4 root (IDE, DTM×2, STS×2, FTX×2), SG5_172, SG6 (RFF+Z13, RFF+Z18, DTM+Z20), SG8 Z01/Z03/Z07/Z12/Z35, SG12 DP/Z04/Z05/Z09.

**Files:**
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/prozessdaten.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/prozessdaten_rff_z18.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/prozessdaten_dtm_z20.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z01_marktlokation_info.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z01_sg10_merkmal.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z01_sg9_qty.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z03_zaehler.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z03_sg10_merkmal.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z07_konzessionsabgabe.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z07_sg10_merkmal.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z12_gemeinderabatt.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z12_sg9_qty.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z35_lastprofil.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/sg4_sg8_z35_sg10_merkmal.toml`
- Create: `mappings/FV2504/UTILMD_Gas/pid_44001/geschaeftspartner.toml`

**Approach:** Use `schema-lookup` CLI to generate TOML templates for each group, then manually refine entity names and field mappings.

**Step 1: Generate TOML templates using schema-lookup**

```bash
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4 --toml-template
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4.sg8_z01 --toml-template
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4.sg8_z01.sg10 --toml-template
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4.sg8_z01.sg9 --toml-template
# ... etc for each group
```

**Step 2: Create prozessdaten.toml** — SG4 root segments

Map IDE+24 (vorgangId), DTM+92/93 (gueltigAb/gueltigBis), STS+7 (transaktionsgrund), STS+Z17 (lieferendeGrund), FTX+ACB (text), FTX+ADM (zaehlerstandInfo).

Consult the PID schema JSON:
```bash
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4 --toml-template
```

The SG4 root has: IDE, DTM+92, DTM+93, STS+7, STS+Z17, FTX+ACB, FTX+ADM.

```toml
[meta]
entity = "Prozessdaten"
bo4e_type = "Prozessdaten"
source_group = "SG4"
source_path = "sg4"

[fields]
# IDE+24 (Vorgang)
"ide.d7495" = { target = "", default = "24" }
"ide.c206.d7402" = "vorgangId"

# DTM+92 (Beginn Lieferung/Zuordnung)
"dtm[92].c507.d2005" = { target = "", default = "92" }
"dtm[92].c507.d2380" = "gueltigAb"
"dtm[92].c507.d2379" = { target = "", default = "303" }

# DTM+93 (Ende Netznutzung/Zuordnung)
"dtm[93].c507.d2005" = { target = "", default = "93" }
"dtm[93].c507.d2380" = "gueltigBis"
"dtm[93].c507.d2379" = { target = "", default = "303" }

# STS+7 (Transaktionsgrund)
"sts[7].c601.d9015" = { target = "", default = "7" }
"sts[7].c555.d4405" = "statusCode"
"sts[7].c556.d9013" = "transaktionsgrund"

# STS+Z17 (Transaktionsgrund Lieferende)
"sts[Z17].c601.d9015" = { target = "", default = "Z17" }
"sts[Z17].c555.d4405" = "lieferendeStatusCode"
"sts[Z17].c556.d9013" = "lieferendeGrund"

# FTX+ACB (Freitext)
"ftx[ACB].d4451" = { target = "", default = "ACB" }
"ftx[ACB].d4453" = "textFunktionCode"
"ftx[ACB].c107.d4441" = "textReferenzCode"
"ftx[ACB].c108.d4440" = "freitext1"
"ftx[ACB].c108.d4440_2" = "freitext2"
"ftx[ACB].c108.d4440_3" = "freitext3"
"ftx[ACB].c108.d4440_4" = "freitext4"
"ftx[ACB].c108.d4440_5" = "freitext5"

# FTX+ADM (Zählerstand Information)
"ftx[ADM].d4451" = { target = "", default = "ADM" }
"ftx[ADM].d4453" = "zaehlerstandTextFunktion"
"ftx[ADM].c107.d4441" = "zaehlerstandInfo"
```

**Step 3: Create SG6 RFF/DTM mappings**

SG6 has RFF+Z13 (from common), RFF+Z18, and DTM+Z20. Create per-PID files for Z18 and Z20:

`prozessdaten_rff_z18.toml`:
```toml
[meta]
entity = "Prozessdaten"
bo4e_type = "Prozessdaten"
source_group = "SG4.SG6"
source_path = "sg4.sg6"
discriminator = "RFF.c506.d1153=Z18"

[fields]
"rff.c506.d1153" = { target = "", default = "Z18" }
"rff.c506.d1154" = "marktlokationId"
```

`prozessdaten_dtm_z20.toml`:
```toml
[meta]
entity = "Prozessdaten"
bo4e_type = "Prozessdaten"
source_group = "SG4.SG6"
source_path = "sg4.sg6"
discriminator = "DTM.c507.d2005=Z20"

[fields]
"dtm.c507.d2005" = { target = "", default = "Z20" }
"dtm.c507.d2380" = "messintervallMonate"
"dtm.c507.d2379" = { target = "", default = "802" }
```

**Step 4: Create SG8 TOML files**

For each SG8 variant (Z01, Z03, Z07, Z12, Z35), create:
1. An entry-level TOML (SEQ + any RFF/PIA segments)
2. SG10 child TOML (CCI/CAV segments) if SG10 exists
3. SG9 child TOML (QTY segments) if SG9 exists

Use `schema-lookup` for the exact field paths:
```bash
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4.sg8_z01 --toml-template
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4.sg8_z01.sg10 --toml-template
cargo run -p automapper-generator -- schema-lookup --pid 44001 --group sg4.sg8_z01.sg9 --toml-template
# Repeat for z03, z07, z12, z35
```

**Key SG8 entities for Gas:**
- Z01 → "MarktlokationDaten" (market location data, SEQ+Z01, child SG10 + SG9)
- Z03 → "Zaehler" (meter device, SEQ+Z03 + RFF+Z19, child SG10)
- Z07 → "Konzessionsabgabe" (concession fee, SEQ+Z07, child SG10)
- Z12 → "Gemeinderabatt" (municipal discount, SEQ+Z12, child SG9 QTY)
- Z35 → "Lastprofil" (load profile, SEQ+Z35, child SG10)

Each file must have the correct `source_path` (e.g., `sg4.sg8_z01`), `source_group` (e.g., `SG4.SG8`), and `discriminator` (e.g., `SEQ.d1229=Z01`).

For SG10 children, use CCI/CAV qualifier syntax. Consult the schema for exact CCI codes per SG8 variant.

**Step 5: Create geschaeftspartner.toml** — SG12 NAD override

44001 has SG12 variants: DP, Z04, Z05, Z09. Z05 and Z09 have child RFF segments. If the common geschaeftspartner.toml doesn't handle child RFF, create a PID-specific override.

Check whether Z05's `RFF+Z19` and Z09's `RFF+AVC/Z01` can be mapped as additional companion_fields. If so, the common template needs extension. If not, create a PID-specific override that adds RFF mappings.

**Step 6: Write a failing roundtrip test**

Create `crates/mig-bo4e/tests/utilmd_gas_44001_test.rs`:

```rust
mod common;
use common::utilmd_gas;

#[test]
fn test_pid_44001_generated_roundtrip() {
    utilmd_gas::run_full_roundtrip("44001");
}
```

**Step 7: Run the test — expect failure initially**

```bash
cargo test -p mig-bo4e --test utilmd_gas_44001_test -- --nocapture
```

Iterate on TOML files until the roundtrip is byte-identical.

**Step 8: Commit**

```bash
git add mappings/FV2504/UTILMD_Gas/pid_44001/ crates/mig-bo4e/tests/utilmd_gas_44001_test.rs
git commit -m "feat(utilmd-gas): add PID 44001 mappings — reference implementation"
```

---

## Task 6: Extract Common SG8/SG10 Patterns

After 44001 passes, extract reusable common/ patterns for SG8 variants that appear across many PIDs.

**Analysis of SG8 reuse potential (sorted by frequency):**
- sg8_z01: 23 PIDs → `common/sg8_z01_info.toml` + `common/sg8_z01_sg10.toml` + `common/sg8_z01_sg9.toml`
- sg8_z03: 21 PIDs → `common/sg8_z03_zaehler.toml` + `common/sg8_z03_sg10.toml`
- sg8_z05: 16 PIDs → `common/sg8_z05_komm.toml` + `common/sg8_z05_sg10.toml`
- sg8_z13: 16 PIDs → `common/sg8_z13_smgw.toml` + `common/sg8_z13_sg10.toml`
- sg8_z20: 16 PIDs → `common/sg8_z20_obis.toml` + `common/sg8_z20_sg10.toml`
- sg8_z09: 15 PIDs → `common/sg8_z09_umwerter.toml` + `common/sg8_z09_sg10.toml`
- sg8_z50: 15 PIDs → `common/sg8_z50.toml` + `common/sg8_z50_sg10.toml`
- sg8_z35: 13 PIDs → `common/sg8_z35_lastprofil.toml` + `common/sg8_z35_sg10.toml`
- sg8_z18: 13 PIDs → `common/sg8_z18_messlok.toml` + `common/sg8_z18_sg10.toml`
- sg8_z07: 8 PIDs → `common/sg8_z07_konzession.toml` + `common/sg8_z07_sg10.toml`
- sg8_z12: 8 PIDs → `common/sg8_z12_gemeinde.toml` + `common/sg8_z12_sg9.toml`
- sg8_z02: 7 PIDs → `common/sg8_z02_obis.toml` + `common/sg8_z02_sg10.toml`
- sg8_z19: 4 PIDs → `common/sg8_z19.toml` + `common/sg8_z19_sg10.toml`
- sg8_z26: 2 PIDs → keep as per-PID only

**Step 1: Verify that 44001's SG8 TOMLs work identically as common/**

Move the working 44001 SG8 TOMLs to common/, then re-run the 44001 roundtrip to confirm common/ inheritance works.

**IMPORTANT:** Before moving, verify that the SG8 structure (CCI codes, CAV patterns, QTY fields) is consistent across PIDs sharing the same SG8 variant. Use schema-lookup on 2-3 other PIDs with the same SG8:

```bash
cargo run -p automapper-generator -- schema-lookup --pid 44002 --group sg4.sg8_z01.sg10
cargo run -p automapper-generator -- schema-lookup --pid 44019 --group sg4.sg8_z01.sg10
```

If the CCI/CAV structure differs between PIDs, keep the superset in common/ and rely on the engine's empty-value omission.

**Step 2: Move verified SG8 TOMLs to common/**

For each SG8 variant that is consistent across PIDs, move from `pid_44001/` to `common/`.

**Step 3: Re-run 44001 roundtrip to confirm common/ inheritance works**

```bash
cargo test -p mig-bo4e --test utilmd_gas_44001_test -- --nocapture
```

**Step 4: Commit**

```bash
git add mappings/FV2504/UTILMD_Gas/common/ mappings/FV2504/UTILMD_Gas/pid_44001/
git commit -m "refactor(utilmd-gas): extract common SG8/SG10 patterns from 44001"
```

---

## Task 7: Map PID 44002 — Complex Reference (Bestätigung Anmeldung)

PID 44002 has 18 SG4 groups — the most complex Gas PID after 44013. It adds SG8 variants not in 44001: Z02, Z05, Z09, Z13, Z18, Z20, Z50. It also adds STS+E01 (Antwort-Status).

**Step 1: Use schema-lookup for all new SG8 variants**

```bash
for group in sg4.sg8_z02 sg4.sg8_z05 sg4.sg8_z09 sg4.sg8_z13 sg4.sg8_z18 sg4.sg8_z20 sg4.sg8_z50; do
    echo "=== $group ==="
    cargo run -p automapper-generator -- schema-lookup --pid 44002 --group $group --toml-template
done
```

**Step 2: Create per-PID TOMLs for 44002**

44002's prozessdaten.toml will differ from 44001 (adds STS+E01 Antwort-Status, more DTM variants in SG4 root and SG6).

Create TOMLs for each new SG8 variant (Z02, Z05, Z09, Z13, Z18, Z20, Z50) and their SG10 children. These become candidates for common/ if they work across multiple PIDs.

**Step 3: Write roundtrip test**

```rust
#[test]
fn test_pid_44002_generated_roundtrip() {
    utilmd_gas::run_full_roundtrip("44002");
}
```

**Step 4: Iterate until byte-identical**

```bash
cargo test -p mig-bo4e --test utilmd_gas_44001_test -- --nocapture test_pid_44002
```

**Step 5: Move new common-worthy SG8 TOMLs to common/**

After 44002 passes, move Z02/Z05/Z09/Z13/Z18/Z20/Z50 TOMLs to common/ if their structure is consistent with other PIDs.

**Step 6: Commit**

```bash
git commit -m "feat(utilmd-gas): add PID 44002 mappings — 18 groups, complex reference"
```

---

## Task 8: Bulk-Map Simple PIDs (39 PIDs — SG5/SG6 only)

These 39 PIDs have no SG8 or SG12 — just SG5_172 + SG6 (+ SG4 root). They're the easiest to map since they only need a `prozessdaten.toml` per PID (SG4 root segments differ per PID: different DTM/STS/FTX combinations).

**33 PIDs** have `(sg5_172, sg6)`, **4 PIDs** have only `(sg6)`, **2 PIDs** have `(sg5_172, sg5_z08, sg6)`.

**Approach:**

**Step 1: Survey SG4 root segment variations**

```bash
python3 -c "
import json, os
schema_dir = 'crates/mig-types/src/generated/fv2504/utilmd/pids'
for f in sorted(os.listdir(schema_dir)):
    if not (f.startswith('pid_44') and f.endswith('_schema.json')): continue
    pid = f.replace('pid_','').replace('_schema.json','')
    s = json.load(open(os.path.join(schema_dir, f)))
    sg4 = s['fields'].get('sg4', {})
    children = set(sg4.get('children', {}).keys())
    has_sg8 = any(k.startswith('sg8_') for k in children)
    has_sg12 = any(k.startswith('sg12_') for k in children)
    if has_sg8 or has_sg12: continue
    root_segs = [(seg['id'], [el.get('id','') for el in seg.get('elements',[]) if isinstance(el,dict) and 'codes' in el][:1]) for seg in sg4.get('segments',[])]
    print(f'{pid}: {root_segs}')
"
```

**Step 2: Group PIDs by SG4 root structure**

Most simple PIDs share one of a few SG4 patterns (e.g., IDE+24, DTM+93, STS+7, FTX+ACB). Create a template prozessdaten.toml per pattern, then copy with PID-specific adjustments.

**Step 3: Create per-PID directories and prozessdaten.toml for each**

For each simple PID, create `mappings/FV2504/UTILMD_Gas/pid_{PID}/prozessdaten.toml` using the appropriate template. Use schema-lookup to verify exact segments:

```bash
cargo run -p automapper-generator -- schema-lookup --pid {PID} --group sg4 --toml-template
```

**Step 4: Handle sg5_z08 PIDs (44129, 44130)**

These 2 PIDs have both `sg5_172` and `sg5_z08`. Create `common/sg5_z08.toml`:

```bash
cargo run -p automapper-generator -- schema-lookup --pid 44129 --group sg4.sg5_z08 --toml-template
```

**Step 5: Write bulk roundtrip test**

```rust
#[test]
fn test_simple_gas_pids_roundtrip() {
    let simple_pids = ["44004", "44005", "44006", /* ... all 39 PIDs */];
    for pid in &simple_pids {
        utilmd_gas::run_full_roundtrip(pid);
    }
}
```

**Step 6: Iterate until all pass**

```bash
cargo test -p mig-bo4e --test utilmd_gas_bulk_test -- --nocapture
```

**Step 7: Commit**

```bash
git commit -m "feat(utilmd-gas): add 39 simple PID mappings (SG5/SG6 only)"
```

---

## Task 9: Bulk-Map Medium PIDs (44 PIDs — SG8/SG12, < 10 groups)

These PIDs have SG8 and/or SG12 variants but fewer than 10 total groups. They should work mostly with common/ TOMLs plus a per-PID prozessdaten.toml.

**Approach:**

**Step 1: Create per-PID prozessdaten.toml for each**

Each PID needs its own SG4 root mapping (IDE, DTM, STS variants differ per PID).

**Step 2: Check if additional per-PID SG6 TOMLs are needed**

Some PIDs have RFF+Z19, RFF+AAV, RFF+ACW — create common/ files for frequent ones:
- `_22_rff_z18.toml` (11 PIDs)
- `_23_rff_z19.toml` (7 PIDs)
- `_24_rff_aav.toml` (2 PIDs)
- `_25_dtm_z20.toml` (DTM+Z20 in SG6)

**Step 3: Verify common/ SG8 patterns cover these PIDs**

Most medium PIDs reuse the common/ SG8 patterns from Task 6. Check a few edge cases where SG10 CCI/CAV structure might differ.

**Step 4: Handle SG12 variants needing per-PID geschaeftspartner.toml**

PIDs with SG12 variants that have child RFF segments (Z05+RFF, Z09+RFF) need per-PID overrides of the common geschaeftspartner.toml. Check which variants have child segments:

```bash
for pid in 44109 44116 44137 44143 44160 44162 44163; do
    echo "=== PID $pid ==="
    cargo run -p automapper-generator -- schema-lookup --pid $pid --group sg4.sg12_z05
done
```

**Step 5: Write bulk roundtrip test and iterate**

**Step 6: Commit**

```bash
git commit -m "feat(utilmd-gas): add 44 medium PID mappings (SG8/SG12, < 10 groups)"
```

---

## Task 10: Map Complex PIDs (12 PIDs — 10+ groups)

These 12 PIDs have the most groups: 44013 (22), 44002 (18, already done), 44014 (18), 44035 (16), 44168 (16), 44043 (14), 44169 (14), 44112 (12), 44139 (12), 44142 (12), 44001 (11, already done), 44060 (11).

**Approach:**

Most complex PIDs should work with common/ patterns + per-PID prozessdaten.toml. The main complexity is:
- More SG12 NAD qualifiers (DDO, EO, VY, Z25, Z26 — in complex PIDs only)
- More SG6 RFF/DTM variants
- Some have SG8 Z19 or Z26 (rare variants, only 4 and 2 PIDs respectively)

**Step 1: Start with PIDs that share common structures**

Group by similarity:
- 44112/44139/44142: Same structure (12 groups each, DDO/DP/EO/Z25/Z26)
- 44014: Similar to 44002 (Bestätigung variant)
- 44013: Largest (22 groups)

**Step 2: Create any remaining common/ patterns**

SG8 Z19 (4 PIDs) and Z26 (2 PIDs) if not already in common/.

**Step 3: Write roundtrip tests and iterate**

**Step 4: Commit**

```bash
git commit -m "feat(utilmd-gas): add 12 complex PID mappings (10+ groups)"
```

---

## Task 11: Create Gas Bulk Roundtrip Test

Create a comprehensive test file that runs all 95 Gas PIDs through the full roundtrip pipeline.

**Files:**
- Create: `crates/mig-bo4e/tests/utilmd_gas_bulk_roundtrip_test.rs`

**Step 1: Create the test file**

Follow the pattern from `pid_bulk_roundtrip_test.rs` but use `utilmd_gas::` helpers:

```rust
//! Bulk roundtrip tests for all 95 UTILMD Gas PIDs (FV2504).

mod common;
use common::utilmd_gas;

const KNOWN_INCOMPLETE: &[&str] = &[];

#[test]
fn test_all_gas_pids_roundtrip() {
    let all_pids = [
        "44001", "44002", "44003", "44004", "44005", "44006",
        // ... all 95 PIDs
    ];

    for pid in &all_pids {
        if KNOWN_INCOMPLETE.contains(pid) {
            eprintln!("Skipping known-incomplete PID {pid}");
            continue;
        }
        eprintln!("Testing PID {pid}...");
        utilmd_gas::run_full_roundtrip(pid);
    }
}
```

**Step 2: Run and verify all 95 pass**

```bash
cargo test -p mig-bo4e --test utilmd_gas_bulk_roundtrip_test -- --nocapture
```

**Step 3: Commit**

```bash
git commit -m "test(utilmd-gas): add bulk roundtrip test for all 95 Gas PIDs"
```

---

## Task 12: Final Verification and Cleanup

**Step 1: Run full workspace checks**

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

**Step 2: Count final metrics**

```bash
find mappings/FV2504/UTILMD_Gas -name "*.toml" | wc -l  # Total TOML files
find mappings/FV2504/UTILMD_Gas -name "*.toml" -path "*/common/*" | wc -l  # Common
find mappings/FV2504/UTILMD_Gas -name "*.toml" -path "*/pid_*/*" | wc -l  # Per-PID
```

**Step 3: Update MEMORY.md**

Add Gas mapping stats to the MIG-Driven Pipeline Progress section.

**Step 4: Final commit**

```bash
git commit -m "feat(utilmd-gas): complete UTILMD Gas FV2504 mapping — 95 PIDs, N TOML files, 95 roundtrip tests"
```

---

## Key Technical Notes

1. **Gas shares schema_dir with Strom**: Both 44* and 55* schemas live in `crates/mig-types/src/generated/fv2504/utilmd/pids/`. The PathResolver loads ALL schemas from this directory, so Gas PIDs can resolve EDIFACT ID paths without any changes.

2. **Gas generated fixtures already exist**: All 95 are in `fixtures/generated/fv2504/utilmd/44*.edi`. No fixture generation needed.

3. **MIG XML variant**: Gas uses `variant: Some("Gas")` which tells `parse_mig()` and `parse_ahb()` to use the Gas-specific MIG/AHB XMLs.

4. **SG2/SG3 normalization**: The `normalize_sg2_sg3_ordering()` function from Strom's bulk test may be needed for Gas too — generated fixtures place CTA/COM after all NADs.

5. **NAD entity reuse**: Follow CLAUDE.md's NAD pattern — single `geschaeftspartner.toml` with no discriminator → auto-array for all SG12 variants. PIDs with child RFF under SG12 need per-PID overrides.

6. **LOC+172 entity naming**: Gas uses LOC+172 ("Meldepunkt" / network node), not Z16-Z22 location types. Verify the BO4E entity type — it may be `Marktlokation` with a different lokationstyp code, or a new entity.

7. **SG6 DTM discriminators**: DTM+Z20 in SG6 uses format code 802 (month count), not 303 (datetime). Handle this in the DTM TOML mapping.
