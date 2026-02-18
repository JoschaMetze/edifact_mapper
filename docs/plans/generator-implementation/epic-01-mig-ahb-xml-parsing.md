---
feature: generator-implementation
epic: 1
title: "MIG/AHB XML Schema Parsing"
depends_on: []
estimated_tasks: 7
crate: automapper-generator
status: complete
---

# Epic 1: MIG/AHB XML Schema Parsing

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-generator/src/`. The binary entry point is `crates/automapper-generator/src/main.rs`. Use types from Section 7 of the design doc exactly. All code must compile with `cargo check -p automapper-generator`.

**Goal:** Build the XML parsing layer that reads BDEW MIG (Message Implementation Guide) and AHB (Anwendungshandbuch) XML files into strongly-typed Rust structs. These structs are the foundation for all code generation in Epics 2 and 3. Port the C# `MigXmlParser` and `AhbXmlParser` classes to idiomatic Rust using `quick-xml`.

**Architecture:** SAX-style pull parsing with `quick-xml::Reader` for memory efficiency on large XML files. MIG XML uses element-name prefixes (`S_`, `G_`, `C_`, `D_`, `M_`) to distinguish segments, groups, composites, data elements, and message containers. AHB XML uses `AWF` elements with `Pruefidentifikator` attributes and a `Bedingungen` section. Both parsers produce owned schema structs (no lifetimes needed since we process one file at a time).

**Tech Stack:** quick-xml 0.37, thiserror 2.x, serde + serde_json (for schema serialization in tests), insta (snapshot tests)

---

## Task 1: Crate scaffold and `GeneratorError`

### Step 1 — Write the test

Create the crate directory structure and a test that `GeneratorError` variants exist and display correctly.

Create `crates/automapper-generator/Cargo.toml`:

```toml
[package]
name = "automapper-generator"
version = "0.1.0"
edition = "2021"
description = "CLI tool for generating Rust mapper code from MIG/AHB XML schemas"

[[bin]]
name = "automapper-generator"
path = "src/main.rs"

[dependencies]
thiserror = "2"
quick-xml = "0.37"
clap = { version = "4", features = ["derive"] }
askama = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"

[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
tempfile = "3"
```

Add the crate to the workspace `Cargo.toml` members list.

Create `crates/automapper-generator/src/error.rs`:

```rust
use std::path::PathBuf;

/// Errors that can occur during code generation.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("XML parsing error in {path}: {message}")]
    XmlParse {
        path: PathBuf,
        message: String,
        #[source]
        source: Option<quick_xml::Error>,
    },

    #[error("missing required attribute '{attribute}' on element '{element}' in {path}")]
    MissingAttribute {
        path: PathBuf,
        element: String,
        attribute: String,
    },

    #[error("invalid element '{element}' in {path} at line {line}")]
    InvalidElement {
        path: PathBuf,
        element: String,
        line: usize,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("template rendering error: {0}")]
    Template(String),

    #[error("claude CLI error: {message}")]
    ClaudeCli { message: String },

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("validation error: {message}")]
    Validation { message: String },

    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
}
```

Create `crates/automapper-generator/src/lib.rs`:

```rust
pub mod error;

pub use error::GeneratorError;
```

Create `crates/automapper-generator/src/main.rs`:

```rust
fn main() {
    println!("automapper-generator: not yet implemented");
}
```

Create `crates/automapper-generator/tests/error_tests.rs`:

```rust
use automapper_generator::GeneratorError;
use std::path::PathBuf;

#[test]
fn test_xml_parse_error_display() {
    let err = GeneratorError::XmlParse {
        path: PathBuf::from("test.xml"),
        message: "unexpected EOF".to_string(),
        source: None,
    };
    assert_eq!(
        err.to_string(),
        "XML parsing error in test.xml: unexpected EOF"
    );
}

#[test]
fn test_missing_attribute_error_display() {
    let err = GeneratorError::MissingAttribute {
        path: PathBuf::from("mig.xml"),
        element: "S_UNH".to_string(),
        attribute: "Versionsnummer".to_string(),
    };
    assert!(err
        .to_string()
        .contains("missing required attribute 'Versionsnummer'"));
}

#[test]
fn test_file_not_found_error_display() {
    let err = GeneratorError::FileNotFound(PathBuf::from("/nonexistent/file.xml"));
    assert!(err.to_string().contains("/nonexistent/file.xml"));
}

#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let gen_err: GeneratorError = io_err.into();
    assert!(gen_err.to_string().contains("file not found"));
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test error_tests
```

Expected: Tests fail because the crate does not exist yet or compile errors.

### Step 3 — Implement

Create the files listed above. Ensure the workspace `Cargo.toml` includes `"crates/automapper-generator"` in members.

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test error_tests
```

Expected: All 4 tests pass.

```
running 4 tests
test test_xml_parse_error_display ... ok
test test_missing_attribute_error_display ... ok
test test_file_not_found_error_display ... ok
test test_io_error_conversion ... ok
```

### Step 5 — Verify compilation

```bash
cargo check -p automapper-generator
```

### Step 6 — Commit

```bash
git add -A && git commit -m "feat(generator): scaffold automapper-generator crate with GeneratorError"
```

---

## Task 2: MIG schema types — `MigSchema`, `MigSegment`, `MigComposite`, `MigDataElement`

### Step 1 — Write the test

Create `crates/automapper-generator/src/schema/mod.rs`:

```rust
pub mod mig;
pub mod ahb;
pub mod common;
```

Create `crates/automapper-generator/src/schema/common.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Cardinality of a segment or group in the MIG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cardinality {
    /// Mandatory (M) — must appear exactly once or as specified.
    Mandatory,
    /// Required (R) — must appear.
    Required,
    /// Dependent (D) — depends on other segments.
    Dependent,
    /// Optional (O) — may appear.
    Optional,
    /// Not used (N) — must not appear.
    NotUsed,
    /// Conditional (C) — conditional on context.
    Conditional,
}

impl Cardinality {
    /// Parse from a status string (e.g., "M", "C", "R", "D", "O", "N").
    pub fn from_status(status: &str) -> Self {
        match status.trim() {
            "M" => Cardinality::Mandatory,
            "R" => Cardinality::Required,
            "D" => Cardinality::Dependent,
            "O" => Cardinality::Optional,
            "N" => Cardinality::NotUsed,
            "C" => Cardinality::Conditional,
            _ => Cardinality::Conditional, // Default for unknown
        }
    }

    /// Whether this cardinality means the element is required.
    pub fn is_required(&self) -> bool {
        matches!(self, Cardinality::Mandatory | Cardinality::Required)
    }
}

/// An allowed code value for a data element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeDefinition {
    /// The code value (e.g., "ORDERS", "E40").
    pub value: String,
    /// Human-readable name of the code.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
}

/// EDIFACT data element format type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdifactDataType {
    Alphabetic,
    Numeric,
    Alphanumeric,
}

/// Parsed EDIFACT format specification (e.g., "an..35", "n13").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdifactFormat {
    pub data_type: EdifactDataType,
    /// Minimum length (None for variable-length formats like "an..35").
    pub min_length: Option<usize>,
    /// Maximum length.
    pub max_length: usize,
}

impl EdifactFormat {
    /// Parse an EDIFACT format string (e.g., "an..35", "n13", "a3").
    pub fn parse(format: &str) -> Option<Self> {
        let format = format.trim();
        if format.is_empty() {
            return None;
        }

        // Regex-free parsing: extract type prefix, optional "..", and length
        let (type_str, rest) = if format.starts_with("an") {
            ("an", &format[2..])
        } else if format.starts_with('a') {
            ("a", &format[1..])
        } else if format.starts_with('n') {
            ("n", &format[1..])
        } else {
            return None;
        };

        let data_type = match type_str {
            "a" => EdifactDataType::Alphabetic,
            "n" => EdifactDataType::Numeric,
            "an" => EdifactDataType::Alphanumeric,
            _ => return None,
        };

        let (is_variable, length_str) = if let Some(stripped) = rest.strip_prefix("..") {
            (true, stripped)
        } else {
            (false, rest)
        };

        let max_length: usize = length_str.parse().ok()?;
        let min_length = if is_variable { None } else { Some(max_length) };

        Some(EdifactFormat {
            data_type,
            min_length,
            max_length,
        })
    }
}
```

Create `crates/automapper-generator/src/schema/mig.rs`:

