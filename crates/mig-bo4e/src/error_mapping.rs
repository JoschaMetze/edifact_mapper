//! Maps EDIFACT validation paths to BO4E field paths.
//!
//! When validation runs on EDIFACT produced by reverse-mapping BO4E JSON,
//! the resulting `ValidationIssue`s contain EDIFACT segment paths like
//! `SG4/SG5/LOC/C517/3225`. This module resolves those back to BO4E paths
//! like `stammdaten.Marktlokation.marktlokationsId` so users can find
//! the source of the problem in their BO4E input.

use automapper_generator::schema::mig::{MigSchema, MigSegment, MigSegmentGroup};

use crate::definition::{FieldMapping, MappingDefinition};

/// Maps EDIFACT segment paths from validation errors to BO4E field paths.
pub struct Bo4eFieldIndex {
    entries: Vec<IndexEntry>,
}

struct IndexEntry {
    /// EDIFACT group+segment prefix: "SG4/SG5/LOC", "SG2/NAD", "SG4/IDE", etc.
    edifact_prefix: String,
    /// BO4E entity name from TOML meta: "Marktlokation", "Prozessdaten"
    entity: String,
    /// Whether this entity is in stammdaten or transaktionsdaten.
    location: FieldLocation,
    /// Optional companion type (for companion_fields entries).
    companion_type: Option<String>,
    /// Individual field mappings within this segment.
    fields: Vec<FieldEntry>,
}

#[derive(Clone, Copy)]
enum FieldLocation {
    Stammdaten,
    Transaktionsdaten,
}

struct FieldEntry {
    /// Full EDIFACT field_path this matches (e.g., "SG4/SG5/LOC/C517/3225").
    edifact_path: String,
    /// BO4E target field name (e.g., "marktlokationsId").
    bo4e_field: String,
    /// Whether this is a companion field.
    is_companion: bool,
}

impl Bo4eFieldIndex {
    /// Build the index from TOML mapping definitions and a MIG schema.
    ///
    /// For each field in each definition, resolves the TOML numeric path
    /// (e.g., `loc.1.0`) to an AHB-style EDIFACT path (e.g., `SG4/SG5/LOC/C517/3225`)
    /// using the MIG schema for element ID lookup.
    pub fn build(definitions: &[MappingDefinition], mig: &MigSchema) -> Self {
        let mut entries = Vec::new();

        for def in definitions {
            let group_path = source_group_to_slash(&def.meta.source_group);
            let location = classify_entity(&def.meta.entity);
            let companion_type = def.meta.companion_type.clone();

            let mut fields = Vec::new();

            // Process [fields]
            Self::collect_fields(&def.fields, &group_path, mig, false, &mut fields);

            // Process [companion_fields]
            if let Some(ref companion) = def.companion_fields {
                Self::collect_fields(companion, &group_path, mig, true, &mut fields);
            }

            if !fields.is_empty() {
                entries.push(IndexEntry {
                    edifact_prefix: group_path.clone(),
                    entity: def.meta.entity.clone(),
                    location,
                    companion_type,
                    fields,
                });
            }
        }

        Self { entries }
    }

    /// Given an EDIFACT field_path from a ValidationIssue, return the BO4E path.
    pub fn resolve(&self, edifact_field_path: &str) -> Option<String> {
        // Exact match on field entries
        for entry in &self.entries {
            for field in &entry.fields {
                if field.edifact_path == edifact_field_path {
                    return Some(self.build_bo4e_path(entry, field));
                }
            }
        }
        // Prefix match for code/qualifier paths — longest prefix wins
        let mut best: Option<&IndexEntry> = None;
        for entry in &self.entries {
            if !entry.edifact_prefix.is_empty()
                && edifact_field_path.starts_with(&entry.edifact_prefix)
                && best
                    .map(|b| entry.edifact_prefix.len() > b.edifact_prefix.len())
                    .unwrap_or(true)
            {
                best = Some(entry);
            }
        }
        best.map(|entry| self.build_entity_path(entry))
    }

    fn collect_fields(
        field_map: &std::collections::BTreeMap<String, FieldMapping>,
        group_path: &str,
        mig: &MigSchema,
        is_companion: bool,
        out: &mut Vec<FieldEntry>,
    ) {
        for (toml_path, mapping) in field_map {
            let target = match mapping {
                FieldMapping::Simple(s) => s.as_str(),
                FieldMapping::Structured(s) => s.target.as_str(),
                FieldMapping::Nested(_) => continue,
            };

            // Skip qualifiers/defaults with empty target
            if target.is_empty() {
                continue;
            }

            // Parse TOML path: "loc.1.0" → ("loc", Some(1), Some(0))
            //                   "ide.1"  → ("ide", Some(1), None)
            let parsed = match parse_toml_path(toml_path) {
                Some(p) => p,
                None => continue,
            };

            // Resolve via MIG to get the AHB-style EDIFACT path
            if let Some(edifact_path) = resolve_edifact_path(group_path, &parsed, mig) {
                out.push(FieldEntry {
                    edifact_path,
                    bo4e_field: target.to_string(),
                    is_companion,
                });
            }
        }
    }

