# Rust Port Design — edifact_bo4e_automapper

## Overview

Full port of the C#/.NET edifact_bo4e_automapper to Rust, covering the parser, mappers, validation, code generator, web API, and frontend. The primary goals are **performance** (batch processing millions of messages) and **publishing reusable Rust crates** for the energy market ecosystem.

## Document Version

- **Created**: 2026-02-18
- **Based on**: C# edifact_bo4e_automapper at commit cee0b09
- **Status**: Approved Design

---

## 1. Crate Structure

Cargo workspace with eight crates, ordered from lowest to highest dependency:

```
edifact-bo4e-automapper/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── edifact-types/         # Shared EDIFACT primitives (no dependencies)
│   ├── edifact-parser/        # Streaming EDIFACT tokenizer + parser
│   ├── bo4e-extensions/       # *Edifact companion types, WithValidity, LinkRegistry
│   ├── automapper-core/       # Coordinators, mappers, builders, writers
│   ├── automapper-validation/ # Condition evaluator, AHB expression parser, validator
│   ├── automapper-generator/  # CLI: reads MIG/AHB XML, generates Rust code, shells out to claude
│   ├── automapper-api/        # Axum REST + tonic gRPC server
│   └── automapper-web/        # Leptos WASM frontend
├── generated/                 # Generated mapper/condition code (committed)
├── xml-migs-and-ahbs/         # Git submodule (unchanged)
├── stammdatenmodell/          # Git submodule (unchanged)
├── example_market_communication_bo4e_transactions/  # Git submodule (unchanged)
└── tests/                     # Integration tests, fixtures, snapshots
```

**Key design decision**: `edifact-parser` is a standalone crate with no BO4E dependency. Anyone in the Rust ecosystem can use it for generic EDIFACT parsing.

### Dependency Graph

```
edifact-types
    ↑
edifact-parser
    ↑
bo4e-extensions  (also depends on: bo4e crate, serde, chrono)
    ↑
automapper-core  (also depends on: rayon)
    ↑
├── automapper-validation
├── automapper-generator  (also depends on: clap, quick-xml)
└── automapper-api        (also depends on: axum, tonic, tokio)
        ↑
    automapper-web        (also depends on: leptos)
```

---

## 2. `edifact-types` — Shared Primitives

Zero-dependency crate defining the core EDIFACT data structures.

```rust
pub struct EdifactDelimiters {
    pub component: u8,    // default ':'
    pub element: u8,      // default '+'
    pub decimal: u8,      // default '.'
    pub release: u8,      // default '?'
    pub segment: u8,      // default '\''
}

pub struct SegmentPosition {
    pub segment_number: u32,
    pub byte_offset: usize,
    pub message_number: u32,
}

/// Zero-copy segment: borrows from the input buffer
pub struct RawSegment<'a> {
    pub id: &'a str,                     // e.g. "NAD", "LOC", "DTM"
    pub elements: Vec<Vec<&'a str>>,     // elements[i][j] = component j of element i
    pub position: SegmentPosition,
}
```

Everything borrows from the input buffer (`'a` lifetime) — zero-copy parsing with no string allocations during the hot path.

---

## 3. `edifact-parser` — Streaming Parser

SAX-style event-driven parser matching the C# `EdifactStreamParser` architecture.

### Handler Trait

```rust
pub trait EdifactHandler {
    fn on_delimiters(&mut self, delimiters: &EdifactDelimiters, explicit_una: bool) {}
    fn on_interchange_start(&mut self, unb: &RawSegment) -> Control { Control::Continue }
    fn on_message_start(&mut self, unh: &RawSegment) -> Control { Control::Continue }
    fn on_segment(&mut self, segment: &RawSegment) -> Control { Control::Continue }
    fn on_message_end(&mut self, unt: &RawSegment) {}
    fn on_interchange_end(&mut self, unz: &RawSegment) {}
    fn on_error(&mut self, error: ParseError) -> Control { Control::Stop }
}

pub enum Control { Continue, Stop }
```

### Parser API

