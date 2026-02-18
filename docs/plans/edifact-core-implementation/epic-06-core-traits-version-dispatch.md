---
feature: edifact-core-implementation
epic: 6
title: "automapper-core Traits & Version Dispatch"
depends_on: [edifact-core-implementation/E04, edifact-core-implementation/E05]
estimated_tasks: 6
crate: automapper-core
---

# Epic 6: automapper-core Traits & Version Dispatch

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-core/src/`. All code must compile with `cargo check -p automapper-core`.

**Goal:** Implement the core trait hierarchy for `automapper-core`: `SegmentHandler`, `Builder<T>`, `EntityWriter`, `Mapper` traits, `FormatVersion` enum with `FV2504`/`FV2510` marker types, `VersionConfig` trait with associated types, `TransactionContext` struct, `Coordinator` trait, `create_coordinator()` runtime entry point, and `AutomapperError` error type with `thiserror`.

**Architecture:** The trait hierarchy mirrors the C# `ISegmentHandler`, `IBuilder<T>`, `IEntityWriter`, and `IMapper` interfaces. Format version dispatch uses a hybrid approach: trait-based generics for zero-cost compile-time dispatch on the hot path, with an enum boundary at the runtime entry point (`create_coordinator()`). The `VersionConfig` trait binds associated mapper types per format version. `TransactionContext` holds cross-cutting state shared across mappers during parsing. See design doc section 5.

**Tech Stack:** Rust, edifact-types, edifact-parser, bo4e-extensions, thiserror

---

## Task 1: AutomapperError Error Type

**Files:**
- Create: `crates/automapper-core/src/error.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/error.rs`:

```rust
//! Error types for the automapper-core crate.

use edifact_parser::ParseError;

/// Errors that can occur during automapping operations.
#[derive(Debug, thiserror::Error)]
pub enum AutomapperError {
    /// A parse error occurred in the EDIFACT parser.
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// An unknown format version was specified.
    #[error("unknown format version: {0}")]
    UnknownFormatVersion(String),

    /// A mapping error occurred while processing a segment.
    #[error("mapping error in {segment} at position {position}: {message}")]
    Mapping {
        segment: String,
        position: u32,
        message: String,
    },

    /// A roundtrip mismatch was detected.
    #[error("roundtrip mismatch: {message}")]
    RoundtripMismatch { message: String },

    /// A required field was missing during building.
    #[error("missing required field '{field}' in {entity}")]
    MissingField { entity: String, field: String },

    /// A writer error occurred during EDIFACT generation.
    #[error("writer error: {message}")]
    WriterError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automapper_error_display_unknown_version() {
        let err = AutomapperError::UnknownFormatVersion("FV9999".to_string());
        assert_eq!(err.to_string(), "unknown format version: FV9999");
    }

    #[test]
    fn test_automapper_error_display_mapping() {
        let err = AutomapperError::Mapping {
            segment: "LOC".to_string(),
            position: 42,
            message: "invalid qualifier".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "mapping error in LOC at position 42: invalid qualifier"
        );
    }

    #[test]
    fn test_automapper_error_display_roundtrip() {
        let err = AutomapperError::RoundtripMismatch {
            message: "segment count differs".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "roundtrip mismatch: segment count differs"
        );
    }

    #[test]
    fn test_automapper_error_display_missing_field() {
        let err = AutomapperError::MissingField {
            entity: "Marktlokation".to_string(),
            field: "marktlokations_id".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "missing required field 'marktlokations_id' in Marktlokation"
        );
    }

    #[test]
    fn test_automapper_error_display_writer() {
        let err = AutomapperError::WriterError {
            message: "buffer full".to_string(),
        };
        assert_eq!(err.to_string(), "writer error: buffer full");
    }