```rust
use serde::{Deserialize, Serialize};

use super::common::{Cardinality, CodeDefinition};

/// Complete MIG schema for a message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSchema {
    /// The EDIFACT message type (e.g., "UTILMD", "ORDERS").
    pub message_type: String,
    /// Optional variant (e.g., "Strom", "Gas").
    pub variant: Option<String>,
    /// Version number from the MIG (e.g., "S2.1", "1.4a").
    pub version: String,
    /// Publication date string.
    pub publication_date: String,
    /// Author (typically "BDEW").
    pub author: String,
    /// Format version directory (e.g., "FV2504").
    pub format_version: String,
    /// Path to the source XML file.
    pub source_file: String,
    /// Top-level segment definitions (not in groups).
    pub segments: Vec<MigSegment>,
    /// Segment group definitions (contain more segments).
    pub segment_groups: Vec<MigSegmentGroup>,
}

/// A segment (S_*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSegment {
    /// Segment identifier (e.g., "UNH", "BGM", "NAD").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of the segment.
    pub description: Option<String>,
    /// Position counter (e.g., "0010", "0020").
    pub counter: Option<String>,
    /// Nesting level (0=root, 1=first level, etc.).
    pub level: i32,
    /// Sequence number within the message.
    pub number: Option<String>,
    /// Standard maximum repetitions.
    pub max_rep_std: i32,
    /// Specification maximum repetitions.
    pub max_rep_spec: i32,
    /// Standard status (M=Mandatory, C=Conditional, etc.).
    pub status_std: Option<String>,
    /// Specification status (M, R, D, O, N).
    pub status_spec: Option<String>,
    /// Example EDIFACT string.
    pub example: Option<String>,
    /// Direct child data elements.
    pub data_elements: Vec<MigDataElement>,
    /// Child composite elements.
    pub composites: Vec<MigComposite>,
}

impl MigSegment {
    /// Returns the effective cardinality based on spec or std status.
    pub fn cardinality(&self) -> Cardinality {
        let status = self
            .status_spec
            .as_deref()
            .or(self.status_std.as_deref())
            .unwrap_or("C");
        Cardinality::from_status(status)
    }

    /// Returns the effective max repetitions (spec overrides std).
    pub fn max_rep(&self) -> i32 {
        self.max_rep_spec.max(self.max_rep_std)
    }
}

/// A segment group (G_SG*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigSegmentGroup {
    /// Group identifier (e.g., "SG1", "SG2", "SG10").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of the segment group.
    pub description: Option<String>,
    /// Position counter (e.g., "0070", "0500").
    pub counter: Option<String>,
    /// Nesting level.
    pub level: i32,
    /// Standard maximum repetitions.
    pub max_rep_std: i32,
    /// Specification maximum repetitions.
    pub max_rep_spec: i32,
    /// Standard status.
    pub status_std: Option<String>,
    /// Specification status.
    pub status_spec: Option<String>,
    /// Segments directly in this group.
    pub segments: Vec<MigSegment>,
    /// Nested segment groups.
    pub nested_groups: Vec<MigSegmentGroup>,
}

impl MigSegmentGroup {
    /// Returns the effective cardinality.
    pub fn cardinality(&self) -> Cardinality {
        let status = self
            .status_spec
            .as_deref()
            .or(self.status_std.as_deref())
            .unwrap_or("C");
        Cardinality::from_status(status)
    }
}

/// A composite element (C_*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigComposite {
    /// Composite identifier (e.g., "S009", "C002").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Standard status.
    pub status_std: Option<String>,
    /// Specification status.
    pub status_spec: Option<String>,
    /// Child data elements within this composite.
    pub data_elements: Vec<MigDataElement>,
    /// Position of this composite within its parent segment (0-based).
    pub position: usize,
}

/// A data element (D_*) definition from the MIG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigDataElement {
    /// Element identifier (e.g., "0062", "3035").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Standard status.
    pub status_std: Option<String>,
    /// Specification status.
    pub status_spec: Option<String>,
    /// Standard format (e.g., "an..14", "n13").
    pub format_std: Option<String>,
    /// Specification format.
    pub format_spec: Option<String>,
    /// Allowed code values, if restricted.
    pub codes: Vec<CodeDefinition>,
    /// Position within parent (0-based).
    pub position: usize,
}
```

Update `crates/automapper-generator/src/lib.rs`:

```rust
pub mod error;
pub mod schema;

pub use error::GeneratorError;
```

Write the test at `crates/automapper-generator/tests/schema_types_tests.rs`:

```rust
use automapper_generator::schema::common::{Cardinality, CodeDefinition, EdifactFormat, EdifactDataType};
use automapper_generator::schema::mig::*;

#[test]
fn test_cardinality_from_status() {
    assert_eq!(Cardinality::from_status("M"), Cardinality::Mandatory);
    assert_eq!(Cardinality::from_status("R"), Cardinality::Required);
    assert_eq!(Cardinality::from_status("C"), Cardinality::Conditional);
    assert_eq!(Cardinality::from_status("D"), Cardinality::Dependent);
    assert_eq!(Cardinality::from_status("O"), Cardinality::Optional);
    assert_eq!(Cardinality::from_status("N"), Cardinality::NotUsed);
}

#[test]
fn test_cardinality_is_required() {
    assert!(Cardinality::Mandatory.is_required());
    assert!(Cardinality::Required.is_required());
    assert!(!Cardinality::Conditional.is_required());
    assert!(!Cardinality::Optional.is_required());
}

#[test]
fn test_edifact_format_parse() {
    let f = EdifactFormat::parse("an..35").unwrap();
    assert_eq!(f.data_type, EdifactDataType::Alphanumeric);
    assert_eq!(f.min_length, None);
    assert_eq!(f.max_length, 35);

    let f = EdifactFormat::parse("n13").unwrap();
    assert_eq!(f.data_type, EdifactDataType::Numeric);
    assert_eq!(f.min_length, Some(13));
    assert_eq!(f.max_length, 13);

    let f = EdifactFormat::parse("a3").unwrap();
    assert_eq!(f.data_type, EdifactDataType::Alphabetic);
    assert_eq!(f.min_length, Some(3));
    assert_eq!(f.max_length, 3);

    assert!(EdifactFormat::parse("").is_none());
    assert!(EdifactFormat::parse("xyz").is_none());
}

#[test]
fn test_mig_segment_cardinality() {
    let seg = MigSegment {
        id: "UNH".to_string(),
        name: "Message Header".to_string(),
        description: None,
        counter: Some("0010".to_string()),
        level: 0,
        number: None,
        max_rep_std: 1,
        max_rep_spec: 1,
        status_std: Some("M".to_string()),
        status_spec: Some("M".to_string()),
        example: None,
        data_elements: vec![],
        composites: vec![],
    };
    assert!(seg.cardinality().is_required());
    assert_eq!(seg.max_rep(), 1);
}

#[test]
fn test_mig_schema_serialization_roundtrip() {
    let schema = MigSchema {
        message_type: "UTILMD".to_string(),
        variant: Some("Strom".to_string()),
        version: "S2.1".to_string(),
        publication_date: "20250320".to_string(),
        author: "BDEW".to_string(),
        format_version: "FV2510".to_string(),
        source_file: "test.xml".to_string(),
        segments: vec![MigSegment {
            id: "UNH".to_string(),
            name: "Message Header".to_string(),
            description: Some("Nachrichtenkopfsegment".to_string()),
            counter: Some("0010".to_string()),
            level: 0,
            number: Some("1".to_string()),
            max_rep_std: 1,
            max_rep_spec: 1,
            status_std: Some("M".to_string()),
            status_spec: Some("M".to_string()),
            example: Some("UNH+1+UTILMD:D:11A:UN:S2.1".to_string()),
            data_elements: vec![MigDataElement {
                id: "0062".to_string(),
                name: "Nachrichten-Referenznummer".to_string(),
                description: None,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                format_std: Some("an..14".to_string()),
                format_spec: Some("an..14".to_string()),
                codes: vec![],
                position: 0,
            }],
            composites: vec![MigComposite {
                id: "S009".to_string(),
                name: "Nachrichtenkennung".to_string(),
                description: None,
                status_std: Some("M".to_string()),
                status_spec: Some("M".to_string()),
                data_elements: vec![MigDataElement {
                    id: "0065".to_string(),
                    name: "Nachrichtentyp-Kennung".to_string(),
                    description: None,
                    status_std: Some("M".to_string()),
                    status_spec: Some("M".to_string()),
                    format_std: Some("an..6".to_string()),
                    format_spec: Some("an..6".to_string()),
                    codes: vec![CodeDefinition {
                        value: "UTILMD".to_string(),
                        name: "Stammdaten".to_string(),
                        description: None,
                    }],
                    position: 0,
                }],
                position: 1,
            }],
        }],
        segment_groups: vec![],
    };

    let json = serde_json::to_string_pretty(&schema).unwrap();
    let roundtripped: MigSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtripped.message_type, "UTILMD");
    assert_eq!(roundtripped.variant, Some("Strom".to_string()));
    assert_eq!(roundtripped.segments.len(), 1);
    assert_eq!(roundtripped.segments[0].composites[0].data_elements[0].codes[0].value, "UTILMD");
}

#[test]
fn test_code_definition() {
    let code = CodeDefinition {
        value: "E40".to_string(),
        name: "Energieart Strom".to_string(),
        description: Some("Electricity energy type".to_string()),
    };
    assert_eq!(code.value, "E40");
    assert_eq!(code.description.as_deref(), Some("Electricity energy type"));
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test schema_types_tests
```

