use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidSchemaDiff {
    pub old_version: String,
    pub new_version: String,
    pub message_type: String,
    pub pid: String,
    pub unh_version: Option<VersionChange>,
    pub segments: SegmentDiff,
    pub codes: CodeDiff,
    pub groups: GroupDiff,
    pub elements: ElementDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDiff {
    pub added: Vec<SegmentEntry>,
    pub removed: Vec<SegmentEntry>,
    pub unchanged: Vec<SegmentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEntry {
    pub group: String,
    pub tag: String,
    /// Human-readable context (e.g., "New metering segment in SG8_Z98")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiff {
    pub changed: Vec<CodeChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub segment: String,
    pub element: String,
    pub group: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDiff {
    pub added: Vec<GroupEntry>,
    pub removed: Vec<GroupEntry>,
    pub restructured: Vec<RestructuredGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupEntry {
    pub group: String,
    pub parent: String,
    /// Entry segment with qualifier, e.g., "SEQ+ZH5"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_segment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestructuredGroup {
    pub group: String,
    pub description: String,
    pub manual_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDiff {
    pub added: Vec<ElementChange>,
    pub removed: Vec<ElementChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementChange {
    pub segment: String,
    pub group: String,
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PidSchemaDiff {
    /// Returns true if the diff contains no changes.
    pub fn is_empty(&self) -> bool {
        self.segments.added.is_empty()
            && self.segments.removed.is_empty()
            && self.codes.changed.is_empty()
            && self.groups.added.is_empty()
            && self.groups.removed.is_empty()
            && self.groups.restructured.is_empty()
            && self.elements.added.is_empty()
            && self.elements.removed.is_empty()
    }
}