    #[test]
    fn test_automapper_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AutomapperError>();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_automapper_error`
Expected: FAIL -- module not found.

**Step 3: Write minimal implementation**

Update `crates/automapper-core/src/lib.rs`:

```rust
//! Core automapper library.
//!
//! Coordinators, mappers, builders, and writers for bidirectional
//! EDIFACT <-> BO4E conversion. Supports streaming parsing with
//! format-version-dispatched mappers and parallel batch processing.

pub mod error;

pub use error::AutomapperError;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add AutomapperError with thiserror

Typed error variants for parse errors, unknown format versions,
mapping errors, roundtrip mismatches, missing fields, and writer errors.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: TransactionContext Struct

**Files:**
- Create: `crates/automapper-core/src/context.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/context.rs`:

```rust
//! Transaction context shared across mappers during parsing.

use std::any::Any;
use std::collections::HashMap;

/// Holds cross-cutting transaction-level state during EDIFACT parsing.
///
/// Mappers use the context to share data across segment boundaries.
/// For example, the message reference from UNH is stored here so that
/// all mappers can access it.
///
/// Mirrors the C# `ITransactionContext` / `TransactionContext`.
#[derive(Debug)]
pub struct TransactionContext {
    /// The format version being processed (e.g., "FV2504").
    pub format_version: String,

    /// The message reference number from UNH segment.
    pub message_reference: Option<String>,

    /// The Pruefidentifikator for this transaction.
    pub pruefidentifikator: Option<String>,

    /// The sender MP-ID from the message header.
    pub sender_mp_id: Option<String>,

    /// The recipient MP-ID from the message header.
    pub recipient_mp_id: Option<String>,

    /// The current transaction ID from IDE segment.
    pub transaction_id: Option<String>,

    /// The current Zeitscheibe reference being processed.
    pub current_zeitscheibe_ref: Option<String>,

    /// Registered objects keyed by type name and ID.
    objects: HashMap<String, Box<dyn Any + Send>>,
}

impl TransactionContext {
    /// Creates a new context for the given format version.
    pub fn new(format_version: impl Into<String>) -> Self {
        Self {
            format_version: format_version.into(),
            message_reference: None,
            pruefidentifikator: None,
            sender_mp_id: None,
            recipient_mp_id: None,
            transaction_id: None,
            current_zeitscheibe_ref: None,
            objects: HashMap::new(),
        }
    }

    /// Sets the message reference from UNH.
    pub fn set_message_reference(&mut self, reference: impl Into<String>) {
        self.message_reference = Some(reference.into());
    }

    /// Sets the Pruefidentifikator.
    pub fn set_pruefidentifikator(&mut self, pi: impl Into<String>) {
        self.pruefidentifikator = Some(pi.into());
    }

    /// Sets the sender MP-ID.
    pub fn set_sender_mp_id(&mut self, id: impl Into<String>) {
        self.sender_mp_id = Some(id.into());
    }

    /// Sets the recipient MP-ID.
    pub fn set_recipient_mp_id(&mut self, id: impl Into<String>) {
        self.recipient_mp_id = Some(id.into());
    }

    /// Registers an object for later retrieval.
    pub fn register_object<T: Any + Send>(&mut self, key: impl Into<String>, obj: T) {
        self.objects.insert(key.into(), Box::new(obj));
    }

    /// Gets a registered object by key.
    pub fn get_object<T: Any + Send>(&self, key: &str) -> Option<&T> {
        self.objects.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Resets the context for a new message, clearing all transient state.
    pub fn reset(&mut self) {
        self.message_reference = None;
        self.pruefidentifikator = None;
        self.transaction_id = None;
        self.current_zeitscheibe_ref = None;
        self.objects.clear();
        // Note: format_version, sender_mp_id, recipient_mp_id persist across messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_context_new() {
        let ctx = TransactionContext::new("FV2504");
        assert_eq!(ctx.format_version, "FV2504");
        assert!(ctx.message_reference.is_none());
        assert!(ctx.pruefidentifikator.is_none());
        assert!(ctx.sender_mp_id.is_none());
        assert!(ctx.recipient_mp_id.is_none());
    }

    #[test]
    fn test_transaction_context_set_fields() {
        let mut ctx = TransactionContext::new("FV2510");
        ctx.set_message_reference("MSG001");
        ctx.set_pruefidentifikator("11042");
        ctx.set_sender_mp_id("9900123000002");
        ctx.set_recipient_mp_id("9900456000001");

        assert_eq!(ctx.message_reference, Some("MSG001".to_string()));
        assert_eq!(ctx.pruefidentifikator, Some("11042".to_string()));
        assert_eq!(ctx.sender_mp_id, Some("9900123000002".to_string()));
        assert_eq!(ctx.recipient_mp_id, Some("9900456000001".to_string()));
    }

    #[test]
    fn test_transaction_context_register_and_get_object() {
        let mut ctx = TransactionContext::new("FV2504");
        ctx.register_object("test_string", "hello".to_string());

        let retrieved = ctx.get_object::<String>("test_string");
        assert_eq!(retrieved, Some(&"hello".to_string()));

        // Wrong type returns None
        let wrong_type = ctx.get_object::<u32>("test_string");
        assert!(wrong_type.is_none());

        // Missing key returns None
        let missing = ctx.get_object::<String>("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_transaction_context_reset() {
        let mut ctx = TransactionContext::new("FV2504");
        ctx.set_message_reference("MSG001");
        ctx.set_pruefidentifikator("11042");
        ctx.set_sender_mp_id("9900123000002");
        ctx.register_object("key", 42u32);

        ctx.reset();

        // Transient state is cleared
        assert!(ctx.message_reference.is_none());
        assert!(ctx.pruefidentifikator.is_none());
        assert!(ctx.get_object::<u32>("key").is_none());

        // Persistent state is preserved
        assert_eq!(ctx.format_version, "FV2504");
        assert_eq!(ctx.sender_mp_id, Some("9900123000002".to_string()));
    }

    #[test]
    fn test_transaction_context_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TransactionContext>();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_transaction_context`
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
pub mod error;

pub use context::TransactionContext;
pub use error::AutomapperError;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add TransactionContext for cross-mapper state

Holds message reference, Pruefidentifikator, sender/recipient IDs,
and a typed object registry. Resets transient state between messages
while preserving format version and party IDs.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: SegmentHandler and Builder Traits

**Files:**
- Create: `crates/automapper-core/src/traits.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/traits.rs`:

```rust
//! Core trait definitions for the automapper pipeline.
//!
//! These traits mirror the C# `ISegmentHandler`, `IBuilder<T>`, `IEntityWriter`,
//! and `IMapper` interfaces. They define the contract for bidirectional
//! EDIFACT <-> BO4E mapping.

use edifact_types::RawSegment;

use crate::context::TransactionContext;

/// Reads EDIFACT segments into domain objects.
///
/// Implementations handle specific segment types (e.g., LOC+Z16 for Marktlokation).
/// The coordinator routes segments to handlers via `can_handle()` checks.
///
/// Mirrors C# `ISegmentHandler`.
pub trait SegmentHandler: Send {
    /// Determines whether this handler can process the given segment.
    ///
    /// Typically checks the segment ID and qualifier (e.g., `segment.is("LOC")`
    /// and the first element equals "Z16").
    fn can_handle(&self, segment: &RawSegment) -> bool;

    /// Processes the segment, accumulating state for later building.
    ///
    /// Called by the coordinator for every segment where `can_handle()` returned true.
    fn handle(&mut self, segment: &RawSegment, ctx: &mut TransactionContext);
}

/// Accumulates state across multiple segments and builds a domain object.
///
/// Builders are used by mappers to collect data from multiple segments
/// and produce a single business object once all relevant segments are processed.
///
/// Mirrors C# `IBuilder<T>` / `BusinessObjectBuilder<T>`.
pub trait Builder<T>: Send {
    /// Returns true if no data has been accumulated.
    fn is_empty(&self) -> bool;

    /// Consumes the accumulated state and produces the domain object.
    ///
    /// After calling `build()`, the builder should be considered consumed.
    /// Call `reset()` to reuse the builder for another object.
    fn build(&mut self) -> T;

    /// Resets the builder to its initial empty state for reuse.
    fn reset(&mut self);
}

/// Serializes domain objects back to EDIFACT segments.
///
/// Entity writers are the reverse of segment handlers: they take a domain
/// object and produce EDIFACT segment data. Used for the generation
/// (BO4E -> EDIFACT) direction.
///
/// Mirrors C# `IEntityWriter<TEntity>`.
pub trait EntityWriter: Send {
    /// Writes the entity as EDIFACT segments.
    ///
    /// The writer appends segments to the provided segment buffer. The
    /// `TransactionContext` provides shared state needed during writing
    /// (e.g., Zeitscheibe references, format version).
    fn write(&self, segments: &mut Vec<Vec<Vec<String>>>, ctx: &TransactionContext);
}

/// Bidirectional mapper combining reading + writing for one entity type.
///
/// A `Mapper` implements both `SegmentHandler` (for parsing) and `EntityWriter`
/// (for generation). This ensures that every entity that can be parsed can
/// also be written back, supporting roundtrip fidelity.
///
/// Mirrors C# `IMapper`.
pub trait Mapper: SegmentHandler + EntityWriter {
    /// Returns the format version this mapper targets.
    fn format_version(&self) -> FormatVersion;
}

/// EDIFACT format version identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatVersion {
    /// Format version April 2025.
    FV2504,
    /// Format version October 2025.
    FV2510,
}

impl FormatVersion {
    /// Returns the string representation (e.g., "FV2504").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FV2504 => "FV2504",
            Self::FV2510 => "FV2510",
        }
    }

    /// Parses a format version from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FV2504" => Some(Self::FV2504),
            "FV2510" => Some(Self::FV2510),
            _ => None,
        }
    }
}

impl std::fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edifact_types::SegmentPosition;

    // --- Test helpers ---

    struct TestHandler {
        handled_ids: Vec<String>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                handled_ids: Vec::new(),
            }
        }
    }

    impl SegmentHandler for TestHandler {
        fn can_handle(&self, segment: &RawSegment) -> bool {
            segment.id == "LOC"
        }

        fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
            self.handled_ids.push(segment.id.to_string());
        }
    }

    struct TestBuilder {
        value: Option<String>,
    }

    impl TestBuilder {
        fn new() -> Self {
            Self { value: None }
        }
    }

    impl Builder<String> for TestBuilder {
        fn is_empty(&self) -> bool {
            self.value.is_none()
        }

        fn build(&mut self) -> String {
            self.value.take().unwrap_or_default()
        }

        fn reset(&mut self) {
            self.value = None;
        }
    }

    // --- Tests ---

    #[test]
    fn test_segment_handler_can_handle() {
        let handler = TestHandler::new();
        let pos = SegmentPosition::new(1, 0, 1);
        let loc = RawSegment::new("LOC", vec![], pos);
        let bgm = RawSegment::new("BGM", vec![], pos);

        assert!(handler.can_handle(&loc));
        assert!(!handler.can_handle(&bgm));
    }

    #[test]
    fn test_segment_handler_handle() {
        let mut handler = TestHandler::new();
        let mut ctx = TransactionContext::new("FV2504");
        let pos = SegmentPosition::new(1, 0, 1);
        let seg = RawSegment::new("LOC", vec![], pos);

        handler.handle(&seg, &mut ctx);
        assert_eq!(handler.handled_ids, vec!["LOC"]);
    }

    #[test]
    fn test_builder_lifecycle() {
        let mut builder = TestBuilder::new();
        assert!(builder.is_empty());

        builder.value = Some("test".to_string());
        assert!(!builder.is_empty());

        let result = builder.build();
        assert_eq!(result, "test");

        // After build, value is consumed
        assert!(builder.is_empty());
    }

    #[test]
    fn test_builder_reset() {
        let mut builder = TestBuilder::new();
        builder.value = Some("data".to_string());

        builder.reset();
        assert!(builder.is_empty());
        assert_eq!(builder.build(), "");
    }

    #[test]
    fn test_format_version_as_str() {
        assert_eq!(FormatVersion::FV2504.as_str(), "FV2504");
        assert_eq!(FormatVersion::FV2510.as_str(), "FV2510");
    }

    #[test]
    fn test_format_version_from_str() {
        assert_eq!(FormatVersion::from_str("FV2504"), Some(FormatVersion::FV2504));
        assert_eq!(FormatVersion::from_str("FV2510"), Some(FormatVersion::FV2510));
        assert_eq!(FormatVersion::from_str("FV9999"), None);
        assert_eq!(FormatVersion::from_str(""), None);
    }

    #[test]
    fn test_format_version_display() {
        assert_eq!(format!("{}", FormatVersion::FV2504), "FV2504");
        assert_eq!(format!("{}", FormatVersion::FV2510), "FV2510");
    }

    #[test]
    fn test_format_version_equality() {
        assert_eq!(FormatVersion::FV2504, FormatVersion::FV2504);
        assert_ne!(FormatVersion::FV2504, FormatVersion::FV2510);
    }

    #[test]
    fn test_segment_handler_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestHandler>();
    }

    #[test]
    fn test_builder_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestBuilder>();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_segment_handler`
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
pub mod error;
pub mod traits;

pub use context::TransactionContext;
pub use error::AutomapperError;
pub use traits::*;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add SegmentHandler, Builder, EntityWriter, Mapper traits

Core trait hierarchy for bidirectional EDIFACT mapping.
FormatVersion enum with FV2504/FV2510 variants and string conversion.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: VersionConfig Trait and Marker Types

**Files:**
- Create: `crates/automapper-core/src/version.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/version.rs`:

```rust
//! Format version dispatch via marker types and the VersionConfig trait.
//!
//! The hybrid approach: traits for zero-cost compile-time dispatch on the
//! hot path, enum boundary at the runtime entry point.
//!
//! See design doc section 5 (Format Version Dispatch).

use std::marker::PhantomData;

use crate::traits::FormatVersion;

/// Marker type for format version April 2025.
pub struct FV2504;

/// Marker type for format version October 2025.
pub struct FV2510;

/// Associates mapper types with a format version at compile time.
///
/// Each format version implements this trait with its own set of mapper types.
/// This provides zero-cost dispatch for the hot path (no dynamic dispatch).
/// The `create_coordinator()` function uses the `FormatVersion` enum to select
/// the right `VersionConfig` at runtime.
///
/// # Example
///
/// ```ignore
/// impl VersionConfig for FV2504 {
///     const VERSION: FormatVersion = FormatVersion::FV2504;
///     type MarktlokationMapper = marktlokation::MapperV2504;
///     // ... one associated type per entity mapper
/// }
/// ```
pub trait VersionConfig: Send + 'static {
    /// The format version this config represents.
    const VERSION: FormatVersion;

