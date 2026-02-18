---
feature: validation-implementation
epic: 2
title: "Condition Evaluator Traits & Registry"
depends_on: [validation-implementation/E01]
estimated_tasks: 4
crate: automapper-validation
status: in_progress
---

# Epic 2: Condition Evaluator Traits & Registry

> **For Claude:** Implement each task in sequence. Every task follows TDD: write the test FIRST, run it to see it fail, write the implementation, run it to see it pass, then commit. All code lives in `crates/automapper-validation/src/`. All code must compile with `cargo check -p automapper-validation`.

**Goal:** Implement the condition evaluation layer that bridges the parsed `ConditionExpr` AST (from Epic 1) with concrete condition logic. The key abstraction is a `ConditionEvaluator` trait that maps condition numbers to three-valued `ConditionResult` (True/False/Unknown). An `ExternalConditionProvider` trait handles conditions that depend on context outside the EDIFACT message. A `ConditionExprEvaluator` walks the AST using short-circuit boolean logic with `Unknown` propagation. A registry maps `(message_type, format_version)` pairs to evaluator instances. This ports the C# types `IConditionEvaluator`, `IExternalConditionProvider`, `ConditionExpressionEvaluator`, and `ConditionEvaluatorRegistry`. The critical Rust difference: we use three-valued logic (`ConditionResult::Unknown`) instead of C#'s boolean, enabling partial evaluation when external conditions are unavailable.

**Architecture:** Trait-based evaluation with `ConditionEvaluator` for individual condition lookup, `ExternalConditionProvider` for business-context conditions, and `ConditionExprEvaluator` for AST walking with short-circuit three-valued logic (AND short-circuits on False, OR on True, XOR requires both known, NOT inverts). Thread-safe `EvaluatorRegistry` using `RwLock<HashMap>` for concurrent read access.

**Tech Stack:** thiserror 2.x, serde 1.x, edifact-types (path dep), edifact-parser (path dep)

---

## Task 1: Define ConditionResult and EvaluationContext

### Description
Define the three-valued `ConditionResult` enum and the `EvaluationContext` struct that carries transaction data and external provider references through the evaluation pipeline.

### Implementation

**`crates/automapper-validation/src/eval.rs`**:
```rust
//! Condition evaluation traits and expression evaluator.

mod context;
mod evaluator;
mod expr_eval;
mod registry;

pub use context::EvaluationContext;
pub use evaluator::{ConditionEvaluator, ConditionResult, ExternalConditionProvider};
pub use expr_eval::ConditionExprEvaluator;
pub use registry::EvaluatorRegistry;
```

**`crates/automapper-validation/src/eval/evaluator.rs`**:
```rust
//! Core condition evaluation traits.

use super::context::EvaluationContext;

/// Three-valued result of evaluating a single condition.
///
/// Unlike the C# implementation which uses `bool`, we use three-valued logic
/// to support partial evaluation when external conditions are unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionResult {
    /// The condition is satisfied.
    True,
    /// The condition is not satisfied.
    False,
    /// The condition cannot be determined (e.g., external condition without a provider).
    Unknown,
}

impl ConditionResult {
    /// Returns `true` if this is `ConditionResult::True`.
    pub fn is_true(self) -> bool {
        matches!(self, ConditionResult::True)
    }

    /// Returns `true` if this is `ConditionResult::False`.
    pub fn is_false(self) -> bool {
        matches!(self, ConditionResult::False)
    }

    /// Returns `true` if this is `ConditionResult::Unknown`.
    pub fn is_unknown(self) -> bool {
        matches!(self, ConditionResult::Unknown)
    }

    /// Converts to `Option<bool>`: True -> Some(true), False -> Some(false), Unknown -> None.
    pub fn to_option(self) -> Option<bool> {
        match self {
            ConditionResult::True => Some(true),
            ConditionResult::False => Some(false),
            ConditionResult::Unknown => None,
        }
    }
}

impl From<bool> for ConditionResult {
    fn from(value: bool) -> Self {
        if value {
            ConditionResult::True
        } else {
            ConditionResult::False
        }
    }
}

impl std::fmt::Display for ConditionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionResult::True => write!(f, "True"),
            ConditionResult::False => write!(f, "False"),
            ConditionResult::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Evaluates individual AHB conditions by number.
///
/// Implementations are typically generated from AHB XML schemas (one per
/// message type and format version). Each condition number maps to a
/// specific business rule check.
///
/// # Example
///
/// ```ignore
/// // Generated evaluator for UTILMD FV2510
/// struct UtilmdConditionEvaluatorFV2510;
///
/// impl ConditionEvaluator for UtilmdConditionEvaluatorFV2510 {
///     fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult {
///         match condition {
///             2 => {
///                 // [2] Wenn UNH DE0070 mit 1 vorhanden
///                 // Check transmission sequence number
///                 ConditionResult::from(/* ... */)
///             }
///             8 => ConditionResult::Unknown, // external
///             _ => ConditionResult::Unknown,
///         }
///     }
///
///     fn is_external(&self, condition: u32) -> bool {
///         matches!(condition, 1 | 3 | 8 | 14 | 30 | 31 | 34)
///     }
/// }
/// ```
pub trait ConditionEvaluator: Send + Sync {
    /// Evaluate a single condition by number.
    ///
    /// Returns `ConditionResult::Unknown` for unrecognized condition numbers
    /// or conditions that require unavailable external context.
    fn evaluate(&self, condition: u32, ctx: &EvaluationContext) -> ConditionResult;

