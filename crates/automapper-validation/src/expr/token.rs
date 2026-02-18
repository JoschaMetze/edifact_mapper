//! Tokenizer for condition expression strings.

use crate::error::ParseError;

/// Token types produced by the condition expression tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A condition reference number, e.g., `931` from `[931]`.
    ConditionId(String),
    /// AND operator (`∧` or `AND`).
    And,
    /// OR operator (`∨` or `OR`).
    Or,
    /// XOR operator (`⊻` or `XOR`).
    Xor,
    /// NOT operator (`NOT`).
    Not,
    /// Opening parenthesis `(`.
    LeftParen,
    /// Closing parenthesis `)`.
    RightParen,
}

/// A token with its position in the source string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub position: usize,
}

/// AHB status prefixes that are stripped before tokenizing.
const STATUS_PREFIXES: &[&str] = &["Muss", "Soll", "Kann", "X"];

/// Strip the AHB status prefix (Muss, Soll, Kann, X) from the input.
///
/// Returns the remainder of the string after the prefix, or the original
/// string if no prefix is found.
pub fn strip_status_prefix(input: &str) -> &str {
    let trimmed = input.trim();
    for prefix in STATUS_PREFIXES {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let rest = rest.trim_start();
            if !rest.is_empty() {
                return rest;
            }
        }
    }
    trimmed
}