    // Associated mapper types will be added as mappers are implemented.
    // For now we define the trait structure without requiring concrete types.
}

impl VersionConfig for FV2504 {
    const VERSION: FormatVersion = FormatVersion::FV2504;
}

impl VersionConfig for FV2510 {
    const VERSION: FormatVersion = FormatVersion::FV2510;
}

/// Helper struct that carries a version config type parameter.
///
/// Used by `UtilmdCoordinator<V>` to associate with a specific version
/// without storing version-specific data.
#[derive(Debug)]
pub struct VersionPhantom<V: VersionConfig> {
    _marker: PhantomData<V>,
}

impl<V: VersionConfig> VersionPhantom<V> {
    /// Creates a new VersionPhantom.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns the format version.
    pub fn version(&self) -> FormatVersion {
        V::VERSION
    }
}

impl<V: VersionConfig> Default for VersionPhantom<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fv2504_version_config() {
        assert_eq!(FV2504::VERSION, FormatVersion::FV2504);
    }

    #[test]
    fn test_fv2510_version_config() {
        assert_eq!(FV2510::VERSION, FormatVersion::FV2510);
    }

    #[test]
    fn test_version_phantom_fv2504() {
        let phantom = VersionPhantom::<FV2504>::new();
        assert_eq!(phantom.version(), FormatVersion::FV2504);
    }

    #[test]
    fn test_version_phantom_fv2510() {
        let phantom = VersionPhantom::<FV2510>::default();
        assert_eq!(phantom.version(), FormatVersion::FV2510);
    }

    #[test]
    fn test_version_config_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<FV2504>();
        assert_send::<FV2510>();
    }

    #[test]
    fn test_version_config_static_lifetime() {
        fn assert_static<T: 'static>() {}
        assert_static::<FV2504>();
        assert_static::<FV2510>();
    }

    /// Verify that generics parameterized by VersionConfig work correctly.
    #[test]
    fn test_generic_over_version_config() {
        fn get_version_str<V: VersionConfig>() -> &'static str {
            V::VERSION.as_str()
        }

        assert_eq!(get_version_str::<FV2504>(), "FV2504");
        assert_eq!(get_version_str::<FV2510>(), "FV2510");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_fv2504`
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
pub mod error;
pub mod traits;
pub mod version;

pub use context::TransactionContext;
pub use error::AutomapperError;
pub use traits::*;
pub use version::{FV2504, FV2510, VersionConfig, VersionPhantom};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add VersionConfig trait with FV2504/FV2510 markers

Hybrid dispatch: compile-time VersionConfig trait with marker types
for zero-cost generics, FormatVersion enum at the runtime boundary.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Coordinator Trait and create_coordinator Entry Point

**Files:**
- Create: `crates/automapper-core/src/coordinator.rs`
- Modify: `crates/automapper-core/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/automapper-core/src/coordinator.rs`:

```rust
//! Coordinator trait and runtime entry point.
//!
//! The coordinator orchestrates mappers during EDIFACT processing. It implements
//! `EdifactHandler` from the parser crate and routes segments to registered
//! mappers. The `create_coordinator()` function is the runtime entry point
//! that selects the correct version-parameterized coordinator.
//!
//! See design doc section 5 (Coordinator).

