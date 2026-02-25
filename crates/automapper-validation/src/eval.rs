//! Condition evaluation traits and expression evaluator.

mod context;
mod evaluator;
mod expr_eval;
pub mod providers;
mod registry;

pub use context::{EvaluationContext, NoOpGroupNavigator};
pub use evaluator::{
    ConditionEvaluator, ConditionResult, ExternalConditionProvider, NoOpExternalProvider,
};
pub use expr_eval::ConditionExprEvaluator;
pub use mig_types::navigator::GroupNavigator;
pub use providers::{CompositeExternalProvider, MapExternalProvider};
pub use registry::EvaluatorRegistry;
