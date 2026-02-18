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
fn parse_expression(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<ConditionExpr>, ParseError> {
    parse_xor(tokens, pos)
}

/// XOR has the lowest precedence.
fn parse_xor(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<ConditionExpr>, ParseError> {
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
fn parse_or(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<ConditionExpr>, ParseError> {
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
fn parse_and(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<ConditionExpr>, ParseError> {
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
fn parse_not(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<ConditionExpr>, ParseError> {
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
fn parse_primary(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<Option<ConditionExpr>, ParseError> {
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
/// are kept as-is by extracting numeric portions. For the Rust port,
/// pure numeric IDs use `Ref(u32)`. Non-numeric IDs extract leading
/// digits if present, otherwise use 0 as a sentinel.
fn parse_condition_id(id: &str) -> ConditionExpr {
    // Try to parse as a pure numeric ID
    if let Ok(num) = id.parse::<u32>() {
        ConditionExpr::Ref(num)
    } else {
        // For non-numeric IDs, extract leading digits if any
        let numeric_part: String = id.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = numeric_part.parse::<u32>() {
            // Use the numeric prefix (e.g., "10P1..5" -> 10)
            ConditionExpr::Ref(num)
        } else {
            // No numeric prefix (e.g., "UB1") - use 0 as a sentinel
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
        let result = ConditionParser::parse("([1] ∨ [2]) ∧ [3]")
            .unwrap()
            .unwrap();
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
        let result = ConditionParser::parse("[1] ∨ [2] ∧ [3]")
            .unwrap()
            .unwrap();
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
        let result = ConditionParser::parse("[1] ⊻ [2] ∨ [3]")
            .unwrap()
            .unwrap();
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
        assert_eq!(result.condition_ids(), [147, 148, 567, 939, 940].into());
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
        let result = ConditionParser::parse("[1] ∧ [2] OR [3]")
            .unwrap()
            .unwrap();
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