use bo4e_extensions::UtilmdTransaktion;
use edifact_parser::EdifactHandler;

use crate::error::AutomapperError;
use crate::traits::FormatVersion;

/// Orchestrates mappers during EDIFACT processing.
///
/// A coordinator implements `EdifactHandler` (from the parser crate) and
/// exposes `parse()` and `generate()` for bidirectional conversion.
///
/// Mirrors C# `CoordinatorBase`.
pub trait Coordinator: EdifactHandler + Send {
    /// Parses an EDIFACT interchange and returns the extracted transactions.
    ///
    /// This is the main forward-mapping entry point. It feeds the input through
    /// the streaming parser with `self` as the handler, then collects all
    /// completed transactions.
    fn parse(&mut self, input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError>;

    /// Generates EDIFACT bytes from a transaction.
    ///
    /// This is the main reverse-mapping entry point. It takes a transaction
    /// and serializes it back to EDIFACT format.
    fn generate(&self, transaktion: &UtilmdTransaktion) -> Result<Vec<u8>, AutomapperError>;

    /// Returns the format version this coordinator handles.
    fn format_version(&self) -> FormatVersion;
}

/// Creates a coordinator for the specified format version.
///
/// This is the **runtime entry point** -- the enum boundary where dynamic
/// dispatch begins. Internally, the returned `Box<dyn Coordinator>` contains
/// a `UtilmdCoordinator<FV2504>` or `UtilmdCoordinator<FV2510>` with
/// compile-time dispatched mappers.
///
/// # Example
///
/// ```ignore
/// let fv = FormatVersion::FV2504;
/// let mut coord = create_coordinator(fv);
/// let transactions = coord.parse(edifact_bytes)?;
/// ```
pub fn create_coordinator(fv: FormatVersion) -> Result<Box<dyn Coordinator>, AutomapperError> {
    match fv {
        FormatVersion::FV2504 => Ok(Box::new(StubCoordinator::new(FormatVersion::FV2504))),
        FormatVersion::FV2510 => Ok(Box::new(StubCoordinator::new(FormatVersion::FV2510))),
    }
}

/// Detects the format version from EDIFACT input.
///
/// Scans for UNH segment and extracts the message version from the
/// message identifier composite (element 1, components 0-4).
/// Returns `None` if the format version cannot be determined.
pub fn detect_format_version(input: &[u8]) -> Option<FormatVersion> {
    let input_str = std::str::from_utf8(input).ok()?;

    // Look for UNH segment to find message type identifier
    // UNH+ref+UTILMD:D:11A:UN:S2.1' -- the S2.1 suffix indicates version
    // For now, we use a simple heuristic based on the message version
    if input_str.contains("S2.1") || input_str.contains("FV2504") {
        Some(FormatVersion::FV2504)
    } else if input_str.contains("S2.2") || input_str.contains("FV2510") {
        Some(FormatVersion::FV2510)
    } else {
        // Default to FV2504 if we can detect it's a UTILMD message
        if input_str.contains("UTILMD") {
            Some(FormatVersion::FV2504)
        } else {
            None
        }
    }
}

/// Stub coordinator used until real UtilmdCoordinator is implemented in Epic 7.
///
/// This allows the trait, create_coordinator(), and tests to work while
/// the full mapper infrastructure is being built.
struct StubCoordinator {
    fv: FormatVersion,
}

impl StubCoordinator {
    fn new(fv: FormatVersion) -> Self {
        Self { fv }
    }
}

impl EdifactHandler for StubCoordinator {}

impl Coordinator for StubCoordinator {
    fn parse(&mut self, _input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError> {
        Ok(Vec::new())
    }

    fn generate(&self, _transaktion: &UtilmdTransaktion) -> Result<Vec<u8>, AutomapperError> {
        Ok(Vec::new())
    }

    fn format_version(&self) -> FormatVersion {
        self.fv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_coordinator_fv2504() {
        let coord = create_coordinator(FormatVersion::FV2504).unwrap();
        assert_eq!(coord.format_version(), FormatVersion::FV2504);
    }

    #[test]
    fn test_create_coordinator_fv2510() {
        let coord = create_coordinator(FormatVersion::FV2510).unwrap();
        assert_eq!(coord.format_version(), FormatVersion::FV2510);
    }

    #[test]
    fn test_stub_coordinator_parse_returns_empty() {
        let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
        let result = coord.parse(b"").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_stub_coordinator_generate_returns_empty() {
        let coord = create_coordinator(FormatVersion::FV2504).unwrap();
        let tx = UtilmdTransaktion::default();
        let result = coord.generate(&tx).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_detect_format_version_fv2504() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN:S2.1'";
        assert_eq!(detect_format_version(input), Some(FormatVersion::FV2504));
    }

    #[test]
    fn test_detect_format_version_fv2510() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN:S2.2'";
        assert_eq!(detect_format_version(input), Some(FormatVersion::FV2510));
    }

    #[test]
    fn test_detect_format_version_utilmd_default() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN'";
        assert_eq!(detect_format_version(input), Some(FormatVersion::FV2504));
    }

    #[test]
    fn test_detect_format_version_unknown() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+APERAK:D:11A:UN'";
        assert_eq!(detect_format_version(input), None);
    }

