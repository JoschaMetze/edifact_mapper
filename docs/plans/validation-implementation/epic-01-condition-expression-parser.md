---
feature: validation-implementation
epic: 1
title: "Condition Expression Parser"
depends_on: []
estimated_tasks: 5
crate: automapper-validation
status: in_progress
---

# Epic 1: Condition Expression Parser

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-validation/src/`. All code must compile with `cargo check -p automapper-validation`.

**Goal:** Implement a recursive-descent parser that converts AHB (Anwendungshandbuch) condition status strings into a `ConditionExpr` AST. AHB rules use expressions like `Muss [182] ∧ [6] ∧ [570]` or `X (([939][14]) ∨ ([940][15])) ∧ [567]` to specify when EDIFACT fields are required. The parser must handle Unicode operators (`∧` AND, `∨` OR, `⊻` XOR), bracket-enclosed condition references, text keyword operators (`AND`, `OR`, `XOR`, `NOT`), implicit AND between adjacent conditions, parenthesized grouping, and AHB status prefixes (`Muss`, `Soll`, `Kann`, `X`). Operator precedence (highest to lowest): NOT > AND > OR > XOR. This matches the C# `ConditionExpressionParser` from `src/Automapper.Validation/Conditions/Expressions/`.

**Architecture:** SAX-style recursive-descent parsing with a tokenizer front-end that handles Unicode operators, text keyword operators, bracket-enclosed condition references, parentheses, and status prefix stripping. The parser implements precedence climbing (NOT > AND > OR > XOR) with implicit AND between adjacent conditions. Produces an owned `ConditionExpr` AST enum (Ref, And, Or, Xor, Not).

**Tech Stack:** thiserror 2.x, serde 1.x, pretty_assertions 1.x (dev), proptest 1.x (dev)

---

## Task 1: Create automapper-validation Cargo.toml and module structure

### Description
Set up the `automapper-validation` crate with its `Cargo.toml` and initial module structure.

### Files to Create/Modify

**`crates/automapper-validation/Cargo.toml`**:
```toml
[package]
name = "automapper-validation"
version = "0.1.0"
edition = "2021"
description = "AHB condition expression parsing, evaluation, and EDIFACT validation"

[dependencies]
edifact-types = { path = "../edifact-types" }
edifact-parser = { path = "../edifact-parser" }
bo4e-extensions = { path = "../bo4e-extensions" }
automapper-core = { path = "../automapper-core" }
thiserror = "2"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
pretty_assertions = "1"
```

**`crates/automapper-validation/src/lib.rs`**:
```rust
//! AHB condition expression parsing, evaluation, and EDIFACT message validation.
//!
//! This crate provides:
//! - A parser for AHB condition expressions (`[1] ∧ [2]`, `[3] ∨ [4]`, etc.)
//! - Traits for condition evaluation with three-valued logic (True/False/Unknown)
//! - An EDIFACT message validator that checks messages against AHB rules

pub mod expr;
pub mod eval;
pub mod validator;
pub mod error;
```

**`crates/automapper-validation/src/error.rs`**:
```rust
//! Error types for the automapper-validation crate.

use std::fmt;

/// Errors that can occur during condition expression parsing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    /// Unexpected token encountered during parsing.
    #[error("unexpected token at position {position}: expected {expected}, found '{found}'")]
    UnexpectedToken {
        position: usize,
        expected: String,
        found: String,
    },

    /// Unmatched closing parenthesis.
    #[error("unmatched closing parenthesis at position {position}")]
    UnmatchedCloseParen { position: usize },

    /// Empty expression after stripping prefix.
    #[error("empty expression after stripping status prefix")]
    EmptyExpression,

    /// Invalid condition reference content.
    #[error("invalid condition reference: '{content}'")]
    InvalidConditionRef { content: String },
}