    /// Returns `true` if the given condition requires external context
    /// (i.e., cannot be determined from the EDIFACT message alone).
    fn is_external(&self, condition: u32) -> bool;

    /// Returns the message type this evaluator handles (e.g., "UTILMD").
    fn message_type(&self) -> &str;

    /// Returns the format version this evaluator handles (e.g., "FV2510").
    fn format_version(&self) -> &str;
}

/// Provider for external conditions that depend on context outside the EDIFACT message.
///
/// External conditions are things like:
/// - [1] "Wenn Aufteilung vorhanden" (message splitting status)
/// - [14] "Wenn Datum bekannt" (whether a date is known)
/// - [30] "Wenn Antwort auf Aktivierung" (response to activation)
///
/// These cannot be determined from the EDIFACT content alone and require
/// business context from the calling system.
pub trait ExternalConditionProvider: Send + Sync {
    /// Evaluate an external condition by name.
    ///
    /// The `condition_name` corresponds to the speaking name from the
    /// generated external conditions constants (e.g., "MessageSplitting",
    /// "DateKnown").
    fn evaluate(&self, condition_name: &str) -> ConditionResult;
}

/// A no-op external condition provider that returns `Unknown` for everything.
///
/// Useful when no external context is available — conditions will propagate
/// as `Unknown` through the expression evaluator.
pub struct NoOpExternalProvider;

impl ExternalConditionProvider for NoOpExternalProvider {
    fn evaluate(&self, _condition_name: &str) -> ConditionResult {
        ConditionResult::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_result_is_methods() {
        assert!(ConditionResult::True.is_true());
        assert!(!ConditionResult::True.is_false());
        assert!(!ConditionResult::True.is_unknown());

        assert!(!ConditionResult::False.is_true());
        assert!(ConditionResult::False.is_false());

        assert!(ConditionResult::Unknown.is_unknown());
    }

    #[test]
    fn test_condition_result_to_option() {
        assert_eq!(ConditionResult::True.to_option(), Some(true));
        assert_eq!(ConditionResult::False.to_option(), Some(false));
        assert_eq!(ConditionResult::Unknown.to_option(), None);
    }

    #[test]
    fn test_condition_result_from_bool() {
        assert_eq!(ConditionResult::from(true), ConditionResult::True);
        assert_eq!(ConditionResult::from(false), ConditionResult::False);
    }

    #[test]
    fn test_condition_result_display() {
        assert_eq!(format!("{}", ConditionResult::True), "True");
        assert_eq!(format!("{}", ConditionResult::False), "False");
        assert_eq!(format!("{}", ConditionResult::Unknown), "Unknown");
    }

    #[test]
    fn test_noop_external_provider() {
        let provider = NoOpExternalProvider;
        assert_eq!(
            provider.evaluate("MessageSplitting"),
            ConditionResult::Unknown
        );
        assert_eq!(provider.evaluate("anything"), ConditionResult::Unknown);
    }
}
```

**`crates/automapper-validation/src/eval/context.rs`**:
```rust
//! Evaluation context for condition evaluation.

use super::evaluator::ExternalConditionProvider;

/// Context passed to condition evaluators during evaluation.
///
/// Carries references to the transaction data and external condition
/// provider needed to evaluate AHB conditions.
pub struct EvaluationContext<'a> {
    /// The Pruefidentifikator (e.g., "11001", "55001") that identifies
    /// the specific AHB workflow being validated against.
    pub pruefidentifikator: &'a str,

