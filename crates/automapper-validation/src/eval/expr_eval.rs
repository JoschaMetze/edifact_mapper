//! Evaluates `ConditionExpr` trees using a `ConditionEvaluator`.

use crate::expr::ConditionExpr;
use super::context::EvaluationContext;
use super::evaluator::{ConditionEvaluator, ConditionResult};

/// Evaluates a `ConditionExpr` AST against an evaluation context.
///
/// Uses three-valued short-circuit logic:
/// - AND: False short-circuits to False; all True -> True; else Unknown
/// - OR: True short-circuits to True; all False -> False; else Unknown
/// - XOR: requires both operands known; Unknown if either is Unknown
/// - NOT: inverts True/False; preserves Unknown
pub struct ConditionExprEvaluator<'a, E: ConditionEvaluator> {
    evaluator: &'a E,
}

impl<'a, E: ConditionEvaluator> ConditionExprEvaluator<'a, E> {
    /// Create a new expression evaluator wrapping a condition evaluator.
    pub fn new(evaluator: &'a E) -> Self {
        Self { evaluator }
    }

    /// Evaluate a condition expression tree.
    pub fn evaluate(&self, expr: &ConditionExpr, ctx: &EvaluationContext) -> ConditionResult {
        match expr {
            ConditionExpr::Ref(id) => self.evaluator.evaluate(*id, ctx),

            ConditionExpr::And(exprs) => self.evaluate_and(exprs, ctx),

            ConditionExpr::Or(exprs) => self.evaluate_or(exprs, ctx),

            ConditionExpr::Xor(left, right) => {
                let l = self.evaluate(left, ctx);
                let r = self.evaluate(right, ctx);
                self.evaluate_xor(l, r)
            }

            ConditionExpr::Not(inner) => {
                let result = self.evaluate(inner, ctx);
                self.evaluate_not(result)
            }
        }
    }

    /// AND with short-circuit: any False -> False, all True -> True, else Unknown.
    fn evaluate_and(&self, exprs: &[ConditionExpr], ctx: &EvaluationContext) -> ConditionResult {
        let mut has_unknown = false;

        for expr in exprs {
            match self.evaluate(expr, ctx) {
                ConditionResult::False => return ConditionResult::False,
                ConditionResult::Unknown => has_unknown = true,
                ConditionResult::True => {}
            }
        }

        if has_unknown {
            ConditionResult::Unknown
        } else {
            ConditionResult::True
        }
    }

    /// OR with short-circuit: any True -> True, all False -> False, else Unknown.
    fn evaluate_or(&self, exprs: &[ConditionExpr], ctx: &EvaluationContext) -> ConditionResult {
        let mut has_unknown = false;

        for expr in exprs {
            match self.evaluate(expr, ctx) {
                ConditionResult::True => return ConditionResult::True,
                ConditionResult::Unknown => has_unknown = true,
                ConditionResult::False => {}
            }
        }

        if has_unknown {
            ConditionResult::Unknown
        } else {
            ConditionResult::False
        }
    }

    /// XOR: both must be known. True XOR False = True, same values = False, Unknown if either Unknown.
    fn evaluate_xor(&self, left: ConditionResult, right: ConditionResult) -> ConditionResult {
        match (left, right) {
            (ConditionResult::True, ConditionResult::False)
            | (ConditionResult::False, ConditionResult::True) => ConditionResult::True,
            (ConditionResult::True, ConditionResult::True)
            | (ConditionResult::False, ConditionResult::False) => ConditionResult::False,
            _ => ConditionResult::Unknown,
        }
    }

    /// NOT: inverts True/False, preserves Unknown.
    fn evaluate_not(&self, result: ConditionResult) -> ConditionResult {
        match result {
            ConditionResult::True => ConditionResult::False,
            ConditionResult::False => ConditionResult::True,
            ConditionResult::Unknown => ConditionResult::Unknown,
        }
    }

