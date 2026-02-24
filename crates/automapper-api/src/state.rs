//! Application state and coordinator registry.

use std::collections::HashMap;
use std::sync::Arc;

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::schema::ahb::AhbSchema;
use mig_assembly::ConversionService;
use mig_bo4e::code_lookup::CodeLookup;
use mig_bo4e::segment_structure::SegmentStructure;
use mig_bo4e::MappingEngine;

use crate::contracts::coordinators::CoordinatorInfo;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<CoordinatorRegistry>,
    pub mig_registry: Arc<MigServiceRegistry>,
    pub startup: std::time::Instant,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(CoordinatorRegistry::discover()),
            mig_registry: Arc::new(MigServiceRegistry::discover()),
            startup: std::time::Instant::now(),
        }
    }
}

/// Registry for MIG-driven conversion services, keyed by format version.
pub struct MigServiceRegistry {
    services: HashMap<String, ConversionService>,
    /// Combined engines (message + transaction) per PID, for backward compat.
    /// Key: "{fv}/{variant}/pid_{pid}" e.g. "FV2504/UTILMD_Strom/pid_55001"
    mapping_engines: HashMap<String, MappingEngine>,
    /// Message-level engines per variant (shared across PIDs).
    /// Key: "{fv}/{variant}" e.g. "FV2504/UTILMD_Strom"
    message_engines: HashMap<String, MappingEngine>,
    /// Transaction-only engines per PID (without message-level defs).
    /// Key: "{fv}/{variant}/pid_{pid}" e.g. "FV2504/UTILMD_Strom/pid_55001"
    transaction_engines: HashMap<String, MappingEngine>,
    ahb_schemas: HashMap<String, AhbSchema>,
}

