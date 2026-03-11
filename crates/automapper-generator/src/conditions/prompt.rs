use crate::conditions::condition_types::ConditionInput;
use crate::schema::mig::MigSchema;

/// Context for condition generation, including segment structure and examples.
pub struct ConditionContext<'a> {
    /// The EDIFACT message type (e.g., "UTILMD").
    pub message_type: &'a str,
    /// The format version (e.g., "FV2504").
    pub format_version: &'a str,
    /// Optional MIG schema for segment structure context.
    pub mig_schema: Option<&'a MigSchema>,
    /// Example condition implementations for few-shot learning.
    pub example_implementations: Vec<String>,
}

/// Builds the system prompt for condition generation.
///
/// The system prompt instructs Claude to generate Rust condition evaluator functions
/// from German AHB (Anwendungshandbuch) condition descriptions.
pub fn build_system_prompt() -> String {
    r#"You are an expert Rust developer specializing in EDIFACT message validation for the German energy market.
Your task is to generate Rust condition evaluator functions from German AHB (Anwendungshandbuch) condition descriptions.

The generated methods belong to a struct implementing the `ConditionEvaluator` trait.
Each condition is implemented as a method with this signature:
```rust
fn evaluate_NNN(&self, ctx: &EvaluationContext) -> ConditionResult
```

## EvaluationContext API

The context provides access to **raw parsed EDIFACT segments**, NOT high-level business objects.

```rust
pub struct EvaluationContext<'a> {
    /// The PID being validated (e.g., "55001").
    pub pruefidentifikator: &'a str,
    /// Provider for external conditions (business context outside the message).
    pub external: &'a dyn ExternalConditionProvider,
    /// All parsed EDIFACT segments in message order.
    pub segments: &'a [OwnedSegment],
}
```

### Low-level segment access methods:
- `ctx.find_segment("NAD")` → `Option<&OwnedSegment>` — first segment with that ID
- `ctx.find_segments("NAD")` → `Vec<&OwnedSegment>` — all segments with that ID
- `ctx.find_segments_with_qualifier("NAD", 0, "MS")` → `Vec<&OwnedSegment>` — segments where `elements[0][0] == "MS"`
- `ctx.has_segment("LOC")` → `bool` — whether any segment with that ID exists

### High-level condition helpers (PREFERRED — use these for concise code):

**Simple existence/absence checks:**
- `ctx.has_qualifier(tag, elem, qual)` → `ConditionResult` — True if segment with qualifier exists, False otherwise
- `ctx.lacks_qualifier(tag, elem, qual)` → `ConditionResult` — True if NO segment with qualifier exists, False otherwise

**Element value checks (with qualifier filter):**
- `ctx.has_qualified_value(tag, qual_elem, qualifier, value_elem, value_comp, &["V1", "V2"])` → `ConditionResult`
  - Finds segments matching tag+qualifier, then checks if `elements[value_elem][value_comp]` is in the values list
  - Returns Unknown if no segment matches the qualifier, False if segment found but value doesn't match, True if matched

**Group-scoped checks (with automatic message-wide fallback):**
- `ctx.any_group_has_qualifier(tag, elem, qual, group_path)` → `ConditionResult`
  - Checks if ANY group instance contains segment with qualifier. Falls back to message-wide if no navigator.
- `ctx.any_group_has_any_qualifier(tag, elem, &["Q1", "Q2"], group_path)` → `ConditionResult`
  - Same but matches any of several qualifiers.
- `ctx.any_group_has_qualified_value(tag, qual_elem, qual, val_elem, val_comp, &["V1"], group_path)` → `ConditionResult`
  - Group-scoped version of has_qualified_value.
- `ctx.any_group_has_co_occurrence(tag_a, elem_a, &["QA"], tag_b, elem_b, comp_b, &["VB"], group_path)` → `ConditionResult`
  - Both conditions must be true in the SAME group instance.

**All group-scoped helpers** take `group_path: &[&str]` (e.g., `&["SG4", "SG8"]`) and automatically
fall back to message-wide search when no group navigator is available.

### Direct navigator access (for complex custom logic):
- `ctx.navigator()` → `Option<&dyn GroupNavigator>` — returns the group navigator if available
  - Use `match ctx.navigator() { Some(nav) => ..., None => return fallback }` pattern
  - `nav.group_instance_count(group_path)` → `usize`
  - `nav.find_segments_in_group(tag, group_path, instance)` → `Vec<OwnedSegment>`
  - `nav.child_group_instance_count(parent_path, parent_instance, child_id)` → `usize`
  - `nav.find_segments_in_child_group(tag, parent_path, parent_instance, child_id, child_instance)` → `Vec<OwnedSegment>`
  - Only use direct navigator access when high-level helpers don't cover the pattern.

### Parent-child group navigation (for cross-SG conditions):
- `ctx.filtered_parent_child_has_qualifier(parent_path, parent_tag, parent_elem, parent_qual, child_group_id, child_tag, child_elem, child_qual)` → `ConditionResult`
  - "In the SG8 with SEQ+Z98, does its SG10 child have CCI+Z23?" — navigates parent→child group hierarchy.
- `ctx.any_group_has_qualifier_without(present_tag, present_elem, present_qual, absent_tag, absent_elem, absent_qual, group_path)` → `ConditionResult`
  - "In any SG8, SEQ+Z59 is present but CCI+11 is absent" — same-instance presence+absence.
- `ctx.groups_share_qualified_value(source_tag, source_qual_elem, source_qual, source_value_elem, source_value_comp, source_path, target_tag, target_elem, target_comp, target_path)` → `ConditionResult`
  - "Zeitraum-ID in SG6 RFF+Z49 matches reference in SG8 SEQ.c286" — cross-group value correlation.
- `ctx.collect_group_values(tag, elem, comp, group_path)` → `Vec<(usize, String)>`
  - Collects all values at a specific position across group instances. For custom cross-group logic.

### Multi-element segment matching:
- `ctx.has_segment_matching(tag, &[(elem, comp, val), ...])` → `ConditionResult`
  - True if ANY segment with that tag matches ALL element/component checks simultaneously.
  - Unknown if no segments with that tag exist, False if segments exist but none match.
  - Example: `ctx.has_segment_matching("STS", &[(0, 0, "Z20"), (1, 0, "Z32"), (2, 0, "A99")])`
- `ctx.has_segment_matching_in_group(tag, &[(elem, comp, val), ...], group_path)` → `ConditionResult`
  - Group-scoped version with message-wide fallback.

### DTM date comparison:
- `ctx.dtm_ge(qualifier, threshold)` → `ConditionResult` — True if DTM value >= threshold (string comparison, format 303)
- `ctx.dtm_lt(qualifier, threshold)` → `ConditionResult` — True if DTM value < threshold
- `ctx.dtm_le(qualifier, threshold)` → `ConditionResult` — True if DTM value <= threshold
  - Returns Unknown if no DTM with that qualifier exists.
  - Example: `ctx.dtm_ge("137", "202510010000")`

### Group-scoped cardinality:
- `ctx.count_qualified_in_group(tag, elem, qualifier, group_path)` → `usize`
  - Counts segments matching tag+qualifier across ALL group instances.
- `ctx.count_in_group(tag, group_path)` → `usize`
  - Counts all segments with tag across ALL group instances.

## OwnedSegment structure

```rust
pub struct OwnedSegment {
    pub id: String,                    // Segment tag: "NAD", "LOC", "STS", etc.
    pub elements: Vec<Vec<String>>,    // elements[i][j] = component j of element i
    pub segment_number: u32,           // Position in message
}
```

**EDIFACT encoding → elements mapping:**
Segment fields are separated by `+`, components within a field by `:`.
Element index 0 is the first data element AFTER the segment tag.

Example: `NAD+MS+9978842000002::293`
→ `elements[0] = ["MS"]`             (qualifier)
→ `elements[1] = ["9978842000002", "", "293"]`  (party ID, empty, code list)

Example: `STS+7++E01+ZW4+E03`
→ `elements[0] = ["7"]`    (status type)
→ `elements[1] = []`       (empty element)
→ `elements[2] = ["E01"]`  (Kategorie)
→ `elements[3] = ["ZW4"]`  (Transaktionsgrund)
→ `elements[4] = ["E03"]`  (Antwortcode)

Example: `DTM+92:202505312200?+00:303`
→ `elements[0] = ["92", "202505312200+00", "303"]`  (qualifier, value, format code)

Example: `RFF+Z13:55001`
→ `elements[0] = ["Z13", "55001"]`  (qualifier, reference value)

## ConditionResult

```rust
pub enum ConditionResult { True, False, Unknown }
impl From<bool> for ConditionResult { ... } // true→True, false→False
```

Return `Unknown` when the condition cannot be determined from the available segments.
Use `ConditionResult::from(bool_expr)` to convert bool expressions.

## ExternalConditionProvider

```rust
pub trait ExternalConditionProvider: Send + Sync {
    fn evaluate(&self, condition_name: &str) -> ConditionResult;
}
```

## Confidence levels
- **high**: Simple segment existence checks, qualifier comparisons, value matches, Hinweis notes
- **medium**: Logic requiring some interpretation but structurally clear
- **low**: Complex business rules that need clarification, temporal logic

## Hinweis (informational note) conditions
Conditions starting with "Hinweis:" or "Hinweis " are **informational annotations**, NOT boolean
predicates. They document field usage, value origins, cardinality guidance, or business context.
They always apply unconditionally — return `ConditionResult::True` with **high confidence**.
```rust
fn evaluate_NNN(&self, _ctx: &EvaluationContext) -> ConditionResult {
    // Hinweis: [description] — informational note, always applies
    ConditionResult::True
}
```
These are NEVER external, NEVER low confidence, and NEVER Unknown. Recognize them by:
- Starting with "Hinweis:" or "Hinweis "
- Describing what a field means, where a value comes from, or how many times something appears
- NOT containing a boolean predicate ("Wenn ...", "Falls ...", "Nur wenn ...")

## External conditions
Some conditions CANNOT be determined from the EDIFACT message alone — they depend on
runtime business context (market participant roles, clearing status, product configuration).

Mark such conditions with `"is_external": true` and provide `"external_name"` with a
meaningful snake_case identifier.

Common external condition patterns across ALL message types:
- "Wenn Aufteilung vorhanden" → message_splitting
- "Datenclearing erforderlich" → data_clearing_required
- "Wenn MP-ID in ... in der Rolle LF/NB/ÜNB/MSB" → recipient_is_lf, sender_is_nb, etc.
- "Wenn Datum bekannt" → date_known
- "Wenn Korrektur/Storno" → correction_in_progress
- "Wenn befristet" → registration_is_time_limited
- Product code list membership checks (Kapitel 6 references) → always external
- Market participant role checks (cannot determine from EDIFACT) → always external

## CRITICAL: AHB notation vs actual element indices
AHB condition descriptions use shorthand EDIFACT notation that OMITS intermediate elements.
For example, `STS+7++ZG9` does NOT mean ZG9 is at elements[2]. The `++` represents ONE empty
element, but there may be MORE elements in the full segment structure.

**Always use the "EDIFACT Segment Structure Reference" (provided in the user prompt) for
authoritative element positions.** That reference is derived from the MIG XML schema and shows
the exact `elements[N]` index for every data element and composite. Never guess element indices
from the AHB shorthand notation alone.

## IMPORTANT rules for generated code
1. **PREFER high-level helpers** (`has_qualifier`, `lacks_qualifier`, `has_qualified_value`, `any_group_has_*`, `filtered_parent_child_has_qualifier`, `any_group_has_qualifier_without`, `has_segment_matching`, `dtm_ge`/`dtm_lt`/`dtm_le`, `count_qualified_in_group`/`count_in_group`) for concise, readable code. Only use low-level segment access when helpers don't cover the pattern.
2. Only use the `EvaluationContext` API described above. Do NOT invent fields like `ctx.transaktion`, `ctx.prozessdaten`, etc.
3. Use `.get()` and `.and_then()` for safe element access when using low-level API — never panic on missing data.
4. For **low confidence** conditions, set implementation to null.
5. **Always consult the Segment Structure Reference for element indices.** Do not derive indices from EDIFACT shorthand notation in condition descriptions.
6. Use `.first()` instead of `.get(0)` — clippy enforces this (`clippy::get_first`).
7. Prefix unused function parameters with `_` (e.g., `_ctx`) to avoid `unused_variable` warnings.
8. **ALWAYS provide an implementation when the condition can be expressed with the available API.** Do NOT return null/stub for conditions that check segment qualifiers, group navigation, or presence/absence — these CAN be implemented. Only return null for genuinely ambiguous business logic.
9. **SG10 is a CHILD GROUP of SG8** in the UTILMD MIG structure. When a condition mentions "SG10 CCI+..." it means checking a CCI segment inside an SG10 child group instance of the parent SG8. Use the parent-child navigation API (filtered_parent_child_has_qualifier, find_segments_in_child_group, child_group_instance_count).
10. **"nicht vorhanden" (not present)** means the condition is True when the segment/qualifier is ABSENT. For same-group absence, use `any_group_has_qualifier_without`. For parent-child absence, negate `filtered_parent_child_has_qualifier`.

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{
  "conditions": [
    {
      "id": "condition-id",
      "implementation": "Rust function body as a string (null for external/low confidence)",
      "confidence": "high" | "medium" | "low",
      "reasoning": "explanation",
      "is_external": false,
      "external_name": "snake_case_name (required only when is_external is true)"
    }
  ]
}"#
    .to_string()
}