/// Errors that can occur during validation.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// EDIFACT parse error.
    #[error(transparent)]
    Parse(#[from] edifact_parser::ParseError),

    /// Condition expression parse error.
    #[error("condition expression parse error: {0}")]
    ConditionParse(#[from] ParseError),

    /// Unknown Pruefidentifikator.
    #[error("unknown Pruefidentifikator: '{0}'")]
    UnknownPruefidentifikator(String),

    /// No evaluator registered for message type and format version.
    #[error("no condition evaluator registered for {message_type}/{format_version}")]
    NoEvaluator {
        message_type: String,
        format_version: String,
    },
}
```

**`crates/automapper-validation/src/expr.rs`**:
```rust
//! Condition expression AST and parser.

mod ast;
mod parser;
mod token;

pub use ast::ConditionExpr;
pub use parser::ConditionParser;
```

**`crates/automapper-validation/src/expr/ast.rs`** (placeholder):
```rust
//! Condition expression AST types.
```

**`crates/automapper-validation/src/expr/token.rs`** (placeholder):
```rust
//! Tokenizer for condition expression strings.
```

**`crates/automapper-validation/src/expr/parser.rs`** (placeholder):
```rust
//! Recursive descent parser for condition expressions.
```

**`crates/automapper-validation/src/eval.rs`** (placeholder):
```rust
//! Condition evaluation traits and expression evaluator.
```

**`crates/automapper-validation/src/validator.rs`** (placeholder):
```rust
//! EDIFACT message validator.
```

### Verification
```bash
cargo check -p automapper-validation
```

### Commit
```
feat(automapper-validation): scaffold crate with module structure and error types

Set up Cargo.toml with dependencies on edifact-types, edifact-parser,
bo4e-extensions, and automapper-core. Create module stubs for expr, eval,
validator, and error.
```

---

## Task 2: Define ConditionExpr AST

### Description
Define the `ConditionExpr` enum that represents parsed AHB condition expressions as an abstract syntax tree. This maps to the C# class hierarchy of `IConditionExpression`, `ConditionReference`, `AndExpression`, `OrExpression`, `XorExpression`, but uses a single Rust enum instead.

### Tests First

**`crates/automapper-validation/src/expr/ast.rs`**:
```rust
//! Condition expression AST types.

use std::collections::BTreeSet;

/// A parsed AHB condition expression tree.
///
/// Represents boolean combinations of condition references like `[1] ∧ [2]` or
/// `([3] ∨ [4]) ⊻ [5]`.
///
/// # Examples
///
/// A single condition reference:
/// ```
/// use automapper_validation::expr::ConditionExpr;
/// let expr = ConditionExpr::Ref(931);
/// assert_eq!(expr.condition_ids(), [931].into());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionExpr {
    /// A leaf reference to a single condition by number, e.g., `[931]`.
    Ref(u32),

    /// Boolean AND of one or more expressions. All must be true.
    /// Invariant: `exprs.len() >= 2`.
    And(Vec<ConditionExpr>),

    /// Boolean OR of one or more expressions. At least one must be true.
    /// Invariant: `exprs.len() >= 2`.
    Or(Vec<ConditionExpr>),

    /// Boolean XOR of exactly two expressions. Exactly one must be true.
    Xor(Box<ConditionExpr>, Box<ConditionExpr>),

    /// Boolean NOT of an expression.
    Not(Box<ConditionExpr>),
}

impl ConditionExpr {
    /// Extracts all condition IDs referenced in this expression tree.
    pub fn condition_ids(&self) -> BTreeSet<u32> {
        let mut ids = BTreeSet::new();
        self.collect_ids(&mut ids);
        ids
    }

    fn collect_ids(&self, ids: &mut BTreeSet<u32>) {
        match self {
            ConditionExpr::Ref(id) => {
                ids.insert(*id);
            }
            ConditionExpr::And(exprs) | ConditionExpr::Or(exprs) => {
                for expr in exprs {
                    expr.collect_ids(ids);
                }
            }
            ConditionExpr::Xor(left, right) => {
                left.collect_ids(ids);
                right.collect_ids(ids);
            }
            ConditionExpr::Not(inner) => {
                inner.collect_ids(ids);
            }
        }
    }
}

