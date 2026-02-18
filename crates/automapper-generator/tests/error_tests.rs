use automapper_generator::GeneratorError;
use std::path::PathBuf;

#[test]
fn test_xml_parse_error_display() {
    let err = GeneratorError::XmlParse {
        path: PathBuf::from("test.xml"),
        message: "unexpected EOF".to_string(),
        source: None,
    };
    assert_eq!(
        err.to_string(),
        "XML parsing error in test.xml: unexpected EOF"
    );
}

#[test]
fn test_missing_attribute_error_display() {
    let err = GeneratorError::MissingAttribute {
        path: PathBuf::from("mig.xml"),
        element: "S_UNH".to_string(),
        attribute: "Versionsnummer".to_string(),
    };
    assert!(err
        .to_string()
        .contains("missing required attribute 'Versionsnummer'"));
}

#[test]
fn test_file_not_found_error_display() {
    let err = GeneratorError::FileNotFound(PathBuf::from("/nonexistent/file.xml"));
    assert!(err.to_string().contains("/nonexistent/file.xml"));
}

#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let gen_err: GeneratorError = io_err.into();
    assert!(gen_err.to_string().contains("file not found"));
}
