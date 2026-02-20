/// Verify that the generated types compile and are accessible.
#[test]
fn test_generated_types_compile() {
    // This test passing means the generated code compiles
    use mig_types::generated::fv2504::utilmd::composites::*;
    use mig_types::generated::fv2504::utilmd::enums::*;
    use mig_types::generated::fv2504::utilmd::groups::*;
    use mig_types::generated::fv2504::utilmd::segments::*;

    // Verify a few types exist by constructing them
    let _ = std::mem::size_of::<D3035Qualifier>();
    let _ = std::mem::size_of::<CompositeC082>();
    let _ = std::mem::size_of::<SegNad>();
    let _ = std::mem::size_of::<Sg2>();
}

#[test]
fn test_enum_display_and_from_str() {
    use mig_types::generated::fv2504::utilmd::enums::D3035Qualifier;
    use std::str::FromStr;

    let q = D3035Qualifier::MS;
    assert_eq!(q.to_string(), "MS");

    let parsed = D3035Qualifier::from_str("MS").unwrap();
    assert_eq!(parsed, D3035Qualifier::MS);

    let unknown = D3035Qualifier::from_str("XX").unwrap();
    assert_eq!(unknown, D3035Qualifier::Unknown("XX".to_string()));
}

#[test]
fn test_enum_serde_roundtrip() {
    use mig_types::generated::fv2504::utilmd::enums::D3035Qualifier;

    let q = D3035Qualifier::MR;
    let json = serde_json::to_string(&q).unwrap();
    let parsed: D3035Qualifier = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, q);
}

#[test]
fn test_pid_types_instantiable() {
    // Verify a few PID types can be referenced and are real types
    let _ = std::mem::size_of::<
        mig_types::generated::fv2504::utilmd::pids::pid_55035::Pid55035,
    >();
}
