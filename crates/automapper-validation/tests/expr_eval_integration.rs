//! Integration tests: parse AHB expressions and evaluate them.

use automapper_validation::eval::{
    ConditionEvaluator, ConditionExprEvaluator, ConditionResult, EvaluationContext,
    ExternalConditionProvider,
};
use automapper_validation::expr::ConditionParser;
use edifact_types::RawSegment;
use std::collections::HashMap;

/// Mock evaluator with configurable condition results.
struct MockEvaluator {
    results: HashMap<u32, ConditionResult>,
    external_ids: Vec<u32>,
}

impl MockEvaluator {
    fn new(results: Vec<(u32, ConditionResult)>) -> Self {
        Self {
            results: results.into_iter().collect(),
            external_ids: Vec::new(),
        }
    }
}

impl ConditionEvaluator for MockEvaluator {
    fn evaluate(&self, condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
        self.results
            .get(&condition)
            .copied()
            .unwrap_or(ConditionResult::Unknown)
    }
    fn is_external(&self, condition: u32) -> bool {
        self.external_ids.contains(&condition)
    }
    fn message_type(&self) -> &str {
        "UTILMD"
    }
    fn format_version(&self) -> &str {
        "FV2510"
    }
}

struct NoExternal;
impl ExternalConditionProvider for NoExternal {
    fn evaluate(&self, _: &str) -> ConditionResult {
        ConditionResult::Unknown
    }
}

fn eval_status(ahb_status: &str, conditions: Vec<(u32, ConditionResult)>) -> ConditionResult {
    let mock = MockEvaluator::new(conditions);
    let ext = NoExternal;
    let segments: Vec<RawSegment> = Vec::new();
    let ctx = EvaluationContext::new("11001", &ext, &segments);
    let expr_eval = ConditionExprEvaluator::new(&mock);
    expr_eval.evaluate_status(ahb_status, &ctx)
}

// === Real-world ORDERS AHB expression ===

#[test]
fn test_orders_complex_expression_true() {
    // "X (([939] [147]) ∨ ([940] [148])) ∧ [567]"
    // [939]=T, [147]=T, [940]=F, [148]=F, [567]=T -> T
    let result = eval_status(
        "X (([939] [147]) ∨ ([940] [148])) ∧ [567]",
        vec![
            (939, ConditionResult::True),
            (147, ConditionResult::True),
            (940, ConditionResult::False),
            (148, ConditionResult::False),
            (567, ConditionResult::True),
        ],
    );
    assert_eq!(result, ConditionResult::True);
}

#[test]
fn test_orders_complex_expression_false() {
    // Same expression but [567]=F
    let result = eval_status(
        "X (([939] [147]) ∨ ([940] [148])) ∧ [567]",
        vec![
            (939, ConditionResult::True),
            (147, ConditionResult::True),
            (940, ConditionResult::False),
            (148, ConditionResult::False),
            (567, ConditionResult::False),
        ],
    );
    assert_eq!(result, ConditionResult::False);
}

// === Real-world UTILMD XOR expression ===

#[test]
fn test_utilmd_xor_expression_first_branch_true() {
    // "Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])"
    // [102]=T, [2006]=T, [103]=F, [2005]=F -> T⊻F = T
    let result = eval_status(
        "Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])",
        vec![
            (102, ConditionResult::True),
            (2006, ConditionResult::True),
            (103, ConditionResult::False),
            (2005, ConditionResult::False),
        ],
    );
    assert_eq!(result, ConditionResult::True);
}

#[test]
fn test_utilmd_xor_expression_both_true() {
    // Both branches true -> F (XOR)
    let result = eval_status(
        "Muss ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])",
        vec![
            (102, ConditionResult::True),
            (2006, ConditionResult::True),
            (103, ConditionResult::True),
            (2005, ConditionResult::True),
        ],
    );
    assert_eq!(result, ConditionResult::False);
}

// === Three-way AND chain ===

#[test]
fn test_three_way_and_all_true() {
    let result = eval_status(
        "Kann [182] ∧ [6] ∧ [570]",
        vec![
            (182, ConditionResult::True),
            (6, ConditionResult::True),
            (570, ConditionResult::True),
        ],
    );
    assert_eq!(result, ConditionResult::True);
}

#[test]
fn test_three_way_and_one_false() {
    let result = eval_status(
        "Kann [182] ∧ [6] ∧ [570]",
        vec![
            (182, ConditionResult::True),
            (6, ConditionResult::True),
            (570, ConditionResult::False),
        ],
    );
    assert_eq!(result, ConditionResult::False);
}

// === Unknown propagation in real expressions ===

#[test]
fn test_unknown_external_in_and_chain() {
    // [182]=T, [6]=Unknown (external), [570]=T -> Unknown
    let result = eval_status(
        "Kann [182] ∧ [6] ∧ [570]",
        vec![
            (182, ConditionResult::True),
            // 6 not registered -> Unknown
            (570, ConditionResult::True),
        ],
    );
    assert_eq!(result, ConditionResult::Unknown);
}

#[test]
fn test_unknown_external_in_and_chain_with_false() {
    // [182]=T, [6]=Unknown, [570]=F -> False (short-circuit beats Unknown)
    let result = eval_status(
        "Kann [182] ∧ [6] ∧ [570]",
        vec![
            (182, ConditionResult::True),
            // 6 not registered -> Unknown
            (570, ConditionResult::False),
        ],
    );
    assert_eq!(result, ConditionResult::False);
}

// === Bare status (no conditions) ===

#[test]
fn test_bare_muss_is_unconditionally_true() {
    let result = eval_status("Muss", vec![]);
    assert_eq!(result, ConditionResult::True);
}

#[test]
fn test_bare_x_is_unconditionally_true() {
    let result = eval_status("X", vec![]);
    assert_eq!(result, ConditionResult::True);
}

#[test]
fn test_empty_is_unconditionally_true() {
    let result = eval_status("", vec![]);
    assert_eq!(result, ConditionResult::True);
}
