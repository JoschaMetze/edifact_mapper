//! AHB condition expression parsing, evaluation, and EDIFACT message validation.
//!
//! This crate provides:
//! - A parser for AHB condition expressions (`[1] ∧ [2]`, `[3] ∨ [4]`, etc.)
//! - Traits for condition evaluation with three-valued logic (True/False/Unknown)
//! - An EDIFACT message validator that checks messages against AHB rules

pub mod error;
pub mod eval;
pub mod expr;
pub mod validator;
