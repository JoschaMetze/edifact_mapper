use std::path::Path;

use crate::error::GeneratorError;
use crate::parsing::ahb_parser::parse_ahb;
use mig_assembly::assembler::Assembler;
use mig_assembly::parsing::parse_mig;
use mig_assembly::pid_filter::filter_mig_for_pid;
use mig_assembly::tokenize::parse_to_segments;
use std::collections::HashSet;

/// Result of fixture validation.
pub struct ValidationResult {
    /// Number of segments parsed from the EDIFACT.
    pub segment_count: usize,
    /// Number of segments captured in the assembled tree.
    pub assembled_segment_count: usize,
    /// Number of groups in the assembled tree.
    pub assembled_group_count: usize,
    /// Warnings (non-fatal issues).
    pub warnings: Vec<String>,
    /// Errors (fatal issues).
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate a generated EDIFACT fixture against MIG/AHB schemas.
///
/// Performs structural validation:
/// 1. Tokenize the EDIFACT string into segments
/// 2. Parse MIG XML and AHB XML
/// 3. Filter MIG for the specified PID
/// 4. Assemble the tree using the PID-filtered MIG
///
/// Returns a `ValidationResult` with segment counts and any issues found.
pub fn validate_fixture(
    edifact: &str,
    pid: &str,
    mig_xml_path: &Path,
    ahb_xml_path: &Path,
    message_type: &str,
    variant: Option<&str>,
    format_version: &str,
) -> Result<ValidationResult, GeneratorError> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // 1. Tokenize
    let segments =
        parse_to_segments(edifact.as_bytes()).map_err(|e| GeneratorError::Validation {
            message: format!("tokenization failed: {e}"),
        })?;
    let segment_count = segments.len();

    // 2. Parse MIG + AHB
    let mig = parse_mig(mig_xml_path, message_type, variant, format_version)?;
    let ahb = parse_ahb(ahb_xml_path, message_type, variant, format_version)?;

    // 3. Find PID in AHB and get segment numbers
    let pid_def =
        ahb.workflows
            .iter()
            .find(|w| w.id == pid)
            .ok_or_else(|| GeneratorError::Validation {
                message: format!("PID {pid} not found in AHB"),
            })?;

    let ahb_numbers: HashSet<String> = pid_def.segment_numbers.iter().cloned().collect();
    if ahb_numbers.is_empty() {
        warnings.push(format!(
            "PID {pid} has no AHB segment numbers — MIG filtering will be a no-op"
        ));
    }

    // 4. Filter MIG for PID
    let filtered_mig = filter_mig_for_pid(&mig, &ahb_numbers);

    // 5. Assemble tree
    // Skip UNB/UNZ (interchange envelope) — assembler expects message content only
    let message_segments: Vec<_> = segments
        .iter()
        .filter(|s| s.id != "UNB" && s.id != "UNZ")
        .cloned()
        .collect();

    let assembler = Assembler::new(&filtered_mig);
    match assembler.assemble_generic(&message_segments) {
        Ok(tree) => {
            let assembled_segment_count = count_tree_segments(&tree);
            let assembled_group_count = tree.groups.len();

            // Check for uncaptured segments
            let total_message_segs = message_segments.len();
            if assembled_segment_count < total_message_segs {
                let uncaptured = total_message_segs - assembled_segment_count;
                warnings.push(format!(
                    "{uncaptured} of {total_message_segs} message segments not captured in tree \
                     (may be expected for transport segments like UNH/UNT)"
                ));
            }

            Ok(ValidationResult {
                segment_count,
                assembled_segment_count,
                assembled_group_count,
                warnings,
                errors,
            })
        }
        Err(e) => {
            errors.push(format!("assembly failed: {e}"));
            Ok(ValidationResult {
                segment_count,
                assembled_segment_count: 0,
                assembled_group_count: 0,
                warnings,
                errors,
            })
        }
    }
}

/// Count total segments in an assembled tree (root + nested in groups).
fn count_tree_segments(tree: &mig_assembly::assembler::AssembledTree) -> usize {
    let mut count = tree.segments.len();
    for group in &tree.groups {
        for instance in &group.repetitions {
            count += count_group_segments(instance);
        }
    }
    count
}

fn count_group_segments(instance: &mig_assembly::assembler::AssembledGroupInstance) -> usize {
    let mut count = instance.segments.len();
    for group in &instance.child_groups {
        for nested in &group.repetitions {
            count += count_group_segments(nested);
        }
    }
    count
}
