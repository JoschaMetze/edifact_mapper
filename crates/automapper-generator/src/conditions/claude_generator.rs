use std::io::Write;
use std::process::{Command, Stdio};

use crate::conditions::condition_types::*;
use crate::conditions::prompt::{self, ConditionContext};
use crate::error::GeneratorError;

/// Shells out to the `claude` CLI to generate condition evaluator code.
///
/// Uses `claude --print` with a JSON-structured prompt. No SDK dependency --
/// reuses the user's existing Claude CLI subscription.
pub struct ClaudeConditionGenerator {
    /// Maximum concurrent Claude CLI calls for batch generation.
    pub max_concurrent: usize,
}

impl ClaudeConditionGenerator {
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    /// Generates conditions for a single batch by shelling out to `claude --print`.
    ///
    /// Returns parsed `GeneratedCondition` structs.
    pub fn generate_batch(
        &self,
        conditions: &[ConditionInput],
        context: &ConditionContext<'_>,
    ) -> Result<Vec<GeneratedCondition>, GeneratorError> {
        let system_prompt = prompt::build_system_prompt();
        let user_prompt = prompt::build_user_prompt(conditions, context);

        let full_prompt = format!("{}\n\n---\n\n{}", system_prompt, user_prompt);

        let raw_response = self.invoke_claude_cli(&full_prompt)?;
        let parsed = self.parse_response(&raw_response)?;

        Ok(parsed)
    }

    /// Invokes the `claude` CLI with `--print` flag and returns stdout.
    fn invoke_claude_cli(&self, prompt: &str) -> Result<String, GeneratorError> {
        let mut child = Command::new("claude")
            .args(["--print", "--model", "sonnet"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GeneratorError::ClaudeCli {
                message: format!("failed to spawn claude CLI: {}", e),
            })?;

        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| GeneratorError::ClaudeCli {
                    message: format!("failed to write to claude stdin: {}", e),
                })?;
        }
        // Drop stdin to signal EOF
        drop(child.stdin.take());

        let output = child
            .wait_with_output()
            .map_err(|e| GeneratorError::ClaudeCli {
                message: format!("claude CLI failed: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GeneratorError::ClaudeCli {
                message: format!(
                    "claude CLI exited with status {}: {}",
                    output.status, stderr
                ),
            });
        }

        String::from_utf8(output.stdout).map_err(|e| GeneratorError::ClaudeCli {
            message: format!("claude CLI returned invalid UTF-8: {}", e),
        })
    }

    /// Parses the JSON response from Claude into GeneratedCondition structs.
    ///
    /// Handles truncated responses by recovering as many complete condition
    /// entries as possible from the partial JSON.
    pub fn parse_response(
        &self,
        raw_response: &str,
    ) -> Result<Vec<GeneratedCondition>, GeneratorError> {
        let cleaned = strip_markdown_code_blocks(raw_response);

        // Try full parse first
        let entries: Vec<ClaudeConditionEntry> = if let Ok(response) =
            serde_json::from_str::<ClaudeConditionResponse>(&cleaned)
        {
            response.conditions
        } else {
            // Response may be truncated — try to recover individual condition objects
            let recovered = recover_partial_conditions(&cleaned);
            if recovered.is_empty() {
                return Err(GeneratorError::ClaudeCli {
                        message: format!(
                            "failed to parse Claude JSON response (no recoverable conditions). Response was: {}",
                            &cleaned[..cleaned.len().min(500)]
                        ),
                    });
            }
            eprintln!(
                "WARNING: Response was truncated, recovered {} complete conditions",
                recovered.len()
            );
            recovered
        };

        let mut results = Vec::new();
        for entry in entries {
            let condition_number: u32 =
                entry.id.parse().map_err(|e| GeneratorError::ClaudeCli {
                    message: format!("invalid condition ID '{}': {}", entry.id, e),
                })?;

            let confidence: ConfidenceLevel =
                entry.confidence.parse().unwrap_or(ConfidenceLevel::Medium);

            results.push(GeneratedCondition {
                condition_number,
                rust_code: entry.implementation,
                is_external: entry.is_external,
                confidence,
                reasoning: entry.reasoning,
                external_name: entry.external_name,
                original_description: None,
                referencing_fields: None,
            });
        }

        Ok(results)
    }
}

/// Strips markdown code block wrappers (```json ... ```) from a response string.
/// Handles both complete (``` ... ```) and truncated (``` ... <eof>) blocks.
fn strip_markdown_code_blocks(response: &str) -> String {
    let trimmed = response.trim();

    if trimmed.starts_with("```") {
        let rest = if let Some(newline_pos) = trimmed.find('\n') {
            &trimmed[newline_pos + 1..]
        } else {
            return trimmed.to_string();
        };

        // Complete block: strip closing ```
        if let Some(stripped) = rest.strip_suffix("```") {
            return stripped.trim().to_string();
        }

        // Truncated block: no closing ```, just strip the opening line
        return rest.trim().to_string();
    }

    trimmed.to_string()
}

/// Recovers complete condition entries from a truncated JSON response.
///
/// Finds each `{ "id": ... }` object within the conditions array by
/// tracking brace depth, and parses those that are complete.
fn recover_partial_conditions(json: &str) -> Vec<ClaudeConditionEntry> {
    let mut results = Vec::new();

    // Find the start of the conditions array
    let Some(arr_start) = json.find('"').and_then(|_| json.find('[')) else {
        return results;
    };

    let bytes = json.as_bytes();
    let mut i = arr_start + 1;

    while i < bytes.len() {
        // Skip whitespace and commas
        if bytes[i] == b' '
            || bytes[i] == b'\n'
            || bytes[i] == b'\r'
            || bytes[i] == b'\t'
            || bytes[i] == b','
        {
            i += 1;
            continue;
        }

        // Found start of an object
        if bytes[i] == b'{' {
            let obj_start = i;
            let mut depth = 1;
            i += 1;

            // Track whether we're inside a string (to handle braces in string values)
            let mut in_string = false;
            let mut escape_next = false;

            while i < bytes.len() && depth > 0 {
                if escape_next {
                    escape_next = false;
                    i += 1;
                    continue;
                }
                match bytes[i] {
                    b'\\' if in_string => escape_next = true,
                    b'"' => in_string = !in_string,
                    b'{' if !in_string => depth += 1,
                    b'}' if !in_string => depth -= 1,
                    _ => {}
                }
                i += 1;
            }

            if depth == 0 {
                // Complete object — try to parse it
                let obj_str = &json[obj_start..i];
                if let Ok(entry) = serde_json::from_str::<ClaudeConditionEntry>(obj_str) {
                    results.push(entry);
                }
            }
            // If depth > 0, the object was truncated — skip it
        } else {
            // End of array or unexpected character
            break;
        }
    }

    results
}
