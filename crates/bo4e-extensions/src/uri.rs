use serde::{Deserialize, Serialize};

/// A URI identifying a BO4E business object.
///
/// Format: `bo4e://TypeName/Identifier`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bo4eUri(String);

impl Bo4eUri {
    /// Creates a new BO4E URI.
    pub fn new(type_name: &str, id: &str) -> Self {
        Self(format!("bo4e://{}/{}", type_name, id))
    }

    /// Extracts the type name from the URI.
    pub fn type_name(&self) -> &str {
        let after_scheme = &self.0["bo4e://".len()..];
        after_scheme.split('/').next().unwrap_or("")
    }

    /// Extracts the identifier from the URI.
    pub fn id(&self) -> &str {
        let after_scheme = &self.0["bo4e://".len()..];
        after_scheme.split('/').nth(1).unwrap_or("")
    }

    /// Attempts to parse a string as a Bo4eUri.
    pub fn parse(s: &str) -> Option<Self> {
        if !s.starts_with("bo4e://") {
            return None;
        }
        let path = &s["bo4e://".len()..];
        let slash = path.find('/')?;
        if slash == 0 || slash == path.len() - 1 {
            return None;
        }
        Some(Self(s.to_string()))
    }

    /// Returns the full URI string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Bo4eUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bo4e_uri_new() {
        let uri = Bo4eUri::new("Marktlokation", "DE001");
        assert_eq!(uri.to_string(), "bo4e://Marktlokation/DE001");
        assert_eq!(uri.type_name(), "Marktlokation");
        assert_eq!(uri.id(), "DE001");
    }

    #[test]
    fn test_bo4e_uri_parse() {
        let uri = Bo4eUri::parse("bo4e://Zaehler/Z001").unwrap();
        assert_eq!(uri.type_name(), "Zaehler");
        assert_eq!(uri.id(), "Z001");
    }

    #[test]
    fn test_bo4e_uri_parse_invalid() {
        assert!(Bo4eUri::parse("http://example.com").is_none());
        assert!(Bo4eUri::parse("bo4e://").is_none());
        assert!(Bo4eUri::parse("bo4e:///id").is_none());
        assert!(Bo4eUri::parse("bo4e://Type/").is_none());
    }

    #[test]
    fn test_bo4e_uri_equality() {
        let a = Bo4eUri::new("Marktlokation", "DE001");
        let b = Bo4eUri::new("Marktlokation", "DE001");
        let c = Bo4eUri::new("Marktlokation", "DE002");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_bo4e_uri_serde() {
        let uri = Bo4eUri::new("Geschaeftspartner", "GP001");
        let json = serde_json::to_string(&uri).unwrap();
        let de: Bo4eUri = serde_json::from_str(&json).unwrap();
        assert_eq!(uri, de);
    }
}
