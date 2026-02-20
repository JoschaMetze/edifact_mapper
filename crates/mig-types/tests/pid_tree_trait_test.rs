use mig_types::traits::PidTree;

// This test verifies the trait exists and has the expected methods
#[test]
fn test_pid_tree_trait_exists() {
    // PidTree should be object-safe for dynamic dispatch
    fn _assert_object_safe(_: &dyn PidTree) {}
}
