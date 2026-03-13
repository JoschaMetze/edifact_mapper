//! Application state and coordinator registry.

use std::collections::HashMap;
use std::sync::Arc;

use automapper_generator::parsing::ahb_parser::parse_ahb;
use automapper_generator::schema::ahb::AhbSchema;
use mig_assembly::parsing::parse_mig;
use mig_assembly::ConversionService;
use mig_bo4e::code_lookup::CodeLookup;
use mig_bo4e::engine::VariantCache;
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_bo4e::segment_structure::SegmentStructure;
use mig_bo4e::MappingEngine;
use mig_types::schema::mig::MigSchema;

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
    /// PID → variant mapping derived from cache keys.
    /// Key: "{fv}/pid_{pid}" → variant name (e.g., "UTILMD_Strom")
    pid_to_variant: HashMap<String, String>,
    /// Per-PID AHB segment numbers from cache.
    /// Key: "{fv}/{variant}/pid_{pid}" → segment number list
    pid_segment_numbers: HashMap<String, Vec<String>>,
    /// Flat mapping engines for response messages (APERAK, CONTRL).
    /// Key: "{fv}/{msg_type}" e.g. "FV2504/APERAK"
    response_engines: HashMap<String, MappingEngine>,
    /// MIG schemas for response message types.
    /// Key: "{fv}/{msg_type}" e.g. "FV2504/APERAK"
    response_migs: HashMap<String, MigSchema>,
}