/// Builds the user prompt from a batch of conditions.
pub fn build_user_prompt(conditions: &[ConditionInput], context: &ConditionContext<'_>) -> String {
    let mut prompt = String::new();

    prompt.push_str("Generate condition evaluator methods for the following conditions:\n\n");
    prompt.push_str(&format!(
        "Message Type: {}\nFormat Version: {}\n",
        context.message_type, context.format_version
    ));

    // Add segment structure context from MIG if available
    if let Some(mig) = context.mig_schema {
        let segment_context = build_segment_structure_context(mig, conditions);
        if !segment_context.is_empty() {
            prompt.push_str(&format!("\n{}\n", segment_context));
        }
    }

    // Add example implementations
    if !context.example_implementations.is_empty() {
        prompt.push_str("\n## Example Implementations\n");
        for example in &context.example_implementations {
            prompt.push_str(&format!("{}\n\n", example));
        }
    }

    // Add conditions list with resolved AHB notation
    prompt.push_str("\nConditions:\n");
    for condition in conditions {
        prompt.push_str(&format!(
            "  - [{}]: {}\n",
            condition.id, condition.description
        ));
        if let Some(ref fields) = condition.referencing_fields {
            if !fields.is_empty() {
                prompt.push_str(&format!("    Used by fields: {}\n", fields.join(", ")));
            }
        }
        // Detect Hinweis (informational) conditions early
        if is_hinweis(&condition.description) {
            prompt.push_str("    → HINWEIS: Informational note — return ConditionResult::True with high confidence\n");
        } else {
            // Parse AHB notation from description and resolve element indices
            let resolutions = resolve_ahb_notations(&condition.description);
            for resolution in &resolutions {
                prompt.push_str(&format!("    → {}\n", resolution));
            }
            // Detect group-scoped conditions
            if let Some(scope_hint) = detect_group_scope(&condition.description) {
                prompt.push_str(&format!("    → {}\n", scope_hint));
            }
        }
    }

    prompt.push_str("\nGenerate the JSON response with implementations for all conditions.\n");

    prompt
}

