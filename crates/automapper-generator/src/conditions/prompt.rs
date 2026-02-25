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

**Helper methods on EvaluationContext:**
- `ctx.find_segment("NAD")` → `Option<&OwnedSegment>` — first segment with that ID
- `ctx.find_segments("NAD")` → `Vec<&OwnedSegment>` — all segments with that ID
- `ctx.find_segments_with_qualifier("NAD", 0, "MS")` → `Vec<&OwnedSegment>` — segments where `elements[0][0] == "MS"`
- `ctx.has_segment("LOC")` → `bool` — whether any segment with that ID exists

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

Example: `LOC+Z16+12345678900`
→ `elements[0] = ["Z16"]`           (qualifier: Z16=Marktlokation)
→ `elements[1] = ["12345678900"]`   (location ID)

Example: `DTM+92:202505312200?+00:303`
→ `elements[0] = ["92", "202505312200+00", "303"]`  (qualifier, value, format code)

Example: `RFF+Z13:55001`
→ `elements[0] = ["Z13", "55001"]`  (qualifier, reference value)

**Accessing element values safely:**
```rust
// Get element i, component j:
segment.elements.get(i).and_then(|e| e.get(j)).map(|s| s.as_str())

// Check qualifier (element 0, component 0):
segment.elements.get(0).and_then(|e| e.first()).is_some_and(|v| v == "MS")
```

## ConditionResult

```rust
pub enum ConditionResult { True, False, Unknown }
```

Return `Unknown` when the condition cannot be determined from the available segments.

## ExternalConditionProvider

```rust
pub trait ExternalConditionProvider: Send + Sync {
    fn evaluate(&self, condition_name: &str) -> ConditionResult;
}
```

## Confidence levels
- **high**: Simple segment existence checks, qualifier comparisons, value matches
- **medium**: Logic requiring some interpretation but structurally clear
- **low**: Complex business rules that need clarification, temporal logic

## External conditions
Some conditions CANNOT be determined from the EDIFACT message alone — they depend on
runtime business context (market participant roles, clearing status, product configuration).

Mark such conditions with `"is_external": true` and provide `"external_name"` with a
meaningful snake_case identifier.

Common external conditions in UTILMD:
- "Wenn Aufteilung vorhanden" → message_splitting
- "Datenclearing erforderlich" → data_clearing_required
- "Wenn MP-ID in ... in der Rolle LF/NB/ÜNB/MSB" → these check market participant roles which are NOT in the EDIFACT data. Mark as external with name like `recipient_is_lf`, `sender_is_nb`, etc.

## CRITICAL: AHB notation vs actual element indices
AHB condition descriptions use shorthand EDIFACT notation that OMITS intermediate elements.
For example, `STS+7++ZG9` does NOT mean ZG9 is at elements[2]. The `++` represents ONE empty
element, but there may be MORE elements in the full segment structure.

**Always use the "EDIFACT Segment Structure Reference" (provided in the user prompt) for
authoritative element positions.** That reference is derived from the MIG XML schema and shows
the exact `elements[N]` index for every data element and composite. Never guess element indices
from the AHB shorthand notation alone.

## IMPORTANT rules for generated code
1. Only use the `EvaluationContext` API described above. Do NOT invent fields like `ctx.transaktion`, `ctx.prozessdaten`, etc.
2. Access segment data through `ctx.find_segment()`, `ctx.find_segments_with_qualifier()`, and element indexing.
3. Use `.get()` and `.and_then()` for safe element access — never panic on missing data.
4. When a condition references a specific segment qualifier (e.g., "STS+7", "NAD+MR", "LOC+Z16"), use `ctx.find_segments_with_qualifier()`.
5. For **low confidence** conditions, set implementation to null.
6. **Always consult the Segment Structure Reference for element indices.** Do not derive indices from EDIFACT shorthand notation in condition descriptions.

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

    // Add conditions list
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
    }

    prompt.push_str("\nGenerate the JSON response with implementations for all conditions.\n");

    prompt
}

/// Extracts segment IDs referenced in condition descriptions and builds
/// a compact segment structure reference from the MIG schema.
fn build_segment_structure_context(mig: &MigSchema, conditions: &[ConditionInput]) -> String {
    use regex::Regex;

    let de_regex =
        Regex::new(r"(?i)(?:SG\d+\s+)?([A-Z]{3})(?:\+[A-Z0-9]+)?\s+DE(\d{4})").unwrap();
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
    context.push_str("These are the AUTHORITATIVE element positions. Use these indices in your code.\n");
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

/// Default example implementations for few-shot prompting.
pub fn default_example_implementations() -> Vec<String> {
    vec![
        r#"// Example 1: Check if a LOC+Z16 (Marktlokation) segment exists
fn evaluate_494(&self, ctx: &EvaluationContext) -> ConditionResult {
    if ctx.find_segments_with_qualifier("LOC", 0, "Z16").is_empty() {
        ConditionResult::False
    } else {
        ConditionResult::True
    }
}"#
        .to_string(),
        r#"// Example 2: Check STS Transaktionsgrund value (element 3)
// EDIFACT: STS+7++E01+ZW4+E03 → elements[3][0] = "ZW4"
fn evaluate_501(&self, ctx: &EvaluationContext) -> ConditionResult {
    let sts_segments = ctx.find_segments_with_qualifier("STS", 0, "7");
    match sts_segments.first() {
        Some(sts) => {
            match sts.elements.get(3).and_then(|e| e.first()).map(|s| s.as_str()) {
                Some("E01") | Some("E02") => ConditionResult::True,
                Some(_) => ConditionResult::False,
                None => ConditionResult::Unknown,
            }
        }
        None => ConditionResult::Unknown,
    }
}"#
        .to_string(),
        r#"// Example 3: External condition — cannot determine from message alone
fn evaluate_1(&self, ctx: &EvaluationContext) -> ConditionResult {
    // "Wenn Aufteilung vorhanden" — requires external business context
    ctx.external.evaluate("message_splitting")
}"#
        .to_string(),
        r#"// Example 4: Check RFF reference value
// EDIFACT: RFF+Z13:55001 → elements[0] = ["Z13", "55001"]
fn evaluate_17(&self, ctx: &EvaluationContext) -> ConditionResult {
    let rff_segments = ctx.find_segments_with_qualifier("RFF", 0, "Z13");
    if rff_segments.is_empty() {
        ConditionResult::False
    } else {
        ConditionResult::True
    }
}"#
        .to_string(),
        r#"// Example 5: Check DTM date qualifier exists
// EDIFACT: DTM+92:202505312200+00:303 → elements[0] = ["92", "202505312200+00", "303"]
fn evaluate_42(&self, ctx: &EvaluationContext) -> ConditionResult {
    let dtm_segments = ctx.find_segments_with_qualifier("DTM", 0, "92");
    if dtm_segments.is_empty() {
        ConditionResult::False
    } else {
        ConditionResult::True
    }
}"#
        .to_string(),
    ]
}
