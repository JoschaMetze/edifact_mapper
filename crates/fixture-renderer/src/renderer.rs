//! Core rendering functions for the fixture renderer.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use automapper_generator::fixture_renderer::types::{
    CanonicalBo4e, CanonicalMeta, NachrichtBo4e, TransaktionBo4e,
};
use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::{DisassembledSegment, Disassembler};
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::{parse_to_segments, split_messages};
use mig_bo4e::model::{
    extract_nachrichtendaten, extract_unh_fields, rebuild_unb, rebuild_unh, rebuild_unt,
    rebuild_unz, MappedMessage, Transaktion,
};
use mig_bo4e::path_resolver::PathResolver;
use mig_bo4e::pid_schema_index::PidSchemaIndex;
use mig_bo4e::MappingEngine;
use mig_types::schema::mig::MigSchema;

use crate::RendererError;

/// Input configuration for rendering a fixture.
pub struct RenderInput {
    pub mig_xml_path: PathBuf,
    pub ahb_xml_path: PathBuf,
    pub message_mappings_dir: PathBuf,
    pub transaction_mappings_dir: PathBuf,
    pub message_type: String,
    pub variant: Option<String>,
    pub format_version: String,
    pub pid: String,
}

/// Parse MIG/AHB XMLs and return a PID-filtered MIG schema.
fn load_filtered_mig(input: &RenderInput) -> Result<MigSchema, RendererError> {
    let mig = automapper_generator::parsing::mig_parser::parse_mig(
        &input.mig_xml_path,
        &input.message_type,
        input.variant.as_deref(),
        &input.format_version,
    )?;

    let ahb = automapper_generator::parsing::ahb_parser::parse_ahb(
        &input.ahb_xml_path,
        &input.message_type,
        input.variant.as_deref(),
        &input.format_version,
    )?;

    let pid_def = ahb
        .workflows
        .iter()
        .find(|w| w.id == input.pid)
        .ok_or_else(|| RendererError::PidNotFound {
            pid: input.pid.clone(),
        })?;

    let ahb_numbers: HashSet<String> = pid_def.segment_numbers.iter().cloned().collect();
    Ok(filter_mig_for_pid(&mig, &ahb_numbers))
}

/// Load split mapping engines with common/ inheritance and PathResolver.
fn load_engines(input: &RenderInput) -> Result<(MappingEngine, MappingEngine), RendererError> {
    let fv_lower = input.format_version.to_lowercase();
    let msg_type_lower = input.message_type.to_lowercase();
    let schema_dir_path = format!(
        "crates/mig-types/src/generated/{}/{}/pids",
        fv_lower, msg_type_lower
    );
    let common_dir = input
        .transaction_mappings_dir
        .parent()
        .map(|p| p.join("common"));

    let (msg_engine, tx_engine) = if let Some(ref cmn) = common_dir {
        if cmn.is_dir() {
            let schema_file =
                Path::new(&schema_dir_path).join(format!("pid_{}_schema.json", input.pid));
            if let Ok(schema_index) = PidSchemaIndex::from_schema_file(&schema_file) {
                MappingEngine::load_split_with_common(
                    &input.message_mappings_dir,
                    cmn,
                    &input.transaction_mappings_dir,
                    &schema_index,
                )
                .map_err(|e| RendererError::Mapping(e.to_string()))?
            } else {
                MappingEngine::load_split(
                    &input.message_mappings_dir,
                    &input.transaction_mappings_dir,
                )
                .map_err(|e| RendererError::Mapping(e.to_string()))?
            }
        } else {
            MappingEngine::load_split(&input.message_mappings_dir, &input.transaction_mappings_dir)
                .map_err(|e| RendererError::Mapping(e.to_string()))?
        }
    } else {
        MappingEngine::load_split(&input.message_mappings_dir, &input.transaction_mappings_dir)
            .map_err(|e| RendererError::Mapping(e.to_string()))?
    };

    // Apply PathResolver for EDIFACT ID path resolution
    if Path::new(&schema_dir_path).is_dir() {
        let resolver = PathResolver::from_schema_dir(Path::new(&schema_dir_path));
        Ok((
            msg_engine.with_path_resolver(resolver.clone()),
            tx_engine.with_path_resolver(resolver),
        ))
    } else {
        Ok((msg_engine, tx_engine))
    }
}