/// Extracts segment IDs referenced in condition descriptions and builds
/// a compact segment structure reference from the MIG schema.
fn build_segment_structure_context(mig: &MigSchema, conditions: &[ConditionInput]) -> String {
    use regex::Regex;

    let de_regex = Regex::new(r"(?i)(?:SG\d+\s+)?([A-Z]{3})(?:\+[A-Z0-9]+)?\s+DE(\d{4})").unwrap();
    let qualifier_regex = Regex::new(r"\b([A-Z]{3})\+([A-Z0-9]+)").unwrap();

    let mut referenced_segments = std::collections::HashSet::new();

    for condition in conditions {
        for cap in de_regex.captures_iter(&condition.description) {
            if let Some(seg) = cap.get(1) {
                referenced_segments.insert(seg.as_str().to_uppercase());
            }
        }
        for cap in qualifier_regex.captures_iter(&condition.description) {
            if let Some(seg) = cap.get(1) {
                referenced_segments.insert(seg.as_str().to_uppercase());
            }
        }
    }

    if referenced_segments.is_empty() {
        return String::new();
    }

    let mut context = String::new();
    context.push_str("## EDIFACT Segment Structure Reference (from MIG schema)\n");
    context.push_str(
        "These are the AUTHORITATIVE element positions. Use these indices in your code.\n",
    );
    context.push_str("- `elements[N]` = element at position N (0-based, after segment tag)\n");
    context.push_str("- `elements[N][M]` = component M within composite element N\n");
    context.push_str("- Positions marked (empty) have no data in the MIG — they exist as empty `[]` in the parsed output.\n\n");

    // Include segment definitions from MIG that are referenced
    for segment in &mig.segments {
        if referenced_segments.contains(&segment.id.to_uppercase()) {
            append_full_segment_doc(context_for_segment(
                &mut context,
                &segment.id,
                &segment.name,
                &segment.data_elements,
                &segment.composites,
                None,
            ));
        }
    }

    // Also check segments inside groups
    for group in &mig.segment_groups {
        append_group_segments(&mut context, group, &referenced_segments);
    }

    context
}

