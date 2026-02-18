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
