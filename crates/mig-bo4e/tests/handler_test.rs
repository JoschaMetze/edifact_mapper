use mig_assembly::assembler::{AssembledGroupInstance, AssembledSegment};
use mig_bo4e::handlers::HandlerRegistry;

#[test]
fn test_register_and_invoke_handler() {
    let mut registry = HandlerRegistry::new();

    registry.register("test_handler", |_instance| {
        Ok(serde_json::json!({"result": "handled"}))
    });

    assert!(registry.has_handler("test_handler"));
    assert!(!registry.has_handler("nonexistent"));

    let instance = AssembledGroupInstance {
        segments: vec![],
        child_groups: vec![],
    };

    let result = registry.invoke("test_handler", &instance);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["result"], "handled");
}

#[test]
fn test_invoke_nonexistent_handler() {
    let registry = HandlerRegistry::new();

    let instance = AssembledGroupInstance {
        segments: vec![],
        child_groups: vec![],
    };

    let result = registry.invoke("missing", &instance);
    assert!(result.is_err());
}

#[test]
fn test_handler_receives_instance_data() {
    let mut registry = HandlerRegistry::new();

    registry.register("extract_tag", |instance| {
        let tag = instance
            .segments
            .first()
            .map(|s| s.tag.clone())
            .unwrap_or_default();
        Ok(serde_json::json!({ "tag": tag }))
    });

    let instance = AssembledGroupInstance {
        segments: vec![AssembledSegment {
            tag: "LOC".to_string(),
            elements: vec![vec!["Z16".to_string()]],
        }],
        child_groups: vec![],
    };

    let result = registry.invoke("extract_tag", &instance).unwrap();
    assert_eq!(result["tag"], "LOC");
}

#[test]
fn test_registry_default() {
    let registry = HandlerRegistry::default();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_multiple_handlers() {
    let mut registry = HandlerRegistry::new();

    registry.register("handler_a", |_| Ok(serde_json::json!("a")));
    registry.register("handler_b", |_| Ok(serde_json::json!("b")));
    registry.register("handler_c", |_| Ok(serde_json::json!("c")));

    assert_eq!(registry.len(), 3);
    assert!(!registry.is_empty());
    assert!(registry.has_handler("handler_a"));
    assert!(registry.has_handler("handler_b"));
    assert!(registry.has_handler("handler_c"));
}