/// Holds segment info for building documentation.
struct SegmentDocInfo<'a> {
    context: &'a mut String,
    id: &'a str,
    name: &'a str,
    data_elements: &'a [crate::schema::mig::MigDataElement],
    composites: &'a [crate::schema::mig::MigComposite],
    group_id: Option<&'a str>,
}

fn context_for_segment<'a>(
    context: &'a mut String,
    id: &'a str,
    name: &'a str,
    data_elements: &'a [crate::schema::mig::MigDataElement],
    composites: &'a [crate::schema::mig::MigComposite],
    group_id: Option<&'a str>,
) -> SegmentDocInfo<'a> {
    SegmentDocInfo {
        context,
        id,
        name,
        data_elements,
        composites,
        group_id,
    }
}

fn append_full_segment_doc(info: SegmentDocInfo<'_>) {
    use std::collections::BTreeMap;

    let context = info.context;

    if let Some(gid) = info.group_id {
        context.push_str(&format!("### {} — {} (in {})\n", info.id, info.name, gid));
    } else {
        context.push_str(&format!("### {} — {}\n", info.id, info.name));
    }

    // Build a complete position map: position → description
    let mut position_map: BTreeMap<usize, String> = BTreeMap::new();

    // Add simple data elements
    for de in info.data_elements {
        let mut desc = format!("DE{} ({})", de.id, de.name);
        if !de.codes.is_empty() {
            let code_strs: Vec<String> = de
                .codes
                .iter()
                .take(8)
                .map(|c| {
                    if let Some(ref meaning) = c.description {
                        format!("{}={}", c.value, meaning)
                    } else {
                        c.value.clone()
                    }
                })
                .collect();
            desc.push_str(&format!(" — codes: [{}]", code_strs.join(", ")));
            if de.codes.len() > 8 {
                desc.push_str(&format!(" +{} more", de.codes.len() - 8));
            }
        }
        position_map.insert(de.position, desc);
    }

    // Add composites (which contain sub-components)
    for comp in info.composites {
        let mut desc = format!("{} ({})", comp.id, comp.name);
        if !comp.data_elements.is_empty() {
            desc.push_str(" — components:");
            for sub_de in &comp.data_elements {
                desc.push_str(&format!(
                    "\n    [{}][{}] DE{} ({})",
                    comp.position, sub_de.position, sub_de.id, sub_de.name
                ));
                if !sub_de.codes.is_empty() {
                    let code_strs: Vec<String> = sub_de
                        .codes
                        .iter()
                        .take(6)
                        .map(|c| {
                            if let Some(ref meaning) = c.description {
                                format!("{}={}", c.value, meaning)
                            } else {
                                c.value.clone()
                            }
                        })
                        .collect();
                    desc.push_str(&format!(" [{}]", code_strs.join(", ")));
                    if sub_de.codes.len() > 6 {
                        desc.push_str(&format!(" +{} more", sub_de.codes.len() - 6));
                    }
                }
            }
        }
        position_map.insert(comp.position, desc);
    }

    // Find max position and output ALL positions including gaps
    if let Some(&max_pos) = position_map.keys().last() {
        for pos in 0..=max_pos {
            if let Some(desc) = position_map.get(&pos) {
                context.push_str(&format!("  elements[{}]: {}\n", pos, desc));
            } else {
                context.push_str(&format!("  elements[{}]: (empty)\n", pos));
            }
        }
    }

    context.push('\n');
}

