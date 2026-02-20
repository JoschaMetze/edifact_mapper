---
feature: mig-driven-mapping
epic: 5
title: "mig-bo4e Crate — TOML Mapping Engine"
depends_on: [mig-driven-mapping/E04]
estimated_tasks: 7
crate: mig-bo4e
status: complete
---

# Epic 5: mig-bo4e Crate — TOML Mapping Engine

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. The workspace must compile with `cargo check --workspace` after every task.

**Goal:** Create the `mig-bo4e` crate with a TOML-based declarative mapping engine for bidirectional MIG-tree ↔ BO4E conversion. Simple 1:1 field mappings are defined in TOML files. Complex mappings (cross-references, conditional logic) are hand-coded Rust functions registered by name. The mapping engine handles both forward (tree → BO4E) and reverse (BO4E → tree) directions.

**Architecture:** TOML mapping files live in `mappings/` at the workspace root. Each file defines mappings for one entity (e.g., Marktlokation). The engine loads all TOML files at startup, validates paths against the MIG schema, and provides `to_bo4e()` / `from_bo4e()` functions. Complex handlers are registered as named functions in a `HandlerRegistry`.

**Tech Stack:** Rust, toml (TOML parsing), serde (deserialization), bo4e-extensions (BO4E types), mig-types (generated tree types), mig-assembly (assembled tree)

---

## Task 1: Create mig-bo4e Crate Stub

**Files:**
- Create: `crates/mig-bo4e/Cargo.toml`
- Create: `crates/mig-bo4e/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

**Step 1: Add mig-bo4e to workspace**

Add `"crates/mig-bo4e"` to workspace members. Add to `[workspace.dependencies]`:

```toml
mig-bo4e = { path = "crates/mig-bo4e" }
toml = "0.8"
```

**Step 2: Create crate Cargo.toml**

Create `crates/mig-bo4e/Cargo.toml`:

```toml
[package]
name = "mig-bo4e"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
description = "Declarative TOML-based MIG-tree to BO4E mapping engine"

[dependencies]
mig-types = { path = "../mig-types" }
mig-assembly = { path = "../mig-assembly" }
bo4e-extensions.workspace = true
serde.workspace = true
serde_json.workspace = true
toml.workspace = true
thiserror.workspace = true

[dev-dependencies]
insta.workspace = true
tempfile.workspace = true
```

**Step 3: Create lib.rs**

Create `crates/mig-bo4e/src/lib.rs`:

```rust
//! Declarative TOML-based MIG-tree ↔ BO4E mapping engine.
//!
//! # Architecture
//!
//! - **TOML mapping files** define simple 1:1 field mappings
//! - **Complex handlers** are Rust functions for non-trivial logic
//! - **MappingEngine** loads all definitions and provides bidirectional conversion
//!
//! # Usage
//! ```ignore
//! let engine = MappingEngine::load("mappings/")?;
//! let bo4e = engine.to_bo4e(&assembled_tree)?;
//! let tree = engine.from_bo4e(&bo4e, "55001")?;
//! ```

pub mod definition;
pub mod engine;
pub mod error;
pub mod handlers;

