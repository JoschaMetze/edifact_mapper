//! Integration tests for the trait hierarchy and version dispatch mechanism.

use automapper_core::{
    create_coordinator, detect_format_version, AutomapperError, Builder, FormatVersion,
    SegmentHandler, TransactionContext, VersionConfig, FV2504, FV2510,
};
use edifact_types::{RawSegment, SegmentPosition};

/// Verify that VersionConfig is correctly parameterized.
#[test]
fn test_version_config_dispatch() {
    fn version_string<V: VersionConfig>() -> &'static str {
        V::VERSION.as_str()
    }

    assert_eq!(version_string::<FV2504>(), "FV2504");
    assert_eq!(version_string::<FV2510>(), "FV2510");
}

/// Verify the full pipeline: detect version -> create coordinator -> parse.
#[test]
fn test_full_pipeline_stub() {
    let input = b"UNA:+.? 'UNB+UNOC:3+9900123:500+9900456:500+251217:1229+REF'UNH+MSG+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'UNT+3+MSG'UNZ+1+REF'";

    // Step 1: Detect version
    let fv = detect_format_version(input).expect("should detect FV2504");
    assert_eq!(fv, FormatVersion::FV2504);

    // Step 2: Create coordinator
    let mut coord = create_coordinator(fv).expect("should create coordinator");
    assert_eq!(coord.format_version(), FormatVersion::FV2504);

    // Step 3: Parse (stub returns empty for now)
    let result = coord.parse(input).expect("should parse without error");
    assert!(result.is_empty(), "stub coordinator returns empty");
}

/// Verify that FormatVersion round-trips through string conversion.
#[test]
fn test_format_version_string_roundtrip() {
    for fv in [FormatVersion::FV2504, FormatVersion::FV2510] {
        let s = fv.as_str();
        let parsed: FormatVersion = s.parse().expect("should parse back");
        assert_eq!(fv, parsed);
    }
}

/// Verify TransactionContext reset behavior preserves format version.
#[test]
fn test_context_reset_preserves_format_version() {
    let mut ctx = TransactionContext::new("FV2504");
    ctx.set_message_reference("MSG001");
    ctx.set_sender_mp_id("9900123");

    ctx.reset();

    assert_eq!(ctx.format_version, "FV2504");
    assert_eq!(ctx.sender_mp_id, Some("9900123".to_string()));
    assert!(ctx.message_reference.is_none());
}

/// Verify that a custom SegmentHandler + Builder work together.
#[test]
fn test_handler_and_builder_integration() {
    struct LocHandler {
        location_ids: Vec<String>,
    }

    impl SegmentHandler for LocHandler {
        fn can_handle(&self, segment: &RawSegment) -> bool {
            segment.id == "LOC"
        }

        fn handle(&mut self, segment: &RawSegment, _ctx: &mut TransactionContext) {
            let qualifier = segment.get_element(0);
            if qualifier == "Z16" {
                let id = segment.get_component(1, 0);
                if !id.is_empty() {
                    self.location_ids.push(id.to_string());
                }
            }
        }
    }

    struct LocBuilder {
        ids: Vec<String>,
    }

    impl Builder<Vec<String>> for LocBuilder {
        fn is_empty(&self) -> bool {
            self.ids.is_empty()
        }

        fn build(&mut self) -> Vec<String> {
            std::mem::take(&mut self.ids)
        }

        fn reset(&mut self) {
            self.ids.clear();
        }
    }

    // Simulate parsing
    let mut handler = LocHandler {
        location_ids: Vec::new(),
    };
    let mut ctx = TransactionContext::new("FV2504");

    let pos = SegmentPosition::new(1, 0, 1);
    let loc1 = RawSegment::new(
        "LOC",
        vec![vec!["Z16"], vec!["DE00014545768S0000000000000003054"]],
        pos,
    );
    let loc2 = RawSegment::new(
        "LOC",
        vec![vec!["Z17"], vec!["DE00098765432100000000000000012"]],
        pos,
    );
    let loc3 = RawSegment::new(
        "LOC",
        vec![vec!["Z16"], vec!["DE00099887766500000000000000034"]],
        pos,
    );

    for seg in [&loc1, &loc2, &loc3] {
        if handler.can_handle(seg) {
            handler.handle(seg, &mut ctx);
        }
    }

    // Only LOC+Z16 segments should be handled
    assert_eq!(handler.location_ids.len(), 2);
    assert_eq!(handler.location_ids[0], "DE00014545768S0000000000000003054");
    assert_eq!(handler.location_ids[1], "DE00099887766500000000000000034");

    // Build
    let mut builder = LocBuilder {
        ids: handler.location_ids,
    };
    assert!(!builder.is_empty());

    let result = builder.build();
    assert_eq!(result.len(), 2);
    assert!(builder.is_empty());
}

/// Verify AutomapperError variants can be created from ParseError.
#[test]
fn test_automapper_error_from_parse_error() {
    use edifact_parser::ParseError;

    let parse_err = ParseError::UnexpectedEof;
    let auto_err: AutomapperError = parse_err.into();

    match auto_err {
        AutomapperError::Parse(_) => {} // expected
        other => panic!("expected Parse variant, got: {}", other),
    }
}