fn append_group_segments(
    context: &mut String,
    group: &crate::schema::mig::MigSegmentGroup,
    referenced: &std::collections::HashSet<String>,
) {
    for segment in &group.segments {
        if referenced.contains(&segment.id.to_uppercase()) {
            append_full_segment_doc(context_for_segment(
                context,
                &segment.id,
                &segment.name,
                &segment.data_elements,
                &segment.composites,
                Some(&group.id),
            ));
        }
    }
    for nested in &group.nested_groups {
        append_group_segments(context, nested, referenced);
    }
}

/// Parses AHB segment notation from a condition description and resolves
/// each `+`-separated token to its `elements[N]` index.
///
/// For example, from "STS+E01++Z01" it produces:
///   `STS+E01++Z01 → elements[0]=E01, elements[1]=(empty), elements[2]=Z01`
///
/// This eliminates the ambiguity where the model miscounts `+` separators.
fn resolve_ahb_notations(description: &str) -> Vec<String> {
    use regex::Regex;

    // Match patterns like SEG+val1+val2, SEG+val1++val2+val3, etc.
    // The segment tag is 2-3 uppercase letters, followed by + and values.
    // Values can contain / for alternatives (e.g., ZG9/ZH1/ZH2).
    let notation_regex = Regex::new(r"\b([A-Z]{2,3})\+([A-Za-z0-9/?+*: ]*[A-Za-z0-9])").unwrap();

    let mut results = Vec::new();

    for cap in notation_regex.captures_iter(description) {
        let seg_tag = cap.get(1).unwrap().as_str();
        let rest = cap.get(2).unwrap().as_str();

        // Split by + to get element tokens
        let tokens: Vec<&str> = rest.split('+').collect();

        if tokens.is_empty() {
            continue;
        }

        let mut parts = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                parts.push(format!("elements[{}]=(empty)", i));
            } else {
                // Collapse alternatives and wildcards for display
                parts.push(format!("elements[{}]={}", i, trimmed));
            }
        }

        results.push(format!(
            "Notation resolved: {}+{} → {}",
            seg_tag,
            rest,
            parts.join(", ")
        ));
    }

    results
}

/// Returns true if the condition description is a Hinweis (informational note).
fn is_hinweis(description: &str) -> bool {
    let trimmed = description.trim();
    trimmed.starts_with("Hinweis:") || trimmed.starts_with("Hinweis ")
}