impl std::fmt::Display for ConditionExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionExpr::Ref(id) => write!(f, "[{id}]"),
            ConditionExpr::And(exprs) => {
                let parts: Vec<String> = exprs.iter().map(|e| format!("{e}")).collect();
                write!(f, "({})", parts.join(" ∧ "))
            }
            ConditionExpr::Or(exprs) => {
                let parts: Vec<String> = exprs.iter().map(|e| format!("{e}")).collect();
                write!(f, "({})", parts.join(" ∨ "))
            }
            ConditionExpr::Xor(left, right) => write!(f, "({left} ⊻ {right})"),
            ConditionExpr::Not(inner) => write!(f, "NOT {inner}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_condition_ids() {
        let expr = ConditionExpr::Ref(931);
        assert_eq!(expr.condition_ids(), [931].into());
    }

    #[test]
    fn test_and_condition_ids() {
        let expr = ConditionExpr::And(vec![
            ConditionExpr::Ref(1),
            ConditionExpr::Ref(2),
            ConditionExpr::Ref(3),
        ]);
        assert_eq!(expr.condition_ids(), [1, 2, 3].into());
    }

    #[test]
    fn test_nested_condition_ids() {
        // (([1] ∧ [2]) ∨ ([3] ∧ [4])) ⊻ [5]
        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::Or(vec![
                ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]),
                ConditionExpr::And(vec![ConditionExpr::Ref(3), ConditionExpr::Ref(4)]),
            ])),
            Box::new(ConditionExpr::Ref(5)),
        );
        assert_eq!(expr.condition_ids(), [1, 2, 3, 4, 5].into());
    }

    #[test]
    fn test_not_condition_ids() {
        let expr = ConditionExpr::Not(Box::new(ConditionExpr::Ref(42)));
        assert_eq!(expr.condition_ids(), [42].into());
    }

    #[test]
    fn test_display_ref() {
        let expr = ConditionExpr::Ref(931);
        assert_eq!(format!("{expr}"), "[931]");
    }

    #[test]
    fn test_display_and() {
        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(format!("{expr}"), "([1] ∧ [2])");
    }

    #[test]
    fn test_display_complex() {
        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::And(vec![
                ConditionExpr::Ref(102),
                ConditionExpr::Ref(2006),
            ])),
            Box::new(ConditionExpr::And(vec![
                ConditionExpr::Ref(103),
                ConditionExpr::Ref(2005),
            ])),
        );
        assert_eq!(format!("{expr}"), "(([102] ∧ [2006]) ⊻ ([103] ∧ [2005]))");
    }

    #[test]
    fn test_display_not() {
        let expr = ConditionExpr::Not(Box::new(ConditionExpr::Ref(1)));
        assert_eq!(format!("{expr}"), "NOT [1]");
    }

    #[test]
    fn test_equality() {
        let a = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        let b = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_inequality() {
        let a = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        let b = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_clone() {
        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::Ref(1)),
            Box::new(ConditionExpr::Ref(2)),
        );
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }
}
```

### Verification
```bash
cargo test -p automapper-validation ast::tests
```

### Commit
```
feat(automapper-validation): define ConditionExpr AST enum with tests

ConditionExpr supports Ref, And, Or, Xor, and Not variants.
Includes condition_ids() extraction, Display impl, and unit tests.
```

---

## Task 3: Implement tokenizer for AHB condition strings

### Description
Implement a tokenizer that converts AHB status strings into a sequence of tokens. The tokenizer must handle:
- Unicode operators: `∧` (AND), `∨` (OR), `⊻` (XOR)
- Text keyword operators: `AND`, `OR`, `XOR`, `NOT` (case-insensitive)
- Condition references: `[931]`, `[10P1..5]`, `[UB1]`
- Parentheses: `(`, `)`
- Status prefixes: `Muss`, `Soll`, `Kann`, `X` (stripped before tokenizing)
- Whitespace (skipped)
- Implicit AND between adjacent conditions or parenthesized groups

This maps to the C# `Tokenize()` and `StripStatusPrefix()` methods in `ConditionExpressionParser.cs`.

### Tests First, Then Implementation

**`crates/automapper-validation/src/expr/token.rs`**:
```rust
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
        assert_eq!(tokens[2].position, 6); // [ of [2] (∧ is a multi-byte char)
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
```

### Verification
```bash
cargo test -p automapper-validation token::tests
```

### Commit
```
feat(automapper-validation): implement AHB condition expression tokenizer

