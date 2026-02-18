use crate::conditions::condition_types::ConditionInput;
use crate::schema::mig::MigSchema;

/// Context for condition generation, including segment structure and examples.
pub struct ConditionContext<'a> {
    /// The EDIFACT message type (e.g., "UTILMD").
    pub message_type: &'a str,
    /// The format version (e.g., "FV2510").
    pub format_version: &'a str,
    /// Optional MIG schema for segment structure context.
    pub mig_schema: Option<&'a MigSchema>,
    /// Example condition implementations for few-shot learning.
    pub example_implementations: Vec<String>,
}

/// Builds the system prompt for condition generation.
///
/// The system prompt instructs Claude to generate Rust condition evaluator functions
/// from German AHB condition descriptions.
pub fn build_system_prompt() -> String {
    r#"You are an expert Rust developer specializing in EDIFACT message validation.
Your task is to generate Rust condition evaluator functions from German AHB (Anwendungshandbuch) condition descriptions.

The generated functions will be used in a struct that implements ConditionEvaluator:
```rust
pub trait ConditionEvaluator: Send + Sync {
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult;
    fn is_external(&self, condition: u32) -> bool;
}
```

Each condition is implemented as a method with this signature:
```rust
fn evaluate_NNN(&self, ctx: &EvaluationContext) -> ConditionResult {
    // Your implementation here
}
```

**EvaluationContext API:**
- `ctx.transaktion` - the `UtilmdTransaktion` struct with all parsed business objects
- `ctx.pruefidentifikator` - the current Pruefidentifikator being validated
- `ctx.external` - an `&dyn ExternalConditionProvider` for conditions requiring runtime business context

**UtilmdTransaktion fields:**
- `ctx.transaktion.marktlokationen` - `Vec<WithValidity<Marktlokation, MarktlokationEdifact>>`
- `ctx.transaktion.messlokationen` - `Vec<WithValidity<Messlokation, MesslokationEdifact>>`
- `ctx.transaktion.netzlokationen` - `Vec<WithValidity<Netzlokation, NetzlokationEdifact>>`
- `ctx.transaktion.zaehler` - `Vec<WithValidity<Zaehler, ZaehlerEdifact>>`
- `ctx.transaktion.parteien` - `Vec<WithValidity<Geschaeftspartner, GeschaeftspartnerEdifact>>`
- `ctx.transaktion.vertrag` - `Option<WithValidity<Vertrag, VertragEdifact>>`
- `ctx.transaktion.prozessdaten` - `Prozessdaten`
- `ctx.transaktion.zeitscheiben` - `Vec<Zeitscheibe>`
- `ctx.transaktion.steuerbare_ressourcen` - `Vec<WithValidity<SteuerbareRessource, SteuerbareRessourceEdifact>>`
- `ctx.transaktion.technische_ressourcen` - `Vec<WithValidity<TechnischeRessource, TechnischeRessourceEdifact>>`

**ConditionResult:**
```rust
pub enum ConditionResult { True, False, Unknown }
```

Return `ConditionResult::Unknown` when the condition cannot be determined from the available data.

**Confidence levels:**
- **high**: Simple field existence checks or value comparisons
- **medium**: Logic that requires some interpretation but is straightforward
- **low**: Complex temporal logic, business rules that need clarification

**External conditions:**
Some conditions CANNOT be determined from the message alone. These depend on external runtime context such as:
- Message splitting status ("Wenn Aufteilung vorhanden")
- Data clearing requirements ("Datenclearing erforderlich")

Mark such conditions with `"is_external": true`. For external conditions, provide an `"external_name"` field with a meaningful snake_case name.

For **low confidence** conditions, set implementation to null.

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
    context.push_str("## EDIFACT Segment Structure Reference\n");
    context.push_str("This shows how to access data elements from the parsed transaction.\n\n");

    // Include segment definitions from MIG that are referenced
    for segment in &mig.segments {
        if referenced_segments.contains(&segment.id.to_uppercase()) {
            context.push_str(&format!("### {} - {}\n", segment.id, segment.name));
            for de in &segment.data_elements {
                context.push_str(&format!(
                    "- DE{} ({}): element position {}\n",
                    de.id, de.name, de.position
                ));
            }
            context.push('\n');
        }
    }

    // Also check segments inside groups
    for group in &mig.segment_groups {
        append_group_segments(&mut context, group, &referenced_segments);
    }

    context
}

fn append_group_segments(
    context: &mut String,
    group: &crate::schema::mig::MigSegmentGroup,
    referenced: &std::collections::HashSet<String>,
) {
    for segment in &group.segments {
        if referenced.contains(&segment.id.to_uppercase()) {
            context.push_str(&format!(
                "### {} - {} (in {})\n",
                segment.id, segment.name, group.id
            ));
            for de in &segment.data_elements {
                context.push_str(&format!(
                    "- DE{} ({}): element position {}\n",
                    de.id, de.name, de.position
                ));
            }
            context.push('\n');
        }
    }
    for nested in &group.nested_groups {
        append_group_segments(context, nested, referenced);
    }
}

/// Default example implementations for few-shot prompting.
pub fn default_example_implementations() -> Vec<String> {
    vec![
        r#"// Example 1: Field existence check
fn evaluate_494(&self, ctx: &EvaluationContext) -> ConditionResult {
    if ctx.transaktion.marktlokationen.is_empty() {
        ConditionResult::False
    } else {
        ConditionResult::True
    }
}"#
        .to_string(),
        r#"// Example 2: Value comparison
fn evaluate_501(&self, ctx: &EvaluationContext) -> ConditionResult {
    match ctx.transaktion.prozessdaten.kategorie.as_deref() {
        Some("E01") | Some("E02") => ConditionResult::True,
        Some(_) => ConditionResult::False,
        None => ConditionResult::Unknown,
    }
}"#
        .to_string(),
        r#"// Example 3: External condition
fn evaluate_1(&self, ctx: &EvaluationContext) -> ConditionResult {
    // "Wenn Aufteilung vorhanden" — requires external context
    ctx.external.evaluate("message_splitting")
}"#
        .to_string(),
    ]
}