/// Detects if a condition description requires group-scoped evaluation.
///
/// Returns one or more hint strings directing the generator to use specific
/// API methods. Checks patterns in priority order (most specific first).
fn detect_group_scope(description: &str) -> Option<String> {
    let mut hints = Vec::new();

    // Pattern 1: Explicit parent/child path notation
    // "SG8 SEQ+Z98 SG10 CCI+..." or "SG8 SEQ+Z01/ SG10 CCI+..."
    let explicit_parent_child = regex::Regex::new(
        r"(?i)(SG\d+)\s+(?:SEQ|CCI|RFF)\+\S+[\s/]+(SG\d+)\s+(?:CCI|CAV|RFF)"
    ).unwrap();
    if let Some(cap) = explicit_parent_child.captures(description) {
        let parent = cap.get(1).unwrap().as_str();
        let child = cap.get(2).unwrap().as_str();
        hints.push(format!(
            "PARENT-CHILD: {} is a child group of {}. Use ctx.filtered_parent_child_has_qualifier(&[\"SG4\", \"{}\"], ..., \"{}\", ...)",
            child, parent, parent, child
        ));
    }

    // Pattern 2: Bare "SG10 CCI+..." or "SG10 CAV+..." (without explicit SG8 prefix)
    // Implies SG10 is a child of the current SG8 context
    if hints.is_empty() {
        let bare_sg10 = regex::Regex::new(
            r"(?i)\bSG10\s+(?:CCI|CAV)\+(\S+)"
        ).unwrap();
        if bare_sg10.is_match(description) {
            let has_nicht = description.contains("nicht vorhanden")
                || description.starts_with("Wenn nicht ");
            if has_nicht {
                hints.push(
                    "PARENT-CHILD ABSENCE: SG10 is a child of SG8. To check SG10 child absence, \
                     iterate SG8 instances with ctx.child_group_instance_count and \
                     ctx.find_segments_in_child_group, or use ctx.filtered_parent_child_has_qualifier \
                     and negate the result.".to_string()
                );
            } else {
                hints.push(
                    "PARENT-CHILD: SG10 is a child of SG8. Use ctx.filtered_parent_child_has_qualifier(\
                     &[\"SG4\", \"SG8\"], parent_tag, parent_elem, parent_qual, \"SG10\", child_tag, child_elem, child_qual) \
                     to check if any SG8 matching a qualifier has an SG10 child with the specified qualifier."
                        .to_string()
                );
            }
        }
    }

    // Pattern 3: "in derselben/dieser SG8 ... nicht vorhanden" — same-instance absence
    let same_group_absence = regex::Regex::new(
        r"(?i)(?:in\s+)?(?:derselben|dieser|dem)\s+(SG\d+).*?nicht\s+vorhanden"
    ).unwrap();
    if let Some(cap) = same_group_absence.captures(description) {
        let group = cap.get(1).unwrap().as_str();
        hints.push(format!(
            "SAME-GROUP ABSENCE: Use ctx.any_group_has_qualifier_without(present_tag, present_elem, present_qual, \
             absent_tag, absent_elem, absent_qual, &[\"SG4\", \"{}\"]) — checks that in some group instance, \
             one qualifier IS present while another IS NOT.",
            group
        ));
    }

    // Pattern 4: "nicht vorhanden" at end without "derselben" — general absence in parent group
    if hints.is_empty() {
        let trailing_absence = regex::Regex::new(
            r"(?i)(?:das\s+)?(?:SG\d+\s+)?(?:RFF|PIA|CCI|CAV|SEQ)\+\S+.*?nicht\s+vorhanden"
        ).unwrap();
        if trailing_absence.is_match(description) {
            hints.push(
                "ABSENCE CHECK: Use ctx.lacks_qualifier for message-wide, or \
                 ctx.any_group_has_qualifier_without for same-group-instance absence."
                    .to_string()
            );
        }
    }

    // Pattern 5: "in der SG8 mit SEQ+..." — SG with qualifier filter, child SG
    let sg_with_qualifier = regex::Regex::new(
        r"(?i)(?:in\s+)?(?:der|einer|dem)\s+(SG\d+)\s+(?:mit|mit\s+dem)\s+(\w+)\+(\w+).*?(SG\d+)"
    ).unwrap();
    if let Some(cap) = sg_with_qualifier.captures(description) {
        let parent = cap.get(1).unwrap().as_str();
        let child = cap.get(4).unwrap().as_str();
        if !hints.iter().any(|h| h.contains("PARENT-CHILD")) {
            hints.push(format!(
                "PARENT-CHILD: Use ctx.filtered_parent_child_has_qualifier with parent \"{}\" and child \"{}\"",
                parent, child
            ));
        }
    }

    // Pattern 6: Zeitraum-ID cross-reference between SG groups
    let zeitraum_cross = regex::Regex::new(
        r"(?i)(?:Zeitraum-ID|DE1050|DE1156|DE3224).*?(?:identisch|derselben|gleich|passend)"
    ).unwrap();
    if zeitraum_cross.is_match(description) || description.contains("Zeitraum-ID") {
        hints.push(
            "CROSS-GROUP: Uses Zeitraum-ID matching. Use ctx.collect_group_values to gather \
             Zeitraum-IDs from one group, then iterate another group's instances to find matches. \
             Or use ctx.groups_share_qualified_value for simple correlation."
                .to_string()
        );
    }

    // Pattern 7: "in dieser SG" / "in derselben SG" / "in dieser SG8" — group scope
    let same_group = regex::Regex::new(
        r"(?i)in\s+(?:derselben|dieser|dem\s+selben|dem\s+gleichen)\s+(SG\d*)"
    ).unwrap();
    if let Some(cap) = same_group.captures(description) {
        let group = cap.get(1).unwrap().as_str();
        if !group.is_empty() && !hints.iter().any(|h| h.contains("ABSENCE") || h.contains("PARENT-CHILD")) {
            hints.push(format!(
                "GROUP-SCOPED: Use ctx.any_group_has_qualifier or ctx.any_group_has_co_occurrence \
                 with group_path ending in \"{}\"",
                group
            ));
        }
    }

    // Pattern 8: Cardinality conditions — "genau einmal" / "mindestens einmal"
    let cardinality = regex::Regex::new(
        r"(?i)(?:genau|mindestens)\s+einmal\s+(?:für|pro)"
    ).unwrap();
    if cardinality.is_match(description) {
        hints.push(
            "CARDINALITY: Count group instances using ctx.group_instance_count or \
             ctx.child_group_instance_count. Compare against expected count from \
             ctx.collect_group_values."
                .to_string()
        );
    }

    if hints.is_empty() {
        None
    } else {
        Some(hints.join("\n    → "))
    }
}