    fn build_bo4e_path(&self, entry: &IndexEntry, field: &FieldEntry) -> String {
        let location = match entry.location {
            FieldLocation::Stammdaten => "stammdaten",
            FieldLocation::Transaktionsdaten => "transaktionsdaten",
        };
        if field.is_companion {
            if let Some(ref ct) = entry.companion_type {
                format!(
                    "{}.{}.{}.{}",
                    location,
                    entry.entity,
                    to_camel_first_lower(ct),
                    field.bo4e_field
                )
            } else {
                format!("{}.{}.{}", location, entry.entity, field.bo4e_field)
            }
        } else {
            format!("{}.{}.{}", location, entry.entity, field.bo4e_field)
        }
    }

    fn build_entity_path(&self, entry: &IndexEntry) -> String {
        let location = match entry.location {
            FieldLocation::Stammdaten => "stammdaten",
            FieldLocation::Transaktionsdaten => "transaktionsdaten",
        };
        format!("{}.{}", location, entry.entity)
    }
}

/// Parsed TOML field path components.
struct ParsedTomlPath {
    /// Segment tag in uppercase (e.g., "LOC", "DTM").
    segment_tag: String,
    /// Element index (e.g., 1 in "loc.1.0").
    element_idx: usize,
    /// Optional component sub-index (e.g., 0 in "loc.1.0").
    component_idx: Option<usize>,
}

/// Parse a TOML field path like "loc.1.0" or "dtm[92].0.1".
fn parse_toml_path(path: &str) -> Option<ParsedTomlPath> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    // Strip qualifier from tag: "dtm[92]" → "DTM"
    let raw_tag = parts[0];
    let tag = if let Some(bracket) = raw_tag.find('[') {
        &raw_tag[..bracket]
    } else {
        raw_tag
    };

    let element_idx: usize = parts[1].parse().ok()?;
    let component_idx = if parts.len() > 2 {
        Some(parts[2].parse::<usize>().ok()?)
    } else {
        None
    };

    Some(ParsedTomlPath {
        segment_tag: tag.to_uppercase(),
        element_idx,
        component_idx,
    })
}

/// Convert source_group dot notation to slash notation, stripping `:N` suffixes.
/// "SG4.SG5" → "SG4/SG5", "SG8:1.SG10" → "SG8/SG10"
fn source_group_to_slash(source_group: &str) -> String {
    source_group
        .split('.')
        .map(|part| {
            if let Some(colon) = part.find(':') {
                &part[..colon]
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Classify entity into stammdaten vs transaktionsdaten.
fn classify_entity(entity: &str) -> FieldLocation {
    match entity {
        "Prozessdaten" | "Nachricht" => FieldLocation::Transaktionsdaten,
        _ => FieldLocation::Stammdaten,
    }
}

/// Resolve a parsed TOML path to an AHB-style EDIFACT path using the MIG.
fn resolve_edifact_path(
    group_path: &str,
    parsed: &ParsedTomlPath,
    mig: &MigSchema,
) -> Option<String> {
    // Find the segment in the MIG
    let segment = find_segment_in_mig(mig, group_path, &parsed.segment_tag)?;

    // Build a unified list of (position, element_kind) sorted by position
    let resolved = resolve_element_at_position(segment, parsed.element_idx, parsed.component_idx)?;

    let prefix = if group_path.is_empty() {
        parsed.segment_tag.clone()
    } else {
        format!("{}/{}", group_path, parsed.segment_tag)
    };

    match resolved {
        ResolvedElement::DataElement(id) => Some(format!("{}/{}", prefix, id)),
        ResolvedElement::CompositeElement(composite_id, element_id) => {
            Some(format!("{}/{}/{}", prefix, composite_id, element_id))
        }
    }
}

enum ResolvedElement {
    /// A standalone data element: just the element ID.
    DataElement(String),
    /// A component within a composite: (composite_id, data_element_id).
    CompositeElement(String, String),
}

/// Find a segment by tag within a group path in the MIG.
fn find_segment_in_mig<'a>(
    mig: &'a MigSchema,
    group_path: &str,
    segment_tag: &str,
) -> Option<&'a MigSegment> {
    if group_path.is_empty() {
        // Root-level segment
        return mig
            .segments
            .iter()
            .find(|s| s.id.eq_ignore_ascii_case(segment_tag));
    }

    let parts: Vec<&str> = group_path.split('/').collect();

    // Find the first group
    let mut current_group = mig
        .segment_groups
        .iter()
        .find(|g| g.id.eq_ignore_ascii_case(parts[0]))?;

    // Navigate nested groups
    for &part in &parts[1..] {
        current_group = current_group
            .nested_groups
            .iter()
            .find(|g| g.id.eq_ignore_ascii_case(part))?;
    }

    find_segment_in_group(current_group, segment_tag)
}

/// Find a segment by tag within a group (checking the group and its nested groups).
fn find_segment_in_group<'a>(
    group: &'a MigSegmentGroup,
    segment_tag: &str,
) -> Option<&'a MigSegment> {
    group
        .segments
        .iter()
        .find(|s| s.id.eq_ignore_ascii_case(segment_tag))
}

