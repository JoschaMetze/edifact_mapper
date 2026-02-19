//! Condition evaluation traits and expression evaluator.

mod context;
mod evaluator;
mod expr_eval;
mod registry;

pub use context::EvaluationContext;
pub use evaluator::{
    ConditionEvaluator, ConditionResult, ExternalConditionProvider, NoOpExternalProvider,
};
pub use expr_eval::ConditionExprEvaluator;
pub use registry::EvaluatorRegistry;