    /// Parse an AHB status string, evaluate it, and return the result.
    ///
    /// Returns `ConditionResult::True` if there are no conditions (unconditionally required).
    pub fn evaluate_status(&self, ahb_status: &str, ctx: &EvaluationContext) -> ConditionResult {
        use crate::expr::ConditionParser;

        match ConditionParser::parse(ahb_status) {
            Ok(Some(expr)) => self.evaluate(&expr, ctx),
            Ok(None) => ConditionResult::True, // No conditions = unconditionally true
            Err(_) => ConditionResult::Unknown, // Parse error = treat as unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::evaluator::{ConditionResult as CR, NoOpExternalProvider};
    use super::*;
    use edifact_types::RawSegment;
    use std::collections::HashMap;

    /// A mock condition evaluator for testing.
    struct MockEvaluator {
        results: HashMap<u32, ConditionResult>,
        external_ids: Vec<u32>,
    }

    impl MockEvaluator {
        fn new() -> Self {
            Self {
                results: HashMap::new(),
                external_ids: Vec::new(),
            }
        }

        fn with_condition(mut self, id: u32, result: ConditionResult) -> Self {
            self.results.insert(id, result);
            self
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
            "TEST"
        }

        fn format_version(&self) -> &str {
            "FV_TEST"
        }
    }

    fn empty_context() -> (NoOpExternalProvider, Vec<RawSegment<'static>>) {
        (NoOpExternalProvider, Vec::new())
    }

    fn make_ctx<'a>(
        external: &'a NoOpExternalProvider,
        segments: &'a [RawSegment<'a>],
    ) -> EvaluationContext<'a> {
        EvaluationContext::new("11001", external, segments)
    }

    // === Single condition ===

    #[test]
    fn test_eval_single_true() {
        let eval = MockEvaluator::new().with_condition(1, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        assert_eq!(expr_eval.evaluate(&ConditionExpr::Ref(1), &ctx), CR::True);
    }

    #[test]
    fn test_eval_single_false() {
        let eval = MockEvaluator::new().with_condition(1, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        assert_eq!(expr_eval.evaluate(&ConditionExpr::Ref(1), &ctx), CR::False);
    }

    #[test]
    fn test_eval_single_unknown() {
        let eval = MockEvaluator::new(); // No condition registered -> Unknown
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        assert_eq!(
            expr_eval.evaluate(&ConditionExpr::Ref(999), &ctx),
            CR::Unknown
        );
    }

    // === AND ===

    #[test]
    fn test_eval_and_both_true() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::True)
            .with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_and_one_false_short_circuits() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::False)
            .with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    #[test]
    fn test_eval_and_one_unknown_true_gives_unknown() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::True)
            .with_condition(2, CR::Unknown);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::Unknown);
    }

    #[test]
    fn test_eval_and_false_beats_unknown() {
        // AND with False and Unknown should be False (short-circuit)
        let eval = MockEvaluator::new()
            .with_condition(1, CR::False)
            .with_condition(2, CR::Unknown);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    #[test]
    fn test_eval_and_three_way() {
        let eval = MockEvaluator::new()
            .with_condition(182, CR::True)
            .with_condition(6, CR::True)
            .with_condition(570, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![
            ConditionExpr::Ref(182),
            ConditionExpr::Ref(6),
            ConditionExpr::Ref(570),
        ]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_and_three_way_one_false() {
        let eval = MockEvaluator::new()
            .with_condition(182, CR::True)
            .with_condition(6, CR::True)
            .with_condition(570, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![
            ConditionExpr::Ref(182),
            ConditionExpr::Ref(6),
            ConditionExpr::Ref(570),
        ]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    // === OR ===

    #[test]
    fn test_eval_or_both_false() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::False)
            .with_condition(2, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    #[test]
    fn test_eval_or_one_true_short_circuits() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::False)
            .with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_or_true_beats_unknown() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::Unknown)
            .with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_or_false_and_unknown_gives_unknown() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::False)
            .with_condition(2, CR::Unknown);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::Unknown);
    }

    // === XOR ===

    #[test]
    fn test_eval_xor_true_false() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::True)
            .with_condition(2, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::Ref(1)),
            Box::new(ConditionExpr::Ref(2)),
        );
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_xor_both_true() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::True)
            .with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::Ref(1)),
            Box::new(ConditionExpr::Ref(2)),
        );
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    #[test]
    fn test_eval_xor_both_false() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::False)
            .with_condition(2, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::Ref(1)),
            Box::new(ConditionExpr::Ref(2)),
        );
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    #[test]
    fn test_eval_xor_unknown_propagates() {
        let eval = MockEvaluator::new()
            .with_condition(1, CR::True)
            .with_condition(2, CR::Unknown);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Xor(
            Box::new(ConditionExpr::Ref(1)),
            Box::new(ConditionExpr::Ref(2)),
        );
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::Unknown);
    }

    // === NOT ===

    #[test]
    fn test_eval_not_true() {
        let eval = MockEvaluator::new().with_condition(1, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Not(Box::new(ConditionExpr::Ref(1)));
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }

    #[test]
    fn test_eval_not_false() {
        let eval = MockEvaluator::new().with_condition(1, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Not(Box::new(ConditionExpr::Ref(1)));
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_not_unknown() {
        let eval = MockEvaluator::new(); // 1 -> Unknown
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Not(Box::new(ConditionExpr::Ref(1)));
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::Unknown);
    }

    // === Complex expressions ===

    #[test]
    fn test_eval_complex_nested() {
        // (([1] ∧ [2]) ∨ ([3] ∧ [4])) ∧ [5]
        // [1]=T, [2]=F, [3]=T, [4]=T, [5]=T
        // ([1]∧[2])=F, ([3]∧[4])=T, F∨T=T, T∧[5]=T
        let eval = MockEvaluator::new()
            .with_condition(1, CR::True)
            .with_condition(2, CR::False)
            .with_condition(3, CR::True)
            .with_condition(4, CR::True)
            .with_condition(5, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![
            ConditionExpr::Or(vec![
                ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]),
                ConditionExpr::And(vec![ConditionExpr::Ref(3), ConditionExpr::Ref(4)]),
            ]),
            ConditionExpr::Ref(5),
        ]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_eval_xor_with_nested_and() {
        // ([102] ∧ [2006]) ⊻ ([103] ∧ [2005])
        // [102]=T, [2006]=T, [103]=F, [2005]=F
        // T∧T=T, F∧F=F, T⊻F=T
        let eval = MockEvaluator::new()
            .with_condition(102, CR::True)
            .with_condition(2006, CR::True)
            .with_condition(103, CR::False)
            .with_condition(2005, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

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
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    // === evaluate_status ===

    #[test]
    fn test_evaluate_status_with_conditions() {
        let eval = MockEvaluator::new()
            .with_condition(182, CR::True)
            .with_condition(152, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        assert_eq!(
            expr_eval.evaluate_status("Muss [182] ∧ [152]", &ctx),
            CR::True
        );
    }

    #[test]
    fn test_evaluate_status_no_conditions() {
        let eval = MockEvaluator::new();
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        assert_eq!(expr_eval.evaluate_status("Muss", &ctx), CR::True);
    }

    #[test]
    fn test_evaluate_status_empty() {
        let eval = MockEvaluator::new();
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        assert_eq!(expr_eval.evaluate_status("", &ctx), CR::True);
    }

    // === Unknown propagation comprehensive ===

    #[test]
    fn test_unknown_propagation_and_or_mix() {
        // [1](Unknown) ∨ ([2](True) ∧ [3](Unknown))
        // [2]∧[3] = Unknown (True ∧ Unknown)
        // Unknown ∨ Unknown = Unknown
        let eval = MockEvaluator::new().with_condition(2, CR::True);
        // 1 and 3 default to Unknown
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![
            ConditionExpr::Ref(1),
            ConditionExpr::And(vec![ConditionExpr::Ref(2), ConditionExpr::Ref(3)]),
        ]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::Unknown);
    }

    #[test]
    fn test_or_short_circuits_past_unknown() {
        // [1](Unknown) ∨ [2](True) -> True (True short-circuits)
        let eval = MockEvaluator::new().with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_and_short_circuits_past_unknown() {
        // [1](Unknown) ∧ [2](False) -> False (False short-circuits)
        let eval = MockEvaluator::new().with_condition(2, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }
}