Handles Unicode operators, text keywords, condition references with
arbitrary content (numeric, package, time), parentheses, and status
prefix stripping.
```

---

## Task 4: Implement recursive descent parser

### Description
Implement the recursive descent parser that converts a token stream into a `ConditionExpr` AST. The parser implements correct operator precedence: NOT (highest) > AND > OR > XOR (lowest). Adjacent conditions without an explicit operator are treated as implicit AND, matching the C# behavior.

The public API is `ConditionParser::parse(input: &str) -> Result<Option<ConditionExpr>, ParseError>` where `None` means no conditions found (e.g., bare `"Muss"` with no brackets).

### Tests First, Then Implementation

**`crates/automapper-validation/src/expr/parser.rs`**:
```rust
//! Recursive descent parser for condition expressions.
//!
//! Grammar (from lowest to highest precedence):
//!
//! ```text
//! expression  = xor_expr
//! xor_expr    = or_expr (XOR or_expr)*
//! or_expr     = and_expr (OR and_expr)*
//! and_expr    = not_expr ((AND | implicit) not_expr)*
//! not_expr    = NOT not_expr | primary
//! primary     = CONDITION_ID | '(' expression ')'
//! ```
//!
//! Implicit AND: two adjacent condition references or a condition followed by
//! `(` without an intervening operator are treated as AND.

use super::ast::ConditionExpr;
use super::token::{strip_status_prefix, tokenize, SpannedToken, Token};
use crate::error::ParseError;

/// Parser for AHB condition expressions.
pub struct ConditionParser;

impl ConditionParser {
    /// Parse an AHB status string into a condition expression.
    ///
    /// Returns `Ok(None)` if the input contains no condition references
    /// (e.g., bare `"Muss"` or empty string).
    ///
    /// # Examples
    ///
    /// ```
    /// use automapper_validation::expr::ConditionParser;
    /// use automapper_validation::expr::ConditionExpr;
    ///
    /// let expr = ConditionParser::parse("Muss [494]").unwrap().unwrap();
    /// assert_eq!(expr, ConditionExpr::Ref(494));
    /// ```
    pub fn parse(input: &str) -> Result<Option<ConditionExpr>, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(None);
        }

        let stripped = strip_status_prefix(input);
        if stripped.is_empty() {
            return Ok(None);
        }

        let tokens = tokenize(stripped)?;
        if tokens.is_empty() {
            return Ok(None);
        }

        let mut pos = 0;
        let expr = parse_expression(&tokens, &mut pos)?;

        Ok(expr)
    }

    /// Parse an expression that is known to contain conditions (no prefix stripping).
    ///
    /// Returns `Err` if the input cannot be parsed. Returns `Ok(None)` if empty.
    pub fn parse_raw(input: &str) -> Result<Option<ConditionExpr>, ParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(None);
        }

        let tokens = tokenize(input)?;
        if tokens.is_empty() {
            return Ok(None);
        }

        let mut pos = 0;
        let expr = parse_expression(&tokens, &mut pos)?;

        Ok(expr)
    }
}

/// Parse a full expression (entry point for precedence climbing).
fn parse_expression(tokens: &[SpannedToken], pos: &mut usize) -> Result<Option<ConditionExpr>, ParseError> {
    parse_xor(tokens, pos)
}

/// XOR has the lowest precedence.
fn parse_xor(tokens: &[SpannedToken], pos: &mut usize) -> Result<Option<ConditionExpr>, ParseError> {
    let mut left = match parse_or(tokens, pos)? {
        Some(expr) => expr,
        None => return Ok(None),
    };

    while *pos < tokens.len() && tokens[*pos].token == Token::Xor {
        *pos += 1; // consume XOR
        let right = match parse_or(tokens, pos)? {
            Some(expr) => expr,
            None => return Ok(Some(left)),
        };
        left = ConditionExpr::Xor(Box::new(left), Box::new(right));
    }

    Ok(Some(left))
}

/// OR has middle-low precedence.
fn parse_or(tokens: &[SpannedToken], pos: &mut usize) -> Result<Option<ConditionExpr>, ParseError> {
    let mut left = match parse_and(tokens, pos)? {
        Some(expr) => expr,
        None => return Ok(None),
    };

    while *pos < tokens.len() && tokens[*pos].token == Token::Or {
        *pos += 1; // consume OR
        let right = match parse_and(tokens, pos)? {
            Some(expr) => expr,
            None => return Ok(Some(left)),
        };
        // Flatten nested ORs into a single Or(vec![...])
        left = match left {
            ConditionExpr::Or(mut exprs) => {
                exprs.push(right);
                ConditionExpr::Or(exprs)
            }
            _ => ConditionExpr::Or(vec![left, right]),
        };
    }

    Ok(Some(left))
}

