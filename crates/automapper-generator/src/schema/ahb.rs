use serde::{Deserialize, Serialize};

/// Complete AHB schema for a message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbSchema {
    /// The EDIFACT message type (e.g., "UTILMD", "ORDERS").
    pub message_type: String,
    /// Optional variant (e.g., "Strom", "Gas").
    pub variant: Option<String>,
    /// Version number from the AHB (e.g., "2.1").
    pub version: String,
    /// Format version directory (e.g., "FV2510").
    pub format_version: String,
    /// Path to the source XML file.
    pub source_file: String,
    /// All AWF (Anwendungsfall) definitions with their PIDs.
    pub workflows: Vec<Pruefidentifikator>,
    /// All condition definitions from the Bedingungen section.
    pub bedingungen: Vec<BedingungDefinition>,
}

/// An Anwendungsfall (AWF/workflow) definition from an AHB.
/// Each AWF has a unique Pruefidentifikator (PID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pruefidentifikator {
    /// The unique PID (e.g., "55001", "55002").
    pub id: String,
    /// Description of the workflow.
    pub beschreibung: String,
    /// Communication direction (e.g., "NB an LF").
    pub kommunikation_von: Option<String>,
    /// All fields required/allowed for this PID.
    pub fields: Vec<AhbFieldDefinition>,
    /// MIG segment Number attributes referenced by this PID.
    /// Links AHB segments to their MIG counterparts for PID-specific assembly.
    #[serde(default)]
    pub segment_numbers: Vec<String>,
}

/// A field definition extracted from an AHB for a specific Pruefidentifikator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbFieldDefinition {
    /// Path to the field (e.g., "SG2/NAD/C082/3039").
    pub segment_path: String,
    /// Human-readable name of the field.
    pub name: String,
    /// Status in this AHB context (e.g., "X", "Muss", "Kann", "X [condition]").
    pub ahb_status: String,
    /// Optional description.
    pub description: Option<String>,
    /// Valid code values for this field (if restricted).
    pub codes: Vec<AhbCodeValue>,
    /// MIG Number of the parent S_* segment (links to MigSegment.number).
    #[serde(default)]
    pub mig_number: Option<String>,
    /// AHB status of the innermost parent group (e.g., "Kann", "Muss", "Soll [46]").
    ///
    /// This enables the validator to skip mandatory checks for fields inside
    /// optional groups whose qualifier variant is absent from the message.
    #[serde(default)]
    pub parent_group_ahb_status: Option<String>,
}

/// A valid code value for an AHB field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbCodeValue {
    /// The code value.
    pub value: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Status in this AHB context.
    pub ahb_status: Option<String>,
}

/// A condition definition from the Bedingungen section of an AHB XML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedingungDefinition {
    /// The condition ID (e.g., "931", "494").
    pub id: String,
    /// The German description text.
    pub description: String,
}

/// An AHB rule binding a condition expression to a segment/field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AhbRule {
    /// The segment path this rule applies to.
    pub segment_path: String,
    /// The raw condition expression (e.g., "[1] AND [2]", "Muss [494]").
    pub condition_expression: String,
    /// Whether the field is mandatory under this rule.
    pub is_mandatory: bool,
}

impl AhbFieldDefinition {
    /// Whether this field is mandatory (status is "Muss" or "X" without conditions).
    pub fn is_mandatory(&self) -> bool {
        self.ahb_status == "Muss" || self.ahb_status == "X"
    }

    /// Extract all condition IDs referenced in the AHB status string.
    /// Matches patterns like "[931]", "[494]", "[1] AND [2]".
    pub fn condition_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        let mut chars = self.ahb_status.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '[' {
                let id: String = chars.by_ref().take_while(|&c| c != ']').collect();
                if !id.is_empty() {
                    ids.push(id);
                }
            }
        }
        ids
    }
}