    #[test]
    fn test_coordinator_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Box<dyn Coordinator>>();
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p automapper-core test_create_coordinator`
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
pub mod traits;
pub mod version;

pub use context::TransactionContext;
pub use coordinator::{create_coordinator, detect_format_version, Coordinator};
pub use error::AutomapperError;
pub use traits::*;
pub use version::{FV2504, FV2510, VersionConfig, VersionPhantom};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p automapper-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/automapper-core/
git commit -m "$(cat <<'EOF'
feat(automapper-core): add Coordinator trait and create_coordinator()

Runtime entry point that creates version-dispatched coordinators.
Includes detect_format_version() for automatic version detection from
UNH segment data. Stub coordinator for testing until Epic 7.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Integration Test for Trait Hierarchy and Version Dispatch

**Files:**
- Create: `crates/automapper-core/tests/trait_dispatch_test.rs`

**Step 1: Write the integration test**

Create `crates/automapper-core/tests/trait_dispatch_test.rs`:

```rust
//! Integration tests for the trait hierarchy and version dispatch mechanism.

use automapper_core::{
    create_coordinator, detect_format_version, AutomapperError, Builder, Coordinator,
    FormatVersion, SegmentHandler, TransactionContext, VersionConfig, FV2504, FV2510,
};
use edifact_types::{RawSegment, SegmentPosition};

/// Verify that VersionConfig is correctly parameterized.
#[test]
fn test_version_config_dispatch() {
    fn version_string<V: VersionConfig>() -> &'static str {
        V::VERSION.as_str()
    }

