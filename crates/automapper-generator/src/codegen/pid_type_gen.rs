//! Code generation for PID-specific composition types.
//!
//! Cross-references AHB field definitions against the MIG tree
//! to determine which segment groups, segments, and fields
//! exist for each PID.

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::ahb::Pruefidentifikator;
use crate::schema::mig::MigSchema;

/// Analyzed structure of a single PID.
#[derive(Debug)]
pub struct PidStructure {
    pub pid_id: String,
    pub beschreibung: String,
    pub kommunikation_von: Option<String>,
    /// Top-level groups present in this PID.
    pub groups: Vec<PidGroupInfo>,
    /// Top-level segments (outside groups) present in this PID.
    pub top_level_segments: Vec<String>,
}

/// Information about a segment group's usage within a PID.
#[derive(Debug)]
pub struct PidGroupInfo {
    pub group_id: String,
    /// Qualifier values that disambiguate this group (e.g., SEQ+Z01).
    /// Empty if the group is not qualifier-disambiguated.
    pub qualifier_values: Vec<String>,
    /// AHB status for this group occurrence ("Muss", "Kann", etc.)
    pub ahb_status: String,
    /// Nested child groups present in this PID's usage.
    pub child_groups: Vec<PidGroupInfo>,
    /// Segments present in this group for this PID.
    pub segments: BTreeSet<String>,
}

/// Analyze which MIG tree nodes a PID uses, based on its AHB field definitions.
pub fn analyze_pid_structure(pid: &Pruefidentifikator, _mig: &MigSchema) -> PidStructure {
    let mut top_level_segments: BTreeSet<String> = BTreeSet::new();
    let mut group_map: BTreeMap<String, PidGroupInfo> = BTreeMap::new();

    for field in &pid.fields {
        let parts: Vec<&str> = field.segment_path.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        // Determine if path starts with a group (SGn) or a segment
        if parts[0].starts_with("SG") {
            let group_id = parts[0].to_string();
            let entry = group_map.entry(group_id.clone()).or_insert_with(|| PidGroupInfo {
                group_id: group_id.clone(),
                qualifier_values: Vec::new(),
                ahb_status: field.ahb_status.clone(),
                child_groups: Vec::new(),
                segments: BTreeSet::new(),
            });

            // Find the first non-SG part as the segment within this group
            if parts.len() > 1 && !parts[1].starts_with("SG") {
                entry.segments.insert(parts[1].to_string());
            }

            // Handle nested groups (SG4/SG8/...)
            if parts.len() > 1 && parts[1].starts_with("SG") {
                let child_id = parts[1].to_string();
                if !entry.child_groups.iter().any(|c| c.group_id == child_id) {
                    let mut child_segments = BTreeSet::new();
                    if parts.len() > 2 && !parts[2].starts_with("SG") {
                        child_segments.insert(parts[2].to_string());
                    }
                    entry.child_groups.push(PidGroupInfo {
                        group_id: child_id,
                        qualifier_values: Vec::new(),
                        ahb_status: field.ahb_status.clone(),
                        child_groups: Vec::new(),
                        segments: child_segments,
                    });
                } else if parts.len() > 2 && !parts[2].starts_with("SG") {
                    if let Some(child) =
                        entry.child_groups.iter_mut().find(|c| c.group_id == child_id)
                    {
                        child.segments.insert(parts[2].to_string());
                    }
                }
            }
        } else {
            // Top-level segment (not in a group)
            top_level_segments.insert(parts[0].to_string());
        }
    }

    PidStructure {
        pid_id: pid.id.clone(),
        beschreibung: pid.beschreibung.clone(),
        kommunikation_von: pid.kommunikation_von.clone(),
        groups: group_map.into_values().collect(),
        top_level_segments: top_level_segments.into_iter().collect(),
    }
}