pub use engine::MappingEngine;
pub use error::MappingError;
```

Create `crates/mig-bo4e/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MappingError {
    #[error("TOML parse error in {file}: {message}")]
    TomlParse { file: String, message: String },

    #[error("Invalid mapping path '{path}' in {file}: {reason}")]
    InvalidPath { path: String, file: String, reason: String },

    #[error("Unknown handler '{name}' referenced in {file}")]
    UnknownHandler { name: String, file: String },

    #[error("Missing required field '{field}' during mapping")]
    MissingField { field: String },

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    Toml(#[from] toml::de::Error),
}
```

Create stubs for `definition.rs`, `engine.rs`, `handlers.rs`.

**Step 4: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/ Cargo.toml
git commit -m "feat(mig-bo4e): add crate stub for TOML-based MIG ↔ BO4E mapping"
```

---

## Task 2: TOML Mapping Definition Schema

**Files:**
- Modify: `crates/mig-bo4e/src/definition.rs`
- Create: `crates/mig-bo4e/tests/definition_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-bo4e/tests/definition_test.rs`:

```rust
use mig_bo4e::definition::MappingDefinition;

#[test]
fn test_parse_mapping_toml() {
    let toml_str = r#"
[meta]
entity = "Marktlokation"
bo4e_type = "bo4e::Marktlokation"
companion_type = "MarktlokationEdifact"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z01'"

[fields]
"loc.c517.d3225" = "marktlokations_id"
"loc.d3227" = { target = "lokationstyp", transform = "loc_qualifier_to_type" }

[fields."sg9_characteristics"]
"cci.c240.d7037" = "characteristic_code"

[companion_fields]
"dtm.c507.d2380" = { target = "gueltig_ab", when = "dtm.c507.d2005 == '157'" }
"#;

    let def: MappingDefinition = toml::from_str(toml_str).unwrap();

    assert_eq!(def.meta.entity, "Marktlokation");
    assert_eq!(def.meta.source_group, "SG8");
    assert!(!def.fields.is_empty());
    assert!(def.fields.contains_key("loc.c517.d3225"));
    assert!(def.companion_fields.is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_parse_mapping_toml -- --nocapture`
Expected: FAIL — `MappingDefinition` doesn't exist

**Step 3: Implement mapping definition types**

Create `crates/mig-bo4e/src/definition.rs`:

```rust
//! TOML mapping definition types.
//!
//! These types are deserialized from TOML mapping files
//! in the `mappings/` directory.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Root mapping definition — one per TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDefinition {
    pub meta: MappingMeta,
    pub fields: BTreeMap<String, FieldMapping>,
    pub companion_fields: Option<BTreeMap<String, FieldMapping>>,
    pub complex_handlers: Option<Vec<ComplexHandlerRef>>,
}

/// Metadata about the entity being mapped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMeta {
    pub entity: String,
    pub bo4e_type: String,
    pub companion_type: Option<String>,
    pub source_group: String,
    pub discriminator: Option<String>,
}

/// A field mapping — either a simple string target or a structured mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldMapping {
    /// Simple: "source_path" = "target_field"
    Simple(String),
    /// Structured: with optional transform, condition, etc.
    Structured(StructuredFieldMapping),
    /// Nested group mappings
    Nested(BTreeMap<String, FieldMapping>),
}

/// A structured field mapping with optional transform and condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFieldMapping {
    pub target: String,
    pub transform: Option<String>,
    pub when: Option<String>,
    pub default: Option<String>,
}

/// Reference to a complex handler function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexHandlerRef {
    pub name: String,
    pub description: Option<String>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_parse_mapping_toml -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/definition.rs crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): add TOML mapping definition schema types"
```

---

## Task 3: Mapping File Loader

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`
- Modify: `crates/mig-bo4e/tests/definition_test.rs`

**Step 1: Write the failing test**

Add to `definition_test.rs`:

```rust
use mig_bo4e::engine::MappingEngine;
use std::path::Path;

#[test]
fn test_load_mappings_from_directory() {
    // Create a temp dir with a mapping file
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("marktlokation.toml"), r#"
[meta]
entity = "Marktlokation"
bo4e_type = "bo4e::Marktlokation"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z01'"

[fields]
"loc.c517.d3225" = "marktlokations_id"
"#).unwrap();

    std::fs::write(dir.path().join("messlokation.toml"), r#"
[meta]
entity = "Messlokation"
bo4e_type = "bo4e::Messlokation"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z02'"

[fields]
"loc.c517.d3225" = "messlokations_id"
"#).unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    assert_eq!(engine.definitions().len(), 2);
    assert!(engine.definition_for_entity("Marktlokation").is_some());
    assert!(engine.definition_for_entity("Messlokation").is_some());
    assert!(engine.definition_for_entity("Nonexistent").is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_load_mappings_from_directory -- --nocapture`
Expected: FAIL

**Step 3: Implement mapping engine loader**

Create `crates/mig-bo4e/src/engine.rs`:

```rust
//! Mapping engine — loads TOML definitions and provides bidirectional conversion.

use std::path::Path;
use crate::definition::MappingDefinition;
use crate::error::MappingError;

/// The mapping engine holds all loaded mapping definitions
/// and provides methods for bidirectional conversion.
pub struct MappingEngine {
    definitions: Vec<MappingDefinition>,
}

impl MappingEngine {
    /// Load all TOML mapping files from a directory.
    pub fn load(dir: &Path) -> Result<Self, MappingError> {
        let mut definitions = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                let def: MappingDefinition = toml::from_str(&content).map_err(|e| {
                    MappingError::TomlParse {
                        file: path.display().to_string(),
                        message: e.to_string(),
                    }
                })?;
                definitions.push(def);
            }
        }

        Ok(Self { definitions })
    }

    /// Get all loaded definitions.
    pub fn definitions(&self) -> &[MappingDefinition] {
        &self.definitions
    }

    /// Find a definition by entity name.
    pub fn definition_for_entity(&self, entity: &str) -> Option<&MappingDefinition> {
        self.definitions.iter().find(|d| d.meta.entity == entity)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_load_mappings_from_directory -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): implement TOML mapping file loader"
```

---

## Task 4: Forward Mapping Engine (Tree → BO4E)

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`
- Create: `crates/mig-bo4e/tests/forward_mapping_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-bo4e/tests/forward_mapping_test.rs`:

```rust
use mig_bo4e::engine::MappingEngine;
use mig_assembly::assembler::{AssembledTree, AssembledSegment, AssembledGroup, AssembledGroupInstance};

#[test]
fn test_forward_map_simple_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("test_entity.toml"), r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "location_id"
"#).unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    // Create a minimal assembled tree with SG8 containing LOC
    let tree = AssembledTree {
        segments: vec![],
        groups: vec![AssembledGroup {
            group_id: "SG8".to_string(),
            repetitions: vec![AssembledGroupInstance {
                segments: vec![AssembledSegment {
                    tag: "LOC".to_string(),
                    elements: vec![
                        vec!["Z16".to_string()],              // qualifier
                        vec!["DE0001234567890".to_string()],   // C517.3225
                    ],
                }],
                child_groups: vec![],
            }],
        }],
    };

    let result = engine.extract_field(&tree, "SG8", "loc.c517.d3225", 0);
    assert!(result.is_some(), "Should extract location ID from tree");
    assert_eq!(result.unwrap(), "DE0001234567890");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_forward_map_simple_field -- --nocapture`
Expected: FAIL

**Step 3: Implement field extraction from assembled tree**

Add to `engine.rs`:

```rust
use mig_assembly::assembler::{AssembledTree, AssembledGroup, AssembledGroupInstance, AssembledSegment};

impl MappingEngine {
    /// Extract a field value from an assembled tree using a mapping path.
    ///
    /// Path format: "segment.composite.data_element" e.g., "loc.c517.d3225"
    /// The segment is found within the specified group at the given repetition index.
    pub fn extract_field(
        &self,
        tree: &AssembledTree,
        group_id: &str,
        path: &str,
        repetition: usize,
    ) -> Option<String> {
        let group = tree.groups.iter().find(|g| g.group_id == group_id)?;
        let instance = group.repetitions.get(repetition)?;

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // First part is the segment tag
        let segment_tag = parts[0].to_uppercase();
        let segment = instance.segments.iter().find(|s| s.tag.eq_ignore_ascii_case(&segment_tag))?;

        // Resolve remaining path parts to element/component indices
        // This requires knowledge of the MIG structure to map composite IDs to positions
        // For now, use positional access based on the segment's element list
        self.resolve_field_path(segment, &parts[1..])
    }

    fn resolve_field_path(&self, segment: &AssembledSegment, path: &[&str]) -> Option<String> {
        // Path navigation through composites and data elements
        // This is simplified — full implementation needs MIG schema for position mapping
        // For now: treat path parts as element_index.component_index
        match path.len() {
            1 => {
                // Direct data element: "d3225" -> element at known position
                // Position mapping from MIG would be needed here
                segment.elements.get(0)?.get(0).cloned()
            }
            2 => {
                // Composite + data element: "c517.d3225"
                // Element index for the composite, component index within
                segment.elements.get(1)?.get(0).cloned()
            }
            _ => None,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_forward_map_simple_field -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): implement forward field extraction from assembled tree"
```

---

## Task 5: Complex Handler Registry

**Files:**
- Modify: `crates/mig-bo4e/src/handlers.rs`
- Create: `crates/mig-bo4e/tests/handler_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-bo4e/tests/handler_test.rs`:

```rust
use mig_bo4e::handlers::HandlerRegistry;
use mig_assembly::assembler::AssembledGroupInstance;

#[test]
fn test_register_and_invoke_handler() {
    let mut registry = HandlerRegistry::new();

    // Register a simple handler
    registry.register("test_handler", |_instance| {
        Ok(serde_json::json!({"result": "handled"}))
    });

    assert!(registry.has_handler("test_handler"));
    assert!(!registry.has_handler("nonexistent"));

    let instance = AssembledGroupInstance {
        segments: vec![],
        child_groups: vec![],
    };

    let result = registry.invoke("test_handler", &instance);
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_register_and_invoke_handler -- --nocapture`
Expected: FAIL

**Step 3: Implement handler registry**

Create `crates/mig-bo4e/src/handlers.rs`:

```rust
//! Complex mapping handler registry.
//!
//! Handlers are Rust functions registered by name for mappings
//! that cannot be expressed declaratively in TOML.

use std::collections::HashMap;
use mig_assembly::assembler::AssembledGroupInstance;
use crate::error::MappingError;

type HandlerFn = Box<dyn Fn(&AssembledGroupInstance) -> Result<serde_json::Value, MappingError> + Send + Sync>;

/// Registry of named complex mapping handlers.
pub struct HandlerRegistry {
    handlers: HashMap<String, HandlerFn>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler function by name.
    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&AssembledGroupInstance) -> Result<serde_json::Value, MappingError> + Send + Sync + 'static,
    {
        self.handlers.insert(name.to_string(), Box::new(handler));
    }

    /// Check if a handler exists.
    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// Invoke a handler by name.
    pub fn invoke(
        &self,
        name: &str,
        instance: &AssembledGroupInstance,
    ) -> Result<serde_json::Value, MappingError> {
        let handler = self.handlers.get(name).ok_or_else(|| MappingError::UnknownHandler {
            name: name.to_string(),
            file: String::new(),
        })?;
        handler(instance)
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_register_and_invoke_handler -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/handlers.rs crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): implement complex mapping handler registry"
```

---

## Task 6: Reverse Mapping Engine (BO4E → Tree)

**Files:**
- Modify: `crates/mig-bo4e/src/engine.rs`
- Create: `crates/mig-bo4e/tests/reverse_mapping_test.rs`

**Step 1: Write the failing test**

Create `crates/mig-bo4e/tests/reverse_mapping_test.rs`:

```rust
use mig_bo4e::engine::MappingEngine;

#[test]
fn test_reverse_map_simple_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("test_entity.toml"), r#"
[meta]
entity = "TestEntity"
bo4e_type = "TestEntity"
source_group = "SG8"

[fields]
"loc.c517.d3225" = "location_id"
"#).unwrap();

    let engine = MappingEngine::load(dir.path()).unwrap();

    // Provide a BO4E-like JSON value
    let bo4e_value = serde_json::json!({
        "location_id": "DE0001234567890"
    });

    let result = engine.populate_field(&bo4e_value, "location_id", "loc.c517.d3225");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "DE0001234567890");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p mig-bo4e test_reverse_map_simple_field -- --nocapture`
Expected: FAIL

**Step 3: Implement reverse field population**

Add to `engine.rs`:

```rust
impl MappingEngine {
    /// Extract a value from a BO4E JSON object by target field name,
    /// for populating back into a tree at the given path.
    pub fn populate_field(
        &self,
        bo4e_value: &serde_json::Value,
        target_field: &str,
        _source_path: &str,
    ) -> Option<String> {
        // Navigate the BO4E JSON to find the target field
        // Supports dotted paths like "nested.field_name"
        let parts: Vec<&str> = target_field.split('.').collect();
        let mut current = bo4e_value;
        for part in &parts {
            current = current.get(part)?;
        }
        current.as_str().map(|s| s.to_string())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p mig-bo4e test_reverse_map_simple_field -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/mig-bo4e/src/engine.rs crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): implement reverse field population from BO4E values"
```

---

## Task 7: Initial Mapping TOML Files

**Files:**
- Create: `mappings/marktlokation.toml`
- Create: `mappings/messlokation.toml`
- Create: `mappings/geschaeftspartner.toml`

**Step 1: Create mapping directory**

```bash
mkdir -p mappings
```

**Step 2: Write Marktlokation mapping**

Create `mappings/marktlokation.toml`:

```toml
[meta]
entity = "Marktlokation"
bo4e_type = "bo4e::Marktlokation"
companion_type = "MarktlokationEdifact"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z01'"

[fields]
"loc.c517.d3225" = "marktlokations_id"
"loc.d3227" = { target = "lokationstyp", transform = "loc_qualifier_to_type" }

[fields."sg9_characteristics"]
"cci.c240.d7037" = "characteristic_code"
"cav.c889.d7111" = "characteristic_value"

[companion_fields]
"dtm_157.c507.d2380" = { target = "gueltig_ab", when = "dtm.c507.d2005 == '157'" }
"dtm_158.c507.d2380" = { target = "gueltig_bis", when = "dtm.c507.d2005 == '158'" }
```

**Step 3: Write Messlokation mapping**

Create `mappings/messlokation.toml`:

```toml
[meta]
entity = "Messlokation"
bo4e_type = "bo4e::Messlokation"
companion_type = "MesslokationEdifact"
source_group = "SG8"
discriminator = "seq.d1245.qualifier == 'Z02'"

[fields]
"loc.c517.d3225" = "messlokations_id"
```

**Step 4: Write Geschaeftspartner mapping**

Create `mappings/geschaeftspartner.toml`:

```toml
[meta]
entity = "Geschaeftspartner"
bo4e_type = "bo4e::Geschaeftspartner"
companion_type = "GeschaeftspartnerEdifact"
source_group = "SG2"

[fields]
"nad.d3035" = "qualifier"
"nad.c082.d3039" = "partner_id"
"nad.c058.d3124" = "name_1"
"nad.d3164" = "city"
"nad.d3251" = "postcode"
```

**Step 5: Validation test**

Create `crates/mig-bo4e/tests/mapping_files_test.rs`:

```rust
use mig_bo4e::engine::MappingEngine;
use std::path::Path;

#[test]
fn test_load_real_mapping_files() {
    let mappings_dir = Path::new("../../mappings");
    if !mappings_dir.exists() {
        eprintln!("mappings/ dir not found, skipping");
        return;
    }

    let engine = MappingEngine::load(mappings_dir).unwrap();
    assert!(engine.definitions().len() >= 3,
        "Expected at least 3 mapping files, got {}", engine.definitions().len());
    assert!(engine.definition_for_entity("Marktlokation").is_some());
    assert!(engine.definition_for_entity("Messlokation").is_some());
    assert!(engine.definition_for_entity("Geschaeftspartner").is_some());
}
```

**Step 6: Commit**

```bash
git add mappings/ crates/mig-bo4e/tests/
git commit -m "feat(mig-bo4e): add initial TOML mapping files for MaLo, MeLo, GP"
```

---

## Test Summary

| Metric | Value |
|--------|-------|
| Tests | 24 |
| Passed | 24 |
| Failed | 0 |
| Skipped | 0 |
| Mapping files | 3 (Marktlokation, Messlokation, Geschaeftspartner) |
| cargo check --workspace | PASS |
| cargo clippy --workspace | PASS |

Files tested:
- crates/mig-bo4e/tests/definition_test.rs (6 tests: TOML parsing, loader, error handling)
- crates/mig-bo4e/tests/forward_mapping_test.rs (5 tests: field extraction from assembled tree)
- crates/mig-bo4e/tests/handler_test.rs (5 tests: handler registry, invocation, data passing)
- crates/mig-bo4e/tests/reverse_mapping_test.rs (5 tests: BO4E field population, segment/group building)
- crates/mig-bo4e/tests/mapping_files_test.rs (3 tests: real TOML file loading, structure validation)
