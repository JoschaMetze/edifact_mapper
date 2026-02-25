//! AHB condition expression parsing, evaluation, and EDIFACT message validation.
//!
//! This crate provides three layers of functionality:
//!
//! 1. **Expression parsing** ([`expr`]): Parses AHB status strings like
//!    `"Muss [182] ∧ [6] ∧ [570]"` into a [`ConditionExpr`] AST.
//!
//! 2. **Condition evaluation** ([`eval`]): Evaluates condition expressions
//!    using a [`ConditionEvaluator`] trait with three-valued logic
//!    (True/False/Unknown) for graceful handling of external conditions.
//!
//! 3. **Message validation** ([`validator`]): Validates EDIFACT messages
//!    against AHB rules, producing a [`ValidationReport`] with typed issues.
//!
//! # Quick Start
//!
//! ```ignore
//! use automapper_validation::expr::{ConditionParser, ConditionExpr};
//! use automapper_validation::eval::{ConditionExprEvaluator, ConditionResult};
//! use automapper_validation::validator::{EdifactValidator, ValidationLevel};
//!
//! // Parse a condition expression
//! let expr = ConditionParser::parse("Muss [182] ∧ [152]").unwrap();
//!
//! // Validate pre-parsed segments against AHB workflow
//! let validator = EdifactValidator::new(my_evaluator);
//! let report = validator.validate(&segments, &workflow, &external, ValidationLevel::Full);
//! ```

pub mod error;
pub mod eval;
pub mod expr;
pub mod generated;
pub mod validator;

// Re-export key types at crate root for convenience
pub use error::{ParseError, ValidationError};
pub use eval::{
    CompositeExternalProvider, ConditionEvaluator, ConditionExprEvaluator, ConditionResult,
    EvaluationContext, GroupNavigator, MapExternalProvider, NoOpGroupNavigator,
};
pub use expr::{ConditionExpr, ConditionParser};
pub use validator::{
    EdifactValidator, ErrorCodes, Severity, ValidationCategory, ValidationIssue, ValidationLevel,
    ValidationReport,
};

// Re-export AHB workflow types for callers
pub use validator::validate::{AhbCodeRule, AhbFieldRule, AhbWorkflow};

// Re-export generated evaluators
pub use generated::UtilmdConditionEvaluatorFV2504;
