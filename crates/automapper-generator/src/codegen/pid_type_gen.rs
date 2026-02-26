//! Code generation for PID-specific composition types.
//!
//! Cross-references AHB field definitions against the MIG tree
//! to determine which segment groups, segments, and fields
//! exist for each PID.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::schema::ahb::{AhbSchema, Pruefidentifikator};
use crate::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

// ---------------------------------------------------------------------------
// Qualifier Code Name Parsing
// ---------------------------------------------------------------------------

/// Metadata derived from a MIG/AHB qualifier code name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifierMeta {
    /// The qualifier code (e.g., "ZF0", "Z98").
    pub code: String,
    /// BO4E entity name (e.g., "Marktlokation", "TechnischeRessource").
    pub entity: String,
    /// Data quality/context: "informativ", "erwartet", "im_system", "differenz", or "base".
    pub data_quality: String,
    /// Content type (e.g., "Daten", "OBIS-Daten", "Produkt-Daten", "Profildaten").
    pub content_type: String,
    /// Original code name for reference.
    pub mig_name: String,
}

/// Bidirectional qualifier lookup map.
///
/// Separates SEQ (D_1229) and LOC (D_3227) code namespaces since the same
/// code value can have different meanings in different data elements.
///
/// Forward: (element_type, code) → QualifierMeta
/// Reverse: (entity, data_quality, content_type) → code
#[derive(Debug, Clone)]
pub struct QualifierMap {
    /// SEQ D_1229 qualifier codes (SG8 discrimination).
    seq_codes: HashMap<String, QualifierMeta>,
    /// LOC D_3227 qualifier codes (SG5 discrimination).
    loc_codes: HashMap<String, QualifierMeta>,
    /// Reverse lookup for SEQ codes: (entity, quality, content) → code.
    by_entity: HashMap<(String, String, String), String>,
}

impl QualifierMap {
    /// Build a qualifier map from MIG and AHB sources.
    ///
    /// Walks MIG `D_1229` (SEQ) and `D_3227` (LOC) code definitions,
    /// then enriches with AHB codes (which is the superset, especially for
    /// quality-prefixed variants like "Informative", "Erwartete", etc.).
    pub fn from_mig_and_ahb(mig: &MigSchema, ahb: &AhbSchema) -> Self {
        let mut seq_codes: HashMap<String, QualifierMeta> = HashMap::new();
        let mut loc_codes: HashMap<String, QualifierMeta> = HashMap::new();

        // 1) Walk MIG D_1229 codes (SEQ qualifiers)
        for group in &mig.segment_groups {
            collect_qualifier_codes_from_group(group, "1229", &mut seq_codes);
        }
        // Walk MIG D_3227 codes (LOC qualifiers for SG5)
        for group in &mig.segment_groups {
            collect_qualifier_codes_from_group(group, "3227", &mut loc_codes);
        }

        // 2) Walk AHB field codes for D_1229 and D_3227 (superset)
        for pid in &ahb.workflows {
            for field in &pid.fields {
                let is_seq = field.segment_path.ends_with("SEQ/1229")
                    || field.segment_path.ends_with("/1229");
                let is_loc = field.segment_path.ends_with("LOC/3227")
                    || field.segment_path.ends_with("/3227");
                if !is_seq && !is_loc {
                    continue;
                }
                let target = if is_seq {
                    &mut seq_codes
                } else {
                    &mut loc_codes
                };
                for code in &field.codes {
                    if target.contains_key(&code.value) {
                        continue;
                    }
                    let name = if !code.name.is_empty() {
                        &code.name
                    } else if let Some(ref desc) = code.description {
                        desc
                    } else {
                        continue;
                    };
                    if let Some(mut meta) = parse_qualifier_code_name(name) {
                        meta.code = code.value.clone();
                        target.insert(code.value.clone(), meta);
                    }
                }
            }
        }

        // Build reverse map from SEQ codes (primary for SG8)
        let mut by_entity = HashMap::new();
        for (code, meta) in &seq_codes {
            let key = (
                meta.entity.clone(),
                meta.data_quality.clone(),
                meta.content_type.clone(),
            );
            by_entity.entry(key).or_insert_with(|| code.clone());
        }

        Self {
            seq_codes,
            loc_codes,
            by_entity,
        }
    }

    /// Look up metadata for a qualifier code.
    ///
    /// Checks SEQ (D_1229) codes first, then LOC (D_3227) codes.
    pub fn get(&self, code: &str) -> Option<&QualifierMeta> {
        self.seq_codes
            .get(code)
            .or_else(|| self.loc_codes.get(code))
    }

    /// Look up metadata specifically from SEQ D_1229 codes.
    pub fn get_seq(&self, code: &str) -> Option<&QualifierMeta> {
        self.seq_codes.get(code)
    }

    /// Look up metadata specifically from LOC D_3227 codes.
    pub fn get_loc(&self, code: &str) -> Option<&QualifierMeta> {
        self.loc_codes.get(code)
    }

    /// Reverse lookup: find the SEQ qualifier code for a given entity, data quality, and content type.
    pub fn reverse_lookup(
        &self,
        entity: &str,
        data_quality: &str,
        content_type: &str,
    ) -> Option<&str> {
        self.by_entity
            .get(&(
                entity.to_string(),
                data_quality.to_string(),
                content_type.to_string(),
            ))
            .map(|s| s.as_str())
    }
}

/// Recursively collect qualifier codes from a MIG segment group.
fn collect_qualifier_codes_from_group(
    group: &MigSegmentGroup,
    target_de_id: &str,
    by_code: &mut HashMap<String, QualifierMeta>,
) {
    for seg in &group.segments {
        // Check direct data elements
        for de in &seg.data_elements {
            if de.id == target_de_id {
                for code_def in &de.codes {
                    if by_code.contains_key(&code_def.value) {
                        continue;
                    }
                    let name = code_def
                        .description
                        .as_deref()
                        .filter(|d| !d.is_empty())
                        .unwrap_or(&code_def.name);
                    if let Some(mut meta) = parse_qualifier_code_name(name) {
                        meta.code = code_def.value.clone();
                        by_code.insert(code_def.value.clone(), meta);
                    }
                }
            }
        }
        // Check composite data elements
        for comp in &seg.composites {
            for de in &comp.data_elements {
                if de.id == target_de_id {
                    for code_def in &de.codes {
                        if by_code.contains_key(&code_def.value) {
                            continue;
                        }
                        let name = code_def
                            .description
                            .as_deref()
                            .filter(|d| !d.is_empty())
                            .unwrap_or(&code_def.name);
                        if let Some(mut meta) = parse_qualifier_code_name(name) {
                            meta.code = code_def.value.clone();
                            by_code.insert(code_def.value.clone(), meta);
                        }
                    }
                }
            }
        }
    }
    for nested in &group.nested_groups {
        collect_qualifier_codes_from_group(nested, target_de_id, by_code);
    }
}

/// Parse a MIG/AHB qualifier code name into entity, data quality, and content type.
///
/// Handles patterns like:
/// - "Daten der Marktlokation" → entity=Marktlokation, quality=base, content=Daten
/// - "Informative Daten der Technischen Ressource" → entity=TechnischeRessource, quality=informativ
/// - "Erwartete OBIS-Daten der Zähleinrichtung" → entity=Zaehler, quality=erwartet
/// - "Im System vorhandene Daten der Marktlokation" → entity=Marktlokation, quality=im_system
/// - "Differenz-Netznutzungsabrechnungsdaten der Marktlokation" → entity=Marktlokation, quality=differenz
pub fn parse_qualifier_code_name(name: &str) -> Option<QualifierMeta> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    // Try static fallback first for non-standard names
    if let Some(meta) = static_qualifier_fallback(name) {
        return Some(meta);
    }

    // 1) Strip quality prefix
    let (data_quality, remainder) = strip_quality_prefix(name);

    // 2) Split on " der " or " des " (last occurrence) to get content_type and entity_raw
    let (content_type, entity_raw) = split_on_entity_marker(remainder)?;

    // 3) Normalize entity name to BO4E type
    let entity = normalize_entity_name(entity_raw)?;

    Some(QualifierMeta {
        code: String::new(), // Filled in by caller
        entity,
        data_quality: data_quality.to_string(),
        content_type: content_type.to_string(),
        mig_name: name.to_string(),
    })
}

/// Strip a quality prefix from a code name, returning (quality, remainder).
fn strip_quality_prefix(name: &str) -> (&str, &str) {
    if let Some(rest) = name.strip_prefix("Informative ") {
        ("informativ", rest)
    } else if let Some(rest) = name.strip_prefix("Informatives ") {
        ("informativ", rest)
    } else if let Some(rest) = name.strip_prefix("Erwartete ") {
        ("erwartet", rest)
    } else if let Some(rest) = name.strip_prefix("Erwartetes ") {
        ("erwartet", rest)
    } else if let Some(rest) = name.strip_prefix("Im System vorhandene ") {
        ("im_system", rest)
    } else if let Some(rest) = name.strip_prefix("Im System vorhandenes ") {
        ("im_system", rest)
    } else if let Some(rest) = name.strip_prefix("Differenz-") {
        ("differenz", rest)
    } else if let Some(rest) = name.strip_prefix("Abgerechnete ") {
        ("abgerechnet", rest)
    } else {
        ("base", name)
    }
}