    /// Provider for external conditions that depend on business context
    /// outside the EDIFACT message.
    pub external: &'a dyn ExternalConditionProvider,

    /// Raw EDIFACT segments for direct segment inspection by condition
    /// evaluators. Conditions often need to check specific segment values.
    pub segments: &'a [edifact_types::RawSegment<'a>],
}

impl<'a> EvaluationContext<'a> {
    /// Create a new evaluation context.
    pub fn new(
        pruefidentifikator: &'a str,
        external: &'a dyn ExternalConditionProvider,
        segments: &'a [edifact_types::RawSegment<'a>],
    ) -> Self {
        Self {
            pruefidentifikator,
            external,
            segments,
        }
    }

    /// Find the first segment with the given ID.
    pub fn find_segment(&self, segment_id: &str) -> Option<&edifact_types::RawSegment<'a>> {
        self.segments.iter().find(|s| s.id == segment_id)
    }

    /// Find all segments with the given ID.
    pub fn find_segments(&self, segment_id: &str) -> Vec<&edifact_types::RawSegment<'a>> {
        self.segments.iter().filter(|s| s.id == segment_id).collect()
    }

    /// Find segments with a specific qualifier value on a given element.
    pub fn find_segments_with_qualifier(
        &self,
        segment_id: &str,
        element_index: usize,
        qualifier: &str,
    ) -> Vec<&edifact_types::RawSegment<'a>> {
        self.segments
            .iter()
            .filter(|s| {
                s.id == segment_id
                    && s.elements
                        .get(element_index)
                        .and_then(|e| e.first())
                        .map_or(false, |v| *v == qualifier)
            })
            .collect()
    }

    /// Check if a segment with the given ID exists.
    pub fn has_segment(&self, segment_id: &str) -> bool {
        self.segments.iter().any(|s| s.id == segment_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::evaluator::NoOpExternalProvider;
    use edifact_types::{RawSegment, SegmentPosition};

    fn make_segment<'a>(id: &'a str, elements: Vec<Vec<&'a str>>) -> RawSegment<'a> {
        RawSegment {
            id,
            elements,
            position: SegmentPosition {
                segment_number: 0,
                byte_offset: 0,
                message_number: 0,
            },
        }
    }

    #[test]
    fn test_find_segment() {
        let segments = vec![
            make_segment("UNH", vec![vec!["test"]]),
            make_segment("NAD", vec![vec!["MS"], vec!["123456789", "", "293"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        assert!(ctx.find_segment("NAD").is_some());
        assert!(ctx.find_segment("DTM").is_none());
    }

    #[test]
    fn test_find_segments_with_qualifier() {
        let segments = vec![
            make_segment("NAD", vec![vec!["MS"], vec!["111"]]),
            make_segment("NAD", vec![vec!["MR"], vec!["222"]]),
            make_segment("NAD", vec![vec!["MS"], vec!["333"]]),
        ];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        let ms_nads = ctx.find_segments_with_qualifier("NAD", 0, "MS");
        assert_eq!(ms_nads.len(), 2);
    }

    #[test]
    fn test_has_segment() {
        let segments = vec![make_segment("UNH", vec![vec!["test"]])];
        let external = NoOpExternalProvider;
        let ctx = EvaluationContext::new("11001", &external, &segments);

        assert!(ctx.has_segment("UNH"));
        assert!(!ctx.has_segment("NAD"));
    }
}
```

### Verification
```bash
cargo test -p automapper-validation eval::evaluator::tests
cargo test -p automapper-validation eval::context::tests
```

### Commit
```
feat(automapper-validation): define ConditionResult, EvaluationContext, and traits

Three-valued ConditionResult enum (True/False/Unknown), ConditionEvaluator
trait for individual condition evaluation, ExternalConditionProvider trait
for business context conditions, and EvaluationContext for segment access.
```

---

## Task 2: Implement ConditionExprEvaluator with three-valued logic

### Description
Implement the expression evaluator that walks a `ConditionExpr` AST and evaluates it using a `ConditionEvaluator`. The critical behavior is **short-circuit evaluation with Unknown propagation**:

- **AND**: If any operand is `False`, result is `False` (short-circuit). If all are `True`, result is `True`. Otherwise `Unknown`.
- **OR**: If any operand is `True`, result is `True` (short-circuit). If all are `False`, result is `False`. Otherwise `Unknown`.
- **XOR**: Both operands must be known (`True` or `False`) to produce a result. If either is `Unknown`, result is `Unknown`.
- **NOT**: `True` -> `False`, `False` -> `True`, `Unknown` -> `Unknown`.

This extends the C# `ConditionExpressionEvaluator` which only used `bool`, adding `Unknown` support for graceful degradation when external conditions are unavailable.

### Tests First, Then Implementation

**`crates/automapper-validation/src/eval/expr_eval.rs`**:
```rust
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
    use super::*;
    use super::super::evaluator::{NoOpExternalProvider, ConditionResult as CR};
    use edifact_types::{RawSegment, SegmentPosition};
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

        fn with_external(mut self, id: u32) -> Self {
            self.external_ids.push(id);
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
        let eval = MockEvaluator::new()
            .with_condition(2, CR::True);
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
        let eval = MockEvaluator::new()
            .with_condition(2, CR::True);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::Or(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::True);
    }

    #[test]
    fn test_and_short_circuits_past_unknown() {
        // [1](Unknown) ∧ [2](False) -> False (False short-circuits)
        let eval = MockEvaluator::new()
            .with_condition(2, CR::False);
        let (ext, segs) = empty_context();
        let ctx = make_ctx(&ext, &segs);
        let expr_eval = ConditionExprEvaluator::new(&eval);

        let expr = ConditionExpr::And(vec![ConditionExpr::Ref(1), ConditionExpr::Ref(2)]);
        assert_eq!(expr_eval.evaluate(&expr, &ctx), CR::False);
    }
}
```

### Verification
```bash
cargo test -p automapper-validation eval::expr_eval::tests
```

### Commit
```
feat(automapper-validation): implement ConditionExprEvaluator with three-valued logic

Short-circuit AND/OR evaluation with Unknown propagation. AND short-circuits
on False, OR short-circuits on True. XOR requires both operands known. NOT
inverts True/False and preserves Unknown. Comprehensive tests cover all
truth table combinations including Unknown.
```

---

## Task 3: Implement evaluator registry

### Description
Implement the `EvaluatorRegistry` that maps `(message_type, format_version)` pairs to `ConditionEvaluator` instances. This is the Rust equivalent of the C# `ConditionEvaluatorRegistry` (without the source generator, since Rust generated code is committed directly).

### Implementation

**`crates/automapper-validation/src/eval/registry.rs`**:
```rust
//! Registry of condition evaluators keyed by (message_type, format_version).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::evaluator::ConditionEvaluator;

/// Global registry of condition evaluators.
///
/// Evaluators are registered at startup (typically from generated code)
/// and looked up at runtime based on the detected message type and format
/// version.
///
/// Thread-safe: uses `RwLock` for concurrent read access with exclusive
/// write access during registration.
pub struct EvaluatorRegistry {
    evaluators: RwLock<HashMap<(String, String), Arc<dyn ConditionEvaluator>>>,
}

impl EvaluatorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            evaluators: RwLock::new(HashMap::new()),
        }
    }

    /// Register a condition evaluator for a message type and format version.
    ///
    /// Overwrites any previously registered evaluator for the same key.
    pub fn register<E: ConditionEvaluator + 'static>(&self, evaluator: E) {
        let key = (
            evaluator.message_type().to_string(),
            evaluator.format_version().to_string(),
        );
        self.evaluators
            .write()
            .expect("registry lock poisoned")
            .insert(key, Arc::new(evaluator));
    }

    /// Register an already-Arc'd evaluator.
    pub fn register_arc(&self, evaluator: Arc<dyn ConditionEvaluator>) {
        let key = (
            evaluator.message_type().to_string(),
            evaluator.format_version().to_string(),
        );
        self.evaluators
            .write()
            .expect("registry lock poisoned")
            .insert(key, evaluator);
    }

    /// Look up an evaluator by message type and format version.
    ///
    /// Returns `None` if no evaluator is registered for the given key.
    pub fn get(&self, message_type: &str, format_version: &str) -> Option<Arc<dyn ConditionEvaluator>> {
        self.evaluators
            .read()
            .expect("registry lock poisoned")
            .get(&(message_type.to_string(), format_version.to_string()))
            .cloned()
    }

    /// Look up an evaluator, returning an error if not found.
    pub fn get_or_err(
        &self,
        message_type: &str,
        format_version: &str,
    ) -> Result<Arc<dyn ConditionEvaluator>, crate::error::ValidationError> {
        self.get(message_type, format_version)
            .ok_or_else(|| crate::error::ValidationError::NoEvaluator {
                message_type: message_type.to_string(),
                format_version: format_version.to_string(),
            })
    }

    /// List all registered (message_type, format_version) keys.
    pub fn registered_keys(&self) -> Vec<(String, String)> {
        self.evaluators
            .read()
            .expect("registry lock poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Clear all registered evaluators. Primarily for testing.
    pub fn clear(&self) {
        self.evaluators
            .write()
            .expect("registry lock poisoned")
            .clear();
    }
}

impl Default for EvaluatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::context::EvaluationContext;
    use super::super::evaluator::{ConditionResult, NoOpExternalProvider};

    /// Minimal evaluator for testing registry operations.
    struct TestEvaluator {
        msg_type: String,
        fmt_version: String,
    }

    impl TestEvaluator {
        fn new(msg_type: &str, fmt_version: &str) -> Self {
            Self {
                msg_type: msg_type.to_string(),
                fmt_version: fmt_version.to_string(),
            }
        }
    }

    impl ConditionEvaluator for TestEvaluator {
        fn evaluate(&self, _condition: u32, _ctx: &EvaluationContext) -> ConditionResult {
            ConditionResult::Unknown
        }
        fn is_external(&self, _condition: u32) -> bool {
            false
        }
        fn message_type(&self) -> &str {
            &self.msg_type
        }
        fn format_version(&self) -> &str {
            &self.fmt_version
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));

        let eval = registry.get("UTILMD", "FV2510");
        assert!(eval.is_some());
        assert_eq!(eval.unwrap().message_type(), "UTILMD");
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let registry = EvaluatorRegistry::new();
        assert!(registry.get("UTILMD", "FV2510").is_none());
    }

    #[test]
    fn test_get_or_err_returns_error() {
        let registry = EvaluatorRegistry::new();
        let result = registry.get_or_err("UTILMD", "FV2510");
        assert!(result.is_err());
    }

    #[test]
    fn test_register_overwrites() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));

        let keys = registry.registered_keys();
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn test_multiple_registrations() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2504"));
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));
        registry.register(TestEvaluator::new("ORDERS", "FV2510"));

        let keys = registry.registered_keys();
        assert_eq!(keys.len(), 3);

        assert!(registry.get("UTILMD", "FV2504").is_some());
        assert!(registry.get("UTILMD", "FV2510").is_some());
        assert!(registry.get("ORDERS", "FV2510").is_some());
        assert!(registry.get("ORDERS", "FV2504").is_none());
    }

    #[test]
    fn test_clear() {
        let registry = EvaluatorRegistry::new();
        registry.register(TestEvaluator::new("UTILMD", "FV2510"));
        assert!(!registry.registered_keys().is_empty());

        registry.clear();
        assert!(registry.registered_keys().is_empty());
    }

    #[test]
    fn test_register_arc() {
        let registry = EvaluatorRegistry::new();
        let eval: Arc<dyn ConditionEvaluator> =
            Arc::new(TestEvaluator::new("UTILMD", "FV2510"));
        registry.register_arc(eval);

        assert!(registry.get("UTILMD", "FV2510").is_some());
    }
}
```

### Verification
```bash
cargo test -p automapper-validation eval::registry::tests
```

### Commit
```
feat(automapper-validation): implement thread-safe evaluator registry

EvaluatorRegistry maps (message_type, format_version) to ConditionEvaluator
instances using RwLock for concurrent read access. Supports registration,
lookup, error on missing, key listing, and clearing.
```

---

## Task 4: Integration tests for parser + evaluator pipeline

### Description
Create integration tests that exercise the complete pipeline: parse an AHB status string, then evaluate it using a mock evaluator. Tests use real-world AHB expressions from the UTILMD AHB.

### Implementation

**`crates/automapper-validation/tests/expr_eval_integration.rs`**:
```rust
//! Integration tests: parse AHB expressions and evaluate them.

use automapper_validation::eval::{
    ConditionExprEvaluator, ConditionResult, EvaluationContext,
};
use automapper_validation::eval::{ConditionEvaluator, ExternalConditionProvider};
use automapper_validation::expr::ConditionParser;
use edifact_types::{RawSegment, SegmentPosition};
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
```

### Verification
```bash
cargo test -p automapper-validation --test expr_eval_integration
```

### Commit
```
test(automapper-validation): add parser + evaluator integration tests

End-to-end tests parsing real AHB expressions and evaluating them with
mock condition evaluators. Covers ORDERS complex expressions, UTILMD XOR,
three-way AND chains, Unknown propagation, and bare status strings.
```