    assert_eq!(version_string::<FV2504>(), "FV2504");
    assert_eq!(version_string::<FV2510>(), "FV2510");
}

/// Verify the full pipeline: detect version -> create coordinator -> parse.
#[test]
fn test_full_pipeline_stub() {
    let input = b"UNA:+.? 'UNB+UNOC:3+9900123:500+9900456:500+251217:1229+REF'UNH+MSG+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'UNT+3+MSG'UNZ+1+REF'";

    // Step 1: Detect version
    let fv = detect_format_version(input).expect("should detect FV2504");
    assert_eq!(fv, FormatVersion::FV2504);

    // Step 2: Create coordinator
    let mut coord = create_coordinator(fv).expect("should create coordinator");
    assert_eq!(coord.format_version(), FormatVersion::FV2504);

    // Step 3: Parse (stub returns empty for now)
    let result = coord.parse(input).expect("should parse without error");
    assert!(result.is_empty(), "stub coordinator returns empty");
}

/// Verify that FormatVersion round-trips through string conversion.
#[test]
fn test_format_version_string_roundtrip() {
    for fv in [FormatVersion::FV2504, FormatVersion::FV2510] {
        let s = fv.as_str();
        let parsed = FormatVersion::from_str(s).expect("should parse back");
        assert_eq!(fv, parsed);
    }
}

/// Verify TransactionContext reset behavior preserves format version.
#[test]
fn test_context_reset_preserves_format_version() {
    let mut ctx = TransactionContext::new("FV2504");
    ctx.set_message_reference("MSG001");
    ctx.set_sender_mp_id("9900123");

    ctx.reset();

    assert_eq!(ctx.format_version, "FV2504");
    assert_eq!(ctx.sender_mp_id, Some("9900123".to_string()));
    assert!(ctx.message_reference.is_none());
}

/// Verify that a custom SegmentHandler + Builder work together.
#[test]
fn test_handler_and_builder_integration() {
    struct LocHandler {
        location_ids: Vec<String>,
    }

    impl SegmentHandler for LocHandler {
        fn can_handle(&self, segment: &RawSegment) -> bool {
            segment.id == "LOC"
        }

        fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
            let qualifier = segment.get_element(0);
            if qualifier == "Z16" {
                let id = segment.get_component(1, 0);
                if !id.is_empty() {
                    self.location_ids.push(id.to_string());
                }
            }
        }
    }