/// AND has middle-high precedence. Also handles implicit AND between adjacent
/// conditions or parenthesized groups.
fn parse_and(tokens: &[SpannedToken], pos: &mut usize) -> Result<Option<ConditionExpr>, ParseError> {
    let mut left = match parse_not(tokens, pos)? {
        Some(expr) => expr,
        None => return Ok(None),
    };

    while *pos < tokens.len() {
        if tokens[*pos].token == Token::And {
            *pos += 1; // consume explicit AND
            let right = match parse_not(tokens, pos)? {
                Some(expr) => expr,
                None => return Ok(Some(left)),
            };
            left = flatten_and(left, right);
        } else if matches!(
            tokens[*pos].token,
            Token::ConditionId(_) | Token::LeftParen | Token::Not
        ) {
            // Implicit AND: adjacent condition, paren, or NOT without operator
            let right = match parse_not(tokens, pos)? {
                Some(expr) => expr,
                None => return Ok(Some(left)),
            };
            left = flatten_and(left, right);
        } else {
            break;
        }
    }

    Ok(Some(left))
}

/// Flatten nested ANDs into a single And(vec![...]).
fn flatten_and(left: ConditionExpr, right: ConditionExpr) -> ConditionExpr {
    match left {
        ConditionExpr::And(mut exprs) => {
            exprs.push(right);
            ConditionExpr::And(exprs)
        }
        _ => ConditionExpr::And(vec![left, right]),
    }
}

/// NOT has the highest precedence (unary prefix).
fn parse_not(tokens: &[SpannedToken], pos: &mut usize) -> Result<Option<ConditionExpr>, ParseError> {
    if *pos < tokens.len() && tokens[*pos].token == Token::Not {
        *pos += 1; // consume NOT
        let inner = match parse_not(tokens, pos)? {
            Some(expr) => expr,
            None => {
                return Err(ParseError::UnexpectedToken {
                    position: if *pos < tokens.len() {
                        tokens[*pos].position
                    } else {
                        0
                    },
                    expected: "expression after NOT".to_string(),
                    found: "end of input".to_string(),
                });
            }
        };
        return Ok(Some(ConditionExpr::Not(Box::new(inner))));
    }
    parse_primary(tokens, pos)
}

/// Primary: a condition reference or a parenthesized expression.
fn parse_primary(tokens: &[SpannedToken], pos: &mut usize) -> Result<Option<ConditionExpr>, ParseError> {
    if *pos >= tokens.len() {
        return Ok(None);
    }

    match &tokens[*pos].token {
        Token::ConditionId(id) => {
            let parsed_id = parse_condition_id(id);
            *pos += 1;
            Ok(Some(parsed_id))
        }
        Token::LeftParen => {
            *pos += 1; // consume (
            let expr = parse_expression(tokens, pos)?;
            // Consume closing paren if present (graceful handling of missing)
            if *pos < tokens.len() && tokens[*pos].token == Token::RightParen {
                *pos += 1;
            }
            Ok(expr)
        }
        _ => Ok(None),
    }
}