/// Split a content+entity string on " der " or " des " (first occurrence).
///
/// Returns (content_type, entity_raw) or None if no marker found.
///
/// Uses the **first** occurrence because compound entity names like
/// "Kunden des Lieferanten" should stay intact — "Daten des Kunden des Lieferanten"
/// should split into ("Daten", "Kunden des Lieferanten").
fn split_on_entity_marker(s: &str) -> Option<(&str, &str)> {
    // Try " der " first occurrence
    if let Some(pos) = s.find(" der ") {
        return Some((&s[..pos], &s[pos + 5..]));
    }
    // Try " des " first occurrence
    if let Some(pos) = s.find(" des ") {
        return Some((&s[..pos], &s[pos + 5..]));
    }
    None
}

/// Normalize a German entity name from MIG/AHB to a BO4E type name.
fn normalize_entity_name(raw: &str) -> Option<String> {
    let raw = raw.trim();
    // Exact match first
    match raw {
        "Marktlokation" => return Some("Marktlokation".to_string()),
        "Messlokation" => return Some("Messlokation".to_string()),
        "Netzlokation" => return Some("Netzlokation".to_string()),
        "NeLo" => return Some("Netzlokation".to_string()),
        "Tranche" => return Some("Tranche".to_string()),
        "Summenzeitreihe" => return Some("Summenzeitreihe".to_string()),
        "Überführungszeitreihe" => return Some("Ueberfuehrungszeitreihe".to_string()),
        "Ruhende Marktlokation" => return Some("RuhendeMarktlokation".to_string()),
        // German declension forms
        "Technischen Ressource" => return Some("TechnischeRessource".to_string()),
        "Technische Ressource" => return Some("TechnischeRessource".to_string()),
        "Steuerbaren Ressource" => return Some("SteuerbareRessource".to_string()),
        "Steuerbare Ressource" => return Some("SteuerbareRessource".to_string()),
        // Equipment
        "Zähleinrichtung" => return Some("Zaehler".to_string()),
        "Zähleinrichtung / Smartmeter-Gateway" => return Some("Zaehler".to_string()),
        "Steuerbox" => return Some("Steuerbox".to_string()),
        // People/partners
        "Kunden des Lieferanten" | "Kunden" => return Some("Geschaeftspartner".to_string()),
        // Other
        "ÜNB" => return Some("Datenstand".to_string()),
        "NB" => return Some("Datenstand".to_string()),
        "Lokationsbündelstruktur" => return Some("Lokationsbuendel".to_string()),
        _ => {}
    }

    // Partial matches for compound names
    if raw.contains("Marktlokation") {
        return Some("Marktlokation".to_string());
    }
    if raw.contains("Messlokation") {
        return Some("Messlokation".to_string());
    }
    if raw.contains("Netzlokation") || raw.contains("NeLo") {
        return Some("Netzlokation".to_string());
    }
    if raw.contains("Tranche") {
        return Some("Tranche".to_string());
    }
    if raw.contains("Technische") || raw.contains("Technischen") {
        return Some("TechnischeRessource".to_string());
    }
    if raw.contains("Steuerbare") || raw.contains("Steuerbaren") {
        return Some("SteuerbareRessource".to_string());
    }

    // Unknown entity — return as-is with spaces removed
    Some(raw.replace(' ', ""))
}

/// Static fallback map for codes whose names don't follow the standard pattern.
///
/// Also handles quality-prefixed variants of static names (e.g., "Informative Zähleinrichtungsdaten").
fn static_qualifier_fallback(name: &str) -> Option<QualifierMeta> {
    // Try exact match first (base quality)
    if let Some(meta) = static_base_match(name) {
        return Some(meta);
    }

    // Try stripping quality prefix and matching the remainder
    let (quality, remainder) = strip_quality_prefix(name);
    if quality != "base" {
        if let Some(mut meta) = static_base_match(remainder) {
            meta.data_quality = quality.to_string();
            meta.mig_name = name.to_string();
            return Some(meta);
        }
    }

    None
}

/// Match a base (no quality prefix) name to a static entity.
fn static_base_match(name: &str) -> Option<QualifierMeta> {
    let (entity, content_type) = match name {
        // SG5 LOC qualifiers — simple entity names
        "Marktlokation" => ("Marktlokation", "Standort"),
        "Messlokation" => ("Messlokation", "Standort"),
        "Netzlokation" => ("Netzlokation", "Standort"),
        "Steuerbare Ressource" => ("SteuerbareRessource", "Standort"),
        "Technische Ressource" => ("TechnischeRessource", "Standort"),
        "Tranche" => ("Tranche", "Standort"),
        "Ruhende Marktlokation" => ("RuhendeMarktlokation", "Standort"),
        // Non-standard SG8 names — no " der "/" des " marker
        "Zähleinrichtungsdaten" => ("Zaehler", "Daten"),
        "Wandlerdaten" => ("Zaehler", "Wandlerdaten"),
        "Profilschardaten" => ("Profil", "Profilschardaten"),
        "Profildaten" => ("Profil", "Profildaten"),
        "Referenzprofildaten" => ("Profil", "Referenzprofildaten"),
        "Smartmeter-Gateway" => ("SmartmeterGateway", "Daten"),
        "Steuerbox" => ("Steuerbox", "Daten"),
        "Kommunikationseinrichtungsdaten" => ("Kommunikationseinrichtung", "Daten"),
        "Bestandteil eines Produktpakets" => ("Produktpaket", "Bestandteil"),
        "Priorisierung erforderliches Produktpaket" => ("Produktpaket", "Priorisierung"),
        "Meldepunkt" => ("Meldepunkt", "Standort"),
        // Lokationsbündel variants
        "Referenz auf die Lokationsbündelstruktur" => ("Lokationsbuendel", "Referenz"),
        "Zuordnung Lokation zum Objektcode des Lokationsbündels" => {
            ("Lokationsbuendel", "Zuordnung")
        }
        _ => return None,
    };
    Some(QualifierMeta {
        code: String::new(),
        entity: entity.to_string(),
        data_quality: "base".to_string(),
        content_type: content_type.to_string(),
        mig_name: name.to_string(),
    })
}