/// Render an EDIFACT fixture from a source .edi file through the full
/// forward -> reverse roundtrip pipeline.
///
/// Pipeline: .edi -> tokenize -> assemble -> forward map -> reverse map
///           -> disassemble -> render -> .edi
///
/// This validates that the TOML mappings can produce a complete EDIFACT message.
pub fn render_fixture(
    source_edi_path: &Path,
    input: &RenderInput,
) -> Result<String, RendererError> {
    let filtered_mig = load_filtered_mig(input)?;

    // Read and tokenize source EDIFACT
    let edi_bytes = std::fs::read(source_edi_path)?;
    let segments = parse_to_segments(&edi_bytes)?;
    let chunks = split_messages(segments)?;

    let (msg_engine, tx_engine) = load_engines(input)?;

    // Process each message through forward+reverse roundtrip
    let assembler = Assembler::new(&filtered_mig);
    let disassembler = Disassembler::new(&filtered_mig);
    let delimiters = edifact_types::EdifactDelimiters::default();

    let mut all_edifact_parts: Vec<String> = Vec::new();

    for msg_chunk in &chunks.messages {
        let tree = assembler.assemble_generic(&msg_chunk.body)?;

        // Forward map to BO4E
        let mapped = MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", true);

        // Reverse map back to tree
        let reverse_tree =
            MappingEngine::map_interchange_reverse(&msg_engine, &tx_engine, &mapped, "SG4", Some(&filtered_mig));

        // Disassemble tree -> ordered segments
        let dis_segments = disassembler.disassemble(&reverse_tree);

        // Build UNH + body + UNT
        let (unh_ref, nachrichten_typ) = extract_unh_fields(&msg_chunk.unh);
        let unh = rebuild_unh(&unh_ref, &nachrichten_typ);
        let unh_dis = DisassembledSegment {
            tag: unh.id.clone(),
            elements: unh.elements.clone(),
        };

        let seg_count = 1 + dis_segments.len() + 1; // UNH + body + UNT
        let unt = rebuild_unt(seg_count, &unh_ref);
        let unt_dis = DisassembledSegment {
            tag: unt.id.clone(),
            elements: unt.elements.clone(),
        };

        let mut msg_segments = vec![unh_dis];
        msg_segments.extend(dis_segments);
        msg_segments.push(unt_dis);

        all_edifact_parts.push(render_edifact(&msg_segments, &delimiters));
    }

    // Build envelope (UNA + UNB + messages + UNZ)
    let nachrichtendaten = extract_nachrichtendaten(&chunks.envelope);

    let una_str = delimiters.to_una_string();
    let unb = rebuild_unb(&nachrichtendaten);
    let unb_segments = vec![DisassembledSegment {
        tag: unb.id.clone(),
        elements: unb.elements.clone(),
    }];
    let unb_str = render_edifact(&unb_segments, &delimiters);

    let interchange_ref = nachrichtendaten
        .get("interchangeRef")
        .and_then(|v| v.as_str())
        .unwrap_or("00000");
    let unz = rebuild_unz(chunks.messages.len(), interchange_ref);
    let unz_segments = vec![DisassembledSegment {
        tag: unz.id.clone(),
        elements: unz.elements.clone(),
    }];
    let unz_str = render_edifact(&unz_segments, &delimiters);

    let mut result = una_str;
    result.push_str(&unb_str);
    for part in &all_edifact_parts {
        result.push_str(part);
    }
    result.push_str(&unz_str);

    Ok(result)
}

/// Generate a canonical `.mig.bo.json` from an existing `.edi` fixture.
///
/// Pipeline: .edi -> tokenize -> assemble -> forward map -> CanonicalBo4e JSON
///
/// This bootstraps the version-independent test corpus from existing fixtures.
pub fn generate_canonical_bo4e(
    source_edi_path: &Path,
    input: &RenderInput,
) -> Result<CanonicalBo4e, RendererError> {
    let filtered_mig = load_filtered_mig(input)?;

    // Tokenize and split
    let edi_bytes = std::fs::read(source_edi_path)?;
    let segments = parse_to_segments(&edi_bytes)?;
    let chunks = split_messages(segments)?;

    let nachrichtendaten = extract_nachrichtendaten(&chunks.envelope);

    let (msg_engine, tx_engine) = load_engines(input)?;

    // Process first message
    let msg = chunks.messages.first().ok_or(RendererError::NoMessages)?;
    let (unh_ref, nachrichten_typ) = extract_unh_fields(&msg.unh);

    let assembler = Assembler::new(&filtered_mig);
    let tree = assembler.assemble_generic(&msg.body)?;

    let mapped: MappedMessage =
        MappingEngine::map_interchange(&msg_engine, &tx_engine, &tree, "SG4", true);

    let canonical = CanonicalBo4e {
        meta: CanonicalMeta {
            pid: input.pid.clone(),
            message_type: input.message_type.clone(),
            source_format_version: input.format_version.clone(),
            source_fixture: source_edi_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        },
        nachrichtendaten,
        nachricht: NachrichtBo4e {
            unh_referenz: unh_ref,
            nachrichten_typ,
            stammdaten: mapped.stammdaten.clone(),
            transaktionen: mapped
                .transaktionen
                .iter()
                .map(|tx: &Transaktion| TransaktionBo4e {
                    stammdaten: tx.stammdaten.clone(),
                    transaktionsdaten: tx.transaktionsdaten.clone(),
                })
                .collect(),
        },
    };

    Ok(canonical)
}
