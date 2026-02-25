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
    pub fn parse_response(
        &self,
        raw_response: &str,
    ) -> Result<Vec<GeneratedCondition>, GeneratorError> {
        let cleaned = strip_markdown_code_blocks(raw_response);

        let response: ClaudeConditionResponse =
            serde_json::from_str(&cleaned).map_err(|e| GeneratorError::ClaudeCli {
                message: format!(
                    "failed to parse Claude JSON response: {}. Response was: {}",
                    e,
                    &cleaned[..cleaned.len().min(500)]
                ),
            })?;

        let mut results = Vec::new();
        for entry in response.conditions {
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
fn strip_markdown_code_blocks(response: &str) -> String {
    let trimmed = response.trim();

    if trimmed.starts_with("```") {
        let rest = if let Some(newline_pos) = trimmed.find('\n') {
            &trimmed[newline_pos + 1..]
        } else {
            trimmed
        };

        if let Some(stripped) = rest.strip_suffix("```") {
            return stripped.trim().to_string();
        }
    }

    trimmed.to_string()
}
