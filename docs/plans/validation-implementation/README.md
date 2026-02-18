# Validation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement AHB (Anwendungshandbuch) condition expression parsing, evaluation, and EDIFACT message validation as the `automapper-validation` crate, porting the C# `Automapper.Validation` subsystem to Rust with three-valued logic (True/False/Unknown) for external conditions.

**Architecture:** A recursive-descent parser converts AHB status strings (e.g., `Muss [182] ∧ [6] ∧ [570]`) into a `ConditionExpr` AST. A `ConditionEvaluator` trait maps individual condition numbers to True/False/Unknown results. A `ConditionExprEvaluator` walks the AST using short-circuit boolean logic with Unknown propagation. The `EdifactValidator` orchestrates the full pipeline: parse EDIFACT input, look up AHB rules per Pruefidentifikator, evaluate condition expressions per field, and produce a `ValidationReport` with typed `ValidationIssue` entries.

**Tech Stack:** Rust 2021 edition, thiserror for errors, serde for serializable report types, dependencies on `edifact-types`, `edifact-parser`, `bo4e-extensions`, `automapper-core` from Feature 1

---

## Epic Overview

| Epic | Title | Description | Depends On |
|------|-------|-------------|------------|
| 1 | Condition Expression Parser | `ConditionExpr` AST enum, tokenizer for AHB status strings (Unicode operators, bracket references, parentheses), recursive descent parser with correct precedence (NOT > AND > OR > XOR), implicit AND for adjacent conditions, comprehensive unit tests | - |
| 2 | Condition Evaluator Traits & Registry | `ConditionResult` three-valued enum, `ConditionEvaluator` trait, `EvaluationContext`, `ExternalConditionProvider` trait, `ConditionExprEvaluator` that walks the AST with short-circuit logic and Unknown propagation, evaluator registry keyed by (message_type, format_version), unit tests with mock evaluators | Epic 1 |
| 3 | EdifactValidator & Integration | `Severity`, `ValidationCategory`, `ValidationIssue`, `ValidationReport`, `ValidationLevel` enum, error code constants, `EdifactValidator<E>` struct, `validate()` method that parses EDIFACT and evaluates conditions per field, integration tests using real AHB rules and EDIFACT fixtures | Epic 2 |

---

## Files in This Plan

1. [Epic 1: Condition Expression Parser](./epic-01-condition-expression-parser.md)
2. [Epic 2: Condition Evaluator Traits & Registry](./epic-02-condition-evaluator-traits.md)
3. [Epic 3: EdifactValidator & Integration](./epic-03-edifact-validator-integration.md)

---

## Test Strategy

- **Unit tests**: `#[cfg(test)]` modules in each source file, TDD red-green-refactor cycle
- **Integration tests**: `crates/automapper-validation/tests/` directory with EDIFACT fixtures
- **Parser edge cases**: Every expression pattern found in real AHB rules (implicit AND, nested parens, Unicode operators, package conditions, time conditions)
- **Three-valued logic**: Exhaustive tests for Unknown propagation through AND, OR, XOR, NOT
- **Mock evaluators**: Custom `ConditionEvaluator` implementations for testing expression evaluation without real condition logic
- **Real AHB rules**: Integration tests using actual Pruefidentifikator definitions from the UTILMD AHB

## Commands Reference

```bash
# Check the validation crate compiles
cargo check -p automapper-validation

# Run all validation tests
cargo test -p automapper-validation

# Run a specific test
cargo test -p automapper-validation test_parse_single_condition

# Run clippy lints
cargo clippy -p automapper-validation -- -D warnings

# Run formatter check
cargo fmt --all -- --check
```
