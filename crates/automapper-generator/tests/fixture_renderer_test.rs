//! Tests for the fixture renderer types (BO4E canonical format).

use automapper_generator::fixture_renderer::{
    CanonicalBo4e, CanonicalMeta, NachrichtBo4e, TransaktionBo4e,
};

#[test]
fn test_canonical_bo4e_roundtrip_serialization() {
    let canonical = CanonicalBo4e {
        meta: CanonicalMeta {
            pid: "55001".into(),
            message_type: "UTILMD".into(),
            source_format_version: "FV2504".into(),
            source_fixture: "55001_UTILMD_S2.1_ALEXANDE121980.edi".into(),
        },
        nachrichtendaten: serde_json::json!({
            "absender": "9978842000002",
            "empfaenger": "9900269000000",
            "erstellungsdatum": "250331:1329",
            "referenznummer": "ALEXANDE121980"
        }),
        nachricht: NachrichtBo4e {
            unh_referenz: "ALEXANDE951842".into(),
            nachrichten_typ: "UTILMD:D:11A:UN:S2.1".into(),
            stammdaten: serde_json::json!({
                "Marktteilnehmer": [
                    {"marktrolle": "MS", "rollencodenummer": "9978842000002"},
                    {"marktrolle": "MR", "rollencodenummer": "9900269000000"}
                ]
            }),
            transaktionen: vec![TransaktionBo4e {
                stammdaten: serde_json::json!({
                    "Marktlokation": [{"malo_id": "12345678900"}]
                }),
                transaktionsdaten: serde_json::json!({
                    "Prozessdaten": [{"pruefidentifikator": "55001"}]
                }),
            }],
        },
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&canonical).unwrap();
    assert!(json.contains("55001"));
    assert!(json.contains("ALEXANDE121980"));

    // Deserialize back
    let parsed: CanonicalBo4e = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.meta.pid, "55001");
    assert_eq!(parsed.nachricht.transaktionen.len(), 1);
}

#[test]
fn test_canonical_bo4e_structure_from_json() {
    let canonical_json = r#"{
        "meta": {
            "pid": "55001",
            "message_type": "UTILMD",
            "source_format_version": "FV2504",
            "source_fixture": "55001_UTILMD_S2.1_test.edi"
        },
        "nachrichtendaten": {
            "absender": "9978842000002",
            "empfaenger": "9900269000000"
        },
        "nachricht": {
            "unh_referenz": "MSG001",
            "nachrichten_typ": "UTILMD:D:11A:UN:S2.1",
            "stammdaten": {
                "Marktteilnehmer": [
                    {"marktrolle": "MS"}
                ]
            },
            "transaktionen": [
                {
                    "stammdaten": {
                        "Marktlokation": [{"malo_id": "12345678900"}]
                    },
                    "transaktionsdaten": {
                        "Prozessdaten": [{"pruefidentifikator": "55001"}]
                    }
                }
            ]
        }
    }"#;

    let canonical: CanonicalBo4e = serde_json::from_str(canonical_json).unwrap();
    assert_eq!(canonical.meta.pid, "55001");
    assert_eq!(canonical.meta.message_type, "UTILMD");
    assert_eq!(canonical.nachricht.unh_referenz, "MSG001");
    assert_eq!(canonical.nachricht.transaktionen.len(), 1);
    assert_eq!(
        canonical.nachricht.transaktionen[0].transaktionsdaten["Prozessdaten"][0]
            ["pruefidentifikator"]
            .as_str()
            .unwrap(),
        "55001"
    );
}