/// Tokenize an AHB condition expression string.
///
/// The input should already have the status prefix stripped.
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, ParseError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        let position = i;

        // Parentheses
        if c == '(' {
            tokens.push(SpannedToken {
                token: Token::LeftParen,
                position,
            });
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(SpannedToken {
                token: Token::RightParen,
                position,
            });
            i += 1;
            continue;
        }

        // Unicode operators
        if c == '\u{2227}' {
            // ∧ AND
            tokens.push(SpannedToken {
                token: Token::And,
                position,
            });
            i += 1;
            continue;
        }
        if c == '\u{2228}' {
            // ∨ OR
            tokens.push(SpannedToken {
                token: Token::Or,
                position,
            });
            i += 1;
            continue;
        }
        if c == '\u{22BB}' {
            // ⊻ XOR
            tokens.push(SpannedToken {
                token: Token::Xor,
                position,
            });
            i += 1;
            continue;
        }

        // Condition reference [...]
        if c == '[' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            if i < chars.len() {
                let content: String = chars[start + 1..i].iter().collect();
                tokens.push(SpannedToken {
                    token: Token::ConditionId(content),
                    position: start,
                });
                i += 1; // skip closing ]
            } else {
                let content: String = chars[start + 1..].iter().collect();
                return Err(ParseError::InvalidConditionRef { content });
            }
            continue;
        }

        // Text keywords: AND, OR, XOR, NOT (case-insensitive)
        if c.is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            match word.to_uppercase().as_str() {
                "AND" => tokens.push(SpannedToken {
                    token: Token::And,
                    position: start,
                }),
                "OR" => tokens.push(SpannedToken {
                    token: Token::Or,
                    position: start,
                }),
                "XOR" => tokens.push(SpannedToken {
                    token: Token::Xor,
                    position: start,
                }),
                "NOT" => tokens.push(SpannedToken {
                    token: Token::Not,
                    position: start,
                }),
                _ => {
                    // Skip unknown words (could be status prefix remnants)
                }
            }
            continue;
        }

        // Skip unknown characters
        i += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- strip_status_prefix tests ---

    #[test]
    fn test_strip_muss_prefix() {
        assert_eq!(strip_status_prefix("Muss [494]"), "[494]");
    }

    #[test]
    fn test_strip_soll_prefix() {
        assert_eq!(strip_status_prefix("Soll [494]"), "[494]");
    }

    #[test]
    fn test_strip_kann_prefix() {
        assert_eq!(strip_status_prefix("Kann [182] ∧ [6]"), "[182] ∧ [6]");
    }

    #[test]
    fn test_strip_x_prefix() {
        assert_eq!(
            strip_status_prefix("X (([939][14]) ∨ ([940][15]))"),
            "(([939][14]) ∨ ([940][15]))"
        );
    }

    #[test]
    fn test_strip_no_prefix() {
        assert_eq!(strip_status_prefix("[1] ∧ [2]"), "[1] ∧ [2]");
    }

    #[test]
    fn test_strip_muss_only_returns_trimmed() {
        // "Muss" alone with nothing after has no conditions
        assert_eq!(strip_status_prefix("Muss"), "Muss");
    }

    #[test]
    fn test_strip_whitespace_only() {
        assert_eq!(strip_status_prefix("   "), "");
    }

    #[test]
    fn test_strip_preserves_leading_whitespace_in_content() {
        assert_eq!(strip_status_prefix("Muss   [1]"), "[1]");
    }

    // --- tokenize tests ---

    #[test]
    fn test_tokenize_single_condition() {
        let tokens = tokenize("[931]").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::ConditionId("931".to_string()));
    }

    #[test]
    fn test_tokenize_and_unicode() {
        let tokens = tokenize("[1] ∧ [2]").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::ConditionId("1".to_string()));
        assert_eq!(tokens[1].token, Token::And);
        assert_eq!(tokens[2].token, Token::ConditionId("2".to_string()));
    }

    #[test]
    fn test_tokenize_or_unicode() {
        let tokens = tokenize("[1] ∨ [2]").unwrap();
        assert_eq!(tokens[1].token, Token::Or);
    }

    #[test]
    fn test_tokenize_xor_unicode() {
        let tokens = tokenize("[1] ⊻ [2]").unwrap();
        assert_eq!(tokens[1].token, Token::Xor);
    }

    #[test]
    fn test_tokenize_text_keywords() {
        let tokens = tokenize("[1] AND [2] OR [3] XOR [4]").unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[1].token, Token::And);
        assert_eq!(tokens[3].token, Token::Or);
        assert_eq!(tokens[5].token, Token::Xor);
    }

    #[test]
    fn test_tokenize_not_keyword() {
        let tokens = tokenize("NOT [1]").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Not);
        assert_eq!(tokens[1].token, Token::ConditionId("1".to_string()));
    }

    #[test]
    fn test_tokenize_parentheses() {
        let tokens = tokenize("([1] ∨ [2]) ∧ [3]").unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].token, Token::LeftParen);
        assert_eq!(tokens[4].token, Token::RightParen);
    }

    #[test]
    fn test_tokenize_adjacent_conditions_no_space() {
        let tokens = tokenize("[939][14]").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::ConditionId("939".to_string()));
        assert_eq!(tokens[1].token, Token::ConditionId("14".to_string()));
    }

    #[test]
    fn test_tokenize_package_condition() {
        let tokens = tokenize("[10P1..5]").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::ConditionId("10P1..5".to_string()));
    }

    #[test]
    fn test_tokenize_time_condition() {
        let tokens = tokenize("[UB1]").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::ConditionId("UB1".to_string()));
    }

    #[test]
    fn test_tokenize_tabs_and_multiple_spaces() {
        let tokens = tokenize("[1]\t∧\t[2]").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].token, Token::And);
    }

    #[test]
    fn test_tokenize_multiple_spaces() {
        let tokens = tokenize("[1]    ∧    [2]").unwrap();
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokens = tokenize("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_complex_real_world() {
        // "X (([939] [147]) ∨ ([940] [148])) ∧ [567]"
        // After prefix strip: "(([939] [147]) ∨ ([940] [148])) ∧ [567]"
        let tokens = tokenize("(([939] [147]) ∨ ([940] [148])) ∧ [567]").unwrap();
        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[0].token, Token::LeftParen);
        assert_eq!(tokens[1].token, Token::LeftParen);
        assert_eq!(tokens[2].token, Token::ConditionId("939".to_string()));
        assert_eq!(tokens[3].token, Token::ConditionId("147".to_string()));
        assert_eq!(tokens[4].token, Token::RightParen);
        assert_eq!(tokens[5].token, Token::Or);
        assert_eq!(tokens[6].token, Token::LeftParen);
        assert_eq!(tokens[7].token, Token::ConditionId("940".to_string()));
        assert_eq!(tokens[8].token, Token::ConditionId("148".to_string()));
        assert_eq!(tokens[9].token, Token::RightParen);
        assert_eq!(tokens[10].token, Token::RightParen);
        assert_eq!(tokens[11].token, Token::And);
        assert_eq!(tokens[12].token, Token::ConditionId("567".to_string()));
    }

    #[test]
    fn test_tokenize_positions_are_correct() {
        let tokens = tokenize("[1] ∧ [2]").unwrap();
        assert_eq!(tokens[0].position, 0); // [
        assert_eq!(tokens[2].position, 6); // [ of [2] (∧ is a single char in char index)
    }

    #[test]
    fn test_tokenize_case_insensitive_keywords() {
        let tokens = tokenize("[1] and [2] or [3]").unwrap();
        assert_eq!(tokens[1].token, Token::And);
        assert_eq!(tokens[3].token, Token::Or);
    }

    #[test]
    fn test_tokenize_unclosed_bracket_returns_error() {
        let result = tokenize("[931");
        assert!(result.is_err());
    }
}