Expected: Compilation errors because schema module doesn't exist yet.

### Step 3 — Implement

Create the files listed above.

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test schema_types_tests
```

Expected:

```
running 6 tests
test test_cardinality_from_status ... ok
test test_cardinality_is_required ... ok
test test_edifact_format_parse ... ok
test test_mig_segment_cardinality ... ok
test test_mig_schema_serialization_roundtrip ... ok
test test_code_definition ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): add MIG schema types with Cardinality, EdifactFormat, and serialization"
```

---

## Task 3: AHB schema types — `AhbSchema`, `Pruefidentifikator`, `AhbRule`, `BedingungDefinition`

### Step 1 — Write the test

Create `crates/automapper-generator/src/schema/ahb.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Complete AHB schema for a message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbSchema {
    /// The EDIFACT message type (e.g., "UTILMD", "ORDERS").
    pub message_type: String,
    /// Optional variant (e.g., "Strom", "Gas").
    pub variant: Option<String>,
    /// Version number from the AHB (e.g., "2.1").
    pub version: String,
    /// Format version directory (e.g., "FV2510").
    pub format_version: String,
    /// Path to the source XML file.
    pub source_file: String,
    /// All AWF (Anwendungsfall) definitions with their PIDs.
    pub workflows: Vec<Pruefidentifikator>,
    /// All condition definitions from the Bedingungen section.
    pub bedingungen: Vec<BedingungDefinition>,
}

/// An Anwendungsfall (AWF/workflow) definition from an AHB.
/// Each AWF has a unique Pruefidentifikator (PID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pruefidentifikator {
    /// The unique PID (e.g., "55001", "55002").
    pub id: String,
    /// Description of the workflow.
    pub beschreibung: String,
    /// Communication direction (e.g., "NB an LF").
    pub kommunikation_von: Option<String>,
    /// All fields required/allowed for this PID.
    pub fields: Vec<AhbFieldDefinition>,
}

/// A field definition extracted from an AHB for a specific Pruefidentifikator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbFieldDefinition {
    /// Path to the field (e.g., "SG2/NAD/C082/3039").
    pub segment_path: String,
    /// Human-readable name of the field.
    pub name: String,
    /// Status in this AHB context (e.g., "X", "Muss", "Kann", "X [condition]").
    pub ahb_status: String,
    /// Optional description.
    pub description: Option<String>,
    /// Valid code values for this field (if restricted).
    pub codes: Vec<AhbCodeValue>,
}

/// A valid code value for an AHB field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbCodeValue {
    /// The code value.
    pub value: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Status in this AHB context.
    pub ahb_status: Option<String>,
}

/// A condition definition from the Bedingungen section of an AHB XML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedingungDefinition {
    /// The condition ID (e.g., "931", "494").
    pub id: String,
    /// The German description text.
    pub description: String,
}

/// An AHB rule binding a condition expression to a segment/field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbRule {
    /// The segment path this rule applies to.
    pub segment_path: String,
    /// The raw condition expression (e.g., "[1] AND [2]", "Muss [494]").
    pub condition_expression: String,
    /// Whether the field is mandatory under this rule.
    pub is_mandatory: bool,
}

impl AhbFieldDefinition {
    /// Whether this field is mandatory (status is "Muss" or "X" without conditions).
    pub fn is_mandatory(&self) -> bool {
        self.ahb_status == "Muss" || self.ahb_status == "X"
    }

    /// Extract all condition IDs referenced in the AHB status string.
    /// Matches patterns like "[931]", "[494]", "[1] AND [2]".
    pub fn condition_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        let mut chars = self.ahb_status.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '[' {
                let id: String = chars.by_ref().take_while(|&c| c != ']').collect();
                if !id.is_empty() {
                    ids.push(id);
                }
            }
        }
        ids
    }
}
```

Write `crates/automapper-generator/tests/ahb_types_tests.rs`:

```rust
use automapper_generator::schema::ahb::*;