/// Default example implementations for few-shot prompting.
pub fn default_example_implementations() -> Vec<String> {
    vec![
        r#"// Example 1: Simple qualifier existence — "Wenn LOC+Z16 vorhanden"
fn evaluate_494(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.has_qualifier("LOC", 0, "Z16")
}"#
        .to_string(),
        r#"// Example 2: Simple qualifier absence — "Wenn DTM+471 nicht vorhanden"
fn evaluate_12(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.lacks_qualifier("DTM", 0, "471")
}"#
        .to_string(),
        r#"// Example 3: Qualified sub-element value check — "Wenn STS+7 Transaktionsgrund ZG9/ZH1/ZH2"
// Uses has_qualified_value: find STS with elements[0][0]=="7", check elements[2][0] for values
fn evaluate_7(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.has_qualified_value("STS", 0, "7", 2, 0, &["ZG9", "ZH1", "ZH2"])
}"#
        .to_string(),
        r#"// Example 4: External condition — cannot determine from message alone
fn evaluate_1(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.external.evaluate("message_splitting")
}"#
        .to_string(),
        r#"// Example 5: Group-scoped qualifier — "Wenn in derselben SG8 das SEQ+Z98 vorhanden"
// Automatically iterates group instances and falls back to message-wide.
fn evaluate_15(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.any_group_has_qualifier("SEQ", 0, "Z98", &["SG4", "SG8"])
}"#
        .to_string(),
        r#"// Example 6: Group-scoped multi-qualifier — "Wenn SEQ+Z01/Z80/Z81 vorhanden"
fn evaluate_17(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.any_group_has_any_qualifier("SEQ", 0, &["Z01", "Z80", "Z81"], &["SG4", "SG8"])
}"#
        .to_string(),
        r#"// Example 7: Group-scoped co-occurrence — "SEQ+Z01 AND CCI+Z30++Z07 in same SG8"
fn evaluate_20(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.any_group_has_co_occurrence(
        "SEQ", 0, &["Z01"],
        "CCI", 2, 0, &["Z07"],
        &["SG4", "SG8"],
    )
}"#
        .to_string(),
        r#"// Example 8: Low-level element value check (when helpers don't fit)
// Check DTM+Z01 4th character for 'T' or 'R'
fn evaluate_27(&self, ctx: &EvaluationContext) -> ConditionResult {
    let dtm_segments = ctx.find_segments_with_qualifier("DTM", 0, "Z01");
    match dtm_segments.first() {
        Some(dtm) => {
            match dtm.elements.first().and_then(|e| e.get(1)).map(|s| s.as_str()) {
                Some(value) => match value.chars().nth(3) {
                    Some('T') | Some('R') => ConditionResult::True,
                    Some(_) => ConditionResult::False,
                    None => ConditionResult::Unknown,
                },
                None => ConditionResult::Unknown,
            }
        }
        None => ConditionResult::Unknown,
    }
}"#
        .to_string(),
        r#"// Example 9: Parent-child qualifier check — "Wenn in dieser SG8 SEQ+Z98 SG10 CCI+++ZA6 CAV+E02 vorhanden"
// SG10 is a CHILD GROUP of SG8. Use filtered_parent_child_has_qualifier to:
// 1. Find SG8 instances where SEQ has qualifier Z98
// 2. Check if their SG10 children have CCI with qualifier ZA6
fn evaluate_118(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.filtered_parent_child_has_qualifier(
        &["SG4", "SG8"], "SEQ", 0, "Z98",
        "SG10", "CCI", 2, "ZA6",
    )
}"#
        .to_string(),
        r#"// Example 10: Parent-child with CAV value check — "Wenn SG10 CCI+15++Z21 CAV+AUS vorhanden"
// SG10 CCI and CAV are segments inside child group SG10 of parent SG8.
// filtered_parent_child_has_qualifier checks the child's CCI qualifier.
// For the CAV value, use find_segments_in_child_group for detailed access.
fn evaluate_377(&self, ctx: &EvaluationContext) -> ConditionResult {
    let nav = match ctx.navigator() {
        Some(n) => n,
        None => return ctx.has_qualified_value("CAV", 0, "AUS", 0, 0, &["AUS"]),
    };
    let sg8_count = nav.group_instance_count(&["SG4", "SG8"]);
    for i in 0..sg8_count {
        let sg10_count = nav.child_group_instance_count(&["SG4", "SG8"], i, "SG10");
        for j in 0..sg10_count {
            let ccis = nav.find_segments_in_child_group("CCI", &["SG4", "SG8"], i, "SG10", j);
            let has_cci = ccis.iter().any(|s| {
                s.elements.get(0).and_then(|e| e.first()).is_some_and(|v| v == "15")
                && s.elements.get(2).and_then(|e| e.first()).is_some_and(|v| v == "Z21")
            });
            if has_cci {
                let cavs = nav.find_segments_in_child_group("CAV", &["SG4", "SG8"], i, "SG10", j);
                if cavs.iter().any(|s| s.elements.first().and_then(|e| e.first()).is_some_and(|v| v == "AUS")) {
                    return ConditionResult::True;
                }
            }
        }
    }
    ConditionResult::False
}"#
        .to_string(),
        r#"// Example 11: Same-group absence — "Wenn in derselben SG8 SEQ+Z59 das PIA+5 nicht vorhanden"
// Checks that in some SG8 instance, SEQ+Z59 IS present but PIA+5 IS NOT.
fn evaluate_111(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.any_group_has_qualifier_without(
        "SEQ", 0, "Z59",
        "PIA", 0, "5",
        &["SG4", "SG8"],
    )
}"#
        .to_string(),
        r#"// Example 12: Same-group absence for child — "Wenn in derselben SG8 das SG10 CCI+11 nicht vorhanden"
// SG10 is a child of SG8. Check if any SG8 instance has NO SG10 child with CCI qualifier "11".
fn evaluate_445(&self, ctx: &EvaluationContext) -> ConditionResult {
    let result = ctx.filtered_parent_child_has_qualifier(
        &["SG4", "SG8"], "SEQ", 0, "Z59",
        "SG10", "CCI", 0, "11",
    );
    // Negate: we want absence (True when CCI+11 is NOT found)
    match result {
        ConditionResult::True => ConditionResult::False,
        ConditionResult::False => ConditionResult::True,
        ConditionResult::Unknown => ConditionResult::Unknown,
    }
}"#
        .to_string(),
        r#"// Example 13: Zeitraum-ID cross-reference — collect values from one group, match in another
fn evaluate_306(&self, ctx: &EvaluationContext) -> ConditionResult {
    // "SG5 LOC+Z22 with same Zeitraum-ID as SG8 SEQ+Z58 DE1050"
    // Collect Zeitraum-IDs from SG5 LOC+Z22 (element 2 = DE3224)
    let loc_values = ctx.collect_group_values("LOC", 2, 0, &["SG4", "SG5"]);
    let loc_z22_ids: Vec<&str> = loc_values.iter()
        .filter_map(|(_, _v)| None::<&str>) // placeholder — real code checks LOC qualifier
        .collect();
    // Collect Zeitraum-IDs from SG8 SEQ+Z58 (element 1 = DE1050)
    let seq_values = ctx.collect_group_values("SEQ", 1, 0, &["SG4", "SG8"]);
    if loc_z22_ids.is_empty() || seq_values.is_empty() {
        return ConditionResult::Unknown;
    }
    // Check if any SEQ Zeitraum-ID matches a LOC Zeitraum-ID
    ConditionResult::from(seq_values.iter().any(|(_, v)| loc_z22_ids.contains(&v.as_str())))
}"#
        .to_string(),
        r#"// Example 14: Same-group RFF absence — "Wenn in derselben SG8 das RFF+Z14 nicht vorhanden"
fn evaluate_316(&self, ctx: &EvaluationContext) -> ConditionResult {
    // SEQ is always present in SG8 (entry segment), RFF+Z14 may be absent
    // Use any_group_has_qualifier_without: SEQ present (any qualifier), RFF+Z14 absent
    // But since we want "any SG8 where RFF+Z14 is absent", use lacks_qualifier as fallback
    ctx.any_group_has_qualifier_without(
        "SEQ", 0, "Z03",  // present: SEQ+Z03 or ZF5
        "RFF", 0, "Z14",  // absent: RFF+Z14
        &["SG4", "SG8"],
    )
}"#
        .to_string(),
        r#"// Example 15: Hinweis (informational note) — always True, high confidence
// "Hinweis: Nur ein Wert ist in DE3148 zu übermitteln."
fn evaluate_534(&self, _ctx: &EvaluationContext) -> ConditionResult {
    // Hinweis: informational annotation, always applies
    ConditionResult::True
}"#
        .to_string(),
        r#"// Example 16: Multi-element match — "Wenn STS+Z20++Z32++A99"
// Check if STS segment exists where elements[0][0]=="Z20" AND elements[1][0]=="Z32" AND elements[2][0]=="A99"
fn evaluate_550(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.has_segment_matching("STS", &[(0, 0, "Z20"), (1, 0, "Z32"), (2, 0, "A99")])
}"#
        .to_string(),
        r#"// Example 17: DTM date comparison — "Wenn Lieferbeginn (DTM+137) >= 01.10.2025"
fn evaluate_551(&self, ctx: &EvaluationContext) -> ConditionResult {
    ctx.dtm_ge("137", "202510010000")
}"#
        .to_string(),
        r#"// Example 18: Group cardinality — "Wenn mehr als ein CCI+Z23 in SG8 vorhanden"
fn evaluate_552(&self, ctx: &EvaluationContext) -> ConditionResult {
    ConditionResult::from(ctx.count_qualified_in_group("CCI", 0, "Z23", &["SG4", "SG8"]) > 1)
}"#
        .to_string(),
    ]
}