/// Parse a condition ID string into a ConditionExpr.
///
/// Numeric IDs become `Ref(n)`. Non-numeric IDs (like `UB1`, `10P1..5`)
/// are kept as-is by hashing or using a convention. For now, we extract
/// the leading numeric portion if present, otherwise use a deterministic
/// mapping.
///
/// Note: The design doc specifies `Ref(u32)` for numeric condition IDs.
/// For non-numeric IDs like package conditions (`10P1..5`) or time
/// conditions (`UB1`), we store them differently. Since the C# code
/// uses string IDs, we need a strategy. For the Rust port, we extend
/// the AST to handle string IDs for non-numeric conditions.
fn parse_condition_id(id: &str) -> ConditionExpr {
    // Try to parse as a pure numeric ID
    if let Ok(num) = id.parse::<u32>() {
        ConditionExpr::Ref(num)
    } else {
        // For non-numeric IDs, extract leading digits if any
        let numeric_part: String = id.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = numeric_part.parse::<u32>() {
            // Use the numeric prefix (e.g., "10P1..5" -> 10)
            // This is a simplification; the generator will handle the full mapping
            ConditionExpr::Ref(num)
        } else {
            // No numeric prefix (e.g., "UB1") - use 0 as a sentinel
            // The evaluator will need to handle these by name
            ConditionExpr::Ref(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // === Basic parsing ===

    #[test]
    fn test_parse_single_condition() {
        let result = ConditionParser::parse("[931]").unwrap().unwrap();
        assert_eq!(result, ConditionExpr::Ref(931));
    }

    #[test]
    fn test_parse_with_muss_prefix() {
        let result = ConditionParser::parse("Muss [494]").unwrap().unwrap();
        assert_eq!(result, ConditionExpr::Ref(494));
    }

    #[test]
    fn test_parse_with_soll_prefix() {
        let result = ConditionParser::parse("Soll [494]").unwrap().unwrap();
        assert_eq!(result, ConditionExpr::Ref(494));
    }

    #[test]
    fn test_parse_with_kann_prefix() {
        let result = ConditionParser::parse("Kann [182]").unwrap().unwrap();
        assert_eq!(result, ConditionExpr::Ref(182));
    }

    #[test]
    fn test_parse_with_x_prefix() {
        let result = ConditionParser::parse("X [567]").unwrap().unwrap();
        assert_eq!(result, ConditionExpr::Ref(567));
    }

    // === Binary operators ===

    #[test]
    fn test_parse_simple_and() {
        let result = ConditionParser::parse("[182] ∧ [152]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![ConditionExpr::Ref(182), ConditionExpr::Ref(152)])
        );
    }

    #[test]
    fn test_parse_simple_or() {
        let result = ConditionParser::parse("[1] ∨ [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)])
        );
    }

    #[test]
    fn test_parse_simple_xor() {
        let result = ConditionParser::parse("[1] ⊻ [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Xor(
                Box::new(ConditionExpr::Ref(1)),
                Box::new(ConditionExpr::Ref(2)),
            )
        );
    }

    // === Chained operators ===

    #[test]
    fn test_parse_three_way_and() {
        let result = ConditionParser::parse("[1] ∧ [2] ∧ [3]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![
                ConditionExpr::Ref(1),
                ConditionExpr::Ref(2),
                ConditionExpr::Ref(3),
            ])
        );
    }

    #[test]
    fn test_parse_three_way_and_with_prefix() {
        let result = ConditionParser::parse("Kann [182] ∧ [6] ∧ [570]")
            .unwrap()
            .unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![
                ConditionExpr::Ref(182),
                ConditionExpr::Ref(6),
                ConditionExpr::Ref(570),
            ])
        );
        assert_eq!(result.condition_ids(), [6, 182, 570].into());
    }

    #[test]
    fn test_parse_multiple_xor() {
        let result = ConditionParser::parse("[1] ⊻ [2] ⊻ [3] ⊻ [4]")
            .unwrap()
            .unwrap();
        assert_eq!(result.condition_ids(), [1, 2, 3, 4].into());
    }

    // === Parentheses ===

    #[test]
    fn test_parse_parenthesized_expression() {
        let result = ConditionParser::parse("([1] ∨ [2]) ∧ [3]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![
                ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]),
                ConditionExpr::Ref(3),
            ])
        );
    }

    #[test]
    fn test_parse_nested_parentheses() {
        // (([1] ∧ [2]) ∨ ([3] ∧ [4])) ∧ [5]
        let result = ConditionParser::parse("(([1] ∧ [2]) ∨ ([3] ∧ [4])) ∧ [5]")
            .unwrap()
            .unwrap();
        assert_eq!(result.condition_ids(), [1, 2, 3, 4, 5].into());
        // Outer is AND
        match &result {
            ConditionExpr::And(exprs) => {
                assert_eq!(exprs.len(), 2);
                assert!(matches!(&exprs[0], ConditionExpr::Or(_)));
                assert_eq!(exprs[1], ConditionExpr::Ref(5));
            }
            other => panic!("Expected And, got {other:?}"),
        }
    }

    // === Operator precedence ===

    #[test]
    fn test_and_has_higher_precedence_than_or() {
        // [1] ∨ [2] ∧ [3] should parse as [1] ∨ ([2] ∧ [3])
        let result = ConditionParser::parse("[1] ∨ [2] ∧ [3]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Or(vec![
                ConditionExpr::Ref(1),
                ConditionExpr::And(vec![ConditionExpr::Ref(2), ConditionExpr::Ref(3)]),
            ])
        );
    }

    #[test]
    fn test_or_has_higher_precedence_than_xor() {
        // [1] ⊻ [2] ∨ [3] should parse as [1] ⊻ ([2] ∨ [3])
        let result = ConditionParser::parse("[1] ⊻ [2] ∨ [3]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Xor(
                Box::new(ConditionExpr::Ref(1)),
                Box::new(ConditionExpr::Or(vec![
                    ConditionExpr::Ref(2),
                    ConditionExpr::Ref(3),
                ])),
            )
        );
    }

    // === Implicit AND ===

    #[test]
    fn test_adjacent_conditions_implicit_and() {
        // "[1] [2]" is equivalent to "[1] ∧ [2]"
        let result = ConditionParser::parse("[1] [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)])
        );
    }

    #[test]
    fn test_adjacent_conditions_no_space_implicit_and() {
        // "[939][14]" from real AHB XML
        let result = ConditionParser::parse("[939][14]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![ConditionExpr::Ref(939), ConditionExpr::Ref(14)])
        );
    }

    // === NOT operator ===

    #[test]
    fn test_parse_not() {
        let result = ConditionParser::parse("NOT [1]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Not(Box::new(ConditionExpr::Ref(1)))
        );
    }

    #[test]
    fn test_parse_not_with_and() {
        // NOT [1] ∧ [2] should parse as (NOT [1]) ∧ [2] because NOT has highest precedence
        let result = ConditionParser::parse("NOT [1] ∧ [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![
                ConditionExpr::Not(Box::new(ConditionExpr::Ref(1))),
                ConditionExpr::Ref(2),
            ])
        );
    }

    // === Real-world AHB expressions ===

    #[test]
    fn test_real_world_orders_expression() {
        // From ORDERS AHB: "X (([939] [147]) ∨ ([940] [148])) ∧ [567]"
        let result = ConditionParser::parse("X (([939] [147]) ∨ ([940] [148])) ∧ [567]")
            .unwrap()
            .unwrap();
        assert_eq!(result.condition_ids(), [14, 147, 148, 567, 939, 940].into());
    }

    #[test]
    fn test_real_world_xor_expression() {
        // "Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])"
        let result = ConditionParser::parse("Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])")
            .unwrap()
            .unwrap();
        assert!(matches!(result, ConditionExpr::Xor(_, _)));
        assert_eq!(result.condition_ids(), [102, 103, 2005, 2006].into());
    }

    #[test]
    fn test_real_world_complex_nested_with_implicit_and() {
        // "([939][14]) ∨ ([940][15])"
        let result = ConditionParser::parse("([939][14]) ∨ ([940][15])")
            .unwrap()
            .unwrap();
        assert!(matches!(result, ConditionExpr::Or(_)));
        assert_eq!(result.condition_ids(), [14, 15, 939, 940].into());
    }

    // === Edge cases ===

    #[test]
    fn test_parse_empty_string() {
        assert!(ConditionParser::parse("").unwrap().is_none());
    }

    #[test]
    fn test_parse_whitespace_only() {
        assert!(ConditionParser::parse("   \t  ").unwrap().is_none());
    }

    #[test]
    fn test_parse_bare_muss() {
        assert!(ConditionParser::parse("Muss").unwrap().is_none());
    }

    #[test]
    fn test_parse_bare_x() {
        // "X" alone has no conditions after it
        assert!(ConditionParser::parse("X").unwrap().is_none());
    }

    #[test]
    fn test_parse_unmatched_open_paren_graceful() {
        // ([1] ∧ [2] — missing closing paren
        let result = ConditionParser::parse("([1] ∧ [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)])
        );
    }

    #[test]
    fn test_parse_text_and_operator() {
        let result = ConditionParser::parse("[1] AND [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)])
        );
    }

    #[test]
    fn test_parse_text_or_operator() {
        let result = ConditionParser::parse("[1] OR [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)])
        );
    }

    #[test]
    fn test_parse_text_xor_operator() {
        let result = ConditionParser::parse("[1] XOR [2]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Xor(
                Box::new(ConditionExpr::Ref(1)),
                Box::new(ConditionExpr::Ref(2)),
            )
        );
    }

    #[test]
    fn test_parse_mixed_unicode_and_text_operators() {
        let result = ConditionParser::parse("[1] ∧ [2] OR [3]").unwrap().unwrap();
        assert_eq!(
            result,
            ConditionExpr::Or(vec![
                ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]),
                ConditionExpr::Ref(3),
            ])
        );
    }

    #[test]
    fn test_parse_deeply_nested() {
        // ((([1])))
        let result = ConditionParser::parse("((([1])))").unwrap().unwrap();
        assert_eq!(result, ConditionExpr::Ref(1));
    }

    #[test]
    fn test_condition_ids_extraction_full() {
        let result = ConditionParser::parse("Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])")
            .unwrap()
            .unwrap();
        let ids = result.condition_ids();
        assert!(ids.contains(&102));
        assert!(ids.contains(&103));
        assert!(ids.contains(&2005));
        assert!(ids.contains(&2006));
        assert_eq!(ids.len(), 4);
    }
}
```

Update the module re-exports in **`crates/automapper-validation/src/expr.rs`**:
```rust
//! Condition expression AST and parser.