#[test]
fn test_ahb_field_is_mandatory() {
    let field_muss = AhbFieldDefinition {
        segment_path: "SG2/NAD/3035".to_string(),
        name: "Qualifier".to_string(),
        ahb_status: "Muss".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(field_muss.is_mandatory());

    let field_x = AhbFieldDefinition {
        segment_path: "SG2/NAD/3035".to_string(),
        name: "Qualifier".to_string(),
        ahb_status: "X".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(field_x.is_mandatory());

    let field_conditional = AhbFieldDefinition {
        segment_path: "SG2/NAD/3035".to_string(),
        name: "Qualifier".to_string(),
        ahb_status: "X [931]".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(!field_conditional.is_mandatory());
}

#[test]
fn test_ahb_field_condition_ids() {
    let field = AhbFieldDefinition {
        segment_path: "SG8/SEQ/1245".to_string(),
        name: "Status".to_string(),
        ahb_status: "Muss [1] \u{2227} [2]".to_string(), // "Muss [1] ∧ [2]"
        description: None,
        codes: vec![],
    };
    let ids = field.condition_ids();
    assert_eq!(ids, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn test_ahb_field_no_conditions() {
    let field = AhbFieldDefinition {
        segment_path: "BGM/C002/1001".to_string(),
        name: "Doc Type".to_string(),
        ahb_status: "Muss".to_string(),
        description: None,
        codes: vec![],
    };
    assert!(field.condition_ids().is_empty());
}

#[test]
fn test_bedingung_definition() {
    let bed = BedingungDefinition {
        id: "931".to_string(),
        description: "Wenn Zeitformat korrekt ist".to_string(),
    };
    assert_eq!(bed.id, "931");
}

#[test]
fn test_ahb_schema_serialization_roundtrip() {
    let schema = AhbSchema {
        message_type: "UTILMD".to_string(),
        variant: Some("Strom".to_string()),
        version: "2.1".to_string(),
        format_version: "FV2510".to_string(),
        source_file: "test_ahb.xml".to_string(),
        workflows: vec![Pruefidentifikator {
            id: "55001".to_string(),
            beschreibung: "Lieferantenwechsel".to_string(),
            kommunikation_von: Some("NB an LF".to_string()),
            fields: vec![AhbFieldDefinition {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Qualifier".to_string(),
                ahb_status: "X".to_string(),
                description: None,
                codes: vec![AhbCodeValue {
                    value: "MS".to_string(),
                    name: "Absender".to_string(),
                    description: None,
                    ahb_status: Some("X".to_string()),
                }],
            }],
        }],
        bedingungen: vec![BedingungDefinition {
            id: "1".to_string(),
            description: "Wenn Aufteilung vorhanden".to_string(),
        }],
    };

    let json = serde_json::to_string_pretty(&schema).unwrap();
    let roundtripped: AhbSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtripped.message_type, "UTILMD");
    assert_eq!(roundtripped.workflows.len(), 1);
    assert_eq!(roundtripped.workflows[0].id, "55001");
    assert_eq!(roundtripped.bedingungen.len(), 1);
}

#[test]
fn test_pruefidentifikator_summary() {
    let pid = Pruefidentifikator {
        id: "55001".to_string(),
        beschreibung: "Lieferantenwechsel".to_string(),
        kommunikation_von: Some("NB an LF".to_string()),
        fields: vec![
            AhbFieldDefinition {
                segment_path: "BGM/1001".to_string(),
                name: "Doc Type".to_string(),
                ahb_status: "Muss".to_string(),
                description: None,
                codes: vec![],
            },
            AhbFieldDefinition {
                segment_path: "SG2/NAD/3035".to_string(),
                name: "Qualifier".to_string(),
                ahb_status: "X [931]".to_string(),
                description: None,
                codes: vec![],
            },
        ],
    };

    let total = pid.fields.len();
    let mandatory_count = pid.fields.iter().filter(|f| f.is_mandatory()).count();
    assert_eq!(total, 2);
    assert_eq!(mandatory_count, 1); // Only "Muss" is mandatory, "X [931]" is conditional
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test ahb_types_tests
```

### Step 3 — Implement

Create the `ahb.rs` file as shown above.

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test ahb_types_tests
```

Expected:

```
running 6 tests
test test_ahb_field_is_mandatory ... ok
test test_ahb_field_condition_ids ... ok
test test_ahb_field_no_conditions ... ok
test test_bedingung_definition ... ok
test test_ahb_schema_serialization_roundtrip ... ok
test test_pruefidentifikator_summary ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): add AHB schema types with Pruefidentifikator and condition extraction"
```

---

## Task 4: MIG XML parser — `parse_mig()` with `quick-xml`

### Step 1 — Write the test

Create `crates/automapper-generator/src/parsing/mod.rs`:

```rust
pub mod mig_parser;
pub mod ahb_parser;
```

Create a minimal test XML fixture at `crates/automapper-generator/tests/fixtures/minimal_mig.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<M_UTILMD Versionsnummer="S2.1" Veroeffentlichungsdatum="20250320" Author="BDEW" Bezeichnung="UTILMD Strom">
    <S_UNH Name="Nachrichtenkopfsegment" Counter="0010" Level="0" Number="1" MaxRep_Std="1" MaxRep_Specification="1" Status_Std="M" Status_Specification="M" Example="UNH+1+UTILMD:D:11A:UN:S2.1">
        <D_0062 Name="Nachrichten-Referenznummer" Status_Std="M" Status_Specification="M" Format_Std="an..14" Format_Specification="an..14"/>
        <C_S009 Name="Nachrichtenkennung" Status_Std="M" Status_Specification="M">
            <D_0065 Name="Nachrichtentyp-Kennung" Status_Std="M" Status_Specification="M" Format_Std="an..6" Format_Specification="an..6">
                <Code Name="Stammdaten">UTILMD</Code>
            </D_0065>
            <D_0052 Name="Versionsnummer" Status_Std="M" Status_Specification="M" Format_Std="an..3" Format_Specification="an..3">
                <Code Name="Entwurfs-Verzeichnis">D</Code>
            </D_0052>
        </C_S009>
    </S_UNH>
    <S_BGM Name="Beginn der Nachricht" Counter="0020" Level="0" MaxRep_Std="1" MaxRep_Specification="1" Status_Std="M" Status_Specification="M">
        <C_C002 Name="Dokumentenname" Status_Std="C" Status_Specification="R">
            <D_1001 Name="Dokumentenname, Code" Status_Std="C" Status_Specification="R" Format_Std="an..3" Format_Specification="an..3">
                <Code Name="Utilities master data message">E40</Code>
            </D_1001>
        </C_C002>
    </S_BGM>
    <G_SG2 Name="Segment Gruppe 2" Counter="0070" Level="1" MaxRep_Std="99" MaxRep_Specification="99" Status_Std="C" Status_Specification="M">
        <S_NAD Name="Name und Adresse" Counter="0080" Level="2" MaxRep_Std="1" MaxRep_Specification="1" Status_Std="M" Status_Specification="M">
            <D_3035 Name="Beteiligter, Qualifier" Status_Std="M" Status_Specification="M" Format_Std="an..3" Format_Specification="an..3">
                <Code Name="Absender">MS</Code>
                <Code Name="Empfaenger">MR</Code>
            </D_3035>
            <C_C082 Name="Beteiligtenidentifikation" Status_Std="C" Status_Specification="R">
                <D_3039 Name="Beteiligtenkennung" Status_Std="M" Status_Specification="M" Format_Std="an..35" Format_Specification="an..35"/>
                <D_1131 Name="Codeliste, Qualifier" Status_Std="C" Status_Specification="N" Format_Std="an..17" Format_Specification="an..17"/>
                <D_3055 Name="Verantwortliche Stelle Code" Status_Std="C" Status_Specification="R" Format_Std="an..3" Format_Specification="an..3">
                    <Code Name="DE, BDEW">293</Code>
                </D_3055>
            </C_C082>
        </S_NAD>
    </G_SG2>
</M_UTILMD>
```

Write `crates/automapper-generator/tests/mig_parsing_tests.rs`:

```rust
use automapper_generator::parsing::mig_parser::parse_mig;
use std::path::Path;

#[test]
fn test_parse_minimal_mig() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510")
        .expect("should parse minimal MIG XML");

    assert_eq!(schema.message_type, "UTILMD");
    assert_eq!(schema.variant, Some("Strom".to_string()));
    assert_eq!(schema.version, "S2.1");
    assert_eq!(schema.author, "BDEW");
    assert_eq!(schema.format_version, "FV2510");
}

#[test]
fn test_parse_mig_segments() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Should have 2 top-level segments: UNH, BGM
    assert_eq!(schema.segments.len(), 2);

    let unh = &schema.segments[0];
    assert_eq!(unh.id, "UNH");
    assert_eq!(unh.name, "Nachrichtenkopfsegment");
    assert_eq!(unh.counter, Some("0010".to_string()));
    assert_eq!(unh.status_std, Some("M".to_string()));
    assert_eq!(unh.example, Some("UNH+1+UTILMD:D:11A:UN:S2.1".to_string()));

    // UNH should have 1 data element (D_0062) and 1 composite (C_S009)
    assert_eq!(unh.data_elements.len(), 1);
    assert_eq!(unh.composites.len(), 1);

    let de_0062 = &unh.data_elements[0];
    assert_eq!(de_0062.id, "0062");
    assert_eq!(de_0062.format_std, Some("an..14".to_string()));
    assert_eq!(de_0062.position, 0);

    let s009 = &unh.composites[0];
    assert_eq!(s009.id, "S009");
    assert_eq!(s009.position, 1); // After the data element
    assert_eq!(s009.data_elements.len(), 2); // D_0065 and D_0052

    let de_0065 = &s009.data_elements[0];
    assert_eq!(de_0065.id, "0065");
    assert_eq!(de_0065.codes.len(), 1);
    assert_eq!(de_0065.codes[0].value, "UTILMD");
    assert_eq!(de_0065.codes[0].name, "Stammdaten");
}

#[test]
fn test_parse_mig_segment_groups() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Should have 1 segment group: SG2
    assert_eq!(schema.segment_groups.len(), 1);

    let sg2 = &schema.segment_groups[0];
    assert_eq!(sg2.id, "SG2");
    assert_eq!(sg2.counter, Some("0070".to_string()));
    assert_eq!(sg2.max_rep_spec, 99);
    assert_eq!(sg2.status_spec, Some("M".to_string()));

    // SG2 should contain NAD segment
    assert_eq!(sg2.segments.len(), 1);
    let nad = &sg2.segments[0];
    assert_eq!(nad.id, "NAD");

    // NAD should have 1 data element (D_3035) and 1 composite (C_C082)
    assert_eq!(nad.data_elements.len(), 1);
    assert_eq!(nad.composites.len(), 1);

    let d_3035 = &nad.data_elements[0];
    assert_eq!(d_3035.codes.len(), 2); // MS and MR
    assert_eq!(d_3035.codes[0].value, "MS");
    assert_eq!(d_3035.codes[1].value, "MR");

    let c082 = &nad.composites[0];
    assert_eq!(c082.data_elements.len(), 3); // 3039, 1131, 3055
}

#[test]
fn test_parse_mig_bgm_codes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let bgm = &schema.segments[1];
    assert_eq!(bgm.id, "BGM");
    assert_eq!(bgm.composites.len(), 1);

    let c002 = &bgm.composites[0];
    assert_eq!(c002.id, "C002");
    assert_eq!(c002.data_elements[0].codes[0].value, "E40");
}

#[test]
fn test_parse_mig_nonexistent_file() {
    let result = parse_mig(
        Path::new("/nonexistent/file.xml"),
        "UTILMD",
        None,
        "FV2510",
    );
    assert!(result.is_err());
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test mig_parsing_tests
```

### Step 3 — Implement

Create `crates/automapper-generator/src/parsing/mig_parser.rs`:

```rust
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::GeneratorError;
use crate::schema::common::CodeDefinition;
use crate::schema::mig::*;

/// Parses a MIG XML file into a `MigSchema`.
///
/// The MIG XML uses element-name prefixes to distinguish types:
/// - `S_*` — segments (e.g., `S_UNH`, `S_BGM`)
/// - `G_*` — segment groups (e.g., `G_SG2`)
/// - `C_*` — composites (e.g., `C_S009`, `C_C002`)
/// - `D_*` — data elements (e.g., `D_0062`, `D_3035`)
/// - `M_*` — message containers (e.g., `M_UTILMD`)
/// - `Code` — code values within data elements
pub fn parse_mig(
    path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
) -> Result<MigSchema, GeneratorError> {
    if !path.exists() {
        return Err(GeneratorError::FileNotFound(path.to_path_buf()));
    }

    let xml_content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut schema = MigSchema {
        message_type: message_type.to_string(),
        variant: variant.map(|v| v.to_string()),
        version: String::new(),
        publication_date: String::new(),
        author: "BDEW".to_string(),
        format_version: format_version.to_string(),
        source_file: path.to_string_lossy().to_string(),
        segments: Vec::new(),
        segment_groups: Vec::new(),
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|_| GeneratorError::XmlParse {
                        path: path.to_path_buf(),
                        message: "invalid UTF-8 in element name".to_string(),
                        source: None,
                    })?
                    .to_string();

                if name.starts_with("M_") {
                    // Message container — extract root attributes
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = attr.unescape_value().unwrap_or_default().to_string();
                        match key {
                            "Versionsnummer" => schema.version = val,
                            "Veroeffentlichungsdatum" => schema.publication_date = val,
                            "Author" => schema.author = val,
                            _ => {}
                        }
                    }
                    // Continue parsing children — don't skip
                } else if name.starts_with("S_") {
                    let is_empty = matches!(reader.read_event_into(&mut Vec::new()), _);
                    // Re-parse: we need to handle both self-closing and nested
                    // Use the dedicated segment parser
                    let segment = parse_segment_from_xml(&name, e, &mut reader, path)?;
                    schema.segments.push(segment);
                } else if name.starts_with("G_") {
                    let group = parse_group_from_xml(&name, e, &mut reader, path)?;
                    schema.segment_groups.push(group);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    if schema.version.is_empty() {
        return Err(GeneratorError::MissingAttribute {
            path: path.to_path_buf(),
            element: format!("M_{}", message_type),
            attribute: "Versionsnummer".to_string(),
        });
    }

    Ok(schema)
}

fn get_attr(e: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .and_then(|a| a.unescape_value().ok().map(|v| v.to_string()))
}

fn get_attr_i32(e: &quick_xml::events::BytesStart, key: &str, default: i32) -> i32 {
    get_attr(e, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_segment_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<MigSegment, GeneratorError> {
    let id = element_name.strip_prefix("S_").unwrap_or(element_name).to_string();

    let mut segment = MigSegment {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        counter: get_attr(start, "Counter"),
        level: get_attr_i32(start, "Level", 0),
        number: get_attr(start, "Number"),
        max_rep_std: get_attr_i32(start, "MaxRep_Std", 1),
        max_rep_spec: get_attr_i32(start, "MaxRep_Specification", 1),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        example: get_attr(start, "Example"),
        data_elements: Vec::new(),
        composites: Vec::new(),
    };

    let mut position: usize = 0;
    let mut buf = Vec::new();
    let end_name = element_name.to_string();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("D_") {
                    let de = parse_data_element_from_xml(&name, e, reader, path, position)?;
                    segment.data_elements.push(de);
                    position += 1;
                } else if name.starts_with("C_") {
                    let comp = parse_composite_from_xml(&name, e, reader, path, position)?;
                    segment.composites.push(comp);
                    position += 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("D_") {
                    let de = MigDataElement {
                        id: name.strip_prefix("D_").unwrap_or(&name).to_string(),
                        name: get_attr(e, "Name").unwrap_or_default(),
                        description: get_attr(e, "Description"),
                        status_std: get_attr(e, "Status_Std"),
                        status_spec: get_attr(e, "Status_Specification"),
                        format_std: get_attr(e, "Format_Std"),
                        format_spec: get_attr(e, "Format_Specification"),
                        codes: Vec::new(),
                        position,
                    };
                    segment.data_elements.push(de);
                    position += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == end_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(segment)
}

fn parse_group_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<MigSegmentGroup, GeneratorError> {
    let id = element_name.strip_prefix("G_").unwrap_or(element_name).to_string();

    let mut group = MigSegmentGroup {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        counter: get_attr(start, "Counter"),
        level: get_attr_i32(start, "Level", 0),
        max_rep_std: get_attr_i32(start, "MaxRep_Std", 1),
        max_rep_spec: get_attr_i32(start, "MaxRep_Specification", 1),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        segments: Vec::new(),
        nested_groups: Vec::new(),
    };

    let mut buf = Vec::new();
    let end_name = element_name.to_string();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("S_") {
                    let seg = parse_segment_from_xml(&name, e, reader, path)?;
                    group.segments.push(seg);
                } else if name.starts_with("G_") {
                    let nested = parse_group_from_xml(&name, e, reader, path)?;
                    group.nested_groups.push(nested);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == end_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(group)
}

fn parse_composite_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
    position: usize,
) -> Result<MigComposite, GeneratorError> {
    let id = element_name.strip_prefix("C_").unwrap_or(element_name).to_string();

    let mut composite = MigComposite {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        data_elements: Vec::new(),
        position,
    };

    let mut component_position: usize = 0;
    let mut buf = Vec::new();
    let end_name = element_name.to_string();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("D_") {
                    let de =
                        parse_data_element_from_xml(&name, e, reader, path, component_position)?;
                    composite.data_elements.push(de);
                    component_position += 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("D_") {
                    let de = MigDataElement {
                        id: name.strip_prefix("D_").unwrap_or(&name).to_string(),
                        name: get_attr(e, "Name").unwrap_or_default(),
                        description: get_attr(e, "Description"),
                        status_std: get_attr(e, "Status_Std"),
                        status_spec: get_attr(e, "Status_Specification"),
                        format_std: get_attr(e, "Format_Std"),
                        format_spec: get_attr(e, "Format_Specification"),
                        codes: Vec::new(),
                        position: component_position,
                    };
                    composite.data_elements.push(de);
                    component_position += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == end_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(composite)
}

fn parse_data_element_from_xml(
    element_name: &str,
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
    position: usize,
) -> Result<MigDataElement, GeneratorError> {
    let id = element_name.strip_prefix("D_").unwrap_or(element_name).to_string();

    let mut de = MigDataElement {
        id,
        name: get_attr(start, "Name").unwrap_or_default(),
        description: get_attr(start, "Description"),
        status_std: get_attr(start, "Status_Std"),
        status_spec: get_attr(start, "Status_Specification"),
        format_std: get_attr(start, "Format_Std"),
        format_spec: get_attr(start, "Format_Specification"),
        codes: Vec::new(),
        position,
    };

    let mut buf = Vec::new();
    let end_name = element_name.to_string();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == "Code" {
                    let code = parse_code_from_xml(e, reader, path)?;
                    de.codes.push(code);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == end_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(de)
}

fn parse_code_from_xml(
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<CodeDefinition, GeneratorError> {
    let name = get_attr(start, "Name").unwrap_or_default();
    let description = get_attr(start, "Description");

    // Read the text content of the Code element
    let mut value = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                value = t
                    .unescape()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
            }
            Ok(Event::End(ref e)) => {
                let tag = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if tag == "Code" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(CodeDefinition {
        value,
        name,
        description,
    })
}
```

Update `crates/automapper-generator/src/lib.rs`:

```rust
pub mod error;
pub mod schema;
pub mod parsing;

pub use error::GeneratorError;
```

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test mig_parsing_tests
```

Expected:

```
running 5 tests
test test_parse_minimal_mig ... ok
test test_parse_mig_segments ... ok
test test_parse_mig_segment_groups ... ok
test test_parse_mig_bgm_codes ... ok
test test_parse_mig_nonexistent_file ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): implement MIG XML parser with quick-xml"
```

---

## Task 5: AHB XML parser — `parse_ahb()` with `quick-xml`

### Step 1 — Write the test

Create `crates/automapper-generator/tests/fixtures/minimal_ahb.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<AHB_UTILMD Versionsnummer="2.1">
    <AWF Pruefidentifikator="55001" Beschreibung="Lieferantenwechsel" Kommunikation_von="NB an LF">
        <Uebertragungsdatei>
            <M_UTILMD>
                <S_BGM Name="Beginn der Nachricht">
                    <C_C002 Name="Dokumentenname">
                        <D_1001 Name="Dokumentenname, Code" AHB_Status="X">
                            <Code Name="UTILMD" AHB_Status="X">E40</Code>
                        </D_1001>
                    </C_C002>
                </S_BGM>
                <G_SG2 Name="Segment Gruppe 2" AHB_Status="Muss">
                    <S_NAD Name="Name und Adresse">
                        <D_3035 Name="Qualifier" AHB_Status="X">
                            <Code Name="Absender" AHB_Status="X">MS</Code>
                            <Code Name="Empfaenger" AHB_Status="X">MR</Code>
                        </D_3035>
                        <C_C082 Name="Beteiligtenidentifikation">
                            <D_3039 Name="MP-ID" AHB_Status="X"/>
                        </C_C082>
                    </S_NAD>
                </G_SG2>
                <G_SG8 Name="Segment Gruppe 8" AHB_Status="[166] &#x2227; [2351]">
                    <S_SEQ Name="Sequenz">
                        <D_1245 Name="Statusanzeiger" AHB_Status="X [931]"/>
                    </S_SEQ>
                </G_SG8>
            </M_UTILMD>
        </Uebertragungsdatei>
    </AWF>
    <AWF Pruefidentifikator="55002" Beschreibung="Ein-/Auszug" Kommunikation_von="LF an NB">
        <Uebertragungsdatei>
            <M_UTILMD>
                <S_BGM Name="Beginn der Nachricht">
                    <C_C002 Name="Dokumentenname">
                        <D_1001 Name="Dokumentenname, Code" AHB_Status="Muss"/>
                    </C_C002>
                </S_BGM>
            </M_UTILMD>
        </Uebertragungsdatei>
    </AWF>
    <Bedingungen>
        <Bedingung Nummer="[1]">Wenn Aufteilung vorhanden</Bedingung>
        <Bedingung Nummer="[2]">Wenn Transaktionsgrund vorhanden</Bedingung>
        <Bedingung Nummer="[931]">Wenn Zeitformat korrekt ist</Bedingung>
    </Bedingungen>
</AHB_UTILMD>
```

Write `crates/automapper-generator/tests/ahb_parsing_tests.rs`:

```rust
use automapper_generator::parsing::ahb_parser::parse_ahb;
use std::path::Path;

#[test]
fn test_parse_minimal_ahb() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510")
        .expect("should parse minimal AHB XML");

    assert_eq!(schema.message_type, "UTILMD");
    assert_eq!(schema.variant, Some("Strom".to_string()));
    assert_eq!(schema.version, "2.1");
    assert_eq!(schema.format_version, "FV2510");
}

#[test]
fn test_parse_ahb_workflows() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    assert_eq!(schema.workflows.len(), 2);

    let wf1 = &schema.workflows[0];
    assert_eq!(wf1.id, "55001");
    assert_eq!(wf1.beschreibung, "Lieferantenwechsel");
    assert_eq!(wf1.kommunikation_von, Some("NB an LF".to_string()));
}

#[test]
fn test_parse_ahb_fields() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf1 = &schema.workflows[0];

    // Should capture fields with AHB_Status
    assert!(!wf1.fields.is_empty());

    // Find the D_1001 field
    let d1001 = wf1.fields.iter().find(|f| f.segment_path.contains("1001"));
    assert!(d1001.is_some(), "should find D_1001 field");
    let d1001 = d1001.unwrap();
    assert_eq!(d1001.ahb_status, "X");
    assert_eq!(d1001.codes.len(), 1); // E40
    assert_eq!(d1001.codes[0].value, "E40");

    // Find the D_3035 field (in SG2/NAD)
    let d3035 = wf1.fields.iter().find(|f| f.segment_path.contains("3035"));
    assert!(d3035.is_some(), "should find D_3035 field");
    let d3035 = d3035.unwrap();
    assert_eq!(d3035.codes.len(), 2); // MS and MR

    // Find the D_3039 field (in SG2/NAD/C082)
    let d3039 = wf1.fields.iter().find(|f| f.segment_path.contains("3039"));
    assert!(d3039.is_some(), "should find D_3039 field");
}

#[test]
fn test_parse_ahb_conditional_group() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf1 = &schema.workflows[0];

    // SG8 has a conditional status "[166] AND [2351]" — should be captured as a field
    let sg8_field = wf1
        .fields
        .iter()
        .find(|f| f.segment_path.contains("SG8") && f.ahb_status.contains("[166]"));
    assert!(
        sg8_field.is_some(),
        "should capture SG8 group-level conditional status"
    );

    // D_1245 has "X [931]"
    let d1245 = wf1
        .fields
        .iter()
        .find(|f| f.segment_path.contains("1245"));
    assert!(d1245.is_some(), "should find D_1245 field");
    assert_eq!(d1245.unwrap().ahb_status, "X [931]");
}

