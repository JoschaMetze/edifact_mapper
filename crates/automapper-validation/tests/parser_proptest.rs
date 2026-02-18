//! Property-based tests for the condition expression parser.

use automapper_validation::expr::{ConditionExpr, ConditionParser};
use proptest::prelude::*;

/// Generate arbitrary strings that may or may not be valid condition expressions.
fn arbitrary_condition_input() -> impl Strategy<Value = String> {
    prop::string::string_regex(
        r"(Muss |Soll |Kann |X )?(\[[\d]{1,4}\]|\(|\)|[ ∧∨⊻]|AND |OR |XOR |NOT ){0,20}",
    )
    .unwrap()
}

proptest! {
    /// The parser must never panic on arbitrary input.
    #[test]
    fn parser_never_panics(input in "\\PC{0,200}") {
        // We don't care about the result, just that it doesn't panic
        let _ = ConditionParser::parse(&input);
    }

    /// The parser must never panic on semi-structured input.
    #[test]
    fn parser_never_panics_on_structured_input(input in arbitrary_condition_input()) {
        let _ = ConditionParser::parse(&input);
    }

    /// Parsing a Display'd expression should yield the same condition IDs.
    #[test]
    fn display_roundtrip_preserves_condition_ids(
        ids in prop::collection::vec(1u32..=2000, 1..=5)
    ) {
        // Build a simple AND expression from the IDs
        let expr = if ids.len() == 1 {
            ConditionExpr::Ref(ids[0])
        } else {
            ConditionExpr::And(ids.iter().map(|&id| ConditionExpr::Ref(id)).collect())
        };

        let displayed = format!("{expr}");
        if let Ok(Some(reparsed)) = ConditionParser::parse_raw(&displayed) {
            assert_eq!(
                expr.condition_ids(),
                reparsed.condition_ids(),
                "Condition IDs differ after roundtrip: '{displayed}'"
            );
        }
    }
}