/// Analyzed structure of a single PID.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct PidGroupInfo {
    pub group_id: String,
    /// Qualifier values that disambiguate this group (e.g., "ZD5", "ZD6").
    /// Empty if the group is not qualifier-disambiguated.
    pub qualifier_values: Vec<String>,
    /// AHB status for this group occurrence ("Muss", "Kann", etc.)
    pub ahb_status: String,
    /// Human-readable AHB-derived field name (e.g., "Absender", "Summenzeitreihe Arbeit/Leistung").
    pub ahb_name: Option<String>,
    /// Trigger segment + data element for qualifier discrimination.
    /// E.g., ("NAD", "3035") for SG2, ("SEQ", "1229") for SG8.
    pub discriminator: Option<(String, String)>,
    /// Canonical BO4E entity name derived from MIG/AHB qualifier code name.
    pub entity_hint: Option<String>,
    /// Data quality context derived from MIG/AHB qualifier code name.
    /// Values: "base", "informativ", "erwartet", "im_system", "differenz", "abgerechnet".
    pub data_quality_hint: Option<String>,
    /// Nested child groups present in this PID's usage.
    pub child_groups: Vec<PidGroupInfo>,
    /// Segments present in this group for this PID.
    pub segments: BTreeSet<String>,
    /// MIG Number for each segment (seg_id → Number). Used for direct MIG segment lookup.
    pub segment_mig_numbers: BTreeMap<String, String>,
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

        if parts[0].starts_with("SG") {
            let group_id = parts[0].to_string();
            let entry = group_map
                .entry(group_id.clone())
                .or_insert_with(|| PidGroupInfo {
                    group_id: group_id.clone(),
                    qualifier_values: Vec::new(),
                    ahb_status: field.ahb_status.clone(),
                    ahb_name: None,
                    discriminator: None,
                    entity_hint: None,
                    data_quality_hint: None,
                    child_groups: Vec::new(),
                    segments: BTreeSet::new(),
                    segment_mig_numbers: BTreeMap::new(),
                });

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
                        ahb_name: None,
                        discriminator: None,
                        entity_hint: None,
                        data_quality_hint: None,
                        child_groups: Vec::new(),
                        segments: child_segments,
                        segment_mig_numbers: BTreeMap::new(),
                    });
                } else if parts.len() > 2 && !parts[2].starts_with("SG") {
                    if let Some(child) = entry
                        .child_groups
                        .iter_mut()
                        .find(|c| c.group_id == child_id)
                    {
                        child.segments.insert(parts[2].to_string());
                    }
                }
            }
        } else {
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

/// Find the trigger segment and qualifying data element for a group from the MIG.
///
/// Returns `(segment_id, data_element_id)` — e.g., `("SEQ", "1229")` for SG8.
fn find_group_qualifier(group_id: &str, mig: &MigSchema) -> Option<(String, String)> {
    fn find_in_group(target_id: &str, group: &MigSegmentGroup) -> Option<(String, String)> {
        if group.id == target_id {
            if let Some(seg) = group.segments.first() {
                for de in &seg.data_elements {
                    if !de.codes.is_empty() {
                        return Some((seg.id.clone(), de.id.clone()));
                    }
                }
                for comp in &seg.composites {
                    for de in &comp.data_elements {
                        if !de.codes.is_empty() {
                            return Some((seg.id.clone(), de.id.clone()));
                        }
                    }
                }
            }
            return None;
        }
        for nested in &group.nested_groups {
            if let Some(result) = find_in_group(target_id, nested) {
                return Some(result);
            }
        }
        None
    }

    for group in &mig.segment_groups {
        if let Some(result) = find_in_group(group_id, group) {
            return Some(result);
        }
    }
    None
}

/// Derive the AHB field name for a group from its entry segment's AHB definition.
///
/// Looks for the AHB field that references this group's entry segment path
/// (e.g., "SG2/NAD" for SG2, "SG4/SG8/SEQ" for SG8 under SG4).
fn derive_ahb_name(
    pid: &Pruefidentifikator,
    group_path: &str,
    entry_segment: &str,
) -> Option<String> {
    let target_prefix = format!("{}/{}", group_path, entry_segment);
    pid.fields
        .iter()
        .find(|f| f.segment_path.starts_with(&target_prefix))
        .and_then(|f| {
            let name = f.name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
}

/// Populate discriminator, ahb_name, entity_hint, and data_quality_hint
/// on top-level groups and their children.
fn enrich_group_info(
    group: &mut PidGroupInfo,
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    parent_path: &str,
    qualifier_map: &QualifierMap,
) {
    let group_path = if parent_path.is_empty() {
        group.group_id.clone()
    } else {
        format!("{}/{}", parent_path, group.group_id)
    };

    // Set discriminator from MIG qualifier detection
    if let Some((seg, de)) = find_group_qualifier(&group.group_id, mig) {
        group.discriminator = Some((seg.clone(), de));

        // Derive AHB name from the trigger segment's first field definition
        if group.ahb_name.is_none() {
            group.ahb_name = derive_ahb_name(pid, &group_path, &seg);
        }
    }

    // Derive entity_hint and data_quality_hint from qualifier codes.
    // Use the appropriate namespace based on discriminator segment type.
    if group.entity_hint.is_none() {
        let is_loc = group
            .discriminator
            .as_ref()
            .is_some_and(|(seg, _)| seg == "LOC");
        for q in &group.qualifier_values {
            let meta = if is_loc {
                qualifier_map.get_loc(q)
            } else {
                qualifier_map.get_seq(q)
            };
            if let Some(meta) = meta {
                group.entity_hint = Some(meta.entity.clone());
                group.data_quality_hint = Some(meta.data_quality.clone());
                break;
            }
        }
    }

    // Recursively enrich child groups — children inherit parent's hints
    for child in &mut group.child_groups {
        if child.entity_hint.is_none() {
            child.entity_hint.clone_from(&group.entity_hint);
        }
        if child.data_quality_hint.is_none() {
            child.data_quality_hint.clone_from(&group.data_quality_hint);
        }
        enrich_group_info(child, pid, mig, &group_path, qualifier_map);
    }
}

/// Enhanced analysis that detects qualifier disambiguation for repeated segment groups.
///
/// When the same group (e.g., SG8) appears multiple times under a parent with different
/// qualifying values (e.g., SEQ+ZD5 vs SEQ+ZD6), this function splits them into separate
/// `PidGroupInfo` entries with their respective `qualifier_values`.
pub fn analyze_pid_structure_with_qualifiers(
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    ahb: &AhbSchema,
) -> PidStructure {
    let qualifier_map = QualifierMap::from_mig_and_ahb(mig, ahb);
    let mut top_level_segments: BTreeSet<String> = BTreeSet::new();
    let mut group_map: BTreeMap<String, GroupOccurrenceTracker> = BTreeMap::new();

    for field in &pid.fields {
        let parts: Vec<&str> = field.segment_path.split('/').collect();
        if parts.is_empty() {
            continue;
        }

        if parts[0].starts_with("SG") {
            let group_id = parts[0].to_string();
            let tracker = group_map
                .entry(group_id.clone())
                .or_insert_with(|| GroupOccurrenceTracker::new(group_id.clone()));

            if parts.len() > 1 && !parts[1].starts_with("SG") {
                tracker.add_segment(parts[1], field.mig_number.as_deref());
            }

            // Handle nested groups with qualifier tracking
            if parts.len() > 1 && parts[1].starts_with("SG") {
                let child_id = parts[1].to_string();

                // Check if this field is a qualifying field for the child group
                if let Some((trigger_seg, trigger_de)) = find_group_qualifier(&child_id, mig) {
                    // Build the full qualifying path: SG4/SG8/SEQ/1229
                    let qual_suffix = format!("{}/{}", trigger_seg, trigger_de);
                    let remaining: String = parts[2..].join("/");

                    if remaining == qual_suffix && !field.codes.is_empty() {
                        // This is a qualifying field — extract code values
                        let codes: Vec<String> = field
                            .codes
                            .iter()
                            .filter(|c| c.ahb_status.as_deref().is_some_and(|s| s.contains('X')))
                            .map(|c| c.value.clone())
                            .collect();

                        if !codes.is_empty() {
                            tracker.start_child_occurrence(
                                &child_id,
                                codes,
                                field.ahb_status.clone(),
                            );
                            // Also add the trigger segment
                            tracker.add_child_segment(
                                &child_id,
                                &trigger_seg,
                                field.mig_number.as_deref(),
                            );
                            continue;
                        }
                    }
                }

                // Check for group-level field (path is just "SG4/SG8" — marks occurrence start)
                if parts.len() == 2 {
                    tracker.mark_child_occurrence_boundary(&child_id, field.ahb_status.clone());
                    continue;
                }

                // Regular child group field — add segment to current occurrence
                if parts.len() > 2 && !parts[2].starts_with("SG") {
                    tracker.add_child_segment(&child_id, parts[2], field.mig_number.as_deref());
                }

                // Handle deeply nested groups (SG4/SG8/SG10/...)
                if parts.len() > 2 && parts[2].starts_with("SG") {
                    tracker.add_child_nested_group(&child_id, parts[2]);
                    if parts.len() > 3 && !parts[3].starts_with("SG") {
                        tracker.add_child_nested_segment(
                            &child_id,
                            parts[2],
                            parts[3],
                            field.mig_number.as_deref(),
                        );
                    }
                }
            }
        } else {
            top_level_segments.insert(parts[0].to_string());
        }
    }

    let mut groups: Vec<PidGroupInfo> = group_map
        .into_values()
        .map(|t| t.into_group_info())
        .collect();

    // Enrich all groups with discriminator and AHB name info
    for group in &mut groups {
        enrich_group_info(group, pid, mig, "", &qualifier_map);
    }

    PidStructure {
        pid_id: pid.id.clone(),
        beschreibung: pid.beschreibung.clone(),
        kommunikation_von: pid.kommunikation_von.clone(),
        groups,
        top_level_segments: top_level_segments.into_iter().collect(),
    }
}

/// Tracks multiple occurrences of child groups under a parent.
struct GroupOccurrenceTracker {
    group_id: String,
    segments: BTreeSet<String>,
    segment_numbers: BTreeMap<String, String>,
    child_trackers: BTreeMap<String, ChildGroupTracker>,
}

struct ChildGroupTracker {
    group_id: String,
    /// Each occurrence is (qualifier_values, ahb_status, segments, nested_groups)
    occurrences: Vec<ChildOccurrence>,
}

struct ChildOccurrence {
    qualifier_values: Vec<String>,
    ahb_status: String,
    segments: BTreeSet<String>,
    segment_numbers: BTreeMap<String, String>,
    nested_groups: BTreeMap<String, BTreeSet<String>>,
    nested_segment_numbers: BTreeMap<String, BTreeMap<String, String>>,
}

impl GroupOccurrenceTracker {
    fn new(group_id: String) -> Self {
        Self {
            group_id,
            segments: BTreeSet::new(),
            segment_numbers: BTreeMap::new(),
            child_trackers: BTreeMap::new(),
        }
    }

    fn add_segment(&mut self, seg_id: &str, mig_number: Option<&str>) {
        self.segments.insert(seg_id.to_string());
        if let Some(num) = mig_number {
            self.segment_numbers
                .insert(seg_id.to_string(), num.to_string());
        }
    }

    fn ensure_child(&mut self, child_id: &str) -> &mut ChildGroupTracker {
        self.child_trackers
            .entry(child_id.to_string())
            .or_insert_with(|| ChildGroupTracker {
                group_id: child_id.to_string(),
                occurrences: Vec::new(),
            })
    }

    fn start_child_occurrence(
        &mut self,
        child_id: &str,
        qualifier_values: Vec<String>,
        ahb_status: String,
    ) {
        let tracker = self.ensure_child(child_id);
        tracker.occurrences.push(ChildOccurrence {
            qualifier_values,
            ahb_status,
            segments: BTreeSet::new(),
            segment_numbers: BTreeMap::new(),
            nested_groups: BTreeMap::new(),
            nested_segment_numbers: BTreeMap::new(),
        });
    }

    fn mark_child_occurrence_boundary(&mut self, child_id: &str, ahb_status: String) {
        let tracker = self.ensure_child(child_id);
        // If no occurrences yet, or the last occurrence already has qualifier values,
        // this group-level field starts a new (potentially qualifier-less) occurrence
        if tracker.occurrences.is_empty()
            || !tracker
                .occurrences
                .last()
                .unwrap()
                .qualifier_values
                .is_empty()
        {
            tracker.occurrences.push(ChildOccurrence {
                qualifier_values: Vec::new(),
                ahb_status,
                segments: BTreeSet::new(),
                segment_numbers: BTreeMap::new(),
                nested_groups: BTreeMap::new(),
                nested_segment_numbers: BTreeMap::new(),
            });
        }
    }

    fn add_child_segment(&mut self, child_id: &str, seg_id: &str, mig_number: Option<&str>) {
        let tracker = self.ensure_child(child_id);
        if let Some(occ) = tracker.occurrences.last_mut() {
            occ.segments.insert(seg_id.to_string());
            if let Some(num) = mig_number {
                occ.segment_numbers
                    .insert(seg_id.to_string(), num.to_string());
            }
        } else {
            // No occurrence started yet — create a default one
            let mut segment_numbers = BTreeMap::new();
            if let Some(num) = mig_number {
                segment_numbers.insert(seg_id.to_string(), num.to_string());
            }
            tracker.occurrences.push(ChildOccurrence {
                qualifier_values: Vec::new(),
                ahb_status: String::new(),
                segments: BTreeSet::from([seg_id.to_string()]),
                segment_numbers,
                nested_groups: BTreeMap::new(),
                nested_segment_numbers: BTreeMap::new(),
            });
        }
    }

    fn add_child_nested_group(&mut self, child_id: &str, nested_id: &str) {
        let tracker = self.ensure_child(child_id);
        if let Some(occ) = tracker.occurrences.last_mut() {
            occ.nested_groups.entry(nested_id.to_string()).or_default();
        }
    }

    fn add_child_nested_segment(
        &mut self,
        child_id: &str,
        nested_id: &str,
        seg_id: &str,
        mig_number: Option<&str>,
    ) {
        let tracker = self.ensure_child(child_id);
        if let Some(occ) = tracker.occurrences.last_mut() {
            occ.nested_groups
                .entry(nested_id.to_string())
                .or_default()
                .insert(seg_id.to_string());
            if let Some(num) = mig_number {
                occ.nested_segment_numbers
                    .entry(nested_id.to_string())
                    .or_default()
                    .insert(seg_id.to_string(), num.to_string());
            }
        }
    }

    fn into_group_info(self) -> PidGroupInfo {
        let mut child_groups = Vec::new();

        for (_child_id, tracker) in self.child_trackers {
            if tracker.occurrences.len() <= 1 {
                // Single occurrence — merge into one PidGroupInfo
                let occ = tracker.occurrences.into_iter().next();
                let (
                    qualifier_values,
                    ahb_status,
                    segments,
                    segment_numbers,
                    nested,
                    nested_numbers,
                ) = match occ {
                    Some(o) => (
                        o.qualifier_values,
                        o.ahb_status,
                        o.segments,
                        o.segment_numbers,
                        o.nested_groups,
                        o.nested_segment_numbers,
                    ),
                    None => (
                        Vec::new(),
                        String::new(),
                        BTreeSet::new(),
                        BTreeMap::new(),
                        BTreeMap::new(),
                        BTreeMap::new(),
                    ),
                };

                child_groups.push(PidGroupInfo {
                    group_id: tracker.group_id,
                    qualifier_values,
                    ahb_status,
                    ahb_name: None,
                    discriminator: None,
                    entity_hint: None,
                    data_quality_hint: None,
                    child_groups: nested
                        .into_iter()
                        .map(|(nid, segs)| {
                            let nums = nested_numbers.get(&nid).cloned().unwrap_or_default();
                            PidGroupInfo {
                                group_id: nid,
                                qualifier_values: Vec::new(),
                                ahb_status: String::new(),
                                ahb_name: None,
                                discriminator: None,
                                entity_hint: None,
                                data_quality_hint: None,
                                child_groups: Vec::new(),
                                segments: segs,
                                segment_mig_numbers: nums,
                            }
                        })
                        .collect(),
                    segments,
                    segment_mig_numbers: segment_numbers,
                });
            } else {
                // Multiple occurrences — create separate entries
                for occ in tracker.occurrences {
                    child_groups.push(PidGroupInfo {
                        group_id: tracker.group_id.clone(),
                        qualifier_values: occ.qualifier_values,
                        ahb_status: occ.ahb_status,
                        ahb_name: None,
                        discriminator: None,
                        entity_hint: None,
                        data_quality_hint: None,
                        child_groups: occ
                            .nested_groups
                            .into_iter()
                            .map(|(nid, segs)| {
                                let nums = occ
                                    .nested_segment_numbers
                                    .get(&nid)
                                    .cloned()
                                    .unwrap_or_default();
                                PidGroupInfo {
                                    group_id: nid,
                                    qualifier_values: Vec::new(),
                                    ahb_status: String::new(),
                                    ahb_name: None,
                                    discriminator: None,
                                    entity_hint: None,
                                    data_quality_hint: None,
                                    child_groups: Vec::new(),
                                    segments: segs,
                                    segment_mig_numbers: nums,
                                }
                            })
                            .collect(),
                        segments: occ.segments,
                        segment_mig_numbers: occ.segment_numbers,
                    });
                }
            }
        }

        PidGroupInfo {
            group_id: self.group_id,
            qualifier_values: Vec::new(),
            ahb_status: String::new(),
            ahb_name: None,
            discriminator: None,
            entity_hint: None,
            data_quality_hint: None,
            child_groups,
            segments: self.segments,
            segment_mig_numbers: self.segment_numbers,
        }
    }
}

// ---------------------------------------------------------------------------
// Code Generation: PID Structs
// ---------------------------------------------------------------------------

/// Sanitize text for use in a `///` doc comment — collapse newlines and trim.
fn sanitize_doc(s: &str) -> String {
    s.replace('\r', "")
        .replace('\n', " ")
        .replace("  ", " ")
        .trim()
        .to_string()
}

/// Generate a Rust struct source for a specific PID that composes PID-specific wrapper types.
pub fn generate_pid_struct(pid: &Pruefidentifikator, mig: &MigSchema, ahb: &AhbSchema) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let mut out = String::new();

    let struct_name = format!("Pid{}", pid.id);

    // First, emit wrapper structs for all groups (deduplicated)
    emit_wrapper_structs(&struct_name, &structure.groups, &mut out);

    // Then emit the main PID struct
    out.push_str(&format!(
        "/// PID {}: {}\n",
        pid.id,
        sanitize_doc(&pid.beschreibung)
    ));
    if let Some(ref komm) = pid.kommunikation_von {
        out.push_str(&format!("/// Kommunikation: {}\n", sanitize_doc(komm)));
    }
    out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {struct_name} {{\n"));

    // Top-level segments — mandatory, stored as OwnedSegment
    for seg_id in &structure.top_level_segments {
        let field_name = seg_id.to_lowercase();
        out.push_str(&format!("    pub {field_name}: OwnedSegment,\n"));
    }

    // Groups with wrapper type names
    for group in &structure.groups {
        emit_pid_group_field_v2(&struct_name, group, &mut out, "    ");
    }

    out.push_str("}\n");

    out
}

/// Capitalize a segment ID for struct naming: "NAD" -> "Nad", "UNH" -> "Unh"
fn capitalize_segment_id(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let rest: String = chars.map(|c| c.to_ascii_lowercase()).collect();
            format!("{}{}", first.to_ascii_uppercase(), rest)
        }
    }
}

/// Make a Rust field name for a group (snake_case).
pub fn make_wrapper_field_name(group: &PidGroupInfo) -> String {
    if group.qualifier_values.is_empty() {
        group.group_id.to_lowercase()
    } else {
        format!(
            "{}_{}",
            group.group_id.to_lowercase(),
            group.qualifier_values.join("_").to_lowercase()
        )
    }
}

/// Make a Rust wrapper type name for a group (PascalCase).
fn make_wrapper_type_name(pid_struct_name: &str, group: &PidGroupInfo) -> String {
    let suffix = capitalize_segment_id(&group.group_id);
    if group.qualifier_values.is_empty() {
        format!("{pid_struct_name}{suffix}")
    } else {
        let qual_suffix: String = group
            .qualifier_values
            .iter()
            .map(|v| capitalize_segment_id(v))
            .collect::<Vec<_>>()
            .join("");
        format!("{pid_struct_name}{suffix}{qual_suffix}")
    }
}

/// Check if a group is an empty boundary marker (no segments, no qualifiers, no useful children).
fn is_empty_group(group: &PidGroupInfo) -> bool {
    group.segments.is_empty()
        && group.qualifier_values.is_empty()
        && group.child_groups.iter().all(is_empty_group)
}

/// Collected wrapper struct definition: name → (doc_comment, segments, child_field_lines).
/// Used to deduplicate child types (e.g., SG10) that appear under multiple parent instances.
struct WrapperDef {
    doc: String,
    segments: BTreeSet<String>,
    child_fields: Vec<String>,
}

/// Collect all wrapper struct definitions recursively, merging duplicates.
fn collect_wrapper_defs(
    pid_struct_name: &str,
    group: &PidGroupInfo,
    defs: &mut BTreeMap<String, WrapperDef>,
) {
    if is_empty_group(group) {
        return;
    }

    // Collect children first (depth-first)
    for child in &group.child_groups {
        collect_wrapper_defs(pid_struct_name, child, defs);
    }

    let wrapper_name = make_wrapper_type_name(pid_struct_name, group);

    // Build doc comment
    let mut doc = String::new();
    if let Some(ref name) = group.ahb_name {
        doc.push_str(&format!(
            "/// {} — {}\n",
            group.group_id,
            sanitize_doc(name)
        ));
    } else {
        doc.push_str(&format!("/// {}\n", group.group_id));
    }
    if !group.qualifier_values.is_empty() {
        doc.push_str(&format!(
            "/// Qualifiers: {}\n",
            group.qualifier_values.join(", ")
        ));
    }

    // Build child field lines (deduplicated — same qualifier group can appear multiple times)
    let mut child_fields = Vec::new();
    let mut seen_child_fields = BTreeSet::new();
    for child in &group.child_groups {
        if is_empty_group(child) {
            continue;
        }
        let child_type = make_wrapper_type_name(pid_struct_name, child);
        let child_field = make_wrapper_field_name(child);
        let line = format!("    pub {child_field}: Vec<{child_type}>,");
        if seen_child_fields.insert(line.clone()) {
            child_fields.push(line);
        }
    }

    // Merge with existing definition (union of segments + child fields)
    if let Some(existing) = defs.get_mut(&wrapper_name) {
        existing.segments.extend(group.segments.iter().cloned());
        for field in &child_fields {
            if !existing.child_fields.contains(field) {
                existing.child_fields.push(field.clone());
            }
        }
    } else {
        defs.insert(
            wrapper_name,
            WrapperDef {
                doc,
                segments: group.segments.clone(),
                child_fields,
            },
        );
    }
}

/// Emit all wrapper structs (deduplicated) for the PID's groups.
fn emit_wrapper_structs(pid_struct_name: &str, groups: &[PidGroupInfo], out: &mut String) {
    let mut defs: BTreeMap<String, WrapperDef> = BTreeMap::new();
    for group in groups {
        collect_wrapper_defs(pid_struct_name, group, &mut defs);
    }

    for (name, def) in &defs {
        out.push_str(&def.doc);
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {name} {{\n"));
        for seg_id in &def.segments {
            let field_name = seg_id.to_lowercase();
            // Use OwnedSegment for segment fields — keeps raw data, avoids typed Seg* dependency
            out.push_str(&format!("    pub {field_name}: Option<OwnedSegment>,\n"));
        }
        for field_line in &def.child_fields {
            out.push_str(field_line);
            out.push('\n');
        }
        out.push_str("}\n\n");
    }
}

/// Emit a field in the containing struct that references the wrapper type.
fn emit_pid_group_field_v2(
    pid_struct_name: &str,
    group: &PidGroupInfo,
    out: &mut String,
    indent: &str,
) {
    if is_empty_group(group) {
        return;
    }
    let wrapper_type = make_wrapper_type_name(pid_struct_name, group);
    let field_name = make_wrapper_field_name(group);

    // Groups can repeat per MIG
    out.push_str(&format!("{indent}pub {field_name}: Vec<{wrapper_type}>,\n"));
}

// ---------------------------------------------------------------------------
// Assembly Code Generation: from_segments()
// ---------------------------------------------------------------------------

/// Generate a `from_segments()` impl block for a PID type.
///
/// The generated code walks segments using `SegmentCursor` and populates
/// typed fields with qualifier discrimination.
pub fn generate_pid_from_segments(
    pid: &Pruefidentifikator,
    mig: &MigSchema,
    ahb: &AhbSchema,
) -> String {
    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);
    let struct_name = format!("Pid{}", pid.id);
    let mut out = String::new();

    // Generate from_segments() for each wrapper struct
    let mut defs: BTreeMap<String, WrapperDef> = BTreeMap::new();
    for group in &structure.groups {
        collect_wrapper_defs(&struct_name, group, &mut defs);
    }

    for (wrapper_name, def) in &defs {
        out.push_str(&format!("impl {wrapper_name} {{\n"));
        out.push_str("    /// Try to assemble this group from segments at the cursor position.\n");
        out.push_str("    pub fn from_segments(\n");
        out.push_str("        segments: &[OwnedSegment],\n");
        out.push_str("        cursor: &mut SegmentCursor,\n");
        out.push_str("    ) -> Option<Self> {\n");
        out.push_str("        let saved = cursor.save();\n");

        // Consume optional segments
        for seg_id in &def.segments {
            let field = seg_id.to_lowercase();
            out.push_str(&format!(
                "        let {field} = if peek_is(segments, cursor, \"{seg_id}\") {{\n"
            ));
            out.push_str("            Some(consume(segments, cursor)?.clone())\n");
            out.push_str("        } else {\n");
            out.push_str("            None\n");
            out.push_str("        };\n");
        }

        // Check if any segment was matched (at least one non-None)
        if !def.segments.is_empty() {
            let checks: Vec<String> = def
                .segments
                .iter()
                .map(|s| format!("{}.is_none()", s.to_lowercase()))
                .collect();
            out.push_str(&format!("        if {} {{\n", checks.join(" && ")));
            out.push_str("            cursor.restore(saved);\n");
            out.push_str("            return None;\n");
            out.push_str("        }\n");
        }

        // Collect child groups
        for child_line in &def.child_fields {
            // Extract field name and type from the line: "    pub field_name: Vec<TypeName>,"
            if let Some((field, type_name)) = parse_child_field_line(child_line) {
                out.push_str(&format!("        let mut {field} = Vec::new();\n"));
                out.push_str(&format!(
                    "        while let Some(item) = {type_name}::from_segments(segments, cursor) {{\n"
                ));
                out.push_str(&format!("            {field}.push(item);\n"));
                out.push_str("        }\n");
            }
        }

        // Build result
        out.push_str("        Some(Self {\n");
        for seg_id in &def.segments {
            let field = seg_id.to_lowercase();
            out.push_str(&format!("            {field},\n"));
        }
        for child_line in &def.child_fields {
            if let Some((field, _)) = parse_child_field_line(child_line) {
                out.push_str(&format!("            {field},\n"));
            }
        }
        out.push_str("        })\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");
    }

    // Generate from_segments() for the main PID struct
    out.push_str(&format!("impl {struct_name} {{\n"));
    out.push_str("    /// Assemble this PID from a pre-tokenized segment list.\n");
    out.push_str("    pub fn from_segments(\n");
    out.push_str("        segments: &[OwnedSegment],\n");
    out.push_str("    ) -> Result<Self, SegmentNotFound> {\n");
    out.push_str("        let mut cursor = SegmentCursor::new(segments.len());\n\n");

    // Top-level segments (mandatory — error if missing)
    for seg_id in &structure.top_level_segments {
        let field = seg_id.to_lowercase();
        out.push_str(&format!(
            "        let {field} = expect_segment(segments, &mut cursor, \"{seg_id}\")?.clone();\n"
        ));
    }

    // Top-level groups
    for group in &structure.groups {
        if is_empty_group(group) {
            continue;
        }
        let field = make_wrapper_field_name(group);
        let type_name = make_wrapper_type_name(&struct_name, group);
        out.push_str(&format!("        let mut {field} = Vec::new();\n"));
        out.push_str(&format!(
            "        while let Some(item) = {type_name}::from_segments(segments, &mut cursor) {{\n"
        ));
        out.push_str(&format!("            {field}.push(item);\n"));
        out.push_str("        }\n");
    }

    // Build result
    out.push_str(&format!("\n        Ok({struct_name} {{\n"));
    for seg_id in &structure.top_level_segments {
        let field = seg_id.to_lowercase();
        out.push_str(&format!("            {field},\n"));
    }
    for group in &structure.groups {
        if is_empty_group(group) {
            continue;
        }
        let field = make_wrapper_field_name(group);
        out.push_str(&format!("            {field},\n"));
    }
    out.push_str("        })\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

/// Parse a child field line like "    pub sg8_z79: Vec<Pid55001Sg8Z79>," into (field, type).
fn parse_child_field_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim().trim_end_matches(',');
    // Format: "pub field_name: Vec<TypeName>"
    let after_pub = trimmed.strip_prefix("pub ")?;
    let colon_pos = after_pub.find(':')?;
    let field = after_pub[..colon_pos].trim().to_string();
    let type_part = after_pub[colon_pos + 1..].trim();
    // Extract type from "Vec<TypeName>"
    let inner = type_part.strip_prefix("Vec<")?.strip_suffix('>')?;
    Some((field, inner.to_string()))
}

// ---------------------------------------------------------------------------
// JSON Schema Generation
// ---------------------------------------------------------------------------

/// Generate a JSON schema describing a PID's structure for runtime use.
pub fn generate_pid_schema(pid: &Pruefidentifikator, mig: &MigSchema, ahb: &AhbSchema) -> String {
    use crate::schema::ahb::AhbCodeValue;

    let structure = analyze_pid_structure_with_qualifiers(pid, mig, ahb);

    // Build direct Number → MigSegment index for O(1) lookups
    let number_index = build_mig_number_index(mig);

    // Build AHB code index: (mig_number, data_element_id, occurrence) → AHB-filtered codes.
    // AHB field paths look like "SG4/SG5/LOC/3227" or "SG2/NAD/C082/3039".
    // The last path component is always the data element ID.
    // A segment can contain the same DE ID multiple times (e.g., STS has three C556/D_9013
    // composites). We use an occurrence counter to distinguish them.
    let mut ahb_codes: HashMap<(String, String, usize), Vec<AhbCodeValue>> = HashMap::new();
    let mut de_occurrence: HashMap<(String, String), usize> = HashMap::new();
    for field in &pid.fields {
        if field.codes.is_empty() {
            continue;
        }
        let Some(ref mig_num) = field.mig_number else {
            continue;
        };
        // Extract the last path component as the data element ID
        if let Some(de_id) = field.segment_path.rsplit('/').next() {
            let occ_key = (mig_num.clone(), de_id.to_string());
            let occurrence = de_occurrence.entry(occ_key).or_insert(0);
            ahb_codes.insert(
                (mig_num.clone(), de_id.to_string(), *occurrence),
                field.codes.clone(),
            );
            *occurrence += 1;
        }
    }

    let mut root = serde_json::Map::new();
    root.insert("pid".to_string(), serde_json::Value::String(pid.id.clone()));
    root.insert(
        "beschreibung".to_string(),
        serde_json::Value::String(pid.beschreibung.clone()),
    );
    root.insert(
        "format_version".to_string(),
        serde_json::Value::String(ahb.format_version.clone()),
    );

    if let Some(ref komm) = pid.kommunikation_von {
        root.insert(
            "kommunikation_von".to_string(),
            serde_json::Value::String(komm.clone()),
        );
    }

    let mut fields = serde_json::Map::new();
    for group in &structure.groups {
        if is_empty_group(group) {
            continue;
        }
        let field_name = make_wrapper_field_name(group);
        fields.insert(
            field_name,
            group_to_schema_value(group, &number_index, mig, &ahb_codes),
        );
    }
    root.insert("fields".to_string(), serde_json::Value::Object(fields));

    // Root-level segments (BGM, DTM, etc.) — outside any group.
    // Emitted under "root_segments" so CodeLookup can enrich message-header fields.
    let root_segments: Vec<_> = structure
        .top_level_segments
        .iter()
        .filter_map(|seg_id| {
            mig.segments
                .iter()
                .find(|s| s.id.eq_ignore_ascii_case(seg_id))
        })
        .map(|seg| segment_to_schema_value(seg, None, &ahb_codes))
        .collect();
    if !root_segments.is_empty() {
        root.insert(
            "root_segments".to_string(),
            serde_json::Value::Array(root_segments),
        );
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap()
}

/// Build an index from MIG segment Number → &MigSegment for direct lookups.
pub fn build_mig_number_index(mig: &MigSchema) -> HashMap<String, &MigSegment> {
    let mut index = HashMap::new();
    for seg in &mig.segments {
        if let Some(ref num) = seg.number {
            index.insert(num.clone(), seg);
        }
    }
    fn walk_groups<'a>(groups: &'a [MigSegmentGroup], index: &mut HashMap<String, &'a MigSegment>) {
        for g in groups {
            for seg in &g.segments {
                if let Some(ref num) = seg.number {
                    index.insert(num.clone(), seg);
                }
            }
            walk_groups(&g.nested_groups, index);
        }
    }
    walk_groups(&mig.segment_groups, &mut index);
    index
}

fn group_to_schema_value(
    group: &PidGroupInfo,
    number_index: &HashMap<String, &MigSegment>,
    mig: &MigSchema,
    ahb_codes: &HashMap<(String, String, usize), Vec<crate::schema::ahb::AhbCodeValue>>,
) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "source_group".to_string(),
        serde_json::Value::String(group.group_id.clone()),
    );

    if let Some(ref hint) = group.entity_hint {
        obj.insert(
            "entity_hint".to_string(),
            serde_json::Value::String(hint.clone()),
        );
    }
    if let Some(ref quality) = group.data_quality_hint {
        obj.insert(
            "data_quality_hint".to_string(),
            serde_json::Value::String(quality.clone()),
        );
    }

    if let Some(ref disc) = group.discriminator {
        let mut d = serde_json::Map::new();
        d.insert(
            "segment".to_string(),
            serde_json::Value::String(disc.0.clone()),
        );
        d.insert(
            "element".to_string(),
            serde_json::Value::String(disc.1.clone()),
        );
        if !group.qualifier_values.is_empty() {
            d.insert(
                "values".to_string(),
                serde_json::Value::Array(
                    group
                        .qualifier_values
                        .iter()
                        .map(|v| serde_json::Value::String(v.clone()))
                        .collect(),
                ),
            );
        }
        obj.insert("discriminator".to_string(), serde_json::Value::Object(d));
    } else {
        obj.insert("discriminator".to_string(), serde_json::Value::Null);
    }

    let segments: Vec<_> = group
        .segments
        .iter()
        .map(|seg_id| {
            let mig_number = group.segment_mig_numbers.get(seg_id);

            // Direct Number-based lookup (precise), fall back to group search
            let mig_seg = mig_number
                .and_then(|num| number_index.get(num).copied())
                .or_else(|| find_segment_in_mig(seg_id, &group.group_id, mig));

            if let Some(mig_seg) = mig_seg {
                segment_to_schema_value(mig_seg, mig_number, ahb_codes)
            } else {
                let mut s = serde_json::Map::new();
                s.insert("id".to_string(), serde_json::Value::String(seg_id.clone()));
                serde_json::Value::Object(s)
            }
        })
        .collect();
    obj.insert("segments".to_string(), serde_json::Value::Array(segments));

    let non_empty_children: Vec<_> = group
        .child_groups
        .iter()
        .filter(|c| !is_empty_group(c))
        .collect();
    if !non_empty_children.is_empty() {
        let mut children = serde_json::Map::new();
        for child in non_empty_children {
            let name = make_wrapper_field_name(child);
            children.insert(
                name,
                group_to_schema_value(child, number_index, mig, ahb_codes),
            );
        }
        obj.insert("children".to_string(), serde_json::Value::Object(children));
    }

    serde_json::Value::Object(obj)
}

/// Convert a MIG segment definition into a rich JSON schema value.
///
/// When `mig_number` is available, AHB-filtered codes are used (only codes the PID allows)
/// instead of the full MIG code list.
fn segment_to_schema_value(
    seg: &MigSegment,
    mig_number: Option<&String>,
    ahb_codes: &HashMap<(String, String, usize), Vec<crate::schema::ahb::AhbCodeValue>>,
) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("id".to_string(), serde_json::Value::String(seg.id.clone()));
    let name = seg
        .description
        .as_deref()
        .filter(|d| !d.is_empty())
        .unwrap_or(&seg.name);
    obj.insert(
        "name".to_string(),
        serde_json::Value::String(sanitize_doc(name)),
    );

    // Track per-DE-ID occurrence counters within this segment so we can
    // disambiguate repeated data elements (e.g., three C556/D_9013 in STS).
    let mut de_occurrence: HashMap<String, usize> = HashMap::new();

    let mut elements = Vec::new();

    // Direct data elements — use wire position, not enumeration index
    for de in &seg.data_elements {
        let mut el = serde_json::Map::new();
        el.insert(
            "index".to_string(),
            serde_json::Value::Number((de.position as u64).into()),
        );
        el.insert("id".to_string(), serde_json::Value::String(de.id.clone()));
        let de_name = de
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .unwrap_or(&de.name);
        el.insert(
            "name".to_string(),
            serde_json::Value::String(sanitize_doc(de_name)),
        );

        let occ = de_occurrence.entry(de.id.clone()).or_insert(0);
        emit_element_codes(&mut el, &de.codes, &de.id, mig_number, *occ, ahb_codes);
        *occ += 1;

        elements.push(serde_json::Value::Object(el));
    }

    // Composite elements — use wire position, not offset from data_elements
    for comp in &seg.composites {
        let mut el = serde_json::Map::new();
        el.insert(
            "index".to_string(),
            serde_json::Value::Number((comp.position as u64).into()),
        );
        el.insert(
            "composite".to_string(),
            serde_json::Value::String(comp.id.clone()),
        );
        let comp_name = comp
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .unwrap_or(&comp.name);
        el.insert(
            "name".to_string(),
            serde_json::Value::String(sanitize_doc(comp_name)),
        );

        // Sub-components — use wire position within composite
        let mut components = Vec::new();
        for de in &comp.data_elements {
            let mut sub = serde_json::Map::new();
            sub.insert(
                "sub_index".to_string(),
                serde_json::Value::Number((de.position as u64).into()),
            );
            sub.insert("id".to_string(), serde_json::Value::String(de.id.clone()));
            let de_name = de
                .description
                .as_deref()
                .filter(|d| !d.is_empty())
                .unwrap_or(&de.name);
            sub.insert(
                "name".to_string(),
                serde_json::Value::String(sanitize_doc(de_name)),
            );

            let occ = de_occurrence.entry(de.id.clone()).or_insert(0);
            emit_element_codes(&mut sub, &de.codes, &de.id, mig_number, *occ, ahb_codes);
            *occ += 1;

            components.push(serde_json::Value::Object(sub));
        }
        el.insert(
            "components".to_string(),
            serde_json::Value::Array(components),
        );

        elements.push(serde_json::Value::Object(el));
    }

    obj.insert("elements".to_string(), serde_json::Value::Array(elements));
    serde_json::Value::Object(obj)
}

/// Emit type + codes for a data element, using AHB-filtered codes when available.
fn emit_element_codes(
    el: &mut serde_json::Map<String, serde_json::Value>,
    mig_codes: &[crate::schema::common::CodeDefinition],
    de_id: &str,
    mig_number: Option<&String>,
    occurrence: usize,
    ahb_codes: &HashMap<(String, String, usize), Vec<crate::schema::ahb::AhbCodeValue>>,
) {
    // Try AHB-filtered codes first (only codes this PID allows)
    let ahb_filtered =
        mig_number.and_then(|num| ahb_codes.get(&(num.clone(), de_id.to_string(), occurrence)));

    if let Some(ahb) = ahb_filtered {
        if !ahb.is_empty() {
            el.insert(
                "type".to_string(),
                serde_json::Value::String("code".to_string()),
            );
            el.insert("codes".to_string(), ahb_codes_to_json(ahb));
            return;
        }
    }

    // Fall back to full MIG codes
    if !mig_codes.is_empty() {
        el.insert(
            "type".to_string(),
            serde_json::Value::String("code".to_string()),
        );
        el.insert("codes".to_string(), codes_to_json(mig_codes));
    } else {
        el.insert(
            "type".to_string(),
            serde_json::Value::String("data".to_string()),
        );
    }
}

/// Convert AHB code values to a JSON array of `{value, name}` objects.
fn ahb_codes_to_json(codes: &[crate::schema::ahb::AhbCodeValue]) -> serde_json::Value {
    serde_json::Value::Array(
        codes
            .iter()
            .map(|c| {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "value".to_string(),
                    serde_json::Value::String(c.value.clone()),
                );
                let name = c
                    .description
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .unwrap_or(&c.name);
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(name.to_string()),
                );
                serde_json::Value::Object(obj)
            })
            .collect(),
    )
}