    struct LocBuilder {
        ids: Vec<String>,
    }

    impl Builder<Vec<String>> for LocBuilder {
        fn is_empty(&self) -> bool {
            self.ids.is_empty()
        }

        fn build(&mut self) -> Vec<String> {
            std::mem::take(&mut self.ids)
        }

        fn reset(&mut self) {
            self.ids.clear();
        }
    }

    // Simulate parsing
    let mut handler = LocHandler {
        location_ids: Vec::new(),
    };
    let mut ctx = TransactionContext::new("FV2504");

    let pos = SegmentPosition::new(1, 0, 1);
    let loc1 = RawSegment::new(
        "LOC",
        vec![vec!["Z16"], vec!["DE00014545768S0000000000000003054"]],
        pos,
    );
    let loc2 = RawSegment::new(
        "LOC",
        vec![vec!["Z17"], vec!["DE00098765432100000000000000012"]],
        pos,
    );
    let loc3 = RawSegment::new(
        "LOC",
        vec![vec!["Z16"], vec!["DE00099887766500000000000000034"]],
        pos,
    );

    for seg in [&loc1, &loc2, &loc3] {
        if handler.can_handle(seg) {
            handler.handle(seg, &mut ctx);
        }
    }

    // Only LOC+Z16 segments should be handled
    assert_eq!(handler.location_ids.len(), 2);
    assert_eq!(
        handler.location_ids[0],
        "DE00014545768S0000000000000003054"
    );
    assert_eq!(
        handler.location_ids[1],
        "DE00099887766500000000000000034"
    );

    // Build
    let mut builder = LocBuilder {
        ids: handler.location_ids,
    };
    assert!(!builder.is_empty());

    let result = builder.build();
    assert_eq!(result.len(), 2);
    assert!(builder.is_empty());
}

/// Verify AutomapperError variants can be created from ParseError.
#[test]
fn test_automapper_error_from_parse_error() {
    use edifact_parser::ParseError;

    let parse_err = ParseError::UnexpectedEof;
    let auto_err: AutomapperError = parse_err.into();

    match auto_err {
        AutomapperError::Parse(_) => {} // expected
        other => panic!("expected Parse variant, got: {}", other),
    }
}
```

**Step 2: Run integration test**

Run: `cargo test -p automapper-core --test trait_dispatch_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/automapper-core/tests/
git commit -m "$(cat <<'EOF'
test(automapper-core): add integration tests for traits and version dispatch

Tests cover: VersionConfig parameterization, full pipeline with stub
coordinator, FormatVersion string roundtrip, TransactionContext reset,
SegmentHandler+Builder integration, and AutomapperError conversions.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
```