impl MigServiceRegistry {
    /// Discover and load available MIG schemas.
    pub fn discover() -> Self {
        let mut services = HashMap::new();

        // Try to load the UTILMD Strom MIG for FV2504
        let mig_path = std::path::Path::new(
            "xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        );
        if mig_path.exists() {
            match ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504") {
                Ok(svc) => {
                    tracing::info!("Loaded MIG conversion service for FV2504");
                    services.insert("FV2504".to_string(), svc);
                }
                Err(e) => {
                    tracing::warn!("Failed to load MIG for FV2504: {e}");
                }
            }
        } else {
            tracing::info!(
                "MIG XML not found at {}, MIG-driven modes unavailable",
                mig_path.display()
            );
        }

        // Load TOML mapping definitions: mappings/{FV}/{msg_variant}/{pid}/
        // Also loads shared message-level definitions from mappings/{FV}/{msg_variant}/message/
        let mut mapping_engines = HashMap::new();
        let mut message_engines = HashMap::new();
        let mut transaction_engines = HashMap::new();
        let mappings_base = std::path::Path::new("mappings");
        if mappings_base.exists() {
            if let Ok(fv_entries) = std::fs::read_dir(mappings_base) {
                for fv_entry in fv_entries.flatten() {
                    let fv_path = fv_entry.path();
                    if !fv_path.is_dir() {
                        continue;
                    }
                    let fv = fv_entry.file_name().to_string_lossy().to_string();
                    // Iterate msg_variant dirs (e.g., UTILMD_Strom)
                    if let Ok(variant_entries) = std::fs::read_dir(&fv_path) {
                        for variant_entry in variant_entries.flatten() {
                            let variant_path = variant_entry.path();
                            if !variant_path.is_dir() {
                                continue;
                            }
                            let variant = variant_entry.file_name().to_string_lossy().to_string();

                            // Load message-level engine (shared across PIDs)
                            let message_dir = variant_path.join("message");
                            if message_dir.is_dir() {
                                match MappingEngine::load(&message_dir) {
                                    Ok(engine) => {
                                        // Attach CodeLookup from any available PID schema
                                        // (root_segments are identical across all PIDs)
                                        let engine = attach_code_lookup_for_message(
                                            &fv,
                                            &variant,
                                            &variant_path,
                                            engine,
                                        );
                                        let key = format!("{}/{}", fv, variant);
                                        tracing::info!(
                                            "Loaded {} message-level TOML mappings for {key}",
                                            engine.definitions().len()
                                        );
                                        message_engines.insert(key, engine);
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to load message mappings from {}: {e}",
                                            message_dir.display()
                                        );
                                    }
                                }
                            }

                            // Iterate PID dirs (e.g., pid_55001), skip message/
                            if let Ok(pid_entries) = std::fs::read_dir(&variant_path) {
                                for pid_entry in pid_entries.flatten() {
                                    let pid_path = pid_entry.path();
                                    if !pid_path.is_dir() {
                                        continue;
                                    }
                                    let dirname =
                                        pid_entry.file_name().to_string_lossy().to_string();
                                    // Skip message/ directory (already loaded above)
                                    if dirname == "message" {
                                        continue;
                                    }
                                    // Load combined engine (message + PID transaction defs)
                                    let load_result = if message_dir.is_dir() {
                                        MappingEngine::load_merged(&[
                                            message_dir.as_path(),
                                            pid_path.as_path(),
                                        ])
                                    } else {
                                        MappingEngine::load(&pid_path)
                                    };
                                    match load_result {
                                        Ok(engine) => {
                                            // Attach MIG-derived SegmentStructure if available
                                            let engine = if let Some(svc) = services.get(&fv) {
                                                engine.with_segment_structure(
                                                    SegmentStructure::from_mig(svc.mig()),
                                                )
                                            } else {
                                                engine
                                            };
                                            // Attach CodeLookup for companion field enrichment
                                            let engine =
                                                attach_code_lookup(&fv, &variant, &dirname, engine);
                                            let key = format!("{}/{}/{}", fv, variant, dirname);
                                            tracing::info!(
                                                "Loaded {} TOML mapping definitions for {key}",
                                                engine.definitions().len()
                                            );
                                            mapping_engines.insert(key, engine);
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to load mappings from {}: {e}",
                                                pid_path.display()
                                            );
                                        }
                                    }

                                    // Also load transaction-only engine (PID dir only, no message defs)
                                    match MappingEngine::load(&pid_path) {
                                        Ok(tx_engine) => {
                                            let tx_engine = if let Some(svc) = services.get(&fv) {
                                                tx_engine.with_segment_structure(
                                                    SegmentStructure::from_mig(svc.mig()),
                                                )
                                            } else {
                                                tx_engine
                                            };
                                            // Attach CodeLookup for companion field enrichment
                                            let tx_engine = attach_code_lookup(
                                                &fv, &variant, &dirname, tx_engine,
                                            );
                                            let key = format!("{}/{}/{}", fv, variant, dirname);
                                            transaction_engines.insert(key, tx_engine);
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to load transaction mappings from {}: {e}",
                                                pid_path.display()
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Load AHB schemas for PID lookup
        let mut ahb_schemas = HashMap::new();
        let ahb_base = std::path::Path::new("xml-migs-and-ahbs");
        if ahb_base.exists() {
            // Known format versions and their AHB configs
            let ahb_configs: Vec<(&str, &str, &str, &str)> =
                vec![("FV2504", "UTILMD", "Strom", "UTILMD_AHB_Strom_")];
            for (fv, msg_type, variant, prefix) in &ahb_configs {
                let fv_dir = ahb_base.join(fv);
                if let Ok(entries) = std::fs::read_dir(&fv_dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if fname.starts_with(prefix) && fname.ends_with(".xml") {
                            match parse_ahb(&entry.path(), msg_type, Some(variant), fv) {
                                Ok(schema) => {
                                    let key = format!("{}/{}_{}", fv, msg_type, variant);
                                    tracing::info!(
                                        "Loaded AHB schema for {key} with {} workflows",
                                        schema.workflows.len()
                                    );
                                    ahb_schemas.insert(key, schema);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to load AHB from {}: {e}",
                                        entry.path().display()
                                    );
                                }
                            }
                            break; // Only load first matching AHB per config
                        }
                    }
                }
            }
        }

        Self {
            services,
            mapping_engines,
            message_engines,
            transaction_engines,
            ahb_schemas,
        }
    }

    /// Get a conversion service for the given format version.
    pub fn service(&self, format_version: &str) -> Option<&ConversionService> {
        self.services.get(format_version)
    }

    /// Get a mapping engine for the given format version and message type/variant key.
    /// Key format: "FV2504/UTILMD_Strom/pid_55001"
    pub fn mapping_engine(&self, key: &str) -> Option<&MappingEngine> {
        self.mapping_engines.get(key)
    }

    /// Get a combined mapping engine (message + transaction) for a specific PID.
    /// Constructs key as "{fv}/{msg_variant}/pid_{pid}" (e.g., "FV2504/UTILMD_Strom/pid_55001").
    pub fn mapping_engine_for_pid(
        &self,
        fv: &str,
        msg_variant: &str,
        pid: &str,
    ) -> Option<&MappingEngine> {
        let key = format!("{}/{}/pid_{}", fv, msg_variant, pid);
        self.mapping_engines.get(&key)
    }

    /// Get split mapping engines for a specific PID.
    ///
    /// Returns `(message_engine, combined_engine)` where:
    /// - `message_engine` contains only message-level definitions (SG2/SG3/root)
    /// - `combined_engine` contains all definitions (message + transaction)
    ///
    /// Use the message engine with `map_interchange()` for multi-transaction mapping.
    pub fn mapping_engines_for_pid(
        &self,
        fv: &str,
        msg_variant: &str,
        pid: &str,
    ) -> Option<(&MappingEngine, &MappingEngine)> {
        let msg_key = format!("{}/{}", fv, msg_variant);
        let combined_key = format!("{}/{}/pid_{}", fv, msg_variant, pid);
        let msg = self.message_engines.get(&msg_key)?;
        let combined = self.mapping_engines.get(&combined_key)?;
        Some((msg, combined))
    }

    /// Get split mapping engines for hierarchical mapping (message + transaction separately).
    ///
    /// Returns `(message_engine, transaction_engine)` where:
    /// - `message_engine` contains only message-level definitions (SG2/SG3/root)
    /// - `transaction_engine` contains only PID-specific transaction definitions (SG4+)
    ///
    /// Use with `MappingEngine::map_interchange()` for the hierarchical pipeline.
    pub fn mapping_engines_split(
        &self,
        fv: &str,
        msg_variant: &str,
        pid: &str,
    ) -> Option<(&MappingEngine, &MappingEngine)> {
        let msg_key = format!("{}/{}", fv, msg_variant);
        let tx_key = format!("{}/{}/pid_{}", fv, msg_variant, pid);
        let msg = self.message_engines.get(&msg_key)?;
        let tx = self.transaction_engines.get(&tx_key)?;
        Some((msg, tx))
    }

    /// Get the message-level mapping engine for a variant.
    /// Key format: "FV2504/UTILMD_Strom"
    pub fn message_engine(&self, fv: &str, msg_variant: &str) -> Option<&MappingEngine> {
        let key = format!("{}/{}", fv, msg_variant);
        self.message_engines.get(&key)
    }

    /// Get an AHB schema for the given format version and message type/variant.
    /// Key format: "FV2504/UTILMD_Strom"
    pub fn ahb_schema(&self, fv: &str, msg_variant: &str) -> Option<&AhbSchema> {
        let key = format!("{}/{}", fv, msg_variant);
        self.ahb_schemas.get(&key)
    }

    /// Check if any MIG services are available.
    pub fn has_services(&self) -> bool {
        !self.services.is_empty()
    }
}

/// Try to load a CodeLookup from the PID schema JSON and attach it to the engine.
///
/// Schema path: `crates/mig-types/src/generated/{fv_lower}/{msg_type_lower}/pids/{pid_dirname}_schema.json`
fn attach_code_lookup(
    fv: &str,
    variant: &str,
    pid_dirname: &str,
    engine: MappingEngine,
) -> MappingEngine {
    let msg_type = variant.split('_').next().unwrap_or(variant).to_lowercase();
    let fv_lower = fv.to_lowercase();
    let schema_path = format!(
        "crates/mig-types/src/generated/{}/{}/pids/{}_schema.json",
        fv_lower, msg_type, pid_dirname
    );
    let schema_path = std::path::Path::new(&schema_path);
    if schema_path.exists() {
        match CodeLookup::from_schema_file(schema_path) {
            Ok(lookup) => {
                tracing::debug!("Loaded CodeLookup for {}/{}/{}", fv, variant, pid_dirname);
                engine.with_code_lookup(lookup)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load CodeLookup from {}: {e}",
                    schema_path.display()
                );
                engine
            }
        }
    } else {
        engine
    }
}

/// Attach a CodeLookup to the message-level engine using any available PID schema.
///
/// Root-level segments (BGM, DTM, etc.) are identical across all PID schemas,
/// so we just pick the first `pid_*` directory we find under the variant path.
fn attach_code_lookup_for_message(
    fv: &str,
    variant: &str,
    variant_path: &std::path::Path,
    engine: MappingEngine,
) -> MappingEngine {
    let Ok(entries) = std::fs::read_dir(variant_path) else {
        return engine;
    };
    for entry in entries.flatten() {
        let dirname = entry.file_name().to_string_lossy().to_string();
        if dirname.starts_with("pid_") {
            return attach_code_lookup(fv, variant, &dirname, engine);
        }
    }
    engine
}

/// Discovers and manages available coordinators.
pub struct CoordinatorRegistry {
    coordinators: HashMap<String, CoordinatorInfo>,
}

impl CoordinatorRegistry {
    /// Discover all available coordinators.
    pub fn discover() -> Self {
        let mut coordinators = HashMap::new();

        // Register UTILMD coordinator with known format versions
        coordinators.insert(
            "UTILMD".to_string(),
            CoordinatorInfo {
                message_type: "UTILMD".to_string(),
                description: "Coordinator for UTILMD (utility master data) messages".to_string(),
                supported_versions: vec!["FV2504".to_string(), "FV2510".to_string()],
            },
        );

        tracing::info!(
            "Discovered {} coordinators: {:?}",
            coordinators.len(),
            coordinators.keys().collect::<Vec<_>>()
        );

        Self { coordinators }
    }

    /// Get all available coordinators.
    pub fn list(&self) -> Vec<&CoordinatorInfo> {
        self.coordinators.values().collect()
    }

    /// Check if a coordinator exists for the given message type.
    pub fn has(&self, message_type: &str) -> bool {
        self.coordinators.contains_key(&message_type.to_uppercase())
    }

    /// Get coordinator info for a specific message type.
    pub fn get(&self, message_type: &str) -> Option<&CoordinatorInfo> {
        self.coordinators.get(&message_type.to_uppercase())
    }

    /// Inspect EDIFACT content, returning a segment tree.
    pub fn inspect_edifact(
        &self,
        edifact: &str,
    ) -> Result<crate::contracts::inspect::InspectResponse, crate::error::ApiError> {
        if edifact.trim().is_empty() {
            return Err(crate::error::ApiError::BadRequest {
                message: "EDIFACT content is required".to_string(),
            });
        }

        let segments = parse_edifact_to_segment_nodes(edifact);
        let segment_count = segments.len();

        // Detect message type from UNH segment
        let mut message_type = None;
        let mut format_version = None;

        for seg in &segments {
            if seg.tag == "UNH" && seg.elements.len() >= 2 {
                if let Some(ref components) = seg.elements[1].components {
                    if !components.is_empty() {
                        message_type = components[0].value.clone();
                    }
                    if components.len() >= 3 {
                        format_version = Some(format!(
                            "{}:{}",
                            components[1].value.as_deref().unwrap_or(""),
                            components[2].value.as_deref().unwrap_or("")
                        ));
                    }
                }
            }
        }

        Ok(crate::contracts::inspect::InspectResponse {
            segments,
            segment_count,
            message_type,
            format_version,
        })
    }
}

/// Parse raw EDIFACT text into a flat list of `SegmentNode` values.
fn parse_edifact_to_segment_nodes(edifact: &str) -> Vec<crate::contracts::inspect::SegmentNode> {
    use crate::contracts::inspect::{ComponentElement, DataElement, SegmentNode};

    let mut segments = Vec::new();
    let parts: Vec<&str> = edifact.split('\'').collect();
    let mut line_number = 0u32;

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        line_number += 1;

        let plus_index = trimmed.find('+');
        let tag = match plus_index {
            Some(idx) => &trimmed[..idx],
            None => trimmed,
        };

        let elements = if let Some(idx) = plus_index {
            let element_strs: Vec<&str> = trimmed[idx + 1..].split('+').collect();
            element_strs
                .iter()
                .enumerate()
                .map(|(i, elem_str)| {
                    let components_strs: Vec<&str> = elem_str.split(':').collect();
                    if components_strs.len() > 1 {
                        DataElement {
                            position: (i + 1) as u32,
                            value: None,
                            components: Some(
                                components_strs
                                    .iter()
                                    .enumerate()
                                    .map(|(j, comp)| ComponentElement {
                                        position: (j + 1) as u32,
                                        value: if comp.is_empty() {
                                            None
                                        } else {
                                            Some(comp.to_string())
                                        },
                                    })
                                    .collect(),
                            ),
                        }
                    } else {
                        DataElement {
                            position: (i + 1) as u32,
                            value: if elem_str.is_empty() {
                                None
                            } else {
                                Some(elem_str.to_string())
                            },
                            components: None,
                        }
                    }
                })
                .collect()
        } else {
            vec![]
        };

        segments.push(SegmentNode {
            tag: tag.to_string(),
            line_number,
            raw_content: trimmed.to_string(),
            elements,
            children: None,
        });
    }

    segments
}