/// Convert code definitions to a JSON array of `{value, name}` objects.
fn codes_to_json(codes: &[crate::schema::common::CodeDefinition]) -> serde_json::Value {
    serde_json::Value::Array(
        codes
            .iter()
            .map(|c| {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "value".to_string(),
                    serde_json::Value::String(c.value.clone()),
                );
                let name = c
                    .description
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .unwrap_or(&c.name);
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(name.to_string()),
                );
                serde_json::Value::Object(obj)
            })
            .collect(),
    )
}

/// Find a segment definition in the MIG tree, searching within the specified group.
fn find_segment_in_mig<'a>(
    seg_id: &str,
    group_id: &str,
    mig: &'a MigSchema,
) -> Option<&'a MigSegment> {
    fn find_in_group<'a>(
        seg_id: &str,
        target_group: &str,
        group: &'a MigSegmentGroup,
    ) -> Option<&'a MigSegment> {
        if group.id == target_group {
            return group
                .segments
                .iter()
                .find(|s| s.id.eq_ignore_ascii_case(seg_id));
        }
        for nested in &group.nested_groups {
            if let Some(s) = find_in_group(seg_id, target_group, nested) {
                return Some(s);
            }
        }
        None
    }

    // Check top-level segments first
    if let Some(seg) = mig
        .segments
        .iter()
        .find(|s| s.id.eq_ignore_ascii_case(seg_id))
    {
        return Some(seg);
    }
    // Check groups
    for group in &mig.segment_groups {
        if let Some(seg) = find_in_group(seg_id, group_id, group) {
            return Some(seg);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Orchestrator: File Generation
// ---------------------------------------------------------------------------

use std::path::Path;

use crate::error::GeneratorError;

/// Generate all PID composition type files for a given AHB and write to disk.
///
/// Creates: `{output_dir}/{fv_lower}/{msg_lower}/pids/pid_{id}.rs` + `mod.rs`
pub fn generate_pid_types(
    mig: &MigSchema,
    ahb: &AhbSchema,
    format_version: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError> {
    let fv_lower = format_version.to_lowercase();
    let msg_lower = ahb.message_type.to_lowercase();
    let pids_dir = output_dir.join(&fv_lower).join(&msg_lower).join("pids");
    std::fs::create_dir_all(&pids_dir)?;

    let mut mod_entries = Vec::new();

    for pid in &ahb.workflows {
        let struct_source = generate_pid_struct(pid, mig, ahb);
        let assembly_source = generate_pid_from_segments(pid, mig, ahb);
        let module_name = format!("pid_{}", pid.id.to_lowercase());
        let filename = format!("{module_name}.rs");

        let full_source = format!(
            "//! Auto-generated PID {} types.\n\
             //! {}\n\
             //! Do not edit manually.\n\n\
             use serde::{{Deserialize, Serialize}};\n\
             use crate::segment::OwnedSegment;\n\
             use crate::cursor::{{SegmentCursor, SegmentNotFound, peek_is, consume, expect_segment}};\n\n\
             {struct_source}\n\
             {assembly_source}",
            pid.id,
            sanitize_doc(&pid.beschreibung)
        );

        std::fs::write(pids_dir.join(&filename), full_source)?;

        // Write companion JSON schema
        let schema = generate_pid_schema(pid, mig, ahb);
        std::fs::write(
            pids_dir.join(format!("pid_{}_schema.json", pid.id.to_lowercase())),
            schema,
        )?;

        mod_entries.push(module_name);
    }

    // Write mod.rs
    let mut mod_rs = String::from(
        "//! Per-PID composition types.\n\
         //! Do not edit manually.\n\n",
    );
    for module in &mod_entries {
        mod_rs.push_str(&format!("pub mod {module};\n"));
    }
    std::fs::write(pids_dir.join("mod.rs"), mod_rs)?;

    // Ensure the parent mod.rs includes `pub mod pids;`
    let parent_mod_path = output_dir.join(&fv_lower).join(&msg_lower).join("mod.rs");
    if parent_mod_path.exists() {
        let parent_mod = std::fs::read_to_string(&parent_mod_path)?;
        if !parent_mod.contains("pub mod pids;") {
            let updated = format!("{parent_mod}pub mod pids;\n");
            std::fs::write(&parent_mod_path, updated)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{ahb_parser, mig_parser};
    use std::path::PathBuf;

    fn load_mig_ahb() -> (MigSchema, AhbSchema) {
        let mig_path = PathBuf::from(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        );
        let ahb_path = PathBuf::from(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml",
        );
        if !mig_path.exists() || !ahb_path.exists() {
            panic!("MIG/AHB XML files not found — run from workspace root");
        }
        let mig = mig_parser::parse_mig(&mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        let ahb = ahb_parser::parse_ahb(&ahb_path, "UTILMD", Some("Strom"), "FV2504").unwrap();
        (mig, ahb)
    }

    #[test]
    fn test_pid_55001_structure_has_named_groups() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

        // SG2 should exist
        let _sg2 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG2")
            .unwrap();
        assert!(!structure.groups.is_empty());

        // SG4 should exist with child groups
        let sg4 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG4")
            .unwrap();
        assert!(!sg4.child_groups.is_empty());

        // SG4's child SG8 groups should have qualifier discrimination
        let sg8_children: Vec<_> = sg4
            .child_groups
            .iter()
            .filter(|c| c.group_id == "SG8")
            .collect();
        let has_qualified = sg8_children.iter().any(|c| !c.qualifier_values.is_empty());
        assert!(
            has_qualified,
            "SG8 groups should have qualifier discrimination"
        );
    }

    #[test]
    fn test_pid_55001_sg2_has_ahb_names() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

        let sg2 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG2")
            .unwrap();
        // SG2 should have a discriminator (NAD qualifier)
        assert!(
            sg2.discriminator.is_some(),
            "SG2 should have NAD discriminator"
        );
    }

    #[test]
    fn test_generate_pid_55001_struct_snapshot() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let source = generate_pid_struct(pid, &mig, &ahb);
        insta::assert_snapshot!("pid_55001_struct", source);
    }

    #[test]
    fn test_generate_pid_55001_schema_snapshot() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let schema = generate_pid_schema(pid, &mig, &ahb);
        insta::assert_snapshot!("pid_55001_schema", schema);
    }

    #[test]
    fn test_generate_pid_55001_from_segments_snapshot() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55001").unwrap();
        let source = generate_pid_from_segments(pid, &mig, &ahb);
        insta::assert_snapshot!("pid_55001_from_segments", source);
    }

    // -----------------------------------------------------------------------
    // Qualifier code name parser tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_base_daten_der_entity() {
        let m = parse_qualifier_code_name("Daten der Marktlokation").unwrap();
        assert_eq!(m.entity, "Marktlokation");
        assert_eq!(m.data_quality, "base");
        assert_eq!(m.content_type, "Daten");
    }

    #[test]
    fn test_parse_informative_daten() {
        let m = parse_qualifier_code_name("Informative Daten der Technischen Ressource").unwrap();
        assert_eq!(m.entity, "TechnischeRessource");
        assert_eq!(m.data_quality, "informativ");
        assert_eq!(m.content_type, "Daten");
    }

    #[test]
    fn test_parse_erwartete_obis_daten() {
        let m = parse_qualifier_code_name("Erwartete OBIS-Daten der Zähleinrichtung").unwrap();
        assert_eq!(m.entity, "Zaehler");
        assert_eq!(m.data_quality, "erwartet");
        assert_eq!(m.content_type, "OBIS-Daten");
    }

    #[test]
    fn test_parse_im_system_vorhandene() {
        let m = parse_qualifier_code_name("Im System vorhandene Daten der Marktlokation").unwrap();
        assert_eq!(m.entity, "Marktlokation");
        assert_eq!(m.data_quality, "im_system");
        assert_eq!(m.content_type, "Daten");
    }

    #[test]
    fn test_parse_differenz() {
        let m =
            parse_qualifier_code_name("Differenz-Netznutzungsabrechnungsdaten der Marktlokation")
                .unwrap();
        assert_eq!(m.entity, "Marktlokation");
        assert_eq!(m.data_quality, "differenz");
    }

    #[test]
    fn test_parse_netzlokation_variant() {
        let m = parse_qualifier_code_name("Daten der Netzlokation").unwrap();
        assert_eq!(m.entity, "Netzlokation");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_steuerbare_ressource() {
        let m = parse_qualifier_code_name("Daten der Steuerbaren Ressource").unwrap();
        assert_eq!(m.entity, "SteuerbareRessource");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_tranche() {
        let m = parse_qualifier_code_name("Daten der Tranche").unwrap();
        assert_eq!(m.entity, "Tranche");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_produkt_daten() {
        let m = parse_qualifier_code_name("Produkt-Daten der Marktlokation").unwrap();
        assert_eq!(m.entity, "Marktlokation");
        assert_eq!(m.data_quality, "base");
        assert_eq!(m.content_type, "Produkt-Daten");
    }

    #[test]
    fn test_parse_kunden_des_lieferanten() {
        let m = parse_qualifier_code_name("Daten des Kunden des Lieferanten").unwrap();
        assert_eq!(m.entity, "Geschaeftspartner");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_static_fallback_zaehler() {
        let m = parse_qualifier_code_name("Zähleinrichtungsdaten").unwrap();
        assert_eq!(m.entity, "Zaehler");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_static_fallback_smartmeter() {
        let m = parse_qualifier_code_name("Smartmeter-Gateway").unwrap();
        assert_eq!(m.entity, "SmartmeterGateway");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_static_fallback_loc_entity() {
        let m = parse_qualifier_code_name("Marktlokation").unwrap();
        assert_eq!(m.entity, "Marktlokation");
        assert_eq!(m.data_quality, "base");
    }

    #[test]
    fn test_parse_empty_returns_none() {
        assert!(parse_qualifier_code_name("").is_none());
    }

    #[test]
    fn test_qualifier_map_from_mig_and_ahb() {
        let (mig, ahb) = load_mig_ahb();
        let qmap = QualifierMap::from_mig_and_ahb(&mig, &ahb);

        // Z01 from MIG: "Daten der Marktlokation"
        let z01 = qmap.get("Z01").expect("Z01 should be in map");
        assert_eq!(z01.entity, "Marktlokation");
        assert_eq!(z01.data_quality, "base");

        // ZF0 from AHB: "Informative Daten der Technischen Ressource"
        let zf0 = qmap.get("ZF0").expect("ZF0 should be in map");
        assert_eq!(zf0.entity, "TechnischeRessource");
        assert_eq!(zf0.data_quality, "informativ");

        // Z98 from AHB: "Informative Daten der Marktlokation"
        let z98 = qmap.get("Z98").expect("Z98 should be in map");
        assert_eq!(z98.entity, "Marktlokation");
        assert_eq!(z98.data_quality, "informativ");

        // ZF3 from AHB: "Informative Daten der Messlokation"
        let zf3 = qmap.get("ZF3").expect("ZF3 should be in map");
        assert_eq!(zf3.entity, "Messlokation");
        assert_eq!(zf3.data_quality, "informativ");

        // SG5 LOC codes (from MIG D_3227) — separate namespace
        let z16 = qmap.get_loc("Z16").expect("Z16 should be in LOC map");
        assert_eq!(z16.entity, "Marktlokation");
    }

    #[test]
    fn test_qualifier_map_reverse_lookup() {
        let (mig, ahb) = load_mig_ahb();
        let qmap = QualifierMap::from_mig_and_ahb(&mig, &ahb);

        // Reverse: Marktlokation + base + Daten → some code (Z01 or Z29)
        let code = qmap.reverse_lookup("Marktlokation", "base", "Daten");
        assert!(
            code.is_some(),
            "Should find a code for Marktlokation base Daten"
        );
        // Verify the returned code maps back to the right entity
        let meta = qmap.get_seq(code.unwrap()).unwrap();
        assert_eq!(meta.entity, "Marktlokation");
        assert_eq!(meta.data_quality, "base");
    }

    #[test]
    fn test_pid_55035_entity_hints() {
        let (mig, ahb) = load_mig_ahb();
        let pid = ahb.workflows.iter().find(|p| p.id == "55035").unwrap();
        let structure = analyze_pid_structure_with_qualifiers(pid, &mig, &ahb);

        let sg4 = structure
            .groups
            .iter()
            .find(|g| g.group_id == "SG4")
            .unwrap();

        // sg8_zf0 → TechnischeRessource
        let sg8_zf0 = sg4
            .child_groups
            .iter()
            .find(|c| c.qualifier_values == vec!["ZF0"])
            .expect("should have ZF0 group");
        assert_eq!(
            sg8_zf0.entity_hint.as_deref(),
            Some("TechnischeRessource"),
            "ZF0 = Informative Daten der Technischen Ressource"
        );
        assert_eq!(sg8_zf0.data_quality_hint.as_deref(), Some("informativ"));

        // sg8_zf3 → Messlokation
        let sg8_zf3 = sg4
            .child_groups
            .iter()
            .find(|c| c.qualifier_values == vec!["ZF3"])
            .expect("should have ZF3 group");
        assert_eq!(
            sg8_zf3.entity_hint.as_deref(),
            Some("Messlokation"),
            "ZF3 = Informative Daten der Messlokation"
        );

        // sg8_z98 → Marktlokation
        let sg8_z98 = sg4
            .child_groups
            .iter()
            .find(|c| c.qualifier_values == vec!["Z98"])
            .expect("should have Z98 group");
        assert_eq!(
            sg8_z98.entity_hint.as_deref(),
            Some("Marktlokation"),
            "Z98 = Informative Daten der Marktlokation"
        );

        // sg8_zd7 → Netzlokation
        let sg8_zd7 = sg4
            .child_groups
            .iter()
            .find(|c| c.qualifier_values == vec!["ZD7"])
            .expect("should have ZD7 group");
        assert_eq!(
            sg8_zd7.entity_hint.as_deref(),
            Some("Netzlokation"),
            "ZD7 = Informative Daten der Netzlokation"
        );

        // sg8_ze1 → Marktlokation (Netznutzungsabrechnungsdaten)
        let sg8_ze1 = sg4
            .child_groups
            .iter()
            .find(|c| c.qualifier_values == vec!["ZE1"])
            .expect("should have ZE1 group");
        assert_eq!(
            sg8_ze1.entity_hint.as_deref(),
            Some("Marktlokation"),
            "ZE1 = Informative Netznutzungsabrechnungsdaten der Marktlokation"
        );

        // Children should inherit parent's entity_hint
        for child in &sg8_zf0.child_groups {
            assert_eq!(
                child.entity_hint.as_deref(),
                Some("TechnischeRessource"),
                "SG10 under ZF0 should inherit TechnischeRessource"
            );
        }
    }
}