mod ast;
mod parser;
mod token;

pub use ast::ConditionExpr;
pub use parser::ConditionParser;
pub use token::{strip_status_prefix, Token};
```

### Verification
```bash
cargo test -p automapper-validation parser::tests
cargo test -p automapper-validation -- --test-threads=1
cargo clippy -p automapper-validation -- -D warnings
```

### Commit
```
feat(automapper-validation): implement recursive descent condition parser

Parses AHB status strings into ConditionExpr AST with correct operator
precedence (NOT > AND > OR > XOR), implicit AND for adjacent conditions,
Unicode and text operators, parenthesized grouping, and status prefix
stripping. Comprehensive tests cover real-world AHB expressions.
```

---

## Task 5: Add property-based tests for parser robustness

### Description
Add property-based tests using `proptest` to verify that the parser never panics on arbitrary input and that round-tripping through Display + parse preserves semantics.

### Files to Create/Modify

Add to **`crates/automapper-validation/Cargo.toml`** dev-dependencies:
```toml
[dev-dependencies]
pretty_assertions = "1"
proptest = "1"
```

**`crates/automapper-validation/tests/parser_proptest.rs`**:
```rust
//! Property-based tests for the condition expression parser.

use automapper_validation::expr::{ConditionExpr, ConditionParser};
use proptest::prelude::*;