#[test]
fn test_parse_ahb_bedingungen() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    assert_eq!(schema.bedingungen.len(), 3);
    assert_eq!(schema.bedingungen[0].id, "1");
    assert_eq!(
        schema.bedingungen[0].description,
        "Wenn Aufteilung vorhanden"
    );
    assert_eq!(schema.bedingungen[1].id, "2");
    assert_eq!(schema.bedingungen[2].id, "931");
    assert_eq!(
        schema.bedingungen[2].description,
        "Wenn Zeitformat korrekt ist"
    );
}

#[test]
fn test_parse_ahb_second_workflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    let wf2 = &schema.workflows[1];
    assert_eq!(wf2.id, "55002");
    assert_eq!(wf2.beschreibung, "Ein-/Auszug");
    assert_eq!(wf2.kommunikation_von, Some("LF an NB".to_string()));

    // Should have at least the D_1001 field with "Muss"
    let d1001 = wf2.fields.iter().find(|f| f.segment_path.contains("1001"));
    assert!(d1001.is_some());
    assert_eq!(d1001.unwrap().ahb_status, "Muss");
}

#[test]
fn test_parse_ahb_nonexistent_file() {
    let result = parse_ahb(
        Path::new("/nonexistent/ahb.xml"),
        "UTILMD",
        None,
        "FV2510",
    );
    assert!(result.is_err());
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test ahb_parsing_tests
```

### Step 3 — Implement

Create `crates/automapper-generator/src/parsing/ahb_parser.rs`:

```rust
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::GeneratorError;
use crate::schema::ahb::*;

/// Parses an AHB XML file into an `AhbSchema`.
///
/// AHB XML structure:
/// - Root element: `AHB_UTILMD` (or similar) with `Versionsnummer`
/// - `AWF` elements with `Pruefidentifikator`, `Beschreibung`, `Kommunikation_von`
///   - `Uebertragungsdatei` → `M_UTILMD` → nested segments/groups with `AHB_Status`
/// - `Bedingungen` → `Bedingung` elements with `Nummer` attribute and text content
pub fn parse_ahb(
    path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
) -> Result<AhbSchema, GeneratorError> {
    if !path.exists() {
        return Err(GeneratorError::FileNotFound(path.to_path_buf()));
    }

    let xml_content = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut schema = AhbSchema {
        message_type: message_type.to_string(),
        variant: variant.map(|v| v.to_string()),
        version: String::new(),
        format_version: format_version.to_string(),
        source_file: path.to_string_lossy().to_string(),
        workflows: Vec::new(),
        bedingungen: Vec::new(),
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("AHB_") {
                    // Root element — extract version
                    if let Some(v) = get_attr(e, "Versionsnummer") {
                        schema.version = v;
                    }
                } else if name == "AWF" {
                    let workflow = parse_workflow(e, &mut reader, path)?;
                    schema.workflows.push(workflow);
                } else if name == "Bedingungen" {
                    schema.bedingungen = parse_bedingungen(&mut reader, path)?;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    if schema.version.is_empty() {
        return Err(GeneratorError::MissingAttribute {
            path: path.to_path_buf(),
            element: format!("AHB_{}", message_type),
            attribute: "Versionsnummer".to_string(),
        });
    }

    Ok(schema)
}

fn get_attr(e: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .and_then(|a| a.unescape_value().ok().map(|v| v.to_string()))
}

fn parse_workflow(
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<Pruefidentifikator, GeneratorError> {
    let pid = get_attr(start, "Pruefidentifikator").unwrap_or_default();
    let beschreibung = get_attr(start, "Beschreibung").unwrap_or_default();
    let kommunikation_von = get_attr(start, "Kommunikation_von");

    let mut fields = Vec::new();
    let mut path_stack: Vec<String> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("S_") || name.starts_with("G_") {
                    let stripped = name.split_at(2).1.to_string();
                    path_stack.push(stripped);

                    // Capture group-level conditional AHB_Status
                    if let Some(ahb_status) = get_attr(e, "AHB_Status") {
                        if ahb_status.contains('[') {
                            let seg_path = path_stack.join("/");
                            let field_name =
                                get_attr(e, "Name").unwrap_or_else(|| name[2..].to_string());
                            fields.push(AhbFieldDefinition {
                                segment_path: seg_path,
                                name: field_name,
                                ahb_status,
                                description: None,
                                codes: Vec::new(),
                            });
                        }
                    }
                } else if name.starts_with("C_") {
                    let stripped = name.split_at(2).1.to_string();
                    path_stack.push(stripped);
                } else if name.starts_with("D_") {
                    let data_element_id = name[2..].to_string();
                    let ahb_status = get_attr(e, "AHB_Status").unwrap_or_default();
                    let field_name =
                        get_attr(e, "Name").unwrap_or_else(|| data_element_id.clone());

                    // Parse codes within this data element
                    let codes = parse_ahb_codes(reader, &name, path)?;

                    let has_ahb_status = !ahb_status.is_empty();
                    let has_codes_with_status = codes.iter().any(|c| c.ahb_status.is_some());

                    if has_ahb_status || has_codes_with_status {
                        let seg_path =
                            format!("{}/{}", path_stack.join("/"), data_element_id);

                        let effective_status = if !ahb_status.is_empty() {
                            ahb_status
                        } else {
                            codes
                                .iter()
                                .find_map(|c| c.ahb_status.clone())
                                .unwrap_or_default()
                        };

                        fields.push(AhbFieldDefinition {
                            segment_path: seg_path,
                            name: field_name,
                            ahb_status: effective_status,
                            description: None,
                            codes,
                        });
                    }
                    // Note: we already consumed the end tag in parse_ahb_codes
                    buf.clear();
                    continue;
                } else if name.starts_with("M_") || name == "Uebertragungsdatei" {
                    // Message containers — just continue parsing children
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with("D_") {
                    let data_element_id = name[2..].to_string();
                    let ahb_status = get_attr(e, "AHB_Status").unwrap_or_default();

                    if !ahb_status.is_empty() {
                        let seg_path =
                            format!("{}/{}", path_stack.join("/"), data_element_id);
                        let field_name =
                            get_attr(e, "Name").unwrap_or_else(|| data_element_id.clone());

                        fields.push(AhbFieldDefinition {
                            segment_path: seg_path,
                            name: field_name,
                            ahb_status,
                            description: None,
                            codes: Vec::new(),
                        });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();

                if name == "AWF" {
                    break;
                } else if name.starts_with("S_")
                    || name.starts_with("G_")
                    || name.starts_with("C_")
                {
                    path_stack.pop();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(Pruefidentifikator {
        id: pid,
        beschreibung,
        kommunikation_von,
        fields,
    })
}

/// Parse Code elements within a data element, consuming up to the closing D_ tag.
fn parse_ahb_codes(
    reader: &mut Reader<&[u8]>,
    end_element: &str,
    path: &Path,
) -> Result<Vec<AhbCodeValue>, GeneratorError> {
    let mut codes = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == "Code" {
                    let code_name = get_attr(e, "Name").unwrap_or_default();
                    let description = get_attr(e, "Description");
                    let ahb_status = get_attr(e, "AHB_Status");

                    // Read text content
                    let mut value = String::new();
                    let mut inner_buf = Vec::new();
                    loop {
                        match reader.read_event_into(&mut inner_buf) {
                            Ok(Event::Text(ref t)) => {
                                value = t.unescape().unwrap_or_default().trim().to_string();
                            }
                            Ok(Event::End(ref end)) => {
                                if std::str::from_utf8(end.name().as_ref()).unwrap_or("") == "Code"
                                {
                                    break;
                                }
                            }
                            Ok(Event::Eof) => break,
                            Err(e) => {
                                return Err(GeneratorError::XmlParse {
                                    path: path.to_path_buf(),
                                    message: e.to_string(),
                                    source: Some(e),
                                })
                            }
                            _ => {}
                        }
                        inner_buf.clear();
                    }

                    codes.push(AhbCodeValue {
                        value,
                        name: code_name,
                        description,
                        ahb_status,
                    });
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == end_element {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(codes)
}

fn parse_bedingungen(
    reader: &mut Reader<&[u8]>,
    path: &Path,
) -> Result<Vec<BedingungDefinition>, GeneratorError> {
    let mut bedingungen = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == "Bedingung" {
                    let nummer = get_attr(e, "Nummer").unwrap_or_default();
                    // Extract ID from [NNN] format
                    let id = nummer.trim_matches(|c| c == '[' || c == ']').to_string();

                    // Read text content
                    let mut description = String::new();
                    let mut inner_buf = Vec::new();
                    loop {
                        match reader.read_event_into(&mut inner_buf) {
                            Ok(Event::Text(ref t)) => {
                                description =
                                    t.unescape().unwrap_or_default().trim().to_string();
                            }
                            Ok(Event::End(ref end)) => {
                                if std::str::from_utf8(end.name().as_ref()).unwrap_or("")
                                    == "Bedingung"
                                {
                                    break;
                                }
                            }
                            Ok(Event::Eof) => break,
                            Err(e) => {
                                return Err(GeneratorError::XmlParse {
                                    path: path.to_path_buf(),
                                    message: e.to_string(),
                                    source: Some(e),
                                })
                            }
                            _ => {}
                        }
                        inner_buf.clear();
                    }

                    if !id.is_empty() {
                        bedingungen.push(BedingungDefinition { id, description });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap_or("");
                if name == "Bedingungen" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(GeneratorError::XmlParse {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                    source: Some(e),
                })
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(bedingungen)
}
```

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test ahb_parsing_tests
```

Expected:

```
running 7 tests
test test_parse_minimal_ahb ... ok
test test_parse_ahb_workflows ... ok
test test_parse_ahb_fields ... ok
test test_parse_ahb_conditional_group ... ok
test test_parse_ahb_bedingungen ... ok
test test_parse_ahb_second_workflow ... ok
test test_parse_ahb_nonexistent_file ... ok
```

### Step 5 — Commit

```bash
git add -A && git commit -m "feat(generator): implement AHB XML parser with workflow and Bedingungen extraction"
```

---

## Task 6: Snapshot tests for parsed schemas

### Step 1 — Write the test

Write `crates/automapper-generator/tests/schema_snapshot_tests.rs`:

```rust
use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::parsing::ahb_parser::parse_ahb;
use std::path::Path;

#[test]
fn test_mig_schema_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    insta::assert_yaml_snapshot!("mig_schema", schema);
}

#[test]
fn test_ahb_schema_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();
    insta::assert_yaml_snapshot!("ahb_schema", schema);
}

#[test]
fn test_mig_segment_details_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_mig.xml");
    let schema = parse_mig(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Snapshot just the UNH segment for detailed inspection
    let unh = &schema.segments[0];
    insta::assert_yaml_snapshot!("mig_unh_segment", unh);
}

#[test]
fn test_ahb_workflow_details_snapshot() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/minimal_ahb.xml");
    let schema = parse_ahb(&path, "UTILMD", Some("Strom"), "FV2510").unwrap();

    // Snapshot the first workflow's fields
    let wf1 = &schema.workflows[0];
    insta::assert_yaml_snapshot!("ahb_workflow_55001", wf1);
}
```

### Step 2 — Run the test (RED)

```bash
cargo test -p automapper-generator --test schema_snapshot_tests
```

Expected: Tests fail because no snapshots exist yet.

### Step 3 — Accept snapshots

```bash
cargo insta test -p automapper-generator --test schema_snapshot_tests --accept
```

### Step 4 — Run the test (GREEN)

```bash
cargo test -p automapper-generator --test schema_snapshot_tests
```

Expected: All 4 snapshot tests pass.

### Step 5 — Verify snapshot files exist

```bash
ls crates/automapper-generator/tests/snapshots/
```

Expected: `schema_snapshot_tests__mig_schema.snap`, `schema_snapshot_tests__ahb_schema.snap`, etc.

### Step 6 — Commit

```bash
git add -A && git commit -m "test(generator): add insta snapshot tests for MIG and AHB parsing"
```

---

## Task 7: Parse real XML files from submodule (integration test)

### Step 1 — Write the test

Write `crates/automapper-generator/tests/real_xml_tests.rs`:

```rust
//! Integration tests that parse real MIG/AHB XML files from the submodule.
//! These tests are gated behind an environment variable since the submodule
//! may not be available in all build environments.
use std::path::Path;

use automapper_generator::parsing::mig_parser::parse_mig;
use automapper_generator::parsing::ahb_parser::parse_ahb;

/// Helper to locate the xml-migs-and-ahbs directory.
/// Returns None if the submodule is not initialized.
fn find_xml_submodule() -> Option<std::path::PathBuf> {
    // Walk up from the crate directory to find the workspace root
    let mut dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    for _ in 0..5 {
        let candidate = dir.join("xml-migs-and-ahbs");
        if candidate.is_dir() {
            // Check it's not empty (submodule is initialized)
            if std::fs::read_dir(&candidate).ok()?.next().is_some() {
                return Some(candidate);
            }
        }
        dir = dir.parent()?.to_path_buf();
    }
    None
}

#[test]
fn test_parse_real_mig_files() {
    let xml_dir = match find_xml_submodule() {
        Some(dir) => dir,
        None => {
            eprintln!("SKIPPED: xml-migs-and-ahbs submodule not found or empty");
            return;
        }
    };

    // Try to find MIG XML files in any format version directory
    let mut parsed_count = 0;
    for entry in std::fs::read_dir(&xml_dir).unwrap() {
        let entry = entry.unwrap();
        let fv_path = entry.path();
        if !fv_path.is_dir() {
            continue;
        }

        let fv_name = fv_path.file_name().unwrap().to_string_lossy().to_string();
        if !fv_name.starts_with("FV") {
            continue;
        }

        for file_entry in std::fs::read_dir(&fv_path).unwrap() {
            let file_entry = file_entry.unwrap();
            let file_name = file_entry.file_name().to_string_lossy().to_string();

            if file_name.contains("_MIG_") && file_name.ends_with(".xml") {
                let path = file_entry.path();
                // Infer message type from filename
                let msg_type = file_name.split('_').next().unwrap_or("UNKNOWN");
                let variant = if file_name.contains("Strom") {
                    Some("Strom")
                } else if file_name.contains("Gas") {
                    Some("Gas")
                } else {
                    None
                };

                match parse_mig(&path, msg_type, variant, &fv_name) {
                    Ok(schema) => {
                        assert!(!schema.version.is_empty(), "MIG version should not be empty");
                        assert!(
                            !schema.segments.is_empty() || !schema.segment_groups.is_empty(),
                            "MIG should have segments or groups"
                        );
                        parsed_count += 1;
                        eprintln!(
                            "  OK: {} {} {} — {} segments, {} groups",
                            schema.message_type,
                            schema.variant.as_deref().unwrap_or(""),
                            fv_name,
                            schema.segments.len(),
                            schema.segment_groups.len()
                        );
                    }
                    Err(e) => {
                        panic!("Failed to parse MIG {}: {}", file_name, e);
                    }
                }
            }
        }
    }

    if parsed_count > 0 {
        eprintln!("Parsed {} MIG files successfully", parsed_count);
    }
}

#[test]
fn test_parse_real_ahb_files() {
    let xml_dir = match find_xml_submodule() {
        Some(dir) => dir,
        None => {
            eprintln!("SKIPPED: xml-migs-and-ahbs submodule not found or empty");
            return;
        }
    };

    let mut parsed_count = 0;
    for entry in std::fs::read_dir(&xml_dir).unwrap() {
        let entry = entry.unwrap();
        let fv_path = entry.path();
        if !fv_path.is_dir() {
            continue;
        }

        let fv_name = fv_path.file_name().unwrap().to_string_lossy().to_string();
        if !fv_name.starts_with("FV") {
            continue;
        }

        for file_entry in std::fs::read_dir(&fv_path).unwrap() {
            let file_entry = file_entry.unwrap();
            let file_name = file_entry.file_name().to_string_lossy().to_string();

            if file_name.contains("_AHB_") && file_name.ends_with(".xml") {
                let path = file_entry.path();
                let msg_type = file_name.split('_').next().unwrap_or("UNKNOWN");
                let variant = if file_name.contains("Strom") {
                    Some("Strom")
                } else if file_name.contains("Gas") {
                    Some("Gas")
                } else {
                    None
                };

                match parse_ahb(&path, msg_type, variant, &fv_name) {
                    Ok(schema) => {
                        assert!(!schema.version.is_empty(), "AHB version should not be empty");
                        parsed_count += 1;
                        eprintln!(
                            "  OK: {} {} {} — {} workflows, {} conditions",
                            schema.message_type,
                            schema.variant.as_deref().unwrap_or(""),
                            fv_name,
                            schema.workflows.len(),
                            schema.bedingungen.len()
                        );
                    }
                    Err(e) => {
                        panic!("Failed to parse AHB {}: {}", file_name, e);
                    }
                }
            }
        }
    }

    if parsed_count > 0 {
        eprintln!("Parsed {} AHB files successfully", parsed_count);
    }
}

#[test]
fn test_mig_captures_all_qualifiers() {
    let xml_dir = match find_xml_submodule() {
        Some(dir) => dir,
        None => {
            eprintln!("SKIPPED: xml-migs-and-ahbs submodule not found or empty");
            return;
        }
    };

    // Find a UTILMD MIG file and verify qualifier capture
    let mut found = false;
    for entry in std::fs::read_dir(&xml_dir).unwrap().flatten() {
        let fv_path = entry.path();
        if !fv_path.is_dir() {
            continue;
        }
        let fv_name = fv_path.file_name().unwrap().to_string_lossy().to_string();

        for file_entry in std::fs::read_dir(&fv_path).unwrap().flatten() {
            let file_name = file_entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("UTILMD_MIG_Strom") && file_name.ends_with(".xml") {
                let schema =
                    parse_mig(&file_entry.path(), "UTILMD", Some("Strom"), &fv_name).unwrap();

                // Collect all code values across all segments (recursively)
                let mut all_codes = Vec::new();
                collect_codes_from_segments(&schema.segments, &mut all_codes);
                for group in &schema.segment_groups {
                    collect_codes_from_group(group, &mut all_codes);
                }

                assert!(
                    !all_codes.is_empty(),
                    "should capture code values from UTILMD MIG"
                );
                eprintln!("Captured {} total code values from UTILMD MIG", all_codes.len());
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
}

fn collect_codes_from_segments(
    segments: &[automapper_generator::schema::mig::MigSegment],
    codes: &mut Vec<String>,
) {
    for seg in segments {
        for de in &seg.data_elements {
            for code in &de.codes {
                codes.push(format!("{}/{}: {}", seg.id, de.id, code.value));
            }
        }
        for comp in &seg.composites {
            for de in &comp.data_elements {
                for code in &de.codes {
                    codes.push(format!("{}/{}/{}: {}", seg.id, comp.id, de.id, code.value));
                }
            }
        }
    }
}

fn collect_codes_from_group(
    group: &automapper_generator::schema::mig::MigSegmentGroup,
    codes: &mut Vec<String>,
) {
    collect_codes_from_segments(&group.segments, codes);
    for nested in &group.nested_groups {
        collect_codes_from_group(nested, codes);
    }
}
```

### Step 2 — Run the test

```bash
cargo test -p automapper-generator --test real_xml_tests -- --nocapture
```

Expected: If the submodule is present, all real XML files parse successfully. If not, tests are skipped gracefully.

### Step 3 — Commit

```bash
git add -A && git commit -m "test(generator): add integration tests for real MIG/AHB XML files from submodule"
```

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 35 |
| Passed | 35 |
| Failed | 0 |
| Skipped | 0 (3 integration tests skip gracefully when submodule absent) |

Files tested:
- crates/automapper-generator/src/error.rs
- crates/automapper-generator/src/schema/common.rs
- crates/automapper-generator/src/schema/mig.rs
- crates/automapper-generator/src/schema/ahb.rs
- crates/automapper-generator/src/parsing/mig_parser.rs
- crates/automapper-generator/src/parsing/ahb_parser.rs