/// Resolve an element at a given position within a MIG segment.
///
/// Builds a unified position list from data_elements and composites,
/// then finds what's at element_idx. If it's a composite and component_idx
/// is provided, returns the sub-element.
fn resolve_element_at_position(
    segment: &MigSegment,
    element_idx: usize,
    component_idx: Option<usize>,
) -> Option<ResolvedElement> {
    // Check composites first — they have a position field
    if let Some(composite) = segment
        .composites
        .iter()
        .find(|c| c.position == element_idx)
    {
        let comp_idx = component_idx.unwrap_or(0);
        // Find the data element at the component sub-index by sorting by position
        let mut sub_elements: Vec<_> = composite.data_elements.iter().collect();
        sub_elements.sort_by_key(|de| de.position);
        let de = sub_elements.get(comp_idx)?;
        return Some(ResolvedElement::CompositeElement(
            composite.id.clone(),
            de.id.clone(),
        ));
    }

    // Check standalone data elements
    if let Some(de) = segment
        .data_elements
        .iter()
        .find(|d| d.position == element_idx)
    {
        return Some(ResolvedElement::DataElement(de.id.clone()));
    }

    None
}

/// Convert PascalCase to camelCase (first char lowercase).
fn to_camel_first_lower(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_group_to_slash() {
        assert_eq!(source_group_to_slash("SG4.SG5"), "SG4/SG5");
        assert_eq!(source_group_to_slash("SG4"), "SG4");
        assert_eq!(source_group_to_slash("SG8:1.SG10"), "SG8/SG10");
        assert_eq!(source_group_to_slash(""), "");
    }

    #[test]
    fn test_parse_toml_path() {
        let p = parse_toml_path("loc.1.0").unwrap();
        assert_eq!(p.segment_tag, "LOC");
        assert_eq!(p.element_idx, 1);
        assert_eq!(p.component_idx, Some(0));

        let p = parse_toml_path("ide.1").unwrap();
        assert_eq!(p.segment_tag, "IDE");
        assert_eq!(p.element_idx, 1);
        assert_eq!(p.component_idx, None);

        let p = parse_toml_path("dtm[92].0.1").unwrap();
        assert_eq!(p.segment_tag, "DTM");
        assert_eq!(p.element_idx, 0);
        assert_eq!(p.component_idx, Some(1));

        assert!(parse_toml_path("loc").is_none());
    }

    #[test]
    fn test_classify_entity() {
        assert!(matches!(
            classify_entity("Prozessdaten"),
            FieldLocation::Transaktionsdaten
        ));
        assert!(matches!(
            classify_entity("Nachricht"),
            FieldLocation::Transaktionsdaten
        ));
        assert!(matches!(
            classify_entity("Marktlokation"),
            FieldLocation::Stammdaten
        ));
        assert!(matches!(
            classify_entity("Marktteilnehmer"),
            FieldLocation::Stammdaten
        ));
    }

    #[test]
    fn test_to_camel_first_lower() {
        assert_eq!(
            to_camel_first_lower("MarktlokationEdifact"),
            "marktlokationEdifact"
        );
        assert_eq!(to_camel_first_lower("Foo"), "foo");
        assert_eq!(to_camel_first_lower(""), "");
    }

    #[test]
    fn test_resolve_returns_none_for_unknown_path() {
        let index = Bo4eFieldIndex { entries: vec![] };
        assert!(index.resolve("SG99/UNKNOWN/9999").is_none());
    }
}