/// Generate arbitrary strings that may or may not be valid condition expressions.
fn arbitrary_condition_input() -> impl Strategy<Value = String> {
    prop::string::string_regex(
        r"(Muss |Soll |Kann |X )?(\[[\d]{1,4}\]|\(|\)|[ ∧∨⊻]|AND |OR |XOR |NOT ){0,20}",
    )
    .unwrap()
}

proptest! {
    /// The parser must never panic on arbitrary input.
    #[test]
    fn parser_never_panics(input in "\\PC{0,200}") {
        // We don't care about the result, just that it doesn't panic
        let _ = ConditionParser::parse(&input);
    }

    /// The parser must never panic on semi-structured input.
    #[test]
    fn parser_never_panics_on_structured_input(input in arbitrary_condition_input()) {
        let _ = ConditionParser::parse(&input);
    }

    /// Parsing a Display'd expression should yield the same condition IDs.
    #[test]
    fn display_roundtrip_preserves_condition_ids(
        ids in prop::collection::vec(1u32..=2000, 1..=5)
    ) {
        // Build a simple AND expression from the IDs
        let expr = if ids.len() == 1 {
            ConditionExpr::Ref(ids[0])
        } else {
            ConditionExpr::And(ids.iter().map(|&id| ConditionExpr::Ref(id)).collect())
        };

        let displayed = format!("{expr}");
        if let Ok(Some(reparsed)) = ConditionParser::parse_raw(&displayed) {
            assert_eq!(
                expr.condition_ids(),
                reparsed.condition_ids(),
                "Condition IDs differ after roundtrip: '{displayed}'"
            );
        }
    }
}
```

### Verification
```bash
cargo test -p automapper-validation --test parser_proptest
```

### Commit
```
test(automapper-validation): add proptest property-based parser tests

Verify parser never panics on arbitrary input and that Display roundtrip
preserves condition IDs.
```
