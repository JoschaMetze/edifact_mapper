//! Shared helpers for the reverse mapping pipeline (BO4E → EDIFACT).
//!
//! Used by both `reverse_v2` and `validate_bo4e` routes.

use std::collections::HashSet;

use automapper_generator::schema::ahb::AhbSchema;
use automapper_generator::schema::mig::MigSchema;
use mig_assembly::disassembler::{DisassembledSegment, Disassembler};
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_bo4e::engine::MappingEngine;

use crate::error::ApiError;
use crate::state::AppState;

/// Resources loaded for a specific PID's reverse pipeline.
pub(crate) struct ReversePipelineContext<'a> {
    pub filtered_mig: MigSchema,
    pub msg_engine: &'a MappingEngine,
    pub tx_engine: &'a MappingEngine,
    pub ahb: &'a AhbSchema,
}

/// Load all MIG/AHB resources needed for reverse mapping a given PID.
pub(crate) fn load_reverse_context<'a>(
    state: &'a AppState,
    format_version: &str,
    msg_variant: &str,
    pid: &str,
) -> Result<ReversePipelineContext<'a>, ApiError> {
    let service =
        state
            .mig_registry
            .service(format_version)
            .ok_or_else(|| ApiError::BadRequest {
                message: format!("No MIG service available for format version '{format_version}'"),
            })?;

    let ahb = state
        .mig_registry
        .ahb_schema(format_version, msg_variant)
        .ok_or_else(|| ApiError::Internal {
            message: format!("No AHB schema available for {format_version}/{msg_variant}"),
        })?;

    let ahb_workflow =
        ahb.workflows
            .iter()
            .find(|w| w.id == pid)
            .ok_or_else(|| ApiError::ConversionError {
                message: format!("PID {pid} not found in AHB"),
            })?;

    let ahb_numbers: HashSet<String> = ahb_workflow.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);

    let (msg_engine, tx_engine) = state
        .mig_registry
        .mapping_engines_split(format_version, msg_variant, pid)
        .ok_or_else(|| ApiError::Internal {
            message: format!("No mapping engines for {format_version}/{msg_variant}/pid_{pid}"),
        })?;

    Ok(ReversePipelineContext {
        filtered_mig,
        msg_engine,
        tx_engine,
        ahb,
    })
}

/// Extract the pruefidentifikator from a Nachricht's first transaction.
pub(crate) fn extract_pid(nachricht: &mig_bo4e::Nachricht) -> Result<&str, ApiError> {
    nachricht
        .transaktionen
        .first()
        .and_then(|tx| tx.transaktionsdaten.get("pruefidentifikator"))
        .and_then(|v| {
            v.as_str()
                .or_else(|| v.get("code").and_then(|c| c.as_str()))
        })
        .ok_or_else(|| ApiError::BadRequest {
            message: "No pruefidentifikator found in transaktionsdaten".to_string(),
        })
}

/// Reverse-map a Nachricht to an AssembledTree using the loaded engines.
pub(crate) fn reverse_map_nachricht(
    ctx: &ReversePipelineContext,
    nachricht: &mig_bo4e::Nachricht,
) -> mig_assembly::assembler::AssembledTree {
    let mapped = mig_bo4e::model::MappedMessage {
        stammdaten: nachricht.stammdaten.clone(),
        transaktionen: nachricht.transaktionen.clone(),
    };
    MappingEngine::map_interchange_reverse(ctx.msg_engine, ctx.tx_engine, &mapped, "SG4")
}

/// Disassemble an AssembledTree and render as EDIFACT message segments (UNH + body + UNT).
pub(crate) fn render_message_segments(
    ctx: &ReversePipelineContext,
    nachricht: &mig_bo4e::Nachricht,
    tree: &mig_assembly::assembler::AssembledTree,
    delimiters: &edifact_types::EdifactDelimiters,
) -> String {
    let disassembler = Disassembler::new(&ctx.filtered_mig);
    let dis_segments = disassembler.disassemble(tree);

    let unh = mig_bo4e::model::rebuild_unh(&nachricht.unh_referenz, &nachricht.nachrichten_typ);
    let unh_dis = DisassembledSegment {
        tag: unh.id,
        elements: unh.elements,
    };

    let seg_count = 1 + dis_segments.len() + 1;
    let unt = mig_bo4e::model::rebuild_unt(seg_count, &nachricht.unh_referenz);
    let unt_dis = DisassembledSegment {
        tag: unt.id,
        elements: unt.elements,
    };

    let mut msg_segments = vec![unh_dis];
    msg_segments.extend(dis_segments);
    msg_segments.push(unt_dis);

    render_edifact(&msg_segments, delimiters)
}

/// Render a full EDIFACT interchange string with UNA/UNB/messages/UNZ envelope.
pub(crate) fn render_full_edifact(
    interchange: &mig_bo4e::Interchange,
    message_parts: &[String],
) -> String {
    let delimiters = edifact_types::EdifactDelimiters::default();
    let una_str = delimiters.to_una_string();

    let unb = mig_bo4e::model::rebuild_unb(&interchange.nachrichtendaten);
    let unb_segments = vec![DisassembledSegment {
        tag: unb.id,
        elements: unb.elements,
    }];

    let interchange_ref = interchange
        .nachrichtendaten
        .get("interchangeRef")
        .and_then(|v| v.as_str())
        .unwrap_or("00000");
    let unz = mig_bo4e::model::rebuild_unz(message_parts.len(), interchange_ref);
    let unz_segments = vec![DisassembledSegment {
        tag: unz.id,
        elements: unz.elements,
    }];

    let mut full_edifact = una_str;
    full_edifact.push_str(&render_edifact(&unb_segments, &delimiters));
    for part in message_parts {
        full_edifact.push_str(part);
    }
    full_edifact.push_str(&render_edifact(&unz_segments, &delimiters));

    full_edifact
}