impl MigServiceRegistry {
    /// Discover and load available MIG schemas.
    pub fn discover() -> Self {
        let mut services = HashMap::new();

        // MIG services are populated from variant cache (preferred) or MIG XML fallback.
        // The variant cache loading below will insert ConversionService entries from
        // cached MigSchema when available.

        // PID resolution indices built from cache
        let mut pid_to_variant: HashMap<String, String> = HashMap::new();
        let mut pid_segment_numbers: HashMap<String, Vec<String>> = HashMap::new();

        // Load TOML mapping definitions: mappings/{FV}/{msg_variant}/{pid}/
        // Also loads shared message-level definitions from mappings/{FV}/{msg_variant}/message/
        // Tries precompiled cache files first (cache/mappings/), falls back to TOML.
        let mut mapping_engines = HashMap::new();
        let mut message_engines = HashMap::new();
        let mut transaction_engines = HashMap::new();
        let mappings_base = std::path::Path::new("mappings");
        let cache_base = std::path::Path::new("cache/mappings");
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

                            // Try consolidated VariantCache first (one file for all engines)
                            let variant_cache_path =
                                cache_base.join(&fv).join(format!("{}.json", variant));
                            if variant_cache_path.exists() {
                                match VariantCache::load(&variant_cache_path) {
                                    Ok(vc) => {
                                        // Register ConversionService from cached MIG schema.
                                        // Only use UTILMD_Strom variants (the primary MIG for assembly).
                                        if let Some(mig) = vc.mig_schema {
                                            if !services.contains_key(&fv)
                                                && variant.starts_with("UTILMD_Strom")
                                            {
                                                tracing::info!(
                                                    "Loaded MIG schema for {fv} from variant cache ({variant})"
                                                );
                                                services.insert(
                                                    fv.clone(),
                                                    ConversionService::from_mig(mig),
                                                );
                                            }
                                        }

                                        // Use first cached code lookup for message engine
                                        let first_code_lookup = vc
                                            .code_lookups
                                            .values()
                                            .next()
                                            .cloned();

                                        // Insert message engine
                                        if !vc.message_defs.is_empty() {
                                            let mut engine =
                                                MappingEngine::from_definitions(vc.message_defs);
                                            if let Some(cl) = first_code_lookup {
                                                engine = engine.with_code_lookup(cl);
                                            }
                                            let key = format!("{}/{}", fv, variant);
                                            tracing::info!(
                                                "Loaded {} message mappings for {key} (variant cache)",
                                                engine.definitions().len()
                                            );
                                            message_engines.insert(key, engine);
                                        }

                                        // Insert combined and transaction engines per PID
                                        // Use cached segment structure (avoids re-deriving from MIG)
                                        let seg_struct = &vc.segment_structure;

                                        for (pid_dirname, combined_defs) in vc.combined_defs {
                                            let mut engine =
                                                MappingEngine::from_definitions(combined_defs);
                                            if let Some(ss) = seg_struct {
                                                engine =
                                                    engine.with_segment_structure(ss.clone());
                                            }
                                            if let Some(cl) = vc.code_lookups.get(&pid_dirname) {
                                                engine = engine.with_code_lookup(cl.clone());
                                            }
                                            let key = format!("{}/{}/{}", fv, variant, pid_dirname);
                                            mapping_engines.insert(key, engine);
                                        }

                                        for (pid_dirname, tx_defs) in vc.transaction_defs {
                                            let mut engine =
                                                MappingEngine::from_definitions(tx_defs);
                                            if let Some(ss) = seg_struct {
                                                engine =
                                                    engine.with_segment_structure(ss.clone());
                                            }
                                            if let Some(cl) = vc.code_lookups.get(&pid_dirname) {
                                                engine = engine.with_code_lookup(cl.clone());
                                            }
                                            let key = format!("{}/{}/{}", fv, variant, pid_dirname);
                                            transaction_engines.insert(key, engine);
                                        }

                                        // Populate PID→variant and PID→segment_numbers from cache
                                        for pid_dirname in mapping_engines
                                            .keys()
                                            .filter(|k| k.starts_with(&format!("{}/{}/", fv, variant)))
                                            .filter_map(|k| k.rsplit('/').next())
                                            .map(|s| s.to_string())
                                            .collect::<Vec<_>>()
                                        {
                                            let pid_key = format!("{}/{}", fv, pid_dirname);
                                            pid_to_variant.insert(pid_key, variant.clone());
                                        }
                                        for (pid_dirname, numbers) in &vc.pid_segment_numbers {
                                            let key = format!("{}/{}/{}", fv, variant, pid_dirname);
                                            pid_segment_numbers.insert(key, numbers.clone());
                                        }

                                        tracing::info!(
                                            "Loaded variant cache for {}/{} ({} combined, {} tx engines, {} code lookups, {} pid seg nums, mig: {})",
                                            fv,
                                            variant,
                                            mapping_engines
                                                .keys()
                                                .filter(|k| k.starts_with(&format!(
                                                    "{}/{}/",
                                                    fv, variant
                                                )))
                                                .count(),
                                            transaction_engines
                                                .keys()
                                                .filter(|k| k.starts_with(&format!(
                                                    "{}/{}/",
                                                    fv, variant
                                                )))
                                                .count(),
                                            vc.code_lookups.len(),
                                            vc.pid_segment_numbers.len(),
                                            if services.contains_key(&fv) { "cached" } else { "none" },
                                        );
                                        continue; // Skip per-PID iteration
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Variant cache load failed for {}: {e}, falling back to individual files",
                                            variant_cache_path.display()
                                        );
                                    }
                                }
                            }

                            // Fall back to individual .bin files or TOML loading

                            // PathResolver for EDIFACT ID path resolution (lazy — only for TOML fallback)
                            let msg_type =
                                variant.split('_').next().unwrap_or(&variant).to_lowercase();
                            let fv_lower = fv.to_lowercase();
                            let schema_dir_path = format!(
                                "crates/mig-types/src/generated/{}/{}/pids",
                                fv_lower, msg_type
                            );
                            let mut resolver: Option<PathResolver> = None;
                            let ensure_resolver = |resolver: &mut Option<PathResolver>| {
                                if resolver.is_none() {
                                    let dir = std::path::Path::new(&schema_dir_path);
                                    if dir.is_dir() {
                                        *resolver = Some(PathResolver::from_schema_dir(dir));
                                    }
                                }
                            };

