//! V2 validate-bo4e endpoint.
//!
//! Accepts BO4E JSON, reverse-maps to EDIFACT, validates against AHB rules,
//! and enriches validation errors with BO4E field paths.

use std::collections::HashSet;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use mig_assembly::assembler::Assembler;
use mig_assembly::disassembler::Disassembler;
use mig_assembly::navigator::AssembledTreeNavigator;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::renderer::render_edifact;
use mig_assembly::tokenize::parse_to_segments;

use crate::contracts::reverse_v2::{normalize_to_interchange, InputLevel};
use crate::contracts::validate_bo4e::{ValidateBo4eRequest, ValidateBo4eResponse};
use crate::error::ApiError;
use crate::state::AppState;

/// Build v2 validate-bo4e routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/validate-bo4e", post(validate_bo4e))
}

/// `POST /api/v2/validate-bo4e` — validate BO4E JSON via reverse mapping.
///
/// Chains: normalize → reverse-map → render EDIFACT → tokenize → assemble →
/// validate → enrich with BO4E field paths.
#[utoipa::path(
    post,
    path = "/api/v2/validate-bo4e",
    request_body = ValidateBo4eRequest,
    responses(
        (status = 200, description = "Validation report with BO4E paths", body = ValidateBo4eResponse),
        (status = 400, description = "Bad request"),
        (status = 422, description = "Conversion or validation error"),
    ),
    tag = "v2"
)]
pub(crate) async fn validate_bo4e(
    State(state): State<AppState>,
    Json(req): Json<ValidateBo4eRequest>,
) -> Result<Json<ValidateBo4eResponse>, ApiError> {
    let start = std::time::Instant::now();

    // Parse input and validation levels
    let input_level = match req.level.as_str() {
        "interchange" => InputLevel::Interchange,
        "nachricht" => InputLevel::Nachricht,
        _ => InputLevel::Transaktion,
    };
    let validation_level = match req.validation_level.as_str() {
        "structure" => automapper_validation::ValidationLevel::Structure,
        "conditions" => automapper_validation::ValidationLevel::Conditions,
        _ => automapper_validation::ValidationLevel::Full,
    };

    // Step 1: Normalize BO4E input to Interchange
    let interchange = normalize_to_interchange(&req.input, &input_level, req.envelope.as_ref())
        .map_err(|e| ApiError::BadRequest {
            message: format!("Input normalization error: {e}"),
        })?;

    // Step 2: Get MIG service
    let service = state
        .mig_registry
        .service(&req.format_version)
        .ok_or_else(|| ApiError::BadRequest {
            message: format!(
                "No MIG service available for format version '{}'",
                req.format_version
            ),
        })?;

    // TODO: detect message type/variant from nachrichtenTyp
    let msg_variant = "UTILMD_Strom";

    // Step 3: Process the first message
    let nachricht = interchange
        .nachrichten
        .first()
        .ok_or_else(|| ApiError::BadRequest {
            message: "No messages (nachrichten) in input".to_string(),
        })?;

    // Extract PID from first transaction
    let pid = nachricht
        .transaktionen
        .first()
        .and_then(|tx| tx.transaktionsdaten.get("pruefidentifikator"))
        .and_then(|v| v.as_str().or_else(|| v.get("code").and_then(|c| c.as_str())))
        .ok_or_else(|| ApiError::BadRequest {
            message: "No pruefidentifikator found in transaktionsdaten".to_string(),
        })?;

    // Step 4: Load AHB schema + workflow
    let ahb = state
        .mig_registry
        .ahb_schema(&req.format_version, msg_variant)
        .ok_or_else(|| ApiError::Internal {
            message: format!(
                "No AHB schema available for {}/{}",
                req.format_version, msg_variant
            ),
        })?;

    let ahb_workflow = ahb
        .workflows
        .iter()
        .find(|w| w.id == pid)
        .ok_or_else(|| ApiError::ConversionError {
            message: format!("PID {pid} not found in AHB"),
        })?;

    let ahb_numbers: HashSet<String> = ahb_workflow.segment_numbers.iter().cloned().collect();
    let filtered_mig = filter_mig_for_pid(service.mig(), &ahb_numbers);

    // Step 5: Load split mapping engines
    let (msg_engine, tx_engine) = state
        .mig_registry
        .mapping_engines_split(&req.format_version, msg_variant, pid)
        .ok_or_else(|| ApiError::Internal {
            message: format!(
                "No mapping engines for {}/{}/pid_{}",
                req.format_version, msg_variant, pid
            ),
        })?;

    // Step 6: Reverse map BO4E → AssembledTree
    let mapped = mig_bo4e::model::MappedMessage {
        stammdaten: nachricht.stammdaten.clone(),
        transaktionen: nachricht.transaktionen.clone(),
    };
    let tree =
        mig_bo4e::MappingEngine::map_interchange_reverse(msg_engine, tx_engine, &mapped, "SG4");

    // Step 7: Disassemble tree → EDIFACT segments → re-tokenize
    let disassembler = Disassembler::new(&filtered_mig);
    let dis_segments = disassembler.disassemble(&tree);

    // Build UNH + body + UNT for rendering
    let unh = mig_bo4e::model::rebuild_unh(&nachricht.unh_referenz, &nachricht.nachrichten_typ);
    let unh_dis = mig_assembly::disassembler::DisassembledSegment {
        tag: unh.id.clone(),
        elements: unh.elements.clone(),
    };
    let seg_count = 1 + dis_segments.len() + 1;
    let unt = mig_bo4e::model::rebuild_unt(seg_count, &nachricht.unh_referenz);
    let unt_dis = mig_assembly::disassembler::DisassembledSegment {
        tag: unt.id.clone(),
        elements: unt.elements.clone(),
    };

    let mut msg_segments = vec![unh_dis];
    msg_segments.extend(dis_segments);
    msg_segments.push(unt_dis);

    // Build full EDIFACT with envelope
    let delimiters = edifact_types::EdifactDelimiters::default();
    let una_str = delimiters.to_una_string();
    let unb = mig_bo4e::model::rebuild_unb(&interchange.nachrichtendaten);
    let unb_segments = vec![mig_assembly::disassembler::DisassembledSegment {
        tag: unb.id.clone(),
        elements: unb.elements.clone(),
    }];

    let interchange_ref = interchange
        .nachrichtendaten
        .get("interchangeRef")
        .and_then(|v| v.as_str())
        .unwrap_or("00000");
    let unz = mig_bo4e::model::rebuild_unz(1, interchange_ref);
    let unz_segments = vec![mig_assembly::disassembler::DisassembledSegment {
        tag: unz.id.clone(),
        elements: unz.elements.clone(),
    }];

    let mut full_edifact = una_str;
    full_edifact.push_str(&render_edifact(&unb_segments, &delimiters));
    full_edifact.push_str(&render_edifact(&msg_segments, &delimiters));
    full_edifact.push_str(&render_edifact(&unz_segments, &delimiters));

    // Step 8: Re-tokenize the rendered EDIFACT
    let segments =
        parse_to_segments(full_edifact.as_bytes()).map_err(|e| ApiError::ConversionError {
            message: format!("Re-tokenization error: {e}"),
        })?;

    // Split into messages
    let chunks = mig_assembly::split_messages(segments).map_err(|e| ApiError::ConversionError {
        message: format!("Message splitting error: {e}"),
    })?;

    let msg_chunk = chunks
        .messages
        .first()
        .ok_or_else(|| ApiError::ConversionError {
            message: "No messages found after re-tokenization".to_string(),
        })?;

    let all_segments = msg_chunk.all_segments();

    // Step 9: Assemble with diagnostics for validation
    let assembler = Assembler::new(&filtered_mig);
    let (assembled_tree, structure_diagnostics) =
        assembler.assemble_with_diagnostics(&all_segments);

    // Step 10: Build AhbWorkflow and validate
    let workflow =
        crate::validation_bridge::ahb_workflow_from_schema(ahb, pid).ok_or_else(|| {
            ApiError::ConversionError {
                message: format!("PID {pid} not found in AHB"),
            }
        })?;

    let external: Box<dyn automapper_validation::eval::ExternalConditionProvider> =
        if let Some(ref conditions) = req.external_conditions {
            Box::new(automapper_validation::MapExternalProvider::new(
                conditions.clone(),
            ))
        } else {
            Box::new(automapper_validation::eval::NoOpExternalProvider)
        };

    let evaluator = automapper_validation::UtilmdConditionEvaluatorFV2504::default();
    let validator = automapper_validation::EdifactValidator::new(evaluator);

    let navigator = AssembledTreeNavigator::new(&assembled_tree);
    let mut report = validator.validate_with_navigator(
        &all_segments,
        &workflow,
        external.as_ref(),
        validation_level,
        &navigator,
    );

    // Add structure diagnostics
    for diag in structure_diagnostics {
        report.add_issue(automapper_validation::ValidationIssue::new(
            automapper_validation::Severity::Warning,
            automapper_validation::ValidationCategory::Structure,
            automapper_validation::ErrorCodes::UNEXPECTED_SEGMENT,
            diag.message,
        ));
    }

    // Step 11: Build Bo4eFieldIndex and enrich report with BO4E paths
    let mut all_defs: Vec<_> = msg_engine.definitions().to_vec();
    all_defs.extend(tx_engine.definitions().iter().cloned());
    let field_index = mig_bo4e::Bo4eFieldIndex::build(&all_defs, &filtered_mig);
    report.enrich_bo4e_paths(|path| field_index.resolve(path));

    // Step 12: Serialize and return
    let report_json = serde_json::to_value(&report).map_err(|e| ApiError::Internal {
        message: format!("Failed to serialize validation report: {e}"),
    })?;

    Ok(Json(ValidateBo4eResponse {
        report: report_json,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}