```rust
pub struct EdifactStreamParser;

impl EdifactStreamParser {
    /// Parse from a byte slice (zero-copy)
    pub fn parse(input: &[u8], handler: &mut dyn EdifactHandler) -> Result<(), ParseError>;

    /// Parse from an async reader (streaming, for large files)
    pub async fn parse_async<R: AsyncRead>(
        reader: R,
        handler: &mut dyn EdifactHandler,
    ) -> Result<(), ParseError>;
}
```

### Batch Processing

```rust
/// Parse multiple interchanges in parallel using rayon
pub fn parse_batch(
    inputs: &[&[u8]],
    handler_factory: impl Fn() -> Box<dyn EdifactHandler> + Send + Sync,
) -> Vec<Result<(), ParseError>>;
```

The parser handles UNA detection, release character escaping, and segment boundary detection. It emits `RawSegment` references that borrow directly from the input buffer.

---

## 4. `bo4e-extensions` — Domain Types

Bridges the standard `bo4e-rust` crate with EDIFACT-specific functional domain data. Depends on the external `bo4e` crate for standard types (Marktlokation, Messlokation, Vertrag, etc.).

### Core Wrapper

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithValidity<T, E> {
    pub data: T,                              // standard BO4E object
    pub edifact: E,                           // EDIFACT-specific functional data
    pub gueltigkeitszeitraum: Option<Zeitraum>,
    pub zeitscheibe_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zeitraum {
    pub von: Option<NaiveDateTime>,
    pub bis: Option<NaiveDateTime>,
}
```

### EDIFACT Companion Types

These store **functional domain data** that exists in EDIFACT but has no home in standard BO4E. They do NOT store transport/serialization concerns (segment ordering, byte offsets). Roundtrip fidelity for ordering is handled by deterministic MIG-derived rules in the coordinator/writer layer.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarktlokationEdifact {
    /// Object code in Lokationsbuendel (SEQ+Z78)
    pub lokationsbuendel_objektcode: Option<String>,
    /// VOLLSTAENDIG, ERWARTET, INFORMATIV, etc.
    pub datenqualitaet: Option<DataQuality>,
    /// RFF+Z32 reference to associated Netzlokation
    pub referenz_netzlokation: Option<String>,
    /// References to upstream locations
    pub vorgelagerte_lokations_ids: Option<Vec<LokationsTypZuordnung>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MesslokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_netzlokation: Option<String>,
    pub vorgelagerte_lokations_ids: Option<Vec<LokationsTypZuordnung>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZaehlerEdifact {
    /// RFF+Z19 reference to Messlokation
    pub referenz_messlokation: Option<String>,
    /// RFF+Z14 reference to Gateway
    pub referenz_gateway: Option<String>,
    /// SEQ+Z79 product package ID
    pub produktpaket_id: Option<String>,
    /// Whether this is a Smartmeter-Gateway entry (SEQ+Z75)
    pub is_smartmeter_gateway: Option<bool>,
    /// Gateway assignment reference
    pub smartmeter_gateway_zuordnung: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeschaeftspartnerEdifact {
    /// NAD qualifier (Z04, Z09, DP, etc.) — which role this party plays
    pub nad_qualifier: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VertragEdifact {
    /// Haushaltskunde indicator from CCI+Z15/Z18
    pub haushaltskunde: Option<bool>,
    /// Versorgungsart from CCI+Z36
    pub versorgungsart: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetzlokationEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_marktlokation: Option<String>,
    pub zugeordnete_messlokationen: Option<Vec<LokationsTypZuordnung>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnischeRessourceEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub referenz_marktlokation: Option<String>,
    pub referenz_steuerbare_ressource: Option<String>,
    pub referenz_messlokation: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SteuerbareRessourceEdifact {
    pub lokationsbuendel_objektcode: Option<String>,
    pub datenqualitaet: Option<DataQuality>,
    pub produktpaket_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataQuality {
    Vollstaendig,
    Erwartet,
    ImSystemVorhanden,
    Informativ,
}
```

### Transaction Container

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilmdNachricht {
    pub nachrichtendaten: Nachrichtendaten,
    pub dokumentennummer: String,
    pub kategorie: DokumentKategorie,
    pub transaktionen: Vec<UtilmdTransaktion>,
}

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
```

### URI-Based Linking

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bo4eUri(String);  // e.g. "bo4e://Marktlokation/DE00014545768S0000000000000003054"

impl Bo4eUri {
    pub fn new(type_name: &str, id: &str) -> Self;
    pub fn type_name(&self) -> &str;
    pub fn id(&self) -> &str;
}

#[derive(Debug, Clone, Default)]
pub struct LinkRegistry {
    links: HashMap<Bo4eUri, Vec<Bo4eUri>>,
}
```

All types derive `Serialize`/`Deserialize`, so the crate doubles as the JSON contract for the API.

---

## 5. `automapper-core` — Coordinators, Mappers, Writers

The main library crate. Streaming handlers coordinated per message type with bidirectional mappers.

### Trait Hierarchy

```rust
/// Reads EDIFACT segments into domain objects
pub trait SegmentHandler: Send {
    fn can_handle(&self, segment: &RawSegment) -> bool;
    fn handle(&mut self, segment: &RawSegment, ctx: &mut TransactionContext);
}

/// Accumulates state across multiple segments
pub trait Builder<T>: Send {
    fn is_empty(&self) -> bool;
    fn build(&mut self) -> T;
    fn reset(&mut self);
}

/// Serializes domain objects back to EDIFACT segments
pub trait EntityWriter: Send {
    fn write(&self, writer: &mut EdifactSegmentWriter, ctx: &TransactionContext);
}

/// Bidirectional mapper combining reading + writing for one entity type
pub trait Mapper: SegmentHandler + EntityWriter {
    fn format_version(&self) -> FormatVersion;
}
```

### Format Version Dispatch — Hybrid Approach

Traits for the hot path (compile-time, zero-cost), enum at the entry point for runtime selection.

```rust
pub enum FormatVersion { FV2504, FV2510 }

/// Marker types for compile-time dispatch
pub struct FV2504;
pub struct FV2510;

pub trait VersionConfig: Send + 'static {
    const VERSION: FormatVersion;
    type MarktlokationMapper: Mapper + Default;
    type VertragMapper: Mapper + Default;
    type ZaehlerMapper: Mapper + Default;
    // ... one associated type per entity mapper
}

impl VersionConfig for FV2504 {
    const VERSION: FormatVersion = FormatVersion::FV2504;
    type MarktlokationMapper = marktlokation::MapperV2504;
    // ...
}

impl VersionConfig for FV2510 {
    const VERSION: FormatVersion = FormatVersion::FV2510;
    type MarktlokationMapper = marktlokation::MapperV2510;
    // FV2510 only overrides what changed from FV2504
    // ...
}
```

### Coordinator

```rust
pub trait Coordinator: EdifactHandler + Send {
    fn parse(&mut self, input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError>;
    fn generate(&self, transaktion: &UtilmdTransaktion) -> Result<Vec<u8>, AutomapperError>;
}

pub struct UtilmdCoordinator<V: VersionConfig> {
    mappers: Vec<Box<dyn Mapper>>,
    context: TransactionContext,
    link_registry: LinkRegistry,
    _version: PhantomData<V>,
}

impl<V: VersionConfig> UtilmdCoordinator<V> {
    pub fn new() -> Self {
        let mut coord = Self { /* ... */ };
        coord.register(V::MarktlokationMapper::default());
        coord.register(V::VertragMapper::default());
        coord.register(V::ZaehlerMapper::default());
        // ...
        coord
    }
}

/// Runtime entry point — the enum boundary
pub fn create_coordinator(fv: FormatVersion) -> Box<dyn Coordinator> {
    match fv {
        FormatVersion::FV2504 => Box::new(UtilmdCoordinator::<FV2504>::new()),
        FormatVersion::FV2510 => Box::new(UtilmdCoordinator::<FV2510>::new()),
    }
}
```

### Batch Processing

```rust
use rayon::prelude::*;

pub fn convert_batch(
    inputs: &[&[u8]],
    fv: FormatVersion,
) -> Vec<Result<Vec<UtilmdTransaktion>, AutomapperError>> {
    inputs.par_iter().map(|input| {
        let mut coord = create_coordinator(fv);
        coord.parse(input)
    }).collect()
}
```

Each message gets its own coordinator instance — no shared mutable state, perfect for rayon parallelism.

---

## 6. `automapper-validation` — AHB Validation

Validates EDIFACT messages against AHB (Anwendungshandbuch) business rules.

### Condition Expression AST

```rust
#[derive(Debug, Clone)]
pub enum ConditionExpr {
    Ref(u32),                                    // [1], [2], etc.
    And(Vec<ConditionExpr>),
    Or(Vec<ConditionExpr>),
    Xor(Box<ConditionExpr>, Box<ConditionExpr>),
    Not(Box<ConditionExpr>),
}

pub struct ConditionParser;

impl ConditionParser {
    pub fn parse(input: &str) -> Result<ConditionExpr, ParseError>;
}
```

### Condition Evaluator

```rust
pub enum ConditionResult { True, False, Unknown }

pub trait ConditionEvaluator: Send + Sync {
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult;
    fn is_external(&self, condition: u32) -> bool;
}

pub struct EvaluationContext<'a> {
    pub transaktion: &'a UtilmdTransaktion,
    pub pruefidentifikator: &'a str,
    pub external: &'a dyn ExternalConditionProvider,
}

/// For conditions requiring runtime business context
pub trait ExternalConditionProvider: Send + Sync {
    fn evaluate(&self, condition_name: &str) -> ConditionResult;
}
```

### Generated Evaluator (output of automapper-generator)

```rust
// generated/utilmd_conditions_fv2510.rs
pub struct UtilmdConditionEvaluatorFV2510;

impl ConditionEvaluator for UtilmdConditionEvaluatorFV2510 {
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult {
        match condition {
            1 => self.evaluate_1(ctx),  // "Wenn Aufteilung vorhanden"
            2 => self.evaluate_2(ctx),
            8 => ConditionResult::Unknown, // external
            // ... 100+ conditions
            _ => ConditionResult::Unknown,
        }
    }

    fn is_external(&self, condition: u32) -> bool {
        matches!(condition, 8 | 15 | 23 /* ... */)
    }
}
```

### Validator

```rust
pub enum ValidationLevel { Structure, Conditions, Full }

pub struct ValidationReport {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

pub struct ValidationIssue {
    pub segment_position: Option<SegmentPosition>,
    pub rule: String,
    pub message: String,
    pub severity: Severity,
}

pub struct EdifactValidator<E: ConditionEvaluator> {
    evaluator: E,
}

impl<E: ConditionEvaluator> EdifactValidator<E> {
    pub fn validate(
        &self,
        input: &[u8],
        level: ValidationLevel,
        external: &dyn ExternalConditionProvider,
    ) -> Result<ValidationReport, AutomapperError>;
}
```

---

## 7. `automapper-generator` — Code Generation CLI

Reads MIG/AHB XML schemas and generates Rust source code. Uses `clap` for CLI, `quick-xml` for XML parsing.

### CLI Commands

```rust
#[derive(Parser)]
#[command(name = "automapper-generator")]
pub enum Cli {
    /// Generate mapper code from MIG XML schemas
    GenerateMappers {
        #[arg(long)]
        mig_path: PathBuf,
        #[arg(long)]
        ahb_path: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
        #[arg(long)]
        format_version: String,
        #[arg(long)]
        message_type: String,
    },

    /// Generate condition evaluators from AHB rules
    GenerateConditions {
        #[arg(long)]
        ahb_path: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
        #[arg(long)]
        format_version: String,
        #[arg(long)]
        message_type: String,
        #[arg(long, default_value = "false")]
        incremental: bool,
    },

    /// Validate generated code against BO4E schema
    ValidateSchema {
        #[arg(long)]
        stammdatenmodell_path: PathBuf,
        #[arg(long)]
        generated_dir: PathBuf,
    },
}
```

### AI-Assisted Condition Generation

Shells out to the `claude` CLI to reuse existing subscription. No SDK dependency.

```rust
pub struct ClaudeConditionGenerator {
    max_concurrent: usize,
}

impl ClaudeConditionGenerator {
    pub fn generate_condition(
        &self,
        condition_number: u32,
        description: &str,
        context: &ConditionContext,
    ) -> Result<GeneratedCondition, GeneratorError> {
        let prompt = self.build_prompt(condition_number, description, context);

        let output = Command::new("claude")
            .args(["--print", "--model", "sonnet", "--max-tokens", "4096"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        // write prompt to stdin, read generated Rust code from stdout
        // parse and validate the output
    }
}

pub struct GeneratedCondition {
    pub condition_number: u32,
    pub rust_code: String,
    pub is_external: bool,
    pub confidence: f32,
}
```

### MIG/AHB Schema Parsing

```rust
pub struct MigSchema {
    pub message_type: String,
    pub format_version: String,
    pub segments: Vec<MigSegment>,
}

pub struct MigSegment {
    pub id: String,
    pub qualifier: Option<String>,
    pub composites: Vec<MigComposite>,
    pub cardinality: Cardinality,
    pub group: Option<String>,
}

pub struct AhbSchema {
    pub pruefidentifikatoren: Vec<Pruefidentifikator>,
}

pub struct Pruefidentifikator {
    pub id: String,
    pub description: String,
    pub rules: Vec<AhbRule>,
}

pub fn parse_mig(path: &Path) -> Result<MigSchema, GeneratorError>;
pub fn parse_ahb(path: &Path) -> Result<AhbSchema, GeneratorError>;
```

Generated files are committed to `generated/` — no build-time codegen, keeping builds simple and generated code inspectable.

---

## 8. `automapper-api` + `automapper-web` — Web Layer

### REST API (Axum)

```rust
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/v1/convert/edifact-to-bo4e", post(convert_edifact_to_bo4e))
        .route("/api/v1/convert/bo4e-to-edifact", post(convert_bo4e_to_edifact))
        .route("/api/v1/inspect/edifact", post(inspect_edifact))
        .route("/api/v1/coordinators", get(list_coordinators))
        .route("/health", get(health))
        .nest_service("/", ServeDir::new("static"))  // serves Leptos WASM
        .with_state(AppState::new());

    // gRPC on the same port
    let grpc = tonic::transport::Server::builder()
        .add_service(TransformServiceServer::new(TransformServiceImpl::new()))
        .add_service(InspectionServiceServer::new(InspectionServiceImpl::new()));
}
```

### REST Contracts

```rust
#[derive(Deserialize)]
pub struct ConvertRequest {
    pub content: String,
    pub format_version: Option<FormatVersion>,
}

#[derive(Serialize)]
pub struct ConvertResponse {
    pub result: String,
    pub trace: Vec<TraceEntry>,
    pub errors: Vec<ApiError>,
    pub duration_ms: f64,
}

#[derive(Serialize)]
pub struct InspectResponse {
    pub segments: Vec<SegmentNode>,
    pub segment_count: usize,
    pub message_type: Option<String>,
    pub format_version: Option<FormatVersion>,
}
```

### Leptos Frontend

Two-panel converter UI with collapsible detail panels:

```rust
#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=ConverterPage />
                <Route path="/coordinators" view=CoordinatorsPage />
            </Routes>
        </Router>
    }
}

#[component]
fn ConverterPage() -> impl IntoView {
    let (input, set_input) = signal(String::new());
    let (output, set_output) = signal(String::new());
    let (direction, set_direction) = signal(Direction::EdifactToBo4e);

    view! {
        <div class="converter-layout">
            <div class="editor-panel">
                <CodeEditor value=input on_change=set_input />
            </div>
            <div class="controls">
                <DirectionToggle direction on_toggle=set_direction />
                <button on:click=move |_| convert.dispatch(())>"Convert"</button>
            </div>
            <div class="editor-panel">
                <CodeEditor value=output readonly=true />
            </div>
        </div>

        <CollapsiblePanel title="Segment Tree">
            <SegmentTreeView segments />
        </CollapsiblePanel>
        <CollapsiblePanel title="Mapping Trace">
            <TraceTable entries=trace />
        </CollapsiblePanel>
        <CollapsiblePanel title="Errors">
            <ErrorList errors />
        </CollapsiblePanel>
    }
}
```

The Leptos app compiles to WASM and is served as static files by the Axum server. Everything ships as a **single binary**.

---

## 9. Error Handling

`thiserror` for typed errors in library crates. No `anyhow` in library code.

```rust
// edifact-parser
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid UNA header at byte {offset}")]
    InvalidUna { offset: usize },
    #[error("unterminated segment at byte {offset}")]
    UnterminatedSegment { offset: usize },
    #[error("unexpected end of input")]
    UnexpectedEof,
}

// automapper-core
#[derive(Debug, thiserror::Error)]
pub enum AutomapperError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("unknown format version: {0}")]
    UnknownFormatVersion(String),
    #[error("mapping error in {segment} at position {position}: {message}")]
    Mapping { segment: String, position: u32, message: String },
    #[error("roundtrip mismatch: {message}")]
    RoundtripMismatch { message: String },
}
```

---

## 10. Testing Strategy

### Test Structure

```
tests/
├── unit/               # per-crate #[cfg(test)] modules
├── integration/
│   ├── roundtrip.rs    # EDIFACT -> BO4E -> EDIFACT byte-identical checks
│   ├── forward.rs      # EDIFACT -> BO4E only
│   └── validation.rs   # AHB rule validation
├── fixtures/           # symlinks to submodule test data
└── snapshots/          # insta snapshot files
```

### Key Testing Crates

| Crate | Purpose |
|-------|---------|
| `insta` | Snapshot testing (replaces Verify.Xunit) |
| `proptest` | Property-based testing for parser (no panics on arbitrary input) |
| `criterion` | Benchmarks for parser throughput and batch conversion |
| `test-case` | Parameterized tests over fixture files |

### Roundtrip Test Pattern

```rust
#[test_case("fixtures/UTILMD/FV2504/supplier_switch.edi")]
#[test_case("fixtures/UTILMD/FV2510/meter_change.edi")]
fn roundtrip(path: &str) {
    let input = std::fs::read(path).unwrap();
    let fv = detect_format_version(&input).unwrap();
    let mut coord = create_coordinator(fv);

    let transactions = coord.parse(&input).unwrap();
    let output = coord.generate(&transactions[0]).unwrap();

    assert_eq!(input, output, "roundtrip mismatch for {path}");
}
```

---

## 11. Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `bo4e` | latest | Standard BO4E Rust types (external) |
| `serde` + `serde_json` | 1.x | Serialization for all domain types |
| `chrono` | 0.4 | Date/time handling |
| `thiserror` | 2.x | Typed errors in library crates |
| `clap` | 4.x | CLI argument parsing (generator) |
| `quick-xml` | 0.37 | MIG/AHB XML schema parsing |
| `rayon` | 1.x | Parallel batch processing |
| `axum` | 0.8 | HTTP REST server |
| `tonic` + `prost` | 0.12 | gRPC server + protobuf |
| `leptos` | 0.7 | WASM frontend |
| `tokio` | 1.x | Async runtime |
| `tracing` | 0.1 | Structured logging/diagnostics |
| `insta` | 1.x | Snapshot testing |
| `criterion` | 0.5 | Benchmarks |
| `proptest` | 1.x | Property-based testing |
| `test-case` | 3.x | Parameterized tests |

---

## 12. Design Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Crate split | 8 crates in workspace | `edifact-parser` publishable standalone; clean dependency layers |
| Parser architecture | SAX-style streaming, zero-copy | Matches C# design; zero allocations in hot path |
| BO4E integration | Depend on `bo4e-rust` + extension types in-repo | Standard types from crate; EDIFACT domain data in companion structs |
| Companion types | Functional domain data only | No transport/ordering concerns; roundtrip via MIG-derived rules |
| Format version dispatch | Trait-based + enum boundary | Zero-cost in hot path; runtime selection at entry point |
| AI condition generation | Shell out to `claude` CLI | Reuses existing subscription; no SDK dependency |
| Web framework | Axum + Leptos | Full Rust stack; single binary deployment |
| API design | Fresh Rust-idiomatic REST + gRPC | Clean break from C# contracts; no backward compat needed |
| Error handling | `thiserror` in libraries | Typed, composable errors; no `anyhow` in library code |
| Testing | `insta` + `proptest` + `criterion` + `test-case` | Snapshots, fuzzing, benchmarks, parameterized fixtures |