                            // Load message-level engine (shared across PIDs)
                            // Try cache first, then fall back to TOML
                            let message_dir = variant_path.join("message");
                            let msg_cache = cache_base.join(&fv).join(&variant).join("msg.bin");
                            if msg_cache.exists() {
                                match MappingEngine::load_cached(&msg_cache) {
                                    Ok(engine) => {
                                        let engine = attach_code_lookup_for_message(
                                            &fv,
                                            &variant,
                                            &variant_path,
                                            engine,
                                        );
                                        let key = format!("{}/{}", fv, variant);
                                        tracing::info!(
                                            "Loaded {} cached message mappings for {key}",
                                            engine.definitions().len()
                                        );
                                        message_engines.insert(key, engine);
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Cache load failed for {}: {e}",
                                            msg_cache.display()
                                        );
                                    }
                                }
                            } else if message_dir.is_dir() {
                                ensure_resolver(&mut resolver);
                                match MappingEngine::load(&message_dir) {
                                    Ok(engine) => {
                                        let engine = if let Some(ref r) = resolver {
                                            engine.with_path_resolver(r.clone())
                                        } else {
                                            engine
                                        };
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

                            // Iterate PID dirs (e.g., pid_55001), skip message/ and common/
                            let common_dir = variant_path.join("common");
                            if let Ok(pid_entries) = std::fs::read_dir(&variant_path) {
                                for pid_entry in pid_entries.flatten() {
                                    let pid_path = pid_entry.path();
                                    if !pid_path.is_dir() {
                                        continue;
                                    }
                                    let dirname =
                                        pid_entry.file_name().to_string_lossy().to_string();
                                    // Skip message/ and common/ directories
                                    if dirname == "message" || dirname == "common" {
                                        continue;
                                    }

                                    // Try cache first for both combined and transaction engines
                                    let cache_dir = cache_base.join(&fv).join(&variant);
                                    let combined_cache =
                                        cache_dir.join(format!("combined_{}.bin", dirname));
                                    let tx_cache = cache_dir.join(format!("tx_{}.bin", dirname));

                                    if combined_cache.exists() && tx_cache.exists() {
                                        if let Ok(engine) =
                                            MappingEngine::load_cached(&combined_cache)
                                        {
                                            let engine = if let Some(svc) = services.get(&fv) {
                                                engine.with_segment_structure(
                                                    SegmentStructure::from_mig(svc.mig()),
                                                )
                                            } else {
                                                engine
                                            };
                                            let engine =
                                                attach_code_lookup(&fv, &variant, &dirname, engine);
                                            let key = format!("{}/{}/{}", fv, variant, dirname);
                                            tracing::info!(
                                                "Loaded {} cached mappings for {key}",
                                                engine.definitions().len()
                                            );
                                            mapping_engines.insert(key, engine);
                                        }
                                        if let Ok(tx_engine) = MappingEngine::load_cached(&tx_cache)
                                        {
                                            let tx_engine = if let Some(svc) = services.get(&fv) {
                                                tx_engine.with_segment_structure(
                                                    SegmentStructure::from_mig(svc.mig()),
                                                )
                                            } else {
                                                tx_engine
                                            };
                                            let tx_engine = attach_code_lookup(
                                                &fv, &variant, &dirname, tx_engine,
                                            );
                                            let key = format!("{}/{}/{}", fv, variant, dirname);
                                            transaction_engines.insert(key, tx_engine);
                                        }
                                        continue;
                                    }

                                    // Fall back to TOML loading
                                    ensure_resolver(&mut resolver);
                                    // Load combined engine (message + common + PID transaction defs)
                                    let load_result = if message_dir.is_dir() {
                                        if common_dir.is_dir() {
                                            // Try schema-aware common loading
                                            let pid_num =
                                                dirname.strip_prefix("pid_").unwrap_or(&dirname);
                                            let schema_file =
                                                std::path::Path::new(&schema_dir_path)
                                                    .join(format!("pid_{pid_num}_schema.json"));
                                            if let Ok(idx) =
                                                PidSchemaIndex::from_schema_file(&schema_file)
                                            {
                                                // Load message + common-aware tx, then merge
                                                let tx = MappingEngine::load_with_common(
                                                    &common_dir,
                                                    &pid_path,
                                                    &idx,
                                                );
                                                let msg = MappingEngine::load(&message_dir);
                                                match (msg, tx) {
                                                    (Ok(m), Ok(t)) => {
                                                        let mut defs = m.definitions().to_vec();
                                                        defs.extend(t.definitions().to_vec());
                                                        Ok(MappingEngine::from_definitions(defs))
                                                    }
                                                    (Err(e), _) | (_, Err(e)) => Err(e),
                                                }
                                            } else {
                                                MappingEngine::load_merged(&[
                                                    message_dir.as_path(),
                                                    pid_path.as_path(),
                                                ])
                                            }
                                        } else {
                                            MappingEngine::load_merged(&[
                                                message_dir.as_path(),
                                                pid_path.as_path(),
                                            ])
                                        }
                                    } else {
                                        MappingEngine::load(&pid_path)
                                    };
                                    match load_result {
                                        Ok(engine) => {
                                            let engine = if let Some(ref r) = resolver {
                                                engine.with_path_resolver(r.clone())
                                            } else {
                                                engine
                                            };
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

                                    // Also load transaction-only engine (with common inheritance)
                                    let tx_load_result = if common_dir.is_dir() {
                                        let pid_num =
                                            dirname.strip_prefix("pid_").unwrap_or(&dirname);
                                        let schema_file = std::path::Path::new(&schema_dir_path)
                                            .join(format!("pid_{pid_num}_schema.json"));
                                        if let Ok(idx) =
                                            PidSchemaIndex::from_schema_file(&schema_file)
                                        {
                                            MappingEngine::load_with_common(
                                                &common_dir,
                                                &pid_path,
                                                &idx,
                                            )
                                        } else {
                                            MappingEngine::load(&pid_path)
                                        }
                                    } else {
                                        MappingEngine::load(&pid_path)
                                    };
                                    match tx_load_result {
                                        Ok(tx_engine) => {
                                            let tx_engine = if let Some(ref r) = resolver {
                                                tx_engine.with_path_resolver(r.clone())
                                            } else {
                                                tx_engine
                                            };
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

        // MIG XML fallback: for any FV without a cached MIG schema, parse from XML
        let xml_base = std::path::Path::new("xml-migs-and-ahbs");
        if xml_base.is_dir() {
            if let Ok(fv_entries) = std::fs::read_dir(xml_base) {
                for fv_entry in fv_entries.flatten() {
                    let fv_path = fv_entry.path();
                    if !fv_path.is_dir() {
                        continue;
                    }
                    let fv = fv_entry.file_name().to_string_lossy().to_string();
                    if services.contains_key(&fv) {
                        continue; // Already loaded from cache
                    }
                    if let Ok(files) = std::fs::read_dir(&fv_path) {
                        for file in files.flatten() {
                            let name = file.file_name().to_string_lossy().to_string();
                            if name.starts_with("UTILMD_MIG_Strom_") && name.ends_with(".xml") {
                                match ConversionService::new(
                                    &file.path(),
                                    "UTILMD",
                                    Some("Strom"),
                                    &fv,
                                ) {
                                    Ok(svc) => {
                                        tracing::info!(
                                            "Loaded MIG from XML for {fv} (cache miss): {name}"
                                        );
                                        services.insert(fv.clone(), svc);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to load MIG for {fv}/{name}: {e}");
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Load AHB schemas for PID lookup — discover all *_AHB_*.xml files
        let mut ahb_schemas = HashMap::new();
        if xml_base.is_dir() {
            if let Ok(fv_entries) = std::fs::read_dir(xml_base) {
                for fv_entry in fv_entries.flatten() {
                    let fv_path = fv_entry.path();
                    if !fv_path.is_dir() {
                        continue;
                    }
                    let fv = fv_entry.file_name().to_string_lossy().to_string();
                    if let Ok(files) = std::fs::read_dir(&fv_path) {
                        for file in files.flatten() {
                            let fname = file.file_name().to_string_lossy().to_string();
                            if !fname.contains("_AHB_") || !fname.ends_with(".xml") {
                                continue;
                            }
                            // Parse filename: {MSG_TYPE}_AHB_{Variant}_... or {MSG_TYPE}_AHB_...
                            // e.g. "UTILMD_AHB_Strom_2_1_..." → msg_type=UTILMD, variant=Strom
                            // e.g. "ORDRSP_AHB_1_0a_..." → msg_type=ORDRSP, variant=None
                            let msg_type = fname.split("_AHB_").next().unwrap_or("");
                            if msg_type.is_empty() {
                                continue;
                            }
                            // Determine variant: check if there's a known variant name after _AHB_
                            let after_ahb = fname
                                .split("_AHB_")
                                .nth(1)
                                .unwrap_or("");
                            let variant = if after_ahb.starts_with("Strom") {
                                Some("Strom")
                            } else if after_ahb.starts_with("Gas") {
                                Some("Gas")
                            } else {
                                None
                            };
                            let key = match variant {
                                Some(v) => format!("{}/{}_{}", fv, msg_type, v),
                                None => format!("{}/{}", fv, msg_type),
                            };
                            // Skip if already loaded (first file wins per key)
                            if ahb_schemas.contains_key(&key) {
                                continue;
                            }
                            match parse_ahb(&file.path(), msg_type, variant, &fv) {
                                Ok(schema) => {
                                    tracing::info!(
                                        "Loaded AHB schema for {key} with {} workflows",
                                        schema.workflows.len()
                                    );
                                    ahb_schemas.insert(key, schema);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to load AHB from {}: {e}",
                                        file.path().display()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Load response message MIGs and mapping engines (APERAK, CONTRL)
        let mut response_engines = HashMap::new();
        let mut response_migs = HashMap::new();
        let response_configs: Vec<(&str, &str, &str)> = vec![
            ("FV2504", "APERAK", "APERAK_MIG_2_1i_20240619.xml"),
            (
                "FV2504",
                "CONTRL",
                "CONTRL_MIG_2_0b_außerordentliche_20240726.xml",
            ),
        ];
        for (fv, msg_type, mig_file) in &response_configs {
            let mig_path = std::path::Path::new("xml-migs-and-ahbs")
                .join(fv)
                .join(mig_file);
            if !mig_path.exists() {
                tracing::info!(
                    "Response MIG not found at {}, skipping {}/{}",
                    mig_path.display(),
                    fv,
                    msg_type
                );
                continue;
            }
            match parse_mig(&mig_path, msg_type, None, fv) {
                Ok(mig) => {
                    let key = format!("{}/{}", fv, msg_type);
                    tracing::info!("Loaded response MIG for {key}");
                    response_migs.insert(key.clone(), mig);

                    // Load mapping engine from mappings/{FV}/{MSG_TYPE}/
                    let mapping_dir = std::path::Path::new("mappings").join(fv).join(msg_type);
                    if mapping_dir.is_dir() {
                        let msg_type_lower = msg_type.to_lowercase();
                        let fv_lower = fv.to_lowercase();
                        let schema_dir = format!(
                            "crates/mig-types/src/generated/{}/{}/pids",
                            fv_lower, msg_type_lower
                        );
                        let resolver = if std::path::Path::new(&schema_dir).is_dir() {
                            PathResolver::from_schema_dir(std::path::Path::new(&schema_dir))
                        } else {
                            PathResolver::from_schema(&serde_json::json!({}))
                        };
                        match MappingEngine::load(&mapping_dir) {
                            Ok(engine) => {
                                let engine = engine.with_path_resolver(resolver);
                                tracing::info!(
                                    "Loaded {} response TOML mappings for {key}",
                                    engine.definitions().len()
                                );
                                response_engines.insert(key, engine);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to load response mappings from {}: {e}",
                                    mapping_dir.display()
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse response MIG from {}: {e}",
                        mig_path.display()
                    );
                }
            }
        }

        Self {
            services,
            mapping_engines,
            message_engines,
            transaction_engines,
            ahb_schemas,
            pid_to_variant,
            pid_segment_numbers,
            response_engines,
            response_migs,
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

    /// Resolve the message variant (e.g., `"UTILMD_Strom"`) for a given PID.
    ///
    /// Checks cached `pid_to_variant` first (populated from VariantCache),
    /// then falls back to scanning AHB schemas.
    pub fn resolve_variant(&self, fv: &str, pid: &str) -> Option<&str> {
        // Check cache first
        let cache_key = format!("{}/pid_{}", fv, pid);
        if let Some(variant) = self.pid_to_variant.get(&cache_key) {
            return Some(variant.as_str());
        }

        // Fallback: scan AHB schemas
        let prefix = format!("{}/", fv);
        for (key, schema) in &self.ahb_schemas {
            if let Some(variant) = key.strip_prefix(&prefix) {
                if schema.workflows.iter().any(|w| w.id == pid) {
                    return Some(variant);
                }
            }
        }
        None
    }

    /// Get cached AHB segment numbers for a specific PID.
    ///
    /// Key: "{fv}/{variant}/pid_{pid}". Returns None if not cached (caller should
    /// fall back to AHB schema lookup).
    pub fn segment_numbers_for_pid(
        &self,
        fv: &str,
        msg_variant: &str,
        pid: &str,
    ) -> Option<&Vec<String>> {
        let key = format!("{}/{}/pid_{}", fv, msg_variant, pid);
        self.pid_segment_numbers.get(&key)
    }

    /// Get a response mapping engine for a message type (e.g., APERAK, CONTRL).
    /// Key format: "FV2504/APERAK"
    pub fn response_engine(&self, fv: &str, msg_type: &str) -> Option<&MappingEngine> {
        let key = format!("{}/{}", fv, msg_type);
        self.response_engines.get(&key)
    }

    /// Get a response MIG schema for a message type (e.g., APERAK, CONTRL).
    /// Key format: "FV2504/APERAK"
    pub fn response_mig(&self, fv: &str, msg_type: &str) -> Option<&MigSchema> {
        let key = format!("{}/{}", fv, msg_type);
        self.response_migs.get(&key)
    }

    /// Build an AhbWorkflow for a specific PID from its PID schema JSON.
    ///
    /// Loads the schema from `crates/mig-types/src/generated/{fv}/{msg_type}/pids/pid_{pid}_schema.json`.
    /// Returns `None` if the schema file doesn't exist or can't be parsed.
    pub fn ahb_workflow_for_pid(
        &self,
        fv: &str,
        msg_variant: &str,
        pid: &str,
    ) -> Option<automapper_validation::AhbWorkflow> {
        let msg_type = msg_variant.split('_').next()?.to_lowercase();
        let fv_lower = fv.to_lowercase();
        let schema_path = format!(
            "crates/mig-types/src/generated/{}/{}/pids/pid_{}_schema.json",
            fv_lower, msg_type, pid
        );
        let schema_str = std::fs::read_to_string(&schema_path).ok()?;
        let schema: serde_json::Value = serde_json::from_str(&schema_str).ok()?;
        crate::validation_bridge::ahb_workflow_from_pid_schema(&schema)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_variant_finds_pid() {
        let mut ahb_schemas = HashMap::new();
        let schema = AhbSchema {
            message_type: "UTILMD".to_string(),
            variant: Some("Strom".to_string()),
            version: "2.1".to_string(),
            format_version: "FV2504".to_string(),
            source_file: String::new(),
            workflows: vec![automapper_generator::schema::ahb::Pruefidentifikator {
                id: "55001".to_string(),
                beschreibung: "Test workflow".to_string(),
                kommunikation_von: None,
                fields: vec![],
                segment_numbers: vec![],
            }],
            bedingungen: vec![],
        };
        ahb_schemas.insert("FV2504/UTILMD_Strom".to_string(), schema);

        let registry = MigServiceRegistry {
            services: HashMap::new(),
            mapping_engines: HashMap::new(),
            message_engines: HashMap::new(),
            transaction_engines: HashMap::new(),
            ahb_schemas,
            pid_to_variant: HashMap::new(),
            pid_segment_numbers: HashMap::new(),
            response_engines: HashMap::new(),
            response_migs: HashMap::new(),
        };

        // Falls back to AHB schema scan when pid_to_variant is empty
        assert_eq!(
            registry.resolve_variant("FV2504", "55001"),
            Some("UTILMD_Strom")
        );
        assert_eq!(registry.resolve_variant("FV2504", "99999"), None);
        assert_eq!(registry.resolve_variant("FV9999", "55001"), None);
    }

    #[test]
    fn test_resolve_variant_uses_cache_first() {
        let mut pid_to_variant = HashMap::new();
        pid_to_variant.insert(
            "FV2504/pid_19120".to_string(),
            "ORDRSP".to_string(),
        );

        let registry = MigServiceRegistry {
            services: HashMap::new(),
            mapping_engines: HashMap::new(),
            message_engines: HashMap::new(),
            transaction_engines: HashMap::new(),
            ahb_schemas: HashMap::new(),
            pid_to_variant,
            pid_segment_numbers: HashMap::new(),
            response_engines: HashMap::new(),
            response_migs: HashMap::new(),
        };

        // Resolves from cache without any AHB schemas loaded
        assert_eq!(
            registry.resolve_variant("FV2504", "19120"),
            Some("ORDRSP")
        );
        assert_eq!(registry.resolve_variant("FV2504", "99999"), None);
    }

    #[test]
    fn test_segment_numbers_for_pid() {
        let mut pid_segment_numbers = HashMap::new();
        pid_segment_numbers.insert(
            "FV2504/UTILMD_Strom/pid_55001".to_string(),
            vec!["0010".to_string(), "0020".to_string()],
        );

        let registry = MigServiceRegistry {
            services: HashMap::new(),
            mapping_engines: HashMap::new(),
            message_engines: HashMap::new(),
            transaction_engines: HashMap::new(),
            ahb_schemas: HashMap::new(),
            pid_to_variant: HashMap::new(),
            pid_segment_numbers,
            response_engines: HashMap::new(),
            response_migs: HashMap::new(),
        };

        let nums = registry.segment_numbers_for_pid("FV2504", "UTILMD_Strom", "55001");
        assert_eq!(nums, Some(&vec!["0010".to_string(), "0020".to_string()]));
        assert_eq!(
            registry.segment_numbers_for_pid("FV2504", "UTILMD_Strom", "99999"),
            None
        );
    }

    #[test]
    fn test_ahb_workflow_for_pid_from_schema() {
        // ahb_workflow_for_pid uses workspace-root-relative paths
        // Tests run from crate dir, so cd to workspace root
        let schema_check = std::path::Path::new(
            "crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json",
        );
        if !schema_check.exists() {
            // Try from workspace root (CI may run from there)
            let alt = std::path::Path::new(
                "../../crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json",
            );
            if alt.exists() {
                // We're in the crate dir — cd to workspace root for this test
                std::env::set_current_dir("../..").unwrap();
            } else {
                eprintln!("Skipping: schema not found");
                return;
            }
        }

        let registry = MigServiceRegistry {
            services: HashMap::new(),
            mapping_engines: HashMap::new(),
            message_engines: HashMap::new(),
            transaction_engines: HashMap::new(),
            ahb_schemas: HashMap::new(),
            pid_to_variant: HashMap::new(),
            pid_segment_numbers: HashMap::new(),
            response_engines: HashMap::new(),
            response_migs: HashMap::new(),
        };

        let workflow = registry.ahb_workflow_for_pid("FV2504", "UTILMD_Strom", "55001");
        assert!(workflow.is_some(), "Should build workflow from schema");
        let wf = workflow.unwrap();
        assert_eq!(wf.pruefidentifikator, "55001");
        assert!(!wf.fields.is_empty());

        // Non-existent PID returns None
        assert!(registry.ahb_workflow_for_pid("FV2504", "UTILMD_Strom", "99999").is_none());
    }
}
